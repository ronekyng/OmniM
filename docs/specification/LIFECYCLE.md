# OmniM Project Lifecycle Data Model

## 1. Overview

OmniM supports the full engineering project lifecycle as a first-class part of the data model. Lifecycle data is not external project-management metadata; it is modeled alongside geometry, objects, and relationships so that every engineering object carries its full history from planning to decommissioning.

The lifecycle is organized into nine phases. Different project types (building, civil, plant, mechanical) may skip certain phases — for example, a building project typically skips `PRODUCTION`, while a plant project enters it after handover.

```
Planning → Design → Procurement → Construction → Commissioning & Acceptance
    → Asset → Operation & Maintenance → Production → Decommissioning
```

### Phase Summary

| # | Phase | Chinese | Purpose |
|---|-------|---------|---------|
| 1 | Planning | 规划 | Project initiation, feasibility, site selection, requirements |
| 2 | Design | 设计 | Architectural, structural, MEP, civil, plant, mechanical design |
| 3 | Procurement | 采购 | Bidding, contracts, material and equipment purchasing, supply chain |
| 4 | Construction | 施工 | Building, installation, schedule, quality, safety, as-built |
| 5 | Commissioning & Acceptance | 调试验收 | Testing, inspection, trial operation, acceptance, handover |
| 6 | Asset | 资产 | Asset registration, warranty setup, handover to owner |
| 7 | Operation & Maintenance | 运维 | Operation, maintenance, facility management, energy |
| 8 | Production | 生产 | Industrial production operation (plant/manufacturing only) |
| 9 | Decommissioning | 退役拆除 | Retirement, demolition, disposal, site restoration |

---

## 2. Phase Details

### 2.1 Planning (规划)

**Purpose**: Project initiation and feasibility determination.

**Key Data**:
- Project brief and requirements
- Site selection and land use
- Topography and existing conditions survey
- Zoning, environmental, and regulatory constraints
- Feasibility studies (technical, economic, environmental)
- Budget envelope and schedule framework

**Linked Objects**: Site, terrain models, existing utilities, alignment corridors.

---

### 2.2 Design (设计)

**Purpose**: Translate requirements into buildable models, drawings, and specifications.

**Sub-disciplines**:
- **Architectural**: Envelope, spatial layout, openings, finishes
- **Structural**: Primary structure, foundations, reinforcement, connections
- **MEP**: HVAC, plumbing, electrical, fire protection distribution
- **Civil**: Roads, earthwork, drainage, site utilities
- **Plant**: Equipment layout, piping, instrumentation, structural steel
- **Mechanical**: Parts, assemblies, GD&T, PMI, product structure

**Key Data**: 3D models, 2D drawings, calculations, specifications, design changes, clash reports.

**Linked Objects**: All designed elements (walls, beams, pipes, equipment, parts, etc.).

---

### 2.3 Procurement (采购)

**Purpose**: Source materials, equipment, and services; manage contracts and deliveries.

**Key Data**:
- Bill of materials / quantity take-off
- Bid packages and bid evaluation
- Contracts and purchase orders
- Supplier and vendor data
- Delivery schedules and tracking
- Cost estimates and actuals

**Linked Objects**: Procured elements (valves, pumps, steel members, materials, parts).

---

### 2.4 Construction (施工)

**Purpose**: Build and install the designed works.

**Key Data**:
- Construction schedule (WBS, activities, dependencies)
- Progress updates (percent complete by object)
- Quality inspections and nonconformance reports
- Safety incidents and hazards
- RFIs, submittals, change orders
- As-built conditions and deviations

**Model Integration**:
- Each object linked to its construction activity
- 4D simulation (time + 3D)
- Color-coded progress visualization
- Defects and issues located on model geometry

---

### 2.5 Commissioning & Acceptance (调试验收)

**Purpose**: Verify that installed works meet design intent and regulatory requirements; hand over to owner.

**Key Data**:
- Equipment and system commissioning records
- Test reports (pressure, electrical, functional, performance)
- Trial operation logs
- Inspection and acceptance certificates
- Punch list and defect resolution
- As-built model validation against installed conditions
- Handover packages (O&M manuals, warranties, certificates)

**Linked Objects**: Equipment, systems, completed elements, as-built objects.

---

### 2.6 Asset (资产)

**Purpose**: Register handed-over objects as managed assets with full lifecycle attributes.

**Key Data**:
- Asset registry (asset ID, barcode/QR, classification)
- Manufacturer, model, serial number
- Installation date and commissioning date
- Warranty terms and expiration
- Expected service life and replacement schedule
- Depreciation and financial data
- Spare parts inventory
- Asset hierarchy (system → component → sub-component)

**Linked Objects**: All handed-over physical elements that become managed assets.

---

### 2.7 Operation & Maintenance (运维)

**Purpose**: Operate the facility and maintain assets in working condition.

**Key Data**:
- **Operation**: Operating parameters, setpoints, logs, shifts
- **Maintenance**: Work orders (preventive, corrective, predictive), maintenance logs, downtime records
- **Facility Management**: Space utilization, occupancy, leases
- **Energy**: Consumption, billing, efficiency tracking
- **Issues**: Faults, alarms, service requests

