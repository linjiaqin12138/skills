# 本仓库里的 Rust 语法

按语言本身归类。库函数（`tokio`、`serde`、`ort`）只在它们逼出某种语法时带一句。

项目结构：`src/lib.rs` 是库 `miniduck`，`src/main.rs` 是守护进程，`src/bin/mini-duckctl.rs` 是另一个可执行文件，`examples/` 是独立示例。库和二进制是两个 crate，二进制通过 `use miniduck::...` 调用库。crate 本身见 [crate 是什么](crate.md)。

## 1. 模块、路径、`use`

`lib.rs` 用 `pub mod` 把同目录下的 `control.rs`、`io.rs` 等挂进库，并对外公开：

```rust
// src/lib.rs
pub mod control;
pub mod io;
pub mod model;
pub mod obs;
pub mod policy;
```

路径有三种起点：

- `crate::`：当前 crate 的根。库内部写 `crate::io::RobotIo`。
- `super::`：父模块。测试里 `use super::*;` 表示把父模块所有公开项引进来。
- 外部 crate 名：二进制里写 `miniduck::policy::Policy`，标准库写 `std::sync::Arc`。

`use` 的几种写法都出现了：

```rust
// src/main.rs
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use futures::{SinkExt, StreamExt};
use miniduck::io::FakeIo;
use miniduck::policy::Policy;
use miniduck::{API_VERSION, METHOD_NOT_FOUND, PARSE_ERROR, Request, ServerMessage, control};
```

花括号是一次引入多项。最后一行把常量、类型和子模块 `control` 混在一次 `use` 里，所以后面可以写 `control::spawn`。

`use std::io::Write as _;` 里的 `as _` 是重命名成匿名。`Write` 这个 trait 必须在作用域里，`.flush()` 才找得到实现，但代码里不需要出现 `Write` 这个名字。

## 2. 注释

- `//` 行注释，编译器丢掉。
- `///` 文档注释，挂在紧跟着的那一项上（函数、结构体、常量）。
- `//!` 内部文档注释，描述当前模块，所以都写在文件最上面。

## 3. 条目：常量、函数、结构体、枚举、类型别名

常量在编译期就定死，类型必须写出来：

```rust
// src/lib.rs
pub const JSONRPC: &str = "2.0";
pub const API_VERSION: u32 = 1;
```

`pub const HEAD_JOINTS: std::ops::Range<usize> = 5..9;` 里，`5..9` 是一个值（区间），类型是 `Range<usize>`，表示 5、6、7、8。

函数的形状是 `fn 名字(参数) -> 返回类型`。没有返回值时写成 `()`，通常省略。参数默认是不可变的；要改就加 `mut`：

```rust
// src/control.rs
pub fn spawn(
    mut io: impl RobotIo + 'static,
    mut policy: Policy,
) -> (Arc<Stats>, watch::Receiver<FrameSnapshot>) {
```

返回值是元组，调用处用 `let (stats, frame_rx) = control::spawn(...)` 拆开。

方法写在 `impl 类型` 里。第一个参数决定它怎么被调用：

| 写法 | 含义 | 调用 |
|---|---|---|
| `fn new() -> Self` | 关联函数，不拿现成的值 | `FakeIo::new()` |
| `fn as_slice(&self)` | 共享借用 | `obs.as_slice()` |
| `fn infer(&mut self, ...)` | 独占借用，可以改自己 | `policy.infer(&observation)` |

`Self` 就是当前 `impl` 的那个类型。`Policy::load(...) -> Result<Self, PolicyError>` 里的 `Self` 就是 `Policy`。

结构体有三种形态，这个项目都有。

普通结构体，字段有名字：

```rust
// src/policy.rs
pub struct Policy {
    session: Session,
    state: Option<LstmState>,
    action_name: String,
    path: PathBuf,
}
```

没写 `pub` 的字段只在本模块里能读写。外部只能通过方法碰它们。

