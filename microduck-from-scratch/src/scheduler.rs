//! 技能调度器（M6）：多策略优先级链。
//!
//! 原版没有独立的调度器对象——调度是控制循环里每拍的一段 if-else 级联
//!（reference/robotd/src/control.rs:483-539）。我们把它收进本模块，是为了让
//! 优先级、command 重编码、窗口计时可以在 `cargo test` 里脱离 ONNX 直接断言：
//! `Cascade` 是纯状态机（不碰策略文件），`Scheduler` 在它外面裹上各槽位的
//! `Policy` 实例。
//!
//! 优先级链（对齐原版）：
//!   一次性技能（窗口未到期） > ground_pick（跑满周期相位） > sit/rise > walk
//!
//! "技能"的全部含义 = 换网络 + 重编码 command 块（reference/duck-control/src/
//! policy.rs:6-8：所有网络共享同一个 61 维观测布局，差异只在 command 块）。
//!
//! 刻意不做的（偏差簿）：chain/unwind 窗口、ground_pick 的 end_phase 提前交还
//! 以外的形态、技能间抢占（运行中一律拒绝新请求，D29）、策略热换（D30）、
//! Mode(Walk/Roller)（D28）、stand 槽与 will_stand 幅值选网（D33）。

use std::path::{Path, PathBuf};

use crate::obs::Command;
use crate::policy::{Policy, PolicyError};

/// 技能请求边沿位掩码的位分配：bit0=ground_pick、bit1=sit_toggle、
/// bit(2+i)=配置技能 i。与原版 intents.rs 的 SkillRequests 同构
///（reference/robotd/src/intents.rs:105-117：位掩码而非队列——同拍两个
/// 不同请求都该被看到，优先级顺序决定谁先）。
pub const EDGE_GROUND_PICK: u32 = 1 << 0;
pub const EDGE_SIT_TOGGLE: u32 = 1 << 1;

/// 技能名 → 边沿位。配置技能按 SKILLS 表序占位（表序即优先级）。
/// 名字不在表里 = 这个机器人不会这个技能。
pub fn request_bit(name: &str) -> Option<u32> {
    match name {
        "ground_pick" => Some(EDGE_GROUND_PICK),
        "sit_toggle" => Some(EDGE_SIT_TOGGLE),
        other => SKILLS
            .iter()
            .position(|s| s.name == other)
            .map(|i| 1u32 << (2 + i)),
    }
}

/// 一个一次性技能的静态配置。Vec 顺序就是优先级（原版注释："Order is
/// priority, replacing the hardcoded roulade > kick precedence"，
/// reference/robotd-params/src/lib.rs:1326）。
pub struct SkillDef {
    /// robot.do {skill: "..."} 请求用的名字。
    pub name: &'static str,
    /// onnx 文件名（相对策略目录）。kick 的文件名是训练 run 名而非角色名
    /// （reference/robotd-params/src/lib.rs:1312-1317 的注释专门解释了这个坑）。
    pub file: &'static str,
    /// 窗口长度（50Hz 拍数）。roulade 1.0s / kick 0.5s
    /// （reference/robotd-params/src/lib.rs:1330 / :1318）。
    pub duration_ticks: u64,
}

/// 配置技能表，顺序=优先级：roulade > kick_left。kick_right 的权重没下载
///（任务范围只要 4 个文件），表里没有它。
pub const SKILLS: [SkillDef; 2] = [
    SkillDef {
        name: "roulade",
        file: "roulade.onnx",
        duration_ticks: 50, // 1.0s × 50Hz
    },
    SkillDef {
        name: "kick_left",
        file: "ball_kick_left.onnx",
        duration_ticks: 25, // 0.5s × 50Hz
    },
];

