# OmniM RFC

RFC (Request for Comments) documents are used to discuss major design changes in OmniM.

## Index

| RFC | Title | Status |
| --- | ----- | ------ |
| [RFC-0001](RFC-0001-omcom-text-dsl.md) | Omcom Text DSL (nom-based text grammar) | **Draft** — the definitive design for the Omcom text language; the in-product console currently executes JSON commands only |
| [STORAGE_MODEL.md](STORAGE_MODEL.md) | Storage model design proposal | **Proposed / 未采纳** — moved here from `docs/` (repo-root docs) as a design proposal; not implemented (current persistence is JSON-file `.omnim`) |

## Scope

RFCs are required for changes to:

- Unified data model
- AMIR instruction schema
- Geometry representation
- Storage model
- Sync protocol
- Permission, security, and encryption model
- Import/export compatibility strategy
- Domain extension rules

## RFC Status

| Status | Description |
| --- | --- |
| Draft | Under discussion |
| Accepted | Accepted and ready for implementation |
| Rejected | Rejected |
| Superseded | Replaced by a newer RFC |
| Proposed / 未采纳 | Written as a proposal but not yet adopted by the implementation |

## File Naming

```text
0001-title.md
0002-title.md
```

## RFC Template

```markdown
# RFC 0001: Title

## Summary

## Motivation

## Detailed Design

## Compatibility

## Alternatives

## Open Questions
```
