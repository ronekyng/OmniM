# Component Inventory & Research Coverage

> 更新日期：2026-08-07（Phase 1-4 调研完成：SolidWorks / Bentley / Tekla / E3D Instrumentation）

## Legend
- ✅ **Researched** — 有专门平台分析文档佐证
- ⚠️ **Inferred** — 未直接调研，但与已调研构件结构相似可推断
- ❌ **Unresearched** — 无平台分析，无法推断
- 📝 **In proto** — ObjectType 存在于 proto/omnim.proto
- 📋 **In DOMAINS.md only** — 仅在 DOMAINS.md 列出，未进 proto

---

## 1. Building Domain

| DOMAINS.md Object | Proto | GenRuleHint | Researched? | Notes |
|---|---|---|---|---|
| **Site** | 📝 | Primitive（容器） | ✅ | site_terrain_components.md §1-8（六平台场地/地形综合调研，空间容器+地形+坐标系统） |
| **Building** | 📝 | Primitive（容器） | ✅ | site_terrain_components.md §7.1（Site→Building→Storey 层级） |
| **BuildingStorey** | 📝 | Primitive（容器） | ✅ | site_terrain_components.md §7.1（空间层级组织） |
| **Space** | 📝 | Primitive（容器） | ✅ | site_terrain_components.md §7.1（BuildingStorey 下的空间单元） |
| Zone | 📋 | — | ❌ | Not in proto, unresearched |
| **Wall** | 📝 | LayerExtrusion | ✅ | revit_components.md §5.1 |
| **Door** | 📝 | BooleanSubtract | ✅ | revit_components.md §2.2 |
| **Window** | 📝 | BooleanSubtract | ✅ | revit_components.md §2.2 |
| **Roof** | 📝 | LayerExtrusion | ✅ | revit_components.md §5.3 |
| Ceiling | 📋 | — | ❌ | Not in proto, unresearched |
| CurtainWall | 📋 | — | ❌ | Not in proto, unresearched |
| **Stair** | 📝 | Assembly | ✅ | revit_components.md §2.2 (OST_Stairs) |
| Ramp | 📋 | — | ❌ | Not in proto, unresearched |
| **Beam** | 📝 | LinearComponent | ✅ | revit_components.md §6 + tekla_components.md §2.1（Beam 类，截面扫掠）— 跨平台验证 |
| **Column** | 📝 | LinearComponent | ✅ | revit_components.md §6 + tekla_components.md §2.2（Tekla 建议合并入 Beam 作用途标签） |
| **Slab** | 📝 | LayerExtrusion | ✅ | revit_components.md §5.2 |
| **Footing** | 📋 | Primitive | ✅ | tekla_components.md §5.1（ContourPlate 独立基础 / Beam 条形基础） |
| **Pile** | 📋 | LinearComponent | ✅ | tekla_components.md §5.2（Beam 类 + 桩截面） |
| **Brace** | 📋 | LinearComponent | ✅ | tekla_components.md §4.1（Beam + Brace profile） |
| **Truss** | 📋 | Assembly | ✅ | tekla_components.md §4.2（多 Beam 杆件装配） |
| **Rebar** | 📋 | LinearComponent | ✅ | tekla_components.md §6（RebarGroup / RebarSet，圆截面沿路径扫掠 + 阵列） |
| **Connection** | 📋 | Assembly | ✅ | tekla_components.md §3（Tekla Connection 组件系统 = OmniM Assembly 工业原型） |
| **Duct** | 📝 | LinearComponent | ✅ | revit_components.md §7 |
| **Pipe** | 📝 | LinearComponent | ✅ | revit_components.md §7 |
| **CableTray** | 📝 | LinearComponent | ✅ | revit_components.md §7 + e3d_instrumentation_components.md §3.1（直段=LayerExtrusion / 弯头=LinearComponent） |
| **Valve** | 📝 | Primitive | ✅ | comparative_analysis.md §4（手动阀 Primitive；调节阀见 ControlValve→Assembly） |
| Damper | 📋 | — | ❌ | Not in any research |
| **Pump** | 📝 | Primitive | ✅ | comparative_analysis.md §4 |
| Fan | 📋 | — | ❌ | Not in any research |
| **AirTerminal** | 📝 | Primitive | ⚠️ | Inferred from Revit MEP |
| **Chiller** | 📝 | Assembly | ✅ | E3D equipment pattern |
| Boiler | 📋 | — | ❌ | Not in any research |
| **AHU** | 📝 | Assembly | ⚠️ | Inferred from E3D equipment pattern |
| **Transformer** | 📝 | Assembly | ⚠️ | Inferred from E3D equipment pattern |
| Switchboard | 📋 | — | ❌ | Not in any research |
| Luminaire | 📝 | Primitive | ❌ | Not in any research |
| Fixture | 📋 | — | ❌ | Not in any research |