/// ground_pick 周期 4.0s（reference/robotd/src/control.rs:105
/// SkillTuning::default ground_pick_period）。
pub const GROUND_PICK_PERIOD_TICKS: u64 = 200; // 4.0s × 50Hz
/// 跑到周期相位的 0.7 交还（reference/robotd-params/src/lib.rs:1256
/// DEFAULT_GROUND_PICK_END_PHASE；D29：无 end_phase 之外的提前交还形态）。
pub const GROUND_PICK_END_TICKS: u64 = 140; // 0.7 × 200

/// 起身过程时长 1.0s（reference/robotd-params/src/lib.rs:1259
/// DEFAULT_SITSTAND_RISE_S）。
pub const SIT_RISE_TICKS: u64 = 50; // 1.0s × 50Hz

/// walk 槽 action_scale：velstand 的训练属性（M3 时是 control.rs 的
/// ACTION_SCALE；原型 alpha 默认 0.9，不是 standing 的 1.0）。
pub const WALK_ACTION_SCALE: f64 = 0.9;
/// sitstand 槽 action_scale：原版 start_sit_toggle 把整个坐/起周期钉在 1.0
///（reference/robotd/src/control.rs:582-584 注释）。
pub const SITSTAND_ACTION_SCALE: f64 = 1.0;
/// ground_pick 槽 action_scale（reference/robotd/src/control.rs:107
/// SkillTuning::default ground_pick_action_scale）。
pub const GROUND_PICK_ACTION_SCALE: f64 = 1.0;
/// 配置技能槽 action_scale：原版 SkillOverrides 默认 None → 回落到站姿
/// tuning；技能 command 全零 → standing_tuned 成立 → standing_action_scale=1.0
///（reference/robotd/src/control.rs:560-578；默认值在
/// reference/robotd-params/src/lib.rs:1749）。
pub const SKILL_ACTION_SCALE: f64 = 1.0;

/// 网络槽位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Net {
    Walk,
    SitStand,
    GroundPick,
    /// 索引进 SKILLS 表。
    Skill(usize),
}

/// 一拍调度决定：选哪个网络、喂什么 command、用什么动作尺度。
#[derive(Debug, Clone, Copy)]
pub struct Decision {
    pub net: Net,
    /// 重编码后的 command（喂观测用；客户端原命令只在 walk/sit 支路存活）。
    pub command: Command,
    /// 当前活跃技能名，给 robot.state 的 skill 字段：
    /// "walk"/"sit"/"rise"/"ground_pick"/技能名。
    pub label: &'static str,
    pub action_scale: f64,
    /// 本拍换了网络。调用方据此 reset 新网络的 LSTM 状态。
    pub switched: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveSkill {
    index: usize,
    /// 剩余拍数；0 = 下一拍交还。
    remaining_ticks: u64,
}

/// 坐姿锁存（reference/robotd/src/control.rs:227-230：锁存在控制器里，
/// 直到下次 toggle）。Sitting 不算 busy——坐着的机器人是停驻，不是行进。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sit {
    Up,
    Sitting,
    Rising { remaining_ticks: u64 },
}

/// 纯调度状态机：不持有 Policy，单测不需要 ONNX Runtime。
/// 可用性（哪个槽加载成功）在构造时钉死；请求不可用技能 = 拒绝。
pub struct Cascade {
    sitstand_available: bool,
    ground_pick_available: bool,
    skills_available: Vec<bool>,
    active_skill: Option<ActiveSkill>,
    /// 已运行拍数；相位 φ = ticks / GROUND_PICK_PERIOD_TICKS。
    ground_pick_ticks: Option<u64>,
    sit: Sit,
    last_net: Option<Net>,
}

impl Cascade {
    pub fn new(sitstand: bool, ground_pick: bool, skills_available: Vec<bool>) -> Self {
        debug_assert_eq!(skills_available.len(), SKILLS.len());
        Cascade {
            sitstand_available: sitstand,
            ground_pick_available: ground_pick,
            skills_available,
            active_skill: None,
            ground_pick_ticks: None,
            sit: Sit::Up,
            last_net: None,
        }
    }

