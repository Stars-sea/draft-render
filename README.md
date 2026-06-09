# draft-render — 论文实验分支 (essay)

> 基于蒙特卡洛路径追踪的渲染方差分析实验平台  
> 概率论与数理统计课程结课论文

## 快速开始

```bash
# 默认渲染 Cornell Box（交互式窗口）
cargo run --release

# Headless 渲染 + 保存文件 (.bin + .png + _depth.csv)
cargo run --release -- --spp 256 --output cornell_e6_spp256

# 生成 Ground Truth（Jitter + 8192 SPP + 无 RR + 无 Clamp）
cargo run --release -- --reference --output cornell_gt

# N=3 独立重复（不同 seed）
cargo run --release -- --spp 64 --seed 0 --output e5_spp64_rep0
cargo run --release -- --spp 64 --seed 1 --output e5_spp64_rep1
cargo run --release -- --spp 64 --seed 2 --output e5_spp64_rep2

# 加载 PMX 模型
cargo run --release -- --pmx models/nahida/纳西妲.pmx --spp 64 --output nahida
```

## CLI 参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `--spp <N>` | u32 | 16 | 每像素采样数 |
| `--sampler <halton\|jitter>` | enum | halton | 采样策略 |
| `--rr-depth <N>` | u32 | 3 | RR 起始深度（0 = 关闭 RR） |
| `--clamp <value>` | f32 | 10.0 | 间接钳制阈值（≤0 = 关闭 Clamp） |
| `--max-depth <N>` | u32 | 8 | 最大路径深度 |
| `--output <prefix>` | string | — | 输出文件前缀（触发 headless 模式） |
| `--reference` | flag | — | 快捷：spp=8192 + **Jitter** + rr=0 + clamp=0 |
| `--width <W>` | usize | 800 | 图像宽度 |
| `--height <H>` | usize | 600 | 图像高度 |
| `--pmx <path>` | string | — | PMX 模型路径（替代 Cornell Box） |
| `--seed <N>` | u64 | 0 | 随机种子（混入像素 seed，不同值产生不同噪声实现） |

## 输出格式

指定 `--output <prefix>` 后，同时生成三个文件：

### `<prefix>.bin` — HDR 浮点数据

```
Offset  Size    Description
0       4       Width  (u32 LE)
4       4       Height (u32 LE)
8       W*H*12  Pixel data (f32 LE, R G B interleaved, row-major)
```

Python 读取：
```python
import numpy as np
with open('output.bin', 'rb') as f:
    w = np.frombuffer(f.read(4), dtype=np.uint32)[0]
    h = np.frombuffer(f.read(4), dtype=np.uint32)[0]
    rgb = np.frombuffer(f.read(), dtype=np.float32).reshape(h, w, 3)
```

### `<prefix>.png` — Tone-mapped 预览

ACES 色调映射 + sRGB gamma，便于直接查看和论文插图。

### `<prefix>_depth.csv` — 路径深度直方图

```csv
depth,count
0,9029
1,5446
...
8,2110
```

用于 §4.3.3 "RR 终止深度的几何分布建模"。每行记录该深度上终止的路径总数。

## 实验矩阵快速参考

| ID | 命令 |
|----|------|
| E1 (Jitter, 无RR, 无Clamp) | `--sampler jitter --rr-depth 0 --clamp 0` |
| E2 (Jitter, RR) | `--sampler jitter --rr-depth 3 --clamp 0` |
| E3 (Jitter, RR+Clamp) | `--sampler jitter --rr-depth 3 --clamp 10.0` |
| E4 (Halton, 无RR, 无Clamp) | `--sampler halton --rr-depth 0 --clamp 0` |
| E5 (Halton, RR) | `--sampler halton --rr-depth 3 --clamp 0` |
| E6 (Halton, RR+Clamp) | `--sampler halton --rr-depth 3 --clamp 10.0` |
| Ground Truth | `--reference` |

每个配置 × 每个 SPP 级别 × 3 个 seed 值（如 `--seed 0,1,2`）。

## Cornell Box 场景

默认场景为标准 Cornell Box，几何数据来自 [bowers.cornell.edu](https://bowers.cornell.edu/computer-graphics/data) 官方实测：

- **几何**：`models/cornell_box/cornell_box.obj`（28 顶点, 36 三角面, 18 面）
- **材质**：Mitsuba 预积分 RGB，配置在 `main.rs` 中
- **光源**：天花板面板中心点光源（钨丝色温近似）
- **摄像机**：官方位置 (278, 273, -800) mm，FOV 39.3°

材质反射率（官方光谱 → CIE XYZ → sRGB）：

| 表面 | sRGB | 线性 RGB |
|------|------|---------|
| 白墙 | `(221, 219, 215)` | `(0.725, 0.710, 0.680)` |
| 红墙 | `(208, 72, 63)` | `(0.630, 0.065, 0.050)` |
| 绿墙 | `(105, 179, 85)` | `(0.140, 0.450, 0.091)` |

## 场景数据文件

| 文件 | 说明 |
|------|------|
| `models/cornell_box/scene.json` | 完整场景描述（光谱、几何、光源、摄像机） |
| `models/cornell_box/cornell_box.glb` | 三角化 glTF 几何（二进制） |
| `models/cornell_box/convert_cornell.py` | 官方数据 → glTF/JSON 转换脚本 |

## 论文工作区

- 论文大纲与文档：`../../概率论论文/`
- 本分支：`essay`（从 `main` 分支）
