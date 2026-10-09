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

/// robot.enable 参数（原版 EnableParams 形状：deny_unknown_fields，
/// toggle 缺省 false——reference/duck-ipc-proto/src/lib.rs:2685-2704）。
///
/// toggle 是手柄 Start 语义：daemon 侧翻转当前 enabled（此时 on 被忽略），
/// 因为客户端自己持有的开关信念会随对端重启、robot.relax 等漂移——信念
/// 一旧，Start 就变成隔次失灵的按钮。开关归属 robot，按一下永远是
/// "另一个状态"。on 没有 serde default：toggle 不免除 on（原版同）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnableParams {
    pub on: bool,
    #[serde(default)]
    pub toggle: bool,
}

/// 解析 robot.enable 参数；Err 的文案即 INVALID_PARAMS 响应的 message。
/// 仿 parse_move 风格，但 bool 没有有限值问题，少一道兜底门。
pub fn parse_enable(params: &Value) -> Result<EnableParams, String> {
    serde_json::from_value(params.clone())
        .map_err(|e| format!("robot.enable wants {{on: bool}} (optional toggle: bool): {e}"))
}

/// robot.subscribe 参数。hz 缺省（或 0）表示客户端要每拍；
/// 本工程解析后丢弃，推送仍是控制循环全速率——逐订阅者降频是 D25，不在这层做。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SubscribeParams {
    pub hz: Option<u32>,
}

/// robot.subscribe 的 ack。只往外序列化。
///
/// 这些字段在进程活着的时候不变，所以放在订阅应答里而不是每帧推送：
/// 50Hz 重复两个文件名没有信息量。stand 恒为 None——本工程没有 stand 槽（D33）。
/// skills 只放配置表里加载成功的一次性技能名（robot.do 用的 name，不是文件名），
/// 不含 ground_pick / sit_toggle：那两个是内置名，名单在 robot.skills 里。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubscribeResult {
    /// 回答时恒为 true：订阅不拒绝，策略没加载上也要让客户端订上推送。
    pub accepted: bool,
    /// 尝试过的 walk 文件名（不是路径）。加载失败也要报，客户端才能看见是哪个文件没起来。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub walk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stand: Option<String>,
    /// walk 没起来的原因。成功则省略。文案是 "policy would not load: …"，
    /// 与 health 的 "policy unavailable: …" 不是同一句。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sitstand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_pick: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
}

/// 解析 robot.subscribe 参数；Err 的文案即 INVALID_PARAMS 响应的 message。
///
/// 缺省 params 在 Request 里是 Null。hz 全可选，Null 与 {} 都是「每拍」，
/// 否则手写的 `{"method":"robot.subscribe"}` 会因为 null 不是对象而被拒。
pub fn parse_subscribe(params: &Value) -> Result<SubscribeParams, String> {
    let params = if params.is_null() {
        Value::Object(serde_json::Map::new())
    } else {
        params.clone()
    };
    serde_json::from_value(params)
        .map_err(|e| format!("robot.subscribe wants {{hz?: u32}}: {e}"))
}

