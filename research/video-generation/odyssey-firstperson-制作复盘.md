# 《奥德赛》第一人称版视频制作复盘（odyssey-firstperson）

> 记录日期：2026-09-10
> 成片：`~/Project/OpenMontage/projects/odyssey-firstperson/renders/odyssey-firstperson.mp4`（10'01"，1920×1080，223MB）
> 生产会话：@session:default/20260910_103026_49f3d2（中断恢复 + 交付）
> 相关文档：`odyssey-firstperson-交接文档.md`（生产过程全貌）、`comfyui-5090-openmontage-环境搭建指南.md`（通用坑）

---

## 1. 与前几次任务的不同点

| 维度 | v3（trojan-odyssey-v3，5'33"） | 本次（odyssey-firstperson，10'01"） |
|------|-------------------------------|--------------------------------------|
| 叙事视角 | 第三人称讲解 | **奥德修斯第一人称自述**（18 节 2515 字） |
| 旁白声音 | IndexTTS 克隆 zh_man_sichuan（被评"太浮夸"） | **male_02 克隆**（4 候选中 pitch_std=11Hz 最稳、均值 84Hz 最低沉；emotion_alpha=0.6, temperature=0.6） |
| 画风 | 希腊陶瓶画 | **吉卜力手绘水彩**（FLUX.2 prompt 风格化） |
| 动态占比 | 每场景仅前 5s 真动态 + 末帧 Ken Burns | **链式 i2v**：clip a 末帧 → clip b 参考图 → clip c，长场景 3 条短场景 2 条，共 **48 条 Wan i2v**，动态占比大幅提升 |
| 时长控制 | 自然产出 5'33" | 目标 10 分钟：TTS `duration_factor=1.28` 把旁白放慢到 ~601s 来撑时长 |
| 生产连续性 | 一台实例一次跑完 | **中断两次、换机两次**：5090 被重置成 3060 → 停机保留磁盘；重开后宿主机 GPU 驱动被邻居租户搞挂 → 再换机。全程靠断点续跑（跳过已存在资产） |
| 合成 | 本机 FFmpeg | **服务器上 compose.py**（FFmpeg，资产不用拉回本地再合成） |
| 通知 | 无 | 看门狗 cron 监控 + 完成后 wechat-notify cron 发微信；中途发现 gateway 静默故障 |

## 2. 本次新踩的坑（v3 记录和指南之外的）

1. **produce.py 参数版本漂移**：本地副本的 `duration_factor` 还是 1.0（旧实例上改成 1.28 后没同步回本机）。换机恢复时差点用旧参数重跑 TTS。**教训：在生产实例上改过的脚本，必须立刻同步回本机/git。**
2. **OpenMontage GitHub HEAD 过期**：上游仓库的 workflows/metadata/comfyui_image.py 仍引用 nvfp4/fp4 模型名（有 eager 反量化 bug）。新实例克隆仓库后**必须用本地已补丁文件覆盖**：`tools/_comfyui/workflows/*.json`、`tools/_comfyui/metadata.py`、`tools/graphics/comfyui_image.py`。
3. **pkill 自杀陷阱升级版**：同一条 ssh 命令里只要命令行文本包含 "produce.py" 字样（哪怕是另一个 grep/cat 的参数），`pkill -f` 都会把这条 ssh 自己的 remote shell 杀掉，`[p]roduce.py` 方括号技巧也救不了。**杀进程永远单独一条 ssh，且命令行里不提任何相关路径。**
4. **VAST 宿主机 GPU 被邻居租户搞挂**：97.70.195.221 实例容器内 nvidia-smi 报错/驱动无响应，容器内任何手段无解（指南坑 11）。VAST 是多租户共享宿主机驱动，**遇到驱动级故障不要排查容器，直接换机**。
5. **VAST 容器里 NVENC 被禁**：compose.py 用 `h264_nvenc` 报 `OpenEncodeSessionEx failed: unsupported device`（容器没透传编码器）。改 **libx264**，192 核 CPU 渲染 10 分钟视频也就几分钟，不必纠结 GPU 编码。
6. **断点续跑跳过 0 字节坏分段**：首轮 nvenc 失败留下的 0 字节输出文件被续跑逻辑当作"已完成"跳过，导致 concat 失败。**续跑前先 `find -size 0` 清掉空文件，或让存在性检查加上 size>0。**
7. **cron 自报 ok ≠ 微信真的送达**：wechat-notify 手动触发返回 ok，但 gateway 实际已静默故障（服务 active、日志停在中午），消息根本没发出去。重启 `hermes-gateway` 后重发成功。**验证投递必须查 `~/.hermes/logs/gateway.log` 里的 `[Weixin] Sending response` 记录。**

## 3. 用户评价（2026-09-10，看完成片后）

**总体：还行，能看，但差很多意思。**

1. **模板味太重**：每个场景都是同一个套路——一段场景动画 + 尾帧图片缓慢放大（zoompan）补足时长。18 个场景看下来节奏完全可预测。
2. **一致性只有风格层面**：人物长相、场景布局前后对不上（FLUX 逐场景独立生图 + i2v 链式只传末帧，没有角色/场景锚定），统一的只是"吉卜力水彩"这个画风。
3. **文字稿太正经**：第一人称自述写出来的稿子太"标准"，没有人类手搓时会有的那种独特味道（口语感、个人化的停顿和小情绪）。

### 对应的下版改进方向

- **打破模板节奏**：场景间变化动态占比和运镜方式；长短片段交替；考虑插入纯图静止 + 音效的"呼吸段"，而不是每个场景都 zoompan 补满。
- **角色/场景一致性**：主角做一张定妆参考图，全程用角色 LoRA 或 IPAdapter/参考图条件生图；场景建立固定几张关键场景图复用。Wan 2.x 的 reference-to-video / FLUX redux 类能力值得试。
- **稿子人味**：先让人（或更口语化的 prompt）写一版带个人语气的初稿再改，而不是直接生成"正确"的叙述文；允许短句、重复、题外话。

## 4. 成本与待办

- 全程本地 ComfyUI 5090，**零 API 成本**（仅 VAST 租金，含两次换机浪费的时间）。
- 生产实例 122.117.118.68 上有全部资产 + compose.py + 成片副本；本机 renders/ 有最终成片。**看完确认后记得销毁 VAST 实例停止计费**（销毁即丢资产，重要文件已在本机）。
