// 观测向量：把传感器压成策略训练时见过的那条定长数组。
// 嘴不进策略。跳过它和顺序填满，差一格，后面的关节全部串位。

const MOUTH = 9;
const home = [0, -0.1, -0.5, 0, 0.4, 0.3, 0.3, 0, 0, 0, 0, 0.1, 0.5, 0, -0.4];
const pos = home.slice();
pos[0] += 0.25; // 左髋 yaw 偏离 home
pos[MOUTH] += 1.0; // 嘴张大，策略不该看见

const policySlots = (values) =>
  values.filter((_, i) => i !== MOUTH).map((v, i) => {
    const homeSlot = i < MOUTH ? home[i] : home[i + 1];
    return +(v - homeSlot).toFixed(2);
  });

const obs = [0, 0, 0, 0, 0, -1, ...policySlots(pos)];
console.log("跳过嘴，前 8 维:", obs.slice(0, 8).join(","));
console.log("位置块是否含 1.0（嘴）:", obs.includes(1));

const naive = pos.map((v, i) => +(v - home[i]).toFixed(2));
console.log("不跳过嘴，下标 9 起:", naive.slice(9, 12).join(","));
console.log("跳过嘴，下标 9 起:", policySlots(pos).slice(9, 12).join(","));
