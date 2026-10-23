# Phase 1 · 自顶向下拆解（归档）

## 确认的推断
- NUM_JOINTS=15，策略输出 14 维（嘴除外）；观测统一 61 维；技能=多 ONNX 网络按优先级链选择：roulade > kick > ground_pick > sit/rise > stand(按速度阈值) > walk
- 50 Hz 与策略训练频率绑定（传感器数据年龄必须与训练分布一致）

## 四张决策卡片
1. **为什么不用 ROS**（见 questions.md Q1）
2. **按故障域切服务**：configd/updaterd/btd 必须在 robotd 死时活着（恢复路径）；跨服务读取一律 last-value-wins 缓存，控制循环永不阻塞等别的服务
3. **意图而非关节角**：客户端只发 intent；跌倒检测/限位/安全姿态收在 robotd 内不可绕过；deadman 停指令即停
4. **updaterd 最先做 + 整包换目录**：失败免费期前置更新风险；releases/<ver>/ + current symlink 原子切换 + robot.health 健康门 + 自动回滚

## 模块契约要点
- duck-control：纯计算库，"不碰电机"由编译器强制（无 tokio 无 socket）
- duck-ipc-proto：只允许 serde/serde_json/semver 依赖（救援路径也要用）
- btd/padd/mediad：传输适配器，own nothing，可替换不影响行为
