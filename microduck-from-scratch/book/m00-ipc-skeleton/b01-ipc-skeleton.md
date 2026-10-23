# 第 1 章 · M0：NDJSON JSON-RPC 守护进程骨架

> 本章对应里程碑 M0（Bite 1，已验收）。交付物：`src/lib.rs`（协议类型）、`src/main.rs`（miniduckd 守护进程）、`src/bin/mini-duckctl.rs`（CLI）。
>
> 注意：教程的代码是连续演进的（里程碑文件夹只放文档，代码不按里程碑切片）。本章引用的行号以当前仓库代码为准——`src/main.rs` 已被 M1 演进过一次，凡是 M1 改动的位置本章都会注明；协议层（`src/lib.rs`）和 CLI 从 M0 至今未变。

## 1. 动机

整个复刻工程的终态是：50 Hz 控制循环跑 ONNX 强化学习策略，驱动 15 个舵机。但第一个里程碑一行控制代码都没有，只有一个守护进程骨架加一把 CLI 钳子。为什么？

因为 **IPC 骨架是后续全部里程碑的验收工具链**。看依赖树：

- M1（身体模型+FakeIo）的健康数据（reads/writes/skipped_reads）要通过 `robot.health` 暴露；
- M2 的 61 维观测向量、M3 的策略推理结果，要通过 `robot.state` 订阅流推出来给肉眼验收；
- M5 的 deadman 急停、M6 的技能切换，都是客户端发进来的 JSON-RPC 调用。

也就是说：没有这把"钳子"，后面每个里程碑都只能靠日志和单测验收，而"daemon 活着、在按 50 Hz 跑、能被外部进程观察和指挥"这件事本身——恰恰是机器人系统区别于纯计算库的地方——将永远得不到端到端验证。M0 把"进程间怎么说话"一次定死，之后每个里程碑只在方法表里加行，不动传输层。

骨架定下来的三个不可再谈判的决定（全部来自原版的架构决策，见 Phase 1 决策卡片）：Unix socket 而非 TCP（文件权限就是鉴权，且默认网络不可达）；JSON-RPC 2.0 over NDJSON 而非 HTTP（一条持久连接上请求/响应/推送三态共存）；hello 握手先行（版本差异只报告、不拒绝）。

## 2. 概念铺垫

### 【背景卡片】守护进程 + Unix domain socket

- **一句话定义**：daemon 是脱离终端常驻后台、通过 socket 接受命令的进程；Unix domain socket 是文件系统上一个特殊的"文件"（`/tmp/miniduckd.sock`），两个本地进程通过它双向通信，不走网络栈。
- **为什么需要**：机器人上有多个进程要指挥同一块硬件（CLI、蓝牙服务、更新服务），硬件所有权必须收在一个进程里，其余进程都当客户端。选 Unix socket 而非 TCP 的两条理由（原版的决策，详见 questions.md Q1）：① socket 文件的 Unix 权限位就是免费鉴权——谁能连由文件系统说了算；② 默认网络不可达，"机器人被局域网里别的机器连上"这个失败模式在传输层就不存在。
- **TS/Node 类比**：`net.createServer().listen('/tmp/x.sock')`——Node 的 `net` 模块原生支持 Unix socket，API 和 TCP 完全一样，只是 listen 的参数从端口换成路径。你也早就用过它：Docker 的 `/var/run/docker.sock`、MySQL 的 `/tmp/mysql.sock` 都是这个东西。
- **差异点**：Node 里 Unix socket 是 TCP 的一个"参数变体"；在系统层面它是独立的协议族（`AF_UNIX`），没有 IP、端口、拥塞控制，内核直接在两个进程间搬字节，且能传文件描述符和进程凭证（`SO_PEERCRED`——M0 没用它做鉴权，登记为偏差 D8）。

### 【背景卡片】JSON-RPC 2.0 / NDJSON 帧格式

