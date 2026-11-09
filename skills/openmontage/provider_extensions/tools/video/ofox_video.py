"""OfoxAI video provider for OpenMontage, backed by the shared generate_video script."""

from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any

from tools.base_tool import (
    BaseTool,
    Determinism,
    ExecutionMode,
    ResumeSupport,
    ToolResult,
    ToolRuntime,
    ToolStability,
    ToolTier,
)


class OfoxVideo(BaseTool):
    name = "ofox_video"
    version = "1.0.0"
    tier = ToolTier.GENERATE
    stability = ToolStability.BETA
    execution_mode = ExecutionMode.ASYNC
    determinism = Determinism.STOCHASTIC
    runtime = ToolRuntime.API
    resume_support = ResumeSupport.FROM_CHECKPOINT
    capability = "video_generation"
    provider = "ofoxai"
    dependencies = ["env:OFOX_API_KEY"]
    install_instructions = "Set OFOX_API_KEY or configure providers.ofoxai.token in the openmontage skill config."
    fallback_tools = ["seedance_ark_video", "minimax_video"]
    best_for = ["short drama", "multi-reference video", "multi-shot narrative", "vertical video"]
    input_schema = {
        "type": "object",
        "required": ["prompt", "output_path"],
        "properties": {
            "prompt": {"type": "string"},
            "output_path": {"type": "string"},
            "model": {"type": "string", "default": "bytedance/seedance-2.5"},
            "duration": {"type": "integer", "minimum": 4, "maximum": 30, "default": 4},
            "resolution": {"type": "string", "default": "480p"},
            "aspect_ratio": {"type": "string", "default": "9:16"},
            "reference_image_paths": {"type": "array", "items": {"type": "string"}, "maxItems": 9},
            "generate_audio": {"type": "boolean", "default": True},
            "real_person": {"type": "boolean", "default": False},
            "provider_type": {"type": "string"},
        },
    }
    output_schema = {
        "type": "object",
        "properties": {
            "video_path": {"type": "string"},
            "task_id": {"type": "string"},
            "usage": {"type": "object"},
        },
    }

    def estimate_cost(self, inputs: dict[str, Any]) -> float:
        return 0.0

    def estimate_runtime(self, inputs: dict[str, Any]) -> float:
        return 180.0

    def execute(self, inputs: dict[str, Any]) -> ToolResult:
        started = time.monotonic()
        output_path = Path(inputs["output_path"]).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        command = [
            os.environ.get("PYTHON", "python3"),
            str(self._generator()),
            "--provider", "ofoxai",
            "--model", inputs.get("model", "bytedance/seedance-2.5"),
            "--resolution", inputs.get("resolution", "480p"),
            "--ratio", inputs.get("aspect_ratio", "9:16"),
            "--duration", str(inputs.get("duration", 4)),
            "--prompt", inputs["prompt"],
            "--out", str(output_path),
        ]
        if not inputs.get("generate_audio", True):
            command.append("--no-audio")
        if inputs.get("real_person"):
            command.append("--real-person")
        if inputs.get("provider_type"):
            command += ["--provider-type", inputs["provider_type"]]
        for path in inputs.get("reference_image_paths", []):
            command += ["--reference-image", str(Path(path).resolve())]
        result = subprocess.run(command, capture_output=True, text=True)
        if result.returncode:
            return ToolResult(success=False, error=result.stderr or result.stdout, duration_seconds=time.monotonic() - started)
        usage = self._usage(result.stdout)
        return ToolResult(
            success=True,
            data={"video_path": str(output_path), "task_id": self._task_id(output_path), "usage": usage},
            artifacts=[str(output_path)],
            cost_usd=usage.get("video_cost"),
            duration_seconds=time.monotonic() - started,
            model=inputs.get("model", "bytedance/seedance-2.5"),
        )

    def _generator(self) -> Path:
        default = Path(__file__).resolve().parents[4]
        root = Path(os.environ.get("OPENMONTAGE_SKILLS_ROOT", default))
        return root / "skills" / "video-generation" / "scripts" / "generate_video.py"

    @staticmethod
    def _task_id(output_path: Path) -> str | None:
        task_file = output_path.with_suffix(output_path.suffix + ".task.json")
        if not task_file.is_file():
            return None
        try:
            return json.loads(task_file.read_text(encoding="utf-8")).get("task_id")
        except (json.JSONDecodeError, OSError):
            return None

    @staticmethod
    def _usage(stdout: str) -> dict[str, Any]:
        for line in reversed(stdout.splitlines()):
            try:
                data = json.loads(line)
            except json.JSONDecodeError:
                continue
            return data.get("usage", data)
        return {}
