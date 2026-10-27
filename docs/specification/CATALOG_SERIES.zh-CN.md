# 元件库与系列库规范

> 状态：与 Phase 7 代码库同步。内容提取自 [DATA_MODEL.zh-CN.md](DATA_MODEL.zh-CN.md) 中
> 与元件目录相关的部分，并对照代码核实：`crates/model/src/catalog.rs`、`series_store.rs`、
> `seed_catalog.rs`、`seed_materials.rs`、Web 端 Series Manager。

## 1. 在统一模型中的定位

OmniM 把「一个东西**可以是什么**」（元件库）与「本项目**允许使用什么**」（系列库）
分开。这对应 E3D/PDMS 中 CATALOGUE 模块与项目规格（SPECON）的分离，使设计纪律通过
数据而非约定来强制执行。

- **元件库（Component Catalog）** —— 参数化三维构件定义（`ComponentDef`），全局存放于
  `Library.catalog`（`CatalogStore`），跨项目共享。
- **系列库（Series Library / 等级库）** —— 规格决策树（`Series`），挂接在项目的
  `SeriesStore` 上，在特定约束下确认可用的元件。

两库均由 `crates/model/src/seed_catalog.rs` / `seed_materials.rs` 预置：新会话即拥有
可用的钢结构、管道、混凝土、建筑、市政、设备、钢筋构件族，以及默认材质库。

```
层 4    项目对象（墙、管道、柱……）
            │ 实例化（CatalogRef 溯源）
层 3    ┌── SeriesStore（项目级）─── 决策树 ──┐
        │            │ 确认选用               │
        └── Library（全局）─ CatalogStore ─────┘
            ComponentDef + 特征树模板 + 材质
```

## 2. 元件库（Component Catalog）

### 2.1 ComponentDef

定义于 `crates/model/src/catalog.rs`（`ComponentDef`）；proto 对应 `omcom.proto` /
`omnim.proto` 中的 `DefineComponentOperation` / `ComponentDef`。

| 字段 | 含义 |
|------|------|
| `id` | UUID；同时以稳定的人类可读**名称**引用（如 `Steel-Beam-IPE`、`IPE240`、`Pipe-CS`） |
| `name`、`category`、`description` | 身份信息；`category` 是斜线分层路径（`"Structural/Steel/Beam"`、`"Piping/Pipe"`），`list_by_category` 支持按类目或其子路径前缀匹配 |
| `params: Vec<ParamDef>` | 类型化参数输入（§2.2） |
| `feature_tree_id` | 指向 `FeatureTreeRegistry` 中 `FeatureTreeTemplate` 的链接——生成真实三维几何的配方 |
| `connectors: Vec<ConnectorDef>` | 连接点（§2.3） |
| `allowed_materials`、`default_material` | 材质门禁（引用共享材质库，如 `S355`、`C30/37`） |
| `ifc_class` | IFC 导出互通类（如 `IfcBeam`、`IfcColumn`） |
| `standard_reference` | 规范依据（如 `EN 10365`、`ASME B31.3`） |
| `tags`、`domain`、`properties` | 检索与元数据 |
| `published` | 只有已发布的构件才会进入 `list_published` 供设计使用 |

### 2.2 ParamDef（参数化定义）

每个参数有类型（proto `ParamType`），并携带：

- `default_value`、`min_value`、`max_value`、`step` —— UI 滑块与校验的数值边界；
- `unit`（如 `mm`）；
- `formula` —— 在参数化上下文中求值的表达式字符串（连接点与派生几何经此引用
  参数，例如位置 `0,0,length`）;
- `choices` —— 枚举取值（选择型参数）；
- `description`。

### 2.3 ConnectorDef（P 点兼容）

`ConnectorDef` 声明一个连接点；所有空间字段都是**表达式字符串**，可引用构件参数：

