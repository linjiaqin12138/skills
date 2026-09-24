//! 单策略 ONNX 推理（M3：只加载 velstand）。
//!
//! load 时校验观测/动作维数；推理前拒绝非有限观测；失败不改 LSTM 状态
//!（若本文件是 feed-forward 则根本没有状态）。多网络 / 热切换留给 M6。

use std::fmt;
use std::path::{Path, PathBuf};

use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::tensor::TensorElementType;
use ort::value::{Tensor, Value, ValueType};

use crate::io::ImuData;
use crate::model::DEFAULT_POSITION;
use crate::obs::{ACTION_LEN, OBS_LEN, Observation};

const INTRA_THREADS: usize = 1;

#[derive(Debug)]
pub enum PolicyError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Load {
        path: PathBuf,
        source: ort::Error,
    },
    Shape {
        path: PathBuf,
        what: &'static str,
        expected: String,
        got: String,
    },
    Inference(String),
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyError::Read { path, source } => write!(f, "reading {}: {source}", path.display()),
            PolicyError::Load { path, source } => write!(f, "loading {}: {source}", path.display()),
            PolicyError::Shape {
                path,
                what,
                expected,
                got,
            } => write!(
                f,
                "{}: {what} is {got}, expected {expected}",
                path.display()
            ),
            PolicyError::Inference(msg) => write!(f, "inference failed: {msg}"),
        }
    }
}

impl std::error::Error for PolicyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PolicyError::Read { source, .. } => Some(source),
            PolicyError::Load { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// 已打开并通过维数校验的策略会话。
pub struct Policy {
    session: Session,
    /// LSTM 时非空：load 时分配零状态，成功推理后拷回，失败清零。
    state: Option<LstmState>,
    action_name: String,
    path: PathBuf,
}

struct LstmState {
    h: Tensor<f32>,
    c: Tensor<f32>,
}

impl Policy {
    /// 打开文件、校验契约、用全零观测做一次 warmup。
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PolicyError> {
        let path = path.as_ref();
        let _bytes = std::fs::read(path).map_err(|source| PolicyError::Read {
            path: path.to_owned(),
            source,
        })?;

        let session = Session::builder()
            .and_then(|b| b.with_optimization_level(GraphOptimizationLevel::Level3))
            .and_then(|b| b.with_intra_threads(INTRA_THREADS))
            .and_then(|b| b.commit_from_file(path))
            .map_err(|source| PolicyError::Load {
                path: path.to_owned(),
                source,
            })?;

        let inputs = session.inputs();
        let outputs = session.outputs();
        check_matrix(path, require_named(path, inputs, "obs")?, OBS_LEN)?;

        let recurrent = match (inputs.len(), outputs.len()) {
            (1, 1) => false,
            (3, 3) => true,
            counts => {
                return Err(shape_err(
                    path,
                    "input/output contract",
                    "obs -> actions, or obs/h_in/c_in -> actions/h_out/c_out",
                    format!("{counts:?} tensors"),
                ));
            }
        };

        let action = if recurrent {
            require_named(path, outputs, "actions")?
        } else {
            &outputs[0]
        };
        check_matrix(path, action, ACTION_LEN)?;
        let action_name = action.name().to_owned();

        let state = if recurrent {
            Some(alloc_lstm_state(path, inputs, outputs)?)
        } else {
            None
        };

        let mut policy = Self {
            session,
            state,
            action_name,
            path: path.to_owned(),
        };

        // warmup：证明 runtime 真能跑，并把第一拍冷启动开销挡在控制循环外。
        let zero = zero_observation();
        let _ = policy.infer(&zero)?;
        policy.reset_state();
        Ok(policy)
    }

    /// 一次前向。观测必须全有限；输出必须正好 14 个有限 f32。
    pub fn infer(&mut self, observation: &Observation) -> Result<[f32; ACTION_LEN], PolicyError> {
        let path_label = self.path.display().to_string();
        let fail = |e: String| PolicyError::Inference(format!("{path_label}: {e}"));

        if !observation.as_slice().iter().all(|v| v.is_finite()) {
            return Err(fail("non-finite observation".into()));
        }

        let input = Value::from_array(([1usize, OBS_LEN], observation.as_slice().to_vec()))
            .map_err(|e| fail(format!("building input: {e}")))?;

        let outputs = match &self.state {
            Some(state) => self.session.run(ort::inputs![
                "obs" => &input,
                "h_in" => &state.h,
                "c_in" => &state.c
            ]),
            None => self.session.run(ort::inputs!["obs" => &input]),
        }
        .map_err(|e| fail(e.to_string()))?;

        let (_, actions) = outputs[self.action_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|e| fail(e.to_string()))?;
        if actions.len() != ACTION_LEN || !actions.iter().all(|v| v.is_finite()) {
            return Err(fail(format!(
                "expected {ACTION_LEN} finite actions, got len={}",
                actions.len()
            )));
        }
        let mut result = [0.0; ACTION_LEN];
        result.copy_from_slice(actions);

        if self.state.is_some() {
            let copy_result = {
                let state = self.state.as_mut().unwrap();
                copy_lstm_outputs(&outputs, state)
            };
            drop(outputs);
            if let Err(e) = copy_result {
                self.reset_state();
                return Err(fail(e));
            }
        }

        Ok(result)
    }

    fn reset_state(&mut self) {
        if let Some(state) = &mut self.state {
            if let Ok((_, h)) = state.h.try_extract_tensor_mut::<f32>() {
                h.fill(0.0);
            }
            if let Ok((_, c)) = state.c.try_extract_tensor_mut::<f32>() {
                c.fill(0.0);
            }
        }
    }
}

fn zero_observation() -> Observation {
    Observation::build(
        &ImuData::default(),
        &DEFAULT_POSITION,
        &[0.0; crate::model::NUM_JOINTS],
        &DEFAULT_POSITION,
        &[0.0; ACTION_LEN],
        &crate::obs::Command::default(),
    )
}

fn shape_err(
    path: &Path,
    what: &'static str,
    expected: impl Into<String>,
    got: impl Into<String>,
) -> PolicyError {
    PolicyError::Shape {
        path: path.to_owned(),
        what,
        expected: expected.into(),
        got: got.into(),
    }
}

fn require_named<'a>(
    path: &Path,
    outlets: &'a [ort::value::Outlet],
    name: &str,
) -> Result<&'a ort::value::Outlet, PolicyError> {
    outlets.iter().find(|o| o.name() == name).ok_or_else(|| {
        shape_err(
            path,
            "tensor names",
            name,
            format!("{:?}", outlets.iter().map(|o| o.name()).collect::<Vec<_>>()),
        )
    })
}

