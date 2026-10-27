# Omcom Specification — the Unified Engineering Command System

> Status: current as of the Phase 7 codebase. This document describes what **exists** in
> `proto/omcom.proto`, `crates/omcom`, `crates/amir`, and the web Omcom Console. A planned
> **text** surface over the same operations is a future RFC — see
> [RFC-0001](../RFC/RFC-0001-omcom-text-dsl.md).

## 1. Role: all mutation paths converge on OmniCommand

Omcom (**Om**niM **Com**mand) is the unified command system of OmniM. Every mutation of the
engineering model — whether initiated by a human in the GUI, typed into the Omcom Console,
planned by the AMIR AI layer, or performed by an import pipeline — is expressed as a typed
`OmniCommand` envelope and flows through the same validation + execution pipeline.

```
  GUI (Project Workspace, drag/click/palette)
  Omcom Console (JSON editor + shortcuts) ──┐
  AMIR (natural language → intent → commands) ├─→ OmniCommand ─→ Validator ─→ Executor ─→ Model
  Import / IFC / scripts ───────────────────┘         (typed)      (stages)      (undo/redo,
                                                                                    transactions)
```

### 1.1 What Omcom is today

- It is a **typed command system**, not a text language. The contract lives in
  `proto/omcom.proto`: an `OmniCommand` envelope plus ~50 operation messages, compiled by
  `prost` into Rust types with `serde` support.
- `crates/omcom` contains two components: a **Validator** (multi-stage, stateless) and an
  **Executor** (stateful, with undo/redo and transactions over the project).
- The web **Omcom Console** accepts **JSON** payloads (serialized `OmniCommand`) plus a few
  ad-hoc text shortcuts for convenience; it does not parse a grammar today.
- There is no `nom`-based text parser. An earlier revision of this document described a text
  DSL as shipped; that is not accurate and the text surface is now captured as a design
  proposal in RFC-0001.

### 1.2 Design principles

- **One gateway**: every mutation path produces the same typed envelope, so validation,
  auditing, undo/redo, and permissions behave identically everywhere.
- **Typed, not parsed**: structure is enforced by the protobuf schema and prost codegen;
  malformed commands fail at serialize/deserialize time, not mid-execution.
- **Validated before execution**: the Executor runs the Validator internally; invalid
  commands never mutate state.
- **Reversible**: every executed operation records an inverse `Command` for undo.
- **Auditable**: each envelope carries `id`, `Actor`, `timestamp`, and free-form metadata.

## 2. Command model

### 2.1 The OmniCommand envelope

```protobuf
message OmniCommand {
  string version = 1;
  string id = 2;                    // caller-generated unique id
  Action action = 3;                // one of ~60 enum actions
  Actor actor = 4;                  // who issued it
  Target target = 5;                // what it acts on
  repeated Operation operations = 6; // the actual work items
  ValidationResult validation = 7;
  Timestamp timestamp = 8;
  map<string, string> metadata = 9;
}
```

### 2.2 Actor

`Actor { id, type }` with `ActorType`:

| ActorType | Meaning | Typical source |
|-----------|---------|----------------|
| `USER` | A human acting directly | GUI, Omcom Console |
| `AI_AGENT` | AMIR-generated commands | POST /api/amir/translate |
| `SYSTEM` | Framework-initiated maintenance | server |
| `IMPORT` | File import pipelines | IFC import |

### 2.3 Target

`Target` is a oneof over scope kinds:

| Target kind | Field | Used by |
|-------------|-------|---------|
| `ObjectTarget` | `object_id` | modify/delete/set_property/set_material/… |
| `GeometryTarget` | `geometry_id` | geometry-level operations |
| `SelectionTarget` | `object_ids[]` | batch operations from a GUI selection |
| `SceneTarget` | `scene_id` | scene-level operations |
| `LifecycleTarget` | `record_id` | lifecycle record mutations |

### 2.4 Operation families

`Action` (the envelope enum, ~60 values) and `Operation` (the message oneof, ~51 variants)
describe the actual mutations. Surveyed from `proto/omcom.proto`:

| Family | Action values | Operation messages (abridged) |
|--------|---------------|-------------------------------|
| **Object** | `CREATE_OBJECT`, `MODIFY_OBJECT`, `DELETE_OBJECT`, `COPY_OBJECT`, `MOVE_OBJECT` | `CreateObjectOperation`, `ModifyObjectOperation`, `CopyObjectOperation`, `MoveObjectOperation` |
| **Geometry** | `CREATE_GEOMETRY`, `MODIFY_GEOMETRY`*, `TRANSFORM_GEOMETRY`, `BOOLEAN_OPERATION` | `TransformGeometryOperation` (translate/rotate/scale), `BooleanOperation` (union/subtract/intersect) |
| **Property** | `SET_PROPERTY`, `DELETE_PROPERTY`, `ADD_PROPERTY_SET`, `REMOVE_PROPERTY_SET` | `Property`, `PropertySet`, `DeletePropertyOperation`, `RemovePropertySetOperation` |
| **Relation** | `ADD_CONTAINMENT`, `REMOVE_CONTAINMENT`, `ADD_CONNECTION`, `REMOVE_CONNECTION` | `Containment`, `Connection`, remove counterparts |
| **Batch** | `ARRAY`, `BATCH_DELETE`*, `BATCH_TRANSFORM`* | `ArrayOperation` (linear/rectangular/circular pattern) |
| **Material** | `SET_MATERIAL` | `SetMaterialOperation` |
| **Lifecycle** | `CREATE_LIFECYCLE_RECORD`, `MODIFY_LIFECYCLE_RECORD`, `DELETE_LIFECYCLE_RECORD`, `LINK_RECORD_TO_OBJECT`, `UPDATE_SCHEDULE`, `UPDATE_COST`, `UPDATE_PROGRESS`, `CREATE_WORK_ORDER`, `UPDATE_WORK_ORDER`, `INGEST_SENSOR_DATA` | corresponding `*Operation` messages |
| **Sketch** | `CREATE_SKETCH`, `ADD_SKETCH_ENTITY`, `ADD_SKETCH_CONSTRAINT`, `SOLVE_SKETCH`, `DELETE_SKETCH_ENTITY`, `REMOVE_SKETCH_CONSTRAINT`, `DELETE_SKETCH` | `CreateSketchOperation`, `AddSketchEntityOperation` (line/circle/arc/rect/polygon/ellipse/spline/polyline), `AddSketchConstraintOperation` (19 constraint types) |
| **Feature / direct modeling** | `EXTRUDE_SKETCH`, `REVOLVE_SKETCH`, `CREATE_COMPONENT`, `ADD_FEATURE`, `REMOVE_FEATURE`, `REBUILD_FEATURES` | `ExtrudeSketchOperation`, `RevolveSketchOperation`, `CreateComponentOperation`, `AddFeatureOperation` (FeatureType: extrude, revolve, sweep, loft, boolean, chamfer, fillet, pattern, shell, fusion, lofted_fusion) |
| **Import/Export** | `IMPORT_FILE`, `EXPORT_FILE`, `PARSE_TO_OMCOM` | — |
| **Catalog (元件库)** | `DEFINE_COMPONENT`, `ADD_COMPONENT_PARAM`, `ADD_CONNECTOR`, `ADD_COMPONENT_FEATURE`*, `PUBLISH_COMPONENT`, `LIST_CATALOG`, `DELETE_COMPONENT_DEF` | `DefineComponentOperation`, `AddComponentParamOperation`, `AddConnectorOperation`, `PublishComponentOperation`, `DeleteComponentDefOperation` |
| **Series (系列库)** | `CREATE_SERIES`, `ADD_SELECTOR`, `ADD_ANSWER`, `ADD_SPEC_COMPONENT`, `ADD_PARAM_CONSTRAINT`, `LIST_SERIES`, `DELETE_SERIES`, `LOAD_SERIES` | `CreateSeriesOperation`, `AddSelectorOperation`, `AddAnswerOperation`, `AddSpecComponentOperation`, `AddParamConstraintOperation`, `LoadSeriesOperation` |
| **Validation** | `VALIDATE` | — |

\* = reserved in the schema and in the `Action` enum; the operation message is documented as
reserved-for-future in the proto comments.

A single envelope carries `repeated Operation operations`, so one command can express a
multi-item edit (e.g., add several sketch entities and constraints in one validated step).

A one-command-to-many-operations principle applies around instantiation:
instantiate-from-catalog (`SelectFromSeries` in AMIR) decomposes into
create-object + containment + property-set operations.

## 3. Validation & execution

`crates/omcom` implements both stages over the same `omnim_core::OmniCommand` types.

### 3.1 Validator stages

`Validator::validate(instr, project) -> Vec<String>` is a stateless function of the command
and the current project snapshot. It runs in (conceptual) stages:

1. **Envelope validity** — non-empty `id`, resolvable `Action` from the enum value.
2. **Target validation** — for target-scoped actions, the referenced object / lifecycle record / sketch / series / catalog component must exist in the project or library.
3. **Operation-level validation** — per-message rules:
   - `CreateObject`: valid `ObjectType`, non-empty name, referenced parent exists; spatial bbox overlap against existing objects is detected and logged as a **warning** (overlaps are normal in multi-discipline engineering — pipes cross walls, etc.).
   - `ModifyObject` / `CopyObject` / `MoveObject` / `SetMaterial` / `AddContainment` / `AddConnection` / `TransformGeometry` / `Array` / `Boolean`: all referenced object IDs must exist; material must exist in the material store.
   - `CreateWorkOrder`: asset object must exist.
   - `AddFeature` / `RemoveFeature`: component object and referenced sketch must exist.
   - Catalog/series ops: component def or series must exist in the library / series store; `DefineComponent` rejects empty names; `LoadSeries` requires a path.
4. **Result** — a list of error strings; the Executor refuses to run when the list is non-empty.

### 3.2 Executor, transactions, undo/redo

`Executor` is stateful and holds:

- `undo_stack: Vec<Command>` (max depth 200)
- `redo_stack: Vec<Command>`
- `transaction_stack: Option<Vec<Command>>` — active transaction

Execution flow of `execute(instr, project)`:

1. Run `Validator::validate`; abort with `success: false` on any error.
2. Execute each operation in order against the `Project`, pushing an inverse `Command`
   (snapshot-based: `ObjectSnapshot`, old property values, old geometry, old relations) into
   the per-call `commands` vec.
3. On a mid-command failure (e.g. `add_containment` cannot cycle-check-cleared), call
   `fail_and_rollback`: undo the already-applied commands of this instruction and return an
   error — an instruction is all-or-nothing.
4. Merge the instruction's commands into the undo stack (or into the active transaction).

Transactions:

- `begin_transaction()` opens a nested command buffer; consecutive instructions executed
  inside it are recorded there.
- `commit()` merges the buffer into the undo stack as one undo unit — a whole transaction
  is undone by a single `undo()` call.
- `rollback()` undoes and discards the buffered commands.

Undo/redo mirror each other through the `Command` enum (~45 variants covering objects,
properties, relations, materials, lifecycle, sketches, features, catalog, and series).

### 3.3 Wire surface

- `CRUD` over the server API: `POST /api/omcom` accepts a serialized `OmniCommand` (JSON) and
  returns `ExecutionResult { success, message, created_ids }`.
- The web Omcom Console edits JSON payloads directly (with undo/redo buttons exposed as
  convenience actions), and so do scripts and CI pipelines.

## 4. How each entry point produces commands

### 4.1 GUI (Project Workspace)

Palette and inspector actions build `OmniCommand` envelopes client-side (Actor = `USER`,
Target = current selection or explicit object) and POST them to `/api/omcom`. Batch UI
actions use `SelectionTarget`.

### 4.2 Omcom Console (JSON)

The Console is a JSON command editor: the user composes an `OmniCommand` envelope
(action + operations + target + actor), the console serializes it, and submits it to
`/api/omcom`. A handful of ad-hoc text shortcuts (e.g., undo/redo convenience) exist in the
frontend but are not a parsed grammar. Command lineage (AI vs GUI) is visible through
`Actor`.

### 4.3 AMIR (natural language)

The AMIR crate implements `PromptParser → IntentClassifier → OmcomGenerator`
(`crates/amir/src/generator.rs`):

1. `PromptParser` — natural language + template prompts → a structured `ParsedIntent`
   (font-end JSON intent, machine readability, deterministic rules).
2. `OmcomGenerator::generate(intent) -> Vec<OmniCommand>` — each supported intent is mapped
   to one or more typed OmniCommands (e.g. `Intent::CreateObject` → a
   `CreateObject` OmniCommand with `CreateObjectOperation`; `Intent::SelectFromSeries`
   → create + containment + property operations).
3. The generated commands are validated and executed by the same Omcom pipeline; on
   validation failure the API surfaces the validator's error strings back to the user
   (and AMIR chat can then fix and retry).

AMIR does **not** emit a free-form text DSL today. The original idea of an
"Omcom text language" that AMIR would output is retained as a design direction in
[RFC-0001](../RFC/RFC-0001-omcom-text-dsl.md).

### 4.4 Import pipelines

File imports (IFC and others) are expressed through `IMPORT_FILE` /
`PARSE_TO_OMCOM` actions with Actor = `IMPORT`, so imported changes pass through the same
validation and are undoable.

## 5. Evolution path: the Omcom text surface (future RFC)

Omcom's long-term vision includes a **human-typable text surface** over the same typed
operations: a deterministic, engineer-friendly syntax (e.g. `create` / `sketch` / `feature`
/ `series` verbs) that parses into standard `OmniCommand` envelopes, so that engineers can
script workflows, and so that a model can be round-tripped to a command script and back.

This is intentionally **not** part of the shipped contract in this phase:

- no text grammar exists in `crates/*`;
- the Console takes JSON today;
- AMIR emits typed commands today.

The design sketch (grammar shape, illustrative examples for create/sketch/extrude/library
operations, reverse model→text generation) lives in
[RFC-0001 — Omcom Text DSL](../RFC/RFC-0001-omcom-text-dsl.md). Implementing that RFC would
add a parser crate that folds text → `OmniCommand` and back; the envelope, validator, and
executor described here are unchanged.

## 6. Versioning

The contract is versioned by the envelope:

- `OmniCommand.version` field on every command.
- `Action` enum values and `Operation` oneof variants are additive; removals are breaking.
- Prost/serde codegen is the compatibility toolchain; a command accepted by an older
  executor must keep deserializing in newer versions (proto3 additive rules).

Current proto module version reflects `omnim.proto`/`omcom.proto` in the repo root
`proto/` directory.

## 7. Open questions

- Transaction scope: should transactions be composable (nested) or flat?
- Permission model per actor type per action.
- Persistent audit log of OmniCommand envelopes (currently in-memory undo/redo only).
- Concurrency policy for multiple concurrent clients (per-project lock vs CRDT offshoot).
- Picking the concrete text surface grammar (RFC-0001 open questions apply).