- **一句话定义**：JSON-RPC 2.0 是一个极简 RPC 约定（请求带 `id`/`method`/`params`，响应带同一个 `id` 加 `result` 或 `error`，不带 `id` 的是通知）；NDJSON（newline-delimited JSON）指"一行一个完整 JSON 对象"，换行符就是帧边界。
- **为什么需要**：一条持久连接上要同时跑三种流量——客户端发请求、服务端回响应、服务端主动推订阅通知。JSON-RPC 的 id 字段让请求和响应配对（同连接多请求并发不串），通知不带 id 所以天然可区分。选 JSON 而非二进制的原因写在 `src/lib.rs:3-6`：一帧几十到几百字节，肉眼可读、`nc -U` 可以直接手打调试，比省下的字节值钱；原版协议 crate 文档记录的是同一条理由。
- **最小示例**（大白话：socket 送来的是字节流，不保证一次到一整帧——可能半帧、可能两帧粘一起。NDJSON 的规则是"攒到换行符才算一帧"）：

```js
// examples/ndjson_frames.mjs —— node examples/ndjson_frames.mjs 可跑
let buf = "";
const chunks = ['{"id":1,"method":"hello"}\n{"id":2,"met', 'hod":"health"}\n'];

for (const chunk of chunks) {
  buf += chunk;
  const lines = buf.split("\n");
  buf = lines.pop(); // 最后一段可能不完整，留到下一片到了再拼
  for (const line of lines) console.log("完整帧:", JSON.parse(line));
}
console.log("缓冲区残留:", JSON.stringify(buf));
```

实际输出——第一片只凑齐一帧，第二片把剩半帧补齐，缓冲区清空：

```text
完整帧: { id: 1, method: 'hello' }
完整帧: { id: 2, method: 'health' }
缓冲区残留: ""
```

- **TS/Node 类比**：VS Code 的 Language Server Protocol（LSP）就是 JSON-RPC over 持久流——你在 TS 生态里天天间接受益。NDJSON 对应 Node 里 `readline` 按行切流的用法。
- **差异点**：LSP 用 `Content-Length` 头做帧边界（HTTP 风格），这里是裸 NDJSON——换行即边界，代价是 payload 内不能出现裸换行（`serde_json` 序列化默认不产生，天然满足）。

### 【背景卡片】tokio 运行时

- **一句话定义**：tokio 是 Rust 的异步运行时：`async fn` 产出的 future 是惰性的，必须有运行时 poll 它才会推进；`#[tokio::main]`（`src/main.rs:23`）把一个 worker 线程池塞进进程，所有 `.await` 点都是协作式让出。
- **为什么需要**：daemon 要同时服务多条连接、每条连接同时读请求和推通知——这是 I/O 密集型并发，正是事件循环的主场。
- **TS/Node 类比**：就是 Node 的事件循环。`tokio::spawn(fut)` ≈ 把一个 async 函数丢进微任务队列；`.await` ≈ `.await`。心智模型可以直接搬。
- **差异点**：① Node 单线程，tokio 默认多线程 work-stealing——一个任务这次 `.await` 回来可能跑在另一个 OS 线程上，所以跨 `.await` 持有的数据必须 `Send`（编译器强制）；② Rust 的 future 是惰性的：不 spawn/不 await 就永远不执行，不像 Promise 创建即开始；③ 没有"隐式全局循环"，`tokio::time::Instant` 和 `std::time::Instant` 是两种钟（这个坑在 M1 踩到，见第 2 章常见坑）。

### 【背景卡片】`select!`：一条连接上的多路复用

- **一句话定义**：`tokio::select!`（`src/main.rs:68`）同时等待多个 future，谁先就绪执行谁的分支，其余分支被**取消**（drop 掉），然后整个 `select!` 重新求值。
- **为什么需要**：订阅连接上，"客户端又发来一个请求"和"1 秒推送定时器到点了"是两条独立事件流，任何一条都不能阻塞另一条。
- **最小示例**（大白话：你同时竖着耳朵听两件事——定时器到点、客户端来消息，谁先有动静就先处理谁）：

```rust
// examples/ex_select.rs —— cargo run --example ex_select 可跑（此处省略 use 与 #[tokio::main] 包装）
let mut timer = interval(Duration::from_millis(100));
let (tx, mut rx) = tokio::sync::mpsc::channel::<&str>(1);
tokio::spawn(async move {
    sleep(Duration::from_millis(250)).await;
    tx.send("客户端请求").await.unwrap();
});
for _ in 0..4 {
    tokio::select! {
        _ = timer.tick() => println!("分支A：定时器到点"),
        Some(msg) = rx.recv() => println!("分支B：收到{msg}"),
    }
}
```

