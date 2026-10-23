# claude-replay（vendored 0.11.0）

把 AI 编码 agent 的会话日志转成**单个自包含 HTML 回放页**（无外部依赖，可发邮件 / 嵌 iframe）。

- 上游：https://github.com/es617/claude-replay
- 许可：MIT（见本目录 `LICENSE`）
- 本地改动：**无**。`bin/`、`src/`、`template/`、`package.json`、`README.md`、`LICENSE` 均为 npm 包 `claude-replay@0.11.0` 的原始内容，未打补丁。

## 收录方式

从 npm tarball 解包（而非全局安装），保证仓库是唯一来源：

```bash
npm pack claude-replay@0.11.0
tar xzf claude-replay-0.11.0.tgz
cp -r package/bin package/src package/template package/{LICENSE,package.json,README.md} vendor/claude-replay/
```

包本身零运行时依赖（esbuild 只是构建模板用的 devDependency），Node 18+ 即可跑，因此源码可以直接进 git，不需要 `node_modules`。

## 本地入口

统一走项目根目录的 `bin/claude-replay`（wrapper，指向本目录的 `bin/claude-replay.mjs`），额外加了 Kimi Code 会话查找：

```bash
bin/claude-replay --list              # 列出 ~/.kimi-code/sessions 下的会话
bin/claude-replay <session-id>        # 回放指定 Kimi Code 会话，输出到 replays/
bin/claude-replay <wire.jsonl 路径>   # 回放指定转录文件
bin/claude-replay                     # 启动 Web 编辑器
bin/claude-replay --list-themes       # 其余参数原样透传给上游 CLI
```

## 更新版本时

1. `npm pack claude-replay@<新版本>`，按上面的命令重新解包覆盖
2. 更新本文件顶部的版本号
3. 跑一遍 `bin/claude-replay --list-themes` 和一次真实会话回放，确认 Kimi Code 解析正常
4. 检查 `src/formats/kimi-code.mjs` 是否有上游格式变更（Kimi Code 的 `wire.jsonl` 事件类型若变动，这里是首要受影响点）
