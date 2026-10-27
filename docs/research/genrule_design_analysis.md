# GenRule 体系设计分析 — 基于跨平台调研的反思

> 分析日期：2026-08-06
> 基于：civil3d_components.md, revit_components.md, aveva_e3d_primitives.md, comparative_analysis.md
> 对照代码：gen_rule/mod.rs, linear_component.rs, layer_extrusion.rs, assembly.rs, boolean_subtract.rs, catalog.rs

---

## 1. 项目现状概述

### 1.1 GenRule 的双层架构

代码中 GenRule 有两个层次，弃用状态不同（见 DATA_MODEL.zh-CN.md:584-592）：

| 层次 | 内容 | 状态 |
|------|------|------|
| **GenRule 作为分类** | `GenRuleHint` 枚举，`primary_gen_rule()` 映射 | ✅ **活跃**——用于模板选择与系列选择 |
| **GenRule 作为生成** | `generate_geometry()` 调用硬编码构件函数 | ❌ **已弃用**——被默认模板注册中心 + 目录特征树模板取代 |

**关键结论**：分析文档中关于"重构 GenRule"的建议，需要明确是在分类层还是生成层。当前生成层已弃用，新路径是 `default_catalog_component()` → `CatalogStore` → `FeatureTreeTemplate` → Feature Engine → BRep。

### 1.2 五种 GenRule 模块现状

| 模块 | 文件 | 状态 | 说明 |
|------|------|------|------|
| Primitive | 无独立模块 | 通过 Catalog 驱动 | 基本体由 `ParametricGeometry` + Feature Engine 处理 |
| LayerExtrusion | `layer_extrusion.rs` (341行) | 仅便捷函数 | `build_wall()`/`build_slab()` 等，用于测试 |
| LinearComponent | `linear_component.rs` (911行) | 完整引擎 | 支持变截面插值、Frenet 框架、多层截面 |
| Assembly | `assembly.rs` (352行) | 完整引擎 | 支持子组件 + Transform，含领域便捷函数 |
| BooleanSubtract | `boolean_subtract.rs` (565行) | 完整引擎 | 基于 mesh 的包围盒检测 + 边界边封闭 |

### 1.3 关键映射关系

`primary_gen_rule()` (mod.rs:68-110) 的 ObjectType → GenRuleHint 映射：
- `Beam | Column → LinearComponent`（固定截面模式，`linear_component.rs` 的 `build_beam()` 即为该模式实现）
- `Wall | Slab | Floor | Roof → LayerExtrusion`
- `Door | Window → BooleanSubtract`
- `Road | Bridge | Tunnel | PipePlant → LinearComponent`
- `Tank | Pump | Valve → Primitive`
- `Ahu | Chiller | Reactor → Assembly`

---

## 2. 对分析文档中建议的评估

### 2.1 ✅ 建议一：LinearComponentGen 拆分为三种模式

**分析文档建议**：固定截面 / 变化横断面 / 元件序列三种模式。

**代码现状**：
- 变化横断面模式已实现：`CrossSectionAtStation` + `interpolate_cross_section_layer()` 支持变截面插值 (linear_component.rs:42-48, 363-397)
- `build_beam()` (662-688) 就是"固定截面"模式的特例（单层 + 单截面站）
- "元件序列"模式（E3D BRANCH 风格）不存在

**调整建议**：
- 固定截面模式：已有，无需新增，通过 `CrossSectionAtStation` 单层默认实现
- 变化横断面模式：已有，完善 `segments` 自适应采样
- 元件序列模式：**需新增**，但应在 `LinearComponentInput` 中增加 `ComponentSequenceItem` 序列，而非拆分为独立模块

### 2.2 ⚠️ 建议二：PrimitiveGen 扩展至 15+ 基本体

**分析文档建议**：将基本体分为 Simple/Contour/Boolean 三类，扩展至 15+ 种。

**代码现状**：
- 无独立 PrimitiveGen 模块
- 基本体通过 `ParametricGeometry` + `ParametricOperation` 表达 (assembly.rs:31-36)
- 当前仅支持 `Box` 和 `Cylinder` 两种基本体操作
- 基本体生成由 Catalog 系统的 `FeatureTreeTemplate` 驱动，非硬编码

**调整建议**：
- 不应创建独立 PrimitiveGen 模块，而应扩展 `ParametricOperation` 枚举
- 新增：`Cone`, `Sphere`, `Torus`, `Dish`, `Extrusion`, `Sweep`, `Revolve`, `Loft` 等操作
- 这些操作应在 `omnim_core` 的 `parametric_operation` 中定义，由 Feature Engine 消费
- 基本体类型扩展与 Catalog 系统配合，而非独立 GenRule

