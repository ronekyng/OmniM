# Omcom 规范 — 统一工程命令体系

> 状态：与 Phase 7 代码库同步。本文描述 `proto/omcom.proto`、`crates/omcom`、`crates/amir`
> 与 Web 端 Omcom Console 中**实际存在**的能力。计划中的**文本**形态（覆盖同一套操作）属于
> 未来 RFC —— 参见 [RFC-0001](../RFC/RFC-0001-omcom-text-dsl.zh-CN.md)。

## 1. 定位：所有变更路径汇聚于 OmniCommand

Omcom（**Om**niM **Com**mand）是 OmniM 的统一命令体系。工程模型的每一次变更——无论是
GUI 中的人工操作、Omcom Console 中直接输入、AMIR AI 层的规划，还是导入流水线——都以
类型化的 `OmniCommand` 信封表达，并经过同一套「校验 + 执行」管线。

```
  GUI（项目工作区：拖放/点击/构件面板）
  Omcom Console（JSON 编辑器 + 快捷方式）──┐
  AMIR（自然语言 → 意图 → 命令）           ├─→ OmniCommand ─→ Validator ─→ Executor ─→ 模型
  导入 / IFC / 脚本 ──────────────────────┘      （类型化）   （多阶段）   （撤销/重做、
                                                                      事务）
```

### 1.1 今日的 Omcom

- 它是**类型化命令体系**，不是一门文本语言。契约位于 `proto/omcom.proto`：一个
  `OmniCommand` 信封加约 50 个操作消息，经由 `prost` 编译为支持 `serde` 的 Rust 类型。
- `crates/omcom` 包含两个组件：**Validator**（多阶段、无状态）与 **Executor**
  （有状态，支持撤销/重做与项目级事务）。
- Web 端 **Omcom Console** 接受 **JSON** 载荷（序列化的 `OmniCommand`），另提供少量
  ad-hoc 文本快捷方式；目前不解析任何语法。
- 仓库中不存在基于 `nom` 的文本解析器。本文档早期版本曾把文本 DSL 描述为已交付能力，
  那是不准确的；文本形态现作为设计提案记录于 RFC-0001。

### 1.2 设计原则

- **单一网关**：所有变更路径产出相同的类型化信封，校验、审计、撤销/重做与权限在
  任何入口的行为完全一致。
- **类型化而非解析**：结构由 protobuf schema 与 prost 代码生成保证；畸形命令在
  序列化/反序列化阶段即失败，不会中途破坏状态。
- **先校验后执行**：Executor 内嵌运行 Validator；无效命令绝不修改状态。
- **可逆**：每个已执行操作都会记录一条反向 `Command` 用于撤销。
- **可审计**：每个信封携带 `id`、`Actor`、`timestamp` 与自由元数据。

## 2. 命令模型

### 2.1 OmniCommand 信封

```protobuf
message OmniCommand {
  string version = 1;
  string id = 2;                    // 调用方生成的唯一 id
  Action action = 3;                // 约 60 个枚举动作之一
  Actor actor = 4;                  // 发起者
  Target target = 5;                // 作用对象
  repeated Operation operations = 6; // 实际工作项
  ValidationResult validation = 7;
  Timestamp timestamp = 8;
  map<string, string> metadata = 9;
}
```

### 2.2 Actor

`Actor { id, type }`，`ActorType` 取值：

| ActorType | 含义 | 典型来源 |
|-----------|------|----------|
| `USER` | 人工直接操作 | GUI、Omcom Console |
| `AI_AGENT` | AMIR 生成的命令 | POST /api/amir/translate |
| `SYSTEM` | 框架维护操作 | server |
| `IMPORT` | 文件导入流水线 | IFC 导入 |

### 2.3 Target

`Target` 是按作用域划分的 oneof：

| 目标类型 | 字段 | 用于 |
|----------|------|------|
| `ObjectTarget` | `object_id` | 修改/删除/设属性/设材质/… |
| `GeometryTarget` | `geometry_id` | 几何级操作 |
| `SelectionTarget` | `object_ids[]` | 来自 GUI 选集的批量操作 |
| `SceneTarget` | `scene_id` | 场景级操作 |
| `LifecycleTarget` | `record_id` | 生命周期记录变更 |

