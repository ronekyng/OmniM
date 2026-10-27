# RFC-0001：Omcom 文本 DSL

- **状态**：Draft（草案）
- **关联文档**：[Omcom 规范](../specification/OMCOM_SPEC.zh-CN.md) · [数据模型](../specification/DATA_MODEL.zh-CN.md)

## 1. 背景

本 RFC 恢复并正式化 Omcom 的**原始文本 DSL 愿景**——这一愿景曾被错误地记录为已交付
能力，现在作为设计提案呈报。

今日实际存在的能力（参见 Omcom 规范）：

- Omcom 是类型化命令体系：`proto/omcom.proto` 定义 `OmniCommand` 信封与操作消息，
  经 prost 编译为支持 serde 的 Rust 类型。
- `crates/omcom` 是 Validator + Executor，支持撤销/重做与事务。
- Web 端 Omcom Console 接受 JSON。
- AMIR 输出类型化的 `OmniCommand` 信封，而非文本。

原始愿景以完整的管线形式表述为：

```
自然语言 → AMIR（意图） → Omcom 文本 DSL → nom 解析器 → Executor → 模型
                                                  ↑ 逆向：
                                        模型 → 文本（双向解析）
```

……并在 UI 中设置**命令窗口入口**，工程师可直接键入命令；同时配备**逆向解析器**，
把模型状态渲染回 Omcom 文本（"model → text"），用于 diff、评审与版本控制。
类型化命令体系已覆盖管线的后半段（Executor → 模型）；本 RFC 提议的是前半段：
在同一套操作之上的确定性文本语法。

## 2. 动机

1. **键盘优先的工程师。** 惯用建模工具（AutoCAD、PDMS/PML、Specon 工作流工具）十分
   依赖命令行熟练度。对日常编辑而言，命令窗口比对话框导航更快。
2. **脚本化与 CI。** 文本格式天然可 diff、可版本化，可以直接作为 `.omcom` 脚本文件
   执行——JSON 信封对此过于冗长。
3. **AMIR 可读性。** 若 AMIR 输出文本，人可以在命令窗口中在执行前评审/修改 AI 的输出；
   今天 AI 输出是不透明的 JSON。
4. **双向往返（bidirectional parse）。** 模型 → 文本渲染可支持：可评审的项目
   "changelog"、指令级版本控制、以及从命令日志重现模型。
5. **审计友好。** 单行命令即是一条紧凑、人类可读的审计记录。

## 3. 提案草稿

### 3.1 设计约束

- 文本 DSL 是- **确定性**前端：文本 → 类型化 `OmniCommand` → Validator → Executor，执行阶段
  不做任何"自然语言理解"。
- **双向无损**：任何可执行文本在 文本 ↔ 信封 往返后语义不变；任何模型状态都必须可
  渲染为文本命令日志，重新执行后得到语义相同的模型。
- 语法增量式演进；DSL 本身不修改 schema（`omcom.proto`）。

### 3.2 通用语法（语法草图 —— 示意）

```
instruction      = verb, { word }, [ flags ] ;
verb             = "create" | "modify" | "delete" | "copy" | "move"
                 | "sketch" | "constraint" | "solve" | "feature" | "component"
                 | "prop" | "material" | "parent" | "connect"
                 | "lifecycle" | "schedule" | "cost" | "progress"
                 | "catalog" | "series" | "undo" | "redo" ;

flags            = "--", name, "=", value, { [ " ", "--", name, "=", value ] } ;
value            = point2d | point3d | number | identifier | string ;
point2d          = number, ",", number ;
point3d          = number, ",", number, ",", number ;
number           = ["-"], digit, { digit }, [ ".", digit, { digit } ] ;
identifier       = (letter|"_"), { letter|digit|"_"|"-" } ;
```

每条指令 1:1（或在 OMCOM_SPEC §2.4 记录的"实例化分解"场景下 1:n）映射到类型化
操作消息。

### 3.3 示例

> 以下示例均为**示意**，不是已承诺的语法。最终表层语法取决于待决问题中
> "人工可输入性 / LLM 可解析性"的取舍。

