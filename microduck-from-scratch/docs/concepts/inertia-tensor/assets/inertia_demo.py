import numpy as np

# 三个质点（质量 m，位置 p），拼一个歪的哑铃形
pts = [(2.0, np.array([1.0, 0.3, 0.0])),
       (2.0, np.array([-0.6, 0.8, 0.2])),
       (1.0, np.array([0.2, -0.5, 0.9]))]
M = sum(m for m, _ in pts)
com = sum(m * p for m, p in pts) / M

# 质心系下累加 3x3 惯量张量
I = np.zeros((3, 3))
for m, p in pts:
    r = p - com
    I += m * ((r @ r) * np.eye(3) - np.outer(r, r))

np.set_printoptions(precision=4, suppress=True)
vals, vecs = np.linalg.eigh(I)   # 特征值 = 主惯量，特征向量 = 主轴
print("惯量张量 I =\n", I)
print("主惯量（对角 3 个数）:", vals)
print("主轴矩阵（每列一根轴）:\n", vecs)
print("转进主轴系 R.T @ I @ R =\n", vecs.T @ I @ vecs)
