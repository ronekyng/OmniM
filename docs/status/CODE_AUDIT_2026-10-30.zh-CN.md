---
title: "代码审计报告（2026-10-30）"
status: history
last_updated: 2026-10-30
applies_to: "crates/, web/, proto/, scripts/, .github/"
evidence: "四路并行审计 + cargo fmt/test/clippy、web tsc/vitest 本机交叉核验，2026-10-30"
languages: zh-CN
---
# 代码审计报告（2026-10-30）

> 本文为单语言（zh-CN）快照，不在 en/zh 双语配对范围内（对应 MODELER_DOCS_AUDIT_2026-10-30 P2-2）。
>
> **整改进度（2026-10-30，同日第一波）**：已修复并核验——① P0 `remove_containment`（`relation.rs`：从 Vec 删除并重建双索引，附回归测试）；② 箍筋 z 坐标（`mesh_gen.rs`：`z: centre_z + r*st`，附 testament 测试断言 z 跨度）；③ P1 `"1+2*3"→9`（`feature_template.rs::resolve_param` 重排 `+/-` 与 `*/÷` 优先级并左结合，附单测）；④ clippy `approx_constant`×6 全部修复（amir/model/server 测试），CI clippy 加 `--all-targets`；⑤ web CI 加 `tsc --noEmit`（`npm run typecheck`）与 package.json 脚本。核验：`cargo fmt --check` ✅、`cargo test --workspace` 457 ✅、`cargo clippy --workspace --all-targets` 0 error ✅、`tsc --noEmit` ✅。其余按第五节路线图分波推进。
>
> **整改进度（2026-10-30，第二波，P1 全清 + 可行 P2）**：安全——server 默认回环监听、`catalog/import`/`series/load`/`omcom LoadSeries` 复用导入根约束（traversal/symlink 双校验，`resolve_project_path` 同步加固）、导出改 `~/.omnim/exports` 纳秒后缀、100MB body 仅挂 IFC 导入路由、vite dev 弃 `--host 0.0.0.0`；model——`set_parent` 原子化+环防护、`by_category` 自定义反序列化重建、`delete_object` 级联（子对象提升/关系边移除/lifecycle+breakdown 引用剥离/锚定 storey 拒绝）+ `DeleteCascade` 一步撤销、`ifc_class/tags` 直写补记 ModifyObject；omcom——未知约束码/OOB 索引报错回滚、阵列上限 1e4、AMIR undo/redo 转真调用、Boolean/Chamfer/Fillet 近似内核 warning 化、自动保存失败 `persist_failed` 响应可见；IFC——`';'` 字符串态切分（顺带修复 `''` 转义死分支）、`parse_value`/`convert_ifc_item`/placement 链深度上限；web——`requestVoid` 统一错误契约（31 个裸 fetch 方法全部收拢、API 层新增 6 个契约测试防回归）、`.ok` 二连判调用点改造、17 处 `.catch(() => {})` 全部变可见告警；CI/脚本——`check_write_paths` 扫描不到 handler 即 FAIL。核验：467 rust 测试、94 网端测试、8 门禁全绿。
>
> **整改进度（2026-10-30，第三波，验收记录回应）**：① web P1 补完——api.ts 剩余 16 个跨行裸 fetch（series 族 7、catalog/feature 族 9）全部收拢 `requestVoid`（现已 31/31），`AvailableComponentsPanel.tsx:41`、`ComponentEditorPage.tsx:86/1116/1123` 四处 `.catch(() => {})` 改可见告警，并新增 `src/shared/api.test.ts` 契约测试（6 条：500/409/204 贯穿 request/requestVoid/series/catalog/exportIfc 族），vitest 100 通过；② `ExecutionResult` 新增 `warnings: Vec<String>` 字段（近似内核告警不再是 message 字符串拼接），`AiChat`/`OmniPanel` 控制台与 AI 链路统一消费：`result.success` 为 false 即显式报错（AMIR 撤重做空栈不再无感），warnings 以 toast 呈现；③ 手册 §1.3 与 `CAPABILITY_MATRIX` 双语撤销条目补两处 WRITE_PATHS 例外（`.omnimlib` 全局库导入、`section_handler` 缓存刷新），`GLOSSARY.md` 补齐 zh 版"约束操作 (EditorToolbar)"小节使 `check_docs.py --strict-parity` 达 57 对全绿（永久豁免解除）。仍未入本轮（结构性/评定为低风险，见下"遗留"）：`App.tsx` 无 selector 全量订阅（20Hz 重渲染）、212 条 clippy 警告与 `-D warnings`、`resolve_param` 未知参数静默回 0、omcom 拆文件、`persist` 移出写锁、真 CSG、react-query hooks 启用或移除、`ThreeViewer` 拆分、token 中间件、sketch 约束构建 `let _ =` 语义化、`OmniPanel dangerouslySetInnerHTML` 改 token 渲染、`get_project` 全量克隆等性能债。
>
> **关闭记录（2026-10-30）**：本轮审计内容处置完毕——P0×1、P1×19 与可落地 P2 全部整改（三波记录见上）；未处理项已按门禁归属登记于 [ROADMAP](ROADMAP.zh-CN.md)「审计残余跟踪」与 `CODE_DEVIATIONS`，不存在未记录的发现。收尾核验：`cargo fmt` ✅、`cargo clippy --workspace --all-targets` 0 error ✅、467 rust tests ✅、100 web tests ✅、`check_docs`（默认+strict）✅、`check_ui_contracts`/`gen_keymap`/`check_write_paths` ✅。
>
> **第二波补注（2026-10-30）**：Rust 侧 16 项整改的验收记录见上"验收记录（2026-10-30，独立复核）"；web p1 项当时仅收拢 15/31，已在第三波补完并加契约测试。