**Building totals**: 37 listed, 22 in proto, 15 in DOMAINS.md only
**Researched**: 24 ✅, 3 ⚠️, 10 ❌

---

## 2. Civil Domain

| DOMAINS.md Object | Proto | GenRuleHint | Researched? | Notes |
|---|---|---|---|---|
| HorizontalAlignment | 📋 | — | ✅ | civil3d_components.md §4.1 (Alignment) |
| VerticalAlignment | 📋 | — | ✅ | civil3d_components.md §4.1 (Profile) |
| Profile | 📋 | — | ✅ | civil3d_components.md §4.1 |
| Station | 📋 | — | ⚠️ | Inferred from Alignment |
| **Road** | 📝 | LinearComponent | ✅ | civil3d_components.md (Corridor) + bentley_openroads_components.md §1（走廊模板=LinearComponent）— 跨平台验证 |
| Lane | 📋 | — | ✅ | civil3d_components.md (BasicLane subassembly) |
| Shoulder | 📋 | — | ✅ | civil3d_components.md (ShoulderExtend) |
| PavementLayer | 📋 | — | ✅ | civil3d_components.md §2.2 (Pavement structure) |
| Median | 📋 | — | ✅ | civil3d_components.md (roadway subassembly) |
| Curb | 📋 | — | ✅ | civil3d_components.md (BasicCurb) |
| **Bridge** | 📝 | LinearComponent | ✅ | bentley_openroads_components.md §2-3（OBM 上部/下部结构）— 专项分析完成 |
| **Pier** | 📝 | Assembly | ✅ | bentley_openroads_components.md §3.1（Cap=LinearComponent 变截面 / Column=LinearComponent / Footing=Primitive / Pile=Primitive） |
| **Abutment** | 📝 | Assembly | ✅ | bentley_openroads_components.md §3.2（Stem/Backwall/Wingwall=LinearComponent / Footing=Primitive） |
| **Girder** | 📋 | LinearComponent | ✅ | bentley_openroads_components.md §2.1（OBM Beam Definition，6 种 Variation + Splice 分段变截面） |
| **Deck** | 📋 | LinearComponent | ✅ | bentley_openroads_components.md §2.2（OBM Deck Template 多层截面沿桥跨扫掠） |
| **Bearing** | 📋 | Primitive / Assembly | ✅ | bentley_openroads_components.md §2.3（单一参数化实体=Primitive；复杂多部件=Assembly） |
| **ExpansionJoint** | 📋 | Assembly | ✅ | bentley_openroads_components.md §2.4（多型材 + 锚固 + 密封件） |
| **Tunnel** | 📝 | LinearComponent | ✅ | bentley_openroads_components.md §4（OTD Tunnel Template 沿 Corridor 扫掠） |
| **Lining** | 📋 | LinearComponent | ✅ | bentley_openroads_components.md §4.1（多层截面：开挖/支护/衬砌；变截面=多 Tunnel Unit） |
| **Portal** | 📋 | Assembly | ✅ | bentley_openroads_components.md §4.2（仰坡/挡墙/截水沟/开挖体；开挖=BooleanSubtract） |
| **Shaft** | 📋 | LinearComponent | ✅ | bentley_openroads_components.md §4.3（环形截面沿垂直路径 + 底板） |
| **Segment** | 📋 | Primitive | ✅ | bentley_openroads_components.md §4.4（TBM UTR 单管片参数化实体） |
| **Ring (TBM)** | 📋 | Assembly | ✅ | bentley_openroads_components.md §4.4（多 Segment + Dowel 组合）— 新增 |
| **CrossPassage** | 📋 | Assembly + BooleanSubtract | ✅ | bentley_openroads_components.md §4.5（横通道=LinearComponent；与主隧道连通=BooleanSubtract） |
| **Dam** | 📝 | LinearComponent | ✅ | bentley_openroads_components.md §5.4（坝体断面沿坝轴线扫掠；多段 Template Drop 变截面）— GenRuleHint 由 Primitive 修正为 LinearComponent |
| **Culvert** | 📝 | LinearComponent | ⚠️ | civil3d_components.md mentions Box Culvert as pipe shape；Bentley 未专项分析 |
| **Channel** | 📋 | LinearComponent | ✅ | bentley_openroads_components.md §5.1（梯形/矩形/U 形断面沿 alignment 扫掠） |
| **Drainage** | 📋 | Assembly | ✅ | bentley_openroads_components.md §5.2（Pipe=LinearComponent / Structure=Primitive / 网络拓扑） |
| **RetentionBasin** | 📋 | LinearComponent + BooleanSubtract | ✅ | bentley_openroads_components.md §5.3（Pond 控制线扫掠 + 与地形求挖填） |
| WaterPipe | 📋 | LinearComponent | ⚠️ | Inferred from civil3d PipeNetwork + Bentley Drainage & Utilities |
| SewerPipe | 📋 | LinearComponent | ⚠️ | Inferred from civil3d PipeNetwork + Bentley Drainage & Utilities |
| StormPipe | 📋 | LinearComponent | ⚠️ | Inferred from civil3d PipeNetwork + Bentley Drainage & Utilities |
| **Manhole** | 📝 | Primitive | ✅ | civil3d_components.md §6.1 |
| UtilityPole | 📋 | — | ❌ | Not in any research |
| DuctBank | 📋 | — | ❌ | Not in any research |
| **Terrain** | 📋 | TerrainModel（独立 ObjectType） | ✅ | site_terrain_components.md §2-3（Civil3D/Bentley TIN 曲面+完整断裂线+边界支持）+ bentley_openroads_components.md §6.1 |
| **CutVolume** | 📋 | BooleanSubtract | ✅ | site_terrain_components.md §7.3（两 TIN 曲面布尔差，Civil3D 三种算法）+ bentley_openroads_components.md §6.2 |
| **FillVolume** | 📋 | BooleanSubtract | ✅ | site_terrain_components.md §7.3（两 TIN 曲面布尔差）+ bentley_openroads_components.md §6.2 |
| **Grading** | 📋 | 操作（基于 TerrainModel） | ✅ | site_terrain_components.md §7.3（Civil3D Grading Group 完整平整系统；Bentley 走廊模板半平整） |
| **Slope** | 📋 | LinearComponent + Terrain 交互 | ✅ | site_terrain_components.md §7.3（Civil3D Daylight 自动求交；边坡与地形自动连接）+ bentley_openroads_components.md §6.3 |
| **RetainingWall** | 📝 | Primitive | ✅ | civil3d_components.md (RetainingWall subassembly) |
| **PipeCivil** | 📝 | LinearComponent | ✅ | civil3d_components.md §6 (Pipe Network) |

