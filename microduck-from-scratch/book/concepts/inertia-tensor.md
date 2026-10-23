---
template: doc
title: 惯量张量：描述一个刚体的转动惯性，需要哪些数据
subtitle: 从手算一只哑铃，到「三个数加一个朝向」
---

## A 结论与验收承诺

推一个箱子走直线，只要知道质量；让它在空中转起来，质量一个数就不够了。完整描述转动惯性，需要 3 个数加一个朝向——也就是 3x3 惯量张量对角化后的三个主惯量，加一组主轴朝向。

读完本页，你能独立做三件事：

1. 手算一只哑铃绕某条轴的转动惯量，判断 3x3 矩阵里哪些元素该为零。
2. 预测坐标系绕 z 轴转过某个角度后，矩阵里哪几个数会变。
3. 解释「三个对角值加一个朝向」为什么不丢信息。

## B 转动惯量是「质量乘距离的平方」

转动惯量衡量绕某条轴转动的难易，定义是把每个质量块到轴的垂直距离平方乘上质量，再全部加起来。绕 x 轴写作：

$$I_{xx} = \int (y^2+z^2)\,\mathrm{d}m$$

绕 y 轴、绕 z 轴对称换元即可。单位是 kg·m²。

手算一例：两个 1 kg 质量块在 x = ±0.5 m，中间一根不计质量的杆。

- 绕 x 轴：质量块在轴上，距离为 0，$I_{xx}=0$。
- 绕 y 轴：每个质量块贡献 $1\times0.5^2=0.25$，合计 $I_{yy}=0.5$。
- 绕 z 轴：同理 $I_{zz}=0.5$。

写成矩阵就是 `[[0,0,0],[0,0.5,0],[0,0,0.5]]`。

一个常见误读：$I=mr^2$ 只在「单个质量点绕固定轴」时成立，不是通用公式。通用算法永远是「逐块算距离平方，再加总」。

```svg
<svg width="640" height="320" viewBox="0 0 640 320" xmlns="http://www.w3.org/2000/svg" font-family="var(--font-sans)">
  <defs>
    <marker id="arrx" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto"><path d="M0,0 L6,3 L0,6 Z" fill="var(--ink-3)"/></marker>
    <marker id="arry" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto"><path d="M0,0 L6,3 L0,6 Z" fill="var(--ink-3)"/></marker>
  </defs>
  <line x1="60" y1="150" x2="596" y2="150" stroke="var(--ink-3)" stroke-width="2" marker-end="url(#arrx)"/>
  <line x1="320" y1="270" x2="320" y2="34" stroke="var(--ink-3)" stroke-width="2" marker-end="url(#arry)"/>
  <line x1="120" y1="150" x2="520" y2="150" stroke="var(--ink-3)" stroke-width="8" stroke-linecap="round"/>
  <circle cx="120" cy="150" r="17" fill="var(--accent)" fill-opacity="0.85"/>
  <circle cx="520" cy="150" r="17" fill="var(--accent)" fill-opacity="0.85"/>
  <text x="120" y="155" text-anchor="middle" font-size="11" fill="var(--bg)">1 kg</text>
  <text x="520" y="155" text-anchor="middle" font-size="11" fill="var(--bg)">1 kg</text>
  <line x1="120" y1="158" x2="120" y2="176" stroke="var(--ink-2)" stroke-width="1" stroke-dasharray="4 3"/>
  <line x1="320" y1="158" x2="320" y2="176" stroke="var(--ink-2)" stroke-width="1" stroke-dasharray="4 3"/>
  <line x1="120" y1="176" x2="320" y2="176" stroke="var(--ink-2)" stroke-width="1.5" stroke-dasharray="4 3"/>
  <line x1="320" y1="176" x2="520" y2="176" stroke="var(--ink-2)" stroke-width="1.5" stroke-dasharray="4 3"/>
  <line x1="520" y1="158" x2="520" y2="176" stroke="var(--ink-2)" stroke-width="1" stroke-dasharray="4 3"/>
  <text x="220" y="194" text-anchor="middle" font-size="12" fill="var(--ink-2)">r = 0.5 m</text>
  <text x="420" y="194" text-anchor="middle" font-size="12" fill="var(--ink-2)">r = 0.5 m</text>
  <text x="600" y="146" font-size="14" fill="var(--ink-3)">x</text>
  <text x="328" y="42" font-size="14" fill="var(--ink-3)">y</text>
  <text x="24" y="246" font-size="13" fill="var(--ink)">绕 y 轴：每个贡献 1 × 0.5² = 0.25</text>
  <text x="24" y="266" font-size="13" fill="var(--ink)">绕 x 轴：质量块在轴上，距离 0</text>
  <text x="24" y="30" font-size="12" fill="var(--ink-3)">杆不计质量</text>
</svg>
```