> **验收记录（2026-10-30，独立复核）**：两路独立验收 + 本机信号复测。信号：`clippy --workspace --all-targets` 0 error ✅（修前为本机编译失败）、`cargo test --workspace` 467 ✅ / 0 失败、`tsc --noEmit` ✅（已进 CI）、vitest 94 ✅、`vite dev` 已弃 `0.0.0.0` ✅。**结论：Rust 侧（server/model/omcom/ifc/sketch/CI/脚本）16 项整改全部为真修——实现、测试、留痕齐全，无表面修改、无回归；验收通过。** P0 `remove_containment`、箍筋 z、`set_parent` 原子化、`by_category` 重建、`delete_object` 级联、`ifc_class/tags` 补记 ModifyObject、`persist_failed`、导入根约束（含 omcom LoadSeries 与 `/api/omcom` 拦截）、回环默认监听、body 上限收敛、omcom 假成功簇（未知约束码/OOB 阻回回滚/阵列上限）、AMIR 撤重做转真调用、sketch 求解失败不回写、IFC 字符串态切分/转义/深度上限、CI `--all-targets`+typecheck、`check_write_paths` 防呆——逐条代码证据确认。**未通过项（web P1 仅修一半）**：`api.ts` 仍有 16 个裸 fetch 方法未收拢 `requestVoid`（series 族 7 个、catalog/feature 族 9 个；`SeriesManagerPage` 等三页未改），4 处 `.catch(() => {})` 残余（`AvailableComponentsPanel.tsx:41`、`ComponentEditorPage.tsx:86,1116,1123`）——"HTTP 500 当保存成功"在该页原地保留，进度文档"P1 全清"表述对本项偏乐观，宜改为"web API 层 15/31 收拢"。次要记录：① `resolve_param` 未知参数静默回 0 未修（文档未声称，未夸大）；② 几何防御上限四条（segments/sphere r=0/Pattern/breakdown 环）与 ifc_class 撤销、`persist_failed` 有实现缺专项测试；③ AMIR 侧生产代码哨兵语义未改、仅靠 server 拦截，前端不查 `result.success`（undo 空栈不显眼）；④ 近似内核 warning 为 message 字符串拼接而非独立字段，前端未读取；⑤ 文档"遗留"清单漏列 `App.tsx` 无 selector 全量订阅（未动，20Hz 重渲染仍在）；⑥ 212 条 clippy 警告积压仍在（CI 无 `-D warnings`）。

> 审计范围：全仓库 —— `crates/`（7 个 Rust crate，约 6.5 万行）、`web/`（React 前端，约 1.7 万行）、`proto/`、`scripts/`、`.github/` CI。
> 方法：4 路并行深度人工审计（server / model / 中间层 crate / 前端），加上客观质量信号（编译、测试、clippy、fmt、tsc）核验。每条发现标注严重级别与 `文件:行号`。

