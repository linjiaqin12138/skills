#!/usr/bin/env python3
import argparse
import base64
import json
import mimetypes
import sys
import urllib.error
import urllib.request
from pathlib import Path

MODELS = {
    "image-01": ("minimax-plan", "image-01"),
    "image-01-live": ("minimax-plan", "image-01-live"),
    "nano-banana-2": ("ofoxai", "google/gemini-3.1-flash-image"),
    "nano-banana-2.1": ("ofoxai", "google/gemini-nano-banana-2.1"),
    "nano-banana-pro": ("ofoxai", "google/gemini-3-pro-image"),
    "seedream-5-flash": ("ofoxai", "volcengine/doubao-seedream-5.0-flash"),
    "seedream-5-pro": ("ofoxai", "volcengine/doubao-seedream-5.0-pro"),
}
DEFAULT_RESOLUTIONS = {
    "nano-banana-2": "512",
    "nano-banana-2.1": "1K",
    "nano-banana-pro": "1K",
    "seedream-5-flash": "1024x1024",
    "seedream-5-pro": "1024x1024",
}


def parse_args():
    parser = argparse.ArgumentParser(description="统一的 MiniMax / OfoxAI 图像生成与参考图编辑入口")
    parser.add_argument("--provider", choices=("minimax-plan", "ofoxai"), help="通常无需指定，由模型自动推断")
    parser.add_argument("--model", choices=MODELS, required=True)
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--reference", action="append", default=[], help="可重复传入；MiniMax/Seedream 限 1 张，Gemini 最多 14 张")
    parser.add_argument("--aspect-ratio", default="1:1")
    parser.add_argument("--resolution", help="Ofox Gemini: 512/1K/2K/4K；Seedream: 如 1024x1024")
    parser.add_argument("--n", type=int, default=1, help="MiniMax 支持 1–9；Ofox 每次一张")
    parser.add_argument("--seed", type=int, help="仅 MiniMax")
    parser.add_argument("--width", type=int, help="仅 MiniMax image-01")
    parser.add_argument("--height", type=int, help="仅 MiniMax image-01")
    parser.add_argument("--out", help="单张图的输出路径")
    parser.add_argument("--out-dir", help="MiniMax 批量输出目录")
    parser.add_argument("--out-prefix", default="image")
    parser.add_argument("--region", choices=("cn", "global"), default="cn", help="仅 MiniMax")
    parser.add_argument("--prompt-optimizer", action="store_true", help="仅 MiniMax")
    parser.add_argument("--aigc-watermark", action="store_true", help="仅 MiniMax")
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    inferred_provider = MODELS[args.model][0]
    if args.provider and args.provider != inferred_provider:
        parser.error(f"模型 {args.model} 属于 {inferred_provider}，不是 {args.provider}")
    args.provider = inferred_provider
    if args.model.startswith("nano-banana") and len(args.reference) > 14:
        parser.error("Nano Banana 最多支持 14 张参考图")
    if not args.model.startswith("nano-banana") and len(args.reference) > 1:
        parser.error("MiniMax 和 Seedream 当前仅支持 1 张参考图")
    if not 1 <= args.n <= 9:
        parser.error("--n 必须在 1–9 之间")
    if len(args.prompt) > 1500 and args.provider == "minimax-plan":
        parser.error("MiniMax prompt 不能超过 1500 字符")
    if args.provider == "ofoxai" and args.n != 1:
        parser.error("Ofox 每次请求生成一张；多个候选请分别调用")
    if args.provider == "ofoxai" and not args.out:
        parser.error("Ofox 生成必须指定 --out")
    if args.out and args.n != 1:
        parser.error("--out 仅支持单张图；MiniMax 批量生成请用 --out-dir")
    if not args.out and not args.out_dir:
        parser.error("必须指定 --out 或 --out-dir")
    if (args.width is None) != (args.height is None):
        parser.error("--width 和 --height 必须同时指定")
    if args.width is not None:
        for value in (args.width, args.height):
            if not 512 <= value <= 2048 or value % 8:
                parser.error("宽高必须在 512–2048 之间且为 8 的倍数")
    return args


def load_token(provider):
    path = Path(__file__).resolve().parent.parent / "config.json"
    try:
        token = json.loads(path.read_text(encoding="utf-8"))["providers"][provider]["token"].strip()
    except (FileNotFoundError, json.JSONDecodeError, KeyError, AttributeError):
        token = ""
    if not token:
        raise ValueError(f"请在 {path} 的 providers.{provider}.token 填入 Key")
    return token


def load_reference(value):
    if not value:
        return None
    if value.startswith(("http://", "https://")):
        return "url", value
    path = Path(value)
    if not path.is_file():
        raise ValueError(f"参考图不存在：{value}")
    if path.stat().st_size >= 10 * 1024 * 1024:
        raise ValueError("参考图必须小于 10MB")
    mime = mimetypes.guess_type(path.name)[0]
    if mime not in ("image/jpeg", "image/png"):
        raise ValueError("参考图仅支持 JPG、JPEG、PNG")
    return mime, base64.b64encode(path.read_bytes()).decode("ascii")