元组结构体，字段靠序号访问。`IoError(pub String)` 的字符串是 `self.0`：

```rust
// src/io.rs
#[derive(Debug, Clone)]
pub struct IoError(pub String);

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
```

构造时字段名和变量名相同可以省略，这叫字段初始化简写：`PolicyError::Read { path: path.to_owned(), source }` 里的 `source` 等价于 `source: source`。

还有结构体更新语法，没写到的字段从另一个值抄来：

```rust
// src/io.rs
Ok(Sensors {
    positions: self.position,
    ..Sensors::default()
})
```

枚举的每个变体可以长得不一样。`ServerMessage` 的两个变体都是带名字字段的结构体变体；`PolicyError::Inference(String)` 是元组变体：

```rust
// src/lib.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
    Response {
        jsonrpc: String,
        id: u64,
        result: Option<Value>,
        error: Option<ErrorObj>,
    },
    Notification {
        jsonrpc: String,
        method: String,
        params: Value,
    },
}
```

类型别名不创造新类型，只是换个短名字。`io.rs` 里的 `pub type Result<T> = std::result::Result<T, IoError>;` 让这个模块里的 `Result<Sensors>` 默认错误类型是 `IoError`。它和标准库的 `Result` 同名，在 `io` 模块内部会挡住标准库那个。

## 4. 可见性

- 什么都不写：仅当前模块（以及它的子模块）可见。`LstmState`、`fn p99` 都是这样。
- `pub`：父模块能看见；若一路 `pub` 到 crate 根，外部 crate 也能用。
- 字段单独标 `pub`：结构体公开，但字段也可以选择不公开。`Policy` 的字段全是私有的，`Stats` 的字段是公开的。

## 5. 属性

属性是写在项上面的 `#[...]`，交给编译器或宏处理。

`#[derive(...)]` 让编译器按固定规则生成实现。这里用过：

- `Debug`：可以用 `{:?}` 打印
- `Clone`：可以显式 `.clone()` 复制一份
- `Copy`：赋值就是按位复制，不会把原值移走。`Command`、`ImuData` 标了 `Copy`，因为里面全是数字
- `Default`：得到默认值，通常是全零。需要特殊默认值时就自己写 `impl Default`，例如 `ImuData::default()` 的重力是 `[0.0, 0.0, -1.0]`
- `PartialEq`：可以用 `==`
- `Serialize` / `Deserialize`：来自 `serde`，变成 JSON 来回转换

`#[serde(...)]` 调整序列化。`#[serde(default)]` 表示 JSON 里缺 `params` 时用默认值。`#[serde(untagged)]` 表示枚举不写变体名，靠字段形状区分 `Response` 和 `Notification`。`#[serde(skip_serializing_if = "Option::is_none")]` 里那个字符串是函数路径：值为 `None` 时不写出这个字段。

`#[cfg(test)]` 表示这段只在 `cargo test` 时编译。`#[test]` 把函数登记成测试。`#[inline]` 建议编译器把 `joint_of` 嵌进调用处。`#[tokio::main]` 是属性宏，把 `async fn main` 包进一个运行时，这样 `main` 里才能 `.await`。

## 6. 类型写法

数组长度是类型的一部分：`[f64; 15]` 和 `[f64; 14]` 是不同类型。`[0.0; NUM_JOINTS]` 是用 `0.0` 重复 N 次。`[0.0f32; ACTION_LEN]` 的 `f32` 后缀指定元素类型。

切片是对一段连续元素的借用：`&[f64]` 长度在运行时才知道，`&[f64; 15]` 长度是类型的一部分。`observation.as_slice()` 返回 `&[f32]`。

区间用于循环和切片：

- `0..NUM_JOINTS`：不含终点
- `1..=50`：含终点
- `d[0..3]`、`d[7..20]`：切出子切片
- `data[OFF_LAST_ACTION..OFF_LAST_ACTION + ACTION_LEN]`：一段连续拷贝的来源或目标