---

## 一、客观质量信号（本机实测）

| 项目 | 结果 |
|---|---|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo test --workspace` | ✅ 454 个测试全部通过（含 model 238 个） |
| `cargo clippy --workspace --all-targets` | ❌ **编译失败**：`crates/amir` 存在 4 处 `approx_constant`（deny-by-default），测试目标无法编译；另有 **212 条警告**积压（clone-on-Copy 类 73 条、`get(0)` 30+ 条等） |
| `web npx tsc --noEmit` | ✅ 通过（但**未纳入 CI**，`npm run build` 用 esbuild 转译不查类型） |
| `web vitest` | ✅ 94 个测试全部通过 |
| `vite build` | ✅ 通过 |
| CI (`ci.yml`) | Rust：fmt/clippy（无 `-D warnings`，且未用 `--all-targets`，致本机 clippy 失败在 CI 上不可见）/test；Web：仅 test+build，**无 typecheck、无 lint** |

**CI 门禁结论**：本地 clippy `--all-targets` 实际报错而 CI 不报，说明 CI 配置偏弱（缺 `--all-targets`）；前端类型检查完全依赖开发者自觉。

---

## 二、总体评级与统计

发现合计 **104 条**：

| 严重级别 | 条数 | 分布 |
|---|---|---|
| P0（真实数据损坏 bug） | **1** | model 自动化索引/Vec 失配 |
| P1（安全高危 / 功能错误 / 假成功） | **19** | server 3、model 6、omcom/amir/ifc 8、web 2 |
| P2（健壮性 / DoS / 数据丢失风险 / 性能债） | **42** | server 9、model 14、中间层 11、web 8 |
| P3（质量 / 可维护性 / 契约卫生） | **42** | — |

**一句话评价**：工程质量在"本地单人桌面工具"定位下属于**中上水平**——分层治理清晰（core proto / model / gen_rule / profile / omcom 命令层）、单写路径守卫脚本、原子写持久化、带深度上限的 undo 栈、充足测试（454+94 个）、文档治理远超同规模项目。**主要风险集中在两条主线**：

1. **信任边界缺失**：server 无认证、默认 `0.0.0.0`、100MB 通配 body；`/api/omcom` 与导入端点构成任意文件读取原语；validator"查不到就不查"、executor"做不了就报成功"的策略放大其影响。
2. **双份真相 / 静默吞错**：validator 与 executor、amir 与 omcom、IFC 导入与导出各自持有语义映射且无契约测试；前端 25 个 API 方法不检查响应 ok、错误被 `.catch(() => {})` 吞掉。两类问题都已实际产生 bug（见下）。

---

## 三、最高优先级发现（P0 + P1 关键项）

### 安全高危（server）
- **P1 无认证 + 默认绑定 `0.0.0.0`**（`crates/server/src/main.rs:58`、`lib.rs:4199`）：79 个路由全部匿名，同网段任意主机可读写整个工程与项目文件。→ 默认改 `127.0.0.1`；加 token 中间件、`TimeoutLayer` 与并发限制。
- **P1 导入路径参数完全不受限 → 任意文件读取**（`server/lib.rs:5495` `/api/catalog/import`、`lib.rs:6059` `/api/series/load`；`omcom/lib.rs:329` `LoadSeries`）：项目 open/save 走了 `resolve_project_path` 约束，但导入未复用。→ 统一根目录约束。
- **P2 `resolve_project_path` 未再校验最终路径（symlink 逃逸）**（`server/lib.rs:353-382`）。
- **P2 导出写 `/tmp` 且文件名秒级可预测（symlink 覆盖/篡改）**（`server/lib.rs:5470`、`6040`）。
- **P2 100MB body 上限全局生效**：仅 IFC 导入需要，现作用于全部端点（`server/lib.rs:234`）。

### 真实 bug（model）
- **P0 `remove_containment` 只清索引不删 Vec（:81-96）**：直接影响关系正确性，且行为与 `remove_connection` 不对称。**修复：仿 `remove_connection` 从 Vec 删除并重建双索引。**
- **P1 修改 `ifc_class`/`tags` 绕过 undo 直写对象**（`server/lib.rs:728`）：README"undo 覆盖所有 mutation"声明在此处不成立。
- **P1 `set_parent` 半事务**（`model/project.rs:87-105`）：回头失败时对象树已改、遗留陈旧关系。**修复：先删旧 containment 再改父指针。**
- **P1 catalog 的 `by_category` 索引 `#[serde(skip)]` 且加载后从不重建**（`model/catalog.rs:262-288`）：**分类浏览功能失效**。
- **P1 布尔/倒角是"占位近似内核"调用方无感知**（`project.rs:1576-1865` 等）：AABB 质心法切错斜墙、Chamfer/Fillet 与真实操作无关，静默产出错误几何进入渲染和出图链路。
- **P1 对象删除无级联清理 → 悬空引用**（`model/store.rs:61-70`）：子对象隐身（不在 children 也不在 roots）、关系/lifecycle/楼层引用均不清理。
- **P1 钢筋笼箍筋 z 坐标 bug**（`model/mesh_gen.rs:474-513`）：`let _z = k * stirrup_spacing;` 计算后丢弃，全部箍筋堆在 z≈0。**修复：`z: _z + r*st`（一行改动）**。