图注（F1）：从 +z 侧俯视 xy 平面，视线沿 -z。灰色箭头是坐标轴 x 与 y，本图里它们同时是两条转轴。灰杆连接两个主色质量块，虚线标出每个质量块到 y 轴的垂直距离。距离按比例绘制。

## C 矩阵里为什么有六个数

惯量张量是对称矩阵（$I_{xy}=I_{yx}$），所以 9 个格子里只有 6 个独立元素。对角元之外的叫惯量积，定义是：

$$I_{xy}=-\int xy\,\mathrm{d}m$$

$I_{xz}$、$I_{yz}$ 同型，换一对坐标即可。惯量积用大白话说：质量分布歪着时，绕 x 轴转也会带出 y 方向角动量。这种现象叫「串轴」——拧一个轴，别的轴跟着动。

| 元素 | 物理含义 | 怎么算 | 什么时候为零 |
|---|---|---|---|
| $I_{xx}$ | 绕 x 轴转的难易 | $\int (y^2+z^2)\,\mathrm{d}m$ | 质量全在 x 轴上 |
| $I_{yy}$ | 绕 y 轴转的难易 | $\int (x^2+z^2)\,\mathrm{d}m$ | 质量全在 y 轴上 |
| $I_{zz}$ | 绕 z 轴转的难易 | $\int (x^2+y^2)\,\mathrm{d}m$ | 质量全在 z 轴上 |
| $I_{xy}$ | x 与 y 之间的串轴强度 | $-\int xy\,\mathrm{d}m$ | 质量分布关于 x 或 y 轴对称 |
| $I_{xz}$ | x 与 z 之间的串轴强度 | $-\int xz\,\mathrm{d}m$ | 质量分布关于 x 或 z 轴对称 |
| $I_{yz}$ | y 与 z 之间的串轴强度 | $-\int yz\,\mathrm{d}m$ | 质量分布关于 y 或 z 轴对称 |

```svg
<svg width="640" height="340" viewBox="0 0 640 340" xmlns="http://www.w3.org/2000/svg" font-family="var(--font-sans)">
  <defs>
    <marker id="f2x" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto"><path d="M0,0 L6,3 L0,6 Z" fill="var(--accent)"/></marker>
    <marker id="f2y" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto"><path d="M0,0 L6,3 L0,6 Z" fill="var(--ink-2)"/></marker>
    <marker id="f2p" markerWidth="9" markerHeight="9" refX="7" refY="3.5" orient="auto"><path d="M0,0 L7,3.5 L0,7 Z" fill="var(--accent)"/></marker>
  </defs>
  <line x1="100" y1="160" x2="580" y2="160" stroke="var(--accent)" stroke-width="1.5" marker-end="url(#f2x)"/>
  <line x1="260" y1="320" x2="260" y2="16" stroke="var(--ink-2)" stroke-width="1.5" marker-end="url(#f2y)"/>
  <text x="586" y="156" font-size="14" fill="var(--accent)">x</text>
  <text x="268" y="24" font-size="14" fill="var(--ink-2)">y</text>
  <circle cx="410" cy="115" r="8" fill="var(--ink)"/>
  <text x="422" y="112" font-size="12" fill="var(--ink)">P1 (1.0, 0.3), z = 0</text>
  <circle cx="170" cy="40" r="8" fill="var(--ink)"/>
  <text x="182" y="44" font-size="12" fill="var(--ink)">P2 (-0.6, 0.8), z = 0.2</text>
  <circle cx="290" cy="235" r="8" fill="var(--ink)"/>
  <text x="302" y="239" font-size="12" fill="var(--ink)">P3 (0.2, -0.5), z = 0.9</text>
  <line x1="283" y1="109" x2="297" y2="109" stroke="var(--bg)" stroke-width="5"/>
  <line x1="290" y1="102" x2="290" y2="116" stroke="var(--bg)" stroke-width="5"/>
  <line x1="283" y1="109" x2="297" y2="109" stroke="var(--ink-3)" stroke-width="1"/>
  <line x1="290" y1="102" x2="290" y2="116" stroke="var(--ink-3)" stroke-width="1"/>
  <text x="302" y="100" font-size="12" fill="var(--ink-3)">质心 (0.2, 0.34)</text>
  <line x1="480" y1="270" x2="480" y2="190" stroke="var(--accent)" stroke-width="2.5" marker-start="url(#f2p)" marker-end="url(#f2p)"/>
  <text x="492" y="238" font-size="12" fill="var(--accent)">绕 x 转带出的</text>
  <text x="492" y="256" font-size="12" fill="var(--accent)">y 方向角动量分量</text>
</svg>
```

