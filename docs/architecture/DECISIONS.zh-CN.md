# OmniM — 架构决策记录(Architecture Decision Records)

 OmniM 的架构决策记录(ADR)索引。

## 我们如何记录决策

我们采用**精简 MADR 风格**。每条 ADR 是一条编号且不可变的记录,放在本目录
(`docs/architecture/DECISIONS.md` 即索引;目前各 ADR 内联在本文件中——当某条超过一页时,
再拆分为 `adr/NNNN-name.md`)。

每条记录的格式:

- **编号与标题**(编号稳定,永不复用)
- **状态** —— 取值之一:Proposed(提案)/ Accepted(已接受)/ Superseded by ADR-XXXX(已被取代)/ Deprecated(已废弃)
- **Context(背景)** —— 迫使做出决策的因素(约束条件、客观事实、当下的代码现实)
- **Decision(决策)** —— 源码树或协议中实际做出的、可验证的那一个选择
- **Consequences(后果)** —— 什么变得更好、什么变得更糟、什么需要在将来重新决策

规则:

1. ADR 描述**已存在的东西**,而不是我们希望存在的东西。被计划阻塞的事项更适合先写成
   [RFC](../RFC/),直到代码把决策真正落实。
2. 被取代的 ADR **永不删除** —— 标记为 Superseded,内容原样保留。
3. 每条 ADR 至少点名一个体现它的代码文件或符号,让读者可以验证决策是真实的。

---

## ADR 索引

