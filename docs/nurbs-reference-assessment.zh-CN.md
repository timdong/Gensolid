# NURBS 参考项目评估：USD BRep / SMLib

评估日期：2026-10-14。结论：**有值得参考的实际内核代码，建议正式加入 Gensolid 的开发参考。近期最有价值的是裁剪拓扑和 NURBS 编辑，中期是求交求解器与布尔流程。** 本次完成静态源码核对和计划补充，尚未构建或运行该项目，也未将其代码移入 Gensolid。

## 1. 项目身份与核验范围

| 项目 | 本地核验结果 |
|---|---|
| 用户目录 | `D:\CAD\liuxin\NURBS` |
| 实际仓库 | `D:\CAD\liuxin\NURBS\usd-brep` |
| origin | `git@github.com:NVIDIA-Omniverse/usd-brep.git` |
| 本地 HEAD | `c9c5979f04cea8c38213004d4f04df0dcfb40d05`，Release 1.0.0 |
| 核心代码 | `source/SMLib`：462 个 `.cpp/.h` 文件，约 96.7 万物理行，包含注释和禁用代码，不等于有效代码量 |
| 许可核对 | 根 LICENSE 为 Apache-2.0；上述 462 个文件头均含 Apache-2.0 SPDX 标识；具体复用仍需核对文件来源与依赖 |
| 其他层 | SM_API、Python 绑定、USD 数据/转换、staging、显示和工具 |
| 本次验证 | 静态阅读、符号定位、文件头核验、所引用文件与 HEAD 的换行归一化比较和哈希；没有运行算法 |

这是带有 SMLib 实现的 USD BRep 项目。已查看的插结和拆分函数有控制点、节点向量、重数、分配和输出处理，不只是接口声明。不能将目录名“NURBS”理解为仅有曲线求值库，也不能将源码存在视为工业正确性已验证。

本地工作树显示大量文件变动。为避免仅凭提交号误认源码版本，本次将引用文件逐一与 HEAD 比较，比较结果和原始字节 SHA-256 保存于[证据清单](evidence/2026-10-14/nurbs-reference/manifest.json)。未修改参考仓库，也未执行其下载/构建脚本。

## 2. 对 Gensolid 最有价值的代码

下列路径均相对于参考仓库；行号对应本次快照。优先级表示推荐采用顺序，尚未完成移植或运行验收。

| 优先级 / 对应任务 | 具体文件与入口 | 可参考内容 | Gensolid 应如何采用 |
|---|---|---|---|
| P0 / GS-009 | `source/SMLib/inc/SmEdge.h:92` 的 `m_vInterval`，`inc/SmEdgeuse.h:91` 的 UV trim，`src/SmEdgeuse.cpp:4259` 的 `GetOrCreateUVTrimCurve` | 边的实际区间与承载曲线分离；面内曲线的使用和缓存 | 依 ADR 0009 建立边区间及 fin 附属表；明确缺失、方向、映射和失效规则 |
| P0 / GS-009 | `src/SmPeriodicExtent1d.cpp:1320`、`:1353` 的 `NormalizeValue/Normalize` | 周期区间、整周期与接缝点的区分 | 参考问题分类，保留 Gensolid 展开参数；建立跨周期、整圆和零长度反例 |
| P0 / GS-009、阶段 I | `src/NL_CrvTool.cpp:65`、`:253`、`:811`、`:3276` 的插结、拆分、Bezier 分解和升阶 | 基于齐次控制点的 NURBS 编辑流程；节点重数与输出结构 | 优先候选为纯几何插结/拆分；随后处理拓扑拆边，二者不能等同 |
| P1 / 阶段 I | `src/SmBSplineSurface.cpp:5016` 的 `SplitAt`；`src/SmBSplineSurfaceNLib.cpp:6435` 的 `InsertOneKnot` | 曲面沿参数方向拆分及插结 | 在曲线算法稳定后扩展；不直接引入整套对象/内存体系 |
| P1 / GS-010、阶段 D | `src/SmPrimitiveCreation.cpp`、`source/SM_API/src/SmApiPrimitives.cpp:383` 的圆柱创建 | 基础实体的面、环、边组织及解析曲面配置 | 帮助设计圆柱、多孔、盲孔的拓扑；结果仍由 Gensolid 自身构造和检查 |
| P1 / GS-016、投影 | `src/SmGlobalSolver.cpp:130`、`:272`、`:1131` 的 SolveIt/SolveBranch/Subdivide；`src/SmLocalSolveNd.cpp:90` | 全局搜索/细分与局部 Newton 求解分层；参数边界、坏步长和失败处理 | 用于设计分支管理与诊断；不能把 Newton 失败当无交，不能据此宣称解集完整 |
| P1 / GS-016 | `src/SmSurfaceDropCurve.cpp:78`、`SmSurfaceIntersector.cpp`、`SmAdvSurfaceIntersector.cpp` | 曲线投到曲面、残差/导数、曲面交线组织 | 先解析支持域，再扩展数值曲线；周期面和极点单列验收 |
| P2 / GS-017、阶段 G | `src/SmMerge.cpp:2510`、`src/SmTopologyIntersector.cpp:1092` 的 `IntersectInsertRelate`；`SM_API/src/SmApiBrep.cpp:208` | 求交、插入拓扑、建立关系和布尔选择的分层 | 可参考流程与模块边界；整体指针式移植成本高，先做平面布尔范围 |
| P1 / GS-012 | `src/SmIntegrator.cpp:78`、`SmGauss.cpp`、`SmFaceProps.cpp`、`SmObjProps.cpp` | 面积/体积积分的职责拆分和精度参数 | 参考接口及反例；积分核心优先独立实现，见第 4 节 |
| P1 / GS-015 | `src/SmTess.cpp:3009`、`:3074` 的边采样与面显示缓存 | B-rep 到显示网格、共享边与面采样组织 | 限于显示/次级导出；精确布尔不能反向使用网格 |
| P2 / 交换层设计 | `source/BREP_STAGING/README.md`、`inc/BrepSourceData.h`、`source/USD_BREP_UTILS/src/UsdBrepKnots.cpp` | 与来源解耦的暂存结构、节点展开/压缩和导入阶段检查 | 参考暂存与检查设计；USD 支持不替代 GS-013 的 STEP 和 GS-014 的 XT |

