//! 技能调度器（M6）：多策略优先级链。
//!
//! 原版没有独立的调度器对象——调度是控制循环里每拍的一段 if-else 级联
//!（upstream/robotd/src/control.rs:483-539）。我们把它收进本模块，是为了让
//! 优先级、command 重编码、窗口计时可以在 `cargo test` 里脱离 ONNX 直接断言：
//! `Cascade` 是纯状态机（不碰策略文件），`Scheduler` 在它外面裹上各槽位的
//! `Policy` 实例。
//!
//! 优先级链（对齐原版）：
//!   一次性技能（窗口未到期） > ground_pick（跑满周期相位） > sit/rise > walk
//!
//! "技能"的全部含义 = 换网络 + 重编码 command 块（upstream/duck-control/src/
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
///（upstream/robotd/src/intents.rs:105-117：位掩码而非队列——同拍两个
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

/// robot.do 的门口裁决（M8-B10，D36），判定顺序与文案逐字对齐原版
/// dispatch（upstream/robotd/src/main.rs:4196-4217）：
///
/// 1. 未 enable → 拒绝（:4196-4200；原文 "the policy is not driving —
///    press Start on the pad"，原版只有 pad 能 Start，我们没有 pad 也照抄
///    文案——逐字对齐优先于本地化）；
/// 2. 未 homed（斜坡回 home 进行中）→ 拒绝（:4201-4204）；
/// 3. 名字不在名单 → 拒绝并附名单（:4205-4217，"accepted 然后什么都不
///    发生是最坏的回答"，:4192-4195）。
///
/// fallen 不是入参——原版明确 "Not refused for being down. A skill on a
/// fallen robot is the human's call"（:4189-4191；其单测 :5771-5785 断言
/// fallen=true 仍 accepted 并入队）。fallen 软倒（我们的 Limp）期间
/// enabled/homed 都保持 true，请求照收后由控制循环当拍静默丢弃
///（control.rs edges_when_driving；原版 :2120-2158 只留日志）。
pub fn do_verdict(enabled: bool, homed: bool, name: &str, known: &[&str]) -> Result<u32, String> {
    if !enabled {
        return Err("the policy is not driving — press Start on the pad".to_owned());
    }
    if !homed {
        return Err("the robot is still going to its home pose; try again in a moment".to_owned());
    }
    match request_bit(name) {
        Some(bit) if known.contains(&name) => Ok(bit),
        _ => Err(if known.is_empty() {
            "this robot has no skills configured".to_owned()
        } else {
            format!(
                "no skill named {name:?}; this robot has {}",
                known.join(", ")
            )
        }),
    }
}

/// 一个一次性技能的静态配置。Vec 顺序就是优先级（原版注释："Order is
/// priority, replacing the hardcoded roulade > kick precedence"，
/// upstream/robotd-params/src/lib.rs:1326）。
pub struct SkillDef {
    /// robot.do {skill: "..."} 请求用的名字。
    pub name: &'static str,
    /// onnx 文件名（相对策略目录）。kick 的文件名是训练 run 名而非角色名
    /// （upstream/robotd-params/src/lib.rs:1312-1317 的注释专门解释了这个坑）。
    pub file: &'static str,
    /// 窗口长度（50Hz 拍数）。roulade 2.0s / kick 0.5s
    /// （roulade 对齐官方 infer_policy --roulade-duration 默认 2.0s；
    /// kick 仍 upstream/robotd-params/src/lib.rs:1318）。
    pub duration_ticks: u64,
}

/// 配置技能表，顺序=优先级：roulade > kick_left。kick_right 的权重没下载
///（任务范围只要 4 个文件），表里没有它。
pub const SKILLS: [SkillDef; 2] = [
    SkillDef {
        name: "roulade",
        file: "roulade.onnx",
        duration_ticks: 100, // 2.0s × 50Hz（官方 --roulade-duration 默认）
    },
    SkillDef {
        name: "kick_left",
        file: "ball_kick_left.onnx",
        duration_ticks: 25, // 0.5s × 50Hz
    },
];

