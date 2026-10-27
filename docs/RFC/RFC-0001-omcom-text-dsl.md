# RFC-0001: Omcom Text DSL

- **Status**: Draft
- **Related**: [Omcom Specification](../specification/OMCOM_SPEC.md) · [DATA_MODEL](../specification/DATA_MODEL.md)

## 1. Context

This RFC restores the **original text-DSL vision** of Omcom — a vision that was once
incorrectly documented as shipped, and which this document now formalizes as a design
proposal.

What actually exists today (see the Omcom Specification):

- Omcom is a typed command system: `proto/omcom.proto` defines the `OmniCommand` envelope
  plus operation messages, compiled via prost into serde-capable Rust types.
- `crates/omcom` is a Validator + Executor with undo/redo and transactions.
- The web Omcom Console takes JSON.
- AMIR emits typed `OmniCommand` envelopes, not text.

The original vision, in its full pipeline form, was:

```
Natural Language → AMIR (intent) → Omcom Text DSL → nom Parser → Executor → Model
                                                        ↑ reverse:
                                        Model → text (bidirectional parsing)
```

…with a **command window entry** in the UI where engineers type commands directly, and a
**reverse parser** that renders model state back into Omcom text ("model → text") for
diffing, review, and version control. The typed command system already covers the back half
of that pipeline (Executor → Model); this RFC proposes the front half: a deterministic text
syntax over the same operations.

## 2. Motivation

1. **Keyboard-first engineers.** Modeling tools of record (AutoCAD, PDMS/PML, Specon-driven
   workflow tools) reward command-line fluency. The command window is faster than dialog
   navigation for routine edits.
2. **Scriptability & CI.** A text format is naturally diffable, versionable, and can be
   executed as a `.omcom` script file — the JSON envelope is verbose for this.
3. **AMIR readability.** If AMIR emits text, humans can review/edit AI output in the command
   window before execution; today AI output is opaque JSON.
4. **Round-trip (bidirectional parse).** Model → text rendering enables: project "changelogs"
   in reviewable text, instruction-level version control, and reproducible regeneration of a
   model from its command log.
5. **Telemetry/Audit friendliness.** One-line commands are compact per-action human-readable
   audit records.

## 3. Proposal sketch

### 3.1 Design constraints

- The text DSL is a **deterministic** front-end: text → typed `OmniCommand` → Validator →
  Executor. No current-tense/free-form interpretation at execution time.
- **Lossless** in both directions: any executable text must round-trip text ↔ envelope with
  identical semantics; any model state must be renderable to a text command log that
  re-executes to a semantically identical model.
- Additive grammar; the schema (`omcom.proto`) is not changed by the DSL itself.

### 3.2 General syntax (grammar sketch — illustrative)

```
instruction      = verb, { word }, [ flags ] ;
verb             = "create" | "modify" | "delete" | "copy" | "move"
                 | "sketch" | "constraint" | "solve" | "feature" | "component"
                 | "prop" | "material" | "parent" | "connect"
                 | "lifecycle" | "schedule" | "cost" | "progress"
                 | "catalog" | "series" | "undo" | "redo" ;

flags            = "--", name, "=", value, { [ " ", "--", name, "=", value ] } ;
value            = point2d | point3d | number | identifier | string ;
point2d          = number, ",", number ;
point3d          = number, ",", number, ",", number ;
number           = ["-"], digit, { digit }, [ ".", digit, { digit } ] ;
identifier       = (letter|"_"), { letter|digit|"_"/"-" } ;
```

Every instruction maps 1:1 (or 1:n where one verb decomposes per the catalog instantiate
rule documented in OMCOM_SPEC §2.4) to typed operation messages.

### 3.3 Illustrative examples

> All examples below are illustrative — they are not a committed grammar. The final
> surface syntax is subject to the human-typability / LLM-parseability trade-offs identified
> in Open Questions.

#### Object creation