可直接从这些文件开始：[NURBS 编辑](../../NURBS/usd-brep/source/SMLib/src/NL_CrvTool.cpp)、[edgeuse](../../NURBS/usd-brep/source/SMLib/inc/SmEdgeuse.h)、[周期区间](../../NURBS/usd-brep/source/SMLib/src/SmPeriodicExtent1d.cpp)、[全局求解](../../NURBS/usd-brep/source/SMLib/src/SmGlobalSolver.cpp)、[布尔流程](../../NURBS/usd-brep/source/SMLib/src/SmMerge.cpp)。

## 3. 与当前实现的差异

Gensolid 的 `gensolid-geom/src/nurbs.rs` 已有 BCurve/BSurface 求值与导数，依据《The NURBS Book》；首要缺口是编辑、裁剪及其与拓扑的关系。替换现有求值器收益有限，优先补充插结、拆分和参数映射更合适。

方向必须按层核对。SMLib runtime 的 SmEdgeuse 以 mate 对存放 UV 曲线，头文件要求其与对应三维边曲线同向；BREP_STAGING 文档要求按 edgeuse 遍历方向存储。它们不是同一种存储契约，不能把其中一种直接等同于 Gensolid 的 fin。Gensolid 仍使用显式 Sense 和参数映射，接缝的两次使用保留各自周期分支。

其他差异包括：SMLib 的指针图、SmContext、可变缓存、原位/消耗式操作与 Gensolid 的 arena ID、事务和默认保留输入的规划不同；部分解析角参数采用度，Gensolid 几何求值采用弧度。移植必须先确定单位、参数域、所有权和错误语义，不能只翻译语法。

## 4. 已发现的不宜照搬之处

以下前四项来自本次代码阅读，描述的是可见分支或注释，尚未做运行复现。后面的运行误差数据是上游 KNOWN_ISSUES 的陈述，不是本次测量。

