---
title: "变更日志（Changelog）"
status: history
last_updated: 2026-10-30
applies_to: "repo"
evidence: "audited against code 2026-10-27"
---
# 变更日志（Changelog）

本文件记录 OmniM 项目的所有重要变更。仅包含阶段性实质性摘要——这不是提交日志。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)；本项目暂不使用语义化版本，条目按路线图阶段组织。

**最后更新：2026-10-30**

## Keep a Changelog 图例

- **新增（Added）** — 新能力
- **变更（Changed）** — 对既有行为的更改
- **修复（Fixed）** — 缺陷修复 / 已解决偏差
- **未实现（已知缺口）** — 已计划但缺失的能力，为诚实起见明确列出

---

## [2026-10-30] — 创建显示与朝向修复

### 修复（Fixed）
- **新建墙体/楼板/屋顶/楼层不可见**：PathSweep/genrule 产出的域对象几何本来是**米**，但带 `catalog_ref` 的对象在 `/api/mesh` 被统一按毫米缩放 0.001，导致 5 m 墙只剩 5 mm。现在只对真正的目录 `Component` 实例（模板几何以毫米编写）缩放；域对象按米直接显示。
- **梁/管/风管创建后竖直**：目录模板沿局部 +Z 挤出，旧逻辑只发送绕 Z 的偏航，无法把长度轴放倒到拾取线段。`InstantiateCatalogOperation` 新增可选 `rotation` 四元数（`position` 平移前应用），界面按“局部 +Z → 拾取方向”生成并让构件从起点延伸；同时移除目录线性构件创建后的重复中点平移。

## [2026-10-30] — 装配容器与子件

### 新增（Added）
- **`POST /api/assemblies`**：创建 `Assembly` **容器**，直接子件是普通对象——目录 `Component`（`catalog_ref`、特征树几何、参数 mm、位姿 m）或基本体 `Part`（box/cylinder/sphere/cone，位姿烘焙）。创建是一条 `Command::CreateBatch`（含实例化草图），**单步撤销**。
- **整树删除**：`DELETE /api/objects/{id}?cascade_children=true`（容器默认）删除全部后代，并把子树快照进 `DeleteCascade.deleted_children`；一次撤销恢复整棵子树。
- **级联变换**：`PUT /api/objects/{容器}/transform` 把矩阵作用到每个直接子件，并记录一条分组撤销条目（`Command::Batch`）。
- 新 OMCOM 面：`ASSEMBLY_CREATE`（130）+ `CreateAssemblyOperation`、`DELETE_OBJECT_SUBTREE`（131）；AssemblyPanel 支持目录构件（可编辑参数）与基本体 + 位姿输入。

### 变更（Changed）
- `Assembly` 对象现在是**结构容器**（自身无几何），不再是不可拆的合并网格。旧 `POST /api/genrule` 的 `type: "assembly"` 路径保留，供脚本与复用 `AssemblyGen` 的内建设备类型（`Stair`、`AHU`、`Chiller`、`Transformer`、`Reactor`）。
- 路由表：87 条唯一路径 / 88 次注册。

### 未实现（已知缺口）
- 逐子件 live placement（位姿已烘焙）、BOM/工程量端点、运动副与嵌套装配——登记于 [ROADMAP](ROADMAP.zh-CN.md) 与 [ASSEMBLY](../specification/ASSEMBLY.zh-CN.md) 契约。

## [2026-10-30] — 审计整改周期（代码 / 文档 / UI-UX）

### 修复（Fixed）
- **model P0/P1**：`remove_containment` 从 Vec 删除并重建双索引；钢筋笼箍筋 z 沿构件长度分布；`resolve_param` 修正运算优先级（`1+2*3 = 7`）；`set_parent` 原子化并加环防护；`CatalogStore.by_category` 反序列化即重建；对象删除级联清理（子对象提升、关系/lifecycle/breakdown 引用剥离）并支持一步撤销（`DeleteCascade`）；创建时 `ifc_class`/`tags` 直写补记 `ModifyObject`。
- **假成功（omcom/amir）**：未知草图约束码与越界索引改为报错回滚；阵列实例上限 1 万；AMIR `amir-undo`/`amir-redo` 哨兵转调真实 `Executor::undo/redo`；草图求解失败不再回写坐标；近似 Boolean/Chamfer/Fillet 通过新增的 `ExecutionResult.warnings` 字段显式告警（控制台与 AI 面板已消费）。
- **IFC**：语句切分改为字符串字面量感知的状态机（名称含 `;` 不再截断），修复 `''` 转义；`parse_value`/`convert_ifc_item`/放置链加递归深度上限。
- **server**：无认证 API 默认绑回环（`OMNIM_HOST` 显式发布）；导入端点收敛到允许根目录（含 `/api/omcom` 的 `LoadSeries`，拒绝 traversal 与 symlink 逃逸）；导出产物移至 `~/.omnim/exports` 带纳秒后缀；100 MB body 上限只挂 IFC 导入；自动保存失败在响应中给出 `persist_failed`。
- **web**：`api.ts` 全部 31 个方法统一走 `request`/`requestVoid`（非 2xx 抛出）并附契约测试；17 处 `.catch(() => {})` 改为可见告警；Assembly/LayerEditor/目录面板补单位标注；"Unsaved changes" 假警报改为 "Auto-saved server-side · Ctrl+S to snapshot"；轴网跨度改语义开关；默认命名弃时间戳改 `<prefix>-<nnn>`；约束工具栏补 title/aria；草图画布改用主壳石墨配色。