图注（F2）：从 +z 侧俯视 xy 平面，视线沿 -z。三个深色点是质量点，坐标按比例，z 值写在各自标注里。主色箭头是 x 轴，灰色箭头是 y 轴，十字标记是质心。主色双头粗箭头表示绕 x 转时会带出 y 方向的角动量分量。它只示意存在这个分量，不代表正负方向。

## D 换个坐标系看同一个刚体

同一个刚体，坐标轴一转，惯量矩阵跟着变：$I' = R^\mathsf{T} I R$。$R$ 是旋转矩阵，绕 z 轴转 θ 时：

$$R=\begin{pmatrix}\cos\theta&-\sin\theta&0\\ \sin\theta&\cos\theta&0\\0&0&1\end{pmatrix}$$

下面这只「歪哑铃」由三个质点组成，质心 (0.2, 0.34, 0.26)：

- m = 2，位置 (1.0, 0.3, 0.0)
- m = 2，位置 (-0.6, 0.8, 0.2)
- m = 1，位置 (0.2, -0.5, 0.9)

它在质心系里的惯量矩阵是：

```
[[1.684  0.8    0.32 ]
 [0.8    3.112  0.572]
 [0.32   0.572  3.692]]
```

拖动滑块，看矩阵随 θ 实时变化：

