# OpenClaw 安装记录

日期：2026-08-20
系统：Ubuntu (Linux 6.8.0, x86_64)

## 背景

OpenClaw 要求 Node.js 版本满足 `>=22.22.3 <23 || >=24.15.0 <25 || >=25.9.0`。
本机原默认版本为 Node v22.14.0（通过 nvm 管理），不满足要求，需先升级 Node。

## 执行的命令

### 1. 检查环境

```bash
node --version          # v22.14.0（不满足 OpenClaw 要求）
npm --version           # 10.9.2
npm config get prefix   # /home/jqlin/.nvm/versions/node/v22.14.0（确认使用 nvm）
```

### 2. 通过 nvm 安装 Node 24 并设为默认

```bash
export NVM_DIR="$HOME/.nvm"
. "$NVM_DIR/nvm.sh"
nvm install 24            # 安装 v24.19.0
nvm alias default 24      # 设为默认版本，新终端自动生效
node --version            # v24.19.0
```

### 3. 全局安装 OpenClaw

第一次直接安装（部分依赖的原生构建脚本被 npm 拦截）：

```bash
npm install -g openclaw@latest
```

按 npm 提示补装一次，允许相关依赖的 install 脚本执行：

```bash
npm install -g --allow-scripts=openclaw,@google/genai,protobufjs,tree-sitter-bash openclaw@latest
```

### 4. 验证安装

```bash
openclaw --version        # OpenClaw 2026.7.1-2 (0790d9f)
```

## 后续步骤（需在终端交互完成，本次未执行）

```bash
openclaw onboard --install-daemon   # 配置 auth、网关，安装后台服务
openclaw gateway status             # 检查网关状态
openclaw dashboard                  # 打开控制台 http://127.0.0.1:18789/
```

## 卸载方法

### 1. 卸载网关服务和本地数据（CLI 保留）

```bash
openclaw uninstall            # 删除网关服务 + ~/.openclaw 本地数据
openclaw daemon uninstall     # 仅卸载 systemd 服务（执行过 onboard --install-daemon 时）
```

### 2. 卸载 CLI 本体

```bash
npm uninstall -g openclaw
```

### 3.（可选）清理 Node 24（若仅为 OpenClaw 安装）

```bash
nvm alias default 22          # 默认版本改回 22.14.0
nvm uninstall 24              # 删除 Node 24.19.0
```

> 注：本次安装未执行 `openclaw onboard`，系统中无守护进程，也未生成 `~/.openclaw`
> 数据，实际只需第 2 步（可选第 3 步）即可卸载干净。

## 参考资料

- [OpenClaw Getting Started](https://openclaws.io/docs/start/getting-started/)
- [OpenClaw 官方文档](https://docs.openclaw.ai/)
