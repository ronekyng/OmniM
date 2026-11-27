---
title: "Accessibility manual review kit (UI-A11Y-05)"
status: current
last_updated: 2026-11-27
applies_to: "web/"
evidence: "checklist + captured aria snapshots for the reviewer sign-off"
---
# Accessibility manual review kit (UI-A11Y-05)

> The machine side is covered by `e2e/axe.spec.ts` (0 serious violations on the WCAG 2.2 AA critical paths) and `e2e/a11y.spec.ts` (names/roles/keyboard). This kit covers the last mile that a real screen reader must judge: reading order, understandable wording, and whether the viewport text alternative is sufficient.

## 1. Environment
| Item | Requirement |
|---|---|
| Browser | Latest stable Chrome/Edge |
| Screen reader | Windows: NVDA 2026.x; macOS: VoiceOver (Safari/Chrome); Linux optionally Orca |
| Data | Project home → Open sample project; plus a project with three objects |
| Reference | `docs/specification/ui/A11Y.md` §6 (names and alternatives) |

## 2. Review script (tick each row)
| # | Surface | Action | Expected announcement | Result |
|---|---|---|---|---|
| 1 | Project home | Tab through cards and buttons | Card name, path, modified time; buttons carry a clear action | ☐ |
| 2 | Workspace entry | Open the sample project | Title-bar region name + project name; no duplicated "button button" | ☐ |
| 3 | Left dock | Arrow through the object tree | tree/treeitem roles, level, selected state; Shift+click announces the range | ☐ |
| 4 | Viewport | Focus `3D viewport` | Object count, camera mode, storey view (`#viewport-description`) | ☐ |
| 5 | Describe scene | Palette → Describe scene | The status region announces `Scene: N objects…` | ☐ |
| 6 | Properties | Select a tree row, then Tab | table role, column count, label/value pairs | ☐ |
| 7 | Creation form | Open the creation panel and fill it | Every numeric input reads label + unit; errors are perceivable | ☐ |
| 8 | Leave guard | Dirty the session, click back | Dialog title/project/last-saved; Esc=cancel, Enter=save | ☐ |
| 9 | Notifications | Save success/failure | `role=status` announces promptly without stealing focus | ☐ |
| 10 | Language switch | Switch to zh-CN and re-run 2–9 | All names/descriptions follow the locale with no leftovers | ☐ |

## 3. Recording and sign-off
1. Log failures with scenario number, screen-reader/browser version, actual text, expected text, and a screenshot or recording.
2. Re-run the affected rows after fixes; once the table is complete, record reviewer/date/build in the `UI-A11Y-05` evidence cell of [REQUIREMENTS_REGISTER](../product/REQUIREMENTS_REGISTER.md).
3. Machine snapshots: `web/e2e/a11ySnapshot.spec.ts` writes aria-tree digests of the key surfaces to `/tmp/opencode/a11y-snapshots.json` for pre-review comparison.

## 4. Pass criteria
- 10/10 scenarios pass, or the remainder are dated known issues that do not affect critical paths.
- `UI-A11Y-05` may only flip to ✅ after the reviewer signs; passing machine checks is not screen-reader acceptance.
