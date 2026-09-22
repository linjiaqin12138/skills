# Bite 1 · M0：NDJSON JSON-RPC 守护进程骨架（已通过验收）

## 交付
- `src/lib.rs`：协议类型。`ServerMessage` 用 `#[serde(untagged)]`，靠 id 字段是否存在区分响应/通知
- `src/main.rs`：miniduckd。50Hz tick 计数器（M1 起成为控制循环心跳）；`select!` 同连接并存读请求+推通知
- `src/bin/mini-duckctl.rs`：CLI，连接先 hello

## 验收实测
- health：握手 + `{"healthy":true,"tick":555,"uptime_s":11}`
- subscribe 3s：通知 tick 556→606→656，**+50/s 精确验证 50Hz 心跳**
- 未知方法 → `-32601 method not found: robot.nope`（带方法名）
- 垃圾帧 → `-32700` parse error

## 坑（实测踩到）
1. `nc -U` 测试会挂住（协议是持久连接），验收脚本必须 `timeout` 包住
2. `pkill -f miniduckd` 会杀掉自己的 shell（命令行含关键词）→ 用 `pkill -x`
3. interval 第一拍立即触发（见 questions.md Q3/Q4）

## 偏差
D1（API 子集）、D7（单 crate）、D8（无 SO_PEERCRED）、D15（版本号起点）——见 deviations.md