- `position_x/y/z_expr` —— 如 `"0"`、`"length"`（由最终参数值解析）；
- `direction_x/y/z_expr` —— 外向轴线，默认 `0,0,1`；
- `bore_expr` —— 通径表达式（管道）；
- `connection_type_code` —— 用于耦合检查的连接类型码（见下）；
- `role` —— proto `ConnectorRole`：`INLET`(1)/`OUTLET`(2)/`BIDIRECTIONAL`(3)/
  `MOUNT`(4)/`BRANCH`(5)。

`catalog.rs` 中的 `ConnectorType` 与 `connector_code_map()` 提供跨平台分类映射：

| OmniM 码 | ConnectorType | Civil3D 码 | E3D P 点 |
|----------|---------------|------------|----------|
| `PIPE-END` | PipeEnd | `PIPE-END-C`/`PIPE-END-S` | `P-END` |
| `FLANGE` | FlangeFace | `FLG-FACE` | `P-FLAN` |
| `ELBOW` | ElbowEnd | `ELB-END` | `P-ELB` |
| `TEE-RUN` / `TEE-BRANCH` | TeeRun / TeeBranch | `TEE-RUN`/`TEE-BR` | `P-TEE`/`P-TBR` |
| `STRUCT-END` / `STRUCT-BASE` | StructuralEnd / StructuralBase | `BEM-END`/`COL-BASE` | `P-STRU`/`P-BASE` |
| `NOZZLE` | EquipmentNozzle | `NOZ` | `P-NOZ` |
| `CABLE` | CableEnd | `CBL-END` | `P-CBL` |
| `DUCT` | DuctEnd | — | — |
| `VALVE-END` / `VALVE-STEM` | ValveEnd / ValveStem | `VALVE-END` | `P-VALVE`/`P-VSTEM` |

对象之间的连接通常只在连接**类型码**匹配时建立——从而围绕同一个 P 点概念实现与
Civil3D / E3D 风格的互通。

### 2.4 发布与实例化流程

1. **定义** —— `DEFINE_COMPONENT`：创建带名称/类目身份信息的 `ComponentDef`。
2. **参数化** —— `ADD_COMPONENT_PARAM`：追加类型化 `ParamDef`
   （内存构建器 API 为 `with_default/with_range/with_step/with_unit/with_choices`）。
3. **添加连接点** —— `ADD_CONNECTOR`：追加以表达式驱动的 `ConnectorDef` P 点。
4. **挂接特征树** —— 经 `seed_feature_trees`/`FeatureTreeRegistry`：构件名称链接到
   `FeatureTreeTemplate`（如 `Steel-Beam-IPE` → IPE 模板），以有序的
   `FeatureTemplateStep` 创建草图与特征。
5. **发布** —— `PUBLISH_COMPONENT` 将 `published` 置为 true；之后设计师才能使用。
6. **实例化** —— 设计动作经构件的特征树创建项目 `OmniObject`，并记录 `CatalogRef`：

```rust
pub struct CatalogRef {
    pub component_def_id: String,
    pub series_id: String,          // 确认选用该构件的系列（如有）
    pub selector_path: Vec<String>, // 在系列决策树中的路径
    pub param_values: HashMap<String, f64>, // 解析后的实例参数
}
```

   保留溯源（由哪个系列确认、走了哪条分支、用了哪些值），并在定义或系列变更时支持
   再生（regeneration）。

### 2.5 内置库（种子数据）

`seed_catalog.rs` 预置了（节选）：

- 钢结构：`Steel-Beam-IPE` + `IPE240`，柱 `Steel-Column-HEA`、`Steel-Column-HEB`、
  `Steel-Column-HEM`，槽钢 `Steel-Channel-UPE/UPN`、T 型截面 `Steel-T-Section`、
  空心截面 `Steel-CHS`/`Steel-SHS`、角钢族。