### 假成功与可信输入（omcom/amir）
- **P1 未知约束类型码静默映射为 `Fixed`、索引越界静默 no-op**（`omcom/lib.rs:2017-2038`、`:2090`）："做不了就当成功"，客户端无从发现；validator 对 entity/constraint 引用越界均不校验。

- **P1 `Array` 阵列 count 无上限**（`omcom/lib.rs:1506-1566`）：一次创建百万对象 → OOM。
- **P1 AMIR Undo/Redo 假命令上架**（`amir/generator.rs:958-1006`）：生成空 `Operation::Validate`，追加未被识别的命令既不撤销动作，还**顺手清空了 redo 栈**。
- **P2 sketch 求解失败仍回写数据中心化坐标**（`sketch/solver.rs:111-147`）：一次失败就把画好的草图拖变形。

### IFC 导入导出
- **P1 `;` 切行不识别字符串字面量内分号**（`ifc/lib.rs:36-57`）：含分号的名称会截断实体，且失败实体静默丢弃。
- **P1 无界递归 → 栈溢出**（`ifc/lib.rs:248` `parse_value`、`geometry.rs:523` 等）：自引用 `IFCBOOLEANRESULT` 爆栈。
- **P1 Profile 读写双方都偏离 IFC schema**（`ifc/lib.rs:1004`、`geometry.rs:523-1021`）：自产自销互通，**任何第三方真实 IFC 文件的矩形/圆截面导入静默无几何**。
- **P2 导出缺 schema 支架**（无 UNITASSIGNMENT/PRODUCTDEFINITIONSHAPE 等）、`IfcBuildingStorey` 属性错位、Name/Description 对调。

### 前端
- **P1 API 层错误处理三套标准并存**（`web/src/shared/api.ts` 约 25 个方法返回裸 Response 不查 `ok`；`MainToolbar.tsx:291`多处 `.catch(() => {})`、`SeriesManagerPage` 的 try/catch 对 fetch 4xx/5xx 从不触发）：**HTTP 500 被当成保存成功**，静默失败属"数据丢失"级体验问题，且现有 react-query hooks 写了却 **0 处 import 使用**。

---

## 四、分域详细清单

### 4.1 server（P1×3 / P2×9 / P3×10）
除上述安全项：
- **P2 构造工程文件可造成死循环/OOM**：`ancestors_of` 无 visited 环检测、`rebuild_children` 无拦环、`build_tree_node` 无限递归（`model/breakdown.rs:196`、`store.rs:29` + `server/lib.rs:574`）。
- **P2 自动保存失败仅 `eprintln!`、请求仍返回成功**（`server/mutation.rs:130-136`）：下次崩溃即丢数据。
- **P2 全局库（catalog）变更从不持久化**：重启即丢，仅靠显式导出（`omcom/lib.rs:2596+`）。
- **P2 持锁做同步磁盘 IO、无 Timeout/并发限制**：每次 mutation 持 project 写锁同步写数 MB JSON（`server/lib.rs` mutation 路径 + `mutation.rs:106`）。
- **P2 ReplaceProject undo = 全项目深拷贝×2**：反复 IFC 导入可致 undo 栈常驻数十 GB（`omcom/lib.rs:3640`、`server/lib.rs:4278`）。
- **P3**：`genrule_create` ~900 行巨函数；66 处 `unwrap/expect` 且无 `CatchPanicLayer`；catalog/handler 失败信息被吞；`import_ifc` 失败返 200、merge 同 id 静默覆盖；`list_projects` 全量解析每个工程、`get_project` 每请求克隆全项目。

