// lerp（线性插值）：从 start 到 end，第 t 拍（共 N 拍）走百分之 t/N
const lerp = (a, b, t) => a + (b - a) * t;

const start = 0.0, home = -0.4579, N = 100; // 一个关节：躺平角 → home 角
for (const tick of [0, 50, 100]) {
  const t = Math.min(tick, N) / N;
  console.log(`第 ${tick} 拍（t=${t}）: 目标角 ${lerp(start, home, t).toFixed(4)}`);
}
