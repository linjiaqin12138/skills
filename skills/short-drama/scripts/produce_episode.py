#!/usr/bin/env python3
import argparse
import concurrent.futures
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
GENERATOR = REPO / "skills" / "video-generation" / "scripts" / "generate_video.py"


def parse_args():
    parser = argparse.ArgumentParser(description="通用 AI 短剧：分任务生成、单镜重跑与合成")
    parser.add_argument("--project", required=True, help="episode 目录，包含 prompts.md 和 tasks.json")
    parser.add_argument("--shot", help="只运行指定任务，例如 05 或 03-04")
    parser.add_argument("--force", action="store_true", help="删除同任务旧视频和任务 ID 后重跑")
    parser.add_argument("--jobs", type=int, default=3)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--no-compose", action="store_true")
    parser.add_argument("--bgm", help="可选背景音乐文件，自动进行人声闪避")
    return parser.parse_args()


def load_config(project):
    return json.loads((project / "tasks.json").read_text(encoding="utf-8"))


def load_prompts(project, prompt_file):
    text = (project / prompt_file).read_text(encoding="utf-8")
    matches = re.findall(r"## 任务 ([\d-]+)\n.*?```text\n(.*?)\n```", text, re.DOTALL)
    return {task_id: prompt.strip() for task_id, prompt in matches}


def generate(project, config, task, prompts, dry_run, force):
    task_id = task["id"]
    output = project / "shots" / f"shot-{task_id}.mp4"
    task_file = output.with_suffix(output.suffix + ".task.json")
    if force:
        for path in (output, task_file):
            if path.exists():
                path.unlink()
    command = [
        sys.executable, str(GENERATOR),
        "--provider", config["provider"],
        "--model", task.get("model", config["model"]),
        "--resolution", config["resolution"],
        "--ratio", config["ratio"],
        "--duration", str(task["duration"]),
        "--prompt", prompts[task_id],
        "--out", str(output),
    ]
    if task.get("real_person"):
        command.append("--real-person")
    if task.get("provider_type"):
        command += ["--provider-type", task["provider_type"]]
    for reference in task["references"]:
        command += ["--reference-image", str((project / reference).resolve())]
    if dry_run:
        command.append("--dry-run")
    result = subprocess.run(command, capture_output=True, text=True)
    print(f"===== {task_id} =====\n{result.stdout}{result.stderr}", flush=True)
    if result.returncode:
        raise RuntimeError(f"任务 {task_id} 失败")
    return output


def compose(project, config, tasks, bgm):
    outputs = [project / "shots" / f"shot-{task['id']}.mp4" for task in tasks]
    missing = [str(path) for path in outputs if not path.is_file() or path.stat().st_size == 0]
    if missing:
        raise RuntimeError(f"缺少镜头：{missing}")
    width = config.get("output", {}).get("width", 480)
    height = config.get("output", {}).get("height", 864)
    command = ["ffmpeg", "-y", "-hide_banner"]
    for path in outputs:
        command += ["-i", str(path)]
    filters = []
    labels = []
    for index in range(len(outputs)):
        filters.append(f"[{index}:v]fps=24,scale={width}:{height}:flags=lanczos,setsar=1,format=yuv420p[v{index}]")
        filters.append(f"[{index}:a]aresample=48000[a{index}]")
        labels.append(f"[v{index}][a{index}]")
    filters.append("".join(labels) + f"concat=n={len(outputs)}:v=1:a=1[v][a]")
    joined = project / "joined.mp4"
    command += ["-filter_complex", ";".join(filters), "-map", "[v]", "-map", "[a]", "-c:v", "libx264", "-crf", "19", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart", str(joined)]
    subprocess.run(command, check=True)
    final = project / "final.mp4"
    command = ["ffmpeg", "-y", "-hide_banner", "-i", str(joined)]
    video_filters = []
    if (project / "episode.ass").is_file():
        video_filters.append(f"ass={project / 'episode.ass'}")
    if bgm:
        command += ["-i", bgm]
        command += ["-filter_complex", f"[1:a]loudnorm=I=-26:TP=-2:LRA=7[bgm];[bgm][0:a]sidechaincompress=threshold=0.02:ratio=8:attack=20:release=400[aout]", "-map", "0:v", "-map", "[aout]", "-c:a", "aac", "-b:a", "192k"]
    else:
        command += ["-map", "0:v", "-map", "0:a", "-c:a", "copy"]
    if video_filters:
        command += ["-vf", ",".join(video_filters), "-c:v", "libx264", "-crf", "19"]
    else:
        command += ["-c:v", "copy"]
    command += ["-movflags", "+faststart", str(final)]
    subprocess.run(command, check=True)
    print(final.resolve(), flush=True)


def main():
    args = parse_args()
    project = Path(args.project).resolve()
    config = load_config(project)
    prompts = load_prompts(project, config["prompt_file"])
    tasks = config["tasks"]
    selected = [task for task in tasks if not args.shot or task["id"] == args.shot]
    if not selected:
        print(f"找不到任务：{args.shot}", file=sys.stderr)
        return 2
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as executor:
            futures = [executor.submit(generate, project, config, task, prompts, args.dry_run, args.force) for task in selected]
            for future in concurrent.futures.as_completed(futures):
                future.result()
        if not args.dry_run and not args.no_compose:
            compose(project, config, tasks, args.bgm)
        return 0
    except (RuntimeError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
