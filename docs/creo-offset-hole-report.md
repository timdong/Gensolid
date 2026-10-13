# 利用 Creo 黑盒对照增强 Gensolid：偏置通孔

## 已完成的增强

Gensolid 已从仅支持中心通孔，扩展为支持**指定局部 XY 位置的圆柱通孔**。本例为 40×25×8 mm 板件、Ø8 mm 通孔，孔中心位于 XY=(6,-3) mm，底面 Z=0。

孔的位置随零件坐标系一起平移和旋转。构造时检查四侧剩余壁厚，拒绝孔接触或越过外壁、壁厚不足及非有限数输入。壁厚判断使用区间算术，阈值来自会话 `Precision`。

新增 Rust API：

```rust
Session::body_create_solid_block_with_offset_hole(x, y, z, radius, offset, basis)
```

其中 `offset` 为局部坐标 `[孔中心X, 孔中心Y]`，`radius` 为半径。原有中心孔接口复用同一实现，偏置取 `[0,0]`。

在项目根目录运行：

```sh
cargo run -p gensolid-api --example creo_compare -- 6 -3
cargo test
```

当前环境的工具链替代运行方法见[中心孔报告](/mnt/d/CAD/liuxin/Gensolid/docs/creo-comparison-demo.md)。

代码与测试：

- [解析 B-rep 构造](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-topo/src/holed_block.rs)
- [Rust API](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-api/src/lib.rs)
- [可指定孔位置的 Demo](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-api/examples/creo_compare.rs)
- [通孔集成与属性测试](/mnt/d/CAD/liuxin/Gensolid/crates/gensolid-api/tests/holed_block.rs)

## Creo 实际实验

2026-10-13，使用外部 build123d 生成偏置通孔 STEP，再通过 J-Link 启动 Creo 10.0.8.0，完成导入、再生、保存和 STEP 导出。运行日志确认 `SAVED_PART`、`EXPORTED_STEP`；导出文件经独立几何测量和有效性检查。

参考文件由外部程序生成，Gensolid 的模型由自身 Rust 构造函数生成。当前 Gensolid 尚无 STEP 导出，二者通过各自检查结果进行对照。

| 检查项 | Gensolid | 外部输入 STEP | Creo 导出 STEP |
| --- | --- | --- | --- |
| 包围盒（mm） | (-20,-12.5,0) 至 (20,12.5,8) | 相同 | 在 Gensolid 默认线性精度内相同 |
| 孔轴 XY 位置（mm） | (6,-3) | (6,-3) | (5.99999999999886,-3.0000000000007) |
| 孔半径（mm） | 4 | 4 | 3.9999999999992397 |
| 孔轴方向 | +Z | +Z | +Z |
| 面数 / 边数 | 7 / 14 | 7 / 15 | 8 / 18 |
| 圆柱面数 | 1 | 1 | 2 |
| 有效性 | `check_body` 零故障，无未执行的几何检查 | OCP 有效，1 个实体、1 个壳 | OCP 有效，1 个实体、1 个壳 |
| 体积（mm³） | 7597.876140，仅为参考公式值 | 7597.8761403404715 | 7597.876140341635 |

圆柱轴线的位置点可沿轴线移动，因此轴线定义点的 Z 坐标不等同于孔底面高度。Gensolid 的孔口位于 Z=0、8 mm；外部模型通过上下平面和孔壁共同限定同一贯穿范围。

Creo 导出件的圆柱孔壁被分成两个面；Gensolid 使用一个周期圆柱面和两条完整圆边。拓扑数量不同，不能仅凭数量判断形状是否一致。

本次 Creo 质量属性接口仍返回 `XToolkitGeneralError`。表中 STEP 的体积由 build123d/OCP 独立计算；Gensolid 的体积输出只是 `(40×25−π×4²)×8` 参考公式，尚非内核质量属性计算结果。

## 验证结果

- 项目默认成员共 **63 项测试全部通过**，通孔集成测试共 10 项。
- 新增测试覆盖偏置、旋转坐标系、随机尺寸与孔位置、接近退化的壁厚/孔半径/板厚、四侧接触与越界，以及拒绝请求后继续建模。
- 原有中心孔检查保留，验证了中心孔接口和拓扑构造入口的兼容性。
- 全项目格式检查通过；修改的 `gensolid-api`、`gensolid-topo` 通过严格 Clippy：`--all-targets --no-deps -- -D warnings`。
- 全项目严格 Clippy 的既有 NURBS 循环警告见中心孔报告；本轮未修改该算法。
- 已执行输入和 Creo 导出 STEP 的 `inspect refs`、`inspect validate`，并以 OCP 独立确认有效实体、包围盒、圆柱轴线、半径和体积。
- 已查看参考试件快照，以及 Creo 导出件的等轴和俯视快照。俯视图显示孔偏向 +X、-Y 方向，孔口可贯穿看到背景；测量结果支持孔的位置与尺寸。

原始证据：

- [外部试件生成源码](/mnt/e/code/Rea-Test/creo_probe/offset_hole/o.step.py)
- [外部输入 STEP](/mnt/e/code/Rea-Test/creo_probe/offset_hole/o.step)
- [Creo 原生保存件](/mnt/e/code/Rea-Test/creo_probe/offset_hole/probe_plate.prt.1)
- [Creo 导出 STEP](/mnt/e/code/Rea-Test/creo_probe/offset_hole/probe_plate_creo.stp)
- [Creo 运行日志](/mnt/e/code/Rea-Test/creo_probe/offset_hole/creo-probe.log)
- [Gensolid Demo 输出](/mnt/e/code/Rea-Test/creo_probe/offset_hole/gensolid-demo.log)
- [独立测量数据及文件 SHA-256](/mnt/e/code/Rea-Test/creo_probe/offset_hole/measurements.json)

## 三维预览与快照

复用已运行的 CAD Viewer，文件存在性已核实：

- [外部参考试件三维预览](http://127.0.0.1:3245/mnt/e/code/Rea-Test/creo_probe?file=offset_hole/o.step.py)
- [Creo 导出件三维预览](http://127.0.0.1:3245/mnt/e/code/Rea-Test/creo_probe?file=offset_hole/probe_plate_creo.stp)

这些预览展示外部参考模型和 Creo 导出件，Gensolid 当前通过控制台和解析 B-rep 检查呈现结果。

![外部参考试件](/mnt/e/code/Rea-Test/creo_probe/offset_hole/offset-reference_20261013T085313Z.png)

![Creo 导出件等轴图](/mnt/e/code/Rea-Test/creo_probe/offset_hole/creo-offset-iso_20261013T085451Z.png)

![Creo 导出件俯视图](/mnt/e/code/Rea-Test/creo_probe/offset_hole/creo-offset-top_20261013T085451Z.png)

## 逆向工作的实际范围

本轮采用黑盒几何对照：运行 Creo、观察导入/再生/导出结果，并用这些结果验证 Gensolid 的独立实现。未使用反汇编恢复 Creo 核心源码，也未创建或追踪 Creo 原生草绘、拉伸、孔特征的内部算法。该实验不能说明 Creo 原生孔特征对临界壁厚的接受规则。

新增功能仍为单个、与局部 Z 平行的圆柱通孔专用原语，尚无通用布尔运算、盲孔、多孔、特征树，以及该操作的 C/Python/WASM 封装。后续可按同样方法逐项补强，并为每项功能保留参考试件、有效性检查和回归测试。
