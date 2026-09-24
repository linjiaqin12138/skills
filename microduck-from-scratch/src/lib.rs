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
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObj {
    pub code: i32,
    pub message: String,
}

/// 服务端发出的消息：响应（带 id）或通知（不带 id，订阅推送用）。
/// untagged 靠"id 字段是否存在"区分两种形态——这是 JSON-RPC 能用同一条
/// 连接同时跑请求/响应/推送而不需要第二条通道的原因。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
    Response {
        jsonrpc: String,
        id: u64,
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
    pub fn ok(id: u64, result: Value) -> Self {
        ServerMessage::Response {
            jsonrpc: JSONRPC.into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: u64, code: i32, message: impl Into<String>) -> Self {
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