### 2.4 操作家族

`Action`（信封枚举，约 60 个值）与 `Operation`(消息 oneof，约 51 个变体) 描述实际变更。
以下清单来源于 `proto/omcom.proto` 实际调研：

| 家族 | Action 值 | 操作消息（节选） |
|------|-----------|------------------|
| **对象** | `CREATE_OBJECT`、`MODIFY_OBJECT`、`DELETE_OBJECT`、`COPY_OBJECT`、`MOVE_OBJECT` | `CreateObjectOperation`、`ModifyObjectOperation` 等 |
| **几何** | `CREATE_GEOMETRY`、`MODIFY_GEOMETRY`*、`TRANSFORM_GEOMETRY`、`BOOLEAN_OPERATION` | `TransformGeometryOperation`（平移/旋转/缩放）、`BooleanOperation`（并/差/交） |
| **属性** | `SET_PROPERTY`、`DELETE_PROPERTY`、`ADD_PROPERTY_SET`、`REMOVE_PROPERTY_SET` | `Property`、`PropertySet` 等 |
| **关系** | `ADD_CONTAINMENT`、`REMOVE_CONTAINMENT`、`ADD_CONNECTION`、`REMOVE_CONNECTION` | `Containment`、`Connection` 及对应移除消息 |
| **批量** | `ARRAY`、`BATCH_DELETE`*、`BATCH_TRANSFORM`* | `ArrayOperation`（线性/矩形/环形阵列） |
| **材质** | `SET_MATERIAL` | `SetMaterialOperation` |
| **生命周期** | `CREATE_LIFECYCLE_RECORD`、`MODIFY_LIFECYCLE_RECORD`、`DELETE_LIFECYCLE_RECORD`、`LINK_RECORD_TO_OBJECT`、`UPDATE_SCHEDULE`、`UPDATE_COST`、`UPDATE_PROGRESS`、`CREATE_WORK_ORDER`、`UPDATE_WORK_ORDER`、`INGEST_SENSOR_DATA` | 对应 `*Operation` 消息 |
| **草图** | `CREATE_SKETCH`、`ADD_SKETCH_ENTITY`、`ADD_SKETCH_CONSTRAINT`、`SOLVE_SKETCH`、`DELETE_SKETCH_ENTITY`、`REMOVE_SKETCH_CONSTRAINT`、`DELETE_SKETCH` | `CreateSketchOperation`、`AddSketchEntityOperation`（直线/圆/弧/矩形/多边形/椭圆/样条/多段线）、`AddSketchConstraintOperation`（19 种约束类型） |
| **特征 / 直接建模** | `EXTRUDE_SKETCH`、`REVOLVE_SKETCH`、`CREATE_COMPONENT`、`ADD_FEATURE`、`REMOVE_FEATURE`、`REBUILD_FEATURES` | `ExtrudeSketchOperation`、`RevolveSketchOperation`、`CreateComponentOperation`、`AddFeatureOperation`（FeatureType：拉伸、旋转、扫掠、放样、布尔、倒角、圆角、阵列、抽壳、融合、放样融合） |
| **导入/导出** | `IMPORT_FILE`、`EXPORT_FILE`、`PARSE_TO_OMCOM` | — |
| **元件库** | `DEFINE_COMPONENT`、`ADD_COMPONENT_PARAM`、`ADD_CONNECTOR`、`ADD_COMPONENT_FEATURE`*、`PUBLISH_COMPONENT`、`LIST_CATALOG`、`DELETE_COMPONENT_DEF` | `DefineComponentOperation`、`AddComponentParamOperation`、`AddConnectorOperation`、`PublishComponentOperation`、`DeleteComponentDefOperation` |
| **系列库** | `CREATE_SERIES`、`ADD_SELECTOR`、`ADD_ANSWER`、`ADD_SPEC_COMPONENT`、`ADD_PARAM_CONSTRAINT`、`LIST_SERIES`、`DELETE_SERIES`、`LOAD_SERIES` | `CreateSeriesOperation`、`AddSelectorOperation`、`AddAnswerOperation`、`AddSpecComponentOperation`、`AddParamConstraintOperation`、`LoadSeriesOperation` |
| **校验** | `VALIDATE` | — |

