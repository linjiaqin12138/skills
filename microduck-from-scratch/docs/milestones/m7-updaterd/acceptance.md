# M7 迷你 updaterd — 验收档案

## 交付物

- `src/updater/`（新建十个模块）：
  - `paths.rs`：目录布局唯一权威——`<root>/source/`（`<ver>.manifest.json`+`<ver>.tar`）、`<root>/install/releases/<ver>/`、`.staging-<ver>`（releases 的邻居，保证 rename 同文件系统）、`install/current` symlink、`<root>/state/{pending.json, update-log.jsonl, update.lock}`。默认根 `/tmp/miniduck-update/`，`MINIDUCK_UPDATER_ROOT` 覆盖（验收隔离）。
  - `manifest.rs`：manifest 解析+形状校验（version 非空 / artifact 必须裸文件名 / sha256 必须 64 位小写 hex，任何不符即拒绝）+ artifact sha256 比对（不符 → Err，调用方保证无副作用）。
  - `store.rs`：`set_current` 原子换指向（清残留 tmp → 相对 symlink `releases/<ver>` → rename 覆盖 → fsync 父目录，对齐 `reference/updater/src/store.rs:160-203`）；`install_release` 解包进 staging 后 rename 落位 + fsync，失败清 staging；已安装版本幂等跳过解包（回滚后再升回同版本合法）；`cleanup_staging` 启动清残留；`current_version`=readlink。
  - `journal.rs`：update-log.jsonl 追加 + sync_data；Outcome=Committed/RolledBack/Rejected，每行带 peer_uid（崩溃恢复等非 RPC 路径为 null，放行/拒绝都记账）；`previous_for`：previous 从最后一行匹配 current 的 Committed 推导，不另开状态文件。
  - `gate.rs`：健康门——预算内每 500ms（对齐 `reference/updater/src/engine.rs:69`）hello+robot.health 轮询 miniduckd socket，healthy 立即放行，预算耗尽=失败；单次尝试硬超时 1s。
  - `lock.rs`：flock(LOCK_EX|LOCK_NB) 单飞锁（libc 直调）；选 flock 不选 O_EXCL——kill -9 后内核自动放锁，O_EXCL 文件残留会把重启后的 apply 全卡成 Busy。
  - `peer.rs`：SO_PEERCRED-lite 放行判定纯函数（uid==0 || socket owner，D40）。
  - `engine.rs`：apply 七阶段编排（Verifying→Extracting→arm pending→kill/swap/spawn→HealthGate→Committing/RollingBack，Phase 枚举为原版 proto 子集）；pending 在 swap 前 arm + sync_all；`MINIDUCK_UPDATER_EXIT_AFTER_SWAP=1` 故障注入（swap 落盘+arm+spawn 后 exit(1)，kill -9 最坏窗口）；`recover()` 启动先跑（清 staging → boots+=1 写回 → 收割孤儿 → 从 current 起 daemon → pending 驱动重跑门裁决 confirm/rollback）；`MAX_BOOT_ATTEMPTS=2`（对齐 `reference/updater/src/engine.rs:38`）；进程 supervisor（D37：spawn `current/bin/miniduckd` + setsid 脱离会话、SIGTERM→2s→SIGKILL、/proc comm 匹配 `miniduckd*` 收割孤儿并跳过僵尸）；rollback_to 换回 previous 后门再确认旧版 healthy；confirm 与两条回滚路径都 clear_pending。
  - `ipc.rs`：update.check/status/log/apply/rollback（+hello）方法面，复用 lib.rs NDJSON JSON-RPC 帧；mutating 两方法先 SO_PEERCRED-lite 门控（peer_cred 失败也拒绝）再拿单飞锁；只读路径不锁 engine；错误码 -32001 Busy / -32002 PermissionDenied / -32003 Rejected / -32004 Failed。
