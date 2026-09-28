//! 安全层（M5）：唯一持有总线写句柄的地方。
//!
//! Safety 拥有并私有化 io——上面的策略、控制循环、RPC 层都拿不到
//! RobotIo，于是"没有任何东西能绕过安全规则碰电机"由借用检查器强制，
//! 而不是靠人人记得守规矩。这正是原版 duck-control/src/safety.rs 的
//! 核心论点：只在出事时才跑的代码最容易悄悄坏掉，所以让坏状态不可表示。
//!
//! 三条无条件规则：
//!   - 非有限目标**拒绝**（不是夹紧）；
//!   - 越界目标**夹紧**到执行器行程；
//!   - 意图失联超 deadman，速度清零。
//!
//! 跌倒判定是**报告**不是门控：fallen 每拍更新、对外发布，但不抢占任何
//! 写入——倒下的机器人照样按调用方给的目标和增益被驱动。摔倒后怎么办
//! （卸力软倒）是控制层的决定，它通过同一个 apply 以普通目标+普通增益
//! 下达，没有特权通道——这就是写句柄必须收在这里的原因。

use std::time::Duration;

use crate::io::{IoError, JointTargets, RobotIo, Sensors};
use crate::model::NUM_JOINTS;
use crate::obs::Command;

/// XL330 的行程：居中一圈，±π。这是**执行器**的行程，不是各关节的解剖
/// 限位（真机关节限位在 MJCF 里）。所以它拦得住策略吐 NaN/离谱动作尺度/
/// 垃圾张量，拦不住"把关节开到机械上不明智的位置"——如实记录，免得
/// 看起来像逐关节保护、实际不是。
pub const ACTUATOR_MIN: f64 = -std::f64::consts::PI;
pub const ACTUATOR_MAX: f64 = std::f64::consts::PI;

/// 数值全部来自原型 alpha 的默认配置（reference safety.rs 的 Default）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafetyConfig {
    /// 投影重力 z 高于此值算"在倒"。直立约 -1.0，侧躺接近 0。
    pub fall_gravity_z: f64,
    /// 持续这么久才算跌倒。去抖：一次扎实的落脚不该判成跌倒。
    pub fall_debounce: Duration,
    /// 意图失联超过此时长，速度清零。
    pub deadman: Duration,
    /// 正常驱动时的增益。
    pub gain_running: u16,
    /// 软倒增益：不再顶着地板较劲。本层不使用它——控制循环在 Limp
    /// 阶段通过 apply 主动要这个值——但它是安全数字，和其他安全数字住一起。
    pub gain_limp: u16,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            fall_gravity_z: -0.5,
            fall_debounce: Duration::from_millis(200),
            deadman: Duration::from_millis(500),
            gain_running: 200,
            gain_limp: 50,
        }
    }
}

/// 一次命令没有按原样执行的原因。上报给调用方，而不是让它看着机器人
/// "不听话"却无从知道为什么。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    /// 意图失联，速度被清零。
    Deadman,
    /// 目标超出执行器行程。
    Range,
    /// 目标是 NaN 或无穷。
    NotFinite,
}

/// 安全层对一拍目标的处置结果。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Applied {
    pub limits: Vec<Limit>,
}

impl Applied {
    pub fn limited_by(&self, limit: Limit) -> bool {
        self.limits.contains(&limit)
    }
}

/// 持续触发时每多少拍再报一次。
///
/// 第一次触发必报——只夹紧一次也是新闻。之后每 50 拍一行：50Hz 下即
/// 每秒一行，既让人知道故障还在，又不至于每拍一行把日志刷成噪音。
const LIMIT_LOG_EVERY: u64 = 50;

/// 三条规则各自的"连续触发"计数，日志限流的依据。
///
/// 三个计数器而非一个：三条规则是独立事件，deadman 触发一秒不代表
/// 有关节在夹紧，共享计数会让触发最勤的那条饿死另外两条的日志。
///
/// 每个计数被"走到该检查且未触发"的拍清零，所以读数是"这次持续多久"
/// 而不是"历史上总共多少次"。非有限拒绝的拍在夹紧之前 return，故意
/// **不清零 range**：否则 NaN 和越界交替出现时两条规则每隔一拍就各自
/// 开启新 run、每拍都刷一行——正是计数要防的东西。
#[derive(Debug, Default, PartialEq)]
struct LimitRuns {
    deadman: u64,
    range: u64,
    not_finite: u64,
}

impl LimitRuns {
    /// run 到第 n 拍是否值得写一行。收在这里而不是各调用点，三条规则
    /// 才不会各自长成三种"够勤"的标准。
    fn worth_logging(n: u64) -> bool {
        n == 1 || n % LIMIT_LOG_EVERY == 0
    }
}

