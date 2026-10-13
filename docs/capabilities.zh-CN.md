# Gensolid 当前能力与错误契约

核验日期：2026-10-13。对应阶段 A、GS-004，以及 GS-011 的既有通孔封装部分。

## 已可实际调用

| 能力 | Rust | C | Python | WASM | 支持范围 |
|---|---|---|---|---|---|
| 长方体实体 | 支持 | 支持 | 支持 | 支持 | 三个局部尺寸、中心位置与正交坐标系 |
| 居中圆通孔长方体 | 支持 | 支持 | 支持 | 支持 | 单一圆孔，沿局部 Z 贯穿 |
| 偏心圆通孔长方体 | 支持 | 版本化 options.offset | 支持 | 支持 | 孔心为局部 XY 偏移，四侧壁必须保留 |
| 面/边/顶点查询 | 支持 | 支持 | 支持 | 支持 | 会话内整数 tag；不存在和错误类型返回错误 |
| 实体检查 | 支持 | 支持 | 支持 | 支持 | 既有检查器；内环绕向和内壁法向另有独立 Rust 测试 |
| 顶点坐标 | 支持 | 支持 | 支持 | 支持 | 世界坐标，遵循输入 basis |

通孔是独立解析基础体：平面外壁、两个带圆形内环的端面、向孔内的圆柱壁，共 7 面、14 边、8 顶点。完整圆边采用无顶点表示。计数属于当前 Gensolid 表示，不要求与 Creo 的分面、接缝边计数一致。

几何层另有解析曲线/曲面、NURBS 求值和导数、投影及距离计算；扫掠面与旋转面求值不能等同于扫掠实体或旋转实体建模。XT 当前只有头部和 token 层。

## 通孔输入约定

- x/y/z 是块的局部尺寸，块中心为 basis.location；默认位置是世界原点。
- radius 是半径，offset 是局部 XY 孔心，孔沿局部 Z 贯穿；孔径等于两倍 radius。
- 调用方统一模型单位。40×25×8、radius=4、offset=[6,-3]、location=[0,0,4] 对应既有毫米试件。
- 输入必须有限；尺寸、半径和四侧剩余壁厚须满足会话 Precision。越界、相切以及不能由区间算术确认充分壁厚的输入保守拒绝。
- Python 与 WASM 支持给定坐标系；WASM 的 At 方法要求三个长度为 3 的数组，偏移数组长度为 2。
- C basis/options 可为空，分别表示世界坐标系和居中孔；options 必须由初始化函数设置版本。必需输出指针为空时，在建模前返回错误。

示例：

```python
import gensolid
s = gensolid.Session()
body = s.body_create_solid_block_with_offset_hole(
    40, 25, 8, 4, [6, -3], location=[0, 0, 4]
)
assert s.body_check(body) == []
```

```c
GS_BODY_create_solid_block_with_hole_o_t options;
GS_BODY_create_solid_block_with_hole_o_init(&options);
options.offset[0] = 6;
options.offset[1] = -3;
/* 先启动会话；生产调用必须检查每个返回码。 */
GS_BODY_t body;
GS_BODY_create_solid_block_with_hole(40, 25, 8, 4, NULL, &options, &body);
```

## 错误分类

| Rust ErrorCode | C | Python | WASM |
|---|---|---|---|
| InvalidArgument / Degenerate / OutOfRange | 对应既有数值 1 / 4 / 5 | ValueError | 抛 Error，保留内核消息 |
| NotFound | 2 | KeyError | 抛 Error |
| WrongEntityClass | 3 | TypeError | 抛 Error |
| Unsupported | 6 | RuntimeError | 抛 Error |
| Unimplemented | 7 | NotImplementedError | 抛 Error |
| CapacityExceeded / Parse / Internal | 8 / 9 / 10 | RuntimeError | 抛 Error |
| C 会话、空指针、版本错误 | 100 / 101 / 102 / 103 | 不适用 | 不适用 |

WASM 目前不提供结构化错误码，这是当前契约的限制。Python 数组形状错误由 PyO3 转为类型/值异常。无效孔参数和坐标系输入经测试不会消耗 body tag；尚未提供覆盖所有容量或内部失败的编辑事务，这属于 GS-008。

## 后续功能

任意实体打孔、圆柱实体、多孔、盲孔、拉伸/旋转实体、通用布尔、圆角、壳体、真实质量属性、STEP 模型交换、XT 实体交换、特征重建和装配均未完成。直角传动装配 RA-001 仍按开发计划的 L1/L2/L3 分层验收。

当前跨语言接口是自主封装和解析 B-rep；Creo 用于独立进程中的行为与交换对照。没有恢复 Creo 原始源码，也没有把外部参考 STEP 生成器当成 Gensolid 的内核功能。
