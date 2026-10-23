# 前置知识清单（M3 概念群补课用）

> 目的：补上理解 Q24~Q37（低通滤波 / action_scale / 观测设计 / sim-to-real / 系统阶数）所需的背景。
> 原则：只学"够解锁问题"的量，不提前学频域、算法推导。链接均已联网核实存在（2026-09-26）。

## 1. 傅立叶思想：信号 = 频率的叠加
解锁：Q30（高频/低通）。只需理解"任何波形可拆成正弦波叠加，低通=调小快的那堆"，不用学公式。

- 【官方双语】形象展示傅里叶变换（3Blue1Brown）：https://www.bilibili.com/video/BV1pW411J7s8/
  （YouTube 原版：https://www.youtube.com/watch?v=spUNpyF58BY）
- 想更进一步：微分方程第四章·但什么是傅立叶级数呢：https://www.bilibili.com/video/BV1vt411N7Ti/
- 交互式网页版（强烈推荐动手玩）：https://www.jezzamon.com/fourier/index.html
- 文字+交互：https://betterexplained.com/articles/an-interactive-guide-to-the-fourier-transform/

## 2. 一阶/二阶系统直觉 ★重点
解锁：Q36/37（系统阶数）、舵机滞后、M5 安全层。看到"阶跃响应、超调、阻尼比"为止，频域/波特图跳过。

DR_CAN《动态系统的建模与分析》系列（B站，动力学博士+机器人工程师，实物+动画、不堆公式）：
- 系列汇总与推荐学习顺序：https://www.bilibili.com/video/BV1KH4y197R1/
- 1_课程介绍：https://www.bilibili.com/video/BV1Bt41187Av/
- 7.5_换个角度分析单位阶跃响应：https://www.bilibili.com/video/BV1is411c7u6/
- 10_二阶系统对初始条件的动态响应：https://www.bilibili.com/video/BV1Sb411L7Fi/
- 二阶系统单位阶跃响应·详细数学推导（数学慎入）：https://www.bilibili.com/video/av50112965
- 13_共振现象·二阶系统频率响应（Q29 滤波绕开共振的背景）：https://www.bilibili.com/video/BV1C4411p73Y/

英文备选：
- Steve Brunton《Control Bootcamp》（华盛顿大学，YouTube 合集）：
  https://www.youtube.com/playlist?list=PLMrJAkhIeNNR20Mz-VpzgfQs5zrYi085m
- Brian Douglas 控制系列：YouTube 搜 "Brian Douglas control systems"

生活锚点回顾：一阶=倒水（无惯性，永不超调），二阶=刹车到停车线（速度记忆→冲过头）。

## 3. MDP → POMDP ★重点
解锁：Q32~36（观测设计、last_action、输入充分性）。抓住两句：马尔可夫性="当前状态够不够决定下一步"；POMDP="观测不是完整状态时的补救思路"。

- 《动手学强化学习》第 3 章 马尔可夫决策过程（文字+代码，免费）：
  https://hrl.boyuai.com/chapter/1/%E9%A9%AC%E5%B0%94%E5%8F%AF%E5%A4%AB%E5%86%B3%E7%AD%96%E8%BF%87%E7%A8%8B/
- POMDP 讲义（英文 slide，看前几页定义即可）：
  https://people.cs.umass.edu/~barto/courses/cs687/Partial_Observability.pdf

## 4. RL 训练循环全景
解锁：训练-推理闭环一致性（action_scale 契约、低通系数锁定）。目标只一句："策略是数据统计出来的，不是设计出来的"。

- 李宏毅《深度强化学习》1-8 课（Policy Gradient / PPO，中文，B站）：
  https://www.bilibili.com/video/BV124411S7au/

## 5.（进 M4 前再看）Sim-to-real / 域随机化
解锁：Q29/31、M4 MuJoCo 验收。搜 "domain randomization sim2real robot" 中文科普即可，届时就着代码讲。

## 建议路径
3B1B 傅立叶(30min) → DR_CAN 一阶/二阶(第 7.5、10 集) → boyuai MDP 章 + 李宏毅 DRL 第 1 课(半天) → 重读 questions.md Q29~Q37