pub struct Safety<T: RobotIo> {
    io: T,
    config: SafetyConfig,
    /// 重力越过阈值已持续多久。任何直立样本清零。
    falling_for: Duration,
    fallen: bool,
    /// 上次写入的增益：值不变不重写——50Hz 下每拍 15 次 gain 写
    /// 会挤爆控制循环要用的总线。
    gain: Option<u16>,
    runs: LimitRuns,
    /// 是否收到过死人开关有效期内的意图。"从没司机"不是新闻，
    /// "司机失联"才是——否则从没被驾驶过的台式机器人从启动后半秒
    /// 开始每秒刷一行，一天八万行。
    deadman_armed: bool,
}

impl<T: RobotIo> Safety<T> {
    pub fn new(io: T, config: SafetyConfig) -> Self {
        Self {
            io,
            config,
            falling_for: Duration::ZERO,
            fallen: false,
            gain: None,
            runs: LimitRuns::default(),
            deadman_armed: false,
        }
    }

    /// 读透传：读不威胁"写句柄唯一"这条不变量。
    pub fn read(&mut self) -> Result<Sensors, IoError> {
        self.io.read()
    }

    pub fn fallen(&self) -> bool {
        self.fallen
    }

    /// 舵机出力开关。经由此处是因为 Safety 拥有唯一写句柄。
    /// 控制循环只在 enable/disable 边沿调它，绝不每拍调，也绝不在
    /// 进程启动时调——舵机 RAM 里的 torque 跨进程存活，被 supervisor
    /// 重启的 daemon 必须让站着的机器人继续站着。
    pub fn set_torque(&mut self, on: bool) -> Result<(), IoError> {
        eprintln!("safety: torque {on}");
        self.io.set_torque(on)
    }

    /// 当前实际运行的增益（上次写入值），未必是调用方最近一次想要的。
    pub fn gain(&self) -> Option<u16> {
        self.gain
    }

    /// 每拍 read 成功后、apply 之前调用，更新跌倒判定。
    ///
    /// 倒下方向去抖、起身方向立即：持续 fall_debounce 才算倒下，而任何
    /// 一个直立样本立刻清零累加器并解除 fallen——不对称是故意的：误判
    /// "倒了"的代价（卸力摔向地板）远高于"多躺一拍"，起身方向拖不得。
    /// 倒下方向不去抖的话，一次扎实落脚的冲击就读成跌倒。
    pub fn observe(&mut self, sensors: &Sensors, dt: Duration) {
        // 姿态滤波没收敛不许投票。SFLP 滤波器要几秒样本四元数才有意义，
        // 此前投影重力是滤波器半路决定的任意值，读作"越过跌倒阈值"，读作
        // "侧躺"——200ms 后一台直立在台架上的机器人被锁成 fallen，增益
        // 被写成 50，几秒后自行恢复却留下低增益，可观测状态里没有任何东西
        // 解释发生了什么（原版实测回归）。两个方向的安全默认都是保持上一拍
        // 判定：启动时是"没倒"（台架上机器人的真实状态），运行中滤波器
        // 中途掉线则倒下的机器人保持倒下。
        if !self.io.imu_ready() {
            return;
        }

        let was_fallen = self.fallen;
        let down = sensors.imu.gravity[2] > self.config.fall_gravity_z;
        if down {
            self.falling_for = self.falling_for.saturating_add(dt);
            if self.falling_for >= self.config.fall_debounce {
                self.fallen = true;
            }
        } else {
            self.falling_for = Duration::ZERO;
            self.fallen = false;
        }

        // 判定翻转必报且不限流：这是事件，不是状态复述。限流丢掉的是
        // "什么时候倒的/什么时候起来的"。
        if self.fallen != was_fallen {
            eprintln!(
                "safety: fall verdict changed: fallen={} gravity_z={:.3}",
                self.fallen, sensors.imu.gravity[2]
            );
        }
    }

    /// 死人开关：意图失联超 deadman，速度清零——失联让机器人**站住**，
    /// 不是瘫软（stop is not limp：站立是双足的安全态）。只清 twist，
    /// 不动 head/body：stale 的头姿无害，stale 的速度会撞墙。
    ///
    /// 每拍都调（armed 语义防日志噪音，见 deadman_armed 字段注释）。
    pub fn gate(&mut self, command: Command, intent_age: Duration) -> (Command, Option<Limit>) {
        if intent_age <= self.config.deadman {
            self.runs.deadman = 0;
            self.deadman_armed = true;
            return (command, None);
        }

        self.runs.deadman += 1;
        if self.deadman_armed && LimitRuns::worth_logging(self.runs.deadman) {
            eprintln!(
                "safety: intents stale {}ms > deadman {}ms (run {}) — twist zeroed",
                intent_age.as_millis(),
                self.config.deadman.as_millis(),
                self.runs.deadman
            );
        }

        let mut stopped = command;
        stopped.twist = [0.0; 3];
        (stopped, Some(Limit::Deadman))
    }

