# Component Catalog & Series Library Specification

> Status: current as of the Phase 7 codebase. Extracted from the catalog-related parts of
> [DATA_MODEL.md](DATA_MODEL.md) and verified against code: `crates/model/src/catalog.rs`,
> `series_store.rs`, `seed_catalog.rs`, `seed_materials.rs`, and the web Series Manager.

## 1. Role in the unified model

OmniM separates **what a thing can be** (the catalog) from **what is allowed in this
project** (the series library). This mirrors the E3D/PDMS separation of a CATALOGUE module
from a project specification (SPECON), and it keeps design discipline enforceable by data
rather than by convention.

- **Component Catalog (元件库)** — parametric 3D component definitions (`ComponentDef`),
  stored globally in `Library.catalog` (`CatalogStore`) and shared across projects.
- **Series Library (系列库 / 等级库)** — spec decision trees (`Series`), rooted in a
  `SeriesStore` attached to the project, which approve specific catalog components under
  specific constraints.

Both libraries are seeded by `crates/model/src/seed_catalog.rs` /
`seed_materials.rs` so a fresh session has working structural, piping, concrete, building,
civil, equipment, and rebar families, plus a default material library.

```
Layer 4  Project objects (walls, pipes, columns …)
             │ instantiated via  (CatalogRef traceability)
Layer 3  ┌── SeriesStore (project) ─── decision trees ──┐
         │            │ approves                        │
         └── Library (global) ─ CatalogStore ────────────┘
             ComponentDef + FeatureTree templates + materials
```

## 2. Component Catalog (元件库)

### 2.1 ComponentDef

Defined in `crates/model/src/catalog.rs` (`ComponentDef`); the proto counterpart is
`DefineComponentOperation` / `ComponentDef` in `omcom.proto` / `omnim.proto`.

| Field | Meaning |
|-------|---------|
| `id` | UUID; also referenced by stable human-readable **names** (e.g. `Steel-Beam-IPE`, `IPE240`, `Pipe-CS`) |
| `name`, `category`, `description` | identity; `category` is a slash-path hierarchy (`"Structural/Steel/Beam"`, `"Piping/Pipe"`); prefix-based category listing is supported (`list_by_category` matches a category or any sub-path) |
| `params: Vec<ParamDef>` | typed parametric inputs (§2.2) |
| `feature_tree_id` | link to a `FeatureTreeTemplate` in the `FeatureTreeRegistry` — the recipe that produces real 3D geometry |
| `connectors: Vec<ConnectorDef>` | connection points (§2.3) |
| `allowed_materials`, `default_material` | material gating (references the shared material library, e.g. `S355`, `C30/37`) |
| `ifc_class` | IFC class for export interoperability (e.g. `IfcBeam`, `IfcColumn`) |
| `standard_reference` | governing standard (e.g. `EN 10365`, `ASME B31.3`) |
| `tags`, `domain`, `properties` | discovery/metadata |
| `published` | only published components are listed by `list_published` for design use |

### 2.2 ParamDef (parametric definition)

Each parameter is typed (proto `ParamType` — LENGTH/ etc.), and carries:

- `default_value`, `min_value`, `max_value`, `step` — numeric bounds for UI sliders and
  validation;
- `unit` (e.g. `mm`);
- `formula` — an expression string evaluated in the parametric context (this is how
  connectors and derived geometry reference params, e.g. position `0,0,length`);
- `choices` — enumerated values (for selection-type params);
- `description`.

### 2.3 ConnectorDef (P-point compatibility)

`ConnectorDef` declares a connection point; all spatial fields are **expression strings**
that may reference component parameters:

- `position_x/y/z_expr` — e.g. `"0"`, `"length"` (axes emitter resolves against the final
  param values);
- `direction_x/y/z_expr` — outward axis, default `0,0,1`;
- `bore_expr` — bore diameter expression (piping);
- `connection_type_code` — a connection-type key used for coupling checks (see below);
- `role` — proto `ConnectorRole`: `INLET`(1) / `OUTLET`(2) / `BIDIRECTIONAL`(3) /
  `MOUNT`(4) / `BRANCH`(5).

`ConnectorType` + `connector_code_map()` in `catalog.rs` give a cross-platform taxonomy:

| OmniM code | ConnectorType | Civil3D code | E3D P-point |
|------------|---------------|--------------|-------------|
| `PIPE-END` | PipeEnd | `PIPE-END-C`/`PIPE-END-S` | `P-END` |
| `FLANGE` | FlangeFace | `FLG-FACE` | `P-FLAN` |
| `ELBOW` | ElbowEnd | `ELB-END` | `P-ELB` |
| `TEE-RUN` / `TEE-BRANCH` | TeeRun / TeeBranch | `TEE-RUN`/`TEE-BR` | `P-TEE`/`P-TBR` |
| `STRUCT-END` / `STRUCT-BASE` | StructuralEnd / StructuralBase | `BEM-END`/`COL-BASE` | `P-STRU`/`P-BASE` |
| `NOZZLE` | EquipmentNozzle | `NOZ` | `P-NOZ` |
| `CABLE` | CableEnd | `CBL-END` | `P-CBL` |
| `DUCT` | DuctEnd | — | — |
| `VALVE-END` / `VALVE-STEM` | ValveEnd / ValveStem | `VALVE-END` | `P-VALVE`/`P-VSTEM` |