    /// busy = 脚本动作在飞：ground_pick / 一次性技能 / 起身中。
    ///（reference/robotd/src/control.rs:318-322）。M6 用它抑制 Driving→Limp
    /// 转移（原版门的是 FallPredictor，简化见偏差 D32）。坐着不算 busy。
    pub fn busy(&self) -> bool {
        self.ground_pick_ticks.is_some()
            || self.active_skill.is_some()
            || matches!(self.sit, Sit::Rising { .. })
    }

    pub fn label(&self) -> Option<&'static str> {
        self.last_net.map(|net| match net {
            Net::Walk => "walk",
            Net::SitStand if self.sit == Sit::Sitting => "sit",
            Net::SitStand => "rise",
            Net::GroundPick => "ground_pick",
            Net::Skill(i) => SKILLS[i].name,
        })
    }

    /// 处理一拍取到的技能请求边沿（每拍取一次清零，取在 control.rs）。
    /// 顺序对齐原版（reference/robotd/src/main.rs:2182-2208）：
    /// ground_pick → sit_toggle → 配置技能按优先级（表序）。
    ///
    /// 抢占简化（偏差 D29）：技能/ground_pick 运行中拒绝新技能/ground_pick
    /// 请求；原版"chain 技能可抢占非 chain 技能尾部 / ground_pick 可 preempt
    /// kick 尾部"不做。
    pub fn handle_requests(&mut self, edges: u32) {
        if edges & EDGE_GROUND_PICK != 0 {
            if !self.ground_pick_available {
                eprintln!("scheduler: ground_pick refused: policy not loaded");
            } else if self.ground_pick_ticks.is_some() || self.active_skill.is_some() {
                eprintln!("scheduler: ground_pick refused: a scripted move is running");
            } else {
                eprintln!("scheduler: ground_pick started");
                self.ground_pick_ticks = Some(0);
            }
        }
        if edges & EDGE_SIT_TOGGLE != 0 {
            match self.sit {
                Sit::Up => {
                    if self.sitstand_available {
                        eprintln!("scheduler: sit");
                        self.sit = Sit::Sitting;
                    } else {
                        eprintln!("scheduler: sit_toggle refused: sitstand policy not loaded");
                    }
                }
                Sit::Sitting => {
                    eprintln!("scheduler: stand up");
                    self.sit = Sit::Rising {
                        remaining_ticks: SIT_RISE_TICKS,
                    };
                }
                // 起身途中拒绝 toggle（原版同：reference/robotd/src/control.rs:391-393）。
                Sit::Rising { .. } => eprintln!("scheduler: sit_toggle refused: already rising"),
            }
        }
        for (i, def) in SKILLS.iter().enumerate() {
            if edges & (1 << (2 + i)) == 0 {
                continue;
            }
            if !self.skills_available[i] {
                eprintln!("scheduler: {} refused: policy not loaded", def.name);
            } else if self.active_skill.is_some() || self.ground_pick_ticks.is_some() {
                eprintln!(
                    "scheduler: {} refused: a scripted move is running",
                    def.name
                );
            } else {
                eprintln!("scheduler: {} started", def.name);
                self.active_skill = Some(ActiveSkill {
                    index: i,
                    remaining_ticks: def.duration_ticks,
                });
            }
        }
    }

    /// 一拍调度：先到期的窗口先交还（"到期后的那一拍跑下一个东西而不是
    /// 多跑一帧已结束的动作"，reference/robotd/src/control.rs:437-440），
    /// 然后按优先级链选网并重编码 command。窗口倒计时在 advance() 里做
    ///（用过本拍之后再推进，与原版"电机写完后推进相位"同序）。
    pub fn step(&mut self, client: &Command) -> Decision {
        if self
            .active_skill
            .is_some_and(|a| a.remaining_ticks == 0)
        {
            self.active_skill = None;
        }
        if self
            .ground_pick_ticks
            .is_some_and(|t| t >= GROUND_PICK_END_TICKS)
        {
            self.ground_pick_ticks = None;
        }
        if let Sit::Rising { remaining_ticks: 0 } = self.sit {
            self.sit = Sit::Up;
        }

        let (net, command, action_scale) = if let Some(active) = self.active_skill {
            // 一次性技能窗口内：twist/head/body 全清零——这些网络的训练环境
            // 是 zero_command_padding（reference/robotd/src/control.rs:485-495）。
            // roulade 与 kick 的 command 都是全零
            //（reference/robotd-params/src/lib.rs:1320,1334）。
            (Net::Skill(active.index), Command::default(), SKILL_ACTION_SCALE)
        } else if let Some(t) = self.ground_pick_ticks {
            // twist 槽载周期相位编码（reference/robotd/src/control.rs:497-505）；
            // head/body 清零，同训练环境的 zero_command_padding。
            let phase = t as f64 / GROUND_PICK_PERIOD_TICKS as f64;
            let angle = std::f64::consts::TAU * phase;
            let command = Command {
                twist: [angle.cos(), angle.sin(), 0.0],
                ..Command::default()
            };
            (Net::GroundPick, command, GROUND_PICK_ACTION_SCALE)
        } else {
            match self.sit {
                Sit::Sitting => {
                    // 姿态标志搭在 twist 的 vx 槽：1=坐（reference/robotd/src/
                    // control.rs:511-516）。head/body 保持客户端活值。
                    let mut c = *client;
                    c.twist = [1.0, 0.0, 0.0];
                    (Net::SitStand, c, SITSTAND_ACTION_SCALE)
                }
                Sit::Rising { .. } => {
                    // 起身过程 twist 全零（reference/robotd/src/control.rs:517-520）。
                    let mut c = *client;
                    c.twist = [0.0; 3];
                    (Net::SitStand, c, SITSTAND_ACTION_SCALE)
                }
                Sit::Up => (Net::Walk, *client, WALK_ACTION_SCALE),
            }
        };

        let switched = self.last_net != Some(net);
        self.last_net = Some(net);
        let label = self.label().expect("last_net just set");
        Decision {
            net,
            command,
            label,
            action_scale,
            switched,
        }
    }

    /// 推进窗口计时（在本拍推理/写电机之后调用）。
    pub fn advance(&mut self) {
        if let Some(active) = &mut self.active_skill {
            active.remaining_ticks = active.remaining_ticks.saturating_sub(1);
        }
        if let Some(t) = &mut self.ground_pick_ticks {
            *t += 1;
        }
        if let Sit::Rising { remaining_ticks } = &mut self.sit {
            *remaining_ticks = remaining_ticks.saturating_sub(1);
        }
    }
}