- 管道：`Pipe-CS`（碳钢管）、`Elbow-LR-CS` 等，带 DN 范围与通径公式。
- 混凝土：`Concrete-Beam-Rect`、`Concrete-Column-Rect` / `Concrete-Column-Circular`、
  `Concrete-Footing-Isolated/Raft/Strip`。
- 建筑：标准墙、门、窗（`Concrete-Wall`、`Door-Single`、`Window-Fixed` 等）。
- 市政道路、设备桩、钢筋构件与领域扩展构件。
- 材质经 `seed_materials.rs`：稳定的人类可读 ID（`C30/37`、`S355`、`S235`、`XPS-100`
  等），被 `allowed_materials` / `default_material` 引用。
- 系列（§3）：`CS150`、`CS300`、`Steel-Q355`、`Concrete-C30`、`SS300`，以及 GB 标准
  公路系列（`Highway-GB`）。

## 3. 系列库（SPECON）

### 3.1 Series 结构

定义于 `crates/model/src/series_store.rs`（对应 AVEVA SPECON）：

```rust
pub struct Series {
    pub id, name, description,
    pub domain, standard_reference,
    pub root_selector: Option<Selector>,  // 决策树根
    pub applicable_phases: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub source: String,  // "project" | "external" | "seed"
}
```

### 3.2 决策树语义：Question → Answer → Selector → SpecComponent

- `Selector { id, question, answers }` —— 一个提问节点；answers 是其分支。
- `Answer { id, value, is_numeric_range, range_min, range_max, outcome }` ——
  或按固定字符串匹配（大小写不敏感），或按数值区间匹配
  （`min <= v <= max`）；`outcome` 为 `AnswerOutcome::NextSelector(Selector)` 或
  `AnswerOutcome::Component(SpecComponent)`。
- `SpecComponent { id, component_def_id, param_constraints, material,
  connection_type_code, design_params }` —— 叶节点：经系列确认的目录构件，附系列级
  参数限制与默认材质/连接码。
- `ParamConstraint { param_name, default_value, min_value, max_value, step, fixed_value }`
  —— 收窄构件 `ParamDef` 的范围；可用 `fixed_value` 钉住某个参数（字符串/表达式
  形式，优先于开放范围）。

`SelectionEngine::select(series, answers)` 遍历决策树：

1. 从 `root_selector` 开始，调用方为每层 selector 提供一个答案字符串。
2. 匹配：数值区间答案把输入解析为 f64 并测试区间；字符串答案大小写不敏感比较。
3. 命中 `NextSelector` 则带下一个答案下探；命中 `Component` 则返回
   `SelectionResult { spec_component, selector_path, component_def_id }`。
4. 答案缺失或未匹配会返回携带失败 selector/question 的描述性错误。

直接选取之外，`all_reachable_components()` 遍历全部树，收集所有已加载系列可达的
def-id——这就是规格门禁（§3.4）的实现基础。

### 3.3 存储、解析优先级与 Web 端 Series Manager

- `SeriesStore` 以 UUID 存放系列；`find_by_name` 支持人类可读系列名（如 `CS150`）。
- **来源优先级**：同 ID 插入时，更高优先级来源胜出 —— `project(3) > external(2) >
  seed(1)`。因此种子系列可被项目级覆盖；种子不能覆盖项目系列。
- 文件：`save_to_file`（pretty JSON）与 `load_from_file`（接受 JSONC/JSON5，加载的
  系列标记 `source: "external"`）；扩展名 `.omnimseries`。
- Web 端 **Series Manager**（`web/src/series/SeriesManagerPage.tsx`）编辑决策树
  （selector、answer、spec component、constraint），并经库端点加载/保存
  `.omnimseries` 文件。

### 3.4 规格强制规则（设计者限定于激活系列）

`crates/model/src/project.rs::check_series_gate` 执行经典的 PDMS/SPECON 规则 ——
**「系列激活后，只允许选择经系列确认的构件」**：