元组把几个不同类型捆在一起：`(Arc<Stats>, watch::Receiver<...>)`、`(u64, [f64; NUM_JOINTS])`。用 `.0`、`.1` 或 `let (a, b) = ...` 取出。

单元类型 `()` 表示没有有意义的值。`write` 成功时返回 `Result<()>`，`Ok(())` 只表示成功这件事。`main` 末尾的 `Ok(())` 同理。

`Option<T>` 表示有或没有：`Some(x)` / `None`。`ramp_origin: Option<(u64, [f64; NUM_JOINTS])>` 在第一次读成功之前是 `None`。

`Result<T, E>` 表示成功或失败：`Ok(v)` / `Err(e)`。

## 7. 所有权、借用、可变性

每个值有一个所有者。把一个不能 `Copy` 的值赋给别人、或传给函数，所有权就移走，原地方不能再使用。`String`、`Policy`、`Vec<f64>` 都是这样。`[f64; 15]` 因为元素是 `Copy`，数组本身也是 `Copy`，`self.position = targets.positions` 是复制，原来的 `targets` 还在。

参数和 `let` 前面的 `mut` 只表示这个绑定可以重新赋值或通过它改内容：

```rust
// src/control.rs
let mut ramp_origin: Option<(u64, [f64; NUM_JOINTS])> = None;
let mut tick: u64 = 0;
let mut last_action = [0.0f32; ACTION_LEN];
let mut previous_targets: Option<[f64; NUM_JOINTS]> = None;
```

共享引用 `&T` 可以同时有很多个，不能通过它改值。独占引用 `&mut T` 同时只能有一个，可以通过它改。

```rust
// src/policy.rs
pub fn infer(&mut self, observation: &Observation) -> Result<[f32; ACTION_LEN], PolicyError> {
```

`&mut self` 是因为推理会改 LSTM 状态。`observation` 用 `&`，因为只读。调用时写 `policy.infer(&observation)`：`&` 在调用点显式借出。

`*` 是解引用，从引用回到值。`Copy` 类型解引用得到副本：

```rust
// src/obs.rs
for (slot, value) in action.iter().enumerate() {
    out[joint_of(slot)] = *value as f64;
}
```

`*rx.borrow()`、`*ramp_origin.get_or_insert(...)` 也是解引用。

`Arc<Stats>` 是共享所有权：`clone()` 只增加引用计数，几个任务看同一份计数器。里面的 `AtomicU64` 允许在共享引用下改数字（内部可变性），所以 `stats.tick.store(...)` 不需要 `&mut Stats`。

`let _ = ...` 和 `let _bytes = ...`：下划线表示这个值我收下但不用。`_bytes` 仍会执行 `std::fs::read`，只是不让编译器报变量未使用。单独的 `_` 模式则是直接丢掉。

显式 `drop(outputs)` 用来提前结束一次借用。`infer` 里先从 `outputs` 借出数据拷进 `state`，拷完必须先丢掉 `outputs` 的借用，才能接着 `&mut self` 去 `reset_state()`。

## 8. 生命周期

大多数引用的生命周期由编译器自动推断（生命周期省略）。函数签名里同时出现输入引用和输出引用、编译器无法唯一对应时，就要写出名字。

```rust
// src/policy.rs
fn require_named<'a>(
    path: &Path,
    outlets: &'a [ort::value::Outlet],
    name: &str,
) -> Result<&'a ort::value::Outlet, PolicyError> {
```

`'a` 把返回的 `&Outlet` 和入参 `outlets` 绑在一起：返回的引用活不过那块切片。

`'_` 是匿名生命周期，用在「这里确实有个生命周期，但名字不重要」的地方：`fmt::Formatter<'_>`、`SessionOutputs<'_>`。

`'static` 表示活得和整个进程一样久。字符串字面量 `&str` 的数据在二进制里，所以是 `'static`。`impl RobotIo + 'static` 表示传进来的 `io` 自己拥有全部数据，不借用函数外面的局部变量，因此可以被 `tokio::spawn` 拿到另一个任务里一直用。