- `src/bin/mini-updaterd.rs`（新建）：建目录 → `recover()` 先于 bind → bind `/tmp/mini-updaterd.sock` 0660（D8 同款第一道门）→ serve。env：`MINIDUCK_UPDATER_GATE_MS`（默认 30s）、`MINIDUCK_UPDATER_SOCK`、`MINIDUCK_DAEMON_CWD`。
- `src/bin/mini-duckctl.rs`：+update 子命令组（check/status/log [N]/apply <ver>/rollback），解析第一个参数即分流到 updaterd socket（`MINIDUCK_UPDATER_SOCK` 可覆盖）。
- `Cargo.toml`：+sha2 0.10（pure-Rust）、+libc 0.2（flock/kill/setsid）、+`[[bin]] mini-updaterd`。`src/lib.rs`：+`pub mod updater`。
- `scripts/accept-m7.sh`（新建）：五场景 39 断言，全程 `MINIDUCK_UPDATER_ROOT=/tmp/m7-demo` 隔离；投毒版 = wrapper 脚本 `export MINIDUCK_POLICY=/nonexistent.onnx` 再 exec 真二进制（D19 既有行为，miniduckd 零改动）。
- **miniduckd 行为零改动**；robot.* 面不变。

## 验收实测

`docker compose exec rust cargo build`：0 警告。`docker compose exec rust cargo test`：

```text
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

构成：M6 的 49 条 + 新增 11 条（store 3：原子换指向/残留 tmp 不挡 swap/staging 清理；manifest 3：良构/畸形拒绝/sha256 不符；journal 1：previous 推导；lock 1：Busy+放锁再拿；peer 1：root/owner/旁观者；engine 2：恢复裁决真值表/pending 读写往返）。

`docker compose exec rust bash scripts/accept-m7.sh`（2026-09-29 复跑实测完整输出）：

```text
===== 0. 构建 + 单测 =====
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
release 1.0.0/2.0.0/2.0.1/3.0.0-bad 就绪，出厂 current=1.0.0

===== 场景 A：开心路径 =====
== A1. updaterd 自起 daemon，mini-duckctl state 通 ==
PASS: A1 daemon 自起且 state 有帧
== A2. update.check 列出可用版本与 current ==
available=['1.0.0', '2.0.0', '2.0.1', '3.0.0-bad'] current=1.0.0
PASS: A2 check
== A3. apply 2.0.0 → committed ==
committed 1.0.0 -> 2.0.0
PASS: A3 apply 2.0.0
PASS: A3 current->2.0.0
PASS: A3 pending 已清
== A4. status / log / 新 daemon healthy ==
PASS: A4 status idle/current=2.0.0/previous=1.0.0
log 末行: committed 1.0.0->2.0.0 peer_uid=1000
PASS: A4 log Committed（带 peer_uid）
PASS: A4 新 daemon healthy
== A5. 手动 rollback 回 1.0.0，再 apply 回 2.0.0（给 B 铺垫）==
PASS: A5 rollback RPC
PASS: A5 current->1.0.0
PASS: A5 重回 2.0.0

