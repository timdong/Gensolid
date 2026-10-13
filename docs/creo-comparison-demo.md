# Gensolid 与 Creo 通孔板件对照报告

后续已新增偏置通孔功能，最新能力与 63 项测试结果见[偏置通孔增强报告](/mnt/d/CAD/liuxin/Gensolid/docs/creo-offset-hole-report.md)。本报告保留中心孔初版的实验记录。

Gensolid 已使用自身的解析边界表示（B-rep），构造出 **40 × 25 × 8 mm、中心带 Ø8 mm 通孔的板件**。模型在 X、Y 方向居中，底面位于 Z=0，顶面位于 Z=8 mm。

Rust 接口中的 `radius` 参数表示孔半径，本例取 **4 mm**。

## 运行方法

在 Gensolid 项目根目录执行：

```sh
cargo run -p gensolid-api --example creo_compare
cargo test -p gensolid-api
```

- [Demo 源码](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-api/examples/creo_compare.rs)
- [通孔构造实现](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-topo/src/holed_block.rs)
- [Rust API](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-api/src/lib.rs)
- [集成与属性测试](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-api/tests/holed_block.rs)

新增接口为：

```rust
Session::body_create_solid_block_with_hole(x, y, z, radius, basis)
```

## 实现方式

构造过程先创建长方体的六个平面，在底面和顶面分别加入方向相反的内圆环，再加入一个法向朝向孔轴的圆柱孔壁。两条不带顶点的完整圆边，将孔壁与上下平面连接起来。圆边和圆柱面均保留为解析几何。

孔半径及 X、Y 方向的剩余壁厚必须能被确定为大于会话线性精度。实现使用区间算术判断壁厚；无法确定的临界输入会被拒绝。长方体尺寸同时遵守现有长方体构造函数的限制，并支持任意有效坐标系。

## Creo 实验与对照结果

Creo 实验通过 J-Link 调用 `CreatePart`、`CreateImportFeat`、`Regenerate`、`Save` 和 `ExportIntf3D`，导入了一个**由外部程序生成的通孔板件**，再保存为 Creo 零件并导出 STEP。

此次实验没有创建 Creo 原生草绘、拉伸和孔特征。再生调用成功返回，但没有枚举失败特征，因此尚无经过验证的失败特征数量。此前 Gensolid 仅有长方体的基准 demo，还未包含 Creo 试件中的通孔；本次已补齐该几何特征。

现有参考文件：

- [外部生成的输入 STEP](/mnt/e/code/Rea-Test/creo_probe/probe_plate.step)
- [Creo 保存的导入零件](/mnt/e/code/Rea-Test/creo_probe/probe_plate.prt.1)
- [Creo 导出的 STEP](/mnt/e/code/Rea-Test/creo_probe/probe_plate_creo.stp)

2026-10-13 重新检查后的结果如下，单位统一为毫米：

| 检查项 | Gensolid B-rep | 外部输入 STEP | Creo 导出 STEP |
| --- | --- | --- | --- |
| 包围盒 | (-20, -12.5, 0) 至 (20, 12.5, 8) | 相同 | 在 Gensolid 线性精度范围内相同 |
| 孔半径 | 4 mm | 4 mm | 3.99999999999924 mm |
| 面数 / 边数 | 7 / 14 | 7 / 15 | 8 / 18 |
| 圆柱面数 | 1 | 1 | 2 |
| 有效性检查 | `check_body` 零故障，零无法执行的检查 | OCP 检查有效：一个实体、一个壳 | OCP 检查有效：一个实体、一个壳 |
| 体积（mm³） | 7597.876140340506，**仅为参考公式计算值** | 7597.876140340493 | 7597.876140341633 |

参考体积公式为：

```text
(40 × 25 − π × 4²) × 8
```

Gensolid 尚未实现内核质量属性计算。表中的 STEP 体积由 build123d/OCP 独立计算得到；此前 Creo 质量属性请求失败，不能将这些数值视为 Creo 接口返回的测量值。本次没有移植或改写 OCCT 的实现代码。

面数和边数的差异与周期曲面的接缝表示，以及 Creo 对圆柱孔壁的分割有关。拓扑数量本身不能证明几何等价。

## 验证内容与结果

Demo 检查了包围盒、圆柱孔壁的半径、轴向、位置、朝内方向、两条圆形孔口边，以及 `body_check` 的结果。

集成测试额外检查了圆环高度、孔壁法向、环边方向，以及旋转后外部角点的位置。三组 proptest 属性测试覆盖随机尺寸和坐标系、接近退化边界的有效几何，以及壁厚不足的无效输入。其他测试覆盖无效尺寸与半径、默认坐标系及同一会话中的多个实体。

- Demo 运行成功：7 个面、14 条边、8 个顶点，包围盒和孔形状检查通过，零检查故障。
- 项目默认成员的 `cargo test` 共 **58 项测试通过**，包括新增的 5 项通孔测试和全部 4 项仓库规则测试。
- 全项目格式检查通过；仅格式化了本次涉及的 Rust 文件。受保护的检查器、容差模块和验证器源码未被修改。
- 本次修改的 `gensolid-api` 和 `gensolid-topo` 两个 crate，连同示例和测试，通过严格 Clippy 检查：`--all-targets --no-deps -- -D warnings`。
- 全项目严格 Clippy 检查仍被既有 NURBS 实现的两处 `needless_range_loop` 警告阻挡，位置为 [nurbs.rs](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-geom/src/nurbs.rs:66) 的第 66、75 行。本次未修改该算法。
- 已执行 Creo STEP 检查及独立 OCP 有效性验证。本次没有修改 STEP 几何，因此未重新生成几何快照。

## 当前实现范围

本次交付的是通过 Rust API 使用的**带中心通孔的长方体专用原语**。尚未实现通用布尔减运算、特征树、STEP 读写，以及该新操作的 C、Python、WASM 接口封装，也未恢复 Creo 源代码。

当前对照验证了这个试件的尺寸及解析几何特征，不能据此推断两个 CAD 内核在一般情况下等价，也不能据此估计 Creo 核心代码的逆向成功比例。

## 本会话的工具链说明

环境中的 `stable` rustup 工具链不完整。本次编译和测试显式使用现有 Rust 1.88 编译器及 rustdoc，配合已安装的 Cargo 二进制程序。在当前 Linux/WSL 环境中可使用以下命令运行 demo：

```sh
RUSTC=/root/.rustup/toolchains/1.88.0-x86_64-unknown-linux-gnu/bin/rustc \
RUSTDOC=/root/.rustup/toolchains/1.88.0-x86_64-unknown-linux-gnu/bin/rustdoc \
/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo \
run -p gensolid-api --example creo_compare
```

与 Rust 1.88 匹配的格式化和 Clippy 工具单独安装于 `/tmp/gensolid-tools-1.88`。使用这些临时工具时，需将 Rust 1.88 工具链的 `lib` 目录加入 `LD_LIBRARY_PATH`。本次没有替换编译器或修改仓库的工具链声明。