`Box<dyn std::error::Error>` 是 trait 对象：运行时才知道具体错误类型。`dyn Error + 'static` 表示这个错误值不包含短命引用。`main` 返回 `Result<(), Box<dyn std::error::Error>>`，这样 `?` 可以把各种错误收成同一种。

## 9. 泛型与 trait

Trait 是一组方法约定。`RobotIo` 还带了一个超 trait `Send`，表示实现者可以安全地跨线程移交：

```rust
// src/io.rs
pub trait RobotIo: Send {
    fn read(&mut self) -> Result<Sensors>;
    fn write(&mut self, targets: &JointTargets) -> Result<()>;
}
```

`impl RobotIo for FakeIo` 是给具体类型补上这组方法。`impl fmt::Display for PolicyError`、`impl std::error::Error for IoError` 同理。`Display` 对应 `{}` 格式化，`Error` 让它能放进 `?` 和 `source()` 错误链。

`impl Trait` 写在参数位置，表示任何实现了这个 trait 的类型，调用点不用写出具体类型名：

```rust
// src/policy.rs
pub fn load(path: impl AsRef<Path>) -> Result<Self, PolicyError> {
```

`&str`、`String`、`PathBuf` 都能当 `path`。`message: impl Into<String>` 表示任何能转成 `String` 的东西（字面量、`String` 都可以），函数内部用 `.into()` 收成 `String`。

`spawn` 的 `impl RobotIo + 'static` 是同一个写法加上生命周期约束。具体类型在编译每个调用点时确定，没有运行时虚表。这和 `Box<dyn Error>` 不同：后者把具体类型藏进堆分配，靠虚函数调用。

`const fn` 表示函数可以在编译期执行。`joint_of` 只被运行时调用，但标了 `const fn` 以后，常量上下文里也能用它。

## 10. 模式匹配与控制流

`match` 必须覆盖所有可能。每个分支是 `模式 => 表达式`。

```rust
// src/main.rs
let resp = match req.method.as_str() {
    "hello" => ServerMessage::ok(...),
    "robot.health" => ServerMessage::ok(...),
    "robot.state" => { subscribed = true; ServerMessage::ok(...) }
    other => ServerMessage::err(..., format!("method not found: {other}")),
};
```

`"hello"` 匹配这一个字符串。`other` 接住其余情况，同时把匹配到的 `&str` 绑到名字 `other` 上。分支可以是一个块，块的最后一个表达式就是该分支的值，所以 `match` 本身可以当作值赋给 `resp`。

嵌套模式一次拆两层：

```rust
// src/main.rs
let line = match frame {
    Some(Ok(line)) => line,
    _ => break,
};
```

`Some(Ok(line))` 同时要求外层是 `Some`、内层是 `Ok`。`_` 匹配其余一切并且不绑定名字。

结构体模式可以只写出关心的字段，`..` 忽略其余：

```rust
// src/policy.rs
match self {
    PolicyError::Read { source, .. } => Some(source),
    PolicyError::Load { source, .. } => Some(source),
    _ => None,
}
```

`if let` 只处理一种成功形态，其余什么都不做：

```rust
// src/control.rs
if let Some(previous) = previous {
```

```rust
// src/control.rs
if let Some(prev) = last_tick_at {
```

`while let` 在模式还能配上时继续循环：

```rust
// src/bin/mini-duckctl.rs
while let Some(frame) = framed.next().await {
```

带守卫的 `match` 分支（`policy.rs` 测试里）：

```rust
// src/policy.rs
match std::env::var("ORT_DYLIB_PATH") {
    Ok(p) if !p.is_empty() => Path::new(&p).exists(),
```

`if` 条件为假时，这个臂不算匹配成功，继续看后面的臂。

