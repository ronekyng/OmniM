# OmniM Storage Model

## 1. Overview

OmniM uses a **cloud-first storage architecture**. All data is stored on the server side. Clients do not persist data — they use ephemeral caching for 3D rendering performance and production-grade operation responsiveness.

The storage model is designed for:
- Server-side authoritative storage for all engineering and lifecycle data
- Efficient geometry streaming to clients
- Client-side caching with version-based invalidation
- Complete version history and audit trail on the server
- Multi-user collaboration with real-time cache synchronization
- Scalable to millions of objects and billions of geometry vertices

---

## 2. Storage Architecture

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

## 3. Server-Side Database

### Primary Database: PostgreSQL with PostGIS

**Rationale**:
- Enterprise-grade ACID compliance
- PostGIS extension for spatial indexing and queries
- JSONB for flexible property storage
- Excellent concurrency for multi-user access
- Mature replication and backup tooling
- Full-text search built-in

**Key Extensions**:
- **PostGIS**: Spatial indexing (R-tree), spatial queries, CRS conversion
- **pg_trgm**: Trigram-based fuzzy text search
- **pg_partman**: Time-based partitioning for AMIR log and sensor data

### Blob Storage: S3-Compatible Object Storage

For large files that don't fit in a relational database:
- Point cloud data (LAS/LAZ files)
- High-resolution textures
- Imported source documents (IFC, STEP, DWG files)
- Geometry mesh data exceeding 1MB

**Strategy**: Files > 1MB stored in blob storage; database stores metadata and references.

---

## 4. Server-Side Stores

| Store | Purpose | Technology |
|-------|---------|------------|
| **Object Store** | OmniObject instances with properties and metadata | PostgreSQL tables + JSONB |
| **Geometry Store** | Parametric, BRep, and mesh geometry data | PostgreSQL (small) + Blob (large) |
| **Relation Store** | Containment, connection, spatial, and lifecycle relationships | PostgreSQL tables |
| **Lifecycle Store** | Requirements, packages, activities, assets, work orders | PostgreSQL tables |
| **AMIR Log** | Complete audit trail of all operations | PostgreSQL partitioned table |
| **Version Store** | Object and lifecycle record version history | PostgreSQL tables |
| **Material Library** | Material definitions and properties | PostgreSQL tables |
| **Spatial Index** | Bounding box queries | PostGIS R-tree |
| **Search Index** | Full-text search across objects | PostgreSQL FTS or Elasticsearch |
| **Blob Store** | Large files (point clouds, textures, documents) | S3-compatible object storage |
| **Library Data** | CatalogStore + ComponentStore + FeatureTreeRegistry | File-based (`~/.omnim/library.omnimlib`); Phase 10 may migrate to database |
| **Series Data** | Spec decision trees (project-private / external / built-in) | Embedded in `.omnim` / `.omnimseries` files / compiled seed |

---

## 5. Library & Series Storage

### Library (Session-Level Global Static)

The Library (`CatalogStore` + `ComponentStore` + `FeatureTreeRegistry`) is loaded once per session into the global static `LIBRARY`.

**Current strategy (file-based)**:
- Library data persisted as `~/.omnim/library.omnimlib` (JSONC format)
- Loaded into `LIBRARY` at session start
- Read-only after loading (modifications require reloading the session)

**Future (Phase 10)**:
- May migrate to server-side database (PostgreSQL) with client-side caching
- Supports large catalogs with incremental loading

### SeriesStore (Independent Module)

Series can originate from three sources:

| Source | Storage | Loaded By |
|--------|---------|-----------|
| Project-private | Embedded in `.omnim` project file | Project deserialization |
| External file | Standalone `.omnimseries` file (JSONC) | `series load <path>` command |
| Built-in seed | Compiled into binary (`seed_catalog.rs`) | Session start (fallback) |

Resolution priority: project-private > external file > built-in seed.

---

## 5. Client-Side Cache

