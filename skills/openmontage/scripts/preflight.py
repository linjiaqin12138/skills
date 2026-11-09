#!/usr/bin/env python3
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SKILL = ROOT / "skills" / "openmontage"


def main():
    checks = []
    upstream = ROOT / "vendor" / "OpenMontage"
    checks.append(("上游仓库", upstream.is_dir(), str(upstream)))
    links = [
        upstream / "tools" / "video" / "ofox_video.py",
        upstream / "pipeline_defs" / "short-drama.yaml",
        upstream / "skills" / "pipelines" / "short-drama",
    ]
    for path in links:
        checks.append((f"扩展链接 {path.name}", path.exists(), str(path)))
    for name in ("image-generation", "video-generation", "short-drama"):
        checks.append((f"{name} 执行脚本", (ROOT / "skills" / name / "scripts").is_dir(), name))
    config_path = SKILL / "config.json"
    try:
        providers = json.loads(config_path.read_text(encoding="utf-8"))["providers"]
        configured = [name for name, value in providers.items() if value.get("token", "").strip()]
        checks.append(("Provider token", bool(configured), ", ".join(configured)))
    except (OSError, json.JSONDecodeError, KeyError, AttributeError) as error:
        checks.append(("Provider token", False, str(error)))
    failed = [item for item in checks if not item[1]]
    for name, ok, detail in checks:
        print(f"{'OK' if ok else 'FAIL'}  {name}: {detail}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