| ADR | 标题 | 状态 |
|---|---|---|
| [ADR-0001](#adr-0001-protobuf-仅作类型-schema传输层-json不使用-grpc) | Protobuf 仅作类型 schema(传输层 JSON,不使用 gRPC) | Accepted |
| [ADR-0002](#adr-0002-二维草图约束采用-slvs-库) | 二维草图约束采用 slvs 库 | Accepted |
| [ADR-0003](#adr-0003-brep-建模采用-truck-内核) | BRep 建模采用 truck 内核 | Accepted |
| [ADR-0004](#adr-0004-json-文件持久化单-project-rwlock) | JSON 文件持久化(单 Project,RwLock)及其已知局限 | Accepted |
| [ADR-0005](#adr-0005-手写-ifc-解析器而非-ifc-crate) | 手写 IFC 解析器,而非 ifc crate | Accepted |
| [ADR-0006](#adr-0006-所有变更路径统一走-executorvalidator) | 所有变更路径统一走 Executor/Validator | Accepted |

---

### ADR-0001. Protobuf 仅作类型 schema(传输层 JSON,不使用 gRPC)

**状态:** Accepted

**背景**
OmniM 的数据模型需要在 Rust 引擎与 JS 浏览器客户端间共享,并尽量避免运行时 schema 分歧。
传统答案是二进制 protobuf 走 gRPC;但它要求每种客户端语言都跑代码生成、需要 HTTP/2 服务器栈、
线上载荷难以人工排查,也与当前简洁的开发循环(Vite + Vite 代理 + Axum)相冲突。

**决策**
把 `.proto` 文件当作**类型真相的唯一来源**(`proto/omnim.proto`, `proto/omcom.proto`),
用 `prost` 生成 Rust 类型(`omnim-omcom`/`omnim-core`),并在 `web/src/shared/types.ts` 里镜像
**TypeScript 类型**。线上所有 HTTP 载荷一律是 **serde-JSON** 字节,而非 protobuf 二进制。
**任何地方都不使用 gRPC。** 那些与 protobuf 枚举难以自然映射的领域类型直接留在 Rust+serde 侧
(`omnim-model`/`omnim-sketch`),不进 proto。

**后果**
- (+) 一处集中、人类可读的 schema 位置;diff 与评审都容易。
- (+) 用 curl / 浏览器 DevTools 排查载荷非常轻松。
- (−) 载荷比二进制 protobuf 更大。
- (−) 传输层没有自动 schema 校验;漂移只能靠手工维护的 TS 类型兜底 —— 这是已知的代价。
- (−) proto 级枚举穷举与字段号安全性在运行时没有强制校验。
- 若云时代需要二进制传输,类型层已经就位(规划的演进方向)。

---

### ADR-0002. 二维草图约束采用 slvs 库

**状态:** Accepted

**背景**
草图编辑器需要真实的几何约束求解(共点、距离、平行、半径,作用于线、圆、弧、工作平面)。
自研求解器的算法负担很重;Rust 生态有 `slvs` —— 对久经考验的 SolveSpace 几何内核的惯用绑定。

**决策**
使用 `slvs = "0.6.0"` crate(`omnim-sketch/src/solver.rs`)驱动二维约束求解。草图实体建模为
`slvs` 的 handle(`EntityHandle<Point>` / `LineSegment` / `Circle` / `ArcOfCircle` / `Workplane`),
外加 OmniM 自己在 `lib.rs`/`profile.rs` 里的包装数据。OmniM 不自己实现约束数学。

**后果**
- (+) 无需自己持有算法即可获得生产级二维约束求解。
- (+) OmniM 草图概念与 SolveSpace 原语之间映射清晰。
- (−) 概念渗漏:solver 的实体类别进入了 OmniM 的类型词汇。
- (−) 绑定于求解器的能力与语义;不支持的约束类型需要上游升级。
- (−) slvs 的 handle 无法直接序列化进 `.omnim` JSON —— 草图必须按 OmniM schema 持久化,
  加载时再重建 handle。

---

### ADR-0003. BRep 建模采用 truck 内核

**状态:** Accepted

**背景**
工程对象需要实体建模(拉伸、轮廓成体、布尔、壳/实体/线框)以及用于显示的网格剖分。
Rust 开源选项中,`truck` 提供 B-rep;其余方案更小、更不完整;OCCT 绑定存在,但 C++/FFI
复杂度与纯 Rust crate 布局相冲突。

**决策**
在 `omnim-model/src/brep.rs` 的全部实体/BRep 工作与 `mesh_gen.rs` 的网格生成中,采用
**truck** 家族 —— `truck-modeling`, `truck-polymesh`, `truck-topology`, `truck-base`。
OmniM 不重写 CSG 或 NURBS 数学。

**后果**
- (+) 拥有真正的 BRep 语义(Shell/Solid/Wire),却不绑定庞大的 C++ 内核。
- (+) `truck-polymesh` 的产物与 three.js 需要的网格交付格式天然吻合。
- (−) 受限于 truck 的功能范围;一些 CAD 级操作在上游仍未成熟或缺失。
- (−) `truck_modeling` 各版本会传递引入;跨 crate 升级必须一起做。

---

### ADR-0004. JSON 文件持久化(单 Project,RwLock)及其已知局限

**状态:** Accepted

**背景**
当前部署目标是单用户本地服务器。最简单正确的持久化就是:把整个应用状态同步到一个文件里。
多用户、数据库版本历史、分支管理等需求都是**规划中**,尚未实现
(见 [PLATFORM_ARCH.zh-CN.md §9](PLATFORM_ARCH.zh-CN.md#9-未来方向尚未实现))。

**决策**
把**整个 `Project`** 持久化为单个 `.omnim` JSON 文件。启动时从 `OMNIM_PROJECT`(默认仓库根目录的
`project.omnim`)读取。每次被接受的变更后,服务器再次整体落盘(`save_path` 调 `Project::save`);
没有增量日志、没有 WAL、没有快照轮转。`Arc<RwLock<Project>>` 守护状态
(见 `server/src/lib.rs`, `model/src/project.rs`)。使用默认占位文件时 `save_path` 为 `None`,
即运行中的服务器可以不自动保存。

**后果**
- (+) 极易检查、diff、拷贝,甚至手工编辑;零基础设施。
- (+) 即使版本不匹配,状态也可读取(serde 默认值填补空缺)。
- (−) 每次变更整体重写文件是 O(项目大小);写盘中途崩溃可能留下截断文件 ——
  目前尚未使用原子重命名的落盘模式。
- (−) 没有多进程安全:两个服务器共享同一文件会互相破坏。
- (−) 没有真正的历史。撤销/重做只存在于 `Executor` 的内存状态。
- **这些局限把 OmniM 当前限定在桌面/本地部署形态。** 云端/DB 支持是规划,不是现状。

---

### ADR-0005. 手写 IFC 解析器,而非 ifc crate

**状态:** Accepted

**背景**
OmniM 必须读入并产出 IFC/STEP 文件(IFC 2x3 / IFC 4 子集)。生态里的 IFC crate 要么只覆盖读取路径,
要么拖入繁重依赖,要么把领域限定在特定 schema;它们也让 OmniM 需要的"部分覆盖"语义难以实现
(我们只关心能消费/产出的那些实体)。

**决策**
在 `omnim-ifc` 中**手工编写**解析器与序列化器(不依赖任何 IFC crate)。解析器映射我们选定的
IFC 2x3/4 记录与几何子集,在 `lib.rs`/`geometry.rs` 中显式控制。不支持的实体被**跳过**,
而不是整体失败。导入内容被转换成 `OmniCommand`,不会绕过变更管线(见 ADR-0006)。

**后果**
- (+) 零外部 IFC 依赖;对产生/消费哪个子集有精确控制。
- (+) 错误信息可读;映射与我们的对象模型直接对齐。
- (−) 持续成本:IFC 覆盖面只随我们手工编码的速度增长。
- (−) 对非标准 IFC 生产者的健壮性有限(解析器以 `fixtures/ifc/` 的真实样例文件测试,
  如 `simple.ifc`, `test_building.ifc`, `part.ifc`)。
- 若 schema 覆盖需求超出可手工维护的范围,应重新审视此决策。

---

### ADR-0006. 所有变更路径统一走 Executor/Validator

**状态:** Accepted

**背景**
变更的来源有多个:REST 处理器、web UI 中的 Omcom 控制台、以及 AI 意图引擎(AMIR 生成的
`OmniCommand`)。如果每种来源直接改 `Project`,会导致撤销语义不一致、校验逻辑重复,
并给 AI 幻觉一个悄然篡改状态的通道。

**决策**
变更管线**只有一条**:构造 `OmniCommand` → `Validator::validate(&cmd, &Project)` →
`Executor::execute(&cmd, &mut Project)` → 记录撤销/重做历史。**每个**来源
(`omnim-server` 处理器、`omnim-omcom` 控制台路径、`omnim-amir` 生成输出)都构造
`OmniCommand`,并走同一个 Validator/Executor。任何路径都不得绕道写 `Project`(只读助手除外)。

**后果**
- (+) 无论变更来自何方,撤销/重做语义完全一致。
- (+) AI 生成的命令与人工命令走同一套校验:**幻觉出来的或危险的命令会大张旗鼓地失败**,
  错误经 `amir::feedback` 回传给 AI,而不是腐蚀状态。
- (+) 以后添加业务规则或审计钩子只需要改一处。
- (−) 每次变更多一次命令构造 + 校验的开销 —— 在单 Project 内存模型的尺度上可接受。
- (−) 服务器处理器代码更啰嗦(要构造命令结构体);为正确性接受的取舍。