### 2.3 ❌ 建议三：BooleanSubtractGen 作为 PrimitiveGen 属性

**分析文档建议**：将 BooleanSubtract 降级为 Primitive 的 `is_negative` 属性。

**代码现状**：
- `boolean_subtract.rs` 是独立的 565 行模块，实现了完整的 mesh 布尔差集算法
- 包含：包围盒检测、边界边提取、去重、cap 面生成、兜底 AABB 端面
- 处理了边界情况：空网格、完全重叠、部分重叠
- 测试覆盖：6 个测试用例

**调整建议**：**保留独立模块**。原因：
1. 布尔运算是一个独立的几何算法，与基本体生成解耦
2. 重构成本高（565 行稳定代码）
3. 在 `Door/Window → BooleanSubtract` 的映射中，布尔运算的宿主是墙（LayerExtrusion），不是基本体
4. 未来可扩展为 `BooleanUnion` 和 `BooleanIntersect`，更不适合作为基本体属性

可以增加的是：在 `ParametricOperation` 中增加 `BooleanOperation` 类型，让 Feature Engine 也能执行布尔运算。

### 2.4 ⚠️ 建议四：Beam/Column 的 GenRule 归属

**分析文档建议**：Beam/Column 应为 LinearComponent（固定截面模式），而非 Primitive。

**代码现状**：
- `primary_gen_rule()` 中 Beam/Column → `LinearComponent`（已按建议修改）
- 但 `linear_component.rs` 已有 `build_beam()` (662-688) 和 `build_tapered_beam()` (695-729)
- `default_catalog_component()` 中 Beam → `"Concrete-Beam-Rect"`, Column → `"Concrete-Column-Rect"`
- 默认模板的几何生成由 Catalog 的 FeatureTreeTemplate 定义，不依赖 `primary_gen_rule()` 的映射

**调整建议**：这个映射实际上是**不矛盾的**。
- `GenRuleHint::Primitive` 意味着"可以直接用基本体表达"（矩形截面 = Box）
- 但 beam 的"语义"是线构件，这与 `build_beam()` 在 LinearComponent 中实现不冲突
- 实际几何生成由 Catalog 模板决定，而非 `primary_gen_rule()`
- 如果未来需要变截面梁，Catalog 模板可以使用 LinearComponent 的 FeatureTreeTemplate

**结论**：已实际修改映射为 `LinearComponent`（`mod.rs:80`），与 `build_beam()` 的语义一致。固定截面的梁/柱作为线构件的特例，在概念和代码层面都准确。

### 2.5 ✅ 建议五：Connector 系统统一编码映射

**分析文档建议**：建立 Civil3D Point Code / OmniM ConnectorType / E3D P-Point 的编码映射表。

**代码现状**：
- `ConnectorDef` (catalog.rs:136-185) 已使用**表达式驱动**的位置/方向
- 支持 `position_x_expr`, `direction_x_expr` 等表达式字符串
- 支持 `connection_type_code` 和 `role` 字段
- 比静态 P-Point 更灵活（表达式可引用参数）

**调整建议**：
- 表达式驱动的 Connector 比静态 P-Point 更强大，应保留
- 在 `connection_type_code` 的基础上建立编码映射表
- 新增 `CrossReference` 映射：Civil3D `"ETW"` → OmniM `"ROAD_EDGE"` → E3D `"P1"`
- 这个映射表应放在 Catalog 系统中，作为 Schema 的一部分

---

## 3. 分析文档中缺失的关键内容

### 3.1 GenRule 双层架构（已覆盖 ↑）

### 3.2 数据流链路

从用户命令到 Three.js 渲染的完整链路（当前实现）：

```
用户输入: create wall W1
  → cli crate 解析命令
  → Project::create_object_from_default_template(Wall)
    → default_catalog_component(Wall) → "Concrete-Wall"
    → CatalogStore::get("Concrete-Wall") → ComponentDef
    → ComponentDef 关联 FeatureTreeTemplate
    → FeatureEngine 执行 FeatureTreeTemplate
      → 生成 BRep (ParametricGeometry)
      → 可选: 转为 MeshGeometry
  → Object 存储 MeshGeometry
  → WebSocket 推送至前端
  → ThreeViewer.tsx 接收 MeshGeometry
  → Three.js BufferGeometry 渲染
```

**关键点**：`linear_component.rs` / `layer_extrusion.rs` 等 GenRule 模块在当前流程中**不被调用**。它们仅作为便捷函数用于测试或旧 `generate_geometry()` 路径。实际几何生成由 Catalog 的 FeatureTreeTemplate 驱动。

### 3.3 性能与大型工程的考虑

