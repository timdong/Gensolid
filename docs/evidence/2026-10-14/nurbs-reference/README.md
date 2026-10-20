# NURBS / USD BRep 静态参考核验

日期：2026-10-14。评估见[中文报告](../../../nurbs-reference-assessment.zh-CN.md)，来源规则见[ADR 0011](../../../adr/0011-usd-brep-reference.md)。

`manifest.json` 登记参考仓库、origin、固定提交、34 个引用文件的原始字节 SHA-256、行数、文件头 SPDX 和与 HEAD 的换行归一化比较。34 个引用文件均与 HEAD 一致（忽略 CRLF）；这不表示整个工作树没有语义改动。SMLib 的 462 个 `.cpp/.h` 文件头均标注 Apache-2.0；这不等于已完成所有代码的来源或算法正确性审查。

本批未修改参考仓库，未构建或运行 SMLib，未移植其算法到 Gensolid。评估中的代码分支问题为静态观察，上游已知误差数据来自其 KNOWN_ISSUES。`SHA256SUMS` 可用于核对本目录的清单及说明。
