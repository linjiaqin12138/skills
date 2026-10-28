## 反馈

1. answer-me-with-html 集成有优化空间，可能要改成更align这个工具的
    - latex公式渲染
    - 动画视频插入
    - 图片链接插入非自包含
    - 支持清空cache/storage
    - 交互动画插入
    - 不需要图片检查效果，ai只负责markdown，dark mode什么的别馆
    - draw-diagram支持
    - **代码引用插不进省略号**：`src=... lines=A-B` 只能抽连续行段，块体必须留空由 CLI 读文件；不能在中间插 `...` 跳过细节。diff 里单独一行的 `...` / `…` 还会直接报错（gutter 算不出跳了几行）。自顶向下领读目前只能用手写 `title="… · sketch"` + `// ...`，或拆成两段短 `src=`。改造方向：支持 `lines=10-15,40-45` 多段引用，或 `src=` 块内允许省略标记并正确编号。


2. explain-it 做交互动画可以考虑一些模板、减少token浪费、无效darkmodecheck


3. 