/// ground_pick 周期 4.0s（upstream/robotd/src/control.rs:105
/// SkillTuning::default ground_pick_period）。
pub const GROUND_PICK_PERIOD_TICKS: u64 = 200; // 4.0s × 50Hz
/// 跑到周期相位的 0.7 交还（upstream/robotd-params/src/lib.rs:1256
/// DEFAULT_GROUND_PICK_END_PHASE；D29：无 end_phase 之外的提前交还形态）。
pub const GROUND_PICK_END_TICKS: u64 = 140; // 0.7 × 200

/// 起身过程时长 1.0s（upstream/robotd-params/src/lib.rs:1259
/// DEFAULT_SITSTAND_RISE_S）。
pub const SIT_RISE_TICKS: u64 = 50; // 1.0s × 50Hz

/// walk 槽 action_scale 缺省：公开 velstand 的训练属性（原型 alpha 0.9）。
/// 自训 Velocity（M9d/M9e，scale=1.0）用环境变量覆盖，见 `walk_action_scale`。
pub const WALK_ACTION_SCALE: f64 = 0.9;

/// 解析 walk 槽 action_scale。`None` / 非法 / 非正数 → [`WALK_ACTION_SCALE`]。
pub fn parse_walk_action_scale(raw: Option<&str>) -> f64 {
    match raw {
        Some(s) => match s.parse::<f64>() {
            Ok(v) if v.is_finite() && v > 0.0 => v,
            _ => WALK_ACTION_SCALE,
        },
        None => WALK_ACTION_SCALE,
    }
}

/// 运行时 walk 槽尺度：读 `MINIDUCK_WALK_ACTION_SCALE`，缺省 0.9。
pub fn walk_action_scale() -> f64 {
    parse_walk_action_scale(std::env::var("MINIDUCK_WALK_ACTION_SCALE").ok().as_deref())
}
/// sitstand 槽 action_scale：原版 start_sit_toggle 把整个坐/起周期钉在 1.0
///（upstream/robotd/src/control.rs:582-584 注释）。
pub const SITSTAND_ACTION_SCALE: f64 = 1.0;
/// ground_pick 槽 action_scale（upstream/robotd/src/control.rs:107
/// SkillTuning::default ground_pick_action_scale）。
pub const GROUND_PICK_ACTION_SCALE: f64 = 1.0;
/// 配置技能槽 action_scale：原版 SkillOverrides 默认 None → 回落到站姿
/// tuning；技能 command 全零 → standing_tuned 成立 → standing_action_scale=1.0
///（upstream/robotd/src/control.rs:560-578；默认值在
/// upstream/robotd-params/src/lib.rs:1749）。
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