**Civil totals**: 41 listed, 10 in proto, 31 in DOMAINS.md only
**Researched**: 34 ✅, 5 ⚠️, 2 ❌

---

## 3. Plant Domain

| DOMAINS.md Object | Proto | GenRuleHint | Researched? | Notes |
|---|---|---|---|---|
| **Tank** | 📝 | Primitive | ✅ | aveva_e3d_primitives.md §7.1 (Vertical_Vessel) |
| Vessel | 📋 | — | ✅ | aveva_e3d_primitives.md §7 (design templates) |
| **Reactor** | 📝 | Assembly | ✅ | aveva_e3d_primitives.md (EQUI equipment) |
| **HeatExchanger** | 📝 | Primitive | ✅ | aveva_e3d_primitives.md (EQUI equipment) |
| **Pump** | 📝 | Primitive | ✅ | aveva_e3d_primitives.md (EQUI template) |
| Compressor | 📝 | Primitive | ✅ | aveva_e3d_primitives.md (EQUI equipment) |
| **Filter** | 📋 | Assembly / Primitive | ✅ | e3d_instrumentation_components.md §5.1（壳体+接管+滤芯+盖板；简单=Primitive） |
| **Pipe** | 📝 | LinearComponent | ✅ | aveva_e3d_primitives.md §4.2 (BRANCH) + e3d_instrumentation_components.md §4.2（ProcessLine 元件序列模式） |
| PipeSegment | 📋 | — | ✅ | aveva_e3d_primitives.md §4.1 (PIPE SEGMENT) |
| Elbow | 📋 | — | ✅ | aveva_e3d_primitives.md §4.1 (ELBO) |
| Tee | 📋 | — | ✅ | aveva_e3d_primitives.md §4.1 (TEE) |
| Reducer | 📋 | — | ✅ | aveva_e3d_primitives.md §4.1 (REDUCER) |
| Flange | 📋 | — | ✅ | aveva_e3d_primitives.md §4.1 (FLAN) |
| Nozzle | 📋 | — | ✅ | aveva_e3d_primitives.md §2.1 (NOZZ) |
| **Valve** | 📝 | Primitive | ✅ | aveva_e3d_primitives.md §4.1 (VALV)；手动阀=Primitive |
| **ControlValve** | 📋 | Assembly | ✅ | e3d_instrumentation_components.md §2.4（阀体+执行机构+定位器+附件）— 建议从 Valve 细分 |
| **Fitting** | 📝 | Primitive | ✅ | aveva_e3d_primitives.md §4.2 (all pipe fittings) |
| **StructuralSteel** | 📝 | Primitive | ✅ | aveva_e3d_primitives.md §5.1 (GENSEC) |
| Platform | 📋 | — | ✅ | aveva_e3d_primitives.md §5.1 (PANE) |
| **Stair** | 📝 | Assembly | ✅ | aveva_e3d_primitives.md §5.1 (STAIR) |
| Ladder | 📋 | — | ✅ | aveva_e3d_primitives.md §5.1 (LADDER) |
| Handrail | 📋 | — | ✅ | aveva_e3d_primitives.md §5.1 (HANDR) |
| **Support** | 📋 | Assembly | ✅ | e3d_instrumentation_components.md §5.2（吊杆+弹簧罐+U卡+管托 目录标准件拼装） |
| **Instrument** | 📝 | Assembly | ✅ | e3d_instrumentation_components.md §2.1（壳体+接管+显示头 Primitive 组合 + Tag/Loop 逻辑） |
| **Sensor** | 📋 | Primitive / Assembly | ✅ | e3d_instrumentation_components.md §2.2（简单=Primitive；流量计=Assembly） |
| **Transmitter** | 📋 | Assembly | ✅ | e3d_instrumentation_components.md §2.3（壳体+显示头+过程接口+格兰+端子排） |
| **JunctionBox** | 📋 | Assembly | ✅ | e3d_instrumentation_components.md §2.5（外壳+端子排+格兰+导轨） |
| **ProcessLine** | 📋 | LinearComponent | ✅ | e3d_instrumentation_components.md §4.2（直管段+弯头+阀门元件序列；Line Number 编码） |
| System | 📋 | — | ⚠️ | Inferred: 逻辑组织容器（无几何），E3D 层级推断 |
| Unit | 📋 | — | ⚠️ | Inferred: 逻辑组织容器（无几何） |
| Area | 📋 | — | ⚠️ | Inferred: 逻辑组织容器（无几何） |
| PipingSpec | 📋 | — | ✅ | aveva_e3d_primitives.md §6 (Piping Specification) |
| EquipmentSpec | 📋 | — | ✅ | aveva_e3d_primitives.md §6 (Design Templates) |
| **CableTray** | 📝 | LayerExtrusion / LinearComponent | ✅ | e3d_instrumentation_components.md §3.1（直段=LayerExtrusion U/C 多层 / 弯头=LinearComponent / 三通十字=Assembly） |
| **Conduit** | 📋 | LinearComponent | ✅ | e3d_instrumentation_components.md §3.2（圆形封闭截面沿中心线） |
| **Panel** | 📋 | Assembly | ✅ | e3d_instrumentation_components.md §3.3（柜体+母线+断路器+端子排；面板开孔=BooleanSubtract） |
| **MotorControlCenter** | 📋 | Assembly | ✅ | e3d_instrumentation_components.md §3.4（多柜体+隔室+单元） |
| **Cable** | 📋 | LinearComponent | ✅ | e3d_instrumentation_components.md §3.5（单芯=圆截面 / 多芯=多层截面沿中心线）— 新增 |
| **Loop** | 📋 | 无几何（逻辑对象） | ✅ | e3d_instrumentation_components.md §4.1（回路关联对象；建议 OmniM 新增 LogicalObject 层）— 新增 |