### 变更（Changed）
- CI：`clippy --all-targets`（失败可见）与 web typecheck 步骤；`check_write_paths.py` 扫描不足 20 个 handler 时拒绝通过。
- 实测：Rust 测试 467（原 454）、web 测试 100（原 94）、`--all-targets` 下 clippy 0 error（原为编译失败）、`check_docs.py --strict-parity` 全绿且无永久豁免。

### 未实现（已知缺口）
- G2/G3 前端项：完整 token 迁移（712 处硬编码颜色）、i18n、A11Y、编辑器视觉统一、图标体系重做。
- 审计进度小节中登记的结构性债务：server 认证中间件/`TimeoutLayer`、persist 移出写锁、omcom 拆文件 + executor Result 化、真 CSG 内核、`App.tsx` selector 粒度、react-query hooks 启用或移除、clippy 警告基线收紧。

## [2026-10-30] — 工作区壳层重排（P9-S3）

### 变更（Changed）
- **工具条改为真正的 ribbon 行**：移除居中浮动的胶囊；命令条横跨工作区，按 创建/修改/历史/交换/编辑器/专业 分组并加分隔符，标签不再被压成竖排，仅在需要时横向滚动。
- **左栏标签页**：Objects 与 Grids 分页（对象数徽标），不再上下硬堆；删除多余的 "OmniM Editor" 标题，对象树获得完整高度。
- **轴网编辑器**：拥挤的行内表单改为带标签的 轴号/方向/坐标/起点/终点 字段，操作按钮整行，轴线列表独立滚动。
- **视口控件**：视图工具收成 42 px 边缘窄栏；楼层选择移到视口左上；快捷键提示改为右下方紧凑条。
- **标题栏**：面包屑去掉重复项目名，clean 状态隐藏保存文案，Save/Save-as/Reset 分组；默认工具改为无，不再一直显示 Transform。
- **底部停靠**：折叠的 Console/AI 标签贴在视口边缘，不再留空条。
- **收尾**：轴网表单紧跟列表，不再被顶到底部；快捷键提示改为紧凑条，完整列表放在 tooltip；Component 按钮固定最小宽度，组间距加大。

### 未实现（已知缺口）
- 本次重排没有视觉/e2e 回归测试；在补齐截图与 e2e 证据前，相关需求仍保持“部分”。

## [2026-10-29] — G1 项目首页（P9-S2）

### 新增（Added）
- **`/projects` 首页**：新建项目名、示例项目、当前会话卡片、最近/可用项目表（大小、修改时间、文件缺失徽标）。最近记录保存在 `localStorage`（`omnim.recent.projects.v1`，最多 10 条），绝不写入项目文件。
- **项目会话 API**：`GET /api/projects`（根目录下现有 `.omnim` 的名称/大小/修改时间）、`GET /api/projects/current`（`open` 仅在服务会话内有效）、`POST /api/projects/new`（唯一 slug、可选示例内容、写盘、清空撤销）、`POST /api/projects/open`（加载、切换保存目标、清空撤销）。
- **示例项目**：两个楼层（0 / 3.9 m）与四条轴网（A/B、1/2），不含几何，打开即用。
- **工作区门禁**：服务重启后工作区重定向到 `/projects`，直到用户新建/打开项目；标题栏项目名也可返回首页（脏状态先确认）。
- **允许根目录**：相对路径在 `OMNIM_PROJECTS_DIR`（默认 `~/.omnim/projects`，或当前项目父目录）下解析；open 与 save-as 对 `..`、根外绝对路径、非 `.omnim` 与符号链接逃逸返回 `400`（`UI-SEC-03`）。

### 变更（Changed）
- 路由：86 条唯一路径 / 87 次注册（新增 4 个项目会话端点）。
- 前端测试：9 个文件 / 94 例；Rust 454。
- 默认项目文件从服务工作目录迁移到项目根目录。

### 未实现（已知缺口）
- 项目元数据编辑（`UI-DAT-07`）、依赖报告（`UI-DAT-09`）、自动保存/恢复（`UI-DAT-04/05`）、缺失文件“定位”、应用内导航拦截与主题/加载态。

## [2026-10-29] — G1 Alpha 外壳与可靠性（P9-S1）