实际输出——前 300ms 定时器连赢三拍，250ms 时消息抢在第 4 拍之前到达：

```text
分支A：定时器到点
分支A：定时器到点
分支A：定时器到点
分支B：收到客户端请求
```

（顺带：第一拍在 t=0 立即输出——这正是本章 Q3 那个"interval 首拍立即触发"的现场。）

- **TS/Node 类比**：`Promise.race([readNextFrame(), pushTimer])` 包在一个 `while` 循环里。
- **差异点**：① `Promise.race` 里输掉的分支**继续跑**（Promise 无法取消），`select!` 里输掉的分支被 drop——写进分支的局部状态会丢，这正是 `subscribed` 标志要放在 `select!` 外面的原因（`src/main.rs:58`）；② `select!` 分支可以带 `if` 守卫（`src/main.rs:118` 的 `, if subscribed`）：条件不满足时该分支本轮不参与竞争，Promise.race 没有对应物；③ 分支是轮询而非回调，宏展开后是一次公平性随机的轮询，不是事件注册。

### 【背景卡片】serde 的 `#[serde(untagged)]`

- **一句话定义**：`ServerMessage`（`src/lib.rs:47-61`）是一个枚举：`Response`（有 `id`）或 `Notification`（无 `id`）。`untagged` 让序列化结果里**不带**枚举标签，反序列化时按声明顺序逐个变体试，哪个能配上就算哪个。
- **为什么需要**：这是 JSON-RPC 能用同一条连接混跑响应和通知而不开第二条通道的关键——线格式本身就是两种形状，靠"id 字段是否存在"区分（`src/lib.rs:42-44` 的注释）。
- **TS/Node 类比**：TS 的 discriminated union，如 `type Msg = {id: number, ...} | {method: string, ...}`，消费端用 `'id' in msg` 收窄。
- **差异点**：TS 的判别是编译期类型收窄、运行时你自己写 `in` 检查；serde untagged 是**运行时逐变体试配**——性能更差、错误信息更含糊，且变体形状必须真有区分度（两个变体字段完全兼容时永远命中先声明的那个）。本项目两个变体一个必有 `id`、一个必无，安全。

## 3. 实现走读

### 3.1 协议层（`src/lib.rs`）

协议层是唯一被 daemon 和 CLI 共同依赖的部分，所以它是一个 lib crate 的根，两个 bin 都吃它。

- **版本号不设卡**。`API_VERSION`（`src/lib.rs:22`）的注释写清了设计原则：真正会弄坏对端的是"方法不存在"和"参数形状变了"，这两件事在调用点各自拒绝（`-32601` / `-32602`）；握手上设卡会把本来能服务的调用一起拒掉。原版行为相同（"版本差异只报告、不拒绝"），只有版本号起点不同（偏差 D15）。
- **一个枚举收编三种出站消息**。`ServerMessage`（`src/lib.rs:47-61`）的 `Response` 变体把 `result` 和 `error` 都做成 `Option` 并配 `skip_serializing_if`——成功响应线上就没有 `error` 字段，反之亦然，帧保持最小。三个构造器 `ok` / `err` / `notify`（`src/lib.rs:63-92`）是全部出站消息的**唯一**入口，保证 `jsonrpc: "2.0"` 这类样板字段不可能漏写。
- **错误码直接用 JSON-RPC 规范保留段**：`-32601` method not found、`-32700` parse error（`src/lib.rs:24-25`）。不自创错误码空间——规范已有的就用，自创的语义客户端还得学一遍。

### 3.2 守护进程（`src/main.rs`）

> M1 演进说明：M0 时 `main.rs` 里有一个 50 Hz 的 tokio interval 任务，只做一件事——递增 tick 计数器（它是"控制循环心跳"这个语义的占位）。M1 把它替换成了真的控制任务 `control::spawn`（`src/main.rs:37`），文件头注释（`src/main.rs:1-7`）记录了这次接管。下面走读的是当前代码，标注 M0 原貌的地方除外。

