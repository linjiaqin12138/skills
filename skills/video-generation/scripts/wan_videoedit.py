#!/usr/bin/env python3
import argparse
import json
import time
import urllib.error
import urllib.request
from pathlib import Path

from wan_animate import BASE_URL, request, token, upload_file


MODEL = "wan2.7-videoedit"


def parse_args():
    parser = argparse.ArgumentParser(description="阿里云百炼 Wan 2.7 视频编辑")
    parser.add_argument("--video", required=True)
    parser.add_argument("--reference-image", action="append", default=[])
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--resolution", default="720P", choices=("720P", "1080P"))
    parser.add_argument("--out", required=True)
    parser.add_argument("--poll-interval", type=int, default=15)
    return parser.parse_args()


def main():
    args = parse_args()
    if len(args.reference_image) > 4:
        raise RuntimeError("参考图最多 4 张")
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
        media = [{"type": "video", "url": upload_file(args.video, api_key, MODEL)}]
        for path in args.reference_image:
            media.append({"type": "reference_image", "url": upload_file(path, api_key, MODEL)})
        result = request(
            f"{BASE_URL}/services/aigc/video-generation/video-synthesis",
            api_key,
            "POST",
            {
                "model": MODEL,
                "input": {"prompt": args.prompt, "media": media},
                "parameters": {
                    "resolution": args.resolution,
                    "ratio": "16:9",
                    "audio_setting": "origin",
                    "prompt_extend": False,
                    "watermark": False,
                },
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
            with urllib.request.urlopen(task["video_url"], timeout=300) as response:
                output.write_bytes(response.read())
            print(output.resolve(), flush=True)
            print(json.dumps({"usage": result.get("usage", {})}, ensure_ascii=False), flush=True)
            return 0
        if status in ("FAILED", "CANCELED", "UNKNOWN"):
            raise RuntimeError(f"生成失败：{task.get('code')}: {task.get('message')}")
        time.sleep(args.poll_interval)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (RuntimeError, KeyError, urllib.error.URLError) as error:
        raise SystemExit(str(error))
