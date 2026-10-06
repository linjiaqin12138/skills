//! 线路协议：JSON-RPC 2.0，一行一个对象（NDJSON），跑在 Unix socket 上。
//!
//! 为什么是 JSON 而不是打包的二进制结构：一帧只有几十到几百字节，
//! 肉眼可读、`nc -U` 可直接调试，比省下的字节值钱——原版 microduck 的
//! `duck-ipc-proto` 文档里记录的是同一条理由，外加"两种语言共享协议时
//! 二进制偏移错了不会报错"的失败史。

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod control;
pub mod io;
pub mod model;
pub mod obs;
pub mod policy;
pub mod safety;
pub mod scheduler;
pub mod updater;

pub const JSONRPC: &str = "2.0";

/// API 版本，hello 时交换。
///
/// 差异只报告、不拒绝：真正会弄坏对端的是"方法不存在"和"参数形状变了"，
/// 这两件事在调用点各自拒绝即可（METHOD_NOT_FOUND / INVALID_PARAMS），
/// 在握手上设卡会连本来能服务的调用一起拒掉。
pub const API_VERSION: u32 = 1;

pub const METHOD_NOT_FOUND: i32 = -32601;
pub const PARSE_ERROR: i32 = -32700;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    /// None = notification：JSON-RPC 规定无 id 的帧不回复。连续意图
    /// （robot.move 等）走这条路——50Hz 的意图每帧都回一包纯属开销，
    /// 而且对一个 20ms 后就被覆盖的速度也没什么可说的（原版 robotd
    /// 同款理由）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObj {
    pub code: i32,
    pub message: String,
}

/// 服务端发出的消息：响应或通知（订阅推送用）。
/// untagged 靠"id 字段是否存在"区分两种形态——这是 JSON-RPC 能用同一条
/// 连接同时跑请求/响应/推送而不需要第二条通道的原因。
/// 响应的 id 可空：请求本身解析失败时没有 id 可回显，按规范回 null。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
    Response {
        jsonrpc: String,
        id: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<ErrorObj>,
    },
    Notification {
        jsonrpc: String,
        method: String,
        params: Value,
    },
}

impl ServerMessage {
    pub fn ok(id: Option<u64>, result: Value) -> Self {
        ServerMessage::Response {
            jsonrpc: JSONRPC.into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Option<u64>, code: i32, message: impl Into<String>) -> Self {
        ServerMessage::Response {
            jsonrpc: JSONRPC.into(),
            id,
            result: None,
            error: Some(ErrorObj {
                code,
                message: message.into(),
            }),
        }
    }

    pub fn notify(method: &str, params: Value) -> Self {
        ServerMessage::Notification {
            jsonrpc: JSONRPC.into(),
            method: method.into(),
            params,
        }
    }
}

/// robot.move 参数：连续速度意图（原版 MoveParams 形状：
/// serde(default, deny_unknown_fields)，缺省按 0，未知字段拒绝）。
///
/// 单位与坐标系（这段注释承重，照原版 duck-ipc-proto 的精神写）：
/// vx/vy 是躯干系线速度 m/s，vyaw 是偏航角速度 rad/s。右手系：
/// x 前、y 左、z 上；vyaw 正为左转。原版的教训是这条约定不写下来，
/// 每个消费方各自 empirical 定号，最后长出一串 --*-sign 开关。
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MoveParams {
    /// 前进，m/s。
    pub vx: f64,
    /// 向左，m/s。
    pub vy: f64,
    /// 偏航角速度，rad/s，正为左转。
    pub vyaw: f64,
}

impl MoveParams {
    /// 有限则给出 [vx, vy, vyaw]。NaN/inf 不是驾驶意图，拒掉。
    /// （JSON 线路其实到不了这里——serde_json 解析 1e999 直接报
    /// "number out of range"；这层是兜底的深度防御。）
    pub fn finite_twist(&self) -> Option<[f64; 3]> {
        let twist = [self.vx, self.vy, self.vyaw];
        twist.iter().all(|v| v.is_finite()).then_some(twist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: Value) -> Result<MoveParams, serde_json::Error> {
        serde_json::from_value(value)
    }

    #[test]
    fn move_params_full_triple_passes() {
        let p = parse(serde_json::json!({"vx": 0.1, "vy": 0.2, "vyaw": -0.3})).unwrap();
        assert_eq!(p.finite_twist(), Some([0.1, 0.2, -0.3]));
    }

    #[test]
    fn move_params_missing_fields_default_zero() {
        let p = parse(serde_json::json!({"vx": 0.1, "vyaw": 0.2})).unwrap();
        assert_eq!(p.finite_twist(), Some([0.1, 0.0, 0.2]));
        // 全缺省 = 停车命令，合法。
        let p = parse(serde_json::json!({})).unwrap();
        assert_eq!(p.finite_twist(), Some([0.0, 0.0, 0.0]));
    }

    #[test]
    fn move_params_unknown_field_rejected() {
        assert!(parse(serde_json::json!({"vx": 0.1, "vz": 9.0})).is_err());
    }

    #[test]
    fn move_params_wrong_type_rejected() {
        assert!(parse(serde_json::json!({"vx": "fast"})).is_err());
    }

    #[test]
    fn move_params_non_finite_rejected() {
        // 线路上到不了的形状（Value 装不下 inf/NaN），直接构造结构体
        // 验证 finite_twist 这道兜底门。
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(MoveParams { vx: bad, ..Default::default() }.finite_twist(), None);
            assert_eq!(MoveParams { vy: bad, ..Default::default() }.finite_twist(), None);
            assert_eq!(MoveParams { vyaw: bad, ..Default::default() }.finite_twist(), None);
        }
    }

    #[test]
    fn move_params_json_overflow_rejected_at_parse() {
        // 1e999 超出 f64：serde_json 在解析阶段就拒（number out of
        // range），所以 daemon 回的是 PARSE_ERROR 而非 INVALID_PARAMS。
        assert!(serde_json::from_str::<Value>("1e999").is_err());
    }

    #[test]
    fn request_without_id_is_notification() {
        let req: Request = serde_json::from_str(
            r#"{"jsonrpc":"2.0","method":"robot.move","params":{"vx":0.1}}"#,
        )
        .unwrap();
        assert_eq!(req.id, None);
        // 通知序列化回去也不带 id 字段。
        let line = serde_json::to_string(&req).unwrap();
        assert!(!line.contains("\"id\""), "{line}");
    }

    #[test]
    fn parse_error_response_has_null_id() {
        let msg = ServerMessage::err(None, PARSE_ERROR, "boom");
        let line = serde_json::to_string(&msg).unwrap();
        assert!(line.contains("\"id\":null"), "{line}");
    }
}