### 新增（Added）
- **停靠式工作区外壳**：属性面板默认右停靠（260–600 px），宽度与停靠模式持久化到 `localStorage` 的 `omnim.layout.v1`；**重置布局**恢复契约默认值。低于 1024 px 显示阻塞提示。
- **标题栏与状态栏**：项目名 + 脏标记、面包屑（`项目 / 工作区 / 工具`）、显式 **保存** / **另存为**；只读状态栏显示光标坐标、选择、单位与保存状态（FPS 仅开发者模式）。
- **错误隔离**：应用级、工作区级与面板级 `ErrorBoundary`；单个面板崩溃时外壳仍可用并提供重试。
- **共享弹窗与通知层**：`ConfirmDialog`（Esc 取消、Enter 确认）替换全部原生弹窗；`notify` 供非 Hook 模块发类型化提示。错误提示改为常驻直到手动关闭。
- **显式保存状态机**：`saveState.ts`（clean/dirty/saving/failed、最近保存时间）、`Ctrl/Cmd+S`、脏状态下 `beforeunload` 拦截，以及 `POST /api/project/save-as`（校验 `.omnim` 并切换会话保存目标）。
- **自动命名**：`naming.ts` 实现 `Type-nnn`、最低空号复用与重名后缀，接入创建、快速创建、子对象、复制与拖入实例化。

### 变更（Changed）
- 路由：82 条唯一路径 / 83 次注册（新增 `/api/project/save-as`）。
- 前端测试：8 个文件 / 91 例；Rust 450。UI 契约扫描：原生弹窗 0。

### 修复（Fixed）
- 消除 `test_find_and_rebuild_catalog_instances` 偶发失败的并行竞态：可能重播全局 `LIBRARY` 的测试统一经 `LIBRARY_TEST_LOCK` 串行。

### 未实现（已知缺口）
- 项目首页（`/projects`）与 open/new 端点（`UI-DAT-01`）；未保存更改的应用内导航拦截（`UI-DAT-06` 目前仅 `beforeunload`）；深浅主题（`UI-DS-02`）；面板加载/空状态与编辑器面包屑。

## [2026-10-29] — 前端文档基线（G0）

### 新增
- **产品层**（`docs/product/`）：愿景与范围、画像与五条旅程、96 条需求的 PRD、需求追溯登记册（RTM）。
- **界面契约**（`docs/specification/ui/`）：外壳/信息架构、设计系统、交互、反馈与错误、数据完整性、可访问性、国际化、性能、兼容性、安全、遥测——11 对双语契约。
- **真值层**：`WEB_ARCHITECTURE`（含实测债务的现状）、`FRONTEND_MATURITY`（ISO/IEC 25010 评分 1.7/5）、`FRONTEND_BASELINE`、`RELEASE_READINESS`（G0–G3）。
- **流程层**：完成定义、设计运营、发布与支持清单、G0 独立评审记录。
- **参考**：快捷键表与稳定的 `OMNI-*` 错误码目录。
- **工具**：`scripts/check_ui_contracts.py`（需求 ID、快捷键冲突、错误码、债务指标，支持 `--strict`）、`scripts/gen_keymap.py --check`；两者与 `npm test` 均已接入 CI。`check_docs.py` 现在也校验 front matter。

### 变更
- **门禁再平衡（O1）：** 6 项完备度需求从 G2 移到 G3（`UI-DS-05/07/10`、`UI-A11Y-06/08`、`UI-I18N-05`）；`UI-DS-08` 保留在 G2。门禁总数为 G1 25 · G2 37 · G3 34；状态总数仍为 ✅1 · 🔀17 · 📅78。
- **更正声明**：81 条唯一 REST 路径（原为 79/83/"约 120"）、731 hex + 61 rgb/hsl 颜色字面量、31 个文件 858 段中文、63 处 emoji、最低支持宽度 1280×720（1024–1279 降级）、`FCP ≤1.2 s` 与 `FMP ≤2.5 s` 分开表述。
- **修复过期引用**：`OmcomConsole.tsx` → `OmniPanel.tsx`、`CatalogBrowser.tsx` → `AvailableComponentsPanel.tsx`；`test-all.sh` 不再重复跑两遍 Web 测试。

### 未实现（已知缺口）
- 视觉基线截图、性能参考机、e2e/视觉/可访问性/性能套件、字符串扫描对英文硬编码的检测。均记录为 G0 评审的 O3–O6。

## [2026-10-29] — 楼层视图与剖切联动（P8-S4c）

### 新增（Added）
- **视口工具栏楼层选择器**：「全部楼层」+ 每个 `BuildingStorey` 一项（标注标高）。选择楼层会把视口过滤到与该楼层关联的对象（优先 `level_id`，回退 `bottom_level_id`；未挂接几何保持可见），并启用位于该楼层标高 +1.2 m 的平面剖切。清除选择恢复全部并关闭由楼层驱动的剖切。
- **`/api/mesh` 携带楼层关联**（`level_id`、`bottom_level_id`、`elevation`），过滤在客户端完成；mesh 端点其余结构不变。

## [2026-10-29] — 管道实例连接器与弯头管件（P8-S4b）

### 新增（Added）
- **`OmniObject.instance_connectors`**（serde default）：物理连接点，含位置、朝外单位方向、口径（m）与可选连接类型码。路由管创建 `start`/`end` 两个连接器（口径 = 2×半径）。
- **`connect()` 连接器绑定**：选择最近连接器对，记录 `from_point`/`to_point`（此前恒为空），`connection_detail` 记录连接器对（`from:end;to:start`）。口径差 > 1.5 mm 或类型码不一致 → `400`；无实例连接器的对象保持原行为。
- **弯头管件**：路由管每个内部弯点生成 `Fitting` 对象（类型 307，父关系挂到管道，`fitting_type: "elbow"`、口径）——在真实斜接/圆弧几何落地前为 1.15×半径的球体占位。
- `Command::CreateBatch`：路由管与弯头作为**一步撤销**（`record_create_batch`），而非 N 条独立记录。