```text
create wall std_wall --geom box 5.0,0.3,3.0 --pos 0,0,1.5
create beam main_beam --component Steel-Beam-IPE --param length=6000mm
create pipe steam_p1 --series CS150 --answers DN=100,schedule=STD
```

#### Sketch

```text
sketch new wall_s1 --plane 0,0,0 0,0,1
sketch add wall_s1 line 0,0 10,0
sketch add wall_s1 circle 5.0,1.8 0.5
sketch constr wall_s1 horizontal 1
sketch constr wall_s1 length 1 10.0
sketch solve wall_s1
```

#### Feature Extrude

```text
feature add wall_s1 extrude --depth 3.0 --dir 0,0,1
```

or a single-verb equivalent (mapping to `ExtrudeSketchOperation`):

```text
extrude wall_s1 --depth 3.0 --dir 0,0,1
```

#### Feature tree

```text
component create beam_001
feature add beam_001 extrude wall_s1 --depth 6.0
feature add beam_001 revolve shaft_s1 --axis 0,0,0 0,0,1 --angle 360
feature remove beam_001 feat_2
feature rebuild beam_001
```

#### Catalog (元件库)

```text
catalog define IPE240 --category Structural/Steel/Beam --ifc IfcBeam
catalog param IPE240 height --type length --default 240 --range 80,600 --unit mm
catalog connector IPE240 start --pos 0,0,0 --dir 0,0,1 --role inlet
catalog connector IPE240 end --pos 0,0,length --dir 0,0,1 --role outlet
catalog publish IPE240
catalog list --category Structural/Steel
```

#### Series (系列库/等级库)

```text
series create CS150 --domain plant --standard "ASME B31.3"
series selector CS150 q1 --question "element_type?"
series answer CS150 q1 PIPE
series component CS150 q1 PIPE --comp Pipe-CS --material A106-B
series constraint CS150 Pipe-CS --param DN --range 15,600
series list
```

#### AMIR integration (final shape)

```
"Create a 10m long, 3m high wall at the origin"
        ↓ AMIR
create wall wall_1 --geom box 10.0,0.2,3.0 --pos 5.0,0,1.5
        ↓ text parser (nom) → OmniCommand → Validator → Executor → Model
```

### 3.4 Reverse parsing (model → text)

For each object / sketch / feature / relation in the project, a `to_text` pass renders the
minimal sufficient command sequence (create → modify → connect) ordered by dependency graph.
Failure to parameterize arbitrary imported meshes degrades them to explicit geometry verbs on
supported primitives only; meshes that cannot be parameterized are exported as no-ops with a
warning in the text log.

## 4. Risks & Open Questions

- **Grammar design speed offset** — natural tense vs deterministic symmetry: a text query
  front-end that can evolve the surface quickly may cost more design iteration in both
  directions (parse → surface, surface → parse). Meaning: the text surface must grow without
  breaking the schema, but every verb addition needs both directions covered.
- **Reverse parsing truthfulness** — how faithfully does model→text contain
  implicitly-checkable semantics (e.g., feature order, parameter provenance, series gate)?
  Lossy reverse output breaks the round-trip invariant.
- **AMIR coupling** — if AMIR emits text, error rates compound at both ends (intent → text
  is LLM-stochastic; text → envelope is deterministic but error surfaces at compile).
  Should AMIR emit typed commands (today) with the text layer reserved for human input?
- **Scope**: should the command window expose full CRUD + lifecycle + catalog/series or start
  narrowed to modeling verbs only?
- **Undo/redo through the DSL**: should `undo` / `redo` in text be first-class verbs, or
  remain executor API calls (today's `make_undo_redo_command` sentinel)?
- **Where the parser lives**: new crate (`crates/omcom-text-parser`) vs inside `crates/omcom`
  behind a feature flag.
- **Versioning the grammar**: matching `OmniCommand.version` bump semantics to text-grammar
  versioning.
