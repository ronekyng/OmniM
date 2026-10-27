# 行业标准参考资料与平台调研

## 目录结构

### IFC4.3 标准参考

| 文件 | 来源 | 内容 |
|------|------|------|
| `ifc4x3_alignment.html` | buildingSMART IFC4.3 | IfcAlignment 线形参考系统、水平/垂直/超高布局、几何表示 |
| `ifc4x3_road.html` | buildingSMART IFC4.3 | IfcRoad 道路空间结构、Pset_RoadDesignCriteriaCommon(设计速度、车道宽度等) |
| `ifc4x3_beam.txt` | buildingSMART IFC4.3 | IfcBeam 梁的材料轮廓集、结构构件属性 |
| `ifc4x3_column.html` | buildingSMART IFC4.3 | IfcColumn 柱的 IfcMaterialProfileSetUsage、混凝土/钢筋属性集 |
| `ifc4x3_wall.txt` | buildingSMART IFC4.3 | IfcWall 墙的材料层集、开洞关系 |
| `ifc4x3_door.html` | buildingSMART IFC4.3 | IfcDoor 参数化门类型、门框/门板属性、操作类型、玻璃属性 |
| `ifc4x3_window.html` | buildingSMART IFC4.3 | IfcWindow 参数化窗类型、分区、窗框/窗扇属性 |
| `ifc4x3_pavement.html` | buildingSMART IFC4.3 | IfcPavement 路面结构层、Pset_PavementCommon(厚度、坡度、宽度) |
| `ifc4x3_pipesegment.html` | buildingSMART IFC4.3 | IfcPipeSegment 管道段、Pset_PipeSegmentTypeCommon(公称直径、工作压力)、法兰连接属性 |

### 商业平台调研（2026-07-09 初版 / 2026-08-07 扩展）

| 文件 | 来源 | 内容 |
|------|------|------|
| `civil3d_components.md` | Autodesk Civil 3D 2026 Help、SAC 文档、SDC Publications | Civil 3D 内置构件底层逻辑：Subassembly 点/链接/形状、Assembly/Corridor 模型、Subassembly Composer 流程图、Pipe Network 管网系统、面/线/体构件分类 |
| `revit_components.md` | Revit 2027 API Docs、Revit API Developers Guide、revitapidocs.com | Revit 内置构件底层逻辑：Element/Family/Type/Instance 四层架构、GenericForm 几何创建（Extrusion/Sweep/Revolve/Blend/Loft）、GeometryCreationUtilities、面构件(Wall/Floor)线构件(Beam/Column)体系 |
| `aveva_e3d_primitives.md` | AVEVA E3D 2.1 Docs、PDMS 设备建模指南 | E3D/PDMS 内置构件底层逻辑：数据库层级(WORLD→SITE→ZONE→EQUI→Primitive)、基本体类型(BOX/CYLI/CONE/SNOUT/NOZZ 等)、P-Point 连接点系统、管道/结构/设备建模体系、目录与规范系统 |
| `solidworks_components.md` | SOLIDWORKS 2026 官方文档（help.solidworks.com/2026/ API Programmer's Guide + API Reference + 用户帮助）、ASME Y14.5.1M 标准 | SOLIDWORKS 内置构件体系（**官方渠道重新调研**）：Part/Assembly/SubAssembly 三文档体系、FeatureManager 特征树(Extrude/Revolve/Sweep/Loft/Boundary 五类草图特征 + Hole Wizard 7 类孔 + Fillet 8 子类型)、**Toolbox 官方 15 类硬件**(Bolt/Nut/Washer/Screw/Pin/Bearing/Gear/Cam/Sprocket/Timing Pulley/Retaining Ring/Jig Bushing/PEM Insert/Structural Shape/Unistrut，支持 11 标准)、**21 类 Mates**(标准8+高级6+机械7)、设计表/配置、MBD/DimXpert/GD&T、Weldments、Sheet Metal、多实体零件；含 §0 重要更正说明（修正初版臆造的 Toolbox 内容） |
| `bentley_openroads_components.md` | Bentley 官方文档(OpenBridge Modeler/OpenRoads/OpenTunnel)、Bentley Communities、TxDOT/FDOT/FLH/Oregon DOT 培训手册 | Bentley 三产品内置构件底层逻辑：走廊模板(Corridor Template)、Linear/Surface Template、Civil Cell、Parametric Cell/Functional Component；OBM 上部结构(Beam Definition/Deck Template/Bearing/Expansion Joint)与下部结构(Pier Config/Abutment/Wingwall)；OTD 衬砌/TBM UTR/Portal/Shaft/CrossPassage；ORD Terrain/Cut-Fill/Slope/Drainage/Pond/Dam；GenRule 跨平台验证 |
| `tekla_components.md` | Tekla Open API 2025/2026 官方文档(developer.tekla.com)、Tekla Structures 用户指南(support.tekla.com)、TSOpenAPIExamples | Tekla Structures 内置构件底层逻辑：Part 基类(Beam/ContourPlate/PolyBeam/Brep/LoftedPlate)、型材库 17 类截面、Connection/Component/Detail/Seam 过程化节点系统(ConnectionBase.Run 输入契约)、基础(Pad/Strip Footing/Pile/PileCap)、7 类钢筋(SingleRebar/RebarGroup/CircleRebarGroup/CurvedRebarGroup/RebarMesh/RebarStrand/RebarSet)、切割特征(BooleanPart/Fitting/CutPlane/EdgeChamfer)、BoltGroup/Weld；Tekla 组件系统=OmniM Assembly 工业原型 |
| `e3d_instrumentation_components.md` | AVEVA 官方文档(help.aveva.com E3D 2.1)、AVEVA Instrumentation/Electrical 产品手册与培训大纲、ISA-5.1 标准 | AVEVA 仪表/电气内置组件逻辑：AVEVA Instrumentation(原 INtools/SPI) 与 AVEVA Electrical(原 SPEL) 四模块结构、E3D 数据库 SCHEMATIC/ENGINEERING(逻辑) vs MODEL(物理) 双视图架构；仪表(Instrument/Sensor/Transmitter/ControlValve/JunctionBox)、电气(CableTray/Conduit/Panel/MCC/Cable)、逻辑对象(Loop/ProcessLine/Tag ISA 体系)、Filter/Support；GenRule 映射(Assembly 主体/LinearComponent 线缆/BooleanSubtract 面板开孔) + LogicalObject 层建议 |
| `site_terrain_components.md` | 六平台场地与地形逻辑专项调研 | Revit/Civil 3D/Bentley/E3D/Tekla/SolidWorks 场地(Site)空间容器、地形(Terrain/TIN)建模、土方计算、场地平整、边坡设计、地形-结构交互的跨平台对比；含六平台能力评分、OmniM Site/Terrain ObjectType 清单、TerrainModel 数据结构建议、IFC 4x3 + LandXML 互操作映射、P0-P3 优先级路线图 |
| `comparative_analysis.md` | 综合对比分析 | 七平台跨平台对比：Civil 3D vs Revit vs E3D vs SOLIDWORKS vs Bentley vs Tekla vs E3D Instrumentation 面构件/线构件/体构件对比、五种 GenRule 七平台收敛验证、连接器系统对比、场地/地形跨平台对比、核心结论与 12 条设计建议(含 TerrainModel/LogicalObject 扩展) |