**Plant totals**: 39 listed, 11 in proto, 28 in DOMAINS.md only
**Researched**: 36 ✅, 3 ⚠️, 0 ❌

---

## 4. Mechanical Domain

| DOMAINS.md Object | Proto | GenRuleHint | Researched? | Notes |
|---|---|---|---|---|
| **Assembly** | 📝 | Assembly | ✅ | solidworks_components.md §1 — Assembly (.sldasm) |
| SubAssembly | 📋 | — | ✅ | solidworks_components.md §1 — nested assembly |
| **Part** | 📝 | Primitive | ✅ | solidworks_components.md §1 — Part (.sldprt) |
| Configuration | 📋 | — | ✅ | solidworks_components.md §1.1 — Configuration variant |
| Variant | 📋 | — | ✅ | solidworks_components.md §5 — Design Table variant |
| **Gear** | 📝 | Primitive | ✅ | solidworks_components.md §5 — Toolbox Gears（官方明确非真渐开线，仅设计示意） |
| **Shaft** | 📝 | Primitive | ⚠️ | 前版误标 Toolbox Keyed Shaft（官方 Toolbox 不含轴）；推断自 Part Revolve 特征 |
| **Bearing** | 📝 | Primitive | ✅ | solidworks_components.md §5 — Toolbox Bearings（示意几何，无精确细节，不适合应力分析） |
| Coupling | 📋 | — | ❌ | 前版误标 Toolbox（官方 Toolbox 不含联轴器）；未调研 |
| **Spring** | 📝 | Primitive | ⚠️ | 前版误标 Toolbox（官方不含弹簧）；推断自 Sweep along Helix 特征 |
| **Seal** | 📝 | Primitive | ⚠️ | 前版误标 Toolbox（官方仅 Grooves 工具加 O 型圈沟槽，非密封件库）；推断自 Part Revolve |
| Bushing | 📋 | — | ⚠️ | 前版误标 Toolbox Bushing（官方仅 Jig bushings 钻模套管，非通用衬套）；推断自 Part Revolve |
| Bolt | 📋 | — | ✅ | solidworks_components.md §5 — Toolbox Bolts |
| Nut | 📋 | — | ✅ | solidworks_components.md §5 — Toolbox Nuts |
| Washer | 📋 | — | ✅ | solidworks_components.md §5 — Toolbox Washers |
| Screw | 📋 | — | ✅ | solidworks_components.md §5 — Toolbox Screws |
| Rivet | 📋 | — | ❌ | 前版误标 Toolbox（官方 Toolbox 不含铆钉）；未调研 |
| Pin | 📋 | — | ✅ | solidworks_components.md §5 — Toolbox Pins |
| ThreadedInsert | 📋 | — | ✅ | solidworks_components.md §5 — Toolbox PEM® inserts（自紧螺纹插件） |
| RetainingRing | 📋 | — | ✅ | solidworks_components.md §5 — Toolbox Retaining rings（挡圈，新增） |
| Joint | 📋 | — | ✅ | solidworks_components.md §6 — Mates (Hinge/UniversalJoint) |
| Constraint | 📋 | — | ✅ | solidworks_components.md §6 — 21 类 Mates（标准 8 + 高级 6 + 机械 7） |
| Axis | 📋 | — | ✅ | solidworks_components.md §9 — Reference Geometry Axis |
| KinematicPair | 📋 | — | ⚠️ | Inferred from Gear/RackPinion/Screw 机械配合 |
| Actuator | 📋 | — | ❌ | Not in SOLIDWORKS core (electrical/pneumatic) |
| Hole | 📋 | — | ✅ | solidworks_components.md §3 — Hole Wizard（7 类孔：Counterbore/Countersink/Hole/Tap/Legacy/Slots） |
| Pocket | 📋 | — | ⚠️ | Inferred from Extruded Cut |
| Slot | 📋 | — | ✅ | solidworks_components.md §3 — Hole Wizard Slots（官方明确含槽口） |
| Chamfer | 📋 | — | ✅ | solidworks_components.md §3 — Chamfer feature |
| Fillet | 📋 | — | ✅ | solidworks_components.md §3 — Fillet feature（8 子类型） |
| Thread | 📋 | — | ⚠️ | solidworks_components.md §0 — 官方文档未明确独立 Thread 特征 API；Cosmetic Thread 标注 |
| PMI | 📋 | — | ✅ | solidworks_components.md §8 — MBD/DimXpert/GD&T (ASME Y14.5.1M) |
| **Motor** | 📝 | Primitive | ⚠️ | Inferred from Part（电气/气动，SOLIDWORKS 核心无专门分析） |
| Encoder | 📋 | — | ❌ | Not in SOLIDWORKS |
| Sensor | 📋 | — | ❌ | Not in SOLIDWORKS |
| Housing | 📋 | — | ⚠️ | Inferred from Part (multi-body) |
| Connector | 📋 | — | ✅ | solidworks_components.md §6 — Mate Reference |
| **Fastener** | 📝 | Primitive | ✅ | solidworks_components.md §5 — Toolbox (Bolt+Nut+Washer+Screw+Pin；官方不含 Rivet) |