- **启动先清场**。`main.rs:40-41` 先 `remove_file` 再 `bind`：上次异常退出残留的 socket 文件会让 bind 报 `AddrInUse`。Node 里 `net` 服务器有同样的坑（`EADDRINUSE`），解法相同。
- **accept 循环，一连接一任务**。`main.rs:44-48`：每来一条连接 `tokio::spawn` 一个 `serve`。注释写明理由——某条连接的客户端卡住不能拖死其他连接。这和 Node 的 `server.on('connection')` 回调同构，只是这里每个连接是显式的独立任务。
- **行帧编解码外包给 codec**。`main.rs:57` 用 `Framed` + `LinesCodec`（tokio-util）把字节流变成"一行一个 `String`"的流，daemon 代码里一个字节解析都没有。Node 类比：给 socket 套一个按 `\n` 切分的 Transform stream。
- **`select!` 是本里程碑的心脏**（`main.rs:68-134`）。两个分支：
  - 读分支（`main.rs:69-117`）：`framed.next()` 等到一行 → 解析失败回 `-32700` 并 `continue`（连接不断，只拒这一帧）；解析成功按 `method` 分派。对端断开或帧损坏统一走 `break` 正常收尸（`main.rs:70-74`）——"任何对端都可能死"是正常结局，不是异常。
  - 推分支（`main.rs:118-133`）：1 Hz 定时器到点且 `subscribed` 为真时，取最新传感数据组装通知推下去。（M0 时推送载荷只有 tick；`positions` 字段是 M1 加的。）
- **解析失败的响应 id 用 0**（`main.rs:78-79`）。JSON-RPC 规范说 id 不可知时用 `null`；本协议 `id` 类型是 `u64` 装不下 null，M0 简化为 0。这是刻意的，注释里标了"M0 简化"。
- **未知方法按名字拒绝**（`main.rs:107-112`）：`method not found: robot.nope`，错误消息带上方法名——调试时不用猜。注释称这是"协议里唯一被允许的拒绝"：能解析、有对应方法的请求永不因版本/身份被拒（呼应上面的版本原则）。
- **健康端点读的是循环自己的账**（M1 形态，`main.rs:92-102`）：`tick`/`reads`/`writes`/`skipped_reads` 全部来自控制任务维护的 `Stats`。语义是"循环在跑"，不是"socket 活着"。M0 时只有 `healthy`/`tick`/`uptime_s` 三个字段，但"由心跳任务自己记账、RPC 层只读不猜"的原则从 M0 就在。

### 3.3 CLI（`src/bin/mini-duckctl.rs`）

- **任何客户端进来第一件事是 hello**（`mini-duckctl.rs:27`，实现在 `57-70`）：握手不是可选项，原版同。hello 响应打到 stderr（`mini-duckctl.rs:67`），真正的调用结果打 stdout——管道下游只拿到干净数据。
- **手写参数解析**（`mini-duckctl.rs:15-19`）：两个子命令不值得引 clap，注释明说"clap 留给功能面膨胀到值得它的里程碑"。这是全项目"当前里程碑够用即可"规则的样板（铁律 9）。
- **subscribe 的实现就是"调一次 `robot.state`，然后把这条连接上剩下的一切原样倒到 stdout"**（`mini-duckctl.rs:33-46`）——订阅没有取消协议，断开即退订。

## 4. 验收