/// 组一份进程生命周期内不变的订阅 ack。load_error 是 PolicyError 的 Display；
/// 调用方在 spawn 前算好文件名传进来，订阅路径不再碰磁盘。
pub fn assemble_subscribe_ack(
    walk_file: Option<String>,
    load_error: Option<&str>,
    sitstand: Option<String>,
    ground_pick: Option<String>,
    skills: Vec<String>,
) -> SubscribeResult {
    SubscribeResult {
        accepted: true,
        walk: walk_file,
        stand: None,
        unavailable: load_error.map(|e| format!("policy would not load: {e}")),
        sitstand,
        ground_pick,
        skills,
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

    #[test]
    fn enable_params_on_true() {
        let p = parse_enable(&serde_json::json!({"on": true})).unwrap();
        assert!(p.on && !p.toggle);
    }

    #[test]
    fn enable_params_toggle_defaults_false() {
        let p = parse_enable(&serde_json::json!({"on": false})).unwrap();
        assert!(!p.on && !p.toggle);
    }

    #[test]
    fn enable_params_toggle_true_alongside_on() {
        // toggle 与 on 同帧是合法形状（daemon 侧忽略 on）——手柄 Start 就是这么发的。
        let p = parse_enable(&serde_json::json!({"on": true, "toggle": true})).unwrap();
        assert!(p.on && p.toggle);
    }

    #[test]
    fn enable_params_toggle_alone_rejected() {
        // on 没有 serde default（对照 duck-ipc-proto EnableParams）：toggle 不免除 on。
        assert!(parse_enable(&serde_json::json!({"toggle": true})).is_err());
    }

    #[test]
    fn enable_params_unknown_field_rejected() {
        assert!(parse_enable(&serde_json::json!({"on": true, "foo": 1})).is_err());
    }

    #[test]
    fn enable_params_missing_on_rejected() {
        assert!(parse_enable(&serde_json::json!({})).is_err());
        assert!(parse_enable(&Value::Null).is_err());
    }

    #[test]
    fn subscribe_params_empty_and_absent_mean_every_tick() {
        assert_eq!(parse_subscribe(&serde_json::json!({})).unwrap().hz, None);
        // Request 缺 params 字段时 serde default 给出 Null。
        let req: Request = serde_json::from_str(
            r#"{"jsonrpc":"2.0","id":1,"method":"robot.subscribe"}"#,
        )
        .unwrap();
        assert!(req.params.is_null());
        assert_eq!(parse_subscribe(&req.params).unwrap().hz, None);
    }

    #[test]
    fn subscribe_params_hz_zero_and_ten_accepted() {
        // 0 与 10 都是合法 u32。降频不在这层：两者都只是 Some(n)。
        assert_eq!(parse_subscribe(&serde_json::json!({"hz": 10})).unwrap().hz, Some(10));
        assert_eq!(parse_subscribe(&serde_json::json!({"hz": 0})).unwrap().hz, Some(0));
    }

    #[test]
    fn subscribe_params_bad_shapes_rejected() {
        for bad in [
            serde_json::json!({"hz": 10, "foo": 1}),
            serde_json::json!({"hz": -1}),
            serde_json::json!({"hz": "10"}),
        ] {
            let err = parse_subscribe(&bad).unwrap_err();
            assert!(err.contains("{hz?: u32}"), "{err}");
        }
    }

    #[test]
    fn subscribe_result_omits_stand_unavailable_and_empty_skills() {
        let full = assemble_subscribe_ack(
            Some("velstand.onnx".into()),
            None,
            Some("alpha_sitstand.onnx".into()),
            Some("alpha_ground_pick.onnx".into()),
            vec!["roulade".into(), "kick_left".into()],
        );
        let v = serde_json::to_value(&full).unwrap();
        assert!(v.get("stand").is_none(), "{v}");
        assert!(v.get("unavailable").is_none(), "{v}");
        assert_eq!(v["accepted"], true);
        assert_eq!(v["skills"], serde_json::json!(["roulade", "kick_left"]));
        // ack 不回显 hz：SubscribeResult 上就没有这个字段。
        assert!(v.get("hz").is_none(), "{v}");

        let bare = assemble_subscribe_ack(Some("velstand.onnx".into()), None, None, None, Vec::new());
        let v = serde_json::to_value(&bare).unwrap();
        assert!(v.get("skills").is_none(), "{v}");
        assert!(v.get("stand").is_none(), "{v}");
        assert!(v.get("unavailable").is_none(), "{v}");
    }

    #[test]
    fn subscribe_result_names_the_walk_file_that_failed() {
        let failed = assemble_subscribe_ack(
            Some("velstand.onnx".into()),
            Some("reading policies/velstand.onnx: No such file or directory"),
            None,
            None,
            Vec::new(),
        );
        let v = serde_json::to_value(&failed).unwrap();
        assert_eq!(v["walk"], "velstand.onnx");
        let unavailable = v["unavailable"].as_str().unwrap();
        assert!(unavailable.contains("policy would not load"), "{unavailable}");
        assert!(v.get("stand").is_none(), "{v}");
        assert!(v.get("skills").is_none(), "{v}");
    }
}
