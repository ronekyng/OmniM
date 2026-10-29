---
title: "OmniM 建模者手册"
status: current
last_updated: 2026-10-27
applies_to: "web/, crates/server, crates/model, crates/ifc"
evidence: "任务流程已对照 web/src/app/components/{MainToolbar,create,viewer}、docs/reference/API.zh-CN.md 与 docs/guides/GETTING_STARTED.zh-CN.md 核实，2026-10-27"
---
# OmniM 建模者手册
一份面向当前 Web 应用与 REST API 的任务式指南。下面的每条流程**今天都可执行**；缺口在出现处直接说明，而不是隐藏。逐对象能力见 [CAPABILITY_MATRIX](CAPABILITY_MATRIX.zh-CN.md)。

## 1. 开始之前
### 1.1 单位，以及为什么有两套
- **视口、拾取点、GenRule 与变换使用米。** 从 (0,0,0) 拾取到 (5,0,0) 的墙长 5 m。`POST /api/genrule` 的 `length`、`width`、`height`、`radius`、`alignment`、`boundary` 等字段都是米。
- **目录 `ComponentDef` 参数使用毫米。** 墙定义的 `length` 是 `5000`，不是 `5`。目录网格在显示时按 `1/1000` 缩放（`crates/server/src/lib.rs` 的 `get_mesh`、`preview_catalog_component`）；存储模型仍是 mm。
- **图层编辑器使用毫米。** `LayerEditor` 里的 `200` 表示 200 mm；`MainToolbar.tsx` 在提交 `layered` GenRule 前除以 1000，而 GenRule 的层厚是米。
- **目录放置的 `position` 是米**，即使它的 `params` 是 mm（服务端构建实例时会换算成 mm）；**Assembly GenRule 的尺寸是米**（`/api/genrule` 的 `type:"assembly"`），与目录参数不同。

经验规则：输入到目录参数面板的数字想成**毫米**；在视口拾取或调用 `/api/genrule` 时想成**米**。

### 1.2 精度与几何注意事项
- `PUT /api/objects/{id}/transform` 会把任何几何转换为 **Mesh** 后重写。凡有几何的对象都能动，但它会压平特征树：移动后，参数化特征与目录参数不再驱动形状。它**可撤销**（恢复变换前几何）。
- 墙 `path` 可以带不同 z 值，但平面拉伸使用**首点 z**作为基底标高；没有逐点放坡。楼板/屋顶/楼面的边界也使用首个边界点的 z。
- 转角在核心里做 miter，预览与之一致；这是平面拉伸，不是完整三维扫掠。
- 楼板/楼面/屋顶边界由服务端校验：少于 3 点、共线或零长度边、退化面积、蝴蝶结自交都会被拒绝。结构有效性、重叠与洞口不校验。
- 桥梁与隧道创建仍只使用**首个和最后一个拾取点**；`road` 是唯一消费完整 alignment 的土木类型。
- `POST /api/objects` 接受字符串 `"floor"`，但会映射为 `BuildingStorey`（`crates/server/src/lib.rs` 的 `parse_type`）。要创建真正的楼面，请用 `/api/genrule` 的 `mode:"floor"`。

### 1.3 持久化：建模前必读
> ✅ **持久化默认开启**（2026-10-28）。服务器在 executor 组变更后自动写 `$OMNIM_PROJECT`（默认 `project.omnim`）；`POST /api/project/save` 可随时强制保存；`OMNIM_EPHEMERAL=1` 关闭持久化。

- **会保存**：创建/删除对象、`/api/genrule`、`/api/omcom`、撤销/重做、sketch 变更、对象改名、特征增加/更新/删除/重建。
- **会保存**：其余全部端点——变换/改父级/连接、材质、测量、管理区/分解、系列、目录定义、目录放置与 IFC 导入。
- 更稳妥的单次会话办法：用 `GET /api/export/ifc`（或工具栏 **Export**）保留几何快照，但要接受 §6.2 的 IFC 限制。
- 撤销覆盖全部变更端点（对象、属性、变换、父子/连接、材质、草图、构件、目录、系列、测量、管理区/分解、实例化、IFC 导入）。见 [WRITE_PATHS](architecture/WRITE_PATHS.zh-CN.md)。

