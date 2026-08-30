# Android 模拟器 Agent Skill 调研

目标：找到能让 AI agent 操作 Android 模拟器、查看 App UI 的现成 Skill（SKILL.md 形态优先），用于 UI 走查 / 自动化验证场景。

调研日期：2026-08-30。结论先行：**有多个现成方案**，首选 `freespirits/android-control-skill` 或 `awesome-android-agent-skills`。

## 1. 纯 SKILL 形态（agent 直接调用，无需 MCP server）

### freespirits/android-control-skill ⭐ 首选

- 站点：https://freespirits.github.io/android-control-skill/
- 结构分两层：
  - `./tools/android`：单个 Python 文件，封装 `adb` 为稳定的 JSON-first 命令接口——截图、UI 树检查、元素定位、手势、等待、App 生命周期、日志、网络。
  - `skills/*/SKILL.md`：给 agent 的技能描述。
- 特点：**不需要 MCP server**；agent 通过截图 + uiautomator UI 层级理解界面，不依赖脆弱像素坐标；同时支持真机和模拟器。

### new-silvermoon/awesome-android-agent-skills（android-emulator-skill）

- 页面：https://crossaitools.com/skills/new-silvermoon/awesome-android-agent-skills/android-emulator-skill
- Python 脚本 + shell 工具集，语义化导航（按文字 / resource-id 找按钮，而非图像坐标）。
- 附带模拟器生命周期管理（AVD 启动/关闭）、实时日志过滤、构建自动化。
- 默认输出精简，可加 JSON 标志输出结构化数据；专为 AI agent 设计。
- 安装：`npx skills add ...`（openagentskill 注册表有对应条目）。

### skydoves/android-testing-skills

- 仓库：https://github.com/skydoves/android-testing-skills
- Android 方向知名工程师出品，skill 集合：Compose UI 测试、AndroidX Test、JVM 单测、ADB。
- 兼容 Claude Code / Android Studio Agent mode / Gemini。
- 偏"写自动化测试"而非"agent 实时操作模拟器"，适合作为测试代码编写的知识库。

### stablyai / orca-emulator-android

- 页面：https://vibehackers.io/claude-code/skills/orca-emulator-android
- 通过 `orca` CLI 控制 adb 连接的设备/模拟器：列出并启动 AVD、tap/swipe/输入、硬件按键（Back/Recents）、旋转、App 安装/启动、运行时权限、accessibility tree、logcat。
- 跨平台（Windows/Linux/macOS），与 orca-emulator（iOS）配套。
- 社区热度不错（约 19k 浏览 / 1.5k 安装量级）。

### 其他（较垂直）

- [theplant/mobai-skill-qa](https://github.com/theplant/mobai-skill-qa)：基于 mobai MCP 的移动端 E2E 验证 skill，iOS/Android 都支持，面向 QA 团队，中文文档。
- [Huc91/pixel-perfect-skill](https://github.com/Huc91/pixel-perfect-skill)：视觉 QA skill，把 Figma 设计和 Web/iOS/Android 实际 UI 对比找缺陷（颜色、间距、字体、对齐）。

## 2. MCP 形态（接受 MCP server 时的选择）

| 项目 | 说明 |
|---|---|
| [adb_mcp](https://lobehub.com/mcp/iksnerd-adb_mcp) | Go 写的 MCP server，过 adb 驱动模拟器：启 AVD、截图、读 UI 层级、tap/swipe/type、logcat。定位是 "Android 版 XcodeBuildMCP" |
| [dondetir/NeuralBridge_mcp](https://github.com/dondetir/neuralbridge_mcp) | 不走 ADB，agent 直接 HTTP 连设备上 AccessibilityService 伴侣 App，宣称 ~6ms 延迟（比 Appium 链路短得多） |
| [ghost-in-the-droid/android-agent](https://github.com/ghost-in-the-droid/android-agent) | 62 个 MCP 工具（tap/swipe/剪贴板/截图/浏览器原语/TTS/相机），Android 5.0+ 真机（USB/Wi-Fi ADB）+ iPhone（WebDriverAgent），`pip install ghost-in-the-droid` |
| RN Debug MCP | 单 MCP server 同时看 Android/iOS 上的 React Native App：读日志、检查 UI、tap、输入、滚动、截图、reload |

## 3. 选型建议

- 需求是"**agent 看模拟器里的 App UI 并操作**"→ 直接用纯 SKILL 形态，clone 进 `skills/` 目录即可，优先 **freespirits/android-control-skill**（无额外依赖，adb 是唯一要求）或 **awesome-android-agent-skills**。
- 环境已有 MCP 基础设施 / 需要更低延迟操控真机 → 考虑 **adb_mcp** 或 **NeuralBridge**。
- 共性前提：本机装 Android SDK（platform-tools 提供 `adb`、emulator），`ANDROID_HOME` 环境变量配好，模拟器已创建 AVD。

## 4. 备注

- 本仓库 `skills/` 下现有技能均为商业/营销方向，无移动端相关 skill；如引入建议放 `skills/android-emulator-qa/` 之类的新目录。
- 汇总注册表可参考 [VoltAgent/awesome-agent-skills](https://github.com/VoltAgent/awesome-agent-skills)（1000+ skill 清单），后续找其他 skill 可先查它。
