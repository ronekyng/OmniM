# AMIR Specification (AI Intermediate Representation)

## 1. Purpose

AMIR (**A**I **I**ntermediate **R**epresentation) is OmniM's AI translation layer responsible for converting **natural language** into **typed Omcom commands** (`OmniCommand`, proto/omcom.proto).

AMIR is purely a translator — it does not execute modeling operations nor define the instruction format. It serves as the bridge between LLMs and the Omcom command system:

```
Natural Language ──→ AMIR ──→ Omcom Commands (typed OmniCommand) ─→ Validator ─→ Executor ─→ Model
(Prompt)                                                                                  ↑
                      │                                            Model State ───────────┘
Model State ─────────┘ (Reverse Interpretation → natural language)
```

> Note: the original idea of AMIR emitting an "Omcom text DSL" is retained as a future
> design direction — see [RFC-0001](../RFC/RFC-0001-omcom-text-dsl.md). Today AMIR emits
> typed commands, and there is no text parser in the codebase.

### 1.1 What AMIR is NOT

| AMIR is | AMIR is NOT |
|---------|-------------|
| Natural language → Omcom command translator (typed OmniCommand) | The modeling instruction format itself (that's proto/omcom.proto) |
| AI intent recognition & parameter extraction | The instruction execution engine (that's Executor) |
| Natural language interpreter of model state | Undo/redo manager |
| Multi-agent orchestration coordinator | Validator (that's Omcom's responsibility) |

Omcom is a separate layer — see [OMCOM_SPEC.md](OMCOM_SPEC.md).

## 2. Core Design Principles

- **Transparency**: AMIR outputs standard typed Omcom commands, no AI-specific extensions
- **Traceability**: Each generated command carries source context (Actor = AI_AGENT)
- **Error Tolerance**: AI may make mistakes; AMIR reduces error probability through structured constraints
- **Domain Adaptation**: Different engineering domains (Building, Civil, Plant, Mechanical) have their own prompt templates
- **Explainability**: AI-generated instruction reasoning is presented in natural language to users

## 3. Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     AMIR Layer                             │
│                                                            │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ Prompt      │───▶│ Intent       │───▶│ Omcom Gen    │  │
│  │ Parser      │    │ Recognition  │    │ (typed cmds) │  │
│  └─────────────┘    └──────────────┘    └──────┬───────┘  │
│                  (JSON structured intent)      │          │
│  ┌─────────────┐    ┌──────────────┐          │          │
│  │ Model State │◀───│ Reverse      │◀─────────┘          │
│  │ Natural     │    │ Description  │                     │
│  │ Language    │    │ (cmds→NL)    │                     │
│  └─────────────┘    └──────────────┘                     │
│                                                            │
│  ┌──────────────────────────────────────────────────┐     │
│  │            Multi-Agent Orchestrator               │     │
│  │  (Building / Civil / Plant / Mechanical Agents)   │     │
│  └──────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────┘
        │
        ▼  Typed OmniCommand
Validator (crates/omcom)
        │
        ▼
Executor → Model
```

### 3.1 Prompt Parser

**Responsibilities**: Receive user natural language input, extract intent and parameters.

- **Intent Classification**: Identify the operation type the user wants to perform
- **Entity Extraction**: Extract engineering objects, properties, values, units from natural language
- **Context Awareness**: Incorporate current viewport selection, conversation history, etc.
- **Ambiguity Resolution**: Guide users to clarify ambiguous input

**Input**: Natural language text (e.g., "Create a 5m long, 3m high wall on Floor 1")

**Output**: Structured intent description (JSON: intent + key-value parameters)

### 3.2 Omcom Generator

**Responsibilities**: Convert parsed intent into typed OmniCommand envelopes (`crates/amir/src/generator.rs`).

- **Instruction Mapping**: Intent → corresponding Omcom action + operation messages
- **Multi-Step Planning**: Decompose complex operations into a sequence ofOmniCommand envelopes
- **Parameter Completion**: Fill in default parameters, compute implicit parameters
- **Reference Resolution**: Resolve natural language object references ("that wall", "Floor 1") to object IDs

**Input**: Structured intent description (JSON)

**Output**: Serialized `OmniCommand` envelopes (JSON on the wire), Actor = AI_AGENT

```
Example:

Input (JSON intent):
  {
    "intent": "create_object",
    "object_type": "wall",
    "name": "wall_1",
    ...
  }

Output (OmniCommand):
  { "action": CREATE_OBJECT,
    "actor": { "id": "amir", "type": AI_AGENT },
    "operations": [ { "create_object": { ... } } ] }
```

### 3.3 Reverse Parser

**Responsibilities**: Convert current model state or object selection into natural language descriptions.

- **Object Description**: Generate human-readable descriptions from object type, geometry, and properties
- **Change Summary**: Diff model baselines and generate natural language summaries of changes
- **Model Interpretation**: Explain complex engineering model structures to users in natural language
- **Omcom → Natural Language**: Translate existing Omcom instruction sequences back to natural language for user understanding and confirmation (this is the "model → text direction" of the original vision; a model→text reverse *parser* would be added by [RFC-0001](../RFC/RFC-0001-omcom-text-dsl.md) if approved)

**Use Cases**:
- AI answering "What's in the current model?"
- Post-import AI interpretation: "This IFC file contains 3 floors, 20 walls, 10 columns..."
- Operation preview: "The following changes will be applied to the model..."

### 3.4 Multi-Agent Orchestrator

**Responsibilities**: Coordinate multiple domain-specific agents to complete complex cross-domain tasks.

- **Domain Agents**:
  - Building Agent (familiar with walls, beams, slabs, doors, windows)
  - Civil Agent (familiar with roads, bridges, tunnels)
  - Plant Agent (familiar with tanks, pipes, valves)
  - Mechanical Agent (familiar with parts, assemblies, gears)
- **Orchestration Logic**: Route to the appropriate agent based on user input domain
- **Result Merging**: Integrate cross-domain operation results into unified Omcom command sequences
- **Conflict Resolution**: Resolve conflicts when different agent outputs disagree

### 3.5 Validation Feedback

**Responsibilities**: When Omcom validation fails, translate error information back to natural language.

- **Error Explanation**: Convert structured validator errors into user-friendly natural language
- **Correction Suggestions**: Provide modification suggestions based on error type and context
- **AI Retry**: For simple errors, AMIR can automatically correct and retry (the
  `/api/amir/chat` endpoint replays corrected commands against the same Omcom pipeline)
- **Human-in-the-Loop**: For complex or high-risk operations, request user confirmation

## 4. AI Integration

### 4.1 LLM Integration Methods

AMIR supports multiple LLM integration methods:

| Method | Use Case | Advantage | Disadvantage |
|--------|----------|-----------|-------------|
| Function Calling | OpenAI / Anthropic | Structured output, high reliability | Cloud-dependent |
| Local LLM | Llama 3 / Qwen | Data stays on-premises, offline capable | Relatively weaker capability |
| Prompt Template | All LLMs | Universal compatibility | Unstable output, needs validation |

### 4.2 Structured Output

AMIR uses JSON internally for structured intent representation, and the **final output is typed Omcom commands** (serialized OmniCommand). The pipeline:

1. LLM returns structured JSON (via function calling or JSON mode)
2. AMIR processes the JSON intent
3. AMIR generates typed OmniCommand envelopes as the final output

```
LLM Output (JSON structured intent):
  {
    "intent": "create_wall",
    ...
  }

AMIR Output (OmniCommand envelopes, JSON):
  { "action": CREATE_OBJECT, "actor": {"type": "AI_AGENT"}, ... }
```

### 4.3 RAG Enhancement

For engineering standards and specification queries, AMIR supports Retrieval-Augmented Generation:

- Embed engineering specifications and standards documents
- When users query about standards, retrieve relevant clauses as prompt context
- Ensure AI-generated modeling instructions comply with engineering standards

## 5. Reverse Parsing (Model → Commands)

The reverse direction (model state or an object selection → natural language descriptions,
and conceptually a model → Omcom command log) enables:
- AI interpretation of existing designs and project records
- Natural language inspection of imported models and lifecycle data
- Change detection and diffing
- Instruction-level versioning (a full model→OmniCommand-script reverse parser is part of the original vision captured in [RFC-0001](../RFC/RFC-0001-omcom-text-dsl.md); "model → text" renderability there)

### Reverse Parsing Strategy
1. **Geometry Simplification**: Convert complex geometry to parametric representations where possible
2. **Object Classification**: Identify and classify objects to appropriate ObjectType
3. **Property Extraction**: Extract properties and property sets
4. **Relation Detection**: Discover containment and connection relationships
5. **Instruction Sequencing**: Order operations logically (create → modify → connect)

## 6. Versioning and Compatibility

As a pure translation layer, AMIR's version relates to its LLM integration interface:
- **AMIR v1**: Function Calling-based structured output, generating Typed OmniCommand commands
- Version compatibility is guaranteed by the corresponding Omcom schema (`omcom.proto`)

## 7. Open Questions

- Should AMIR run on client or server, or hybrid?
- How to evaluate the quality and correctness of AI-generated Omcom commands?
- Should AI-generated instructions have priority ordering or permission control?
- How to handle context maintenance in multi-turn conversations?