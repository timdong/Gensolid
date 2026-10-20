# 建模事务验证证据（2026-10-13）

对应 GS-007 的裁剪语义决策与 GS-008 的现有存储事务实现。中文说明见[第二批开发报告](../../../transaction-development-report.zh-CN.md)。首批证据保留于上级目录。

| 文件 | 内容 |
|---|---|
| rust-tests.log | 默认 Rust 测试：81 通过，0 失败 |
| clippy.log | 全工作空间 all-targets 严格 Clippy |
| fmt.log | 格式检查；成功时日志为空 |
| bindings.log | C 头文件比较与实际调用、Python 4 项测试、真实 WASM + Node 调用 |
| msrv.log | Rust 1.88 编译器的全工作空间 all-targets 检查 |
| demo.log | 真实容量失败与同一会话的恢复示例 |
| metadata.json | 基线 commit、工作树状态、复现命令、退出码、工具版本及源码 SHA-256 |
| implementation.patch | 七个既有 Rust 文件的语义差异及两个新增 Rust 文件；不包含原有行尾差异 |
| SHA256SUMS | 本目录归档文件的 SHA-256 |

本批所有验证命令退出 0。未进行 Windows 本机验证或新的 Creo 重放；裁剪参数、p-curve、接缝和拆边尚未实现。工作树含未提交改动，不能仅凭基线 commit 重现本批结果；应同时使用本批源码/补丁，哈希用于核对归档时的状态。
