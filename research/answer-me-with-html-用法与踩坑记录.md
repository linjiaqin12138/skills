# answer-me-with-html 用法与踩坑记录

> 来源：2026-10-11 实际使用记录（安装、出图文页、出视频、自绘 SVG 动画、集成进 clone-from-scratch-tutor）。
> 仓库：https://github.com/QingYunA/answer-me-with-html ，本地位置 `vendor/answer-me-with-html/`。

## 它是什么

模型只写内容草稿（扩展 Markdown，每个 `##` 一个面板），内置 CLI 负责布局、配色、图表坐标，产出单文件 HTML。官方口径：比手写 HTML 省约 8 倍输出 token、快约 2.6 倍。

- CLI：`node vendor/answer-me-with-html/scripts/am.mjs`(Node 20+、零依赖、单文件）
- 默认输出：页面 `~/.answer-me-with-html/pages/`，视频 `~/.answer-me-with-html/videos/`(`AM_HOME` 可改）
- 页面内嵌 `#am-source`（原草稿）,`am patch` 靠它做局部更新，草稿文件本身不用留

## 日常用法

```bash
AM="node vendor/answer-me-with-html/scripts/am.mjs"
$AM render 草稿.md -o out.html          # 出页;- 从 stdin 读(heredoc 友好)
$AM render 草稿.md --replace <旧页>      # 重渲:新页写成功后删旧页,一个答案只留一页
$AM patch page.html --panel "面板标题"   # 只重渲一个面板(标题/字母 ID 都能匹配)
$AM video 草稿.md                        # 出 3b1b 风格讲解视频(播放器页面)
$AM help <组件名>                        # 组件语法,写草稿前必查(flow/sequence/tree/code...)
$AM list                                 # 模板/主题/组件清单
$AM config                               # 设置:open(自动开浏览器)/theme/mode/style/voice
```

组件速查(按信息形状选):连接关系/架构/分支 `flow [LR]`;角色间消息 `sequence [num]`;层级 `tree`;历史 `timeline`;对比 Markdown 表格(ok/no/warn 状态列自动变徽章);结论警告 `callout <info|ok|warn|err> 标题`;真实代码 ```` ```lang src=path lines=18-30 hl=22 ````(空块,CLI 自己读文件);组件表达不了的图 ```` ```svg ```` 内嵌。

frontmatter:`title` / `subtitle` / `template: sheet|doc` / `theme: auto|blueprint|shadcn|paper|3b1b` / `cols: 3` / `lang: zh`。面板宽用 `{span=2}` 控制,宽表宽图别写 span,要独占一行写 `span=cols`。

## STE 行文检查(会自动跑)

中文草稿常见触发点(本次全踩过):

- **套话**:"打通""多维度"被判 cliché,删掉或改成具体事实
- **"的"字链**:超过三个连用的"的"要拆句
- **句长**:描述句 ≤45 字,步骤句 ≤35 字
- 一句话只说一件事、主动语态、模糊量词("若干""大概")会警告

警告不阻塞出页(style 默认 80),但**带警告的页面不会自动打开浏览器**;按提示改完加 `--replace` 重渲即可,最多两轮,改不掉就保留并说明。

## 坑清单(本次实踩)

### 坑 1:file:// 链接 Firefox 打不开

现象:"Access to the file was denied / Firefox can't connect to the server at local file"。文件本身完好,是 Firefox 对外部点击 file:// 链接的限制。

修法(三选一):① 起本地 HTTP 服务 `python3 -m http.server 8000 --directory ~/.answer-me-with-html`,走 `http://127.0.0.1:8000/pages/xxx.html`;② 地址栏手动粘贴完整 file:// 路径(地址栏输入不受此限制);③ 终端 `firefox <path>`。注意:视频导出按钮需要安全上下文,**localhost 可以,普通 http 不行**。

### 坑 2:视频里自定义 SVG 整幕空白(两个叠加根因)

现象:播放器标题卡正常、字幕在走,画面区全黑。

**根因 A(步进对齐)**:视频场景里每个块按 `data-step` 分组逐步出现;没有 `data-step` 的裸 SVG 整个算 1 步,且步数少于旁白行数时**步骤对齐到最后几拍**——一幕 4 句旁白,图要到第 4 句才出现,前面全是空白。修法:给 SVG 元素分组标 `<g data-step="n">`,让步数 == 旁白行数;未标记的元素全程可见(适合底图)。

**根因 B(尺寸塌缩)**:SVG 只写 `viewBox` 没写 `width`/`height`,播放器的缩放容器(`width: max-content`)量到 0×0,怎么缩放都是 0 像素。修法:**SVG 必须显式写 width 和 height 属性**。普通图文页里的 ```` ```svg ```` 块同理,这条已写进 clone-from-scratch-tutor 的 writer 规范。

排查手法:用 agent-browser 打开页面,`eval` 查 `document.querySelector('.amv-fit svg').getBoundingClientRect()`,宽高为 0 即根因 B;`querySelectorAll('[data-step]').length` 为 0 即根因 A。

### 坑 3:`am serve` 不如 python http.server 直接

`am serve --port 8765` 起后访问 `/` 和各猜测路径都 404(它大概只服务 render 时注册的页面)。别纠结,直接 `python3 -m http.server --directory ~/.answer-me-with-html` 覆盖 pages + videos 两个子目录。

### 坑 4:配音与视频文件导出的环境门槛

- 配音:`--voice auto` 依次找 ElevenLabs(`ELEVENLABS_API_KEY`)→ 系统 TTS(macOS say / Linux espeak-ng)→ 都没有则**纯字幕**。本机 Linux 无 TTS,字幕效果可以接受
- `--mp4` 需要 ffmpeg + Chrome + Node 22+;`--webm` 不需要 ffmpeg 但很慢(实测 2:27 的视频编码 13.5 分钟)。本机 vendor/browsers 有 Chrome、系统有 ffmpeg,`--mp4` 条件实际满足(导出耗时约 1.3 倍视频时长)
- 页面内的导出按钮(⤓)在浏览器里自己编码,无需终端,但要安全上下文

## 视频(am video)机制要点(读播放器源码所得,自绘 SVG 时必用)

- 一个 `##` 一幕;`>` 开头的是旁白,一行一拍;第 N 拍显示第 N 步
- 步进:flow/er/sequence/tree 每个源码行一步;表格/列表逐行;**自定义 SVG 用 `data-step` 标组**
- 旁白里写 `[名字]`:镜头缩放高亮该元素。自定义 SVG 里按 `<text>` 内容 includes 匹配,所以文字标注要写得能被对上
- 相邻两幕中 `data-key` 相同的元素平滑形变(SVG↔SVG):让同一个对象(如光盘、内存条)跨幕连续运动就给它固定 data-key
- 一幕 2~5 句旁白,3~6 幕;旁白要写成说话,不要写成手册

## 自绘 SVG 动画配方(HTML 页面里也能动)

组件表达不了的示意图(内存分布、加载搬运)用 ```` ```svg ```` 内嵌 + SMIL 动画,**打开页面就循环播放,不需要视频播放器**。本次"开机七棒动画版"就是这种做法:

```xml
<svg viewBox="0 0 960 540" width="960" height="540" xmlns="...">
  <rect width="960" height="540" rx="12" fill="#15181d"/>  <!-- 自带深色底,不依赖页面明暗模式 -->
  <circle r="7" fill="#58c4dd" opacity="0">
    <animateMotion dur="6s" repeatCount="indefinite" calcMode="linear"
      keyTimes="0;0.13;0.55;1" keyPoints="0;0;1;1" path="M ..."/>
    <animate attributeName="opacity" values="0;0;1;1;0;0"
      keyTimes="0;0.12;0.13;0.55;0.57;1" dur="6s" repeatCount="indefinite"/>
  </circle>
</svg>
```

编舞要点:所有动画同一 `dur`(一个循环周期),用 `keyTimes` 百分比排相位;`keyPoints="0;0;1;1"` = 先停在起点、移动、再停在终点;区域"点亮"用 `<g opacity="0">` + opacity `values="0;0;1;1"` 阶跃;marker 箭头 id 每个 SVG 加前缀防冲突。验证方法:agent-browser 间隔几秒截两张图,对比元素是否移动/点亮。

## 验证闭环(重要)

`am render` 返回 ✓ 只代表文件写出来了,**不代表画面正常**。本次两个视频坑都是渲染成功但画面空白。硬性做法:用 agent-browser(项目里 `bin/agent-browser`,自带 vendor Chrome)实际打开,截图 + `eval` 查 DOM 尺寸,确认画面非空、动画在动,再交付。

## 集成:clone-from-scratch-tutor

tutorial 产出已从 markdown 改为 am 渲染的单文件 HTML,规范拆成两个 subagent 文件:`skills/clone-from-scrach-tutor/coder-subagent.md` 与 `writer-subagent.md`(后者含「tutorial 产出格式」整节:CLI 位置、面板=固定章节、组件选择、`src=` 代码引用、SVG 必须带 width/height、`am patch` 局部修订)。SKILL.md 只留编排,启动 subagent 时指示它先读对应文件。tutorial HTML 用 `-o` 显式落进 `docs/milestones/m<N>-<slug>/`,不进 `~/.answer-me-with-html/` 默认目录。
