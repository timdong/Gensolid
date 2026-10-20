# Gensolid 第二批开发报告：建模事务与失败恢复

日期：2026-10-13。对应[开发计划 v1.3](gensolid-development-plan.zh-CN.md)的 GS-007/GS-008。本批交付可运行的事务基础，默认 Rust 测试由 63 项增至 81 项，全部通过；C、Python、真实 WASM 调用和严格检查通过。阶段 C 的裁剪参数编辑及世界坐标范围约束仍未完成。

## 1. 实际新增的能力

此前多步构造失败可能留下已经分配的几何与拓扑。本批让长方体、居中通孔和偏心通孔构造具有原子性：普通错误返回后恢复旧模型，可以在同一会话中继续建模。

| 工作项 | 实现结果 | 依据 |
|---|---|---|
| GS-007 裁剪语义 | 明确实际边区间、fin 方向、p-curve 使用、仿射参数映射、周期分支及内外环角色 | [ADR 0009](adr/0009-trimmed-topology.md)；仅设计契约 |
| GS-008 模型事务 | 快照覆盖八类拓扑、曲线、曲面、精度及容量设置；支持嵌套、错误与栈展开恢复 | [ADR 0010](adr/0010-atomic-model-edits.md)、[Model](../crates/gensolid-topo/src/model.rs) |
| 临时 ID 安全 | 快照共享代际分配器；回滚不倒退 generation；耗尽返回 CapacityExceeded | [Arena](../crates/gensolid-core/src/arena.rs) |
| 确定性容量配置 | Rust 新增 ModelLimits 与 Session::start_with_limits；按十类存储设置活动实体上限 | [Session](../crates/gensolid-api/src/lib.rs) |
| 创建与发布 | 预留 tag → 事务构造 → 既有 check_body → 提交并发布；故障或未执行检查返回错误并恢复 | 同上；C/Python/WASM 共享此语义 |
| 批量查询 tag | 预检不同的新实体数量和结果空间，容量错误不发布部分 tag | API 单元测试 |

公开拓扑构造函数也接入事务，但低层事务本身不做有效性检查；Session 创建入口负责发布前检查。内部通孔构造使用长方体助手，避免这一内部调用再次拍快照。旧入口和 C ABI 保持兼容；容量配置目前仅 Rust 公开。

## 2. 可运行的恢复 Demo

源码：[transaction_recovery.rs](../crates/gensolid-api/examples/transaction_recovery.rs)。在项目根目录运行：

```sh
cargo +stable run --locked -p gensolid-api --example transaction_recovery
```

过程如下：

1. 设置 fins 活动容量为 48，先创建 40×25×8 的长方体，中心位于 (0,0,4)。该体占用 24 个 fin。
2. 尝试创建相同尺寸、半径 4、偏移 (6,-3) 的通孔体。其基础长方体可分配，增加孔拓扑时触发 CapacityExceeded。
3. 核对会话仍只有旧实体，旧顶点 tag、坐标及检查结果有效，曲线和曲面数量回到 12、6。
4. 在同一会话再创建普通长方体，成功；失败的请求未消耗下一个 tag。

实际输出见[Demo 日志](evidence/2026-10-13/transactions/demo.log)：

```text
demo=transaction_recovery
injected_limit=fins:48
failure_code=CapacityExceeded
existing_body_preserved=true
next_tag_preserved=true
committed_body_count=2
recovered_body_faults=0
```

两个成功块重合放置是用于验证恢复路径的测试条件；本示例未执行相交检测或装配验收。容量限制由真实存储执行，错误发生于真实构造路径。

## 3. 测试与构建结果

以下命令均退出 0，日志保存在 [transactions 证据目录](evidence/2026-10-13/transactions/README.md)。测试数指默认 Rust 成员，Python 的 4 项单独统计。

| 检查 | 结果 | 日志 |
|---|---|---|
| 默认 Rust 测试 | 81 项通过，0 失败；较首批新增 18 项 | [rust-tests.log](evidence/2026-10-13/transactions/rust-tests.log) |
| 全工作空间 all-targets Clippy | -D warnings 通过 | [clippy.log](evidence/2026-10-13/transactions/clippy.log) |
| 全项目格式 | 通过；空日志表示无格式差异 | [fmt.log](evidence/2026-10-13/transactions/fmt.log) |
| C | 生成头文件比较、原长方体示例、通孔契约实际调用通过 | [bindings.log](evidence/2026-10-13/transactions/bindings.log) |
| Python 3.10 | 扩展安装与 4 项测试通过 | 同上 |
| WASM + Node | 实际 wasm32 构建、wasm-bindgen 生成和 Node 调用通过 | 同上 |
| Rust 1.88 编译器 | 全工作空间 all-targets 检查通过 | [msrv.log](evidence/2026-10-13/transactions/msrv.log) |
| 事务 Demo | 容量失败、旧实体保存、重试成功、tag 保持均通过 | [demo.log](evidence/2026-10-13/transactions/demo.log) |