```html
<div style="border:1px solid var(--line);border-radius:8px;padding:16px;background:var(--bg);font-family:var(--font-sans);max-width:640px">
  <div style="margin-bottom:10px">
    <label for="th" style="font-size:14px;color:var(--ink)">绕 z 轴转角 θ：</label>
    <input id="th" type="range" min="0" max="90" step="1" value="0" style="width:240px;vertical-align:middle">
    <span id="thv" style="font-family:var(--font-mono);font-size:14px;color:var(--accent);margin-left:8px">0°</span>
    <button id="rst" style="margin-left:14px;font-size:13px;padding:2px 10px;border:1px solid var(--line);border-radius:5px;background:var(--fill);color:var(--ink);cursor:pointer">重置</button>
  </div>
  <table id="mtab" style="border-collapse:collapse;font-family:var(--font-mono);font-size:13px;margin-bottom:10px"></table>
  <div style="font-size:12px;color:var(--ink-3);margin-bottom:10px">带底色 = 对角元（绕轴难易）；无底色 = 副对角元（串轴强度）</div>
  <div id="bars" style="font-family:var(--font-mono);font-size:12px;color:var(--ink-2)"></div>
  <div style="font-size:13px;color:var(--ink-2);margin-top:10px">提示：不管 θ 怎么拖，I<sub>zz</sub> 恒为 3.692——绕 z 转不改变质量块到 z 轴的距离。</div>
  <div style="font-size:13px;color:var(--ink);margin-top:8px">观察任务：① 拖到 90°，看 I<sub>xx</sub> 与 I<sub>yy</sub> 互换、I<sub>xy</sub> 变号；② 任意拖，确认 I<sub>zz</sub> 不变；③ 试着找副对角元同时为零的角度——找不到，除非转到主轴系（一般三维旋转，本滑块只绕 z 一根轴）。</div>
</div>
<script>
(function(){
  var I=[[1.684,0.8,0.32],[0.8,3.112,0.572],[0.32,0.572,3.692]];
  var th=document.getElementById('th'),thv=document.getElementById('thv'),
      tab=document.getElementById('mtab'),bars=document.getElementById('bars');
  function calc(deg){
    var t=deg*Math.PI/180,c=Math.cos(t),s=Math.sin(t);
    var R=[[c,-s,0],[s,c,0],[0,0,1]];
    var M=[[0,0,0],[0,0,0],[0,0,0]],i,j,k,l;
    for(i=0;i<3;i++)for(j=0;j<3;j++){var v=0;
      for(k=0;k<3;k++)for(l=0;l<3;l++)v+=R[k][i]*I[k][l]*R[l][j];
      M[i][j]=Math.abs(v)<5e-4?0:v;}
    return M;
  }
  function render(){
    var deg=+th.value;thv.textContent=deg+'°';
    var M=calc(deg),head=['x','y','z'];
    var cell='border:1px solid var(--line);padding:6px 12px;';
    var html='<tr><td style="'+cell+'"></td>';
    for(var c=0;c<3;c++) html+='<td style="'+cell+'text-align:center;color:var(--ink-3)">'+head[c]+'</td>';
    html+='</tr>';
    for(var i=0;i<3;i++){html+='<tr><td style="'+cell+'color:var(--ink-3)">'+head[i]+'</td>';
      for(var j=0;j<3;j++){
        var diag=i===j;
        html+='<td style="'+cell+'text-align:right;'+
          (diag?'background:var(--accent-bg);color:var(--ink);font-weight:bold':'color:var(--ink-2)')+
          '">'+M[i][j].toFixed(4)+'</td>';
      }
      html+='</tr>';}
    tab.innerHTML=html;
    var names=['|I_xy|','|I_xz|','|I_yz|'],vals=[Math.abs(M[0][1]),Math.abs(M[0][2]),Math.abs(M[1][2])],bh='';
    for(var n=0;n<3;n++){
      var w=Math.round(vals[n]/1.2*220);
      bh+='<div style="display:flex;align-items:center;gap:8px;margin:3px 0">'+
        '<span style="width:52px">'+names[n]+'</span>'+
        '<span style="display:inline-block;height:10px;width:'+w+'px;background:var(--accent);border-radius:2px"></span>'+
        '<span>'+vals[n].toFixed(4)+'</span></div>';
    }
    bars.innerHTML=bh;
  }
  th.addEventListener('input',render);
  document.getElementById('rst').addEventListener('click',function(){th.value=0;render();});
  render();
})();
</script>
```

图注（F3）：滑块把坐标系绕 z 轴转 θ，矩阵数值由 $I' = R^\mathsf{T} I R$ 实时算出。行和列的表头是矩阵对应的 x、y、z 分量。带底色格是对角元，无底色格是副对角元，条形是三个副对角元的绝对值。

这四组数由同一条公式用 numpy 实算，拖滑块到对应角度可互相验证：

| θ | $I_{xx}'$ | $I_{yy}'$ | $I_{zz}'$ | $I_{xy}'$ | $I_{xz}'$ | $I_{yz}'$ |
|---|---|---|---|---|---|---|
| 0° | 1.684 | 3.112 | 3.692 | 0.8 | 0.32 | 0.572 |
| 30° | 2.7338 | 2.0622 | 3.692 | 1.0183 | 0.5631 | 0.3354 |
| 45° | 3.198 | 1.598 | 3.692 | 0.714 | 0.6307 | 0.1782 |
| 90° | 3.112 | 1.684 | 3.692 | -0.8 | 0.572 | -0.32 |

