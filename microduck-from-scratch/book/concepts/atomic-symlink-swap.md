# 原子换链接：rename 与 fsync 目录

## 一句话

把 `current` 这个 symlink 从指向旧版本换成指向新版本，正确做法是：先建一个指向新版本的临时 symlink，再用 `rename(2)` 一把盖到 `current` 上，最后 fsync 它所在的目录。读者在任何时刻要么看到旧指向、要么看到新指向，没有中间态，断电也丢不掉这次切换。

## 为什么需要

更新系统的「当前版本」就是一个 symlink：`install/current -> releases/2.0.0`。换版本 = 换这个链接的指向。直觉做法是「先删掉旧链接，再建一个新链接」——但这两步之间有一段窗口 `current` **不存在**，恰好此刻来读的消费者（比如正要启动的 daemon、正在读配置的其他进程）会撞上 FileNotFoundError。

`rename(2)` 在同一个目录内是原子的：内核要么还没改目录项、要么已经改完，没有「改了一半」。所以「先建临时链接、再 rename 覆盖」没有任何空窗。

原子性只解决「读者看不到中间态」，不解决「断电后这次切换还在不在」。write/rename 先落在内核页缓存里，断电时没写盘的目录项就丢了——机器重启后 `current` 可能又指回旧版。所以 rename 之后还要 `fsync` **父目录**：文件内容要 fsync 文件本身，而「这个目录里有哪个名字指向谁」这条目录项本身也是数据，要 fsync 目录才落盘。这是新手最容易漏的一步：文件 fsync 了，目录项没 fsync，崩溃后文件名凭空消失或指回旧值。

## 最小示例

`examples/atomic_symlink_swap.py` 起两个线程：一个读者不停经 `current` 读指向，一个写者换 2000 次指向。先删再建 vs 临时链接+rename 各跑一遍。本机实测：

```text
先删再建        : 读者读 6850 次，撞见 current 不存在 3383 次
临时链接+rename : 读者读 10716 次，撞见 current 不存在 0 次
```

近一半读取撞上空窗 vs 零次。（次数随机器调度浮动，结论不变。）

## 素材

- `man 2 rename`：「newpath 已存在则原子地替换它」。
- `man 2 fsync` 与 [PostgreSQL 关于 fsync 目录的说明](https://www.postgresql.org/docs/current/wal-reliability.html)（数据库圈把这件事写得最透：目录项也是数据）。
- 原版注释把「原子」与「持久」两件事分开讲：`reference/updater/src/store.rs:160-203`（rename 保证前者，末尾 fsync 父目录保证后者，注释明写 "Durability, not just atomicity"）。

## 回到项目

`src/updater/store.rs:20-31`（`set_current`）：先清上次崩溃残留的 `.current.tmp`（`store.rs:24`，原版同 `store.rs:181`）→ 建指向 `releases/<ver>` 的**相对**临时链接（`store.rs:26`，相对路径让整个 root 搬走后链接仍有效）→ rename 覆盖（`store.rs:27-29`）→ fsync 父目录（`store.rs:30`）。

同一对「原子+持久」还出现在两处：`install_release` 解包完成后 rename 进 `releases/` 并 fsync（`store.rs:62-63`）；pending.json 写盘后 `sync_all`（`src/updater/engine.rs:421-425`）——pending 是文件不是链接，fsync 文件本身就够。

## 自测

`set_current` 里如果把 `fsync_dir` 删掉，功能测试（读 readlink 断言指向）还能过吗？什么场景下会暴露这个删除？