References between objects are normally constructed only where connector **type codes**
line up — enabling Civil3D-style and E3D-style interchange around the same P-point concept.

### 2.4 Publishing & instantiation flow

1. **define** — `catalog define` (Omcom action `DEFINE_COMPONENT`): create a
   `ComponentDef` with name/category/identity.
2. **parameterize** — `ADD_COMPONENT_PARAM`: append typed `ParamDef`s
   (in-memory `ParamDef` builder API uses `with_default/with_range/with_step/with_unit/with_choices`).
3. **add connectors** — `ADD_CONNECTOR`: append `ConnectorDef`s with expression-driven
   P-points.
4. **attach a feature tree** — via `seed_feature_trees`/`FeatureTreeRegistry`: the
   component's name is linked to a `FeatureTreeTemplate` (e.g. `Steel-Beam-IPE` → IPE
   template), ordered `FeatureTemplateStep`s that create sketches and features.
5. **publish** — `PUBLISH_COMPONENT` flips `published=true`; only published defs are
   usable by designers.
6. **instantiate** — a design action creates one or more project OmniObjects via the
   component's feature tree. The instance records a `CatalogRef`:

```rust
pub struct CatalogRef {
    pub component_def_id: String,
    pub series_id: String,          // approving series, if any
    pub selector_path: Vec<String>, // path through the series decision tree
    pub param_values: HashMap<String, f64>, // final resolved instance params
}
```

   which preserves traceability (which series approved it, which branch was taken, which
   values were used) and supports regeneration when the definition or series changes.

### 2.5 Built-in library (seeds)

`seed_catalog.rs` seeds, among others:

- Structural steel: `Steel-Beam-IPE` + `IPE240`, columns `Steel-Column-HEA`, `Steel-Column-HEB`, `Steel-Column-HEM`, `Steel-Channel-UPE/UPN`, `Steel-T-Section`, `Steel-CHS`/`Steel-SHS` hollow sections, angle sections.
- Piping: `Pipe-CS` (carbon steel), `Elbow-LR-CS`, … with DN ranges and bore formulas.
- Concrete: `Concrete-Beam-Rect`, `Concrete-Column-Rect` / `Concrete-Column-Circular`,
  `Concrete-Footing-Isolated/Raft/Strip`.
- Building: standard walls, doors, windows (`Concrete-Wall`, `Door-Single`,
  `Window-Fixed`, …).
- Civil roads, equipment stubs, rebar components, and domain-extension components.
- Materials via `seed_materials.rs`: stable human-readable IDs (`C30/37`, `S355`,
  `S235`, `XPS-100`, …) referenced from `allowed_materials` / `default_material`.
- Series (§3): `CS150`, `CS300`, `Steel-Q355`, `Concrete-C30`, `SS300`, and a GB-standard
  highway series (`Highway-GB`).

## 3. Series Library (系列库 / 等级库)

### 3.1 Series structure

Defined in `crates/model/src/series_store.rs` (mirrors AVEVA SPECON):

```rust
pub struct Series {
    pub id, name, description,
    pub domain, standard_reference,
    pub root_selector: Option<Selector>,  // decision tree root
    pub applicable_phases: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub source: String,  // "project" | "external" | "seed"
}
```

### 3.2 Decision-tree semantics: Question → Answer → Selector → SpecComponent

- `Selector { id, question, answers }` — one question node; answers are its branches.
- `Answer { id, value, is_numeric_range, range_min, range_max, outcome }` —
  either fixed-string matching (case-insensitive) or numeric-range matching
  (`min <= v <= max`); `outcome` is either
  `AnswerOutcome::NextSelector(Selector)` or `AnswerOutcome::Component(SpecComponent)`.
- `SpecComponent { id, component_def_id, param_constraints, material, connection_type_code, design_params }`
  — the leaf: an approved catalog component with series-specific parameter restrictions
  and default material/connection code.
- `ParamConstraint { param_name, default_value, min_value, max_value, step, fixed_value }`
  — narrows the component's `ParamDef` ranges; may pin a param with `fixed_value` (a
  string / expression form that takes precedence for open params).

`SelectionEngine::select(series, answers)` walks the tree:

1. Start at `root_selector`; the caller supplies an answer string per selector level.
2. Match: numeric-range answers parse the input as f64 and test the range; string answers
   compare case-insensitively.
3. On `NextSelector` descend with the next answer; on `Component` return
   `SelectionResult { spec_component, selector_path, component_def_id }`.