## [2026-10-29] — 管道路由接入特征树（P8-S4a）

### 新增（Added）
- **`PathSweepSection`**（`Layered` | `Circular { radius }`）加入 `PathSweepParams`（serde default 保证旧项目可加载），并新增 `Circular` 模板步骤类型。
- 种入 **`Pipe-Path`** ComponentDef + 特征树模板；路由 `genrule: "pipe"` 改为实例化该模板（`PathSweep` 圆形截面），不再生成游离网格——路由仍记录在对象上（`route_points`/`pipe_radius`/`route_length`），且网格与 `gen_rule::pipe_route` 内核逐位一致（parity 测试）。
- 无路径直管保持遗留网格构建；标高驱动的“高度”调整仅作用于 layered 截面。

## [2026-10-29] — 管道路由（P8-S3c）

### 新增（Added）
- **路由管道**（`POST /api/genrule`，`genrule: "pipe"` + `path`）：圆形截面沿拾取的 N 点折线经 LinearComponent 内核扫掠——弯头连续过渡，而非两段独立圆柱。路由记录在对象上（`route_points` JSON、`pipe_radius`、`route_length`），对象类型为 `PipePlant`。
- 校验：`< 2` 点、闭合路径、零长度段、`radius ≤ 0` 均在**创建前**返回 `400`（不留孤儿）；选中系列的路由管不再落入直管目录路径。
- 遗留 `pipe`（无路径，`length` + `radius`）行为不变。

### 仍未关闭
- 分支、管件/弯头对象与管道特征树模板仍不在范围内；路由管是网格化 Linear 族路径（CODE_DEVIATIONS §6.3 记为过渡）。

## [2026-10-29] — 独立洞口对象（P8-S3b）

### 新增（Added）
- **`ObjectType::Opening`（116）** 映射 `IfcOpeningElement`（导入 + 导出）：宿主相对的**空腔体积**，经 `POST /api/genrule`（`genrule: "opening"`）创建，参数 `wall_id`、`opening_width`、`opening_height`、`position_x/z`。宿主必须是已存在的 `Wall`（否则 404/400）；洞口父关系挂到墙上并记录 `host_wall_id`。创建/删除走单一写入路径（撤销/重做覆盖）。
- Web：建筑类型列表新增「洞口」，复用门窗的宿主表单（目标墙 + 洞口尺寸 + 偏移）。

### 仍未关闭
- 宿主墙网格**不**重剪（特征树几何保持权威；与门窗同为过渡行为）。已在能力矩阵记录。

## [2026-10-29] — 轴网 UI 与视口（P8-S3a）

### 新增（Added）
- **轴网面板**（左侧停靠）：轴线列表/新增/编辑/删除（`label`、轴族、坐标、范围），对接 `/api/grids`；变更后触发场景重载。
- **视口渲染**：轴网以彩色轴线绘制（数字族/字母族区分），端部带轴号标签；范围 0/0 按 ±20 m 有界显示。纯基准叠加层——不参与拾取与网格同步。

## [2026-10-29] — 标高 UI 与 IFC 标高导出（P8-S3）

### 新增（Added）
- **创建面板标高选择器**：墙/板/楼面/屋顶提供底部/顶部楼层下拉（选项来自 `GET /api/objects`，按标高排序并以米标注）。取值经 `layered` genrule 与通用 `POST /api/objects` 传入；成对/不同由前端校验，顺序由服务端校验。
- **IFC 标高导出**：`IFCBUILDINGSTOREY` 以 schema 形状（10 属性）写出，原生 `Elevation` 属性承载 `OmniObject.elevation`（未设置写 `$`）；导出结果仍可被导入。

### 仍未关闭
- 目录 `Component`（Extrude 特征）放置尚未按标高约束重新求值；普通盒体几何仍仅获得基准。

## [2026-10-29] — 标高驱动竖向范围（P8-S2）

### 新增（Added）
- **`bottom_level_id` / `top_level_id`**（`OmniObject`，成对设置）：基高解析为底部楼层标高，并对路径驱动（`PathSweep`）几何把路径基高平移到该标高、扫掠高度置为 `top − bottom`；顶部须高于底部；无标高的楼层按 0 m 计。
- **楼层标高级联**：修改楼层 `elevation` 会在**一步撤销**内重新解析所有受约束对象（`Command::ModifyObject.cascade` 携带 before/after 镜像；redo 对称）。
- API：`POST`/`PUT /api/objects` 接受范围引用；`layered` genrule 变体在**创建前**校验（拒绝时不留孤儿对象）。
- 删除仍被 `level_id`/`bottom_level_id`/`top_level_id` 引用的楼层返回 **400**（"先解除挂接"）。