/// 调度器 = Cascade 状态机 + 各槽位 Policy。
///
/// walk 是必须槽：加载失败由调用方决定整个调度器不存在（进程进"永远
/// Held"，D19 哲学）。其余槽各自可选：加载失败 = 该技能不可用但进程活着。
pub struct Scheduler {
    cascade: Cascade,
    walk: Policy,
    sitstand: Option<Policy>,
    ground_pick: Option<Policy>,
    skills: Vec<Option<Policy>>,
}

fn load_optional(path: PathBuf) -> Option<Policy> {
    match Policy::load(&path) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("WARNING: {e} — this skill is unavailable, daemon stays up");
            None
        }
    }
}

impl Scheduler {
    /// walk 从 `walk_path` 加载（必须），其余槽从 `dir` 按固定文件名加载。
    pub fn load(walk_path: &Path, dir: &Path) -> Result<Self, PolicyError> {
        let walk = Policy::load(walk_path)?;
        let sitstand = load_optional(dir.join("alpha_sitstand.onnx"));
        let ground_pick = load_optional(dir.join("alpha_ground_pick.onnx"));
        let skills: Vec<Option<Policy>> = SKILLS
            .iter()
            .map(|s| load_optional(dir.join(s.file)))
            .collect();
        let cascade = Cascade::new(
            sitstand.is_some(),
            ground_pick.is_some(),
            skills.iter().map(|s| s.is_some()).collect(),
        );
        Ok(Scheduler {
            cascade,
            walk,
            sitstand,
            ground_pick,
            skills,
        })
    }

