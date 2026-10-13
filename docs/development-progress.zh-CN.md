# Gensolid 首批开发执行报告

日期：2026-10-13。依据：[开发计划 v1.2](gensolid-development-plan.zh-CN.md)。本次完成工具链与质量基线的 Linux/WSL 部分、既有通孔的跨语言封装，以及 Creo 交换探针规范化。完整 A–K 路线和 RA-001 装配目标仍在实施前期。

## 1. 已实现内容

1. **GS-002：NURBS 质量阻塞修复。** 两处索引循环改为迭代器，保留原基函数/导数算法，未修改测试断言。全工作空间严格 Clippy 通过，既有数值回归保持通过。
2. **GS-011：既有单通孔的跨语言接口。** C 新增 `GS_BODY_create_solid_block_with_hole` 和版本化 offset 选项；Python 新增居中/偏心孔方法；WASM 新增两类孔及各自 At 坐标系方法。均调用既有 Rust 语义层，不引入第二套几何算法。决策见 [ADR 0008](adr/0008-through-hole-bindings.md)。
3. **C 空输出指针修复。** 原长方体接口曾在最后写输出时才拒绝空指针，实体和 tag 已经创建。现有长方体与新通孔接口都在建模前拒绝空输出；Rust 会话测试和独立 C 调用测试验证拒绝请求不消耗 tag。已有断言保留，并新增更强断言。
4. **绑定契约测试。** 实际调用 C 动态库、安装后的 Python 扩展和真实 WASM 模块；验证拓扑查询、有效性、世界坐标中的八个角点、居中/偏心/旋转坐标系、无效参数与数组形状错误、C 选项版本和数组释放。核心孔壁法向、圆环位置/半径和随机边界由既有 Rust 测试承担。
5. **GS-004：公开支持域文档。** [当前能力与错误契约](capabilities.zh-CN.md) 明确半径/偏移/坐标系语义、错误映射、单位责任和未实现功能。
6. **GS-005：可重放 Creo 探针。** 独立 JSON 配置、短输入文件名、每次隔离目录、Java 源码快照、UTF-8 JSON 事件、数值特征类型和状态、输入输出/SDK/源码哈希、缺失质量属性原因、超时与进程清理。进程所有权只从带本次唯一标记的 Java 根进程及观测后代建立，终止时复核 PID 与启动时间。见 [探针说明](../tools/creo/README.zh-CN.md)。

输入 STEP 是以前由独立 build123d 程序生成的参考试件，已经归档到 `tools/creo/inputs/`。Gensolid 自主构造相同规格的解析 B-rep；本次没有新增 Gensolid STEP 写出，也没有将参考模型导入当作 Gensolid 建模能力。

## 2. 构建与调用结果

| 验证项 | 结果 | 证据/边界 |
|---|---|---|
| 正式 stable 工具链 | 通过 | rustc/cargo 1.99.0，rustfmt/Clippy 与其匹配，WASM 目标已安装 |
| 格式检查 | 通过 | `cargo fmt --all -- --check`，[日志](evidence/2026-10-13/fmt.log) |
| 全工作空间严格 Clippy | 通过 | 所有 targets，`-D warnings`，[日志](evidence/2026-10-13/clippy.log) |
| 默认 Rust 测试 | 63 项通过，0 失败 | 包含 10 项通孔几何/属性测试和 4 项策略测试，[日志](evidence/2026-10-13/rust-tests.log) |
| Rust 1.88 MSRV 编译检查 | 通过 | cargo 1.99 配合显式 rustc/rustdoc 1.88；`check --workspace --all-targets --locked --offline`，仅此模式验证，不是完整标准 rustup 1.88 验收 |
| C 头文件生成一致性 | 通过 | cbindgen 0.29.4 生成临时头文件并逐字节比较 |
| C 实际构建/调用 | 通过 | 原 block 示例及新 through_hole 测试；GCC 11，C11 严格警告 |
| Python 安装/调用 | 4 项通过 | maturin 1.15.0、CPython 3.10.12；生成并安装 abi3 扩展 |
| WASM 构建/调用 | 通过 | wasm32-unknown-unknown，wasm-bindgen 0.2.129，Node 22.22.0 实际调用 |
| 探针进程/日志测试 | 4 项通过 | PID 复用、既有进程隔离、查询进程不授予根所有权、混合日志解析与批量清理，[日志](evidence/2026-10-13/probe-runner-tests.log) |
| Windows 本机 Rust 构建 | 未验证 | 当前 Windows 命令发现未找到 Rust/C 编译器；不能以 WSL 测试替代 |
| 更多 Python 版本/浏览器 WASM | 未验证 | 当前仅验证 Python 3.10 和 Node 运行环境 |

绑定的完整命令输出见 [绑定验证日志](evidence/2026-10-13/binding-validation.log)。Rust 偏心孔例子继续通过：7 面、14 边、8 顶点、0 检查错误；现在还输出实际 `Precision`。见 [演示日志](evidence/2026-10-13/gensolid-offset-demo.log)。其中体积输出仍标为解析参考公式，不能当作内核质量属性。

## 3. Creo 实际重放与独立测量

三次正式重放使用同一规范化 Java 和运行器源码，另有一次故意设置 1 秒执行时限的失败实验。

