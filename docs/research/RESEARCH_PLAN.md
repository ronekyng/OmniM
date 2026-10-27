# Component Research Plan

## Current State

| Domain | Total Components | Proto | Unresearched ❌ |
|--------|----------------|-------|----------------|
| Building | 35 | 22 | 14 |
| Civil | 38 | 10 | 23 |
| Plant | 37 | 11 | 15 |
| Mechanical | 37 | 8 | **37** |
| **Total** | **147** | **51** | **89** |

89 unresearched components. 37 of them are in the Mechanical domain which has **zero** platform research.

---

## Phase 1: Mechanical Domain (High Priority)

**Target platform**: SolidWorks 2027 API (most widely used mechanical CAD)
**Goal**: Validate all 37 mechanical components, determine GenRule mappings, identify missing components

### Research Topics

| Topic | Components to Cover | Research Output |
|-------|-------------------|-----------------|
| Part/Assembly/SubAssembly hierarchy | Part, Assembly, SubAssembly, Configuration, Variant | ObjectType definitions, GenRule mapping |
| Machine elements (built-in Toolbox) | Gear, Shaft, Bearing, Coupling, Spring, Seal, Bushing, Fastener (Bolt, Nut, Washer, Screw, Rivet, Pin) | Type hierarchy, standard properties |
| Feature-based modeling | Hole, Pocket, Slot, Chamfer, Fillet, Thread | Feature type mapping, PMI handling |
| Mates & Constraints | Joint, Constraint, Axis, KinematicPair, Actuator | Relation mapping |
| Electrical/Connector | Motor, Encoder, Sensor, Housing, Connector | Connector type mapping |
| Design Tables & Configurations | Configuration, Variant, PMI | Series mapping |

### Deliverable
- `docs/research/solidworks_components.md` — SolidWorks built-in component analysis
- Update `docs/research/comparative_analysis.md` with Mechanical column
- Update `COMPONENT_INVENTORY.md` with research results

### Platform
- SolidWorks 2027 API Docs (swdocument, swpart, swassembly, swdrawing)
- SolidWorks Toolbox standards (ANSI, ISO, DIN, JIS)
- SolidWorks Mates & Constraints reference
- SolidWorks FeatureManager tree structure

---

## Phase 2: Civil Domain — Bridge/Tunnel Detail (Medium Priority)

**Target platform**: Bentley OpenRoads / OpenBridge
**Goal**: Validate bridge, tunnel, and hydraulic components

| Topic | Components to Cover | Research Output |
|-------|-------------------|-----------------|
| Bridge superstructure | Girder, Deck, Bearing, ExpansionJoint | ObjectType definitions, GenRule mapping |
| Bridge substructure | Pier, Abutment (already in proto, needs validation) | GenRule confirmation |
| Tunnel structure | Lining, Portal, Shaft, Segment, CrossPassage | ObjectType definitions, GenRule mapping |
| Hydraulic structures | Channel, Drainage, RetentionBasin, Dam | ObjectType definitions |
| Earthwork | Terrain, CutVolume, FillVolume, Slope | ObjectType definitions |

### Deliverable
- `docs/research/bentley_openroads_components.md` — Bentley infrastructure component analysis
- Update `COMPONENT_INVENTORY.md`

---

## Phase 3: Building Domain — Structural Detail (Medium Priority)

**Target platform**: Tekla Structures
**Goal**: Validate structural steel components and rebar

| Topic | Components to Cover | Research Output |
|-------|-------------------|-----------------|
| Steel connections | Brace, Truss, Beam/Column connections | GenRule confirmation |
| Foundation | Footing, Pile | ObjectType definitions |
| Reinforcement | Rebar, Stirrup, Mesh | ObjectType definitions |
| Secondary steel | Girt, Purlin, KickerBrace (from Revit StructuralType) | ObjectType additions |

### Deliverable
- `docs/research/tekla_components.md` — Tekla structural component analysis
- Update `COMPONENT_INVENTORY.md`

---

## Phase 4: Plant Domain — Instrumentation & Electrical (Low Priority)

**Target platform**: AVEVA Instrumentation / E3D Electrical
**Goal**: Validate remaining plant components

| Topic | Components to Cover | Research Output |
|-------|-------------------|-----------------|
| Instrumentation | Instrument, Sensor, Transmitter, ControlValve, JunctionBox | ObjectType definitions |
| Electrical | Conduit, Panel, MotorControlCenter | ObjectType definitions |
| Filtration | Filter | ObjectType definition |
| Supports | Support (pipe support, equipment support) | ObjectType definition |

### Deliverable
- `docs/research/e3d_instrumentation_components.md` — E3D instrumentation analysis
- Update `COMPONENT_INVENTORY.md`

---

## Phase 5: Proto Update (After All Research)

Synthesize all research findings into:
1. Update `proto/omnim.proto` ObjectType enum with new types
2. Update `crates/model/src/gen_rule/mod.rs` `primary_gen_rule()` with new mappings
3. Update `docs/specification/DOMAINS.md` to reflect validated vs aspirational status
4. Remove or mark unresearched components

---

## Timeline Estimate

| Phase | Duration | Output |
|-------|----------|--------|
| P1: SolidWorks research | 3 days | `solidworks_components.md` + comparative update |
| P2: Bentley OpenRoads | 2 days | `bentley_openroads_components.md` |
| P3: Tekla Structures | 2 days | `tekla_components.md` |
| P4: E3D Instrumentation | 1 day | `e3d_instrumentation_components.md` |
| P5: Proto update | 1 day | proto + gen_rule + DOMAINS.md |
| **Total** | **9 days** | |