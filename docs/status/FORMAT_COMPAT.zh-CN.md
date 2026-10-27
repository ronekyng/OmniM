# OmniM 格式兼容性矩阵

**最后更新：2026-10-27**

> **状态诚实性说明**：本文档早期版本声称 IFC/STEP/DXF/GIS/glTF 均"完整支持"。那是愿景而非事实。本文档现在反映**代码的实际行为**。已对照 `crates/ifc/src/lib.rs` 与 `crates/server/src/lib.rs`（仓库中仅有的导入/导出代码路径）核实。

## 1. 支持矩阵（实际）

| 格式 | 导入 | 导出 | 状态 | 代码位置 |
|------|------|------|------|----------|
| **IFC 2x3 / IFC 4** | ✅ 部分支持 | ✅ 部分支持 | 已实现 — 手写解析器/写出器，实体覆盖不完整 | `crates/ifc/src/lib.rs`，REST：`/api/import/ifc`、`/api/export/ifc` |
| **JSON `.omnim`**（原生） | ✅ | ✅ | 已实现 — 主持久化格式 | `crates/server/src/lib.rs`、`crates/model/` |
| `.omnimlib` / `.omnimseries` | ✅ | ✅ | 已实现 — 目录/系列 JSON 文件 | `crates/model/src/library.rs`、`series_store.rs` |
| **STEP（AP203/AP242）** | ❌ | ❌ | 计划中 | — |
| **DXF / DWG** | ❌ | ❌ | 计划中 | — |
| **GIS（Shapefile / GeoJSON / GML / KML / GeoTIFF / LAS）** | ❌ | ❌ | 计划中 | — |
| **glTF / GLB** | ❌ | ❌ | 计划中 | — |
| **OBJ / FBX / STL** | ❌ | ❌ | 计划中 | — |
| **Navisworks / Revit / ArchiCAD / Tekla 原生格式** | ❌ | ❌ | 不计划（专有格式；请用其 IFC 导出） | — |

服务器中恰好只有两条导入/导出路径：IFC 和原生 JSON。不存在任何其他格式的代码。

## 2. IFC 支持（已实现）

### 实现说明

- **手写** STEP 物理文件（SPF）文本解析器 — 不依赖任何 IFC 库。
- 目标 schema：**IFC 2x3 / IFC 4**（同一代码路径；无 IFC 4x3 特有实体）。
- **导入**（`import_ifc_with_options`）：逐行实体解析、通过 `doc.get(ref)` 解析引用、可选的原点归一化、层级与属性集重建。
- **导出**（`export_ifc`）：基于 placement 的实体及实体/参数化几何（经 `IFCEXTRUDEDAREASOLID`/`IFCREVOLVEDAREASOLID` 的拉伸/旋转截面，网格经刻面 BRep），带 IfcGlobalId/名称/描述属性及压缩 GlobalId。

### 导入时实际加载的实体类型

仅当实体类型出现在 `import_ifc_with_options` 的 match 表中（`crates/ifc/src/lib.rs`）时才会创建对象。目前：

| IFC 实体 | 映射为 |
|----------|--------|
| IfcWall / IfcWallStandardCase | Wall |
| IfcBeam / IfcBeamStandardCase | Beam |
| IfcColumn / IfcColumnStandardCase | Column |
| IfcSlab / IfcSlabStandardCase / IfcPlate / IfcPlateStandardCase | Slab |
| IfcDoor | Door |
| IfcWindow | Window |
| IfcStair / IfcStairFlight | Stair |
| IfcRoof | Roof |
| IfcRoad | Road |
| IfcBridge | Bridge |
| IfcBuilding | Building |
| IfcBuildingStorey | BuildingStorey |
| IfcSite | Site |
| IfcSpace | Space |
| IfcMember / IfcMemberStandardCase、IfcFooting、IfcRamp、IfcFurnishingElement / IfcFurniture、IfcBuildingElementProxy | Generic |

间接消费的结构/语义实体：

- **IfcRelAggregates**、**IfcRelContainedInSpatialStructure** → 父子包含层级
- **IfcRelDefinesByProperties** + **IfcPropertySet** + **IfcPropertySingleValue** → 原生 PropertySet
- **IfcLocalPlacement** / **IfcAxis2Placement3D** / **IfcCartesianPoint** → 位置提取与几何平移
- **IfcProductDefinitionShape** / **IfcShapeRepresentation** → 几何 item

*未处理*：所有其他实体（IfcSystem 分组、IfcZone、IfcMaterial/材料层、IfcRelAssociatesMaterial、工程量、分类、高级 BRep/CSG/扫掠/组合几何、带变换的映射项等）会解析进原始实体表，但不做转换。

### 几何覆盖

转换器位于 `crates/ifc/src/geometry.rs`；覆盖有限——主要是拉伸截面实体与简单 item。完整 IFC 几何（高级 BRep、CSG、扫掠组合、带变换的映射项）**不支持**。

## 3. 未实现的格式 — 计划顺序

Phase 11 为归属路线阶段（[ROADMAP.md](ROADMAP.md)）。建议顺序：

1. **拓宽 IFC 覆盖**（在现有阶段内）：更多几何类型（映射项、精化截面）、材料/工程量/分类映射、IFC 4x3 实体 → 将 IFC 升级为 2x3/4 的 ✅ 完整支持。
2. **glTF/GLB 导出** — 面向 three.js 可视化的低成本收益；网格已存在于服务端。
3. **DXF 导入**（通过开源 crate 支持基础 2D 实体）— 供草图工作流使用。
4. **STEP AP242** — 需要正确的 CAD 内核边界；在承诺之前评估现有 crate。
5. **GIS（先 GeoJSON，再 Shapefile）** — 推迟至 Phase 8 土木领域工作之后。
6. OBJ/FBX/STL — 仅按需；当前未排期。

## 4. 专有 BIM 格式

| 格式 | 支持 | 备注 |
|------|------|------|
| Revit (.rvt) | ❌ 不计划 | 直接访问需要 Autodesk SDK/许可证 — 请用 Revit 的 IFC 导出 |
| ArchiCAD (.pln)、Tekla (.tpl) | ❌ 不计划 | 同上 — 请用 IFC 导出 |

## 5. 已移除的内容

对尚未实现格式描述完整几何管线、往返保真度目标和导入性能指标的章节已**移除**，直到有代码支持它们。随各格式落地，将以真实基准测试数据恢复。