\* = schema 与 `Action` 枚举中已预留；proto 注释中标注为「reserved for future」。

一条信封携带 `repeated Operation operations`，因此单条命令可以表达多步骤编辑
（例如一次性添加多条草图实体与约束）。

围绕实例化存在「一命令分解为多操作」的原则：从系列实例化（AMIR 的
`SelectFromSeries`）会分解为创建对象 + 包含关系 + 属性集等操作。

## 3. 校验与执行

`crates/omcom` 基于 `omnim_core::OmniCommand` 类型实现上述两个阶段。

### 3.1 Validator 阶段

`Validator::validate(instr, project) -> Vec<String>` 是关于命令与项目快照的无状态函数，
按（概念上的）阶段运行：

1. **信封校验** —— `id` 非空、`Action` 可从枚举值解析。
2. **目标校验** —— 对以目标为作用域的动作，引用的对象 / 生命周期记录 / 草图 /
   系列 / 元件定义必须存在于项目或库中。
3. **操作级校验** —— 逐消息规则：
   - `CreateObject`：`ObjectType` 合法、名称非空、引用的父级存在；与既有对象的
     空间包围盒重叠会被检测并记录为**警告**（多专业工程中重叠是常态：管道穿墙等）。
   - `ModifyObject` / `CopyObject` / `MoveObject` / `SetMaterial` / `AddContainment` /
     `AddConnection` / `TransformGeometry` / `Array` / `Boolean`：所有引用的对象 ID
     必须存在；材质必须存在于材质库。
   - `CreateWorkOrder`：资产对象必须存在。
   - `AddFeature` / `RemoveFeature`：构件对象与引用的草图必须存在。
   - 元件库/系列库操作：元件定义或系列必须存在于库 / 系列库中；`DefineComponent`
     拒绝空名称；`LoadSeries` 必须提供路径。
4. **结果** —— 错误字符串列表；Executor 在列表非空时拒绝执行。

### 3.2 Executor、事务、撤销/重做

`Executor` 为有状态组件，内部持有：

- `undo_stack: Vec<Command>`（最大深度 200）
- `redo_stack: Vec<Command>`
- `transaction_stack: Option<Vec<Command>>` —— 当前活跃事务

`execute(instr, project)` 的执行流程：

1. 运行 `Validator::validate`；任一错误即以 `success: false` 中止。
2. 顺序执行每个操作并写入 `Project`，同时将一条反向 `Command`
   （基于快照：`ObjectSnapshot`、旧属性值、旧几何、旧关系）压入本次调用的
   `commands` 向量。
3. 若执行中途失败（例如 `add_containment` 循环检查未通过），调用
   `fail_and_rollback`：撤销本指令已应用的操作并返回错误——指令是全有或全无的。
4. 将本指令的操作合并进撤销栈（或合并入活跃事务）。

事务：

- `begin_transaction()` 开启一个嵌套命令缓冲；其后在事务内执行的连续指令
  记录于其中。
- `commit()` 把缓冲合并为撤销栈中的**一个**撤销单元——整个事务只需一次
  `undo()` 即可整体撤销。
- `rollback()` 撤销并丢弃缓冲中的命令。

撤销/重做经由 `Command` 枚举互为镜像（约 45 个变体，覆盖对象、属性、关系、材质、
生命周期、草图、特征、元件库与系列库）。

### 3.3 接口形态

- 服务端 API：`POST /api/omcom` 接受序列化的 `OmniCommand`（JSON），
  返回 `ExecutionResult { success, message, created_ids }`。
- Web 端 Omcom Console 直接编辑 JSON 载荷（撤销/重做是前端提供的便捷动作），
  脚本与 CI 流水线同样以此方式提交。

