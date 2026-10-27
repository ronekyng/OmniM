# OmniCommand 参考 — 类型化命令系统

最后更新：2026-10-27

> **注**：Omcom 的*文本*形态只是草案 RFC（RFC-0001），并未实现；见 [../RFC/RFC-0001-omcom-text-dsl.md](../RFC/RFC-0001-omcom-text-dsl.md)。
> 当前提线上下文里真正存在的是这里描述的**类型化**命令系统：一个由 Protobuf 定义的
> `OmniCommand` 信封（以 JSON 发送），由 Rust crate `crates/omcom`（`omnim_omcom`）校验并执行。

---

## 1. OmniCommand 是什么

OmniCommand 是 Omnim 项目所有变更的统一、类型化协议。一条 `OmniCommand` 消息携带一个
动作码、一个执行者（Actor）、一个可选目标（Target），以及一组具体的 `Operation` 消息。
这里**没有任何需要解析的文法**：操作是 [`proto/omcom.proto`](../../proto/omcom.proto) 中
定义的类型化消息，经 [`prost`](https://docs.rs/prost) 编译成支持 `serde` 的 Rust 类型 ——
因此同一条命令可以以 **JSON** 序列化（snake_case 字段名、枚举为整数）直接 POST 到服务端。

命令的来源：

- **Web 端 Omcom 控制台**构造 `OmniCommand` 对象并以 JSON 发送。
- **REST API** —— `POST /api/omcom` 接收序列化的 `OmniCommand`；
  完整端点列表见 [API.md](API.zh-CN.md)。
- **AI 代理及其他客户端**以同样的 JSON 结构提交，Actor 类型为 `ACTOR_TYPE_AI_AGENT`(2)。

执行由 `crates/omcom/src/lib.rs`（约 3,645 行，单文件）负责，其中分三层：

1. **`Validator`** —— 在任何变更发生*之前*，依据当前 `Project` 状态检查命令。校验失败则整条命令中止。
2. **`Executor`** —— 将每个 `Operation` 应用到项目上，并为每次变更记录一条可逆向的 `Command`。
3. **`undo` / `redo` / 事务** —— 有界历史（深度 200）加显式事务 API（`begin_transaction` / `commit` / `rollback`）。

### 校验与事务语义（一段话概括）

`Executor::execute` 首先调用 `Validator::validate`，其返回一组错误字符串；出现任何错误即整条
命令短路失败（`ExecutionResult { success: false, message: "Validation failed: …" }`）。校验通过后
逐个应用操作；每次变更都会把一条反向 `Command` 压入事务栈。若中途某操作失败（例如包含关系出现环），
执行器会回滚已应用的部分（`fail_and_rollback`）并返回失败。处于 `begin_transaction()` 事务块内时，
命令先暂存，仅在 `commit()` 时一次性推入 undo 栈；`rollback()` 则立即执行各命令的反向操作。undo 栈
深度上限 **200**（`MAX_UNDO_DEPTH`），超出则淘汰最旧记录。此外，每次新的执行都会清空 redo 栈。

---

## 2. 命令信封

`OmniCommand`（`proto/omcom.proto`）：

| 字段 | JSON 名 | 类型 | 说明 |
|---|---|---|---|
| version | `version` | `string` | 信封版本 |
| id | `id` | `string` | **必须非空**（校验错误 `instruction id is empty`） |
| action | `action` | `Action`（整数枚举） | 取自动作枚举的一个值，见附录 A |
| actor | `actor` | `Actor` | `{ id, type }` |
| target | `target` | `Target`（oneof） | 可选；属性/材料等动作使用 |
| operations | `operations` | `Operation[]` | 具体工作；多数动作恰好一个 |
| validation | `validation` | `ValidationResult` | 服务端校验后回填 |
| timestamp | `timestamp` | `Timestamp` | `{ seconds: int64, nanos: int32 }` |
| metadata | `metadata` | `map<string,string>` | 自由键值 |

### 执行者类型（枚举 `ActorType`）

| 值 | 名称 | 含义 |
|---|---|---|
| 0 | `ACTOR_TYPE_UNSPECIFIED` | 未设置 |
| 1 | `USER` | 人类用户 |
| 2 | `AI_AGENT` | AI 代理 |
| 3 | `SYSTEM` | 系统 |
| 4 | `IMPORT` | 导入管线 |

### 目标（oneof `target_type`）

| 变体 | JSON 形状 | 用途 |
|---|---|---|
| `object` | `{"object": {"object_id": "…"}}` | `SET_PROPERTY`、`DELETE_PROPERTY`、`ADD_PROPERTY_SET`、`SET_MATERIAL`、`TRANSFORM_GEOMETRY` 及对象级生命周期目标 |
| `geometry` | `{"geometry": {"geometry_id": "…"}}` | 几何级动作 |
| `selection` | `{"selection": {"object_ids": ["…"]}}` | 批量 / 控制台选集 |
| `scene` | `{"scene": {"scene_id": "…"}}` | 全场景动作 |
| `lifecycle` | `{"lifecycle": {"record_id": "…"}}` | 生命周期记录动作 |

`SET_PROPERTY`、`ADD_PROPERTY_SET` 等动作由 Validator 和执行器在 `Target` 中查找受影响对象；
多数其他操作自带 `*_id` 字段。

> **JSON 编码说明**：经 prost + serde 生成后，枚举序列化为**整数**（`"action": 1` 表示
> `CREATE_OBJECT`），字段名为 snake_case，proto3 `optional` 字段未设置时输出 `null`。
> oneof `target.target_type` 序列化为单个键，例如 `{"object":{"object_id":"…"}}`。

---

## 3. 操作族

`Operation` 是一个 oneof，共 **51 个变体**（`create_geometry` … `load_series`）。按语义
分为以下几族。每个操作给出：消息名、关键字段（JSON snake_case；枚举字段为整数）、
Validator 检查什么、Executor 变更/快照什么。除非另作说明，ID 均为系统生成的 UUID 字符串。

### 3.1 对象创建与修改

**`create_object` → `CreateObjectOperation`**（action `CREATE_OBJECT` = 1）

- 字段：`type`（ObjectType，整数）、`name`、`description`、`geometry`（ParametricGeometry）、
  `properties`（Property[]）、`parent_id`。
- Validator：对象类型必须是合法 `ObjectType`；`name` 非空；`parent_id`（若填写）必须存在。
  与现有对象包围盒的重叠仅按**警告**记录（stderr），不算阻塞错误 —— 多专业工程中管线穿墙、
  柱在墙内等重叠很常见。
- Executor：创建 `OmniObject`，依据 `ParametricGeometry` 构建真实几何
  （经 `omnim_model::geometry::create_geometry`），插入项目，并把新 ID 放入 `created_ids`。

**`modify_object` → `ModifyObjectOperation`**

- 字段：`object_id`，可选 `new_name`、`new_description`、`new_geometry`。
- Validator/Executor：目标对象必须存在；对名称/描述/几何做快照以供 undo。

**`copy_object` → `CopyObjectOperation`**

- 字段：`source_object_id`、`new_name`、`parent_id`。
- Validator：源必须存在；目标父级（若填写）必须存在。
- Executor：以新 UUID 深拷贝对象；undo 删除副本。

**`move_object` → `MoveObjectOperation`**

- 字段：`object_id`、`new_parent_id`。
- Validator：两者必须存在。
- Executor：重新挂父级；undo 恢复原父级。

### 3.2 属性与关系

**`set_property` → `Property`**，action `SET_PROPERTY` (20)。`Target.object` 携带对象；
`Property` 为 `{ name, value, unit, description }`，其中 `value` 是 oneof
（`string_value`、`int_value`、`double_value`、`bool_value`、`point_value`、`vector_value`、
`binary_value`、`timestamp_value`）。Validator：目标对象必须存在。undo 恢复旧值（存在旧的则覆盖，若无则回退为未设置态）。

**`delete_property` → `DeletePropertyOperation`**（`name`）。undo 恢复被删除的属性。

**`add_property_set` → `PropertySet`**（`{ name, description, properties }`）。追加到
`Target.object` 的属性集列表。

**`remove_property_set` → `RemovePropertySetOperation`**（`name`）。undo 恢复该属性集。

**`add_containment` → `Containment`**（`{ parent_id, child_id, … }`）。Validator：两个对象
必须存在。Executor：调用 `project.relations.add_containment` —— 这是库本身可以拒绝操作的
少数场景之一（例如出现环），会触发执行器的"中途失败即回滚"路径。undo 移除该包含关系。

**`remove_containment` → `RemoveContainmentOperation`**（`parent_id`、`child_id`）。undo 恢复。

**`add_connection` → `Connection`**（`from_object_id`、`to_object_id` 等）。Validator：两个
端点必须存在。undo 移除该连接。

**`remove_connection` → `RemoveConnectionOperation`**（`from_object_id`、`to_object_id`）。

### 3.3 材料

**`set_material` → `SetMaterialOperation`**（`object_id`、`material_id`）。Validator：对象
和材料都必须存在于项目的材料库中。Executor 设置 `obj.material_id`；undo 恢复原
`material_id`（包括"无材料"状态）。

### 3.4 几何：创建 / 变换 / 阵列 / 布尔

**`create_geometry` → `ParametricOperation`**（action `CREATE_GEOMETRY` = 10）。与
`create_object` 内联几何并行的另一条几何权柄 —— `ParametricGeometry` 由 `ParametricOperation`
组成：`box`、`cylinder`、`sphere`、`cone`、`extrusion`、`revolve`、`sweep`、`loft`、`torus`、
`dish`（来自 `omnim.proto`）。Validator：动作级目标对象必须存在。

**`transform_geometry` → `TransformGeometryOperation`**

- 字段：`object_id`，可选 `translation`（Point3D）、`rotation`（Rotation，四元数
  x/y/z/w）、`scale`（Vector3D）。
- Validator：对象必须存在。Executor：把变换烘焙进对象的几何；undo 恢复原几何。

**`array` → `ArrayOperation`**

- 字段：`source_object_id`、`pattern`（字符串：`"linear"` / `"rectangular"` / `"circular"`）、
  `count_x/y/z`（int32，下限 1）、`spacing_x/y/z`（double）。
- Validator：源必须存在。Executor：对除 `(0,0,0)` 外的每个 `(ix,iy,iz)` 单元复制源对象，生成
  新 UUID 与名称后缀 `"Name (i,j,k)"`，几何按 `i*spacing_i` 偏移；undo 删除全部副本。

**`boolean` → `BooleanOperation`**

- 字段：`type`（字符串：`"union"` / `"subtract"` / `"intersect"`）、`target_object_id`、
  `operand_object_ids`（repeated）。
- Validator：目标与全部操作数必须存在。Executor 施行网格布尔（`omnim_mesh`）；undo 恢复布尔
  之前的目标几何。

### 3.5 草图

草图的实体索引从 1 开始；约束通过 `SketchPointRef`（`entity_index` 1 起始，`point_index`
0=自由点/1=起点/2=终点）引用实体。实体类型（`SketchEntityType`）：点、线、圆、圆弧、工作面、
矩形、多边形、椭圆、样条、多段线。约束类型（`SketchConstraintType`）：水平、竖直、重合、
平行、垂直、长度、距离、角度、半径、直径、相切、对称、固定、在中点、等长、等半径、点在线上、
点在圆上、同向。

| 操作（oneof） | 消息 | 关键字段 | Validator | Executor |
|---|---|---|---|---|
| `create_sketch` | CreateSketchOperation | `name`、`origin`(Point3D)、`normal`(Vector3D；`(0,0,0)` = 自由 3D) | — | 创建草图；undo 删除 |
| `add_sketch_entity` | AddSketchEntityOperation | `sketch_id`、`entity`(SketchEntityData) | 草图存在 | 追加实体；undo 按索引移除 |
| `add_sketch_constraint` | AddSketchConstraintOperation | `sketch_id`、`constraint`(SketchConstraintData) | 草图存在 | 追加约束；undo 移除 |
| `solve_sketch` | SolveSketchOperation | `sketch_id` | 草图存在 | 调 `omnim_sketch` 求解器；undo 恢复 `solved` 状态 |
| `delete_sketch_entity` | DeleteSketchEntityOperation | `sketch_id`、`entity_index`(1 起始) | 草图存在 | undo 重新插入实体 |
| `remove_sketch_constraint` | RemoveSketchConstraintOperation | `sketch_id`、`constraint_index`(1 起始) | 草图存在 | undo 重新插入约束 |
| `delete_sketch` | DeleteSketchOperation | `sketch_id` | 草图存在 | undo 恢复草图 |
| `extrude_sketch` | ExtrudeSketchOperation | `sketch_id`、`depth`、`direction` | 草图存在 | 由闭合轮廓生成 3D 体；undo 删除 |
| `revolve_sketch` | RevolveSketchOperation | `sketch_id`、`angle`(弧度，0=完整 360°)、`axis_origin`、`axis_direction` | 草图存在 | 生成旋转体；undo 删除 |

草图直接拉伸/旋转也可经 `add_feature`（`ADD_FEATURE` = 111）到达。

### 3.6 组件与特征树

组件拥有有序的特征列表；特征引用草图并可缓存网格几何。`FeatureType`：extrude(1)、
revolve(2)、sweep(3)、loft(4)、boolean(5)、chamfer(6)、fillet(7)、pattern(8)、shell(9)、
fusion(10)、lofted_fusion(11)。

| 操作 | 消息 | 关键字段 | Validator | Executor |
|---|---|---|---|---|
| `create_component` | CreateComponentOperation | `name`、`parent_id` | — | 创建组件；undo 删除 |
| `add_feature` | AddFeatureOperation | `component_id`、`feature_type`、`sketch_id`、`depth`/`direction`（拉伸）、`angle`/`axis_origin`/`axis_direction`（旋转）、`is_cut`、`input_sketch_ids`（扫掠路径 / 放样截面）、`axis_sketch_id` | 组件对象存在；草图存在 | 增加特征；undo 移除 |
| `remove_feature` | RemoveFeatureOperation | `component_id`、`feature_id` | 组件存在 | undo 恢复该特征 |
| `rebuild_features` | RebuildFeaturesOperation | `component_id`，可选 `feature_id`（从此特征向后重建；缺省 = 全部重建） | — | 重新生成组件几何；重建是幂等的，无 undo 操作 |

### 3.7 目录发布（库级构件）

这些操作作用于全局构件库（`omnim_model::catalog`，存于 `LIBRARY`），而不是项目内对象。

| 操作 | 消息 | 关键字段 | Validator | Executor |
|---|---|---|---|---|
| `define_component` | DefineComponentOperation | `name`、`category`、`description`、`ifc_class`、`standard_reference`、`tags`、`domain` | name 非空 | 注册 ComponentDef；undo 移除 |
| `add_component_param` | AddComponentParamOperation | `component_id`、`param_name`、`param_type`（ParamType 整数）、`default_value`、`min_value`、`max_value`、`step`、`unit`、`formula`、`choices`、`description` | 目录构件存在 | 追加参数；undo 按索引移除 |
| `add_connector` | AddConnectorOperation | `component_id`、`connector_id`、`connector_name`、位置/方向/`bore` 的*表达式*字段（`position_x_expr` 等）、`connection_type_code`、`role`（ConnectorRole 整数） | 构件存在 | 追加 ConnectorDef；undo 移除 |
| `publish_component` | PublishComponentOperation | `component_id` | 构件存在 | 标记发布 |
| `delete_component_def` | DeleteComponentDefOperation | `component_id` | 构件存在 | undo 恢复该定义 |

`ParamType`：unspecified 0、length 1、angle 2、integer 3、choice 4、boolean 5、ratio 6。
`ConnectorRole`：unspecified 0、inlet 1、outlet 2、bidirectional 3、mount 4、branch 5。

### 3.8 系列库（规格选型）

规格图册式系列：一棵由问题与答案组成的 `Selector` 树，每个答案绑定规格构件与参数约束。

| 操作 | 消息 | 关键字段 | Validator | Executor |
|---|---|---|---|---|
| `create_series` | CreateSeriesOperation | `name`、`description`、`domain`、`standard_reference` | — | 创建 Series |
| `add_selector` | AddSelectorOperation | `series_id`、`selector_id`、`question` | 系列存在 | 追加选择器 |
| `add_answer` | AddAnswerOperation | `series_id`、`selector_id`、`answer_id`、`answer_value`、`is_numeric_range`、`range_min`、`range_max` | 系列存在 | 追加答案；undo 按索引移除 |
| `add_spec_component` | AddSpecComponentOperation | `series_id`、`selector_id`、`answer_id`、`component_def_id`、`material`、`connection_type_code` | 系列存在 | 挂接规格构件 |
| `add_param_constraint` | AddParamConstraintOperation | `series_id`、`component_def_id`、`param_name`、`default_value`、`min_value`、`max_value`、`step`、`fixed_value` | 系列存在 | 追加约束 |
| `delete_series` | DeleteSeriesOperation | `series_id` | 系列存在 | undo 恢复该系列 |
| `load_series` | LoadSeriesOperation | `path`、`source`（`"standalone"` \| `"library"`） | path 非空 | 从磁盘加载系列；undo 移除已加载 ID |

### 3.9 生命周期、进度计划、成本、进度、工作单

均作用于项目内的 `ProjectLifecycleRecord`。

| 操作 | 消息 | 关键字段 | Validator | Executor |
|---|---|---|---|---|
| `create_lifecycle_record` | CreateLifecycleRecordOperation | `phase`（LifecyclePhase 整数）、`name`、`description`、`record_type`、`related_object_ids`、`properties` | — | 创建记录，状态初始化为 `Draft`(1)；undo 删除 |
| `modify_lifecycle_record` | ModifyLifecycleRecordOperation | `record_id`，可选 `name`/`description`，可选 `status`（RecordStatus 整数），`properties` | 记录存在 | undo 恢复旧 name/description/status |
| `delete_lifecycle_record` | DeleteLifecycleRecordOperation | `record_id` | 记录存在 | undo 恢复该记录 |
| `link_record_to_object` | LinkRecordToObjectOperation | `record_id`、`object_id`、`link_role` | 两者存在 | undo 取消链接 |
| `update_schedule` | UpdateScheduleOperation | `record_id`，可选 `planned_start`/`planned_finish`/`actual_start`/`actual_finish`（Timestamp） | 记录存在 | undo 恢复旧时间戳 |
| `update_cost` | UpdateCostOperation | `record_id`、`planned_cost`、`actual_cost`、`currency` | 记录存在 | undo 恢复旧成本 |
| `update_progress` | UpdateProgressOperation | `record_id`、`percent_complete`（0–100）、`note` | 记录存在 | undo 恢复旧百分比 |
| `create_work_order` | CreateWorkOrderOperation | `asset_object_id`、`title`、`description` | 资产对象存在 | 创建工作单记录（运维阶段）；undo 删除 |
| `update_work_order` | UpdateWorkOrderOperation | `work_order_id`、`status`、`description`（字符串） | — | undo 恢复旧 status/description |

`LifecyclePhase`：unspecified 0、planning 1、design 2、procurement 3、construction 4、
commissioning_acceptance 5、asset 6、operation_maintenance 7、production 8、decommissioning 9。
`RecordStatus`：unspecified 0、draft 1、open 2、in_progress 3、blocked 4、completed 5、
approved 6、rejected 7、closed 8。

### 3.10 暂无对应操作的动作族

若干动作码尚无 `Operation` 变体；它们由执行器层预留，或由其他服务处理：

- `IMPORT_FILE` (60)、`EXPORT_FILE` (61)、`PARSE_TO_OMCOM` (62) —— 导入 / 导出管线，
  另见 API 参考。
- `MODIFY_GEOMETRY` (11)、`DELETE_GEOMETRY` (12) —— proto 中注明预留的
  `ModifyGeometryOperation` / `DeleteGeometryOperation`。
- `BATCH_DELETE` (41)、`BATCH_TRANSFORM` (42) —— 预留。
- `ADD_COMPONENT_FEATURE` (203) —— proto 中注明预留的消息。
- `INGEST_SENSOR_DATA` (82)、`VALIDATE` (90)、`LIST_CATALOG` (205)、`LIST_SERIES` (215) ——
  读 / 杂项动作，由各自层处理。

---

## 4. 附录 A — Action 枚举

`proto/omcom.proto` 中 `enum Action` 的全部动作，按 wire 顺序：

| 代码 | 名称 | 所属族 |
|---|---|---|
| 0 | `ACTION_UNSPECIFIED` | — |
| 1 | `CREATE_OBJECT` | 对象 |
| 2 | `MODIFY_OBJECT` | 对象 |
| 3 | `DELETE_OBJECT` | 对象 |
| 4 | `COPY_OBJECT` | 对象 |
| 5 | `MOVE_OBJECT` | 对象 |
| 10 | `CREATE_GEOMETRY` | 几何 |
| 11 | `MODIFY_GEOMETRY` | 几何（预留） |
| 12 | `DELETE_GEOMETRY` | 几何（预留） |
| 13 | `TRANSFORM_GEOMETRY` | 几何 |
| 14 | `BOOLEAN_OPERATION` | 几何 |
| 20 | `SET_PROPERTY` | 属性 |
| 21 | `DELETE_PROPERTY` | 属性 |
| 22 | `ADD_PROPERTY_SET` | 属性 |
| 23 | `REMOVE_PROPERTY_SET` | 属性 |
| 30 | `ADD_CONTAINMENT` | 关系 |
| 31 | `REMOVE_CONTAINMENT` | 关系 |
| 32 | `ADD_CONNECTION` | 关系 |
| 33 | `REMOVE_CONNECTION` | 关系 |
| 40 | `ARRAY` | 批量 |
| 41 | `BATCH_DELETE` | 批量（预留） |
| 42 | `BATCH_TRANSFORM` | 批量（预留） |
| 50 | `SET_MATERIAL` | 材料 |
| 60 | `IMPORT_FILE` | I/O |
| 61 | `EXPORT_FILE` | I/O |
| 62 | `PARSE_TO_OMCOM` | I/O |
| 70 | `CREATE_LIFECYCLE_RECORD` | 生命周期 |
| 71 | `MODIFY_LIFECYCLE_RECORD` | 生命周期 |
| 72 | `DELETE_LIFECYCLE_RECORD` | 生命周期 |
| 73 | `LINK_RECORD_TO_OBJECT` | 生命周期 |
| 75 | `UPDATE_SCHEDULE` | 生命周期 |
| 76 | `UPDATE_COST` | 生命周期 |
| 77 | `UPDATE_PROGRESS` | 生命周期 |
| 80 | `CREATE_WORK_ORDER` | 生命周期 |
| 81 | `UPDATE_WORK_ORDER` | 生命周期 |
| 82 | `INGEST_SENSOR_DATA` | 生命周期 |
| 90 | `VALIDATE` | 校验 |
| 100 | `CREATE_SKETCH` | 草图 |
| 101 | `ADD_SKETCH_ENTITY` | 草图 |
| 102 | `ADD_SKETCH_CONSTRAINT` | 草图 |
| 103 | `SOLVE_SKETCH` | 草图 |
| 104 | `DELETE_SKETCH_ENTITY` | 草图 |
| 105 | `REMOVE_SKETCH_CONSTRAINT` | 草图 |
| 106 | `DELETE_SKETCH` | 草图 |
| 107 | `EXTRUDE_SKETCH` | 草图 → 体 |
| 108 | `REVOLVE_SKETCH` | 草图 → 体 |
| 110 | `CREATE_COMPONENT` | 组件 |
| 111 | `ADD_FEATURE` | 组件 |
| 112 | `REMOVE_FEATURE` | 组件 |
| 113 | `REBUILD_FEATURES` | 组件 |
| 200 | `DEFINE_COMPONENT` | 目录 |
| 201 | `ADD_COMPONENT_PARAM` | 目录 |
| 202 | `ADD_CONNECTOR` | 目录 |
| 203 | `ADD_COMPONENT_FEATURE` | 目录（预留） |
| 204 | `PUBLISH_COMPONENT` | 目录 |
| 205 | `LIST_CATALOG` | 目录 |
| 206 | `DELETE_COMPONENT_DEF` | 目录 |
| 210 | `CREATE_SERIES` | 系列 |
| 211 | `ADD_SELECTOR` | 系列 |
| 212 | `ADD_ANSWER` | 系列 |
| 213 | `ADD_SPEC_COMPONENT` | 系列 |
| 214 | `ADD_PARAM_CONSTRAINT` | 系列 |
| 215 | `LIST_SERIES` | 系列 |
| 216 | `DELETE_SERIES` | 系列 |
| 217 | `LOAD_SERIES` | 系列 |

---

## 5. 命令查阅惯例

OmniCommand 是 JSON。规范格式是 JSON 本身；不存在文本 DSL。手工书写命令时需要记住两点：

- **枚举就是整数**；枚举按其整数代码序列化。
- **ID 是 UUID**，由执行器自行生成。您不可能在执行前预知自动生成的 ID；若要引用某个
  现存对象，必须先查询项目以获得该对象的真实 ID，或先创建再取回其 `created_ids`。

### POST /api/omcom —— 创建墙

```json
{
  "version": "1.0",
  "id": "cmd-20261027-0001",
  "action": 1,
  "actor": { "id": "user-alice", "type": 1 },
  "target": null,
  "operations": [
    {
      "createObject": {
        "type": 100,
        "name": "Wall A",
        "description": "一层外墙",
        "geometry": {
          "operations": [
            { "box": { "length": 6000.0, "width": 200.0, "height": 3000.0 } }
          ]
        },
        "properties": [
          { "name": "FireRating", "value": { "stringValue": "2h" } }
        ]
      }
    }
  ],
  "metadata": { "source": "console" }
}
```

### POST /api/omcom —— 设置材料

```json
{
  "version": "1.0",
  "id": "cmd-20261027-0002",
  "action": 50,
  "actor": { "id": "user-alice", "type": 1 },
  "operations": [
    { "setMaterial": { "object_id": "<wall-a-uuid>", "material_id": "<material-uuid>" } }
  ]
}
```

（`material_id` 必须引用项目材料库中已定义的材料 —— Validator 会拒绝未知材料。
字段名按 proto 的 snake_case 序列化，即控制台实际发出的结构。）

### POST /api/omcom —— 拉伸草图

```json
{
  "version": "1.0",
  "id": "cmd-20261027-0003",
  "action": 107,
  "actor": { "id": "agent-7", "type": 2 },
  "operations": [
    { "extrudeSketch": {
        "sketch_id": "<sketch-uuid>",
        "depth": 400.0,
        "direction": { "x": 0.0, "y": 0.0, "z": 1.0 }
    } }
  ]
}
```

成功响应即 `ExecutionResult`：

```json
{ "success": true, "message": "…", "created_ids": ["<新对象uuid>"] }
```

失败响应 `success: false`，`message` 以 `"Validation failed: …"` 开头。

---

*事实来源：`proto/omcom.proto`、`crates/omcom/src/lib.rs`。若有出入，以代码为准 ——
请开 issue 而不是盲信本文档。*