**Mechanical totals**: 38 listed, 8 in proto, 30 in DOMAINS.md only
**Researched**: 24 ✅, 9 ⚠️, 5 ❌ (Coupling, Rivet, Actuator, Encoder, Sensor — 官方 Toolbox 不含或 SOLIDWORKS 核心无)

> **重要更正**：前版基于臆造的 Toolbox 清单将 Shaft/Coupling/Spring/Seal/Bushing/Rivet 误标为 ✅。官方验证后 Toolbox 仅含 15 类硬件（Bolts/Nuts/Washers/Screws/Pins/Bearings/Gears/Cams/Sprockets/Timing belt pulleys/Retaining rings/Jig bushings/PEM inserts/Structural shapes/Unistrut），不含铆钉/密封件/弹簧/键/联轴器/衬套/管件。详见 solidworks_components.md §0、§5。

---

## 5. Summary

| Domain | Total Listed | In Proto | Only in DOMAINS.md | Researched ✅ | Inferred ⚠️ | Unresearched ❌ |
|--------|------------|---------|-------------------|-------------|------------|---------------|
| **Building** | 37 | 22 | 15 | 20 | 7 | 10 |
| **Civil** | 41 | 10 | 31 | 34 | 5 | 2 (UtilityPole, DuctBank) |
| **Plant** | 39 | 11 | 28 | 36 | 3 | 0 |
| **Mechanical** | 38 | 8 | 30 | 24 | 9 | 5 |
| **Total** | **155** | **51** | **104** | **114** | **24** | **17** |

