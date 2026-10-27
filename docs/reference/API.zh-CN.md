# OmniM REST API 参考

> **权威唯一来源**：[`crates/server/src/lib.rs`](../../crates/server/src/lib.rs) 中 `build_router` 内的 `.route(...)` 路由表。本文档通过审计该路由表生成；下列所有端点均可在该表中按相同顺序找到。

## 目录

1. [并发、认证与实时协作](#并发认证与实时协作)
2. [基础 URL 约定](#基础-url-约定)
3. [健康、项目与统计](#健康项目与统计)
4. [模型对象与属性](#模型对象与属性)
5. [几何与变换](#几何与变换)
6. [关系](#关系)
7. [材质](#材质)
8. [测量](#测量)
9. [生命周期](#生命周期)
10. [行政区划](#行政区划)
11. [拆分结构](#拆分结构)
12. [Omcom 执行](#omcom-执行)
13. [网格与渲染](#网格与渲染)
14. [IFC 导入与导出](#ifc-导入与导出)
15. [草图系统](#草图系统)
16. [构件与特征](#构件与特征)
17. [目录（Catalog）](#目录catalog)
18. [系列（Series）](#系列series)
19. [GenRule](#genrule)
20. [撤销 / 重做](#撤销--重做)
21. [AMIR 端点](#amir-端点)
22. [静态文件](#静态文件)
23. [状态码与错误处理](#状态码与错误处理)

## 并发、认证与实时协作

- **无认证。** 服务器面向本地 / 单用户使用，没有 API Key、会话或令牌。
- **无 WebSocket / 实时协作。** OmniM 仅提供 **REST** 接口，没有实时同步、事件流或在线状态。需要较新的状态时轮询 `GET /api/tree` 或 `GET /api/objects`；Web UI 在每次变更后简单重新拉取。
- **内存中的单一项目。** 所有路由操作同一个共享 `Project`（`Arc<RwLock<Project>>`，即 `SharedProject`）。读共享读锁；写占用写锁，变更串行执行。
- **撤销/重做后端。** 变更类端点（对象创建/删除、草图编辑、GenRule 创建）会把操作压入 Omcom 执行器的历史栈，通过 `GET /api/undo-status` 查询。

## 基础 URL 约定

- **默认基础 URL**：`http://0.0.0.0:3000` — 服务器默认监听 `0.0.0.0`（所有网卡）端口 `3000`。本机访问可用 `http://localhost:3000`。
- **环境变量**（在 `crates/server/src/main.rs` 中读取）：

  | 变量 | 默认值 | 用途 |
  |---|---|---|
  | `OMNIM_HOST` | `0.0.0.0` | 监听地址 |
  | `OMNIM_PORT` | `3000` | HTTP 端口 |
  | `OMNIM_PROJECT` | `project.omnim` | 启动时加载的项目文件；同时作为自动保存路径（默认文件不启用自动保存）。 |
  | `AMIR_PROVIDER` / `AMIR_API_KEY` / `AMIR_MODEL` | — | 配置 AMIR AI 引擎（`/api/amir/*`）；未配置时 AMIR 端点返回 `501`。 |

- **请求体**为 `application/json`（UTF-8），`POST /api/import/ifc` 例外（原始 IFC 文本）。
- **响应**为 `application/json`，`GET /api/export/ifc`（`application/ifc`）与 `GET /api/health`（纯文本）除外。
- 许多请求/响应类型（`OmniObject`、`OmniCommand`、`Point3D`、`Material`、`Project`、`ExecutionResult` 等）是 **由 protobuf 生成的 Rust 类型**（prost），经 `serde` 序列化：JSON 键为 snake_case，重复字段是 JSON 数组，枚举为整数（如 `"object_type": 1`）。标签值见 `omnim_core` 中生成的绑定代码。
- 请求体大小上限：**100 MB**（`DefaultBodyLimit::max(100 * 1024 * 1024)`），主要为大型 IFC 导入预留。
- `web/src/shared/api.ts` 中有镜像本 API 的 TypeScript 客户端。

```bash
BASE=http://localhost:3000
```

## 健康、项目与统计

| 方法与路径 | 用途 |
|---|---|
| `GET /api/health` | 纯文本存活探针。 |
| `GET /api/project` | 整个项目文档（`Project`）。 |
| `GET /api/tree` | 对象层级树。 |
| `GET /api/stats` | 聚合统计。 |

- `GET /api/health` → `200`，响应体 `ok`。
- `GET /api/project` → 整个 `Project`：`name`、`objects`、`materials`、`lifecycle`、`relations`、`measurements`、`sketches`、`breakdown`、`administrative`、`series` 等。数据量大；UI 状态优先用 `/api/tree` 或 `/api/stats`。
- `GET /api/tree` → `Vec<TreeNode>`；每节点：`{ id, name, object_type, ifc_class, has_geometry, children[] }`。仅根对象在顶层，子对象嵌套。
- `GET /api/stats` → `{ name, object_count, material_count, lifecycle_record_count, connection_count, containment_count }`。

```bash
curl -sS "$BASE/api/stats"
```

## 模型对象与属性

| 方法与路径 | 用途 |
|---|---|
| `GET /api/objects` | 列出所有对象。 |
| `POST /api/objects` | 创建对象。 |
| `GET /api/objects/{id}` | 获取单个对象。 |
| `PUT /api/objects/{id}` | 更新名称/描述/IFC 类/标签。 |
| `DELETE /api/objects/{id}` | 删除对象（可撤销）。 |
| `PUT /api/objects/{id}/prop` | 设置单个字符串属性。 |
| `PUT /api/objects/{id}/properties` | 批量更新字段 + 属性。 |
| `GET /api/object-types/{type}/properties` | 某类型的属性定义。 |

- `POST /api/objects` 请求体（`CreateObjectRequest`）：

  ```json
  {
    "object_type": "wall",
    "name": "North wall",
    "description": "可选",
    "ifc_class": "IfcWall",
    "tags": ["facade"],
    "geometry": { "operations": [ /* ParametricGeometry，可选 */ ] },
    "parent_id": "可选父对象 id"
  }
  ```

  `object_type`（大小写不敏感）接受：`wall`、`slab`、`roof`、`floor`、`beam`、`column`、`door`、`window`、`road`、`pipe`、`duct`、`cabletray`、`tank`、`pump`、`valve`、`gear`、`part`、`assembly`、`component`。未知名称会导致 Omcom 创建操作记录未指定类型——请使用上面列出的名称。请求内部包装为一个 Omcom `CreateObject` 命令（会记入撤销历史），然后追加 `ifc_class`、`tags` 与 `parent_id`。响应：创建的 `OmniObject`；执行失败时 `500`。

- `OmniObject` JSON 结构（protobuf 派生）：`{ id, name, description, object_type（整数枚举）, status（整数）, properties[], property_sets[], geometry（Geometry 或 null）, material_id, parent_id, tags[], ifc_class, features[], sketch_ids[], dirty, catalog_ref }`。
- `PUT /api/objects/{id}` 请求体：`{ name?, description?, ifc_class?, tags? }` → 成功返回空响应，找不到为 `404`。
- `PUT /api/objects/{id}/prop` 请求体：`{ "name": "FireRating", "value": "2h" }` — 值以字符串 `PropertyValue` 存储。成功返回空响应，`404`。
- `PUT /api/objects/{id}/properties` 请求体：类似 `PUT /api/objects/{id}`，外加 `"properties": { "k": "v", ... }`（字符串映射）。适合编辑器一次性同步。成功返回空响应。
- `GET /api/object-types/{type}/properties` → `Vec<ObjectPropertyDef>`（例如 "wall" 的属性定义）；未知类型返回空列表。

```bash
# 创建一面墙
curl -sS -X POST "$BASE/api/objects" -H 'Content-Type: application/json' \
  -d '{"object_type":"wall","name":"North wall","ifc_class":"IfcWall","tags":["facade"]}'

# 设置单个属性
curl -sS -X PUT "$BASE/api/objects/obj-1/prop" -H 'Content-Type: application/json' \
  -d '{"name":"FireRating","value":"2h"}'
```

## 几何与变换

| 方法与路径 | 用途 |
|---|---|
| `PUT /api/objects/{id}/transform` | 平移 / 旋转 / 缩放对象的网格几何。 |

- 请求体（`TransformObjectRequest`），所有字段可选：

  ```json
  {
    "origin": { "x": 0, "y": 0, "z": 0 },            // 枢轴；默认为网格包围盒中心
    "translation": { "x": 1, "y": 0, "z": 0 },
    "rotation": { "x": 0, "y": 0, "z": 0, "w": 1 },  // 四元数
    "scale": { "x": 2, "y": 1, "z": 1 }
  }
  ```

  复合变换：先平移 `−origin`，应用平移/旋转/缩放，再平移回 `+origin`。任意几何表示会先转换为网格，变换后重新计算包围盒，并以 `Mesh` 形式重写对象几何。成功返回空响应；`404` 对象不存在，`400` 无几何。

## 关系

| 方法与路径 | 用途 |
|---|---|
| `PUT /api/objects/{id}/parent` | 设置父对象（包含关系）。 |
| `POST /api/objects/{from}/connect/{to}` | 创建结构连接。 |

- `PUT /api/objects/{id}/parent` 请求体：`{ "parent_id": "parent-1" }` → 成功返回空响应；`400` 非法（成环或父对象不存在）。
- `POST /api/objects/{from}/connect/{to}` — 无请求体；创建 `ConnectionType::Structural` 连接。成功返回空响应；`400` 非法配对。

```bash
curl -sS -X PUT "$BASE/api/objects/child-1/parent" -H 'Content-Type: application/json' \
  -d '{"parent_id":"parent-1"}'
curl -sS -X POST "$BASE/api/objects/column-1/connect/beam-2"
```

## 材质

| 方法与路径 | 用途 |
|---|---|
| `GET /api/materials` | 列出材质。 |
| `POST /api/materials` | 创建材质。 |
| `PUT /api/objects/{id}/material` | 为对象指定材质。 |

- `POST /api/materials` 请求体（`CreateMaterialRequest`）：`{ "id": "concrete-c30", "name": "C30 Concrete", "category": "concrete", "density": 2400.0 }`。`category` 按 `MaterialCategory` 变体大小写不敏感解析（未知 → `Unspecified`）。成功返回空响应。
- `PUT /api/objects/{id}/material` 请求体：`{ "material_id": "concrete-c30" }` → 成功返回空响应；`400` 材质或对象不存在。
- `GET /api/materials` → `Vec<Material>`（protobuf 派生：`id`、`name`、`category` 整数、`density` 等）。

```bash
curl -sS -X POST "$BASE/api/materials" -H 'Content-Type: application/json' \
  -d '{"id":"concrete-c30","name":"C30 Concrete","category":"concrete","density":2400.0}'
curl -sS -X PUT "$BASE/api/objects/obj-1/material" -H 'Content-Type: application/json' \
  -d '{"material_id":"concrete-c30"}'
```

## 测量

| 方法与路径 | 用途 |
|---|---|
| `GET /api/measurements` | 列出已保存的测量。 |
| `POST /api/measurements` | 创建测量。 |
| `DELETE /api/measurements/{id}` | 删除测量。 |

- `POST /api/measurements` 请求体（`CreateMeasurementRequest`）：

  ```json
  { "type": "distance", "points": [{ "x":0,"y":0,"z":0 }, { "x":3,"y":0,"z":0 }],
    "value": 3.0, "label": "房间宽度", "color": "#44ff88" }
  ```

  `type` 为自由字符串（常用的有 `distance`、`area`、`angle`、`volume`）；`color` 默认 `#44ff88`。响应：创建的 `Measurement`（`{ id, type, points, value, color, label }`）。`DELETE` 成功返回空响应，`404` id 不存在。

## 生命周期

| 方法与路径 | 用途 |
|---|---|
| `GET /api/lifecycle` | 列出生命周期记录。 |
| `POST /api/lifecycle` | 创建生命周期记录。 |
| `POST /api/lifecycle/{id}/link/{obj_id}` | 将记录关联到对象。 |

- `POST /api/lifecycle` 请求体（`CreateLifecycleRequest`）：

  ```json
  { "phase": "design", "record_type": "task", "name": "Foundation review",
    "description": "可选" }
  ```

  `phase` 按 `LifecyclePhase` 变体大小写不敏感解析。记录 id 会被赋值为 `lr-N`；未列出的字段取默认值（`status` = Draft，无日期/费用/责任方）。成功返回空响应。

- `POST /api/lifecycle/{id}/link/{obj_id}` — 无请求体。成功返回空响应；失败 `400`。

## 行政区划

| 方法与路径 | 用途 |
|---|---|
| `GET /api/administrative` | 列出区划。 |
| `POST /api/administrative` | 创建区划。 |
| `GET /api/administrative/{id}` | 获取单个。 |
| `DELETE /api/administrative/{id}` | 删除单个。 |
| `GET /api/administrative/{id}/children` | 直接子区划。 |
| `GET /api/administrative/ancestors/{id}` | 祖先链。 |
| `GET /api/administrative/code/{code}` | 按代码查找。 |

- 响应条目（`AdminDivResponse`）：`{ id, level, name, native_name, code, parent_id }` — `level` 是由 `level_name()` 生成的人类可读字符串。
- `POST /api/administrative` 请求体：`{ "level": "country", "name": "Netherlands", "code": "NL", "native_name": "Nederland", "parent_id": null }`。`level` 大小写不敏感解析。响应：创建的 `AdminDivResponse`。

## 拆分结构

| 方法与路径 | 用途 |
|---|---|
| `GET /api/breakdown` | 列出节点。 |
| `POST /api/breakdown` | 创建节点。 |
| `GET /api/breakdown/{id}` | 获取单个。 |
| `DELETE /api/breakdown/{id}` | 删除单个。 |
| `GET /api/breakdown/{id}/children` | 直接子节点。 |
| `GET /api/breakdown/{id}/descendants` | 全部后代。 |
| `GET /api/breakdown/{id}/ancestors` | 到根的祖先链。 |
| `POST /api/breakdown/{id}/link-obj` | 将对象挂到节点。 |
| `POST /api/breakdown/{id}/link-admin` | 将行政区划挂到节点。 |

- 响应条目（`BreakdownResponse`）：`{ id, level, name, code, convention, parent_id, related_object_ids, administrative_division_ids }`。
- `POST /api/breakdown` 请求体：`{ "level": "discipline", "name": "MEP", "convention": "omni", "code": "MEP", "parent_id": null }`。`level`（`project`、`site`、`building`、`storey`、`space`、`discipline` 等）与 `convention`（`omni`、`uniclass`、`omniclass` 等）大小写不敏感解析。响应：创建的 `BreakdownResponse`。
- `POST .../link-obj` 请求体 `{ "object_id": "obj-42" }`；`POST .../link-admin` 请求体 `{ "division_id": "admin-3" }` → 成功返回空响应。

## Omcom 执行

| 方法与路径 | 用途 |
|---|---|
| `POST /api/omcom` | 对项目执行一个 `OmniCommand`。 |

- 请求体：完整的 `omnim_core::OmniCommand`（protobuf 派生）：

  ```json
  {
    "version": "1.0",
    "id": "cmd-001",
    "action": 1,
    "actor": null,
    "target": null,
    "operations": [
      { "op": { "createObject": { "type": 1, "name": "From omcom", "properties": [] } } }
    ],
    "validation": null,
    "timestamp": null,
    "metadata": {}
  }
  ```

- 响应 `omnim_omcom::ExecutionResult`：`{ success, message, created_ids }`。
- 这是面向高级客户端的低层网关；`POST /api/objects` 内部只包装单个 `CreateObject` 命令。
- 命令语义见 [OMCOM_REFERENCE.md](OMCOM_REFERENCE.md)。

## 网格与渲染

| 方法与路径 | 用途 |
|---|---|
| `GET /api/mesh` | 返回渲染器所需的所有对象网格。 |

- 响应 `MeshResponse`：`{ "objects": [MeshObject] }`，每个元素：

  ```json
  { "id": "obj-1", "name": "North wall", "object_type": "wall",
    "vertex_count": 8, "triangle_count": 12,
    "positions": [/* f32 x,y,z 顺序展开的一维数组 */],
    "normals":   [/* f32 平铺的三元组 */],
    "bbox_min": [0,0,0], "bbox_max": [5,3,0.3] }
  ```

  只包含有几何的对象；这是 Web 查看器（`web/src/shared/api.ts`）加载时消费的批量端点。

## IFC 导入与导出

| 方法与路径 | 用途 |
|---|---|
| `GET /api/export/ifc` | 将整个项目导出为 IFC 文本。 |
| `POST /api/import/ifc` | 将 IFC 文本导入项目。 |

- `GET /api/export/ifc` → `200`，`Content-Type: application/ifc`，`Content-Disposition: attachment; filename="project.ifc"`（原始 IFC STEP 文本）。
- `POST /api/import/ifc` — 请求体为 **原始 IFC 文本**，不是 JSON。100 MB 的请求体上限主要为该路由服务。
  - 查询参数：`normalize`（默认 `true` — 将导入几何平移到原点附近）与 `mode`（默认 `merge` | `replace` — 整体替换项目）。
  - 响应 `ImportIfcResponse`：`{ success, imported, objects, message }`。IFC 解析失败仍返回 HTTP `200`，`success: false`。

```bash
# 导出
curl -sS "$BASE/api/export/ifc" -o project.ifc

# 导入（原始文本请求体）
curl -sS -X POST "$BASE/api/import/ifc?normalize=true&mode=merge" \
  --data-binary '@model.ifc' -H 'Content-Type: text/plain'
```

## 草图系统

草图是挂载到项目（可选地挂到某个构件）的 2D 轮廓，可被拉伸/旋转为 3D 实体。

| 方法与路径 | 用途 |
|---|---|
| `GET /api/sketches` | 列出草图（可选 `?component_id=`）。 |
| `POST /api/sketches` | 创建草图。 |
| `GET /api/sketches/{id}` | 获取完整草图（图元 + 约束）。 |
| `PUT /api/sketches/{id}` | 重命名 / 移动原点。 |
| `DELETE /api/sketches/{id}` | 删除草图（可撤销）。 |
| `POST /api/sketches/{id}/entities` | 添加图元（点/线/圆/…）。 |
| `POST /api/sketches/{id}/constraints` | 添加约束。 |
| `POST /api/sketches/{id}/solve` | 运行约束求解器。 |
| `PUT /api/sketches/{id}/entities/{idx}` | 更新图元。 |
| `DELETE /api/sketches/{id}/entities/{idx}` | 删除图元。 |
| `DELETE /api/sketches/{id}/constraints/{idx}` | 删除约束。 |
| `POST /api/sketches/{id}/extrude` | 将轮廓拉伸为 3D Part。 |
| `POST /api/sketches/{id}/revolve` | 将轮廓绕轴旋转成体。 |

- `POST /api/sketches` 请求体：`{ "name": "Facade profile", "origin": { "x":0,"y":0,"z":0 }, "normal": { "x":0,"y":0,"z":1 }, "component_id": null }` — `component_id` 必须引用已存在的构件（否则 `400`）。响应 `SketchResponse`：`{ id, name, origin, normal, entity_count, constraint_count, solved, component_id }`。
- `GET /api/sketches/{id}` → 完整的 `omnim_sketch::Sketch`（`{ id, name, origin, normal, entities[], constraints[], solved, component_id }`）。
- `POST .../entities` 请求体（`AddEntityRequest`）：`{ "entity_type": "circle", "points": [{ "x":1,"y":0,"z":0 }], "radius": 0.5, "sides": 0, "construction": false }`。`entity_type` ∈ `point | line | circle | arc | rectangle | polygon | ellipse | spline | polyline`；`radius`/`sides` 适用于圆/多边形类图元；`construction` 标记为构造几何。响应：`{ "index": 0 }`（图元在草图中的索引）。
- `POST .../constraints` 请求体（`AddConstraintRequest`）：`{ "constraint_type": "length", "entity_refs": [[0,0],[0,1]], "value": 3.5 }`。`entity_refs` 为 `[图元索引, 点索引]` 对。`constraint_type` ∈ `horizontal, vertical, coincident, parallel, perpendicular, length, distance, angle, radius, diameter, tangent, symmetric, fixed, at_midpoint, equal_length, equal_radius, pt_on_line, pt_on_circle, same_orientation`。响应 `{ "index": 0 }`。
- `POST .../solve` → `SolveResponse`：`{ "success": true, "dof": 0, "message": "..." }`。
- `PUT .../entities/{idx}` 请求体（`UpdateEntityRequest`）：`{ points?, radius?, sides?, construction? }` → `204`/空响应；`404` 索引越界。
- `POST .../extrude` 请求体：`{ "depth": 2.5, "direction_x": 0, "direction_y": 0, "direction_z": 1 }`（方向默认沿草图法线 +Z）。会先自动求解。响应 `FeatureResponse`：`{ success, message, vertices, faces, object_id }` — 会创建名为 `extrude-<草图名>` 的新 `Part` 对象。草图不可解或轮廓非法时 `400`。
- `POST .../revolve` 请求体：`{ "angle": 360, "axis_origin_x": 0, "axis_origin_y": 0, "axis_origin_z": 0, "axis_direction_x": 0, "axis_direction_y": 1, "axis_direction_z": 0 }`（轴默认为原点 / (0,0,1)）。响应结构与 `FeatureResponse` 相同；创建名为 `revolve-<草图名>` 的 `Part`。

```bash
# 创建草图，画矩形，求解，拉伸
SID=$(curl -sS -X POST "$BASE/api/sketches" -H 'Content-Type: application/json' \
  -d '{"name":"Pad"}' | python3 -c 'import sys,json;print(json.load(sys.stdin)["id"])')

curl -sS -X POST "$BASE/api/sketches/$SID/entities" -H 'Content-Type: application/json' \
  -d '{"entity_type":"rectangle","points":[{"x":0,"y":0,"z":0},{"x":2,"y":1,"z":0}]}'

curl -sS -X POST "$BASE/api/sketches/$SID/solve"
curl -sS -X POST "$BASE/api/sketches/$SID/extrude" -H 'Content-Type: application/json' \
  -d '{"depth":1.5}'
```

## 构件与特征

构件是带参数特征树的零件；特征重放草图（拉伸/旋转/扫掠/放样/布尔/…）。

| 方法与路径 | 用途 |
|---|---|
| `GET /api/components` | 列出构件。 |
| `POST /api/components` | 创建构件。 |
| `GET /api/components/{id}` | 获取单个。 |
| `DELETE /api/components/{id}` | 删除单个。 |
| `GET /api/components/{id}/features` | 列出特征。 |
| `POST /api/components/{id}/features` | 添加特征。 |
| `PUT /api/components/{id}/features/{fid}` | 更新特征。 |
| `DELETE /api/components/{id}/features/{fid}` | 删除特征。 |
| `POST /api/components/{id}/rebuild` | 重建特征树。 |
| `GET /api/components/{id}/section` | 计算水平剖切线（`?plane_z=`）。 |

- 条目为 `ComponentResponse`：`{ id, name, feature_count, sketch_ids, parent_id }`。
- `POST /api/components` 请求体：`{ "name": "Bolt", "parent_id": null }`。
- 特征列表条目为 `FeatureResponseItem`：`{ id, name, feature_type, sketch_id, input_sketch_ids, axis_sketch_id, depth, angle, is_cut, order, suppressed, dirty }`。
- `POST .../features` 请求体（`AddFeatureRequest`）：

  ```json
  { "feature_type": "extrude", "sketch_id": "sk-1", "input_sketch_ids": [],
    "axis_sketch_id": null, "depth": 2.0,
    "direction_x": 0, "direction_y": 0, "direction_z": 1,
    "angle": 0, "is_cut": false, "lod": 1e-6 }
  ```

  `feature_type` ∈ `extrude | revolve | sweep | loft | fusion | lofted_fusion | boolean | chamfer | fillet | pattern | shell`（未知 → `extrude`）。引用的草图必须存在（否则 `404`）；添加/更新/删除后构件自动重建。响应：创建的 `FeatureResponseItem`。
- `PUT .../features/{fid}` 请求体（`UpdateFeatureRequest`）：可含 `sketch_id, axis_sketch_id, depth, angle, is_cut, suppressed, input_sketch_ids, dependency_ids, direction, axis_origin, axis_direction, lod, boolean_op`（整数：`0=union`、`2=intersect`，其余为 `subtract`）、`pattern_count_x/y/z` → `{ "success": true }`。
- `GET .../section?plane_z=0.0` → `Vec<SectionLine>`（世界坐标 Z = `plane_z` 处水平剖切得到的 2D 折线），由所有特征几何合并后计算。

## 目录（Catalog）

目录构件定义保存在全局库中（`~/.omnim/library.omnimlib`，缺失时使用内置种子库）。

| 方法与路径 | 用途 |
|---|---|
| `GET /api/catalog` | 列出构件定义。 |
| `POST /api/catalog` | 创建定义。 |
| `GET /api/catalog/{id}` | 获取单个。 |
| `PUT /api/catalog/{id}` | 更新单个。 |
| `DELETE /api/catalog/{id}` | 删除单个。 |
| `POST /api/catalog/{id}/params` | 添加参数。 |
| `PUT /api/catalog/{id}/params/{idx}` | 更新参数。 |
| `DELETE /api/catalog/{id}/params/{idx}` | 删除参数。 |
| `POST /api/catalog/{id}/connectors` | 添加连接器。 |
| `PUT /api/catalog/{id}/feature-tree` | 指定特征树 id。 |
| `POST /api/catalog/{id}/publish` | 将定义标记为已发布。 |
| `POST /api/catalog/export` | 将库导出到带时间戳的 `.omnimlib` 文件。 |
| `POST /api/catalog/import` | 从 `.omnimlib` 文件导入定义。 |
| `POST /api/catalog/instantiate` | 将定义实例化到项目中。 |
| `POST /api/catalog/preview` | 临时构建构件并返回其网格。 |

- 条目为 `omnim_model::catalog::ComponentDef`：`{ id, name, category, description, ifc_class, standard_reference, tags, domain, params[], connectors[], allowed_materials, default_material, properties, feature_tree_id, published, ... }`。
- `POST /api/catalog` 请求体：`{ "name": "PB Pipe", "category": "pipe", "description": "...", "ifc_class": "IfcPipeSegment", "standard_reference": "", "tags": [], "domain": "" }` → 创建的 `ComponentDef`。
- `POST .../params` 请求体（`AddParamRequest`）：`{ "name": "length", "param_type": 1, "default_value": 1000, "min_value": 100, "max_value": 5000, "step": 10, "unit": "mm", "formula": "", "choices": [], "description": "" }` → `201`；`404` 定义不存在。
- `POST .../connectors` 请求体（`AddConnectorRequest`）：`{ "connector_id": "c1", "connector_name": "inlet", "position_x_expr": "0", "position_y_expr": "0", "position_z_expr": "L/2", "direction_x_expr": "0", "direction_y_expr": "0", "direction_z_expr": "1", "bore_expr": "bore", "connection_type_code": "", "role": 0 }` — 位置/方向/口径为表达式，在实例化时求值 → `201`。
- `PUT .../feature-tree` 请求体：`{ "feature_tree_id": "..." }`；`POST .../publish` 无请求体（设置 `published = true`） → `200`。
- `POST /api/catalog/export`（无请求体） → `{ "path": "/tmp/omnimlib-<ts>.omnimlib", "component_count": 3 }`。
- `POST /api/catalog/import` 请求体：`{ "path": "/tmp/omnimlib-....omnimlib" }` → `{ "component_count": 3, "names": ["..."] }`；文件无法加载时 `400`。
- `POST /api/catalog/instantiate` 请求体（`InstantiateRequest`）：

  ```json
  { "component_def_id": "def-1", "instance_name": "Pipe A",
    "params": { "length": 2000, "bore": 100 },
    "position": { "x": 0, "y": 0, "z": 0 },
    "rotation_z": 0,
    "skip_series_gate": false }
  ```

  `skip_series_gate: true` 会跳过"目录变换必须来自系列选择"的校验。响应 `InstantiateResponse`：`{ component_id, object_type, name }`；失败（如参数值非法）为 `400`。
- `POST /api/catalog/preview` 请求体：`{ "component_def_id": "def-1", "params": {...} }` → `c::MeshGeometry`（vertices/normals/uvs/faces），由毫米缩放到米；临时实例在响应前被移除，不会持久化到项目。

```bash
# 实例化一个目录构件
curl -sS -X POST "$BASE/api/catalog/instantiate" -H 'Content-Type: application/json' \
  -d '{"component_def_id":"def-1","instance_name":"Pipe A","params":{"length":2000}}'
```

## 系列（Series）

系列为标准构件建模代码驱动的选择逻辑：由选择器/答案组成的决策树，其叶子为 `SpecComponent`（目录定义 + 参数约束）。

| 方法与路径 | 用途 |
|---|---|
| `GET /api/series` | 列出系列。 |
| `POST /api/series` | 创建系列。 |
| `POST /api/series/save` | 将全部系列保存到带时间戳的 `.omnimseries` 文件。 |
| `POST /api/series/load` | 从外部文件加载系列。 |
| `GET /api/series/{id}` | 获取单个。 |
| `PUT /api/series/{id}` | 更新单个。 |
| `DELETE /api/series/{id}` | 删除单个。 |
| `POST /api/series/{id}/selectors` | 添加根选择器（仅第一次调用生效）。 |
| `POST /api/series/{id}/answers` | 为选择器添加答案。 |
| `POST /api/series/{id}/spec` | 将答案绑定到目录构件。 |
| `POST /api/series/{id}/constraints` | 添加参数约束。 |
| `PUT /api/series/{id}/constraints/{idx}` | 更新参数约束。 |
| `DELETE /api/series/{id}/constraints/{idx}` | 删除参数约束。 |
| `POST /api/series/{id}/select` | 运行选择引擎。 |

- 条目为 `omnim_model::series_store::Series`：`{ id, name, description, domain, standard_reference, applicable_phases, metadata, root_selector, ... }`。
- `POST /api/series` 请求体：`{ "name": "Pipe series", "description": "", "domain": "", "standard_reference": "" }` → 创建的 `Series`。
- `POST /api/series/save`（无请求体） → `{ "path": "/tmp/omnimseries-<ts>.omnimseries", "saved_count": 2 }`。
- `POST /api/series/load` 请求体：`{ "path": "/path/to/file.omnimseries", "source": "standalone" }` → `{ "loaded_count": 2, "series_names": ["Pipe series"] }`；系列会记入 `external_series_refs`。
- `POST .../selectors` 请求体：`{ "selector_id": "s-material", "question": "哪种材质？" }` — 每个系列只有第一次调用会设置 `root_selector`；后续调用被接受但不生效 → `201`。
- `POST .../answers` 请求体（`AddAnswerRequest`）：`{ "selector_id": "s-material", "answer_id": "steel", "answer_value": "steel", "is_numeric_range": false, "range_min": 0, "range_max": 0 }` — 新答案的 outcome 初始不指向任何构件；用 `/spec` 绑定（或在后续 `/answers` 中指向新选择器）。在树中找到选择器时 → `201`，否则 `404`。
- `POST .../spec` 请求体：`{ "selector_id": "s-material", "answer_id": "steel", "component_def_id": "def-1", "material": "steel-s235", "connection_type_code": "" }` → 找到时 `201`，否则 `404`。
- `POST .../constraints` 请求体（`AddParamConstraintRequest`）：`{ "component_def_id": "def-1", "param_name": "length", "default_value": 1000, "min_value": 100, "max_value": 5000, "step": 10, "fixed_value": "" }` → `201`（约束追加到匹配的 `SpecComponent`）；否则 `404`。
- `PUT|DELETE .../constraints/{idx}`：同样需要 `component_def_id` 定位构件，`idx` 是其 `param_constraints` 数组内的下标。
- `POST .../select` 请求体：`{ "answers": ["steel", "dn50"] }`（按决策树走的答案 id） → `SelectResponse`：`{ "component_def_id", "selector_path", "spec_component" }`；树不匹配为 `400`。

```bash
# 运行系列选择
curl -sS -X POST "$BASE/api/series/ser-1/select" -H 'Content-Type: application/json' \
  -d '{"answers":["steel","dn50"]}'
```

## GenRule

`POST /api/genrule` — 通过 GenRule 引擎（LayerExtrusion、LinearComponent、Primitive/Assembly、BooleanSubtract）声明式地一次性生成交互式构件。

| 方法与路径 | 用途 |
|---|---|
| `POST /api/genrule` | 按声明式规则生成对象。 |

- 请求体是一个 **带标签的枚举**（`GenRuleRequest`）：JSON 的 `type` 字段取 `wall | slab | road | pipe | door | window | assembly | layered | line`，其余字段按变体而定：

  | type | 关键字段 |
  |---|---|
  | `wall` | `name`, `length`, `height`, `core_thickness`, `finish_thickness`, `start_x/y/z`, `end_x/y/z`, `parent_id?`, `series_id?` |
  | `slab` | `name`, `length`, `width`, `structural_thickness`, `finish_thickness`, `parent_id?`, `series_id?` |
  | `road` | `name`, `length`, `width`, `thickness`, `parent_id?`, `series_id?` |
  | `pipe` | `name`, `length`, `radius`, `parent_id?`, `series_id?` |
  | `door` / `window` | `name`, `wall_id`, `opening_width`, `opening_height`, `position_x`, `position_z`, `parent_id?`（父级为墙；从墙包围盒布尔减去开洞） |
  | `assembly` | `name`, `components[]`（`{ name, shape: box\|cylinder\|sphere\|cone, p1, p2, p3, tx, ty, tz }`）, `parent_id?` |
  | `layered` | `name`, `mode`（`wall\|slab\|roof\|floor`）, `length`, `width`, `height`, `layers[]`（`{ name, thickness, material_id, function }`）, `path[]`（墙中心线 `[x,y,z]` 点）或 `boundary[]`（slab/roof/floor 闭合轮廓）, `start_*`/`end_*`, `parent_id?`, `series_id?` |
  | `line` | `name`, `alignment`（`[x,y,z]` 点，≥ 2）, `diameter`, `parent_id?` — 生成细引导管线（`FeatureLine`） |

  单位为米。`series_id` 非空时改走目录/系列实例化路径，而非直接生成网格。

- 响应：创建的 `OmniObject`（带生成的 `geometry`）。
- 错误：`404` `wall_id` 不存在（door/window），`400`（层数为空、对齐点不足 2、系列不可解等），并透传 GenRule 引擎错误。

```bash
curl -sS -X POST "$BASE/api/genrule" -H 'Content-Type: application/json' \
  -d '{"type":"wall","name":"Wall A","length":5,"height":3,"core_thickness":0.2,"finish_thickness":0.02,"start_x":0,"start_y":0,"start_z":0,"end_x":5,"end_y":0,"end_z":0}'
curl -sS -X POST "$BASE/api/genrule" -H 'Content-Type: application/json' \
  -d '{"type":"layered","mode":"wall","name":"Cavity wall","height":3,"layers":[{"name":"brick","thickness":0.1},{"name":"insulation","thickness":0.08}]}'
```

## 撤销 / 重做

| 方法与路径 | 用途 |
|---|---|
| `POST /api/undo` | 撤销最近一次记录的操作。 |
| `POST /api/redo` | 重做被撤销的操作。 |
| `GET /api/undo-status` | 查询历史栈状态。 |

- `POST /api/undo` / `POST /api/redo`（无请求体） → `UndoRedoResponse`：`{ "success": true, "message": "..." }`。栈为空时响应体中 `success: false`（HTTP 仍为 `200`）。
- `GET /api/undo-status` → `UndoStatusResponse`：`{ "can_undo": true, "can_redo": false, "undo_depth": 4 }`。

```bash
curl -sS -X POST "$BASE/api/undo"
curl -sS "$BASE/api/undo-status"
```

## AMIR 端点

AMIR 是 AI 层（自然语言 ⇄ Omcom）。**必须配置** `AMIR_PROVIDER` + `AMIR_API_KEY`（可选 `AMIR_MODEL`），否则 `/api/amir/translate`、`/chat`、`/describe` 返回 **`501 Not Implemented`**。

| 方法与路径 | 用途 |
|---|---|
| `POST /api/amir/translate` | 自然语言 → Omcom 命令。 |
| `POST /api/amir/chat` | 多轮对话（可生成命令）。 |
| `POST /api/amir/describe` | 模型对象 → 自然语言描述。 |
| `POST /api/amir/explain` | 校验错误 → 通俗易懂的解释。 |

- `POST /api/amir/translate` 请求体：`{ "text": "Create a 5m wall", "selected_ids": [] }` → `AmirTranslateResponse`：`{ success, commands（JSON 对象数组）, summary, errors }`。
- `POST /api/amir/chat` 请求体：`{ "session_id": null, "text": "...", "selected_ids": [] }` → `AmirChatResponse`：`{ session_id, reply, commands, summary }` — 下轮对话请回传 `session_id`。
- `POST /api/amir/describe` 请求体：`{ "object_ids": [], "format": "text" | "json" }`（null = 整个项目） → `{ "description": "..." }`。
- `POST /api/amir/explain` 请求体：`{ "errors": ["...校验器原始错误..."], "context": "可选" }` → `{ "explanations": ["..."] }`。该端点即使没有 LLM 也可用（返回占位说明）。
- LLM/提供方故障返回 `502`；解析问题 `422`；不支持的意图 / 缺参数 `400`；引用不存在 `404`；冲突 `409`。

```bash
curl -sS -X POST "$BASE/api/amir/translate" -H 'Content-Type: application/json' \
  -d '{"text":"Create a 5m long 3m high wall","selected_ids":[]}'

curl -sS -X POST "$BASE/api/amir/describe" -H 'Content-Type: application/json' \
  -d '{"object_ids":null,"format":"text"}'
```

## 静态文件

未匹配任何 `/api/*` 路由的请求会回退到 `ServeDir::new("web")` — 编译后的 Web 前端从服务器工作目录下的 `web` 目录提供。访问 `http://localhost:3000/` 即打开 UI。

## 状态码与错误处理

- `200` 成功（多数变更端点返回空响应体），目录/系列子资源创建成功为 `201`。
- `204` 用于草图删除和目录定义/参数删除。
- `400` 请求无效（非法枚举、无几何、关系非法、引擎错误以纯文本响应体返回）。
- `404` 现有路由引用的对象不存在；拼写错误的未知路径会落入静态文件回退逻辑而不返回 `404`。
- `500` 内部错误（如 `POST /api/objects` 内部的 Omcom 执行失败）。
- `501` AMIR 未配置；`502` LLM/提供方错误；`422` 解析错误。
- 设置了 `OMNIM_PROJECT` 时，多数变更端点会自动保存项目到该路径（失败仅打日志，不暴露给客户端）。

## 相关文档

- [OMCOM_REFERENCE.md](OMCOM_REFERENCE.md) — `POST /api/omcom` 携带的命令/操作协议。
- [GLOSSARY.md](GLOSSARY.md) — 上述术语。
- 服务端单元与集成测试行为：`crates/server/tests/api_integration.rs`。