验收命令（来自 STATE.md 的标准路径）：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo build
./target/debug/miniduckd &
./target/debug/mini-duckctl health
timeout 3 ./target/debug/mini-duckctl subscribe
# 另开途径打错误帧：未知方法 / 垃圾文本
pkill -x miniduckd
```

实测输出（逐字摘自 `docs/milestones/m0-ipc-skeleton/acceptance.md`）：

> - health：握手 + `{"healthy":true,"tick":555,"uptime_s":11}`
> - subscribe 3s：通知 tick 556→606→656，**+50/s 精确验证 50Hz 心跳**
> - 未知方法 → `-32601 method not found: robot.nope`（带方法名）
> - 垃圾帧 → `-32700` parse error

断言解释：

- **`tick:555, uptime_s:11`**：555 ÷ 11 ≈ 50.5，tick 速率就是心跳频率。`healthy:true` 的可信度不在于这个布尔值，而在于它附带的 tick 是**活的**——连发两次 health，tick 必须涨。
- **subscribe 3 秒收到 tick 556→606→656**：推送频率 1 Hz（`main.rs:62`），相邻通知的 tick 差恰好 +50——这是对"50 Hz 心跳"最硬的行为断言：不是日志声称 50 Hz，而是外部观察者用计数差实测出 50 Hz。这个"计数差验证速率"的手法后面每个里程碑都要用。
- **`-32601` 带方法名**：协议承诺的"唯一一种拒绝"按约出现，且消息可定位。
- **`-32700`**：垃圾帧被拒但连接不死（服务端 `continue` 而非 `break`），后续请求照常服务。

## 5. 与原版差异

| 偏差 | 内容 | 预定收敛点 |
|---|---|---|
| D1 | 只实现 `robot.*` 极小子集（hello/health/state），非完整 API 面 | M8 对照 `reference/duck-ipc-proto` 逐项核对 |
| D7 | 单 crate 双 bin，非原版 workspace 多 crate | M8 拆协议 crate，对齐"协议 crate 只许 serde/serde_json/semver"依赖约束 |
| D8 | 无 `SO_PEERCRED` uid/gid 校验 | 第一个 mutating 调用出现时（M5/M7）——只读调用不设防是刻意的 |
| D15 | `API_VERSION=1` 起步；握手差异只报告不拒绝（行为与原版一致，仅版本号起点不同） | M8 |

四条全部是待收敛项而非豁免：每一条都在偏差登记簿里带着收敛条件活着。

## 6. 常见坑

1. **`nc -U` 测试会挂住**。协议是持久连接：daemon 不会主动关，nc 也不会主动退。所有验收脚本必须用 `timeout` 包住（`timeout 3 nc -U ...`），否则 CI 卡死。
2. **`pkill -f miniduckd` 会杀掉自己的 shell**。`-f` 匹配整条命令行，而你的 shell 命令行里恰好含着 "miniduckd" 这几个字符。用 `pkill -x miniduckd`（精确匹配进程名）。
3. **tokio interval 第一拍立即触发**——详见下面读者提问 Q3/Q4。M0 里它无害，但它是 M4 的埋雷点，必须在本章就建立正确心智模型。

## 7. 读者提问

**Q3：怎么理解 tokio interval 第一拍立即触发？**

`tokio::time::interval(period)` 的语义是"周期时刻表，第一个时刻是**现在**"——创建即就绪，第一次 `tick().await` 立刻返回。这和 `setInterval(fn, 20)` 的心智模型直接冲突：`setInterval` 是"延迟重复"，第一次回调在 20 ms 之后。

M0 里这无害：订阅者立刻有反馈甚至是好事。但同样的写法搬进 M4 的控制循环就是事故：第一拍在 t=0 执行，此时一次传感器数据都没读到，用未就绪数据推理 = 舵机抽一下。修法是 `interval_at(Instant::now() + period, period)`，把时刻表整体推迟一个周期——已在 M1 的 `src/control.rs:57` 应用，注释里标了"已知坑，勿改回 interval"（`src/control.rs:53-56`）。

**Q4（追问）："节奏相位永久错位一拍"没理解。**

澄清一下：稳态之后两种写法的节奏**完全相同**——都是每 20 ms 一拍。唯一差别是 tokio 写法在 t=0 多执行了一次循环体。所谓"错位"只是记账问题：tick 计数换算回时间时恒差一拍。真风险从来不是错位，是**第一拍的输入是垃圾**。

顺带一个原版的实测证据，说明控制循环的定时器语义值得较真：原版 robotd 对 `MissedTickBehavior::Delay` 的注释（`reference/robotd/src/main.rs:1786-1801`）记录了实测数据——用 `Delay` 时循环只跑到 43.1 Hz（目标 50 Hz），因为 `Delay` 把每拍的唤醒延迟累加进周期，调度抖动变成永久速率损失，"看起来会像硬件问题"；原版因此选 `Skip`（丢拍保节奏）。这就是为什么 tokio 定时器的三个旋钮（首拍时刻、missed-tick 行为、时钟类型）每个都要想清楚，不能抄默认值。
