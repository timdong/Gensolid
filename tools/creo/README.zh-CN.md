# Creo 可重放交换探针

关联 GS-005。用途：从独立参考 STEP 导入 Creo、再生、保存零件和导出 STEP，并采集可解析的特征状态。当前只接受 `import_step`；不支持原生草绘、拉伸或孔特征创建，也不是 Creo 原始源码恢复工具。

## 运行

目前启动器面向 Windows 上的 Creo + WSL，需 Linux Python 3.10+、javac、Windows PowerShell 和安装自带的 Java/J-Link。默认安装为 Creo 10.0.8.0，可通过 `--creo-root` 更改。Creo 许可证与 SDK 权限仍由安装环境提供。

```sh
python tools/creo/run_probe.py tools/creo/cases/center.json --dry-run
python tools/creo/run_probe.py tools/creo/cases/center.json
python tools/creo/run_probe.py tools/creo/cases/offset.json
python -m unittest discover -s tools/creo/tests -v
```

配置中的输入 STEP 相对配置文件解析；每次使用独立 UUID 输出目录，默认位于 `target/creo-probes/`。运行目录保存 Java 源码快照、编译日志、配置、输入 STEP、Creo 原生零件、实际导出文件及证据 JSON。输入文件复制为短文件名 `in.step`，避开已观察到的 SDK 长字符串限制。

居中与偏心输入从既有独立 build123d 参考试件归档而来，源程序分别为 `E:/code/Rea-Test/creo_probe/probe_plate.step.py` 和 `E:/code/Rea-Test/creo_probe/offset_hole/o.step.py`。这不是 Gensolid STEP 写出。

## 证据与成功含义

- `events.jsonl`：UTF-8 JSON，含特征 ID、数值类型、数值状态、状态枚举名、再生/保存/导出阶段、异常阶段、质量属性缺失原因。
- `stdout.bin`：原始混合 Java/本地 SDK 输出；`stdout.log` 是 UTF-8 可读版本，本地非 UTF-8 字节使用替换字符。不得以本地日志乱码判断模型成功。
- `summary.json`：样例参数、输入单位、执行时间、返回码、超时、源码快照和 SDK 哈希、Gensolid revision/已跟踪补丁摘要、产物哈希及进程清理记录。Java 新增内容也有独立源码哈希；已跟踪补丁摘要不代表完整未提交工作区。
- success 表示导入、再生、保存、导出和连接关闭完成，且有实际零件和导出文件。它不等于独立几何验收或原生孔特征验证。
- Creo 模型长度单位单独查询，不能把输入 STEP 的 mm 直接套到 Creo 质量属性数值。质量属性失败记录 `available=false`、空数值和异常，不用参考公式填充。几何测量仍由另一独立步骤承担。

默认执行阶段超时 180 秒，进程查询和清理各调用另有 20 秒限时。输出根目录带排他锁，防止同一根目录中的探针并行使用 Name Server。请统一使用同一个输出根目录运行探针。

清理只处理带唯一标记的 Java 进程及观察到的后代。终止前同时校验 PID、名称和原始启动时间；现有进程和复用 PID 不授予所有权。连接正常结束后清理本次启动的 nmsd，随后清理残留后代。清理异常使本次 success 为 false；异常证据保留。

## 已知待办

原生特征创建路径（GS-006）、质量属性接口异常、Creo 内部精度设置采集、独立测量自动接入及 Linux 原生 Creo 运行尚未完成。当前数值特征类型和状态可稳定解析，中文类型名仅作诊断显示。

独立几何测量可在有 OCP 的 Python 环境执行：

```sh
python tools/creo/measure_step.py tools/creo/inputs/offset.step <运行目录>/result.stp --output measurements.json
```

该命令通过公开 OCP 接口读取 STEP，明确归一到 mm，记录有效性、实体/壳/面/边数量、包围盒、圆柱半径/轴线及体积。OCP 只作为独立测试进程使用，不是 Gensolid 运行依赖；其体积不能记作 Creo 或 Gensolid 的质量属性结果。