- **已加载系列且可达**（从任一已加载系列的决策树可达该 def-id）→ 允许。
- **已加载系列但不可达** → 拒绝，错误形如
  `series gate: component '…' is not reachable from any loaded series. Load the appropriate
  series or use a series-approved component.`
- **未加载任何系列** → 开放模式，全部目录构件可用。

可达性同时按构件 UUID 与名称检查（SpecComponent 可能存任意一个；大小写不敏感作为
兜底匹配）。

### 3.5 系列库相关的 Omcom 操作

系列编写与一切变更一样经 Omcom 管线执行（见
[OMCOM_SPEC.zh-CN.md](OMCOM_SPEC.zh-CN.md) §2.4）：`CREATE_SERIES`、`ADD_SELECTOR`、
`ADD_ANSWER`、`ADD_SPEC_COMPONENT`、`ADD_PARAM_CONSTRAINT`、`DELETE_SERIES`、
`LOAD_SERIES`（`path` + `source` = `"standalone" | "library"`）与 `LIST_SERIES`。
Validator 会检查系列存在性（`CREATE_SERIES` 除外）与 `LoadSeries` 路径非空；每条已
执行操作都会写入可撤销的反向 `Command`。

## 4. 实例演练

### 4.1 钢梁系列选取（`Steel-Q355` 种子系列）

内置种子（节选）：

```text
Series Steel-Q355 (domain structural, EN 1993)
  root selector q1: "element_type?"
    answer "beam"  → next selector q2_material
      answer "S355" → SpecComponent Steel-Beam-IPE, material S355
    answer "column" → next selector q3_section
      answer "HEA"  → SpecComponent Steel-Column-HEA
      answer "HEB"  → SpecComponent Steel-Column-HEB
```

`SelectionEngine::select(series, &["beam", "S355"])` → `Steel-Beam-IPE`，
selector path `[q1, q2_material]`，确认材质 `S355`。

### 4.2 管道系列 `CS150`

```text
Series CS150 (domain plant, ASME B31.3)
  root selector gt: "GT?"                                       (总类型)
    answer "PIPE"    → next selector sch: "schedule?"
       answer "STD"  → SpecComponent Pipe-CS, material A106-B,
                       约束: DN min 15, max 600, step …
    answer "ELBOW"  → SpecComponent Elbow-LR-CS, material A106-B
```

实例化：答案 `["PIPE", "STD"]` 选中 `Pipe-CS`；执行器创建一个 OmniObject，解析出
DN（如 DN 100），记录 `CatalogRef { component_def_id, series_id: CS150,
selector_path: [gt, sch], param_values { dn: 100.0, length: 6000.0 } }`，并把包含关系
/属性操作纳入同一条 OmniCommand。在 CS150 已加载的情况下，尝试实例化该系列不可达的
构件（如梁）会被系列门禁**拒绝**。

### 4.3 定义一个新目录构件

```text
DEFINE_COMPONENT    { name: "Pipe-CS", category: "Piping/Pipe", ifc_class: "IfcPipe",
                      standard_reference: "ASME B36.10", domain: "plant" }
ADD_COMPONENT_PARAM { DN   min 15   max 600  default 100 mm }
ADD_COMPONENT_PARAM { schedule … choices ["STD", "XS"] }
ADD_CONNECTOR       { id "start", role INLET,  pos "0,0,0",     dir "0,0,1", bore "dn" }
ADD_CONNECTOR       { id "end",   role OUTLET, pos "0,0,length", dir "0,0,1", bore "dn" }
PUBLISH_COMPONENT   → Pipe-CS 上线可用
```

## 5. 待决问题

- `connector_code_map` 是否应改为数据驱动（配置文件）？
- 系列版本化（标准修订历史）—— 当前按 ID 覆盖。
- 完整的组合树连接器（支管 tee-run/branch 配对）—— 当前仅有分类层。
- 删除被系列引用的 ComponentDef 时：级联删除还是拒绝。
