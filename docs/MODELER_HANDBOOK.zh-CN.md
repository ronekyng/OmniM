---
title: "OmniM 建模者手册"
status: current
last_updated: 2026-10-31
applies_to: "web/, crates/server, crates/model, crates/ifc"
evidence: "任务流程已对照 web/src/app/components/{MainToolbar,create,viewer}、web/src/app/{App,ProjectsPage,TitleBar}.tsx、docs/reference/API.zh-CN.md 与 docs/guides/GETTING_STARTED.zh-CN.md 核实，2026-10-30"
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
- 撤销覆盖全部变更端点（对象、属性、变换、父子/连接、材质、草图、构件、目录、系列、测量、管理区/分解、实例化、IFC 导入）。两处已记录例外见 [WRITE_PATHS](architecture/WRITE_PATHS.zh-CN.md)：`.omnimlib` 导入全局库（无项目级撤销条目，库文件非项目）与 `section_handler` 缓存刷新（非用户变更）。

### 1.4 捕捉
- 视口工具栏有 **Toggle snap** 按钮（`IcoSnap`，提示 "Toggle snap"）。每次拾取与悬停端点都会经过 `snapPoint()`。
- 默认：启用，网格 + 顶点 + 端点开启，边与中点关闭，阈值 **0.5 m**，网格 **1.0 m**。需要不同阈值可改 `web/src/app/components/viewer/three/Snapping.ts`。
- 捕捉只是几何吸附：不会推断墙体、楼层或参数化关系。网格显示用 **G** 切换。

### 1.5 界面外壳与快捷键
- 当前构建默认进入工作区；无打开会话时重定向到 `/projects` 项目首页（新建/打开/最近/示例已落地，2026-10-29）——见 [DATA_INTEGRITY](specification/ui/DATA_INTEGRITY.zh-CN.md) §2 与 [CODE_DEVIATIONS](status/CODE_DEVIATIONS.zh-CN.md) P9。
- 属性面板当前以浮层覆盖视口打开；契约要求停靠右侧。见 [CODE_DEVIATIONS §7](status/CODE_DEVIATIONS.zh-CN.md)。
- 今天已验证的快捷键：`W`/`E` 移动/旋转、`F` 适配、`H` 首页、`G` 网格、`O` 正交、`X` 坐标空间、`Del` 删除、`Enter` 完成拾取、`Backspace` 撤点、`Esc` 取消。完整列表：[KEYBOARD_MAP](reference/KEYBOARD_MAP.zh-CN.md)。

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
4. 点击 **创建**。门窗会以其**门扇/窗扇 + 边框**落在墙体中心线上（`position_x` 为沿墙路径的距离，`position_z` 为离地/窗台高）；宿主墙**不重剪**，需要洞体积时另建 `Opening` 对象。
5. API：`POST /api/genrule`，`{"type":"door","wall_id":"...","opening_width":1.0,"opening_height":2.1,"position_x":0,"position_z":0}`；未知 `wall_id` 返回 404。

限制：门窗几何是**宿主填充**（门扇 + 框），不再使用“整面墙 + 洞”的补片；宿主墙不重剪，洞口用独立 `Opening` 对象（`IfcOpeningElement`）表达。仍缺：框类型、窗台/过梁参数组、防火等级、门窗表。旧墙体（创建时未记录 `path_points`）会按包围盒回退放置，建议重建墙体以获得准确贴合。

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
- **楼层与标高**：楼层是**纯标高容器**，创建时只输入 `elevation`（米，对应 `IfcBuildingStorey.Elevation`，P8-S1），不生成几何；竖向范围可经 `bottom_level_id`/`top_level_id` 对路径驱动对象单步级联（P8-S2）；楼层视图（P8-S4）可按楼层过滤网格并置剖切面。楼层管理在左停靠「轴网」页。仍缺：普通几何的 `level_id` 挂接仅 API。
- **轴网**：✅ 左停靠「轴网」页支持轴线 CRUD（P8-S3a），视口绘制轴线与轴号；轴网是基准注记而非 `ObjectType`，IFC `IfcGrid` 导出仍未实现。
- **墙类型**：没有 `WallType`/`FloorType`/`RoofType` 库对象；层栈按实例编辑。
- **门窗与洞口**：门窗是宿主**填充**（门扇 + 框），落在墙体中心线上；宿主墙**不重剪**。独立 `Opening` 空腔对象（`IfcOpeningElement`）已可用。仍缺：框类型、窗台/过梁约束、洞口明细与门窗表。
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

### 5.2 装配（容器 + 子件，2026-10-30）
1. 点击主工具栏 **Assembly** 打开 **AssemblyPanel**。
2. 命名装配、可选父级，然后逐个添加子件，二选一：
   - **目录构件**：选目录 `ComponentDef`，编辑参数（毫米），设置位置（米）与 Z 旋转（度）；
   - **基本体**：box/cylinder/sphere/cone + 尺寸与位置（米）。
3. 点击 **创建 Assembly**。结果是类型为 `Assembly` 的**容器对象**，子件作为普通对象挂在容器下：对象树可分别选择、删除、导出；子件位姿在建时烘焙进几何。
4. API：`POST /api/assemblies`，`{"name":"...","parts":[{"name":"base","shape":"box","p1":2,"p2":1,"p3":0.2,"position":{"x":0,"y":0,"z":0}},{"name":"pump","component_def_id":"<ComponentDef id>","params":{...},"position":{"x":0,"y":0,"z":0.3},"rotation_z":0}]}`；`skip_series_gate` 可选。创建是**单步撤销**。
5. 移动容器会级联到全部子件（一次撤销）；删除容器默认**整树删除**（`DELETE /api/objects/{id}`，容器自动级联；普通对象仍保持"子件提升"）。
6. 旧路径 `POST /api/genrule` `{"type":"assembly",...}` 仍可用，但输出是**合并网格**（无结构），仅供脚本/遗留调用与内建设备类型（Stair/AHU/Chiller/Transformer/Reactor）。

限制：子件没有独立 live placement（位姿已烘焙），没有 BOM/算量端点、没有运动副或约束装配；嵌套装配未开放。设计契约见 [ASSEMBLY](specification/ASSEMBLY.zh-CN.md)。

### 5.3 运动学与制造（计划中）
- **运动副、约束、轴、运动副对与执行器** 仅计划；没有配合求解器或运动仿真。**紧固件**（螺栓/螺母/垫圈/螺钉/铆钉/销）与制造对象类型（孔/型腔/槽/螺纹/PMI）仅计划。
- 构件特征树中已有通用**倒角/圆角特征**；这与计划中的制造对象模型不是一回事。

见 [ROADMAP Phase 8](status/ROADMAP.zh-CN.md) 与 [CODE_DEVIATIONS](status/CODE_DEVIATIONS.zh-CN.md)。

## 6. 数据与交换
### 6.1 保存与加载 `.omnim`
- 项目是启动时从 `OMNIM_PROJECT`（默认 `project.omnim`）加载的单个 JSON 文档。全部变更端点自动落盘（`OMNIM_EPHEMERAL=1` 关闭）；也可随时 `POST /api/project/save` 强制保存，标题栏还有显式 **Save / Save as** 按钮（快捷键见 [KEYBOARD_MAP](reference/KEYBOARD_MAP.zh-CN.md)）。
- 会话习惯：假设 `OMNIM_PROJECT=./dev-project.omnim`，所有动作均自动落盘；排障时同样可以 `POST /api/project/save` 显式刷写。关闭前检查文件时间戳。
- 撤销不能跨重启保护你：撤销栈只在内存中。

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