### 修复（Fixed）
- **layered genrule + 选中系列**不再报 "series not supported for this type"：该族始终走路径驱动模板路径，系列 id 仅作元数据。

### 仍未关闭
- 普通几何对象仅获得 `elevation` 基准（网格不重写）；目录 `Component`（Extrude 特征）放置尚未按标高约束重新求值；IFC 导出标高值仍为计划项。

## [2026-10-29] — 建筑基准：楼层标高与轴网（P8-S1）

### 新增（Added）
- **楼层标高**：`OmniObject.elevation`（可选，米，对应 IfcBuildingStorey.Elevation），创建/更新均可携带；web 创建面板为 `storey` 提供标高输入（允许负值表示地下室）。
- **标高挂接**：`OmniObject.level_id` —— 对象可挂接到既有 `BuildingStorey`；omcom 校验器拒绝未知 id 与非楼层目标（400 附理由）。`top_level_id`/`bottom_level_id` 高度自动调整仍为计划项（DATA_MODEL §5.8 注）。
- **轴网基准**（`Project.grids`，新模块 `crates/model/src/grid.rs`）：轴线是基准注记而非对象——`POST/GET /api/grids`、`PUT/DELETE /api/grids/{id}`，同轴族标签唯一（`along_x` 数字族 / `along_y` 字母族），列表排序输出，经三个新 op（omcom oneof 74–76、Action 231–233）撤销/重做，随项目落盘（serde default 保证旧 `.omnim` 文件可加载）。
- **Web**：storey 标高字段与校验；`createInitialForm('storey')` 现默认 `IfcBuildingStorey`（此前误为 `IfcWall`）且标高 `0`。

### 修复（Fixed）
- **`ModifyObject` 撤销现覆盖全部可修改字段** —— Wave 1 缺口：`ifc_class`/`tags` 编辑此前不可还原、redo 为空操作；命令现携带 after 镜像，撤销/重做对称（P8 字段顺带覆盖）。
- **`POST /api/objects` 校验失败返回 400 与校验理由**（此前是无提示的 500）。
- 文档：能力矩阵"78 条路由"为 Wave 1 前的陈旧值——路由数现已统一为 **83**（API.md 与矩阵，中英同步）。

## [2026-10-28] — 系列与连接强制性（Wave 4）

### 新增（Added）
- **系列门禁强制规格**（审计 P0-5）：系列已加载且构件可达时，定位 `SpecComponent` 强制 `ParamConstraint`（`fixed` 相等、`min`/`max` 边界 → 400 含理由），并把 `SpecComponent.material` 应用到实例（id → 名称忽略大小写；解析失败 → 400 并列出可用材质）。
- **`connect()` 加固**：拒绝自连与重复连接，支持 `?connection_type=`（默认 structural），并检查目录实例间连接器 `connection_type_code` 兼容性。
- **`ConnectorDef.up_x/y/z_expr`**（proto + model + `/api/catalog/{id}/connectors`）：法兰朝向字段，与 position/direction 一样按表达式解析。

### 修复（Fixed）
- 种子材质补齐管道规格引用的不锈钢牌号（`A312-TP304/TP316`）——强制执行把潜在的种子缺口变成了硬失败，故从源头修复。

### 仍未关闭
- 连接时的解析值朝向校验、`step` 强制、`from_point`/`to_point` 填充（CODE_DEVIATIONS §6.4）。

## [2026-10-28] — 几何路径收敛（Wave 3，layered 族）

### 新增（Added）
- **`FeatureType::PathSweep`** 特征 + `gen_rule::path_sweep` 门面（封装 LayerExtrusion 内核——分类之下的常截面原语，DATA_MODEL §1 治理规则 3）。与遗留构建器做逐位 parity 测试（直段/折段/封闭）。
- **路径驱动模板**（`Wall-Path` / `Slab-Path` / `Floor-Path` / `Roof-Path`）：单步 `layered_path_sweep`；拾取路径与 LayerEditor 层栈经 `TemplateContext` 传入（静态轮廓模板无法表达的实例级输入）。
- **`Project::instantiate_domain_object`**：产出**领域对象类型**（`Wall`/`Slab`/`Floor`/`Roof`）的特征树实例化，携带 `catalog_ref` + `param_values` —— 此前特征树只能产出 `ObjectType::Component`。
- genrule 的 **layered** 族改经上述模板实例化：同一内核、同一类型、同一溯源，所有入口一致。路径校验错误（零长度、退化面积、自相交）以内核原因使实例化失败。

### 仍属过渡（已记录）
- GenRule 矩形快速创建（`wall`/`slab` 无路径变体）、屋顶两点坡向、目录 extrude 型 pipe/road 模板（LinearComponent 收敛留给后续切片）。

## [2026-10-28] — 路径契约与创成形式授权（Wave 2）