#### 创建对象

```text
create wall std_wall --geom box 5.0,0.3,3.0 --pos 0,0,1.5
create beam main_beam --component Steel-Beam-IPE --param length=6000mm
create pipe steam_p1 --series CS150 --answers DN=100,schedule=STD
```

#### 草图

```text
sketch new wall_s1 --plane 0,0,0 0,0,1
sketch add wall_s1 line 0,0 10,0
sketch add wall_s1 circle 5.0,1.8 0.5
sketch constr wall_s1 horizontal 1
sketch constr wall_s1 length 1 10.0
sketch solve wall_s1
```

#### 特征拉伸

```text
feature add wall_s1 extrude --depth 3.0 --dir 0,0,1
```

或单动词的等价形式（映射到 `ExtrudeSketchOperation`）：

```text
extrude wall_s1 --depth 3.0 --dir 0,0,1
```

#### 特征树

```text
component create beam_001
feature add beam_001 extrude wall_s1 --depth 6.0
feature add beam_001 revolve shaft_s1 --axis 0,0,0 0,0,1 --angle 360
feature remove beam_001 feat_2
feature rebuild beam_001
```

#### 元件库

```text
catalog define IPE240 --category Structural/Steel/Beam --ifc IfcBeam
catalog param IPE240 height --type length --default 240 --range 80,600 --unit mm
catalog connector IPE240 start --pos 0,0,0 --dir 0,0,1 --role inlet
catalog connector IPE240 end --pos 0,0,length --dir 0,0,1 --role outlet
catalog publish IPE240
catalog list --category Structural/Steel
```

#### 系列库（等级库）

```text
series create CS150 --domain plant --standard "ASME B31.3"
series selector CS150 q1 --question "element_type?"
series answer CS150 q1 PIPE
series component CS150 q1 PIPE --comp Pipe-CS --material A106-B
series constraint CS150 Pipe-CS --param DN --range 15,600
series list
```

#### AMIR 集成（最终形态）

```
"在原点创建一面 10m 长、3m 高的墙"
        ↓ AMIR
create wall wall_1 --geom box 10.0,0.2,3.0 --pos 5.0,0,1.5
        ↓ 文本解析器（nom） → OmniCommand → Validator → Executor → 模型
```

### 3.4 逆向解析（模型 → 文本）

对项目中的每个对象 / 草图 / 特征 / 关系执行 `to_text`，按依赖图顺序渲染
（create → modify → connect）成最小充分命令序列。对无法参数化的任意导入网格，
降级为在其支持的原语上输出显式几何动词；无法参数化的网格在文本日志中以警告
说明。

## 4. 风险与待决问题

- **语法演进速度的权衡** —— 能快速演进的文本查询前端，会在两个方向
  （解析 → 表层，表层 → 解析）上消耗更多设计迭代：文本表层必须在不破坏 schema 的
  前提下扩展，但每个新增动词都要双向覆盖。
- **逆向解析的真值保真** —— 模型→文本能多忠实地保留可校验语义（特征顺序、参数
  出处、系列门禁）？有损的逆向输出会破坏往返不变量。
- **AMIR 耦合** —— 若 AMIR 输出文本，错误率在两端叠加（意图 → 文本受 LLM 的
  随机性影响；文本 → 信封虽是确定性的，但错误在编译时集中暴露）。是否应保留 AMIR
  输出类型化命令（现状），把文本层留给人工输入？
- **范围**：命令窗口应暴露完整 CRUD + 生命周期 + 元件/系列操作，还是先收窄到建模
  动词？
- **DSL 中的撤销/重做**：文本中的 `undo` / `redo` 应是一等动词，还是维持执行器 API
  （今天通过 `make_undo_redo_command` 哨兵命令）？
- **解析器放哪里**：新 crate（`crates/omcom-text-parser`），还是 `crates/omcom`
  内以 feature flag 开关？
- **语法的版本化**：文本语法版本如何与 `OmniCommand.version` 的语义对齐。