def request_json(url, token, payload, protocol, timeout):
    if protocol == "gemini":
        auth = {"x-goog-api-key": token}
    else:
        auth = {"Authorization": f"Bearer {token}"}
    request = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={**auth, "Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        try:
            detail = json.loads(error.read())
            message = detail.get("base_resp", {}).get("status_msg") or detail.get("error", detail)
        except ValueError:
            message = "无法解析错误响应"
        raise RuntimeError(f"HTTP {error.code}: {message}") from error
    except (urllib.error.URLError, TimeoutError) as error:
        raise RuntimeError(f"请求失败：{error}") from error


def generate_minimax(model, token, args, references):
    payload = {
        "model": model,
        "prompt": args.prompt,
        "aspect_ratio": args.aspect_ratio,
        "response_format": "base64",
        "n": args.n,
        "prompt_optimizer": args.prompt_optimizer,
        "aigc_watermark": args.aigc_watermark,
    }
    if references:
        kind, value = references[0]
        image_file = value if kind == "url" else f"data:{kind};base64,{value}"
        payload["subject_reference"] = [{"type": "character", "image_file": image_file}]
    if args.seed is not None:
        payload["seed"] = args.seed
    if args.width is not None:
        payload["width"] = args.width
        payload["height"] = args.height
    host = "api.minimax.cn" if args.region == "cn" else "api.minimax.io"
    result = request_json(f"https://{host}/v1/image_generation", token, payload, "bearer", args.timeout)
    base_resp = result.get("base_resp", {})
    if base_resp.get("status_code", 0) != 0:
        raise RuntimeError(f"MiniMax API 错误 {base_resp.get('status_code')}: {base_resp.get('status_msg')}")
    images = result.get("data", {}).get("image_base64", [])
    if not images:
        raise RuntimeError(f"MiniMax 未返回图片：{result.get('metadata', {})}")
    return images, result.get("metadata", {}), "jpg"


def generate_gemini(model, token, args, references):
    parts = [{"text": args.prompt}]
    for kind, value in references:
        if kind == "url":
            raise ValueError("Ofox Gemini 参考图请使用本地 JPG/PNG 文件")
        parts.append({"inlineData": {"mimeType": kind, "data": value}})
    resolution = args.resolution or DEFAULT_RESOLUTIONS[args.model]
    payload = {
        "contents": [{"role": "user", "parts": parts}],
        "generationConfig": {
            "responseModalities": ["TEXT", "IMAGE"],
            "imageConfig": {"aspectRatio": args.aspect_ratio, "imageSize": resolution},
        },
    }
    url = f"https://api.ofox.ai/gemini/v1beta/models/{model}:generateContent"
    result = request_json(url, token, payload, "gemini", args.timeout)
    for candidate in result.get("candidates", []):
        for part in candidate.get("content", {}).get("parts", []):
            inline = part.get("inlineData")
            if inline and inline.get("data"):
                usage = dict(result.get("usageMetadata", {}))
                usage["resolution"] = resolution
                return [inline["data"]], usage, "png"
    raise RuntimeError(f"Ofox 响应中没有图片：{result.get('error') or result.get('promptFeedback') or '未知原因'}")


def generate_seedream(model, token, args, references):
    resolution = args.resolution or DEFAULT_RESOLUTIONS[args.model]
    payload = {
        "model": model,
        "prompt": args.prompt,
        "size": resolution,
        "response_format": "b64_json",
        "output_format": "png",
        "watermark": False,
    }
    if references:
        kind, value = references[0]
        image = value if kind == "url" else f"data:{kind};base64,{value}"
        payload["image"] = image
    result = request_json("https://api.ofox.ai/v1/images/generations", token, payload, "bearer", args.timeout)
    images = result.get("data", [])
    if not images or not images[0].get("b64_json"):
        raise RuntimeError("Ofox 响应中没有 base64 图片")
    usage = dict(result.get("usage", {}))
    usage["resolution"] = resolution
    return [images[0]["b64_json"]], usage, "png"


def save_images(images, extension, args):
    if args.out:
        paths = [Path(args.out)]
    else:
        paths = [Path(args.out_dir) / f"{args.out_prefix}-{i + 1}.{extension}" for i in range(len(images))]
    for path, encoded in zip(paths, images):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(base64.b64decode(encoded))
        print(path.resolve())


def main():
    args = parse_args()
    try:
        token = load_token(args.provider)
        references = [load_reference(value) for value in args.reference]
        model = MODELS[args.model][1]
        if args.dry_run:
            print(json.dumps({"provider": args.provider, "model": model, "reference_count": len(references)}, ensure_ascii=False))
            return 0
        if args.provider == "minimax-plan":
            images, usage, extension = generate_minimax(model, token, args, references)
        elif args.model.startswith("nano-banana"):
            images, usage, extension = generate_gemini(model, token, args, references)
        else:
            images, usage, extension = generate_seedream(model, token, args, references)
        save_images(images, extension, args)
        print(json.dumps({"provider": args.provider, "model": model, "usage": usage}, ensure_ascii=False))
        return 0
    except (ValueError, RuntimeError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