### 新增（Added）
- **`path { points, closed }` 契约**：`/api/genrule` 的 `layered`、`road`、`line`；遗留 `path[]`/`boundary[]`/`alignment[]` 别名按历史闭合语义归一化；同一输入双拼写返回 400。
- **`(objectType, form)` 授权矩阵**（`gen_rule::matrix`，DATA_MODEL §1 治理规则 4）：已授权 wall×panel/region、slab/floor/roof×region、roof×panel（两点坡向）、road×linear、line×point；矩阵外组合返回 400 及契约理由。创建对象记录 `genesis_class`/`genesis_form` 属性。
- **共享纯几何层**（`gen_rule::path_geom`）：`iter_segments` / `merge_collinear` / `validate_path` 按开/闭分派——位于分类之下、无类语义（§6.4 边界注记已写成两个引擎的模块头契约）。
- **前端**：`CreationForm.form` 字段（墙 panel/region 下拉、屋顶子模式接入），唯一 `form → pickMode` 查表，`withGenesisForm` 辅助；统一 `addPathPreview`（region → 轮廓+填充，panel → 分层 ghost+miter，linear/point → 引导线）。

### 测试（Tests）
- `genrule_contract.rs`（9 个 seed 用例：契约/别名/region/profile/矩阵 400/闭合拒绝/歧义/蝴蝶结）、`path_geom` + `matrix` 单测、web `genesisForm.test.ts`（10 例）。全量：Rust 385，web 69。

## [2026-10-28] — 单一写入路径与持久化（Wave 1）

### 变更（Changed）
- **单一写入路径恢复**（审计 P0-2）：所有变更 REST handler 都构造 `OmniCommand` 并经 `Validator` + `Executor`，或在持锁时委托 `Executor`。直写 `Project` 从 **42 → 0**；新增 op 覆盖材质、草图/构件/目录细节、系列、生命周期、测量、管理区/分解、目录实例化与 `load_series`。
- **持久化默认开启**（审计 P0-3）：变更写入 `$OMNIM_PROJECT`（默认 `project.omnim`）；`OMNIM_EPHEMERAL=1` 关闭写入。`Project::save` 为原子写（`{path}.tmp` → rename，自动建父目录）。新增路由 `POST /api/project/save`（去重后 79 条）。
- **撤销/重做覆盖完整**：端点级测试位于 `crates/server/tests/write_paths.rs` 与 `instantiate_import.rs`，覆盖对象/属性/材质、草图、构件、目录定义、系列、生命周期、测量、管理区/分解、实例化与 IFC 导入。修复了连接删除、生命周期记录、记录关联三处原有的错误撤销/重做。
- **IFC 导入为单步事务**：`Command::ReplaceProject { before, after }`——undo 恢复导入前项目，redo 重放。**目录预览改为只读**（在一次性项目上构建）；目录实例化把对象与生成的草图记录为一条命令。
- **CI 守卫**：`scripts/check_write_paths.py`（要求 0 直写）与 `scripts/check_docs.py` 接入 CI；`scripts/test-all.sh` 现在依次跑两者、`cargo test` 与 web 单测。

### 修复（Fixed）
- 已记录例外：`section_handler`（仅缓存刷新锁）、`import_ifc`（单条整项目命令）、`POST /api/catalog/import`（会话级全局 `LIBRARY`，非项目状态）。见 [WRITE_PATHS](../architecture/WRITE_PATHS.zh-CN.md)。

## [2026-10-28] — Git 分支与工作流规范

### 新增
- 分支模型定稿：`main`（发布基线）+ `develop`（staging，推送至 `origin/develop`）；短期 `<type>/<phase>-<slug>` 分支；新规范先走 `docs/*` 再走 `feat/*`（文档先行）。见[分支与工作流指南](../guides/BRANCHING.zh-CN.md)。
- 提交规范（Conventional Commits，scope 取 crate/area 名）与合并策略：任务 squash 合入 `develop`；`release/*`/`hotfix/*` 以 merge commit 合入 `main`；每次 `main` 变动后立即回合并 `develop`。
- 阶段式 annotated tag（`phase-7` 基线、`phase-8`、`phase-8.1`、`phase-8-rc.1`）；项目仍不使用语义化版本。
- 配套工具：`.github/workflows/ci.yml`（fmt/clippy/test/web 构建/docs 检查）、PR base/head 守卫（`scripts/ci/branch_guard.py`）、PR 模板、`.gitattributes`、本地钩子（`.githooks/commit-msg`、`.githooks/pre-push`）、提交模板（`.gitmessage`）、一键配置（`scripts/setup-git.sh`）。

## [2026-10-27] — 文档重组与死代码清理

### 新增
- `docs/` 重组为 `architecture/`、`guides/`、`specification/`、`reference/`、`status/`、`RFC/`（每篇中英双语），新增文档门户（`docs/README.md`）。
- 新参考文档：全量 REST API 参考（`reference/API.md`，对照 `crates/server/src/lib.rs` 审计出 **78** 条去重路由）。
- 新规范：元件库与系列库（`specification/CATALOG_SERIES.md`）、数据流、架构决策记录（6 条种子 ADR）。
- 新指南：快速开始、构建、架构地图、测试。
- RFC-0001：Omcom 文本 DSL（草案）——把原来的"文本 DSL"愿景归位为未实现的设计提案。
- 首份端到端 CHANGELOG（本文件）。
- N 点拾取几何内核：分层墙/板生成保留拾取标高（`base_z`）、屋顶挤出剖面构建器、道路折线对齐（`Road.alignment`）、转角 miter、底面校验（面积/自交/共线，400 报错）、共线点合并；+9 项内核测试。
- Web 交互打磨：拾取中 Backspace 撤销上一点、按模式/点数的四态拾取提示、多边形闭合高亮段；道路创建提交拾取对齐线；+6 项状态机测试。
- 拾取预览重构：整条多段墙 ghost 预览（逐段盒体 + 转角 miter 与服务端内核一致、修正层厚 mm→m 换算——修复了 20 倍厚度错位）、hover 终点走与点击一致的吸附解析、带光照半透明材质、无效多边形底面（自交/退化）红色预览；预览生命周期拆分为"已确认标记/路径"与"逐帧 hover 组"；+10 项预览几何测试。