    /// robot.skills 的名单：内置两个（ground_pick、sit_toggle）在前，配置技能
    /// 在后——顺序对齐原版 do_names（reference/robotd/src/main.rs:4262-4273）。
    /// 内置两个不是配置表条目但必须在名单里，这是原版实测踩过的坑
    ///（main.rs:4255-4261 注释：pad.bindings 曾因此拒掉五个绑定里的两个）。
    /// 只列加载成功的槽。
    pub fn available_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.ground_pick.is_some() {
            names.push("ground_pick");
        }
        if self.sitstand.is_some() {
            names.push("sit_toggle");
        }
        for (i, p) in self.skills.iter().enumerate() {
            if p.is_some() {
                names.push(SKILLS[i].name);
            }
        }
        names
    }

    /// busy 门控：Driving→Limp 转移的抑制条件（D32）。
    pub fn busy(&self) -> bool {
        self.cascade.busy()
    }

    pub fn handle_requests(&mut self, edges: u32) {
        self.cascade.handle_requests(edges);
    }

    pub fn step(&mut self, client: &Command) -> Decision {
        self.cascade.step(client)
    }

    pub fn advance(&mut self) {
        self.cascade.advance();
    }

    fn net_mut(&mut self, net: Net) -> &mut Policy {
        // cascade 只选已加载的槽；走到 None 是 bug，panic 比静默换网好。
        match net {
            Net::Walk => &mut self.walk,
            Net::SitStand => self.sitstand.as_mut().expect("cascade selected unloaded sitstand"),
            Net::GroundPick => self
                .ground_pick
                .as_mut()
                .expect("cascade selected unloaded ground_pick"),
            Net::Skill(i) => self.skills[i]
                .as_mut()
                .expect("cascade selected unloaded skill"),
        }
    }

    /// 在 decision 选中的网络上推理。换网即 reset 新网络的 LSTM 状态
    ///（reference/duck-control/src/policy.rs:350-360）；last_action 与低通
    /// 锚点不归这里管，跨切换保留（reference/robotd/src/control.rs:211-214）。
    pub fn infer(
        &mut self,
        observation: &crate::obs::Observation,
        decision: &Decision,
    ) -> Result<[f32; crate::obs::ACTION_LEN], PolicyError> {
        let policy = self.net_mut(decision.net);
        if decision.switched {
            policy.reset();
        }
        policy.infer(observation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::BodyPose;

    fn cascade() -> Cascade {
        Cascade::new(true, true, vec![true; SKILLS.len()])
    }

    fn client_command() -> Command {
        Command {
            twist: [0.3, 0.0, -0.2],
            head: [0.1, 0.2, 0.3, 0.4],
            body: BodyPose {
                z: 0.05,
                roll: 0.01,
                pitch: -0.02,
            },
        }
    }

    fn step_n(c: &mut Cascade, client: &Command, n: u64) -> Decision {
        let mut d = c.step(client);
        for _ in 1..n {
            c.advance();
            d = c.step(client);
        }
        d
    }

    #[test]
    fn walk_is_the_default_and_passes_the_client_command_through() {
        let mut c = cascade();
        let client = client_command();
        let d = c.step(&client);
        assert_eq!(d.net, Net::Walk);
        assert_eq!(d.command, client, "walk 支路不改客户端命令");
        assert_eq!(d.label, "walk");
        assert_eq!(d.action_scale, WALK_ACTION_SCALE);
        assert!(d.switched, "第一拍从 None 到 Walk 算切换");
        assert!(!c.busy());

        c.advance();
        let d = c.step(&client);
        assert!(!d.switched, "同一网络连拍不算切换");
    }

    #[test]
    fn sit_latches_and_carries_the_posture_flag_in_vx() {
        let mut c = cascade();
        let client = client_command();
        c.step(&client);
        c.advance();

        c.handle_requests(EDGE_SIT_TOGGLE);
        let d = c.step(&client);
        assert_eq!(d.net, Net::SitStand);
        assert_eq!(d.label, "sit");
        assert_eq!(d.command.twist, [1.0, 0.0, 0.0], "坐姿标志搭在 vx 槽");
        assert_eq!(d.command.head, client.head, "坐姿下 head 保持活值");
        assert_eq!(d.command.body, client.body, "坐姿下 body 保持活值");
        assert_eq!(d.action_scale, SITSTAND_ACTION_SCALE);
        assert!(d.switched);
        // 坐姿是锁存：不再请求也一直坐，且坐着不算 busy。
        let d = step_n(&mut c, &client, 100);
        assert_eq!(d.net, Net::SitStand);
        assert_eq!(d.label, "sit");
        assert!(!c.busy(), "坐着是停驻不是行进");
    }

    #[test]
    fn sit_toggle_rises_with_zeroed_twist_then_latches_up() {
        let mut c = cascade();
        let client = client_command();
        c.handle_requests(EDGE_SIT_TOGGLE);
        c.step(&client);
        c.advance();

        c.handle_requests(EDGE_SIT_TOGGLE);
        let d = c.step(&client);
        assert_eq!(d.net, Net::SitStand);
        assert_eq!(d.label, "rise");
        assert_eq!(d.command.twist, [0.0; 3], "起身过程 twist 全零");
        assert!(c.busy(), "起身中 busy（跌倒反射抑制）");

        // 起身途中再 toggle：拒绝，仍是 Rising。
        c.handle_requests(EDGE_SIT_TOGGLE);
        assert!(matches!(c.sit, Sit::Rising { .. }), "{:?}", c.sit);

        // 50 拍起身窗口到期后回到 walk。
        let d = step_n(&mut c, &client, SIT_RISE_TICKS + 1);
        assert_eq!(d.net, Net::Walk);
        assert_eq!(d.label, "walk");
        assert!(!c.busy());
    }

    #[test]
    fn ground_pick_runs_the_phase_script_then_hands_back() {
        let mut c = cascade();
        let client = client_command();
        c.handle_requests(EDGE_GROUND_PICK);
        assert!(c.busy());

        // 第 0 拍 φ=0 → [cos0, sin0, 0] = [1, 0, 0]。
        let d = c.step(&client);
        assert_eq!(d.net, Net::GroundPick);
        assert_eq!(d.label, "ground_pick");
        assert_eq!(d.command.twist, [1.0, 0.0, 0.0]);
        assert_eq!(d.command.head, [0.0; 4], "zero_command_padding");
        assert_eq!(d.action_scale, GROUND_PICK_ACTION_SCALE);

        // 第 50 拍（advance 50 次后）φ = 50/200 = 0.25 → [cos(π/2), sin(π/2), 0] ≈ [0, 1, 0]。
        let d = step_n(&mut c, &client, 51);
        let angle = std::f64::consts::TAU * 50.0 / GROUND_PICK_PERIOD_TICKS as f64;
        assert!((d.command.twist[0] - angle.cos()).abs() < 1e-12);
        assert!((d.command.twist[1] - angle.sin()).abs() < 1e-12);
        assert_eq!(d.net, Net::GroundPick, "φ=0.25 仍在窗口内");

        // 跑到 140 拍（φ=0.7）交还 walk：第 139 拍（φ=0.695）仍是 ground_pick，之后到期。
        let d = step_n(&mut c, &client, 140 - 50);
        assert_eq!(d.net, Net::GroundPick, "最后一拍（φ=0.695）仍是 ground_pick");
        assert!(c.busy());
        c.advance();
        let d = c.step(&client);
        assert_eq!(d.net, Net::Walk, "φ 到 0.7 后下一拍交还");
        assert_eq!(d.label, "walk");
        assert!(!c.busy());

        // 运行中拒绝重复请求。
        c.handle_requests(EDGE_GROUND_PICK);
        c.step(&client);
        c.handle_requests(EDGE_GROUND_PICK);
        let d = c.step(&client);
        assert_eq!(d.command.twist, [1.0, 0.0, 0.0], "重复请求被拒，相位没有重启");
    }

    #[test]
    fn skill_window_zeroes_the_whole_command_then_hands_back() {
        let mut c = cascade();
        let client = client_command();
        c.handle_requests(request_bit("kick_left").unwrap());
        assert!(c.busy());

        let d = c.step(&client);
        assert_eq!(d.net, Net::Skill(1), "kick_left 是 SKILLS[1]");
        assert_eq!(d.label, "kick_left");
        assert_eq!(d.command, Command::default(), "技能窗口内 command 全零");
        assert_eq!(d.action_scale, SKILL_ACTION_SCALE);

        // 0.5s = 25 拍：第 24 拍仍 kick，第 25 拍交还。
        let d = step_n(&mut c, &client, 25);
        assert_eq!(d.net, Net::Skill(1), "窗口最后一拍仍 kick");
        c.advance();
        let d = c.step(&client);
        assert_eq!(d.net, Net::Walk, "窗口到期交还 walk");
        assert_eq!(d.command, client);
    }

    #[test]
    fn priority_same_tick_requests_first_in_table_wins() {
        // 同拍请求 roulade+kick_left：按表序处理，roulade 先启动，
        // kick_left 撞上"已有脚本动作在飞"被拒（D29 简化抢占）。
        let mut c = cascade();
        let both = request_bit("roulade").unwrap() | request_bit("kick_left").unwrap();
        c.handle_requests(both);
        let d = c.step(&Command::default());
        assert_eq!(d.net, Net::Skill(0), "roulade 优先于 kick_left");
        assert_eq!(d.label, "roulade");

        // 技能 > ground_pick：ground_pick 运行中技能请求被拒，反之亦然。
        let mut c = cascade();
        c.handle_requests(request_bit("roulade").unwrap());
        c.handle_requests(EDGE_GROUND_PICK);
        let d = c.step(&Command::default());
        assert_eq!(d.net, Net::Skill(0), "技能运行中 ground_pick 被拒");
    }

    #[test]
    fn active_skill_outranks_sit_and_returns_to_it() {
        let mut c = cascade();
        let client = Command::default();
        c.handle_requests(EDGE_SIT_TOGGLE);
        let d = c.step(&client);
        assert_eq!(d.net, Net::SitStand);

        // 坐姿中踢一脚：技能抢占（原版 X 键可以从坐姿滚出），
        // 窗口结束回到坐姿——锁存没被技能动过。
        c.handle_requests(request_bit("kick_left").unwrap());
        let d = c.step(&client);
        assert_eq!(d.net, Net::Skill(1));
        let d = step_n(&mut c, &client, 26);
        assert_eq!(d.net, Net::SitStand);
        assert_eq!(d.command.twist, [1.0, 0.0, 0.0], "回到坐姿锁存");
    }

    #[test]
    fn unavailable_skills_are_refused() {
        let mut c = Cascade::new(false, false, vec![false, true]);
        c.handle_requests(EDGE_SIT_TOGGLE | EDGE_GROUND_PICK | request_bit("roulade").unwrap());
        let d = c.step(&Command::default());
        assert_eq!(d.net, Net::Walk, "三个请求都该被拒绝");
        c.handle_requests(request_bit("kick_left").unwrap());
        let d = c.step(&Command::default());
        assert_eq!(d.net, Net::Skill(1), "加载成功的槽照常可用");
    }

    #[test]
    fn busy_is_exactly_skill_or_ground_pick_or_rising() {
        let mut c = cascade();
        let client = Command::default();
        c.step(&client);
        assert!(!c.busy());

        c.handle_requests(EDGE_SIT_TOGGLE);
        c.step(&client);
        assert!(!c.busy(), "坐着不 busy");

        c.handle_requests(EDGE_SIT_TOGGLE);
        c.step(&client);
        assert!(c.busy(), "起身 busy");
        step_n(&mut c, &client, SIT_RISE_TICKS + 1);
        assert!(!c.busy(), "起身完成不 busy");

        c.handle_requests(EDGE_GROUND_PICK);
        c.step(&client);
        assert!(c.busy(), "ground_pick busy");
    }

    #[test]
    fn switch_flag_tracks_net_changes() {
        let mut c = cascade();
        let client = Command::default();
        assert!(c.step(&client).switched); // None → Walk
        c.advance();
        assert!(!c.step(&client).switched); // Walk → Walk
        c.handle_requests(EDGE_SIT_TOGGLE);
        assert!(c.step(&client).switched); // Walk → SitStand
        c.advance();
        assert!(!c.step(&client).switched); // SitStand 连拍
        c.handle_requests(EDGE_SIT_TOGGLE);
        c.advance();
        let d = step_n(&mut c, &client, SIT_RISE_TICKS); // rise 到期
        assert!(d.switched, "SitStand → Walk 是切换");
        assert_eq!(d.net, Net::Walk);
    }

    // ---- 下面是需要真实权重 + ONNX Runtime 的测试 ----

    fn ort_ready() -> bool {
        let so = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("third_party/onnxruntime/lib/libonnxruntime.so");
        if std::env::var_os("ORT_DYLIB_PATH").is_none() && so.exists() {
            // SAFETY: 测试进程内设置一次查找路径；ort 首次 API 调用前必须就绪。
            unsafe {
                std::env::set_var("ORT_DYLIB_PATH", &so);
            }
        }
        std::env::var("ORT_DYLIB_PATH")
            .map(|p| Path::new(&p).exists())
            .unwrap_or(false)
    }

    fn load_scheduler() -> Option<Scheduler> {
        if !ort_ready() {
            eprintln!("skip: ORT dylib missing");
            return None;
        }
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("policies");
        let sched = Scheduler::load(&dir.join("velstand.onnx"), &dir).expect("load scheduler");
        if sched.available_names().len() != 4 {
            eprintln!("skip: not all M6 policies present");
            return None;
        }
        Some(sched)
    }

    /// 换网络必须 reset 新网络（LSTM 契约；feedforward 下以 reset_calls 计数断言）。
    #[test]
    fn switching_nets_resets_the_new_network() {
        let Some(mut s) = load_scheduler() else { return };
        let client = Command::default();

        let d = s.step(&client);
        assert!(d.switched);
        let walk_resets = s.walk.reset_calls;
        let obs = crate::obs::Observation::build(
            &crate::io::ImuData::default(),
            &crate::model::DEFAULT_POSITION,
            &[0.0; crate::model::NUM_JOINTS],
            &crate::model::DEFAULT_POSITION,
            &[0.0; crate::obs::ACTION_LEN],
            &d.command,
        );
        s.infer(&obs, &d).unwrap();
        assert_eq!(s.walk.reset_calls, walk_resets + 1, "换到 walk 要 reset");

        s.advance();
        let d = s.step(&client);
        assert!(!d.switched);
        s.infer(&obs, &d).unwrap();
        assert_eq!(s.walk.reset_calls, walk_resets + 1, "连拍不重复 reset");

        s.handle_requests(EDGE_SIT_TOGGLE);
        let d = s.step(&client);
        assert!(d.switched, "walk → sitstand");
        let sit_resets = s.sitstand.as_ref().unwrap().reset_calls;
        s.infer(&obs, &d).unwrap();
        assert_eq!(s.sitstand.as_ref().unwrap().reset_calls, sit_resets + 1);
        assert_eq!(s.walk.reset_calls, walk_resets + 1, "旧网络不被碰");
    }
}
