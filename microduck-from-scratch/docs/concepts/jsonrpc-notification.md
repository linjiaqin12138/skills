# JSON-RPC 的两种消息：request 与 notification

## 一句话

JSON-RPC 的帧分两种：带 `id` 字段的是 **request**，服务端必须应答、应答里回显同一个 `id`；不带 `id` 的是 **notification**，服务端照常执行，但一个字节都不回。一个 `id` 字段的有无，决定了这条连接上会不会出现回程流量。

## 为什么需要

"发了就要回"是我们对 RPC 的默认直觉——调用方需要知道收没收到、成没成功。但有一类消息这个直觉不成立：**连续意图**。手柄摇杆的位置、机器人的目标速度，是以 20–50Hz 持续重发的流，每一帧 20ms 后就被下一帧覆盖。对这样的流逐帧应答：

- 是纯开销——50Hz 的意图配 50Hz 的应答，回程流量翻倍，而应答的内容（"收到了"）没有任何信息量：下一帧马上就到，这一帧的答案已经过期；
- 更糟的是应答会引诱调用方等它——一等，发送节奏就从"我有新值就发"退化成"上一个被确认才发下一个"，流变成了停走协议，延迟反而变大。

所以 JSON-RPC 在协议层面就准备了无应答形态：不写 `id`，服务端就知道这帧不用回。原版协议文档把消息家族和传输特性直接挂钩：连续意图（`move`/`head`）发 notification，"50Hz 下每帧一个应答是纯开销；重传的 80ms 前摇杆位置比没有更糟"（`reference/duck-ipc-proto/src/lib.rs:586-592`）；离散命令（`stop`/`enable`）发 request，因为调用方必须知道有没有被接受、为什么没接受（`:593-594`）。

配套的两条规范细则，本 Bite 都用到了：

1. **带 `id` 的连续意图也要应答**——规范允许两种写法，客户端拿 `robot.move` 当一次性调用发（带 id）时，服务端照常执行并回 accepted；不能因为"你 framing 选错了"就静默不理（`reference/robotd/src/main.rs:3747-3749` 的注释原话）。
2. **解析失败的请求回 `"id":null`**——帧本身没解析出来，就没有 id 可回显，规范规定错误响应的 id 填 null（`src/lib.rs:54-55`）。

## 最小示例

`examples/jsonrpc_notification.py` 单文件自包含：一个线程当 server（Unix socket + NDJSON，和 miniduckd 同构），client 依次发四种帧。host 直接跑 `python3 examples/jsonrpc_notification.py`，实测输出：

```text
1. request：带 id，服务端应答并回显 id
  发出: {"jsonrpc":"2.0","id":1,"method":"set","params":{"value":0.3}}
  收到: {"jsonrpc": "2.0", "id": 1, "result": 0.3}

2. notification：无 id，服务端静默执行
  发出: {"jsonrpc":"2.0","method":"set","params":{"value":0.7}}
  收到: （0.5s 内什么都没收到）

3. 再发一个 request 查状态——证明通知 0.7 真的落了
  发出: {"jsonrpc":"2.0","id":2,"method":"get"}
  收到: {"jsonrpc": "2.0", "id": 2, "result": 0.7}

4. 非法帧：解析不出 id，错误响应的 id 是 null
  发出: {这根本不是 JSON
  收到: {"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "Expecting property name ..."}}
```

第 2 步是重点：server 什么都没回（client 超时 0.5s 空等），但第 3 步查到的值是 0.7——"没回复"不等于"没执行"，这正是 notification 的语义。另外注意 server 端区分两种帧靠的只是一个 `if "id" in frame`。

## 素材

- [JSON-RPC 2.0 规范](https://www.jsonrpc.org/specification)：notification 一节只有两句话——"服务端不得应答，包括出错时"。
- 原版协议文档的 continuous/discrete 分类（`reference/duck-ipc-proto/src/lib.rs:580-594`）是这个概念落到机器人域的样子：它还顺手回答了"将来走 WebRTC 时该上哪条通道"——notification 形态的意图天然属于不可靠通道，消息家族已经替传输层做了选择。
- 原版服务端的实现（`reference/robotd/src/main.rs:3708-3716`）：`let Some(id) = request.id else { ...apply_intent...; continue; }`——无 id 直接跳过整个响应路径。

## 回到项目

- `src/lib.rs:33-44`：`Request.id` 是 `Option<u64>`，`skip_serializing_if = "Option::is_none"`——无 id 的帧序列化回去也不带 `"id"` 字段（单测 `request_without_id_is_notification`，`src/lib.rs:185-194`）。
- `src/lib.rs:56-72`：`ServerMessage::Response.id` 同为 `Option<u64>`——解析失败时回 `"id":null`（单测 `parse_error_response_has_null_id`，`:197-201`）。
- `src/main.rs:239`：分发 match 返回 `Option<ServerMessage>`，`None` = 本帧无回复；`src/main.rs:392-397`：`if let Some(resp)` 才 send。
- `src/main.rs:266-284`：`robot.move` 分支——`req.id.map(...)` 把"有没有 id"翻译成"回不回包"，通知与请求共用 `apply_move_intent`（`src/main.rs:200-204`）。
- 验收脚本不得不用 python3 裸 socket 发帧：`scripts/accept-m8.sh:36-38`——mini-duckctl 只会发带 id 的请求，发不了 notification。

## 自测

如果客户端把 50Hz 的 `robot.move` 全部以 request（带 id）形式发出，daemon 行为上哪里会和 notification 流不同？（提示：从 `src/main.rs:266-284` 出发想——语义一样吗？回程流量呢？）机器人为什么仍然应该按 notification 发？