## E 主轴：转到不串轴的方向

总存在一个让矩阵变对角的朝向。数学依据是谱分解：实对称矩阵一定能分解出一组互相垂直的特征方向，旋转到这组方向后，$R^\mathsf{T} I R=\mathrm{diag}(I_1,I_2,I_3)$。这组方向叫主轴，特征值叫主惯量，特征向量就是主轴方向。推导略去，记住结论即可。

对上面的歪哑铃跑一遍配套脚本 `python3 assets/inertia_demo.py`（需 numpy），实测输出：

```
主惯量（特征值升序）：[1.3242  2.9198  4.244 ]
主轴矩阵（每列一根轴）：
[[ 0.9156  0.2909  0.2775]
 [-0.4011  0.7072  0.5822]
 [-0.0268 -0.6444  0.7642]]
R.T @ I @ R：
[[1.3242  0.      0.    ]
 [0.      2.9198  0.    ]
 [0.      0.      4.244 ]]
```

转到主轴系后，六个副对角元全部归零，串轴消失。

```svg
<svg width="720" height="320" viewBox="0 0 720 320" xmlns="http://www.w3.org/2000/svg" font-family="var(--font-sans)">
  <defs>
    <marker id="f4ax" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto"><path d="M0,0 L6,3 L0,6 Z" fill="var(--ink-3)"/></marker>
    <marker id="f4a" markerWidth="9" markerHeight="9" refX="7" refY="3.5" orient="auto"><path d="M0,0 L7,3.5 L0,7 Z" fill="var(--ink-2)"/></marker>
  </defs>
  <line x1="250" y1="180" x2="319" y2="190" stroke="var(--ink-3)" stroke-width="1.2" marker-end="url(#f4ax)"/>
  <line x1="250" y1="180" x2="250" y2="117" stroke="var(--ink-3)" stroke-width="1.2" marker-end="url(#f4ax)"/>
  <line x1="250" y1="180" x2="313" y2="155" stroke="var(--ink-3)" stroke-width="1.2" marker-end="url(#f4ax)"/>
  <text x="324" y="206" font-size="12" fill="var(--ink-3)">x</text>
  <text x="256" y="112" font-size="12" fill="var(--ink-3)">y</text>
  <text x="319" y="150" font-size="12" fill="var(--ink-3)">z</text>
  <circle cx="343" cy="167" r="7" fill="var(--ink)"/>
  <text x="355" y="162" font-size="11" fill="var(--ink-2)">P1</text>
  <circle cx="204" cy="83" r="7" fill="var(--ink)"/>
  <text x="192" y="96" font-size="11" fill="var(--ink-2)">P2</text>
  <circle cx="361" cy="198" r="7" fill="var(--ink)"/>
  <text x="373" y="201" font-size="11" fill="var(--ink-2)">P3</text>
  <line x1="192" y1="122" x2="332" y2="200" stroke="var(--accent)" stroke-width="2.5"/>
  <line x1="212" y1="99" x2="312" y2="223" stroke="var(--accent)" stroke-width="2.5" stroke-dasharray="8 5"/>
  <line x1="199" y1="210" x2="325" y2="112" stroke="var(--accent)" stroke-width="2.5" stroke-dasharray="2 4"/>
  <line x1="253" y1="161" x2="271" y2="161" stroke="var(--bg)" stroke-width="5"/>
  <line x1="262" y1="152" x2="262" y2="170" stroke="var(--bg)" stroke-width="5"/>
  <line x1="253" y1="161" x2="271" y2="161" stroke="var(--ink-3)" stroke-width="1.2"/>
  <line x1="262" y1="152" x2="262" y2="170" stroke="var(--ink-3)" stroke-width="1.2"/>
  <line x1="70" y1="26" x2="100" y2="26" stroke="var(--accent)" stroke-width="2.5"/>
  <text x="107" y="30" font-size="12" fill="var(--ink)">主轴 1 · 1.3242</text>
  <line x1="70" y1="46" x2="100" y2="46" stroke="var(--accent)" stroke-width="2.5" stroke-dasharray="8 5"/>
  <text x="107" y="50" font-size="12" fill="var(--ink)">主轴 2 · 2.9198</text>
  <line x1="70" y1="66" x2="100" y2="66" stroke="var(--accent)" stroke-width="2.5" stroke-dasharray="2 4"/>
  <text x="107" y="70" font-size="12" fill="var(--ink)">主轴 3 · 4.244</text>
  <g font-family="var(--font-mono)" font-size="12">
    <rect x="452" y="52" width="58" height="42" fill="var(--accent-bg)" stroke="var(--line)"/>
    <rect x="510" y="52" width="58" height="42" fill="none" stroke="var(--line)"/>
    <rect x="568" y="52" width="58" height="42" fill="none" stroke="var(--line)"/>
    <rect x="452" y="94" width="58" height="42" fill="none" stroke="var(--line)"/>
    <rect x="510" y="94" width="58" height="42" fill="var(--accent-bg)" stroke="var(--line)"/>
    <rect x="568" y="94" width="58" height="42" fill="none" stroke="var(--line)"/>
    <rect x="452" y="136" width="58" height="42" fill="none" stroke="var(--line)"/>
    <rect x="510" y="136" width="58" height="42" fill="none" stroke="var(--line)"/>
    <rect x="568" y="136" width="58" height="42" fill="var(--accent-bg)" stroke="var(--line)"/>
    <text x="481" y="78" text-anchor="middle" fill="var(--ink)">1.3242</text>
    <text x="539" y="78" text-anchor="middle" fill="var(--ink-3)">0</text>
    <text x="597" y="78" text-anchor="middle" fill="var(--ink-3)">0</text>
    <text x="481" y="120" text-anchor="middle" fill="var(--ink-3)">0</text>
    <text x="539" y="120" text-anchor="middle" fill="var(--ink)">2.9198</text>
    <text x="597" y="120" text-anchor="middle" fill="var(--ink-3)">0</text>
    <text x="481" y="162" text-anchor="middle" fill="var(--ink-3)">0</text>
    <text x="539" y="162" text-anchor="middle" fill="var(--ink-3)">0</text>
    <text x="597" y="162" text-anchor="middle" fill="var(--ink)">4.244</text>
  </g>
  <line x1="539" y1="186" x2="539" y2="222" stroke="var(--ink-2)" stroke-width="1.5" marker-end="url(#f4a)"/>
  <text x="539" y="244" text-anchor="middle" font-size="13" fill="var(--ink)">主轴朝向另记一个四元数（3 个自由度）</text>
</svg>
```