    /// 通往电机的唯一路径。
    ///
    /// `hold` 是策略不许驱动时写什么——通常是机器人当前所在姿态。
    /// `running_gain` 是调用方想要的增益（站立/行走/软倒各不相同），
    /// 那是控制决策不是安全决策，所以由参数传入而不是在这里揣测。
    pub fn apply(
        &mut self,
        targets: [f64; NUM_JOINTS],
        hold: [f64; NUM_JOINTS],
        running_gain: u16,
    ) -> Result<Applied, IoError> {
        let mut applied = Applied::default();

        // 注意这里**没有**跌倒门控：倒下不阻止调用方驱动（判定是报告，
        // 见模块文档）。摔倒后怎么办由上层决定，以普通目标和普通增益到达。
        self.set_gain(running_gain)?;

        // 非有限拒绝，不是夹紧：NaN 夹紧会产出看似合理的边界角，
        // 机器人会猛冲到限位——远不如保持不动。
        if targets.iter().any(|v| !v.is_finite()) {
            // 在夹紧之前 return，故意让 range run 站着不动——见 LimitRuns。
            self.runs.not_finite += 1;
            if LimitRuns::worth_logging(self.runs.not_finite) {
                eprintln!(
                    "safety: targets refused: not finite (run {}) — writing hold",
                    self.runs.not_finite
                );
            }
            applied.limits.push(Limit::NotFinite);
            self.io.write(&JointTargets { positions: hold })?;
            return Ok(applied);
        }
        self.runs.not_finite = 0;

        let mut safe = targets;
        let mut clamped_joints = 0u32;
        let mut first_clamped = None;
        for (joint, value) in safe.iter_mut().enumerate() {
            let clamped = value.clamp(ACTUATOR_MIN, ACTUATOR_MAX);
            if clamped != *value {
                clamped_joints += 1;
                first_clamped.get_or_insert(joint);
                *value = clamped;
            }
        }

        if let Some(first) = first_clamped {
            self.runs.range += 1;
            applied.limits.push(Limit::Range);
            if LimitRuns::worth_logging(self.runs.range) {
                eprintln!(
                    "safety: {clamped_joints} joint targets clamped to ±π (first joint {first}, run {})",
                    self.runs.range
                );
            }
        } else {
            self.runs.range = 0;
        }

        self.io.write(&JointTargets { positions: safe })?;
        Ok(applied)
    }

    fn set_gain(&mut self, kp: u16) -> Result<(), IoError> {
        if self.gain == Some(kp) {
            return Ok(());
        }
        self.io.set_gain(kp)?;
        self.gain = Some(kp);
        Ok(())
    }

