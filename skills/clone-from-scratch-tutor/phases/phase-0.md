# Phase 0 · 侦察

一次做完，然后停下等用户说继续。

## 建立目录

在 `<当前目录>/<项目名>-from-scratch/` 用本 skill 的模板落盘：

| 模板 | 复制到 |
|---|---|
| `templates/README.md` | `README.md` |
| `templates/book-index.md` | `book/index.md` |
| `templates/STATE.md` | `tutor/STATE.md` |
| `templates/project-prefs.md` | `tutor/user-prefs.md` |
| `templates/features.md` | `spec/features.md` |
| `templates/deviations.md` | `spec/deviations.md` |

创建这些空目录：`book/arch/src/`、`book/concepts/`、`decisions/`、`accept/`、`exercises/`、`tutor/phases/`、`tutor/runs/`、`tutor/prerequisites/`。再写 `impl/README.md`，内容一行：实现从第一个 Bite 开始。不要把上游源码拷进来。`exercises/` 只放学习者练习，不放正式实现。

把上游加为 submodule，检出要对照的那个 commit，写进 `tutor/STATE.md` 和 `README.md`：

```bash
git submodule add <仓库地址> upstream
git -C upstream checkout <commit>
```

加不上就停下，不要改到普通克隆后把上游历史提交进本仓库。

## 这一轮的回复

只写这四项，并归档到 `tutor/phases/phase-0.md`：

- 项目一句话：什么输入变成什么输出
- 顶层目录各自干什么
- 技术栈、数据来源、外部服务
- 核心主循环在哪个文件

读不到的标【合理推断】。更新 `tutor/STATE.md` 的阶段、下一步（Phase 1）和上游 commit。列出尚未从代码确认的推断。停下。
