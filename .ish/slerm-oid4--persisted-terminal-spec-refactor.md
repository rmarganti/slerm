---
# slerm-oid4
title: Persisted terminal spec refactor
status: completed
type: epic
priority: high
tags:
- architecture
- persistence
- early-refactor
created_at: 2026-06-09T04:29:13.708772Z
updated_at: 2026-06-09T04:50:18.196264Z
parent: slerm-0zd9
---

## Context

Source of truth: `.local/03-early-refactor.md`, especially “Apply One-to-One”, “Proposed Target Types”, and migration phases 1–3.

The current code mixes persisted terminal configuration, UI selection, and task runtime status in `TerminalInstance`, `TerminalKind`, and `Project.items`. This epic makes the persisted workspace model speak Slerm's domain language: projects own terminal specs.

## Dependencies

- Parent milestone: `slerm-0zd9`.
- Child tasks must be executed in dependency order; do not rely on this parent relationship as ordering.

## Work

Deliver a coherent persisted model refactor:

- Vocabulary cleanup from item/instance/kind to terminal/spec/extension.
- Structured process launch config via `ProcessSpec`.
- Persisted extension specs for plain, agent, and task terminals.
- No live task status stored in serialized workspace specs.

## Verification

- All child tasks complete.
- Serialized model types still derive Serde traits where the current workspace persistence requires them.
- `cargo fmt` and `cargo check` pass.


## Completion Notes

- Reviewed child tasks `slerm-71xc`, `slerm-hrxb`, and `slerm-i8ug`; all are completed and together deliver the persisted terminal spec refactor.
- Current persisted model uses `TerminalSpec`/`TerminalId`, `Project.terminals`/`active_terminal`, structured `ProcessSpec`, and `TerminalExtensionSpec::{Plain, Agent, Task}` with persisted agent/task configuration only.
- Live task status has been removed from serialized terminal specs; any `TaskStatus` references now live under runtime-only code.
- Verified no remaining core-domain item/instance vocabulary, optional string terminal command model, or persisted `TerminalKind`/task status usage beyond runtime-only `TaskStatus`.

## Final Verification

- `rg "TerminalInstance|TerminalInstanceId|Project\\.items|active_item|add_item|items_in_sidebar_order|ActiveItem" crates/slerm/src` (no matches)
- `rg "command: Option|and_then\\(\\|.*command" crates/slerm/src` (no matches)
- `rg "Option<String>" crates/slerm/src/terminal crates/slerm/src/workspace crates/slerm/src/ui/terminal_pane.rs` (no matches)
- `rg "TerminalKind|Task \\{ status|TaskStatus::" crates/slerm/src` (matches only runtime-only `TaskStatus` in `crates/slerm/src/runtime/mod.rs`)
- `cargo fmt -- --check`
- `cargo check`
- `cargo test`
- `ish check`