`if` / `else` 也是表达式。`check_matrix` 里用它选出一段字符串。`loop` 是无限循环，靠 `break` 或 `continue` 离开或进入下一轮。`for i in 0..N` 消费一个迭代器。`for _ in 0..3` 的 `_` 表示只要次数、不要元素。`return` 从当前函数返回；写在 `match` 臂里时，是从整个函数返回。

`const fn joint_of` 里的 `if/else` 同样是表达式，两个分支的类型都是 `usize`。

## 11. 错误处理：`?`、组合子、提前返回

`?` 用在返回 `Result` 或 `Option` 的函数里：遇到 `Err` / `None` 立刻用同样的失败返回，遇到 `Ok` / `Some` 则解包出里面的值。

```rust
// examples/ex_onnx_session.rs
let mut session = Session::builder()?.commit_from_file(&path)?;
```

类型不一致时先转换再 `?`。`map_err` 把错误换成另一种，成功值原样通过：

```rust
// src/policy.rs
let session = Session::builder()
    .and_then(|b| b.with_optimization_level(GraphOptimizationLevel::Level3))
    .and_then(|b| b.with_intra_threads(INTRA_THREADS))
    .and_then(|b| b.commit_from_file(path))
    .map_err(|source| PolicyError::Load {
        path: path.to_owned(),
        source,
    })?;
```

`and_then` 在成功时继续调用闭包，闭包自己也返回 `Result`。

可选值的一条链（环境变量解析）：

```rust
// src/main.rs
let failing_reads = std::env::var("MINIDUCK_FAKE_FAILING_READS")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(0);
```

`.ok()` 把 `Result` 收成 `Option`（错误变成 `None`）。`.unwrap_or(0)` 在 `None` 时用 `0`。

其它常见收尾：

- `.unwrap()`：失败就 panic
- `.expect("说明")`：同样 panic，但带一句原因
- `.unwrap_or_else(|| ...)`：失败时调用闭包生产替代值
- `.ok_or_else(|| 错误)`：把 `Option` 变回 `Result`
- `is_ok()` / `is_err()` / `is_some()` / `is_none()`：只问状态，不取出值

`Option` 上还有 `filter`、`checked_mul` 这类返回 `Option` 的运算，失败时整条链变成 `None`，最后 `ok_or_else` 变成错误。见 `alloc_lstm_state` 里对状态长度的检查。

## 12. 闭包与迭代器

闭包是可以捕获周围变量的匿名函数：`|参数| 表达式`。

```rust
// src/main.rs
let policy = Policy::load(&policy_path).map_err(|e| {
    eprintln!("failed to load policy {policy_path}: {e}");
    e
})?;
```

这个闭包捕获了 `policy_path`，接收错误 `e`，打印后原样交回去。块体的最后一个表达式 `e` 就是闭包的返回值。

`|| { ... }` 是没有参数的闭包，`alloc_lstm_state` 用它当造一个张量的小工厂，调用两次得到 `h` 和 `c`。

`async move { ... }` 是异步块，`move` 把捕获的变量所有权带进任务，这样任务可以比当前栈帧活得更久。`spawn` 里的 `stats.clone()` 就是为了把一份 `Arc` 移进任务，外面的 `stats` 还留着。

迭代器是惰性的链式变换，碰到 `collect`、`any`、`all`、`find` 才真正跑：

```rust
// examples/ex_onnx_session.rs
let input_meta: Vec<_> = session
    .inputs()
    .iter()
    .map(|o| (o.name().to_owned(), format!("{:?}", o.dtype())))
    .collect();
```

- `.iter()`：逐个交出 `&元素`
- `.iter_mut()`：交出 `&mut 元素`，测试里用来改 `action` 的每一项
- `.enumerate()`：附上序号 `(索引, 元素)`
- `.map(|v| ...)`：逐个变换
- `.all(|v| v.is_finite())`：全部满足才为真
- `.any(|s| s != &shapes[0])`：有一个满足就为真
- `.find(|o| o.name() == name)`：找到第一个
- `.chain(c)`：把两个迭代器接成一个
- `.collect::<Vec<_>>()`：收成 `Vec`。`Vec<_>` 的 `_` 让编译器推断元素类型；`::<>` 这种写在函数名后面的类型参数叫 turbofish