图注（F4）：左半是斜视的三质点刚体。x 轴朝右下、y 轴朝上、z 轴朝右上，灰色细线是坐标轴。三个深色点是质量点 P1、P2、P3，十字标记是质心。

三根主色线从质心出发，是三根主轴；主轴是无向直线，正向反向等价。方向按主轴矩阵的列投影到该视角：三维投到屏幕二维，只保留方向，不保留长度。右半是主轴系里的 3x3 矩阵：带底色三格是主惯量，无底色 0 是归零后的副对角元。

## F 为什么最后只存三个数加一个朝向

因为不丢信息，而且算起来最省事。先数信息量：三个主惯量 3 个数，朝向 3 个自由度，合计 6；对称矩阵恰好 6 个独立元素，一一对应。

再看计算：主轴系里转动方程没有交叉项，$\tau_x = I_1\alpha_x$，三个分量各算各的，数值积分最干净。物理引擎因此直接存对角化后的结果，而不是 9 个格子。

朝向一般用四元数存，不用三个欧拉角。欧拉角在某些姿态下两个转轴会重合，丢一个自由度，俗称万向节死锁。就像万向节两环转到同一平面，卡死一个方向。细节见复习区的四元数概念页。

前提两条：刚体（形状不变），质量分布不随时间变。软体、可变形体不适用这套存法。

## G 四个常踩的坑