    /// 借出被包装的 io。**仅测试可用**，故意不 pub：生产代码拿到它，
    /// "写句柄唯一"就成了空话。
    #[cfg(test)]
    fn io(&self) -> &T {
        &self.io
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::FakeIo;
    use crate::model::DEFAULT_POSITION;

    fn upright() -> Sensors {
        Sensors::default() // gravity 默认 [0,0,-1]
    }

    fn on_its_side() -> Sensors {
        let mut s = Sensors::default();
        s.imu.gravity = [-1.0, 0.0, 0.0];
        s
    }

    fn safety() -> Safety<FakeIo> {
        Safety::new(FakeIo::new(), SafetyConfig::default())
    }

    fn gain() -> u16 {
        SafetyConfig::default().gain_running
    }

    fn over_travel() -> [f64; NUM_JOINTS] {
        [ACTUATOR_MAX + 1.0; NUM_JOINTS]
    }

    /// 一次扎实的落脚会让重力短暂越线。把这当跌倒会在步态中途卸力——
    /// 这本身就是制造跌倒的方式。
    #[test]
    fn brief_tilt_is_not_a_fall_and_upright_resets() {
        let mut s = safety();
        s.observe(&on_its_side(), Duration::from_millis(100));
        assert!(!s.fallen(), "100ms < 200ms 去抖");
        s.observe(&upright(), Duration::from_millis(20));
        s.observe(&on_its_side(), Duration::from_millis(100));
        assert!(!s.fallen(), "直立样本必须清零累加器");
    }

    #[test]
    fn sustained_tilt_is_a_fall() {
        let mut s = safety();
        for _ in 0..11 {
            s.observe(&on_its_side(), Duration::from_millis(20));
        }
        assert!(s.fallen());
    }

    /// 血泪回归（两个方向都测）：未收敛的滤波器不许判跌——重力再歪也不行。
    #[test]
    fn unconverged_imu_never_declares_a_fall() {
        let mut io = FakeIo::new();
        io.imu_ready = false;
        let mut s = Safety::new(io, SafetyConfig::default());

        let mut sensors = Sensors::default();
        sensors.imu.gravity = [0.0, 0.0, 0.0]; // 未收敛滤波器/侧躺都会报出这种值
        for _ in 0..50 {
            s.observe(&sensors, Duration::from_millis(20));
        }
        assert!(!s.fallen(), "滤波器没收敛时不得锁 fallen");
    }

    /// 收敛之后，同样的样本必须被相信——否则上面的护栏等于关掉了跌倒检测。
    #[test]
    fn converged_imu_still_detects_a_fall() {
        let mut io = FakeIo::new();
        io.imu_ready = false;
        let mut s = Safety::new(io, SafetyConfig::default());

        let mut sensors = Sensors::default();
        sensors.imu.gravity = [0.0, 0.0, 0.0];
        for _ in 0..50 {
            s.observe(&sensors, Duration::from_millis(20));
        }
        assert!(!s.fallen());

        s.io.imu_ready = true;
        for _ in 0..11 {
            s.observe(&sensors, Duration::from_millis(20));
        }
        assert!(s.fallen(), "收敛后同样本必须判跌");
    }

    /// 跌倒不抢占调用方：fallen 时 apply 照写调用方的目标和增益。
    /// 这是软倒能完全建在本层之上的契约——它通过同一个 apply "请求"
    /// 低增益，没有特权也没有后门。
    #[test]
    fn fall_does_not_preempt_the_caller() {
        let mut s = safety();
        for _ in 0..11 {
            s.observe(&on_its_side(), Duration::from_millis(20));
        }
        assert!(s.fallen(), "判定仍在跟踪");

        let mut wanted = DEFAULT_POSITION;
        wanted[0] = 0.9;
        let applied = s.apply(wanted, DEFAULT_POSITION, gain()).unwrap();
        assert!(applied.limits.is_empty(), "{:?}", applied.limits);
        assert_eq!(
            s.read().unwrap().positions,
            wanted,
            "调用方继续驱动倒下的机器人"
        );
        assert_eq!(s.io().gain, Some(gain()));
    }

    /// 调用方主动要软倒增益，直接透传——控制循环 Limp 阶段就是这么做的。
    #[test]
    fn caller_asking_for_limp_gain_gets_it() {
        let mut s = safety();
        let limp = SafetyConfig::default().gain_limp;
        s.apply(DEFAULT_POSITION, DEFAULT_POSITION, limp).unwrap();
        assert_eq!(s.io().gain, Some(limp));
        assert_eq!(s.gain(), Some(limp), "并如实报告正在运行的增益");
    }

    /// NaN 拒绝（不写 NaN，写 hold 保持现状），不是夹紧。
    #[test]
    fn non_finite_target_is_refused_not_clamped() {
        let mut s = safety();
        let mut poisoned = DEFAULT_POSITION;
        poisoned[3] = f64::NAN;
        let applied = s.apply(poisoned, DEFAULT_POSITION, gain()).unwrap();
        assert!(applied.limited_by(Limit::NotFinite));
        assert!(!applied.limited_by(Limit::Range), "不许变成夹紧");
        assert_eq!(s.read().unwrap().positions, DEFAULT_POSITION, "写的是 hold");
    }

    /// 越界夹紧到 ±π 并上报 Range——调用方有权知道命令被改过。
    #[test]
    fn out_of_range_targets_are_clamped_and_reported() {
        let mut s = safety();
        let mut wild = DEFAULT_POSITION;
        wild[2] = 100.0;
        wild[7] = -100.0;
        let applied = s.apply(wild, DEFAULT_POSITION, gain()).unwrap();
        assert!(applied.limited_by(Limit::Range));
        let written = s.read().unwrap().positions;
        assert_eq!(written[2], ACTUATOR_MAX);
        assert_eq!(written[7], ACTUATOR_MIN);
    }

    /// 普通目标必须原样通过——否则夹紧在悄悄破坏正常运行，
    /// 其他所有测试都白做。
    #[test]
    fn ordinary_target_passes_through_unchanged() {
        let mut s = safety();
        let applied = s.apply(DEFAULT_POSITION, DEFAULT_POSITION, gain()).unwrap();
        assert!(applied.limits.is_empty(), "{:?}", applied.limits);
        assert_eq!(s.read().unwrap().positions, DEFAULT_POSITION);
        assert_eq!(s.io().gain, Some(gain()));
    }

    /// 增益只在变化时写总线。50Hz 下每拍重写是每秒 750 次多余总线写。
    #[test]
    fn gain_is_written_only_when_it_changes() {
        let mut s = safety();
        for _ in 0..3 {
            s.apply(DEFAULT_POSITION, DEFAULT_POSITION, gain()).unwrap();
        }
        assert_eq!(s.io().gain_writes, 1, "三次 apply 只写一次 gain");
        assert_eq!(s.io().writes, 3, "位置照写三次");
        s.apply(DEFAULT_POSITION, DEFAULT_POSITION, 100).unwrap();
        assert_eq!(s.io().gain_writes, 2, "变了的增益立刻写下去");
    }

    /// deadman 只清 twist，不动 head：stale 头姿无害，stale 速度撞墙。
    #[test]
    fn deadman_zeroes_twist_only() {
        let mut s = safety();
        let command = Command {
            twist: [0.5, 0.0, 0.3],
            head: [0.1, 0.2, 0.3, 0.4],
            ..Command::default()
        };
        let (fresh, limit) = s.gate(command, Duration::from_millis(100));
        assert_eq!(fresh, command, "新鲜意图原样通过");
        assert!(limit.is_none());

        let (stale, limit) = s.gate(command, Duration::from_secs(5));
        assert_eq!(stale.twist, [0.0; 3], "速度必须停");
        assert_eq!(stale.head, command.head, "头姿 stale 无害，不动");
        assert_eq!(limit, Some(Limit::Deadman));
    }

    /// 从没被驾驶过的机器人不报 deadman（armed 语义）：
    /// "从没司机"不是新闻，"司机失联"才是。
    #[test]
    fn never_driven_robot_reports_no_deadman() {
        let mut s = safety();
        for _ in 0..200 {
            // 不 armed，日志判定内部为否；这里直接验证计数照记、armed 未置位。
            s.gate(Command::default(), Duration::from_secs(5));
            assert!(!s.deadman_armed);
        }
        assert_eq!(s.runs.deadman, 200, "照计数，只是不值得说");

        // 一条新鲜意图即 armed；此后失联才是新闻。
        s.gate(Command::default(), Duration::from_millis(10));
        assert!(s.deadman_armed);
        assert_eq!(s.runs.deadman, 0);
    }

    /// 三条 run 独立计数；干净拍复位对应 run；NaN 拍不清 range run。
    #[test]
    fn limit_runs_are_independent_and_reset_by_clean_ticks() {
        let mut s = safety();

        s.apply(over_travel(), DEFAULT_POSITION, gain()).unwrap();
        s.apply(over_travel(), DEFAULT_POSITION, gain()).unwrap();
        assert_eq!(s.runs.range, 2);

        // NaN 拍在夹紧之前 return：not_finite 自立 run，range 站着不动。
        s.apply([f64::NAN; NUM_JOINTS], DEFAULT_POSITION, gain())
            .unwrap();
        assert_eq!(s.runs.not_finite, 1);
        assert_eq!(s.runs.range, 2, "NaN 拍不清 range run");

        // deadman 与它们互不相干。
        s.gate(Command::default(), Duration::from_secs(5));
        s.gate(Command::default(), Duration::from_secs(5));
        s.gate(Command::default(), Duration::from_secs(5));
        assert_eq!(s.runs.deadman, 3);
        assert_eq!(s.runs.range, 2);

        // 干净拍清零对应 run。
        s.apply(DEFAULT_POSITION, DEFAULT_POSITION, gain()).unwrap();
        assert_eq!(s.runs.range, 0);
        assert_eq!(s.runs.not_finite, 0, "上一拍非 NaN 已清零");
        s.gate(Command::default(), Duration::from_millis(10));
        assert_eq!(s.runs.deadman, 0);
    }

    /// 限流节奏：首拍必报，之后每 50 拍一行。
    #[test]
    fn worth_logging_first_and_every_fifty() {
        assert!(LimitRuns::worth_logging(1));
        assert!(!LimitRuns::worth_logging(2));
        assert!(!LimitRuns::worth_logging(LIMIT_LOG_EVERY - 1));
        assert!(LimitRuns::worth_logging(LIMIT_LOG_EVERY));
        assert!(!LimitRuns::worth_logging(LIMIT_LOG_EVERY + 1));
        assert!(LimitRuns::worth_logging(LIMIT_LOG_EVERY * 3));
    }
}
