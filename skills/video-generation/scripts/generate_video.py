#!/usr/bin/env python3
import argparse
import base64
import json
import mimetypes
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


def parse_args():
    parser = argparse.ArgumentParser(description="MiniMax H3 / Ofox 视频生成：创建、轮询并下载")
    parser.add_argument("--provider", default="minimax-plan", choices=("minimax-plan", "ofoxai"))
    parser.add_argument("--first-frame", help="图生视频首帧；与 --reference-image 互斥")
    parser.add_argument("--reference-image", action="append", default=[], help="多模态参考图，可重复，最多 9 张")
    parser.add_argument("--ratio", default="adaptive", choices=("adaptive", "21:9", "16:9", "4:3", "1:1", "3:4", "9:16"))
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--model", default="MiniMax-H3-Lite", choices=("MiniMax-H3", "MiniMax-H3-Highspeed", "MiniMax-H3-Lite", "MiniMax-H3-Max", "bytedance/seedance-2.0", "bytedance/seedance-2.0-fast", "bytedance/seedance-2.0-mini", "bytedance/seedance-2.5", "alibaba/wan-3.0"))
    parser.add_argument("--resolution", default="480P", choices=("480P", "768P", "2K", "480p", "720p", "1080p", "4k"))
    parser.add_argument("--duration", type=int, default=4)
    parser.add_argument("--no-audio", action="store_true", help="仅 Ofox：关闭同步音轨")
    parser.add_argument("--real-person", action="store_true", help="Ofox Seedance：参考图含真人时启用预处理")
    parser.add_argument("--provider-type", help="Ofox 上游通道，例如 byteplus 或 volcengine")
    parser.add_argument("--poll-interval", type=int, default=10)
    parser.add_argument("--dry-run", action="store_true")
    return parser.parse_args()


def load_token(provider):
    path = Path(__file__).resolve().parent.parent / "config.json"
    try:
        return json.loads(path.read_text(encoding="utf-8"))["providers"][provider]["token"].strip()
    except (FileNotFoundError, json.JSONDecodeError, KeyError, AttributeError) as error:
        raise ValueError(f"无法从 {path} 读取 providers.{provider}.token") from error


def api_request(url, token, method="GET", payload=None):
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    request = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        try:
            detail = json.loads(error.read())
            message = detail.get("base_resp", {}).get("status_msg") or detail.get("error", detail)
        except ValueError:
            message = "无法解析错误响应"
        raise RuntimeError(f"MiniMax HTTP {error.code}: {message}") from error


def image_data_url(path_value):
    path = Path(path_value)
    if not path.is_file():
        raise ValueError(f"首帧不存在：{path_value}")
    mime = mimetypes.guess_type(path.name)[0]
    if mime not in ("image/jpeg", "image/png", "image/webp"):
        raise ValueError("首帧仅支持 JPG、PNG、WEBP")
    return f"data:{mime};base64,{base64.b64encode(path.read_bytes()).decode('ascii')}"


def main():
    args = parse_args()
    output = Path(args.out)
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.is_file() and output.stat().st_size > 0:
        print(output.resolve())
        return 0
    task_file = output.with_suffix(output.suffix + ".task.json")
    try:
        token = load_token(args.provider)
        if args.first_frame and args.reference_image:
            raise ValueError("--first-frame 与 --reference-image 互斥")
        if len(args.reference_image) > 9:
            raise ValueError("参考图最多 9 张")
        if args.provider == "ofoxai" and not args.model.startswith(("bytedance/", "alibaba/")):
            raise ValueError("Ofox 后端当前支持 Seedance 和 Wan 系列")
        if args.provider == "minimax-plan" and args.model.startswith(("bytedance/", "alibaba/")):
            raise ValueError("Seedance 和 Wan 请使用 --provider ofoxai")
        if args.provider == "ofoxai" and args.first_frame:
            raise ValueError("Ofox Seedance 当前请使用 --reference-image")
        if args.dry_run:
            for path in ([args.first_frame] if args.first_frame else args.reference_image):
                if not Path(path).is_file():
                    raise ValueError(f"图片不存在：{path}")
            print(json.dumps({"provider": args.provider, "model": args.model, "resolution": args.resolution, "duration": args.duration, "reference_count": len(args.reference_image)}, ensure_ascii=False))
            return 0
        if task_file.is_file():
            task_id = json.loads(task_file.read_text(encoding="utf-8"))["task_id"]
            print(f"恢复任务：{task_id}", flush=True)
        elif args.provider == "minimax-plan":
            content = [{"type": "text", "text": args.prompt}]
            if args.first_frame:
                content.append({"type": "image_url", "image_url": {"url": image_data_url(args.first_frame)}, "role": "first_frame"})
            for path in args.reference_image:
                content.append({"type": "image_url", "image_url": {"url": image_data_url(path)}, "role": "reference_image"})
            payload = {
                "model": args.model,
                "content": content,
                "resolution": args.resolution,
                "duration": args.duration,
                "ratio": args.ratio,
                "aigc_watermark": False,
            }
            result = api_request("https://api.minimax.cn/v2/video_generation", token, "POST", payload)
            task_id = result.get("task_id")
            if not task_id:
                raise RuntimeError(f"未返回 task_id：{result}")
            task_file.write_text(json.dumps({"task_id": task_id}, indent=2), encoding="utf-8")
            print(f"任务已创建：{task_id}", flush=True)
        else:
            payload = {
                "model": args.model,
                "prompt": args.prompt,
                "duration": args.duration,
                "resolution": args.resolution,
                "aspect_ratio": args.ratio,
                "generate_audio": not args.no_audio,
                "input_references": [
                    {"type": "image_url", "image_url": {"url": image_data_url(path)}}
                    for path in args.reference_image
                ],
            }
            if args.real_person and args.model.startswith("bytedance/"):
                payload["real_person"] = True
            if args.provider_type:
                payload["provider"] = {"type": args.provider_type}
            result = api_request("https://api.ofox.ai/v1/videos", token, "POST", payload)
            task_id = result.get("id")
            if not task_id:
                raise RuntimeError(f"未返回视频任务 ID：{result}")
            task_file.write_text(json.dumps({"task_id": task_id, "provider": "ofoxai"}, indent=2), encoding="utf-8")
            print(f"任务已创建：{task_id}", flush=True)
        while True:
            if args.provider == "minimax-plan":
                result = api_request(f"https://api.minimax.cn/v2/query/video_generation/{task_id}", token)
                task = result.get("task", {})
                status = task.get("status")
                url = task.get("content", {}).get("url")
            else:
                task = api_request(f"https://api.ofox.ai/v1/videos/{task_id}", token)
                status = task.get("status")
                urls = task.get("mirror_urls") or task.get("unsigned_urls") or []
                url = urls[0] if urls else None
            print(f"状态：{status}", flush=True)
            if status in ("succeeded", "completed"):
                if not url:
                    raise RuntimeError(f"成功响应没有视频 URL：{task}")
                with urllib.request.urlopen(url, timeout=300) as response:
                    output.write_bytes(response.read())
                print(output.resolve(), flush=True)
                print(json.dumps({"usage": task.get("usage", {}), "model": task.get("model"), "resolution": task.get("resolution")}, ensure_ascii=False), flush=True)
                return 0
            if status in ("failed", "cancelled", "expired"):
                raise RuntimeError(f"视频生成失败：{task.get('error') or task}")
            time.sleep(args.poll_interval)
    except (ValueError, RuntimeError, urllib.error.URLError, TimeoutError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
