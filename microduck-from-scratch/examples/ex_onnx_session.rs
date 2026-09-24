//! 概念示例：把 ONNX 会话当成「本地函数」——固定形状进、固定形状出。
//! 不经过 `src/policy.rs`，只演示 Runtime 本身。
//!
//! ```bash
//! export ORT_DYLIB_PATH=third_party/onnxruntime/lib/libonnxruntime.so
//! docker compose exec rust cargo run --example ex_onnx_session
//! ```

use ort::session::Session;
use ort::value::Tensor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "policies/velstand.onnx".into());

    let mut session = Session::builder()?.commit_from_file(&path)?;
    let input_meta: Vec<_> = session
        .inputs()
        .iter()
        .map(|o| (o.name().to_owned(), format!("{:?}", o.dtype())))
        .collect();
    let output_meta: Vec<_> = session
        .outputs()
        .iter()
        .map(|o| (o.name().to_owned(), format!("{:?}", o.dtype())))
        .collect();
    println!("inputs:  {} {:?}", input_meta.len(), input_meta);
    println!("outputs: {} {:?}", output_meta.len(), output_meta);
    let action_name = output_meta[0].0.clone();

    // 全零观测，只把重力 z 设成直立时的 -1（与 FakeIo 默认 IMU 一致）。
    let mut obs = vec![0.0f32; 61];
    obs[5] = -1.0;
    let input = Tensor::from_array(([1usize, 61], obs))?;
    let out = session.run(ort::inputs!["obs" => &input])?;
    let (_, actions) = out[action_name.as_str()].try_extract_tensor::<f32>()?;
    println!("action_len: {}", actions.len());
    println!(
        "action: {}",
        actions
            .iter()
            .map(|v| format!("{v:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("all_finite: {}", actions.iter().all(|v| v.is_finite()));
    Ok(())
}