fn tensor_shape(path: &Path, outlet: &ort::value::Outlet) -> Result<Vec<i64>, PolicyError> {
    match outlet.dtype() {
        ValueType::Tensor {
            ty: TensorElementType::Float32,
            shape,
            ..
        } => Ok(shape.to_vec()),
        other => Err(shape_err(
            path,
            "tensor type",
            "float32",
            format!("{}: {other:?}", outlet.name()),
        )),
    }
}

fn check_matrix(path: &Path, outlet: &ort::value::Outlet, width: usize) -> Result<(), PolicyError> {
    let shape = tensor_shape(path, outlet)?;
    if shape.len() != 2 || (shape[0] != 1 && shape[0] != -1) || shape[1] != width as i64 {
        return Err(shape_err(
            path,
            if width == OBS_LEN {
                "observation width"
            } else {
                "action width"
            },
            format!("{width}"),
            format!("{}: shape {shape:?}", outlet.name()),
        ));
    }
    Ok(())
}

fn alloc_lstm_state(
    path: &Path,
    inputs: &[ort::value::Outlet],
    outputs: &[ort::value::Outlet],
) -> Result<LstmState, PolicyError> {
    let mut shapes = Vec::new();
    for (outlets, name) in [
        (inputs, "h_in"),
        (inputs, "c_in"),
        (outputs, "h_out"),
        (outputs, "c_out"),
    ] {
        let shape = tensor_shape(path, require_named(path, outlets, name)?)?;
        if shape.len() != 3
            || shape[0] <= 0
            || (shape[1] != 1 && shape[1] != -1)
            || shape[2] <= 0
        {
            return Err(shape_err(
                path,
                "LSTM state shape",
                "[positive layers, 1 or dynamic batch, positive hidden size]",
                format!("{name}: {shape:?}"),
            ));
        }
        shapes.push(vec![shape[0], 1, shape[2]]);
    }
    if shapes.iter().any(|s| s != &shapes[0]) {
        return Err(shape_err(
            path,
            "LSTM state shapes",
            "matching h/c input and output shapes",
            format!("{shapes:?}"),
        ));
    }
    let shape = &shapes[0];
    let count = usize::try_from(shape[0])
        .ok()
        .and_then(|n| n.checked_mul(shape[2] as usize))
        .filter(|&n| n <= 1_048_576)
        .ok_or_else(|| {
            shape_err(
                path,
                "LSTM state size",
                "at most 1048576 elements per state",
                format!("{shape:?}"),
            )
        })?;
    let make = || {
        Tensor::from_array((shape.clone(), vec![0.0f32; count])).map_err(|source| {
            PolicyError::Load {
                path: path.to_owned(),
                source,
            }
        })
    };
    Ok(LstmState {
        h: make()?,
        c: make()?,
    })
}