### 4.2 model / core（P0×1 / P1×6 / P2×14 / P3×13）
除上述 P0/P1 核心项：
- **P2 几何防御性下限缺失**：`sphere_mesh` 半径 0 → NaN、segments 无上限（1e9 → OOM）（`mesh_gen.rs:165-205`）；`resolve_param` 表达式乘法*加法序错误且静默回 0（`feature_template.rs:556`）；Pattern 循环次数无上限（`project.rs:1766`）；零向量 normalize 未防护（`mesh_gen.rs:540`、`brep.rs:192`）。
- **P2** 全栈环检测：`ObjectStore` 与 `BreakdownStore::ancestors_of` 均允许/恐惧 parent 成环。
- **P2** 索引重复插入（重复 insert push 重复索引到多个 IndexedStore）；`linear_component` UV 与顶点数错配。全局 `RwLock<Option<Library>>` 与四处 `expect("LIBRARY not initialized")` panic。
- **架构**：`project.rs`（2506 行）、`seed_catalog.rs`（3635 行 god-file）按持久化/重建/实例化/级联拆分；辅助索引抽公共 `IndexedStore<T>`；几何缓存与逻辑数据分库（MagArc 共享），避免全量 clone。
- **测试**：238 个全通过、占比 27%——健康；盲区恰是 P0/P1 所在（containment 删除、删除级联、`set_parent` 换父、箍筋 z 分布、`resolve_param`）。

### 4.3 omcom / amir / sketch / ifc + 脚本 CI（P1×8 / P2×11 / P3×12）
除上面 P1 项：
- **P1 `"1+2*3" → 9`**（运算顺序 `feature_template.rs:556`）。
- **P2** omcom `lib.rs` 6969 行单文件，建议：按 `validator.rs / commands.rs / executor/{mod,objects,catalog,series,sketch} / result.rs` 拆分（单函数 `execute` 约 2340 行、52 分支）；未知错误码→错误枚举→amir 契约测试一并焊死。
- **P2** sketch 约束构建全程 `let _ =` 吞错、0/1-based 索引语义含混；求解无时限（ вызов 在持写锁下阻塞后端）。
- **P2/P3 若干 IFC 细节**：畸形行/重复 id 静默处理，`inf/nan` 流入文件；GlobalId 非 22 字符合法 base64；硬编码假时间戳。
- **CI**：web 无 typecheck/lint；clippy 无 `-D warnings` 且缺 `--all-targets`；`scripts/check_write_paths.py` 依赖单文件正则的静态扫描，**重构出子模块即静默失效**（扫描不到 handler 时应直接报错防呆）。`test-all.sh` 较 CI 弱（fmt/clippy 缺位）。`__pycache__` 已有 gitignore 覆盖 ✅。

### 4.4 web 前端（P1×2 / P2×8 / P3×7）
除上面 P1 项：
- **P2 `App.tsx` 无 selector 订阅整个 store**：`cursorWorld` 20Hz 更新驱动 App 全壳重渲染；另有 mount 双重初始化（stats/tree/undo/mesh 各请求两遍），全局 keydown 会吞 Ctrl+W/F 等浏览器快捷键。
- **P2 react-query 装而不用（死代码）**：已编写约 15 个带失效逻辑的 hooks 但 0 处引用，组件手写 fetch+手吞错误——错误处理失控正是这一点造成的。
- **P2 `ThreeViewer.tsx` 1823 行**、init effect ~780 行、用 `as any` 劫持 `three-viewcube ^0.0.10` 私有内部（包一升即碎）；1723 行文件缺 `sky/env` 资源 dispose。
- **P2 性能**：`ObjectTree` 无 memo + `includes` 查找每节点 O(n)，选中变化全树重渲染；数千对象时逐对象新建材构建。
- **P3 `dangerouslySetInnerHTML`**（`OmniPanel.tsx:796`）：当前转义健壮**无 XSS**，但建议改为 token 数组渲染消除隐患；dev server `--host 0.0.0.0` 暴露全局；`getFavorites` 枚举直接拆解（无校验 parse）；若干 index-as-key。
- **测试**：94 个纯函数测试质量优良，但 API 层/全局 store/渲染组件 0 测试——P1 处的 25 个方法若有一个响应 ok 判断的契约测试就会立刻拦住。