**当前代码现状**：
- `segments` 参数硬编码 (linear_component.rs:58)，无自适应采样
- `ring_segments` 固定为 16 (linear_component.rs:577)
- 无 LOD (Level of Detail) 策略
- 布尔运算的 `generate_bbox_cap_faces()` 是兜底方案，精度有限

**建议补充**：
1. **自适应采样**：根据路径曲率动态调整 `segments`，曲率大处加密，直线处稀疏
2. **LOD 策略**：`LinearComponentInput` 增加 `detail_level` 枚举 (Low/Medium/High)
3. **大型装配体**：`AssemblyGen` 应支持懒惰加载子组件
4. **布尔运算优化**：使用空间加速结构（BVH/AABB Tree）提高剪切检测效率

### 3.4 Web 前端集成

**当前实现**：
- `MeshGeometry` 通过 WebSocket 传递给前端
- ThreeViewer.tsx 消费，但无实时预览支持

**建议补充**：
1. 前端应缓存 Catalog 中的 ComponentDef 定义，实现参数化调整实时预览
2. GenRuleHint 信息应随 MeshGeometry 一起发送，便于前端展示类型标签
3. 变截面编辑的 UI 交互（Civil 3D SAC 风格流程图编辑器）

### 3.5 IFC 实体映射

**当前映射**（基于 `primary_gen_rule()` + `default_catalog_component()`）：

| IFC 实体 | OmniM ObjectType | GenRuleHint |
|----------|-----------------|-------------|
| IfcWall | Wall | LayerExtrusion |
| IfcBeam | Beam | LinearComponent |
| IfcColumn | Column | LinearComponent |
| IfcDoor | Door | BooleanSubtract |
| IfcWindow | Window | BooleanSubtract |
| IfcSlab | Slab | LayerExtrusion |
| IfcPipeSegment | PipePlant | LinearComponent |
| IfcRoad | Road | LinearComponent |
| IfcPavement | (未映射) | — |
| IfcPump | Pump | Primitive |
| IfcTank | Tank | Primitive |

**建议补充**：增加 IfcPavement → ObjectType::Pavement 的映射，GenRuleHint 应为 LayerExtrusion（路面是多层结构）。

---

## 4. 三种设计范式在 OmniM 中的实现路径

### 4.1 Revit 式（语义驱动）
- 用户操作：`create wall`, `create beam`, `create slab`
- 实现路径：`default_catalog_component()` → Catalog 模板
- 背后 GenRule：LayerExtrusion / LinearComponent（固定截面）

### 4.2 Civil 3D 式（线构件驱动）
- 用户操作：`create road --alignment ... --cross-section ...`
- 实现路径：用户提供 Alignment + CrossSection 参数
- 背后 GenRule：LinearComponent（变化横断面模式）
- 当前代码：`LinearComponentInput` 已支持，但 CLI 未暴露这些参数

### 4.3 E3D 式（基本体+目录驱动）
- 用户操作：`create equipment --type pump --params ...`
- 实现路径：Catalog 中的 Equipment 模板
- 背后 GenRule：Assembly（多个 Primitive 组合）
- 当前代码：`build_ahu()`, `build_pump()`, `build_reactor()` 等已实现

---

## 5. 总结与优先级建议

### 5.1 高优先级（与当前架构一致，可立即落地）

1. **扩展 ParametricOperation 枚举**：增加 Cone, Sphere, Torus, Dish, Sweep, Revolve, Loft
2. **增加 IfcPavement 映射**：ObjectType::Pavement → LayerExtrusion
3. **完善 Connector 编码映射表**：在 Catalog 中建立跨平台编码映射

### 5.2 中优先级（需设计，但方向明确）

4. **LinearComponent 元件序列模式**：在 `LinearComponentInput` 中增加 `ComponentSequenceItem`
5. **自适应采样**：根据路径曲率动态调整 `segments`
6. **LOD 策略**：增加 `detail_level` 支持

### 5.3 低优先级（需重新设计架构）

7. **BooleanSubtract 作为操作而非独立 GenRule**：在 `ParametricOperation` 中增加 `BooleanOperation`
8. **Beam/Column 的 GenRule 归属调整**：已改为 `LinearComponent` 映射（`mod.rs:80`），与 `build_beam()` 语义一致
9. **流程图式变化规则**：超越 Civil 3D SAC 的 Sequence/Decision/Switch 支持

### 5.4 不需要做

- **创建独立 PrimitiveGen 模块**：当前 Catalog + Feature Engine 路径已足够
- **BooleanSubtract 降级为属性**：独立模块更清晰，重构成本高
- **重构 GenRule 分类层**：`primary_gen_rule()` 映射稳定，无需改动