### 1.4 捕捉
- 视口工具栏有 **Toggle snap** 按钮（`IcoSnap`，提示 "Toggle snap"）。每次拾取与悬停端点都会经过 `snapPoint()`。
- 默认：启用，网格 + 顶点 + 端点开启，边与中点关闭，阈值 **0.5 m**，网格 **1.0 m**。需要不同阈值可改 `web/src/app/components/viewer/three/Snapping.ts`。
- 捕捉只是几何吸附：不会推断墙体、楼层或参数化关系。网格显示用 **G** 切换。

## 2. 建筑
### 2.1 创建墙体（默认 panel；可选 region）
1. 启动服务端与 Web 应用（[GETTING_STARTED](guides/GETTING_STARTED.zh-CN.md)）；打开 <http://localhost:5173>。
2. 点击 **Create**（默认墙），或 **建筑** → **墙体**。
3. 在创建面板选择墙形式：**中心线（面板）**（默认，折线选点）或 **封闭轮廓（块）**（封闭多边形选点）。拾取模式跟随形式。
4. 在视口点击至少两个点（panel）或三个轮廓点（region）。**Backspace** 删除上一点，**Esc** 取消，**Enter** 结束路径。
5. 用 **LayerEditor** 编辑层栈（厚度为 mm；模板含外墙/内墙/隔墙）。可选 **系列**。
6. 点击 **创建**。结果是类型为 `Wall` 的分层拉伸对象，并记录 `genesis_class`/`genesis_form` 属性。
7. API 等价写法：`POST /api/genrule`，`{"type":"layered","mode":"wall","height":3,"layers":[...],"path":{"points":[[x,y,z],…],"closed":false}}`（遗留裸 `path[]` 仍可用）。见 [API §GenRule](reference/API.zh-CN.md#genrule)。

你**不会**得到：WallType/复合类型对象、墙连接、宿主传播或作为对象的洞口。层栈还会以 `layers` 属性保存以便往返，但它不是可复用类型。

### 2.2 楼板、楼面与屋顶（Path 闭合，区域形式）
1. **楼板**：**建筑** → **楼板**。点击 3 个以上边界点；回到首点或双击/回车闭合。
2. **屋顶**：**建筑** → **屋顶**，然后在屋顶模式中选择 **轮廓（多边形）**（足迹，≥3 点）或 **坡屋面（段）**（两点，剖面拉伸）。
3. **楼面**：**菜单没有入口**。请用 `POST /api/genrule` 的 `mode:"floor"` 与 `boundary` 数组，或自行补一个 UI 入口。不要用 `POST /api/objects` 传 `"floor"`（见 §1.2）。
4. 在 **LayerEditor** 设置层栈（mm），再点 **创建**。
5. API 等价写法：`{"type":"layered","mode":"slab|floor|roof","boundary":[[x,y,z],...],"layers":[...]}`。屋顶剖面模式则用两点 `path` 加 `width`。

三者仍是**区域形式**的面对象：闭合边界加层栈，不是扫掠实体，也没有结构分析、洞口或楼层关联。

### 2.3 门与窗（必须有宿主墙）
1. 先创建一面**墙**（§2.1）。没有墙时，门窗面板会显示“请先创建一面墙体”。
2. 点击 **Create**（或对象类型下拉）选择 **门** 或 **窗**。
3. 在下拉中选择目标墙（列表只包含树中类型为 `Wall` 的对象），设置 **洞口宽/洞口高**、**水平位**、**离地高/窗台高**。
4. 点击 **创建**。门窗会挂到墙上，洞口从墙的包围盒中布尔扣除。
5. API：`POST /api/genrule`，`{"type":"door","wall_id":"...","opening_width":1.0,"opening_height":2.1,"position_x":0,"position_z":0}`；未知 `wall_id` 返回 404。

限制：洞口是从墙的**包围盒**扣除，不是从分层实体扣除；没有洞口对象、框类型、窗台/过梁参数组、防火等级或明细表。建筑面板的 **门/窗** 快速按钮实例化的是目录 `Door-Single`/`Window-Fixed`，生成 **Component** 对象——那不是宿主门窗。

### 2.4 草图 → 特征 → BRep（官方精确路径）
1. 点击主工具栏 **+ Component**。这会创建构件并跳转到 `/component-editor?component=<id>`。
2. 在 **2D-Top** 中用 Line/Rectangle/Circle/Arc/Polyline 工具绘制轮廓。
3. 从约束工具栏添加约束；点击 **Solve**，检查 Solved/Unsolved 状态。
4. 点击特征带中的按钮（拉伸 Extrude、旋转 Revolve、扫掠 Sweep、放样 Loft、布尔 Boolean、倒角 Chamfer、圆角 Fillet、阵列 Pattern、抽壳 Shell）。若特征需要轮廓，画完后点 **✓ Done**。
5. 在对话框中设置深度/角度，再到 **3D** 标签查看结果。特征的增/改/删与 **Rebuild** 使用 truck BRep 内核。
6. API 等价写法：`POST /api/sketches`、`POST /api/sketches/{id}/entities`、`/constraints`、`/solve`，然后 `POST /api/components/{id}/features` 或 `/api/sketches/{id}/extrude`。

这条路径创建带特征树的真正 `Component` 对象，与 GenRule 工具栏路径不同。它的几何最可信，但对象类型仍是 `Component`，不是 `Wall`/`Beam`。

### 2.5 目录放置（创建 `Component` 对象）
1. 点击 **Catalog** 打开 **Available Components**（按系列过滤，带参数滑杆）。
2. 把定义拖进视口，或点击 **添加到模型**。会创建带 `catalog_ref` 的 `Component` 对象。
3. 目录面板参数是 **mm**；位置是 **m**。**系列** 选择器按领域收窄列表。
4. 未选系列时 `skip_series_gate` 为 true；选了系列，门禁也只检查可达性。

领域面板（**建筑/土木/工厂/机械**）还提供一键快速创建，绕过系列门禁并实例化目录构件。它们创建的是 `Component` 对象，不要指望得到语义化的 `Tank`/`Beam`/`Door` 对象。

### 2.6 建筑中尚不可用的能力
- **楼层的标高/层高**：`BuildingStorey` 只有名称，没有楼层/标高系统；没有按楼层放置或按楼层宿主。
- **轴网对象**：没有轴网/轴线对象类型或 UI，只有视口网格。
- **墙类型**：没有 `WallType`/`FloorType`/`RoofType` 库对象；层栈按实例编辑。
- **洞口对象**：门窗是对象，但扣洞是包围盒布尔；没有 `Opening` 元素、窗台/过梁约束或洞口明细。
- **楼梯**：没有楼梯生成器；按钮生成基本体占位。**吊顶/幕墙/坡道** 仅计划。**碰撞、工程量统计、图纸/明细表**：任何层都没有实现。

路线图与缺口记录：[ROADMAP Phase 8](status/ROADMAP.zh-CN.md)、[CODE_DEVIATIONS §3](status/CODE_DEVIATIONS.zh-CN.md)。

## 3. 土木
### 3.1 带路线的道路（已实现）
1. 点击 **土木** → **道路**。
2. 在平面中拾取路线点；**Enter** 完成。服务端沿折线逐段铺路，遵守 **断面宽** 与 **断面高**（米）。
3. 可选系列；对道路而言系列只是元数据/归属，不是走廊模板。
4. 点击 **创建**，或调用 `POST /api/genrule`，`{"type":"road","alignment":[[x,y,z],...],"width":7,"thickness":0.3}`。

道路是逐段拉伸。**没有纵断面、超高、走廊装配、地形目标、车道/路肩/路面层模型或桩号**。这些仍属 Phase 8（[ROADMAP](status/ROADMAP.zh-CN.md)）。

### 3.2 特征线与等高线
1. **土木** → **特征线** 或 **等高线**。
2. 拾取 N 点（Path 开放）；**Enter** 完成。
3. 服务端生成细导引管（固定 Ø0.05 m），不是 TIN 断裂线或等高程线。

这些类型是过渡性的：proto 有 501/502，工具栏可用，但原生点类/TIN 几何仍是设计目标（[DATA_MODEL §1](specification/DATA_MODEL.zh-CN.md#1-概述)）。

### 3.3 地形、土方及其余（计划中）
- **TerrainModel/PointCloud/GradingSurface/Slope** 没有 ObjectType、UI 或 API 创建路径；**挖方/填方/土方平衡** 没有实现。
- **路线/纵断面/桩号/车道/路肩/路面层/中央分隔带/路缘** 仅计划。
- **桥梁/隧道** 目前创建两点基本体占位；**挡土墙/坝/涵洞/桥墩/桥台** 仅 API 或仅计划。

见 [ROADMAP Phase 8](status/ROADMAP.zh-CN.md) 与 [CODE_DEVIATIONS §3.2](status/CODE_DEVIATIONS.zh-CN.md)。

## 4. 工厂
### 4.1 直管段与系列选择
1. 点击 **工厂** → **管道**，或 **Create** 后选择 **pipe**。
2. 拾取起点与终点（两次点击）。目录路径使用所选 `ComponentDef` 的 **mm** 参数；GenRule 路径使用 **m** 的 `radius`/`length`。
3. 可选 **系列**。选择器按领域过滤；服务端门禁只检查构件可从已加载系列到达。
4. 点击 **创建**。目录路径创建 `Component`；GenRule 路径创建带直圆柱网格的 `Pipe`。

限制：只有直管段——**没有弯头、三通、异径管、法兰、分支、支吊架或管道拓扑**。`POST /api/objects/{from}/connect/{to}` 会创建结构连接，但只检查两个对象存在；不校验连接点、口径、类型码或压力等级，也不写连接点。系列材质**不会应用**（`SpecComponent.material` 被忽略）。见 [CODE_DEVIATIONS §6.4](status/CODE_DEVIATIONS.zh-CN.md)。

### 4.2 设备放置
- **工厂** 快速创建提供立式储罐、离心泵、闸阀、换热器与压缩机；每个都把目录定义实例化为 **Component** 对象。
- 反应器、仪表、结构钢与管件在 proto 中存在，但**没有 UI**；需要占位时可通过 `/api/objects` 或 `/api/catalog/instantiate` 创建。

### 4.3 路由与连接（部分/计划）
- 管道**路由、分支、弯头、管道规格与轴测图**是 Phase 8 内容（[ROADMAP](status/ROADMAP.zh-CN.md)）。
- 连接仅 API 且不校验（§4.1）。没有 P&ID 或系统拓扑。
- `ConnectorDef` 只存方向；没有 up 轴，因此无法表达法兰螺栓孔或阀杆朝向（[CODE_DEVIATIONS §6.4](status/CODE_DEVIATIONS.zh-CN.md)）。

## 5. 机械
### 5.1 零件
1. **Create** → 对象类型 **part**，选择 Box/Cylinder/Sphere/Cone 与尺寸（米），点击 **创建**。这是基本体占位。
2. 真正的零件请走目录路径（§2.5）或构件编辑器（§2.4）：草图 → 约束 → 求解 → 特征 → BRep。
3. `POST /api/objects` 传 `"part"` 创建 `Part`；目录实例化创建 `Component`。请按后续用途选择路径。

### 5.2 装配
1. 点击主工具栏 **Assy** 打开 **AssemblyPanel**。
2. 添加 box/cylinder/sphere/cone 子件，设置尺寸与 X/Y/Z 偏移（米），点击 **创建 Assembly**。
3. API：`POST /api/genrule`，`{"type":"assembly","name":"...","components":[{"name":"...","shape":"box","p1":2,"p2":1,"p3":1,"tx":0,"ty":0,"tz":0}]}`。
4. 结果是带合并网格的 `Assembly` 对象，不是受约束的产品结构。需要特征树装配时，请在构件编辑器里构建 `Component`。

### 5.3 运动学与制造（计划中）
- **运动副、约束、轴、运动副对与执行器** 仅计划；没有配合求解器或运动仿真。**紧固件**（螺栓/螺母/垫圈/螺钉/铆钉/销）与制造对象类型（孔/型腔/槽/螺纹/PMI）仅计划。
- 构件特征树中已有通用**倒角/圆角特征**；这与计划中的制造对象模型不是一回事。

见 [ROADMAP Phase 8](status/ROADMAP.zh-CN.md) 与 [CODE_DEVIATIONS](status/CODE_DEVIATIONS.zh-CN.md)。

## 6. 数据与交换
### 6.1 保存与加载 `.omnim`
- 项目是启动时从 `OMNIM_PROJECT`（默认 `project.omnim`）加载的单个 JSON 文档。executor 组端点的变更会自动落盘；也存在显式保存路由 `POST /api/project/save`（Web 界面暂无保存按钮）。
- 只有 §1.3 列出的五个处理器会写文件，且 `save_path` 必须为 `Some`。目录放置、GenRule、IFC 导入与系列编辑都只在内存中。
- 会话习惯：假设 `OMNIM_PROJECT=./dev-project.omnim`，用对象创建等 executor 组动作即可自动落盘；遗留直写动作（变换、材质、系列编辑等）仍需用 `POST /api/project/save` 手动补存。关闭前检查文件时间戳。
- 撤销不能跨重启保护你：只有执行器路径上的操作在撤销栈中。

### 6.2 IFC 导入与导出
- 工具栏 **Import** / **Export** 对应 `POST /api/import/ifc` 与 `GET /api/export/ifc`。
- 导入接收**原始 IFC 文本**（不是 multipart）。查询参数：`normalize`（默认 true）与 `mode`（`merge` 默认 / `replace`）。解析失败返回 HTTP 200 且 `success:false`。
- 输入仅支持 **IFC 2x3 / IFC 4 子集**，手写解析器：Wall/Beam/Column/Slab/Door/Window/Stair/Roof/Road/Bridge/Building/Storey/Site/Space 及 Generic 映射。材质、工程量、分类与高级几何不映射。
- 导出按映射写出实体；**楼梯与屋顶导出为 `IfcBuildingElementProxy`**，所有非建筑类型也一样。不保证往返一致。
- 没有 STEP、DXF/DWG、GIS、glTF、OBJ/FBX/STL。见 [FORMAT_COMPAT](status/FORMAT_COMPAT.zh-CN.md)。

### 6.3 库与系列文件
- **Library Manager** 位于 `/library-manager`，编辑 `ComponentDef`；全局库是 `~/.omnim/library.omnimlib`。API：`POST /api/catalog/export` 写出 `/tmp` 下的时间戳路径；`POST /api/catalog/import` 读取你提供的路径。
- **Series Manager** 位于 `/series-manager`，编辑决策树；API：`POST /api/series/save` / `POST /api/series/load` 处理 `.omnimseries` 文件。这些端点返回/接受文件系统路径，不会选择项目相对位置。
- 库与系列**定义**可撤销并随项目落盘；仓库级 `.omnimlib` / `.omnimseries` 导出仍需显式文件操作。

### 6.4 测量与剖切
- 距离/角度/面积测量按钮在视口工具栏；面积用右键结束。测量通过 `POST /api/measurements` 存入 `Project.measurements`，可撤销并随项目落盘。
- **剖切**是只读视角裁剪平面（X/Y/Z 滑杆）。`GET /api/components/{id}/section?plane_z=` 可为 `Component` 返回截面线；没有图纸/图框输出。

完整状态请先读 [ROADMAP](status/ROADMAP.zh-CN.md)、[FORMAT_COMPAT](status/FORMAT_COMPAT.zh-CN.md) 与 [CODE_DEVIATIONS](status/CODE_DEVIATIONS.zh-CN.md)，再对外承诺交付物。