> 调研覆盖率：114/155 = **73.5%** 已研究（SolidWorks 官方重新调研后修正了 6 处臆造的 Toolbox 标记，覆盖率较修正前 78.3% 略降）；剩余 24 个 ⚠️ 为结构相似推断，17 个 ❌ 真正未研究。

## Platform Research Map

| Platform | Domain | Document | Components Covered |
|----------|--------|----------|-------------------|
| ✅ Revit 2027 API | Building | `revit_components.md` | Wall, Floor, Roof, Beam, Column, Brace, Footing, Door, Window, Stair, Duct, Pipe, CableTray, GenericForm |
| ✅ Civil 3D 2026 | Civil | `civil3d_components.md` | Subassembly, Corridor, Alignment, Profile, Assembly, Pavement, Pipe Network, Parts List, SAC |
| ✅ AVEVA E3D 2.1 | Plant | `aveva_e3d_primitives.md` | EQUI, BOX, CYLI, CONE, SNOUT, DISH, TORUS, BRANCH, PIPE, ELBO, TEE, VALV, FLAN, GENSEC, PANE, Catalog/Spec |
| ✅ SOLIDWORKS 2026 | Mechanical | `solidworks_components.md` | Part, Assembly, SubAssembly, Configuration/Design Table, FeatureManager 特征树 (Extrude/Revolve/Sweep/Loft/Boundary/Hole Wizard/Fillet 8 子类型/Chamfer/Pattern), Toolbox 15 类官方硬件 (Bolt/Nut/Washer/Screw/Pin/Bearing/Gear/Cam/Sprocket/Timing Pulley/Retaining Ring/Jig Bushing/PEM Insert/Structural Shape/Unistrut), 21 类 Mates, MBD/DimXpert/GD&T, Weldments, Sheet Metal, Multibody Parts（**官方渠道重新调研**） |
| ✅ Bentley OpenBridge/OpenRoads/OpenTunnel | Civil (detail) | `bentley_openroads_components.md` | Girder (I/U/Box), Deck, Bearing, ExpansionJoint, Pier, Abutment, Lining, Portal, Shaft, Segment, Ring, CrossPassage, Channel, Drainage, RetentionBasin, Dam, Terrain, CutVolume, FillVolume, Slope, Corridor/Linear/Surface Template, Civil Cell, Parametric Cell |
| ✅ Tekla Structures 2025/2026 | Structural (Building detail) | `tekla_components.md` | Beam, Column, ContourPlate, PolyBeam, Brep, LoftedPlate, Connection/Component/Detail/Seam, Footing, Pile, PileCap, Rebar (SingleRebar/RebarGroup/CircleRebarGroup/CurvedRebarGroup/RebarMesh/RebarStrand/RebarSet), BoltGroup, Weld, BooleanPart, Fitting, CutPlane, EdgeChamfer, Profile Library |
| ✅ AVEVA Instrumentation / E3D Electrical | Plant (detail) | `e3d_instrumentation_components.md` | Instrument, Sensor, Transmitter, ControlValve, JunctionBox, CableTray, Conduit, Panel, MCC, Cable, Loop, ProcessLine, Filter, Support, Tag/ISA 体系, 逻辑-物理双视图架构 |