| 实验 | 执行结果 | 特征结果 | 证据 |
|---|---|---|---|
| 偏心孔 1 | 成功，约 31.34 秒 | 坐标系类型值 68、导入类型值 22；两者状态值 0 / FEAT_ACTIVE | [摘要](evidence/2026-10-13/creo-offset-1-summary.json)、[事件](evidence/2026-10-13/creo-offset-1-events.jsonl) |
| 居中孔 | 成功，约 30.49 秒 | 同上 | [摘要](evidence/2026-10-13/creo-center-summary.json)、[事件](evidence/2026-10-13/creo-center-events.jsonl) |
| 偏心孔 2 | 成功，约 28.99 秒 | 同上 | [摘要](evidence/2026-10-13/creo-offset-2-summary.json)、[事件](evidence/2026-10-13/creo-offset-2-events.jsonl) |
| 超时实验 | 按预期失败，timeout=true，非零返回码 | 未完成导入，不伪报成功 | [摘要](evidence/2026-10-13/creo-timeout-summary.json) |

三次成功实验均完成导入、再生、保存、导出、连接结束；保存了原生零件和实际 `.stp`。对全部三次成功实验和超时实验核对进程身份后，未发现本次拥有的进程残留。两次开发调试运行未计入三次正式重放证据。

运行器成功只表示操作链完成，几何验证另行进行。独立 OCP 7.8.1.1.post1 读取两个输入和三个导出件，均为 **1 个有效实体、1 个壳**；输入是 7 面/15 边，Creo 导出是 8 面/18 边。偏心孔导出的半径约 4 mm，轴线 XY 约 (6,-3) mm，包围盒约 (-20,-12.5,0) 至 (20,12.5,8) mm，两次偏心重放的几何测量一致。详细数值见 [独立测量记录](evidence/2026-10-13/creo-step-measurements.json)。交换文件因为包含运行相关信息，SHA-256 不要求相同。

Creo 模型实际长度单位查询为 **in**；独立 STEP 测量明确归一到 mm。`GetMassProperty(null)` 三次仍报 `XToolkitGeneralError`，事件记录 `available=false`，体积/面积为空。独立测量得到的约 7597.876140 mm³ 是 OCP 对导出几何的测量，不能记作 Creo 质量属性接口或 Gensolid 质量属性的成功结果。

试件使用与前次已审阅快照相同的居中/偏心孔形状，本次未新增可见几何，沿用 [偏心孔实验报告](creo-offset-hole-report.md) 的快照。本次主要验收重复交换、可解析记录及生命周期管理。三个导出 STEP 的副本与哈希已归档到本报告证据目录，原生零件留在各次 `target/creo-probes/` 隔离目录。

## 4. 复现方式

在项目根目录执行；不需要修改受保护的验证层。

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --locked

python -m venv .venv
.venv/bin/pip install 'maturin>=1.9,<2'
cargo install cbindgen --version 0.29.4 --locked
cargo install wasm-bindgen-cli --version 0.2.129 --locked
rustup target add wasm32-unknown-unknown
VIRTUAL_ENV="$PWD/.venv" bash scripts/validate-bindings.sh

python -m unittest discover -s tools/creo/tests -v
python tools/creo/run_probe.py tools/creo/cases/center.json
python tools/creo/run_probe.py tools/creo/cases/offset.json
```

`validate-bindings.sh` 自动执行头文件比较、C 构建/运行、Python 安装/测试及 WASM 构建/Node 调用。C 与 Python 原配置都生成 `libgensolid.so`，共用目标目录会覆盖产物；脚本使用各自独立构建目录，保证重复运行调用正确的库。Python 安装完成后又核对两份动态库的导出符号，并再次运行 C 测试确认未被覆盖，见 [隔离验证记录](evidence/2026-10-13/c-output-isolation.log)。独立 STEP 测量需另有 OCP 的 Python 环境，不影响核心或绑定的运行依赖。

当前 Linux 1.88 编译器检查的实际命令：

```sh
RUSTC=/root/.rustup/toolchains/1.88.0-x86_64-unknown-linux-gnu/bin/rustc \
RUSTDOC=/root/.rustup/toolchains/1.88.0-x86_64-unknown-linux-gnu/bin/rustdoc \
CARGO_TARGET_DIR=/tmp/gensolid-msrv-check \
cargo +stable check --workspace --all-targets --locked --offline
```

此命令验证当前代码可以由 1.88 编译器检查。1.88 的 rustup 目录仍缺安装清单，正式工作流已恢复为完整 stable；没有为绕过工具链故障更改项目 MSRV 或 `rust-toolchain.toml`。

## 5. 剩余开发与实施边界

- 阶段 A 的 Windows 本机构建、更多绑定平台覆盖和完整标准 1.88 工具链验收仍待补齐。
- GS-005 交换探针已形成可重放工具；Creo 内部精度设置、质量属性异常及更全面失败分类继续记录。GS-006 的原生草绘/拉伸/孔链尚未验证。
- 下一项内核工作按 GS-007/008 推进裁剪拓扑语义与编辑事务，随后才扩展圆柱实体、多孔和盲孔。GS-011 的后续操作封装随稳定 API 补齐。
- 质量属性、STEP/XT 实体交换、布尔、特征重建、壳体与装配仍未完成；RA-001-L1/L2/L3 目标完整保留在计划内。

本次未改动验证器、检查器、容差模块、AGENTS.md 或 CI 的语义；工作区原有 CRLF 变化保留。没有降低检查标准或改弱测试断言。新增实现为自主 Rust/绑定/公开 SDK 调用；OCP 仅用于独立测量，未移植其源码，未恢复 Creo 原始核心代码。
