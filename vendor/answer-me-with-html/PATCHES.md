# 本地补丁记录（answer-me-with-html vendored 0.5.0）

更新 vendor 副本时逐条核对，别让上游覆盖掉。

## 1. raw-html 白名单放行 MathML（2026-10-22）

**文件**：`scripts/am.mjs`

- `NEVER` 集合移除 `math`
- `PHRASING` 集合加入 MathML 标签：
  `math mrow mi mn mo ms mtext msup msub msubsup mfrac msqrt mroot mstyle mspace mover munder munderover semantics annotation mpadded mphantom menclose merror mtable mtr mtd mmultiscripts mprescripts none mlabeledtr maligngroup malignmark`

**原因**：上游不支持 `$...$`/LaTeX（v0.5.0 无 math 组件），且正文行内的 `<math>` 标签被 NEVER 拦下当文本转义。本项目的 `bin/am-render` 增加预处理（`bin/am-math`，用 `vendor/temml/` 把草稿里的 LaTeX 转成 MathML 临时稿再渲染），需要 sanitizer 放行 MathML 标签才能工作。

**副作用**：
- 页面内嵌的 `#am-source` 是转换后的临时稿（含 MathML 而非 LaTeX）。本项目禁止 `am patch`，无影响。
- 行内手写 `<math>` 现在也会原样通过（以前会被转义），属预期。