4. Missing answers or unmatched values produce a descriptive error naming the failing
   selector/question.

Beyond direct selection, `all_reachable_components()` walks the whole tree to collect all
def-ids reachable from any loaded series — this powers the spec gate (§3.4).

### 3.3 Storage, resolution priority, and the web Series Manager

- `SeriesStore` holds series keyed by UUID; `find_by_name` supports human series names
  (`CS150`).
- **Source priority**: inserting a series with a same-id higher-priority source wins —
  `project(3) > external(2) > seed(1)`. A seed series can therefore be overridden at
  project level; a seed cannot overwrite a project series.
- Files: `save_to_file` (pretty JSON) and `load_from_file` (JSONC/JSON5 accepted, loaded
  series marked `source: "external"`); extension `.omnimseries`.
- The web **Series Manager** page (`web/src/series/SeriesManagerPage.tsx`) edits the
  decision trees (selectors, answers, spec components, constraints) and loads/saves
  `.omnimseries` files through the library endpoints.

### 3.4 Spec enforcement rule (designers limited to active series)

`crates/model/src/project.rs::check_series_gate` enforces the classic PDMS/SPECON rule —
**"only series-approved components are selectable when a series is active"**:

- **Series loaded + component reachable** from any loaded series' decision tree → allowed.
- **Series loaded + component NOT reachable** → rejected with
  `series gate: component '…' is not reachable from any loaded series. Load the appropriate
  series or use a series-approved component.`
- **No series loaded** → open mode; all catalog components are allowed.

Reachability is checked both by component UUID and by name (spec components may store
either identifier; matching is case-insensitive as a fallback).

### 3.5 Omcom operations for series

Series authoring happens through the same Omcom pipeline (see
[OMCOM_SPEC.md](OMCOM_SPEC.md) §2.4): `CREATE_SERIES`, `ADD_SELECTOR`, `ADD_ANSWER`,
`ADD_SPEC_COMPONENT`, `ADD_PARAM_CONSTRAINT`, `DELETE_SERIES`, `LOAD_SERIES`
(`path` + `source` = `"standalone" | "library"`), and `LIST_SERIES`. The Validator checks
series existence (except `CREATE_SERIES`), non-empty `LoadSeries` paths; each executed
series mutation takes an inverse `Command` for undo.

## 4. Worked examples

### 4.1 Structural steel beam selection (`Steel-Q355` seed series)

Built-in seeds (abridged):

```text
Series Steel-Q355 (domain structural, EN 1993)
  root selector q1: "element_type?"
    answer "beam"  → next selector q2_material
      answer "S355" → SpecComponent Steel-Beam-IPE, material S355
    answer "column" → next selector q3_section
      answer "HEA"  → SpecComponent Steel-Column-HEA
      answer "HEB"  → SpecComponent Steel-Column-HEB
```

Selection (`SelectionEngine::select(series, &["beam", "S355"])`) → `Steel-Beam-IPE`,
selector path `[q1, q2_material]`, approved material `S355`.

### 4.2 Piping series `CS150`

```text
Series CS150 (domain plant, ASME B31.3)
  root selector gt: "GT?"                                       (gross type)
    answer "PIPE"    → next selector sch: "schedule?"
       answer "STD"  → SpecComponent Pipe-CS, material A106-B,
                       constraints: DN min 15, max 600, step …
    answer "ELBOW"  → SpecComponent Elbow-LR-CS, material A106-B
```

Instantiation: answers `["PIPE", "STD"]` select `Pipe-CS`; the executor creates an
OmniObject with a resolved DN (e.g. DN 100), records `CatalogRef { component_def_id,
series_id: CS150, selector_path: [gt, sch], param_values { dn: 100.0, length: 6000.0 } }`,
and links containment/property operations under the same OmniCommand. While CS150 is
loaded, attempting to instantiate a beam component (not reachable in this series) is
**rejected** by the series gate.

### 4.3 Defining a new catalog component

```text
DEFINE_COMPONENT    { name: "Pipe-CS", category: "Piping/Pipe", ifc_class: "IfcPipe",
                      standard_reference: "ASME B36.10", domain: "plant" }
ADD_COMPONENT_PARAM { DN   min 15   max 600  default 100 mm }
ADD_COMPONENT_PARAM { schedule … choices ["STD", "XS", "160"] }
ADD_CONNECTOR       { id "start", role INLET,  pos "0,0,0",           dir "0,0,1", bore "dn" }
ADD_CONNECTOR       { id "end",   role OUTLET, pos "0,0,length",      dir "0,0,1", bore "dn" }
PUBLISH_COMPONENT   → Pipe-CS now listed for designers
```

## 5. Open questions

- Should `connector_code_map` be data-driven (config file) instead of code?
- Series versioning (standard revision history) — currently overwritten by id.
- Full composition-tree connectors (branch tee-run/branch pairing) — currently taxonomy
  level only.
- Deleting a ComponentDef that is referenced from a series: cascade vs. reject.
