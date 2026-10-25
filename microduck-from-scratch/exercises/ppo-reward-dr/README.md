# 练习：改奖励权重并预测现象

学习者练习，不是正式实现。先修主题：`tutor/prerequisites/ppo-reward-dr.md`。

不训网络、不上 GPU。脚本只按你写的权重，给三种「行为快照」算总分——体会「评分标准变了，谁会被当成好策略」。

## 步骤（必须按序）

1. 打开 `weights.json`，先**不要改**。三项：`track_lin`（前进跟踪）、`upright`（直立）、`action_rate`（抖动；负权重=惩罚）。
2. 在 `prediction.md` 写下：若把 `track_lin` 改成 `5.0`、`upright` 改成 `0.0`，三种行为（`walk` / `roll` / `twitch`）谁总分最高、为什么。行为原始分在 `behaviors.json`（只读）。
3. 再改 `weights.json`，在项目根运行：

```bash
python3 exercises/ppo-reward-dr/score_behaviors.py
```

4. 把终端输出贴回家教；对照预测，错了写一句错因。

参考解答（做完再看）：`weights_solution.json`。
