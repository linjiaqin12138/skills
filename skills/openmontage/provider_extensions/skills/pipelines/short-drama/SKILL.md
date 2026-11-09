# Short Drama Director

## Asset pack

1. 检查角色、场景、道具和关键空间首图；缺图时先走 image-generation。
2. 资产分固定模板与单集内容，探索素材与正式素材分开。
3. 生成后实际查看角色脸型、服装和道具，不以文件存在代替验收。

## Scene plan

1. 每镜一个主要动作，独立 duration；短镜头可合并并写清硬切点。
2. `prompts.md` 使用 `## 任务 <id>` 和 text 代码块保存中文提示词。
3. `tasks.json` 只保存 provider、model、duration、references、real_person 和 provider_type。
4. 向用户展示完整分镜、参考图和预算，批准后才能生成。

## Assets

1. 每个任务可以覆盖模型；真人审核失败时换 Wan 等替代路径。
2. 已存在且非空的镜头跳过；`--shot` 只运行指定任务，`--force` 重跑。
3. 保存 task ID，超时恢复同一任务，避免重复扣费。
4. 审核门方向、角色出现时机、道具一致性、视线和动作对象。

## Edit 与 compose

1. 按任务顺序拼接，统一帧率和音频采样率。
2. `episode.ass` 存在时烧录字幕，字幕时间以实际动作为准。
3. 可选 `--bgm`，保留对白和关键音效，音乐自动闪避。
4. 输出 final.mp4、每镜成本与返修记录，最终由用户验收。