`std::array::from_fn(|slot| values[joint_of(slot)])` 用闭包按下标生成数组。

`f64::total_cmp` 是一个函数，被当作值传给 `sort_by`，这和闭包同一个用法。

## 13. 字符串、格式化、转换

`String` 拥有一串可增长的 UTF-8 字节。`&str` 是借来的一段字符串。字面量 `"hello"` 的类型是 `&'static str`。

`.as_str()` 从 `String` 借出 `&str`。`.to_owned()` / `.into()` 得到自己拥有的 `String` 或 `PathBuf`。`JSONRPC.into()` 把 `&str` 变成 `String`，因为字段类型是 `String`，而 `From<&str> for String` 存在。

格式化宏：

- `println!` / `eprintln!`：标准输出 / 标准错误
- `format!`：得到一个 `String`
- `write!(f, "...")`：写进 `fmt` 的缓冲区

花括号里可以直接写变量名和表达式：

```rust
// src/main.rs
eprintln!("failed to load policy {policy_path}: {e}");
```

`{hz:.1}` 是保留 1 位小数，`{p99:.2}` 是 2 位。`{}` 走 `Display`，`{:?}` 走 `Debug`。`{mid:?}` 能打印整个数组，因为数组实现了 `Debug`。

原始字符串 `r#" ... "#` 里反斜杠和引号按字面处理，`policy.rs` 用它嵌了一段 Python。

数字字面量：

- `0u32`、`0u64`、`1usize`：类型后缀
- `0.0f32`、`0.0f64`：浮点后缀
- `1e-12`、`1e-6`：科学计数法
- `1_048_576`：下划线只为可读，值仍是 1048576

`as` 是显式数值转换：`imu.gyro[i] as f32`、`tick_in_ramp.min(RAMP_TICKS) as f64`、`(deltas_ms.len() as f64) * 0.99`。它允许截断，所以只出现在项目清楚知道范围的地方。`usize::try_from(...)` 则是可能失败的转换，返回 `Result`。

## 14. 宏

宏在编译期展开成代码，调用时用 `!`。这个仓库没有自己的 `macro_rules!`，用的都是现成宏：

| 宏 | 作用 |
|---|---|
| `println!` / `eprintln!` / `format!` / `write!` | 格式化 |
| `vec![0.0f32; count]` | 建 `Vec` |
| `assert!` / `assert_eq!` / `debug_assert!` / `panic!` | 测试与断言。`debug_assert!` 只在 debug 构建里生效 |
| `json!({ "k": v })` | `serde_json` 按 JSON 形状构造 `Value` |
| `ort::inputs!["obs" => &input]` | 按名字组装推理输入。这里的 `=>` 是宏自己的语法 |
| `env!("CARGO_MANIFEST_DIR")` | 编译期读环境变量，编进二进制 |
| `tokio::select! { ... }` | 同时等几个异步事件 |

`select!` 每个分支是 `模式 = 异步表达式 => 处理`，还可以加前置条件：

```rust
// src/main.rs
_ = push.tick(), if subscribed => {
```

`if subscribed` 为假时这个分支不参与竞争。谁先完成就执行谁，其余分支在这一轮取消。

`examples/ex_select.rs` 里还有模式分支 `Some(msg) = rx.recv()`：收到 `None`（通道关闭）时这个分支不算成功，`select!` 会继续等别的分支。

## 15. 异步语法

`async fn` 把函数变成返回未来的函数，函数体在遇到 `.await` 之前不会往下执行：

```rust
// src/main.rs
async fn serve(
    stream: UnixStream,
    stats: Arc<control::Stats>,
    frame_rx: watch::Receiver<control::FrameSnapshot>,
    boot: Instant,
) {
```

