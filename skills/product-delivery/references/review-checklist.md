# 流程与文档 Review 清单

用于检查「文档/流程是否乱」或变更包是否可冻结。不替代 layered-codebase 的代码 review 清单。

## 阻塞项（必须改）

- [ ] 用 Frozen 变更包或过期 requirements 描述**现行**产品行为
- [ ] S/M 小改回写了 Frozen 包正文
- [ ] 闸门未过已有业务实现（或「先跑一版」）
- [ ] 已确认 / 建议 / 待确认未区分，把推测写成确认
- [ ] 对外契约变了，`contracts/current`（或生成物）未同步
- [ ] 长期模块关系/关键时序只存在于变更包，未进 `architecture/`
- [ ] L 无 Demo/操作对齐却宣称交互已定
- [ ] 编码未遵循 layered-codebase（分层明显违规）且以本变更为由开写

## 建议项（可商量）

- [ ] `product/MVP` 过长，混进了实现细节 → 下沉到 architecture/变更包
- [ ] `architecture/flows` 图过多、不关键 → 删减
- [ ] 变更包内空目录/空文件 → 删除
- [ ] Grill 记录散落在聊天未写入 `design-decisions.md`
- [ ] CHANGELOG 缺失但用户可见行为已变
- [ ] L 整页/重做 Demo、explore、契约起草或领域 design 本应默认派却堆在主会话
- [ ] 事事派子代理导致交接 noise（S/小补丁也派）
- [ ] 主会话被整页 HTML/大 diff 污染，挤掉闸门结论与下一问

## 冻结前（L/M）

- [ ] 档位正确（语义 × 改动面）
- [ ] 对应闸门均有明确确认
- [ ] verification / 演示脚本已在真系统走过
- [ ] 状态改为 Frozen；文首可加关联 CHANGELOG 日期
- [ ] 活文档路径在回复中列出