## 4. 各入口如何产出命令

### 4.1 GUI（项目工作区）

构件面板与检视器在客户端直接构造 `OmniCommand` 信封（Actor = `USER`，
Target 为当前选集或显式对象），并 POST 到 `/api/omcom`。批量 UI 动作使用
`SelectionTarget`。

### 4.2 Omcom Console（JSON）

Console 是 JSON 命令编辑器：用户组装 `OmniCommand` 信封（action + operations +
target + actor），由 Console 序列化后提交到 `/api/omcom`。前端提供少量 ad-hoc 文本
快捷方式（如撤销/重做便捷动作），但不存在被解析的语法。命令来源（AI 或人工）
可通过 `Actor` 字段追溯。

### 4.3 AMIR（自然语言）

AMIR crate 实现 `PromptParser → IntentClassifier → OmcomGenerator`
（`crates/amir/src/generator.rs`）：

1. `PromptParser` —— 自然语言 + 模板提示词 → 结构化 `ParsedIntent`
   （前端 JSON 意图，机器可读，规则确定）。
2. `OmcomGenerator::generate(intent) -> Vec<OmniCommand>` —— 每个受支持的意图映射为
   一条或多条类型化 OmniCommand（例如 `Intent::CreateObject` → 带
   `CreateObjectOperation` 的 `CreateObject` OmniCommand；`Intent::SelectFromSeries`
   → 创建 + 包含 + 属性等操作）。
3. 生成的命令进入同一 Omcom 管线校验并执行；校验失败时 API 会把 validator 的错误
   字符串回传给用户（AMIR chat 可据此修正并重试）。

AMIR **今天不会**输出自由文本 DSL。「AMIR 输出 Omcom 文本语言」的最初构想保留为设计
方向，见 [RFC-0001](../RFC/RFC-0001-omcom-text-dsl.zh-CN.md)。

### 4.4 导入流水线

文件导入（IFC 等）通过 `IMPORT_FILE` / `PARSE_TO_OMCOM` 动作表达，Actor = `IMPORT`，
因此导入的变更也经过同一校验，且可撤销。

## 5. 演进路径：Omcom 文本形态（未来 RFC）

Omcom 的长期愿景包含在同一套类型化操作之上的**可人工输入文本形态**：面向工程师的
确定性语法（例如 `create` / `sketch` / `feature` / `series` 动词），解析为标准
`OmniCommand` 信封，使工程师能编写脚本化工作流，并使模型能在「命令脚本 ↔ 模型」之间
双向往返。

本阶段刻意**不**把该能力纳入已交付契约：

- `crates/*` 中尚无文本语法；
- Console 目前接受 JSON;
- AMIR 目前输出类型化命令。

设计草案（语法形态、create/sketch/extrude/库操作的示意示例、逆向 model→text 生成）
见 [RFC-0001 — Omcom 文本 DSL](../RFC/RFC-0001-omcom-text-dsl.zh-CN.md)。落实该 RFC 时
会增加一个解析 crate，负责 text ↔ `OmniCommand` 的双向转换；本节描述的信封、校验器
与执行器保持不变。

## 6. 版本化

契约以信封自身承载版本：

- 每条命令的 `OmniCommand.version` 字段。
- `Action` 枚举值与 `Operation` oneof 变体做增量添加；删除视为破坏性变更。
- Prost/serde 代码生成是兼容性工具链；旧版执行器接受的命令在新版本中必须保持可反
  序列化（遵循 proto3 增量规则）。

当前 proto 模块版本以仓库根 `proto/` 中 `omnim.proto` / `omcom.proto` 为准。

## 7. 待决问题

- 事务作用域：应支持嵌套组合还是保持扁平？
- 按 Actor 类型 × 动作的权限模型。
- OmniCommand 信封的持久审计日志（当前撤销/重做仅存于内存）。
- 多客户端并发策略（按项目加锁 vs CRDT 分支）。
- 文本形态语法的具体选型（见 RFC-0001 待决问题）。