### 调研进度跟踪

| 文件 | 内容 |
|------|------|
| `RESEARCH_PLAN.md` | 五阶段调研计划（Phase 1-4 平台调研 + Phase 5 Proto 更新）|
| `COMPONENT_INVENTORY.md` | 152 个构件跨四域(Building/Civil/Plant/Mechanical)调研覆盖率追踪，含七平台 GenRule 收敛验证矩阵 |
| `genrule_design_analysis.md` | GenRule 体系设计分析：基于七平台调研对现有 gen_rule 代码(codes/linear_component/layer_extrusion/assembly/boolean_subtract/catalog)的反思与改进建议 |
| `RESEARCH_INTEGRATION.md` | 跨平台调研整合分析：基于八份平台调研文档的选择与实施策略 |

> 📌 路径说明 / Path note: 本目录内指向主文档的路径遵循 2026-10-27 文档重组（specification/、status/、reference/），内容本身未改动。
> Paths into the main docs follow the 2026-10-27 reorganization (specification/, status/, reference/). Content unchanged.

## 来源

### IFC 标准
- IFC4.3 官方文档: https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/HTML/
- IFC4x3 IfcSite / IfcBuilding / IfcBuildingStorey: https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/HTML/entity/ifcsite.htm
- IFC4x3 IfcGeographicElement（地形/地理元素）: https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/HTML/entity/ifcgeographicelement.htm
- LandXML 1.2 规范: https://www.landxml.org/
- buildingSMART Technical: https://technical.buildingsmart.org/

### 商业平台文档
- Autodesk Civil 3D Help: https://help.autodesk.com/view/CIV3D/2026/ENG/
- Autodesk Revit Help: https://help.autodesk.com/view/RVT/2027/ENU/
- Revit API Docs (2027): https://www.revitapidocs.com/2027/
- SOLIDWORKS 2026 Help (官方): https://help.solidworks.com/2026/English/solidworks/
- SOLIDWORKS API Programmer's Guide: https://help.solidworks.com/2026/English/api/sldworksapiprogguide/Welcome.htm
- SOLIDWORKS API Reference: https://help.solidworks.com/2026/English/api/sldworksapi/ (注意 English 首字母大写)
- AVEVA E3D Docs: https://docs.aveva.com/category/e3d (部分页面需登录)
- AVEVA Instrumentation / Electrical Docs: https://docs.aveva.com/ (产品手册与培训大纲)
- Bentley OpenBridge Modeler Help: https://docs.bentley.com/LiveContent/web/OpenBridge%20Modeler%20Help-v16/
- Bentley OpenRoads Designer Help: https://docs.bentley.com/LiveContent/web/OpenRoads%20Designer-v28/
- Bentley OpenTunnel Designer Help: https://docs.bentley.com/LiveContent/web/OpenTunnel%20Designer-v2025/Help/
- Bentley Communities 知识库: https://bentleysystems.service-now.com/community
- Tekla Open API Developer Center: https://developer.tekla.com/
- Tekla Structures User Assistance: https://help.tekla.com/
- ISA-5.1 Instrumentation Symbols Standard: https://www.isa.org/standards-and-publications/isa-standards/isa-standards-committees/isa5

## 说明

- Autodesk 和 AVEVA 文档网站使用 JavaScript 单页应用渲染，部分页面 webfetch 无法直接提取内容
- 调研文档基于 Web 搜索、API 文档抓取和社区资料综合整理
- 初版调研日期：2026-07-09（Civil 3D / Revit / E3D 三平台）
- 扩展调研日期：2026-08-07（补充 SolidWorks / Bentley / Tekla / E3D Instrumentation，共七平台）
- 调研覆盖率：152 个构件中 119 个已研究（78.3%），19 个结构相似推断，13 个未研究