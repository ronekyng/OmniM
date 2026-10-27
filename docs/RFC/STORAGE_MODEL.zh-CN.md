# OmniM 存储模型

## 1. 概述

OmniM 采用**云优先存储架构**。所有数据都存储在服务器端。客户端不持久化数据 —— 它们使用临时缓存来提升 3D 渲染性能和生产级操作响应速度。

该存储模型的设计目标：
- 服务器端权威存储所有工程和生命周期数据
- 高效地向客户端流式传输几何数据
- 基于版本号的客户端缓存失效机制
- 服务器端保留完整的版本历史和审计追踪
- 多用户协作与实时缓存同步
- 可扩展至数百万对象和数十亿几何顶点

---

## 2. 存储架构

```
╔═══════════════════════════════════════════════════════════╗
║                      Client Side                            ║
║  ┌────────────┐  ┌────────────┐  ┌────────────┐           ║
║  │  Object    │  │  Geometry  │  │  Render    │           ║
║  │  Cache     │  │  Cache     │  │  Cache     │           ║
║  │  (Ephemeral)│  │  (Ephemeral)│  │  (GPU)     │           ║
║  └────────────┘  └────────────┘  └────────────┘           ║
╠═══════════════════════════════════════════════════════════╣
║                   Network / API Layer                       ║
║  ┌────────────┐  ┌────────────┐  ┌────────────┐           ║
║  │ REST/Graph │  │ WebSocket  │  │ Geometry   │           ║
║  │ (Query)    │  │ (Realtime) │  │ Streaming  │           ║
║  └────────────┘  └────────────┘  └────────────┘           ║
╠═══════════════════════════════════════════════════════════╣
║                      Server Side                            ║
║  ┌────────────┐  ┌────────────┐  ┌────────────┐           ║
║  │ API Gateway│  │ Auth Svc   │  │ AMIR Exec  │           ║
║  └────────────┘  └────────────┘  └────────────┘           ║
║  ┌────────────┐  ┌────────────┐  ┌────────────┐           ║
║  │ Object     │  │ Geometry   │  │ Lifecycle  │           ║
║  │ Store      │  │ Store      │  │ Store      │           ║
║  └────────────┘  └────────────┘  └────────────┘           ║
║  ┌────────────┐  ┌────────────┐  ┌────────────┐           ║
║  │ PostgreSQL │  │ Blob       │  │ Search     │           ║
║  │ + PostGIS  │  │ Storage    │  │ Index      │           ║
║  └────────────┘  └────────────┘  └────────────┘           ║
╚═══════════════════════════════════════════════════════════╝
```

---

## 3. 服务器端数据库

### 主数据库：PostgreSQL with PostGIS

**理由**：
- 企业级 ACID 合规性
- PostGIS 扩展用于空间索引和查询
- JSONB 用于灵活的属性存储
- 优秀的并发性,支持多用户访问
- 成熟的复制和备份工具
- 内置全文搜索

**关键扩展**：
- **PostGIS**:空间索引(R-tree)、空间查询、CRS 转换
- **pg_trgm**:基于三元组的模糊文本搜索
- **pg_partman**:针对 AMIR 日志和传感器数据的基于时间的分区

### Blob 存储:S3 兼容对象存储

适用于不适合放入关系数据库的大文件:
- 点云数据(LAS/LAZ 文件)
- 高分辨率纹理
- 导入的源文档(IFC、STEP、DWG 文件)
- 超过 1MB 的几何网格数据

**策略**:大于 1MB 的文件存储在 Blob 存储中;数据库存储元数据和引用。

---

## 4. 服务器端存储

| 存储 | 用途 | 技术 |
|------|------|------|
| **Object Store** | OmniObject 实例及其属性和元数据 | PostgreSQL 表 + JSONB |
| **Geometry Store** | 参数化、BRep 和网格几何数据 | PostgreSQL(小)+ Blob(大) |
| **Relation Store** | 包含、连接、空间和生命周期关系 | PostgreSQL 表 |
| **Lifecycle Store** | 需求、包、活动、资产、工单 | PostgreSQL 表 |
| **AMIR Log** | 所有操作的完整审计追踪 | PostgreSQL 分区表 |
| **Version Store** | 对象和生命周期记录版本历史 | PostgreSQL 表 |
| **Material Library** | 材料定义和属性 | PostgreSQL 表 |
| **Spatial Index** | 边界框查询 | PostGIS R-tree |
| **Search Index** | 跨对象全文搜索 | PostgreSQL FTS 或 Elasticsearch |
| **Blob Store** | 大文件(点云、纹理、文档) | S3 兼容对象存储 |
| **Library 数据** | CatalogStore + ComponentStore + FeatureTreeRegistry | 文件存储（`~/.omnim/library.omnimlib`）；Phase 10 可能迁移到数据库 |
| **Series 数据** | 选择器决策树（项目私有/外部/内置） | 嵌入 `.omnim` / `.omnimseries` 文件 / 编译种子 |

---

## 5. Library 与 Series 存储

### Library（Session 级全局静态）

Library（`CatalogStore` + `ComponentStore` + `FeatureTreeRegistry`）在 session 启动时一次加载到全局静态 `LIBRARY`。

**当前策略（文件存储）**：
- Library 数据持久化为 `~/.omnim/library.omnimlib`（JSONC 格式）
- Session 启动时加载到 `LIBRARY`
- 加载后只读（修改需重载 session）

**未来（Phase 10）**：
- 可能迁移到服务端数据库（PostgreSQL）+ 客户端缓存
- 支持大型目录的增量加载

### SeriesStore（独立模块）

系列可来源于三种渠道：