### 变更
- `PLATFORM_ARCH.md` 由云优先愿景式改写为对当前实现的如实描述（REST + JSON 文件持久化，无 WebSocket/DB/WASM/移动端）。
- `FORMAT_COMPAT.md` 修正：IFC 导入/导出为部分支持（手写解析器）；DXF/DWG/STEP-CAD/GIS/glTF 未实现。
- `ROADMAP.md` 状态修正（Phase 1 的文本 DSL 声明改述为 RFC-0001；每阶段附代码佐证）。
- `DATA_MODEL.md` 对照代码审计：修正约 25 处过时/不实声明（如已实现的 Torus/Dish 图元、真实 FeatureType 枚举、"规范-代码"分歧标注 🗓 计划中）。
- `OMCOM_REFERENCE.md` 重写：原为虚构文本 DSL 的参考；现忠实记录真实的类型化 OmniCommand 系统（proto/omcom.proto + crates/omcom Validator/Executor）。

### 删除
- `crates/renderer` —— 无人依赖的 wgpu/winit 演示 crate，从 workspace 移除；渲染由浏览器内 three.js 承担，原生查看器如需重启走 RFC。
- 未用依赖：`prost-types`（core）、`tower`（server）、`omnim-omcom`（amir）。
- 已声明未实现的 Action 枚举值（62/80/81/82 等）在 `proto/omcom.proto` 中标注为保留（编号保留、不复用）。
- 死前端组件：`CatalogBrowser.tsx`、`SeriesBrowser.tsx`、`OmcomConsole.tsx`、`ViewCube.tsx`（已被 OmniPanel / ViewCube3D 取代）。
- 仓库垃圾：web 一次性脚本（`fix_delim*.sh`、`test.sh`、误提交的 `web/dist/`）、过时的 `scripts/update_roadmap.py`、调试 `eprintln!`/`console.log`。
- 仓库根目录测试文件迁移至 `fixtures/ifc/`（文档同步）；`logo.png` 移至 `docs/img/` 并嵌入中英 README。

### 修复
- 整个 workspace 编译警告清零（未用变量/导入、多余 `mut`）。
- 文档内部链接全部校验（0 坏链）。
- 审计整改（docs/status/DOC_AUDIT_2026-10-27）：快速开始命令纠正（真实控制台语法、`/api/undo`、原始文本 IFC 导入）并加显著持久化警告；README 愿景/现状隔离；IFC_EXTENSION 按“部分 IFC”现实重写；ADR-0006 标记为“已接受（目标）——部分实现”并附偏差说明；新增 `architecture/WRITE_PATHS.md`；路由数纠正为 78；CODE_DEVIATIONS §6 记录审计 P0；中文 DATA_MODEL 对齐（+371 行：§5.6、§5.7.1、§16.4、§16a、§17、章节重编号、围栏修复）；DOMAINS/GLOSSARY 增加 proto 状态列；新增 `CAPABILITY_MATRIX` 与 `MODELER_HANDBOOK`；全部 71 篇文档加 front matter；新增 `scripts/check_docs.py`（链接/围栏/编号/双语对齐）。代码级 P0（单一写入路径、保存策略、几何收敛、系列强制性）已记入 ROADMAP 技术债待办。

### 未实现（已知缺口）
- 不变：仅 REST（无 WebSocket/实时协作）、JSON 文件持久化（无数据库/版本历史）、无 WASM 共享内核、无桌面/移动打包（见 `PLATFORM_ARCH` 未来方向章节）。

---

## [Phase 8 — 进行中]（目标 2028 Q1-Q2，已启动）

### 新增
- 生产级目录数据：钢型材（IPE/HEA/HEB/CHS/SHS，按 EN 10365、GB/T 706）、管道等级（CS150、SS300 / ASME B36.10）、混凝土墙/梁/柱/板模板、带法兰等级的连接点定义。
- 材料映射到真实标准（EN 10025、GB/T 1591、ASTM A36）；构件模板经 BRep 正确性验证。

### 未实现（已知缺口）
- 领域专项操作（MEP、土方/路线、管道布线、运动副）、工程量统计、领域专属 UI 面板。

### 修复
- 见 [CODE_DEVIATIONS.md §1/§2](CODE_DEVIATIONS.md)：4 类构件分类法、目录毫米 vs. 米缩放——**均于 2026-08-11 RESOLVED**。

---

## [Phase 7 — 用户界面]（2027 Q4）— 完成