---

## 6. 跨平台 GenRule 验证矩阵

以下构件经多平台调研交叉验证 GenRule 映射的稳健性：

| 构件 | Revit | Civil3D | E3D | Bentley | Tekla | SolidWorks | 收敛 GenRule |
|---|---|---|---|---|---|---|---|
| Beam | ✅ Beam 族 | — | GENSEC | ✅ OBM Girder | ✅ Beam 类 | — | **LinearComponent** |
| Column | ✅ Column 族 | — | CYLI/GENSEC | ✅ Pier Column | ✅ Beam(垂直) | — | **LinearComponent** |
| Pipe | ✅ Pipe 族 | ✅ PipeNetwork | ✅ BRANCH | — | — | — | **LinearComponent** |
| Road | — | ✅ Corridor | — | ✅ ORD Corridor | — | — | **LinearComponent** |
| Wall | ✅ Wall 族 | ✅ RetainingWall | PANE | — | — | — | **LayerExtrusion** |
| Slab/Deck | ✅ Floor | ✅ Pavement | PANE | ✅ OBM Deck | ContourPlate | — | **LayerExtrusion/LinearComponent** |
| Valve | ✅ FamilyInstance | ✅ PipePart | ✅ VALV | — | — | — | **Primitive**（手动）/ Assembly（调节） |
| Stair | ✅ OST_Stairs | — | ✅ STAIR | — | — | — | **Assembly** |
| Connection | — | — | — | — | ✅ Connection | ✅ Mate | **Assembly** |
| Rebar | — | — | — | — | ✅ RebarGroup | — | **LinearComponent** |

**核心结论**：LinearComponent 是跨平台收敛度最高的 GenRule（Beam/Column/Pipe/Road/Girder/Lining/Dam/Slope 均收敛），验证其作为基础设施与结构领域核心几何范式的定位。