| 来源 | 存储 | 加载方式 |
|------|------|---------|
| 项目私有 | 嵌入 `.omnim` 项目文件 | 项目反序列化 |
| 外部文件 | 独立 `.omnimseries` 文件（JSONC） | `series load <path>` 命令 |
| 内置种子 | 编译进二进制（`seed_catalog.rs`） | Session 启动（兜底） |

解析优先级：项目私有 > 外部文件 > 内置种子。

---

## 5. 客户端缓存

客户端缓存是**临时的** —— 它可以随时被清除并从服务器重新获取。它存在的唯一目的是提升性能。

### 5.1 Object Cache

| 属性 | 值 |
|------|-----|
| 存储 | 内存(RAM) |
| 淘汰 | LRU(最近最少使用) |
| 最大大小 | 可配置(默认:桌面端 500MB,移动端 100MB) |
| 失效 | 服务器 WebSocket 推送 + 版本标签比较 |
| 预取 | 基于视口(相机视锥体内的对象) |

缓存内容:
- OmniObject 元数据和属性
- 可见对象的生命周期记录
- 选中对象的关系数据

### 5.2 Geometry Cache

| 属性 | 值 |
|------|-----|
| 存储 | 内存(RAM)+ GPU 缓冲区 |
| 淘汰 | LRU + 基于视口(视锥体外的对象被淘汰) |
| 最大大小 | 可配置(默认:桌面端 2GB,移动端 256MB) |
| 失效 | 服务器 WebSocket 推送 + 基于 LOD |
| 流式传输 | 通过 HTTP Range 请求渐进式下载 |

缓存内容:
- 当前 LOD 的网格几何数据(Draco 压缩)
- 用于参数化编辑操作的 BRep 几何数据
- 用于空间查询的包围体层次结构

### 5.3 Render Cache

| 属性 | 值 |
|------|-----|
| 存储 | GPU 显存(VRAM) |
| 淘汰 | 基于视口 + 基于 LOD |
| 最大大小 | 取决于 GPU |

缓存内容:
- 顶点和索引缓冲区
- 材质纹理
- 阴影贴图
- 实例化数据

---

## 6. 缓存同步

### 6.1 读路径(服务器 → 客户端缓存)

```
Client requests data
    ↓
Check client cache (by object ID + version tag)
    ↓
Cache Hit? ──Yes──► Return cached data
    ↓ No
Fetch from server via REST/GraphQL API
    ↓
Store in client cache with version tag
    ↓
Return data
```

### 6.2 写路径(客户端 → 服务器)

```
User performs action (e.g., move wall)
    ↓
Client generates AMIR instruction
    ↓
Optimistic UI: apply change to local cache immediately
    ↓
Submit AMIR to server via AMIR Command Stream
    ↓
Server validates and executes AMIR
    ↓
Server persists changes and increments version tags
    ↓
Server sends confirmation + updated data to client
    ↓
Client updates cache with confirmed version
    ↓
Server broadcasts change notification to other clients via WebSocket
```

### 6.3 缓存失效

| 触发条件 | 操作 |
|----------|------|
| 服务器变更通知(WebSocket) | 使受影响的缓存条目失效 |
| 版本标签不匹配 | 从服务器重新获取 |
| 视口变更(缩放/平移) | 淘汰视锥体外的几何数据,预取新数据 |
| LOD 变更 | 用适当的 LOD 替换几何数据 |
| 手动清除缓存 | 清除所有缓存,按需重新获取 |

---

## 7. 关键设计原则

### 7.1 服务器权威
- 服务器是唯一的数据真相来源
- 所有写入都通过服务器端 AMIR 执行
- 客户端缓存始终是可丢弃的

### 7.2 通过缓存提升性能
- 客户端缓存确保即时 3D 渲染和响应式交互
- 基于视口的预取预测用户需求
- LOD 感知的几何缓存最小化数据传输

### 7.3 协作
- 通过 WebSocket 实时变更通知
- 多用户近实时地看到彼此的变更
- 服务器按顺序解决冲突

### 7.4 可扩展性
- 服务器水平扩展(读副本、负载均衡)
- 几何 CDN 用于大文件分发
- 边缘服务器用于地理分布式团队

---

## 8. 性能目标

| 指标 | 目标 |
|------|------|
| 服务器对象数量 | 10,000,000+ |
| 服务器数据库大小 | 支持 1TB+ |
| 客户端缓存命中率 | 视口数据 > 95% |
| 客户端空间查询延迟 | < 1ms(从缓存) |
| API 查询延迟 | < 100ms(服务器) |
| 几何流式传输 | 100MB 网格 < 500ms |
| WebSocket 通知延迟 | < 200ms |
| AMIR 命令往返 | < 100ms(LAN)/ < 500ms(WAN) |

---

## 9. 备份与完整性

### 服务器端备份
- 每日完整数据库备份
- 持续 WAL 归档,用于时间点恢复
- Blob 存储版本控制和跨区域复制
- 每周完整性检查

### 灾难恢复
1. 从备份恢复数据库
2. 重放 WAL 到故障点
3. 验证对象数量和校验和
4. 重建空间和搜索索引

---

## 10. 待解决问题

1. 对于水平扩展,使用 PostgreSQL 还是分布式 NewSQL(TiDB、CockroachDB)?
2. 不同设备类别的最佳客户端缓存大小是多少?
3. 边缘服务器应该支持写操作还是只读缓存?
4. 如何处理网络中断 —— 有限的离线编辑与延迟同步?
5. 哪种压缩级别为几何流式传输提供最佳的尺寸/性能平衡?
6. AMIR 日志是否应该在一定年龄后归档到冷存储?