| 位置 | 静态观察 | 对 Gensolid 的要求 |
|---|---|---|
| `NL_CrvTool.cpp:253` 的 N_CrvSplit | 按参数域宽度乘固定 `1.0E-6` 吸附节点 | 不能原样移植；参数精度须与 Precision、曲线尺度及明确误差策略关联 |
| `SmPeriodicExtent1d.cpp:1353` | 注释明确提示 Normalize 会把接缝点变成完整区间；NormalizeValue 只加/减一次周期 | 接缝点、整周期和跨多个周期分别表示和测试，不用于无条件归一化展开端点 |
| `SmEdgeuse.cpp:4259` | UV 为空且禁止创建时允许返回 SM_SUCCESS；最大偏差说明为采样值，可能小于真实最大值 | 缺失映射返回显式诊断，采样最大值不能当成严格误差上界 |
| `SmIntegrator.cpp:78` | ODE 积分返回失败的分支将结果设为零，函数末尾仍返回 SM_SUCCESS；算法选择参数标为未使用 | 积分失败必须可见，不能发布零值质量属性；算法选项须兑现或拒绝 |
| `SmIntegrator.cpp:7` 及后续注释 | 注明部分例程取自 Numerical Recipes in C | 文件头 Apache 标识不足以消除该来源疑点；该文件不列为直接移植候选，按公开数学独立实现 |
| `SmTopologyIntersector.cpp:1092` | 存在固定 capture distance 及容差放大启发式 | 不能代替 Gensolid 的精确谓词、区间判断和统一精度策略 |
| 上游 KNOWN_ISSUES | remove_fillet、healer 有失败仍报成功的记录；周期接缝修复有多类错误 | 作为负面用例清单；不能将该内核作为唯一真值来源 |
| 上游 KNOWN_ISSUES | 圆形 trim 有内部节点时，面积/体积积分误差可能明显大于请求精度 | 建立解析圆孔板、圆柱和球的独立基准，按节点跨度分段并验证误差 |
| 上游 KNOWN_ISSUES / BREP_STAGING | 非法拓扑导入可能越界；修复会改写接缝、面和区域 | 导入先结构检查；未经修复的正确性和修复后效果分别记录 |

操作文档也需与源码及数学核对。例如“有边界交线就能代表全部实体干涉”无法覆盖完全包含，“NURBS 布尔没有任何近似”的文档表述不能代替数值误差证明。本评估未将这些描述当作算法保证。

## 5. 采用方式与许可证

本次新增[ADR 0011](adr/0011-usd-brep-reference.md)，补充现有代码来源政策。**可参考并选择性移植已核验、来源清晰的 Apache-2.0 模块；仍需保留原版权、许可证、固定提交、文件路径和修改说明。** 目前没有移植代码，因此没有新增 Gensolid 内核依赖或上游代码归属声明到 NOTICE。

根许可证不能覆盖所有外部组件：THIRD_PARTY_NOTICES 单列了 OpenUSD、提案补充条款、验证器文档、GUI 依赖等。OCCT 相关适配器按现有来源规则单独处理；本批优先 SMLib 的独立几何算法，未阅读 OCCT 内核源码或将其链接进 Gensolid。SmIntegrator 存在额外来源疑点，按上节处理。

## 6. 纳入开发计划后的顺序

1. **GS-009：先补裁剪与参数。** 参考 SmEdge/SmEdgeuse、周期区间及 staging 方向约定，按 ADR 0009 建立附属表并纳入 GS-008 事务；回归覆盖负向 fin、接缝双重使用、跨周期、整圆和缺失映射。
2. **NURBS 编辑最小模块。** 评估 N_CrvInsertKnot/N_CrvSplit 的依赖，先实现纯几何操作；验收包括几何不变、节点重数、拆分连接、端点/近节点参数及失败不改输入。Bezier 分解、升阶和曲面编辑后置。
3. **GS-010/012/015：实用能力。** 用独立实体构造、解析质量属性基准和显示网格推进 RA-001 的简单子零件。NURBS 编辑不能代替圆柱与孔特征验收。
4. **GS-016/017 与阶段 I：逐步求交、布尔和圆角。** 参考全局/局部求解分层，先通过解析范围；失败分类、误差、分支完整性及事务同时验收。复杂内核流程不整体搬入 Rust。
5. **独立对拍工作项。** 后续构建 SMLib，先对圆孔板、圆柱、裁剪周期面运行其自身检查及解析基准，再决定哪些结果可加入对拍语料；本批保持“未运行”状态。

不增加新的完成率或缩短工期承诺。此次新增参考可降低算法调研成本，但 Rust 适配、退化处理、精度与独立验证仍需逐项完成。完整 RA-001 装配仍依赖原计划阶段 D–K。

## 7. 本批交付与验证

- 已登记参考版本、来源政策、文件/符号清单及其对应 GS 任务；[开发计划](gensolid-development-plan.zh-CN.md)更新为 v1.4。
- 证据：[manifest.json](evidence/2026-10-14/nurbs-reference/manifest.json)，记录本次引用文件的哈希、SPDX、HEAD 对比和方法限制。
- 本批仅修改文档，核验文档链接、29 个既有 GS 编号、RA-001 三层目标、文件哈希及受保护路径；不重复运行未改动的 Rust/C/Python/WASM 测试。
- 上一批 81 项 Rust 测试是已有基线，详见[事务报告](transaction-development-report.zh-CN.md)，不是 SMLib 的测试结果。