```html
<div style="font-family:var(--font-sans);max-width:640px">
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">坑 1：看到非对角元非零，以为数据错了</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">坐标轴没对准主轴时，副对角元必然非零，换一组轴数值就变。先换到主轴系再下结论。</div>
</details>
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">坑 2：忘了惯量相对某个点算</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">同一个刚体，相对质心和相对边角算出的矩阵不同。只有相对质心那份，才和质心平动互不干扰。</div>
</details>
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">坑 3：单位搞错</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">惯量单位是 kg·m²，距离进公式前先平方。m 与 cm 混用一档，结果差一万倍。</div>
</details>
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">坑 4：没确认排序与四元数约定</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">主惯量有升序、降序两种排序约定，各家不同。朝向四元数里 q 与 -q 表示同一个姿态（双覆盖）。读别人数据前，先确认这两条约定。</div>
</details>
</div>
```

## H 自测与复习

```html
<div style="font-family:var(--font-sans);max-width:640px">
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">1. 两个 2 kg 质量块在 y = ±1 m，中间一根不计质量的杆。它在质心处的惯量矩阵是哪个？A) [[4,0,0],[0,0,0],[0,0,4]]　B) [[0,0,0],[0,4,0],[0,0,0]]　C) [[4,2,0],[2,0,0],[0,0,4]]　D) [[2,0,0],[0,0,0],[0,0,2]]</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">答案 A。绕 y 轴转时质量块在轴上，<i>I</i><sub>yy</sub> = 0；绕 x 轴、绕 z 轴转时各离轴 1 m，每块贡献 2 × 1² = 2，两块共 4。质量块全在 y 轴上，任意两个坐标的乘积都是 0，所以三个副对角元全为零。</div>
</details>
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">2. 歪哑铃的坐标系绕 z 轴从 0° 转到 90°，矩阵里哪些元素会变？A) 九个全变　B) 除 I<sub>zz</sub> 外都变　C) 只有对角元变　D) 都不变</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">答案 B。绕 z 旋转只重排 x 和 y 两个方向，z 轴不动，质量块到 z 轴的距离不变，<i>I</i><sub>zz</sub> 不变；其余五个独立元素都变。可拖 D 面板滑块到 90° 对照。</div>
</details>
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">3. E 面板 F4 图右半的矩阵里，六个无底色的 0 格对应刚体的什么性质？A) 主惯量为零　B) 质心在原点　C) 主轴系里没有串轴　D) 刚体不受重力</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">答案 C。那六个 0 是归零后的副对角元：主轴系下绕一根主轴转，不会带出别的方向的角动量。这正是转到主轴系的意义。</div>
</details>
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">4. 为什么「三个对角值加一个朝向四元数」和直接存 3x3 矩阵表示的是同一份信息？A) 四元数比矩阵精确　B) 两种表示都是 6 个独立数，只是打包方式不同　C) 矩阵有 3 个元素恒为零　D) 主轴取法唯一</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">答案 B。对称矩阵有 6 个独立元素；三个主惯量加朝向的 3 个自由度也是 6 个，一一对应。与精度无关，一般坐标系下副对角元也不恒为零。</div>
</details>
<details style="border:1px solid var(--line);border-radius:6px;padding:10px 14px;margin:8px 0">
<summary style="cursor:pointer;color:var(--ink)">5. 某刚体三个主惯量都乘 2，质量不变，同样的力矩和同样的力之下会怎样？A) 角加速度减半，平动加速度不变　B) 角加速度减半，平动加速度也减半　C) 角加速度不变　D) 下落变慢</summary>
<div style="color:var(--ink-2);font-size:14px;margin-top:8px">答案 A。平动加速度等于力除以质量，质量没变所以平动不变；转动按 τ = <i>I</i>α，惯量翻倍角加速度减半。下落是平动，不受影响。</div>
</details>
</div>
```

### 复习与拓展

- [David Tong 动力学讲义 §3.2（英文）](https://www.damtp.cam.ac.uk/user/tong/dynamics/three.pdf)
- [知乎：一般性刚体惯量主轴的选取](https://zhuanlan.zhihu.com/p/137394465)
- [四元数概念页](../quaternion.html)：本页对朝向四元数一句带过，细节看这里