The client cache is **ephemeral** — it can be cleared and re-fetched from the server at any time. It exists solely for performance.

### 5.1 Object Cache

| Property | Value |
|----------|-------|
| Storage | In-memory (RAM) |
| Eviction | LRU (Least Recently Used) |
| Max Size | Configurable (default: 500MB on desktop, 100MB on mobile) |
| Invalidation | Server WebSocket push + version tag comparison |
| Prefetch | Viewport-based (objects in camera frustum) |

Cached content:
- OmniObject metadata and properties
- Lifecycle records for visible objects
- Relationship data for selected objects

### 5.2 Geometry Cache

| Property | Value |
|----------|-------|
| Storage | In-memory (RAM) + GPU buffers |
| Eviction | LRU + viewport-based (objects outside frustum evicted) |
| Max Size | Configurable (default: 2GB on desktop, 256MB on mobile) |
| Invalidation | Server WebSocket push + LOD-based |
| Streaming | Progressive download via HTTP Range requests |

Cached content:
- Mesh geometry (Draco-compressed) for current LOD
- BRep geometry for parametric editing operations
- Bounding volume hierarchy for spatial queries

### 5.3 Render Cache

| Property | Value |
|----------|-------|
| Storage | GPU memory (VRAM) |
| Eviction | Viewport-based + LOD-based |
| Max Size | GPU-dependent |

Cached content:
- Vertex and index buffers
- Material textures
- Shadow maps
- Instancing data

---

## 6. Cache Synchronization

### 6.1 Read Path (Server → Client Cache)

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

### 6.2 Write Path (Client → Server)

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

### 6.3 Cache Invalidation

| Trigger | Action |
|---------|--------|
| Server change notification (WebSocket) | Invalidate affected cache entries |
| Version tag mismatch | Re-fetch from server |
| Viewport change (zoom/pan) | Evict out-of-frustum geometry, prefetch new |
| LOD change | Replace geometry with appropriate LOD |
| Manual cache clear | Clear all cache, re-fetch on demand |

---

## 7. Key Design Principles

### 7.1 Server-Authoritative
- Server is the single source of truth
- All writes go through server-side AMIR execution
- Client cache is always disposable

### 7.2 Performance via Caching
- Client cache ensures instant 3D rendering and responsive interactions
- Viewport-based prefetching anticipates user needs
- LOD-aware geometry caching minimizes data transfer

### 7.3 Collaboration
- Real-time change notifications via WebSocket
- Multiple users see each other's changes in near real-time
- Server resolves conflicts sequentially

### 7.4 Scalability
- Server scales horizontally (read replicas, load balancing)
- Geometry CDN for large file distribution
- Edge servers for geographically distributed teams

---

## 8. Performance Targets

| Metric | Target |
|--------|--------|
| Server object count | 10,000,000+ |
| Server database size | 1TB+ supported |
| Client cache hit rate | > 95% for viewport data |
| Client spatial query latency | < 1ms (from cache) |
| API query latency | < 100ms (server) |
| Geometry streaming | < 500ms for 100MB mesh |
| WebSocket notification latency | < 200ms |
| AMIR command round-trip | < 100ms (LAN) / < 500ms (WAN) |

---

## 9. Backup and Integrity

### Server-Side Backup
- Daily full database backup
- Continuous WAL archiving for point-in-time recovery
- Blob storage versioning and cross-region replication
- Weekly integrity checks

### Disaster Recovery
1. Restore database from backup
2. Replay WAL to point of failure
3. Verify object counts and checksums
4. Rebuild spatial and search indexes

---

## 10. Open Questions

1. PostgreSQL vs distributed NewSQL (TiDB, CockroachDB) for horizontal scaling?
2. What is the optimal client cache size for different device classes?
3. Should edge servers support write operations or read-only caching?
4. How to handle network interruptions — limited offline editing with deferred sync?
5. What compression level provides best size/performance trade-off for geometry streaming?
6. Should the AMIR log be archived to cold storage after a certain age?