/// 坐姿锁存（upstream/robotd/src/control.rs:227-230：锁存在控制器里，
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
    /// busy 与 sitting 都使跌倒预测器不合格并清去抖；坐着本身不算 busy。
    pub fn busy(&self) -> bool {
        self.ground_pick_ticks.is_some()
            || self.active_skill.is_some()
            || matches!(self.sit, Sit::Rising { .. })
    }

    pub fn is_sitting(&self) -> bool {
        self.sit == Sit::Sitting
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
    /// 顺序对齐原版（upstream/robotd/src/main.rs:2182-2208）：
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
                // 起身途中拒绝 toggle（原版同：upstream/robotd/src/control.rs:391-393）。
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
    /// 多跑一帧已结束的动作"，upstream/robotd/src/control.rs:437-440），
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
            // 是 zero_command_padding（upstream/robotd/src/control.rs:485-495）。
            // roulade 与 kick 的 command 都是全零
            //（upstream/robotd-params/src/lib.rs:1320,1334）。
            (Net::Skill(active.index), Command::default(), SKILL_ACTION_SCALE)
        } else if let Some(t) = self.ground_pick_ticks {
            // twist 槽载周期相位编码（upstream/robotd/src/control.rs:497-505）；
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
                    // 姿态标志搭在 twist 的 vx 槽：1=坐（upstream/robotd/src/
                    // control.rs:511-516）。head/body 保持客户端活值。
                    let mut c = *client;
                    c.twist = [1.0, 0.0, 0.0];
                    (Net::SitStand, c, SITSTAND_ACTION_SCALE)
                }
                Sit::Rising { .. } => {
                    // 起身过程 twist 全零（upstream/robotd/src/control.rs:517-520）。
                    let mut c = *client;
                    c.twist = [0.0; 3];
                    (Net::SitStand, c, SITSTAND_ACTION_SCALE)
                }
                Sit::Up => (Net::Walk, *client, walk_action_scale()),
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

    /// 忘掉活跃网选择：下一拍按换网处理（新 episode 从头开始）。
    /// 对应原版 PolicySet 的 active=None（推理失败路径
    /// upstream/duck-control/src/policy.rs:362-365、reset :415-416）。
    fn forget_active_net(&mut self) {
        self.last_net = None;
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

/// subscribe ack 报文件名不是路径，和 load 用同一字面量，避免两处漂移。
const SITSTAND_FILE: &str = "alpha_sitstand.onnx";
const GROUND_PICK_FILE: &str = "alpha_ground_pick.onnx";

/// 已加载槽的对外名单。skills 是配置技能的 robot.do 名字（优先级序），
/// 不含 ground_pick / sit_toggle——那两个是内置名，只出现在 robot.skills。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSlots {
    pub sitstand: Option<&'static str>,
    pub ground_pick: Option<&'static str>,
    pub skills: Vec<&'static str>,
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
        let sitstand = load_optional(dir.join(SITSTAND_FILE));
        let ground_pick = load_optional(dir.join(GROUND_PICK_FILE));
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
    /// 在后——顺序对齐原版 do_names（upstream/robotd/src/main.rs:4262-4273）。
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

    /// 已加载槽的文件名 / 配置技能名。给 subscribe ack 用：调用方在 spawn
    /// 前取一次，订阅时不再碰磁盘。walk 不在这里——加载失败时没有 Scheduler，
    /// 文件名仍要从调用方已知的路径取。
    pub fn loaded_slots(&self) -> LoadedSlots {
        let mut skills = Vec::new();
        for (i, slot) in self.skills.iter().enumerate() {
            if slot.is_some() {
                skills.push(SKILLS[i].name);
            }
        }
        LoadedSlots {
            sitstand: self.sitstand.is_some().then_some(SITSTAND_FILE),
            ground_pick: self.ground_pick.is_some().then_some(GROUND_PICK_FILE),
            skills,
        }
    }

    pub fn busy(&self) -> bool {
        self.cascade.busy()
    }

    pub fn is_sitting(&self) -> bool {
        self.cascade.is_sitting()
    }

    /// 清空所有已加载槽的 LSTM 状态：disable 或跌倒恢复完成时结束 recurrent episode
    ///（原版 was_driving && !driving && !enabled 边沿的 controller.reset()，
    /// upstream/robotd/src/main.rs:2791-2793）。Cascade 的窗口/坐姿锁存
    /// 不动——它们不是 episode 记忆，且非 Driving 阶段不推进；活跃网选择
    /// 清零（原版 PolicySet::reset 的 active=None，
    /// upstream/duck-control/src/policy.rs:415-427），enable 回来的第一拍
    /// 按换网处理（幂等：reset 已清零，再清一次无代价）。
    pub fn reset(&mut self) {
        self.cascade.forget_active_net();
        self.walk.reset();
        if let Some(p) = &mut self.sitstand {
            p.reset();
        }
        if let Some(p) = &mut self.ground_pick {
            p.reset();
        }
        for p in self.skills.iter_mut().flatten() {
            p.reset();
        }
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
    ///（upstream/duck-control/src/policy.rs:350-360）；last_action 与低通
    /// 锚点不归这里管，跨切换保留（upstream/robotd/src/control.rs:211-214）。
    /// 推理失败也 reset 该网络并忘掉活跃选择（原版 :361-365：
    /// 失败的推理状态绝不带进下一拍，下一拍按换网处理）。
    pub fn infer(
        &mut self,
        observation: &crate::obs::Observation,
        decision: &Decision,
    ) -> Result<[f32; crate::obs::ACTION_LEN], PolicyError> {
        let policy = self.net_mut(decision.net);
        if decision.switched {
            policy.reset();
        }
        let result = policy.infer(observation);
        if result.is_err() {
            policy.reset();
            self.cascade.forget_active_net();
        }
        result
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

    /// disable 的 policy reset 覆盖所有已加载槽（原版 controller.reset()
    /// 结束整个 recurrent episode，不只是当前活跃网，
    /// upstream/robotd/src/main.rs:2791-2793）。
    #[test]
    fn reset_clears_every_loaded_slot() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("policies");
        let ort = dir.join("../third_party/onnxruntime/lib/libonnxruntime.so");
        if !ort.exists() || SKILLS.iter().any(|s| !dir.join(s.file).exists()) {
            eprintln!("skip reset_clears_every_loaded_slot: policies or ORT missing");
            return;
        }
        if std::env::var_os("ORT_DYLIB_PATH").is_none() {
            // SAFETY: 测试进程内设置一次查找路径；ort 首次 API 调用前必须就绪。
            unsafe {
                std::env::set_var("ORT_DYLIB_PATH", &ort);
            }
        }
        let mut scheduler =
            Scheduler::load(&dir.join("velstand.onnx"), &dir).expect("load scheduler");
        if scheduler.available_names().len() != 2 + SKILLS.len() {
            eprintln!("skip: not all M6 policies loaded");
            return;
        }
        assert_loaded_slots(&scheduler);
        // load 自己会调一次 reset 初始化 LSTM（policy.rs:149），所以比增量。
        let before: Vec<u64> = std::iter::once(&scheduler.walk)
            .chain([&scheduler.sitstand, &scheduler.ground_pick].into_iter().flatten())
            .chain(scheduler.skills.iter().flatten())
            .map(|p| p.reset_calls)
            .collect();
        scheduler.reset();
        let after: Vec<u64> = std::iter::once(&scheduler.walk)
            .chain([&scheduler.sitstand, &scheduler.ground_pick].into_iter().flatten())
            .chain(scheduler.skills.iter().flatten())
            .map(|p| p.reset_calls)
            .collect();
        assert_eq!(before.len(), 3 + SKILLS.len(), "有槽没加载到？");
        for (b, a) in before.iter().zip(after.iter()) {
            assert_eq!(*a, b + 1, "有槽没被 reset");
        }
    }

    #[test]
    fn walk_action_scale_defaults_and_overrides() {
        assert_eq!(parse_walk_action_scale(None), 0.9);
        assert_eq!(parse_walk_action_scale(Some("1.0")), 1.0);
        assert_eq!(parse_walk_action_scale(Some("0.9")), 0.9);
        assert_eq!(parse_walk_action_scale(Some("")), WALK_ACTION_SCALE);
        assert_eq!(parse_walk_action_scale(Some("nope")), WALK_ACTION_SCALE);
        assert_eq!(parse_walk_action_scale(Some("-1")), WALK_ACTION_SCALE);
        assert_eq!(parse_walk_action_scale(Some("0")), WALK_ACTION_SCALE);
        assert_eq!(parse_walk_action_scale(Some("nan")), WALK_ACTION_SCALE);
        // 无环境变量时，运行时入口与缺省常量一致。
        assert_eq!(walk_action_scale(), WALK_ACTION_SCALE);
    }

    #[test]
    fn walk_is_the_default_and_passes_the_client_command_through() {
        let mut c = cascade();
        let client = client_command();
        let d = c.step(&client);
        assert_eq!(d.net, Net::Walk);
        assert_eq!(d.command, client, "walk 支路不改客户端命令");
        assert_eq!(d.label, "walk");
        assert_eq!(d.action_scale, walk_action_scale());
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

    /// D36：门口裁决 gate 的是 enabled 与 homed（原版 dispatch 的两种
    /// 情形拒绝，upstream/robotd/src/main.rs:4196-4204），fallen 不是
    /// 入参——原版单测 :5771-5785 断言 fallen=true 仍 accepted 且入队。
    /// 我们的 Limp（fallen 软倒）期间 enabled/homed 皆 true，即下面
    /// 第一个分支：照收，丢弃是控制循环的事（control.rs
    /// edges_when_driving）。
    #[test]
    fn do_verdict_gates_on_enabled_and_homed_never_on_fallen() {
        let known = ["ground_pick", "sit_toggle", "roulade", "kick_left"];
        // fallen/Limp 的形态：enabled=true、homed=true → 照收。
        for name in known {
            assert_eq!(
                do_verdict(true, true, name, &known),
                Ok(request_bit(name).unwrap())
            );
        }
        // 未 enable → 原版 :4200 原文；且在名字之前拒（:4192-4195），
        // 未知名也先吃情形拒绝。
        assert_eq!(
            do_verdict(false, true, "kick_left", &known).unwrap_err(),
            "the policy is not driving — press Start on the pad"
        );
        assert_eq!(
            do_verdict(false, false, "no_such", &known).unwrap_err(),
            "the policy is not driving — press Start on the pad"
        );
        // 斜坡途中（未 homed）→ 原版 :4202-4204 原文。
        assert_eq!(
            do_verdict(true, false, "kick_left", &known).unwrap_err(),
            "the robot is still going to its home pose; try again in a moment"
        );
    }

    #[test]
    fn do_verdict_refuses_unknown_names_with_the_list() {
        let known = ["ground_pick", "sit_toggle", "roulade", "kick_left"];
        let err = do_verdict(true, true, "kick_right", &known).unwrap_err();
        assert!(err.contains("\"kick_right\""), "{err}");
        assert!(err.contains("roulade") && err.contains("kick_left"), "{err}");
        // 名单为空（策略没加载，D19）：统一文案，不编造名单。
        assert_eq!(
            do_verdict(true, true, "kick_left", &[]).unwrap_err(),
            "this robot has no skills configured"
        );
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
        assert_loaded_slots(&sched);
        Some(sched)
    }

    fn assert_loaded_slots(scheduler: &Scheduler) {
        let slots = scheduler.loaded_slots();
        assert_eq!(slots.sitstand, Some(SITSTAND_FILE));
        assert_eq!(slots.ground_pick, Some(GROUND_PICK_FILE));
        assert_eq!(slots.skills, vec!["roulade", "kick_left"]);
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

    /// 推理失败即 reset 该网络，且忘掉活跃选择——下一拍按换网处理
    ///（原版 PolicySet::infer 失败路径，upstream/duck-control/src/policy.rs:361-365）。
    #[test]
    fn failed_inference_resets_the_network() {
        let Some(mut s) = load_scheduler() else { return };
        let client = Command::default();

        let d = s.step(&client);
        assert!(d.switched);
        let obs = crate::obs::Observation::build(
            &crate::io::ImuData::default(),
            &crate::model::DEFAULT_POSITION,
            &[0.0; crate::model::NUM_JOINTS],
            &crate::model::DEFAULT_POSITION,
            &[0.0; crate::obs::ACTION_LEN],
            &d.command,
        );
        s.infer(&obs, &d).unwrap(); // 消费掉换网 reset
        s.advance();
        let d = s.step(&client);
        assert!(!d.switched);

        let mut bad_vel = [0.0; crate::model::NUM_JOINTS];
        bad_vel[0] = f64::NAN;
        let bad_obs = crate::obs::Observation::build(
            &crate::io::ImuData::default(),
            &crate::model::DEFAULT_POSITION,
            &bad_vel,
            &crate::model::DEFAULT_POSITION,
            &[0.0; crate::obs::ACTION_LEN],
            &d.command,
        );
        let resets = s.walk.reset_calls;
        assert!(s.infer(&bad_obs, &d).is_err(), "非有限观测必须推理失败");
        assert_eq!(s.walk.reset_calls, resets + 1, "失败即 reset");

        let d2 = s.step(&client);
        assert!(d2.switched, "失败后下一拍按换网处理（active 已忘）");
    }
}