===== 场景 B：投毒回滚 =====
门否决: health gate rejected 3.0.0-bad: unhealthy: ["policy unavailable: reading /nonexi
PASS: B1 门否决自动回滚
PASS: B2 current 回到 2.0.0
PASS: B2b pending 已清
PASS: B3 daemon 恢复 healthy
RolledBack 行: health gate rejected 3.0.0-bad: unhealthy: ["policy unavailable: reading /nonexi
PASS: B4 log RolledBack(2.0.0→3.0.0-bad)

===== 场景 C：篡改拒绝 =====
拒绝: Rejected: sha256 mismatch for 3.0.0-bad.tar: manifest says 187f6307b7e
PASS: C1 sha256 不符被拒
PASS: C2 current 没动
PASS: C3 releases/ 无残留
PASS: C4 staging 已清
PASS: C5 log Rejected

===== 场景 D：并发单飞 =====
第二个 apply 被拒: Busy: another update is in progress
PASS: D1 并发 apply 拿 Busy
PASS: D2 第一个 apply 正常完成

===== 场景 E：崩溃恢复 =====
== E0. kill -9 updaterd 不杀 daemon ==
PASS: E0 kill -9 updaterd 后 daemon 仍在跑
== E1. EXIT_AFTER_SWAP apply 健康 2.0.1 → 重启 confirm ==
PASS: E1 updaterd swap 后 exit(1)
PASS: E1 新 daemon(2.0.1) 仍在跑
pending: {'version': '2.0.1', 'previous': '2.0.0', 'boots': 0}
PASS: E1 pending 已 arm(boots=0)
PASS: E1 current->2.0.1
PASS: E1 重启后 pending 已 confirm
PASS: E1 current 保持 2.0.1
恢复记账: recovered after crash: healthy
PASS: E1 log 恢复后 Committed
== E2. EXIT_AFTER_SWAP apply 投毒版 → 重启裁决回滚 ==
PASS: E2 updaterd swap 后 exit(1)
PASS: E2 投毒 daemon 报病（D19）
PASS: E2 pending(3.0.0-bad)
PASS: E2 重启后 pending 已清
PASS: E2 current 回滚到 2.0.1
恢复回滚: recovered trial still unhealthy: unhealthy: ["policy unavailable: read
PASS: E2 log RolledBack(2.0.1→3.0.0-bad)
PASS: E2 回滚后 daemon healthy

===== 收尾：cargo test 复核 =====
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
PASS: cargo test 全绿

===== M7 汇总：PASS=39 FAIL=0 =====
```

**回归**：miniduckd 零改动，M6 的 49 条单测在 60 总数里原样全绿（控制链路行为回归由单测覆盖；本章未重跑 accept-m5/m6 的 sim 场景，robot.* 面无改动路径）。

**实现期踩坑记录**（已修复并变成回归断言）：① 验收脚本同步 bug——kill -9 后旧 socket 文件残留骗过启动等待循环，修法=启动前 `rm -f`（`scripts/accept-m7.sh:101-103`）；② 回滚路径漏清 pending.json（confirm 清了、rollback 没清）→ 已结束 trial 被下次启动重复裁决，两条回滚路径都补 `clear_pending`（`src/updater/engine.rs:239`、`:302-304`），场景 B2b 是回归断言；③ install 幂等——原实现拒绝重装已安装版本，回滚后再升回同版本被拒，改为已安装则跳过解包（`src/updater/store.rs:41-46`），场景 A5 往返断言。

## 涉及代码文件清单

| 文件 | 变动 |
|---|---|
| `src/updater/mod.rs` | 新建：模块清单 + RPC 错误码 |
| `src/updater/paths.rs` | 新建：目录布局（source/install/state） |
| `src/updater/manifest.rs` | 新建：manifest 校验 + sha256（3 条单测） |
| `src/updater/store.rs` | 新建：staging/rename/原子 swap/fsync/幂等（3 条单测） |
| `src/updater/journal.rs` | 新建：流水账 + previous 推导（1 条单测） |
| `src/updater/gate.rs` | 新建：健康门轮询 |
| `src/updater/lock.rs` | 新建：flock 单飞锁（1 条单测） |
| `src/updater/peer.rs` | 新建：SO_PEERCRED-lite（1 条单测） |
| `src/updater/engine.rs` | 新建：编排+supervisor+崩溃恢复（2 条单测） |
| `src/updater/ipc.rs` | 新建：update.* 方法面+门控+锁 |
| `src/bin/mini-updaterd.rs` | 新建：daemon 入口 |
| `src/bin/mini-duckctl.rs` | +update 子命令组（分流连 updaterd） |
| `src/lib.rs` | +`pub mod updater` |
| `Cargo.toml` | +sha2/+libc/+[[bin]] mini-updaterd |
| `scripts/accept-m7.sh` | 新建：五场景 39 断言 |
| `examples/atomic_symlink_swap.py` | 新建：教程概念演示（rename 原子性） |
| `examples/boot_counter_order.py` | 新建：教程概念演示（arm 顺序） |
| `docs/concepts/atomic-symlink-swap.md` | 新建：概念深读（原子 rename + fsync 目录） |
| `docs/milestones/m7-updaterd/{tutorial,acceptance}.md` | 本档案与教程章节 |
| `docs/milestones/m7-updaterd/{arch,arch-diff}.d2` | 架构图源码（渲染由主 agent 另派） |

## 偏差变动

- **移交承接**：D6（无 systemd 部署、无 minisign 验签）本章落地后拆为 D37/D38 两半承接。
- **新开**：D37（无 systemd：updaterd 直接 spawn/kill+setsid 替代 systemctl restart，无 golden/boot-check 兜底）、D38（无 minisign 验签，sha256-only）、D39（LocalDir 单一 source，无网络发现/channel/定时器；未压缩 .tar 而非 .tar.zst）、D40（SO_PEERCRED 缩为 uid==0||owner，无 allow_uids/allow_gids）、D41（无 hooks/orphan 检查/transcript/Degraded 裁决/subscribe 推送/self-update；Phase 取 proto 子集）。全部预定 M8 裁决。
- **对齐原版未计偏差**：原子 symlink swap+fsync 父目录、boot counter 先 arm 后 swap、MAX_BOOT_ATTEMPTS=2、健康门 500ms 轮询+timeout 即失败、mutating 单点门控。
