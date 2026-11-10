#!/usr/bin/env python3
import argparse
import json
import mimetypes
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path


BASE_URL = "https://dashscope.aliyuncs.com/api/v1"
MODEL = "wan2.2-animate-mix"


def parse_args():
    parser = argparse.ArgumentParser(description="阿里云百炼 Wan Animate 专用视频换人")
    parser.add_argument("--image", required=True)
    parser.add_argument("--video", required=True)
    parser.add_argument("--mode", default="wan-std", choices=("wan-std", "wan-pro"))
    parser.add_argument("--out", required=True)
    parser.add_argument("--poll-interval", type=int, default=15)
    return parser.parse_args()


def token():
    path = Path(__file__).resolve().parent.parent / "config.json"
    return json.loads(path.read_text(encoding="utf-8"))["providers"]["dashscope"]["token"].strip()


def request(url, api_key, method="GET", payload=None, headers=None):
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    req = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            **(headers or {}),
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=120) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        try:
            detail = json.loads(error.read())
        except ValueError:
            detail = {}
        raise RuntimeError(f"HTTP {error.code}: {detail.get('code')}: {detail.get('message')}") from error


def upload_file(path_value, api_key, model=MODEL):
    path = Path(path_value)
    policy = request(f"{BASE_URL}/uploads?action=getPolicy&model={model}", api_key)["data"]
    key = f"{policy['upload_dir']}/{path.name}"
    boundary = f"----kimi-{uuid.uuid4().hex}"
    fields = {
        "OSSAccessKeyId": policy["oss_access_key_id"],
        "Signature": policy["signature"],
        "policy": policy["policy"],
        "x-oss-object-acl": policy["x_oss_object_acl"],
        "x-oss-forbid-overwrite": policy["x_oss_forbid_overwrite"],
        "key": key,
        "success_action_status": "200",
    }
    body = bytearray()
    for name, value in fields.items():
        body.extend(f"--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n".encode())
    mime = mimetypes.guess_type(path.name)[0] or "application/octet-stream"
    body.extend(f"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{path.name}\"\r\nContent-Type: {mime}\r\n\r\n".encode())
    body.extend(path.read_bytes())
    body.extend(f"\r\n--{boundary}--\r\n".encode())
    req = urllib.request.Request(
        policy["upload_host"],
        data=bytes(body),
        method="POST",
        headers={"Content-Type": f"multipart/form-data; boundary={boundary}"},
    )
    with urllib.request.urlopen(req, timeout=300) as response:
        if response.status != 200:
            raise RuntimeError(f"上传失败：HTTP {response.status}")
    print(f"已上传：{path.name}", flush=True)
    return f"oss://{key}"


def main():
    args = parse_args()
    output = Path(args.out)
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.is_file() and output.stat().st_size > 0:
        print(output.resolve())
        return 0
    task_file = output.with_suffix(output.suffix + ".task.json")
    api_key = token()
    if task_file.is_file():
        task_id = json.loads(task_file.read_text(encoding="utf-8"))["task_id"]
        print(f"恢复任务：{task_id}", flush=True)
    else:
        image_url = upload_file(args.image, api_key)
        video_url = upload_file(args.video, api_key)
        result = request(
            f"{BASE_URL}/services/aigc/image2video/video-synthesis",
            api_key,
            "POST",
            {
                "model": MODEL,
                "input": {"image_url": image_url, "video_url": video_url, "watermark": False},
                "parameters": {"mode": args.mode},
            },
            {"X-DashScope-Async": "enable", "X-DashScope-OssResourceResolve": "enable"},
        )
        task_id = result["output"]["task_id"]
        task_file.write_text(json.dumps({"task_id": task_id}, indent=2), encoding="utf-8")
        print(f"任务已创建：{task_id}", flush=True)
    while True:
        result = request(f"{BASE_URL}/tasks/{task_id}", api_key)
        task = result["output"]
        status = task["task_status"]
        print(f"状态：{status}", flush=True)
        if status == "SUCCEEDED":
            url = task["results"]["video_url"]
            with urllib.request.urlopen(url, timeout=300) as response:
                output.write_bytes(response.read())
            print(output.resolve(), flush=True)
            print(json.dumps({"usage": task.get("usage", {})}, ensure_ascii=False), flush=True)
            return 0
        if status in ("FAILED", "CANCELED", "UNKNOWN"):
            raise RuntimeError(f"生成失败：{task.get('code')}: {task.get('message')}")
        time.sleep(args.poll_interval)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (RuntimeError, KeyError, urllib.error.URLError) as error:
        raise SystemExit(str(error))