---

## 五、修复路线图建议

### 第一波（小改动、高收益，约 1–2 周）
1. **server**：默认监听 `127.0.0.1`（`main.rs:58`）；`catalog/import` 与 `series/load` 复用根路径约束；100MB body 上限只挂 IFC 导入路由；自动保存失败在响应上可见（`persist_failed` 标记）。
2. **model P0**：`remove_containment` 从 Vec 删除并重建索引（补回归测试）。
3. **一行修**：箍筋 z 坐标（`mesh_gen.rs:491`）。
4. **amir undo 假命令**：server 对 `amir-undo/redo` 检测转调 `exe.undo/redo`，或废弃该语义与前端直调 `/api/undo`。
5. **web**：`api.ts` 全部方法收拢 `request<T>()`；删除 `.catch(() => {})`；启用已写好的 react-query hooks。
6. **CI**：clippy 加 `--all-targets` + 修 amir 的 4 处 `approx_constant`（先让本机失败在 CI 可见）；web 加 `tsc --noEmit` step。
7. `ifc` 拼串解析改字符级状态机；`parse_value`/几何变换加递归深度上限与 visited 集。

### 第二波（结构性，约 1–2 月迭代）
1. 对象删除统一 `Project::delete_object` 级联清理 + `set_parent` 原子化——与 P0 同属"索引与级联"主题，一次重构可一并解决 CatalogStore 索引 rebuild。
2. omcom 拆文件 + executor 各分支返回 `Result`（ `fake-success` 全部消灭）+ validator 补越界检查；amir 契约改机器错误码。
3. 近似几何内核打"近似 flag"：Boolean/Chamfer/Fillet 返回 warning 或 Err，不再静默输出错误几何。
4. server：token 中间件、`TimeoutLayer`、persist 移出写锁（`spawn_blocking`）。
5. 前端拆 `ThreeViewer`/`ComponentEditorPage` 千行文件；App selector 拆分；补 api 层/appStore 契约测试。
6. CI 加 `clippy -D warnings` 白名单基线（学习前端 FRONTEND_BASELINE 的做法）；`tsc`/lint 进 CI；`check_write_paths.py` 加"扫描不到 handler"报警。

### 长期
- `project.rs` / `seed_catalog.rs` / `serve_lib.rs` 等 god-file 按域拆分；全局 `LIBRARY` 单例下沉到 Project 或显式注入。
- 接入真实三角网格 CSG 或 truck 布尔替代 AABB 近似内核。
- mesh 增量更新替换前端全量 `reloadScene`；共享 `IndexedStore<T>` 收编全部手工索引。

---

## 六、亮点确认（免得只记得问题）

- `Project::save` 是 tmp+rename 原子写；undo 栈 200 深度上限、切项目 `clear_history`、失败回滚 `fail_and_rollback` 架构成立。
- 锁纪律：锁序在注释中写明，虽 IO 时机不佳但未发现死锁环。
- `gen_rule/matrix.rs` 授权矩阵、"语义无感的 path_geom 底层几何内核"是好设计，注释持续给规范文档。
- 前端 `shared/`（layout/saveState/recentProjects）：版本化 + 防御性解析 + 完整单测，是教科书级防御性编码；三层 ErrorBoundary + resetKey 分区设计讲究；`ThreeViewer` 卸载清理链条完整（three.js 常见泄漏以外已服）。
- 文档治理（check_docs / check_write_paths / check_ui_contracts / gen_keymap 四脚本 + 分支/commit 钩子）远超同规模项目水平。
- 454 Rust + 94 前端测试全部通过，测试占比约 25–27%，运行时间合计 <1s。

---

*生成方式：GLM-5.3-Flash 代理四路并行审计 + 本机工具（clippy/test/fmt/tsc/vitest）交叉核验。各级别计数：P0=1，P1=19，P2=42，P3=42。*