**Model Integration**:
- Locate assets in 3D model for maintenance
- Access manuals, specs, and history from model
- Real-time sensor data overlay (feeds into Digital Twin capability)

---

### 2.8 Production (生产)

**Purpose**: For industrial/plant projects, operate the production process to manufacture products. Building and civil projects typically skip this phase.

**Key Data**:
- Production plans and schedules
- Production batches, lots, and throughput records
- Raw material and consumable consumption
- Product quality and yield data
- Equipment downtime and OEE (overall equipment effectiveness)
- Production logs and shift reports

**Linked Objects**: Production lines, reactors, vessels, tanks, pumps, manufacturing equipment.

**Note**: Production and Operation & Maintenance often run in parallel for plant projects — equipment must be maintained while production continues. A single asset may have both maintenance records (O&M phase) and production records (Production phase).

---

### 2.9 Decommissioning (退役拆除)

**Purpose**: Safely retire, dismantle, demolish, and dispose of assets at end of life; restore the site.

**Key Data**:
- Decommissioning plans and procedures
- Hazardous material surveys (asbestos, chemicals, radiation)
- Dismantling sequences and schedules
- Demolition records and progress
- Waste disposal and recycling logs
- Asset disposal / salvage records
- Site restoration and remediation
- Final closeout documentation

**Linked Objects**: Retired assets, structures scheduled for removal, contaminated areas.

---

## 3. Cross-Phase Data Continuity

### 3.1 Object Identity Across Phases
Each engineering object maintains its identity throughout the entire lifecycle. Its state evolves through phase transitions:

```
Designed → Procured → Installed → Commissioned → Registered as Asset
    → Maintained → (Producing) → Decommissioned
```

### 3.2 Phase Transitions
| Transition | Key Handoff Data |
|------------|------------------|
| Planning → Design | Requirements, site constraints, feasibility |
| Design → Procurement | Quantity take-off, specifications, material schedules |
| Procurement → Construction | Delivery status, supplier data, purchase orders |
| Construction → Commissioning | As-built conditions, installation records, test readiness |
| Commissioning → Asset | Acceptance certificates, warranties, O&M manuals |
| Asset → O&M | Asset registry, maintenance schedule, operating procedures |
| Asset/O&M → Production | Production capacity, process parameters, equipment readiness |
| O&M/Production → Decommissioning | Service history, residual life, hazardous material data |

---

## 4. Lifecycle Metadata on Objects

Every engineering object can carry lifecycle metadata:

```json
{
  "lifecycle": {
    "current_phase": "OPERATION_MAINTENANCE",
    "planning": { "feasibility_date": "2024-01-15" },
    "design": { "approved_date": "2024-03-20", "approved_by": "user:jane.smith" },
    "procurement": { "po_number": "PO-2024-0412", "supplier": "Acme Corp" },
    "construction": { "start": "2024-04-01", "complete": "2024-08-15" },
    "commissioning": { "accepted_date": "2024-08-30", "accepted_by": "user:inspector.lee" },
    "asset": { "asset_id": "A-000123", "warranty_end": "2026-08-30", "expected_life_years": 25 },
    "operation_maintenance": { "last_maintenance": "2025-08-01", "next_maintenance": "2026-02-01" }
  }
}
```

---

## 5. Project-Type Phase Paths

Different project types follow different phase paths:

| Project Type | Typical Path |
|--------------|--------------|
| **Building** | Planning → Design → Procurement → Construction → Commissioning → Asset → O&M → Decommissioning |
| **Civil (roads, bridges)** | Planning → Design → Procurement → Construction → Commissioning → Asset → O&M → Decommissioning |
| **Plant (process, power)** | Planning → Design → Procurement → Construction → Commissioning → Asset → O&M + Production → Decommissioning |
| **Mechanical (product)** | Planning → Design → Procurement → Construction (manufacturing) → Commissioning → Asset → O&M → Production → Decommissioning |

---

## 6. Data Retention Policy

| Data Type | Retention Period | Notes |
|-----------|------------------|-------|
| Planning & feasibility | Life of facility + 10 years | Legal archive |
| Design models & drawings | Life of facility + 10 years | Legal archive |
| Procurement contracts | Life of facility + 10 years | Legal archive |
| Construction records | Life of facility + 10 years | Legal archive |
| Commissioning records | Life of facility + 10 years | Legal archive |
| Asset registry | Until decommissioning + 10 years | Financial requirements |
| Maintenance logs | Life of equipment | Replaced with equipment |
| Production records | Per industry regulation (often 5-10 years) | Regulatory compliance |
| Sensor / IoT data | 3 years raw, forever aggregated | Compliance varies |
| Decommissioning records | Permanent | Environmental liability |

---

## 7. Open Questions

1. How to model parallel phases (e.g., O&M and Production running concurrently for a plant)?
2. Should phase transitions be enforced as a state machine, or remain free-form?
3. How to handle phase-specific geometry (design model vs as-built model vs as-maintained model)?
4. What is the right granularity for production records (batch vs continuous process)?
5. How to link decommissioning plans back to original design assumptions for circular economy analysis?