### 新增
- Project Workspace：three.js 视口（轨道/平移/缩放、选择、剖切、测量、线框/着色/X 光/仅边线视图模式）、特征树、可用构件面板（系列筛选/滑块/拖拽放置）、属性面板、Omcom 控制台（语法高亮/历史/Tab 补全）、AI 聊天面板。
- Library Manager（`/library-manager`）：ComponentDef 树、参数编辑器、特征模板分配、3D 预览、`.omnimlib` 导入/导出。
- SeriesStore Manager（`/series-manager`）：决策树编辑器、种子查看器、`.omnimseries` 加载/保存、拖拽映射、约束编辑器。
- UI 架构：Zustand、React Query、PanelRegistry、可调整布局、CSS Modules。

### 未实现（已知缺口）
- 场景内 N 点多边形/折线拾取（见 CODE_DEVIATIONS §3/§5）；AI 命令预览/接受工作流（延至 Phase 9，部分存在）。

---

## [Phase 6 / 6b — GenRule 重构与 Library 基础设施]（2027 Q3-Q4）— 完成

### 新增
- Library ↔ SeriesStore ↔ Project 三层分离；Library 为 session 级全局静态；项目私有 SeriesStore + `external_series_refs`。
- 系列关卡（`check_series_gate`）、GenRule→目录/系列实例化、实例上的 `CatalogRef`、默认模板注册中心、按目录取值域的参数验证。
- `.omnimlib` / `.omnimseries` 外部文件格式、系列解析优先级、session 自动加载。

### 变更
- `generate_geometry()` 弃用，由特征引擎取代；`POST /api/genrule` 现通过目录/系列系统创建对象。

---

## [Phase 5 — 目录构件建模]（2027 Q2）— 完成

### 新增
- 覆盖结构（IPE/HEA/HEB/CHS/SHS/角钢）、管道（直管、弯头、三通、大小头、法兰、阀门）、建筑（混凝土/门/窗）族别的 20+ 参数化目录构件；系列定义 CS150、SS300、Concrete-C30、Steel-Q355、Highway-GB。
- 截面库、ASME B36.10M 壁厚表、FeatureTemplate + FeatureTreeRegistry、实例化管线、REST `/api/catalog/instantiate` 与 `/api/catalog/preview`。

---

## [Phase 4 — 元件目录与系列库]（2027 Q1）— 完成

### 新增
- PARAGON 风格元件目录（ComponentDef/ParamDef/ConnectorDef、Category 层级）与 SPECON 风格系列库（选择器决策树、SpecComponent 约束、数值范围匹配）、选择引擎、连接点类型码兼容性、外部文件存储。

---

## [Phase 3 — 特征引擎与 BRep 内核]（2026 Q3-Q4）— 完成

### 新增
- Feature 作为一等实体（缓存 MeshGeometry、脏跟踪、重建管线、回滚/抑制）；9 种特征类型（Extrude、Revolve、Sweep、Loft、Boolean、Chamfer、Fillet、Pattern、Shell）。
- truck-BRep 集成（实体拉伸/旋转、沿路径扫掠、放样、BRep 布尔运算）、细分网格、带 meval 表达式的参数系统。
- Revit 风格 Component Editor UI（feature 面板/工具带、自动建草图工作流、拖拽排序、Rebuild All）。

---

## [Phase 2 — 约束求解器与草图]（2026 Q2-Q3）— 完成

### 新增
- 草图数据模型（SketchPlane、图元、11 种约束类型、SketchStore）；基于 slvs 的约束求解器，带自由度分析与过/欠约束报告；草图/特征/构件命令带撤销/重做；独立 Sketch Editor 网页；`EXTRUDE_SKETCH` / `REVOLVE_SKETCH` 生成 3D 网格。

---

## [Phase 1 — 核心数据模型与命令层 — 部分完成]（2026 Q1-Q2）

### 新增
- 统一数据模型（OmniObject 层级、属性/属性集、材质、覆盖 4 领域的 ObjectType）、项目生命周期与分解结构、行政分区层级、关系系统。
- `OmniCommand` 类型化 schema，纯作 serde-JSON 类型使用（无 protobuf 线上运行时）以及完整的类型化命令执行器，带完整撤销/重做。

### 未实现（已知缺口）
- **Omcom 文本 DSL / nom 解析器**——Web UI 控制台执行 JSON 命令加少量临时捷径。文本 DSL 为草案 RFC：[RFC-0001](../RFC/README.md)。

---

## 基线（当前运行时）

- Axum REST 服务器（**78** 条去重路由）+ JSON 文件（`.omnim`）持久化；**无 WebSocket/实时协作，无数据库/版本历史/审计追踪**。
- AMIR AI 引擎（OpenAI/Anthropic/Mock 提供方）：自然语言→OmniCommand 翻译、聊天、逆向模型描述。
- 手写 IFC（2x3/4）导入/导出，**实体覆盖不完整**；无 STEP/DXF/GIS/glTF 支持。
- Web UI：基于 three.js 的工作台、草图编辑器、构件编辑器、系列/库管理器、AI 聊天、Omcom 控制台。