新增 18 项的分布为 Arena 4、Model 6、Session 4、[事务集成测试](../crates/gensolid-api/tests/transactions.rs) 4。覆盖十类存储容量失败、修改全部八类旧拓扑后恢复、空快照、临时 ID 防复活、代际耗尽、嵌套事务、栈展开、tag 容量预检及检查器拒绝输出。集成属性测试覆盖随机尺寸、旋转坐标系与接近精度边界的高度；恢复后对模型中全部 body 使用既有 check_body，要求无故障且 unchecked=0。

另有自然失败用例：很大的有限世界坐标使局部角点舍入重合，几何构造返回 Degenerate。测试确认此前分配的内容恢复，旧体有效且下一次创建成功；这验证失败原子性，尚不能证明世界坐标范围已统一约束。

复现命令：

```sh
cargo +stable test --locked
cargo +stable clippy --workspace --all-targets --locked -- -D warnings
cargo +stable fmt --all -- --check
VIRTUAL_ENV=/tmp/gensolid-dev-venv bash scripts/validate-bindings.sh

RUSTC=/root/.rustup/toolchains/1.88.0-x86_64-unknown-linux-gnu/bin/rustc \
RUSTDOC=/root/.rustup/toolchains/1.88.0-x86_64-unknown-linux-gnu/bin/rustdoc \
CARGO_TARGET_DIR=/tmp/gensolid-msrv-check \
cargo +stable check --workspace --all-targets --locked --offline
```

工具版本及源码哈希见[metadata.json](evidence/2026-10-13/transactions/metadata.json)，归档文件哈希见[SHA256SUMS](evidence/2026-10-13/transactions/SHA256SUMS)。上述 venv/编译器绝对路径属于本机复现配置；其他机器需配置自己的对应环境。Windows 本机构建、浏览器和更多 Python 版本仍未验证。

## 4. 实现边界

- 当前使用完整快照，时间和内存成本随已有模型大小增长。API 与内核构造入口可能各有一层快照，后续性能优化须保持现有恢复测试。
- 回滚覆盖普通 Result 错误和 Rust 栈展开；不捕获不可恢复的系统 OOM、进程终止或 panic=abort。克隆、检查和辅助集合仍可能触发系统级分配失败。容量注入测试并不等于捕获了所有内存不足情况。
- 代际时钟有意不回滚，失败请求可消耗内部 generation；耗尽返回 CapacityExceeded。已发布会话 tag 的编号不因失败请求增加。
- ID 仅用于所属模型/arena。事务回调必须编辑提供的存储，替换整个 Model/GeomStore 不在契约内；临时 ID 不得在提交前发布。
- RawEditor 仍是底层未经检查的编辑接口。Model::transaction 保证原子性；其调用方须自行验证修改的几何与拓扑。
- ADR 0009 已确定设计，实际裁剪区间附属表、p-curve、周期接缝、拆边和环重建尚未落地；这些未来元数据必须参加同一事务。
- 本批未增加圆柱实体、多孔、盲孔、质量属性、模型交换或装配。RA-001 仍按 L1/L2/L3 目标推进。

本批没有改变几何规格，也未重新运行 Creo。此前 Creo 对照继续提供交换行为证据；未恢复其原始源码。受保护检查器、验证器、容差、AGENTS.md 和 .github 的语义差异为空，既有断言未减弱。代码由 Codex 自主实现，未移植外部几何内核代码。

## 5. 后续实施顺序

下一批按 GS-009 实现显式边区间、fin 参数使用映射及内外环元数据，先覆盖长方体和通孔的现有表示；缺失或不支持的映射返回明确错误。随后实现方向、跨周期、接缝双重使用、拆边与环重建回归，将新增数据纳入事务，并补齐世界坐标范围验证。阶段 C 达到门槛后再推进 GS-010 的圆柱、多孔和盲孔及其绑定。