`.await` 把控制权交还运行时，等这个操作完成后把结果拿回来。`listener.accept().await?` 是先等待、再对得到的 `Result` 用 `?`。

`tokio::spawn(async move { ... })` 把一段异步块放到运行时上并发执行。`spawn` 的参数必须是 `'static`，所以捕获的东西要么是拥有所有权的，要么是 `Arc` 这种共享所有权。

`#[tokio::main]` 负责启动运行时并执行 `async fn main`。

## 16. `unsafe`

全项目只有一处，在测试里设置环境变量：

```rust
// src/policy.rs
// SAFETY: 测试进程内设置一次查找路径；ort 首次 API 调用前必须就绪。
unsafe {
    std::env::set_var("ORT_DYLIB_PATH", &so);
}
```

`unsafe { ... }` 表示这块里可以调用未在类型系统里保证安全的操作。`set_var` 在多线程同时读环境变量时可能出问题，所以标准库把它标成 `unsafe`。旁边的 `SAFETY:` 注释是项目给出的理由：测试进程里、库第一次使用前只设一次。

## 17. 测试模块与其它零碎语法

```rust
// src/control.rs
#[cfg(test)]
mod tests {
    use super::*;
```

这是写在同一个文件里的子模块。`#[cfg(test)]` 让它不进正式构建。`use super::*` 能使用父模块的私有项，所以测试可以直接碰 `apply_action`、`HEAD_JOINTS`。

其它在代码里反复出现的点：

- 变量遮蔽：`let path = path.as_ref();` 用同名新绑定盖住参数。之后的 `path` 是 `&Path`。
- 块表达式：`let copy_result = { let state = ...; copy_lstm_outputs(...) };` 花括号的值是最后一行。
- 提前声明类型：`let req: Request = match ...` 在 `let` 上写类型，帮助推断 `serde_json::from_str` 要解析成什么。
- 方法的类型参数：`try_extract_tensor::<f32>()`、`channel::<&str>(1)`。尖括号是 turbofish，因为后面有 `()`，不写 `::` 的话 `<` 会被当成小于号。
- 索引：`out[i]`、`outputs["h_out"]`、`output_meta[0].0`（元组字段）。索引失败对切片和 `Vec` 会 panic；对这个 `outputs[name]` 是库的 `Index` 实现。
- 比较与逻辑：`&&`、`||`、`!`、`==`、`!=`、`>`。`==` 来自 `PartialEq`。
- 语句与表达式：`tick += 1;` 带分号，是语句。`match`、`if`、块可以不带分号，作为值。
- 发散：`std::process::exit(1)` 不会返回。写在 `unwrap_or_else` 的闭包里时，成功路径的类型仍由另一侧决定。

## 18. 一份对照

`control.rs` 的 `apply_action` 几乎把日常语法压在一个函数里：

```rust
// src/control.rs
pub fn apply_action(
    action: &[f32; ACTION_LEN],
    previous: Option<&[f64; NUM_JOINTS]>,
) -> [f64; NUM_JOINTS] {
    let offsets = Observation::scatter_action(action);
    let mut targets = [0.0; NUM_JOINTS];
    for j in 0..NUM_JOINTS {
        targets[j] = DEFAULT_POSITION[j] + ACTION_SCALE * offsets[j];
    }

    if let Some(previous) = previous {
        for joint in HEAD_JOINTS {
            targets[joint] =
                HEAD_LOWPASS * targets[joint] + (1.0 - HEAD_LOWPASS) * previous[joint];
        }
        // ...
    }
    // ...
    targets
}
```

这里有：公开函数、定长数组引用、`Option`、关联函数调用、`mut` 绑定、重复数组、`for` 与区间、索引、`if let` 与同名遮蔽、以及函数体最后一个表达式就是返回值（`targets` 后面没有分号，也没有写 `return`）。
