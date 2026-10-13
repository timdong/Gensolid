# ADR 0008：解析通孔基础体的跨语言契约

状态：已采纳（2026-10-13）。关联任务：GS-011。

## 背景

Rust API 已提供居中和偏心通孔长方体，C/Python/WASM 尚未暴露。该操作直接构造解析 B-rep，不能宣称已实现任意实体打孔或通用布尔差。

## 决定

- 各绑定调用现有 `gensolid-api`，尺寸 x/y/z、孔半径 radius 和坐标系的验证只由语义层完成。radius 是半径，模型单位由调用方统一。
- 长方体以 basis.location 为中心，孔沿局部 z 贯穿，offset 是局部 x/y 偏移。孔必须完全位于四侧面内且满足当前 Precision；交界、越界、非有限参数和退化几何返回既有错误。
- C 增加 `GS_BODY_create_solid_block_with_hole` 和版本化 `GS_BODY_create_solid_block_with_hole_o_t`。选项只包含版本和二维 offset，初始化为居中；空 options 等价于默认选项，空 basis 等价于世界坐标系。不改变已有 ABI 或错误码。
- C 建模入口先验证必需输出指针和选项版本，再创建实体。失败不写 body，不因空输出指针分配实体；已有长方体入口同时修复这一问题。
- Python 提供居中/偏心两种方法，沿用已有 location/axis/ref_direction 默认值和异常映射。
- WASM 提供居中/偏心方法及对应 At 方法，At 使用三个长度为 3 的数组，偏移数组长度必须为 2。参数数量由公开跨语言契约决定，对超过 Clippy 建议参数数量的方法局部说明，不关闭工作空间检查。
- B-rep 内容由既有 Rust 几何和随机测试验证；各绑定另验真实调用、错误传播、坐标变换和 C 数组释放。

## 限制与来源

只支持单一直圆通孔长方体；盲孔、多孔和任意实体孔属于后续任务。实现为自主 Rust 封装，未移植 Creo、Parasolid 或 OCCT 代码。Creo 输入输出观察仅作为行为对照证据。