fn copy_lstm_outputs(
    outputs: &ort::session::SessionOutputs<'_>,
    state: &mut LstmState,
) -> Result<(), String> {
    let (hs, h) = outputs["h_out"]
        .try_extract_tensor::<f32>()
        .map_err(|e| e.to_string())?;
    let (cs, c) = outputs["c_out"]
        .try_extract_tensor::<f32>()
        .map_err(|e| e.to_string())?;
    let (expected_h, h_in) = state
        .h
        .try_extract_tensor_mut::<f32>()
        .map_err(|e| e.to_string())?;
    let (expected_c, c_in) = state
        .c
        .try_extract_tensor_mut::<f32>()
        .map_err(|e| e.to_string())?;
    if hs != expected_h || cs != expected_c || !h.iter().chain(c).all(|v| v.is_finite()) {
        return Err("invalid LSTM output state shape or non-finite state".into());
    }
    h_in.copy_from_slice(h);
    c_in.copy_from_slice(c);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn ort_ready() -> bool {
        match std::env::var("ORT_DYLIB_PATH") {
            Ok(p) if !p.is_empty() => Path::new(&p).exists(),
            _ => {
                // 脚本默认摊平路径；测试在仓库根或 docker /work 下跑。
                let candidates = [
                    PathBuf::from("third_party/onnxruntime/lib/libonnxruntime.so"),
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("third_party/onnxruntime/lib/libonnxruntime.so"),
                ];
                candidates.iter().any(|p| p.exists())
            }
        }
    }

    fn ensure_ort_env() {
        if std::env::var_os("ORT_DYLIB_PATH").is_none() {
            let so = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("third_party/onnxruntime/lib/libonnxruntime.so");
            if so.exists() {
                // SAFETY: 测试进程内设置一次查找路径；ort 首次 API 调用前必须就绪。
                unsafe {
                    std::env::set_var("ORT_DYLIB_PATH", &so);
                }
            }
        }
    }

    fn velstand_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("policies/velstand.onnx")
    }

    fn write_bad_obs_width_onnx(out: &Path) {
        // 用 Python onnx 包写 obs 宽 51 的 1-in/1-out 图；load 校验应报 expected 61。
        let script = r#"
import sys
from onnx import helper, TensorProto, save

w = helper.make_tensor("W", TensorProto.FLOAT, [51, 14], [0.0] * (51 * 14))
graph = helper.make_graph(
    [helper.make_node("MatMul", ["obs", "W"], ["actions"])],
    "bad_obs_width",
    [helper.make_tensor_value_info("obs", TensorProto.FLOAT, [1, 51])],
    [helper.make_tensor_value_info("actions", TensorProto.FLOAT, [1, 14])],
    [w],
)
model = helper.make_model(graph, opset_imports=[helper.make_opsetid("", 13)])
save(model, sys.argv[1])
"#;
        let status = Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(out)
            .status()
            .expect("spawn python3 to write bad onnx");
        assert!(status.success(), "python onnx writer failed: {status}");
        assert!(out.is_file() && out.metadata().unwrap().len() > 0);
    }

    #[test]
    fn rejects_wrong_observation_width() {
        if !ort_ready() {
            eprintln!("skip rejects_wrong_observation_width: ORT dylib missing");
            return;
        }
        ensure_ort_env();

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad_obs_width_51.onnx");
        write_bad_obs_width_onnx(&bad);

        let err = match Policy::load(&bad) {
            Err(e) => e,
            Ok(_) => panic!("51-wide obs must fail load"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("61") && (msg.contains("51") || msg.contains("got")),
            "error must mention expected 61 and actual width, got: {msg}"
        );
    }

    #[test]
    fn velstand_zero_obs_then_feedback_stays_finite() {
        let path = velstand_path();
        if !path.is_file() || !ort_ready() {
            eprintln!("skip velstand test: missing policy or ORT");
            return;
        }
        ensure_ort_env();

        let mut policy = Policy::load(&path).expect("load velstand");
        let zero = zero_observation();
        let a1 = policy.infer(&zero).expect("first infer");
        assert_eq!(a1.len(), 14);
        for (i, v) in a1.iter().enumerate() {
            assert!(v.is_finite(), "a1[{i}] not finite: {v}");
            assert!(v.abs() < 2.0, "a1[{i}]={v} exceeds distribution bound");
        }

        let next = Observation::build(
            &ImuData::default(),
            &DEFAULT_POSITION,
            &[0.0; crate::model::NUM_JOINTS],
            &DEFAULT_POSITION,
            &a1,
            &crate::obs::Command::default(),
        );
        let a2 = policy.infer(&next).expect("second infer with last_action");
        for (i, v) in a2.iter().enumerate() {
            assert!(v.is_finite(), "a2[{i}] not finite: {v}");
            assert!(v.abs() < 2.0, "a2[{i}]={v} exceeds distribution bound");
        }
    }
}
