---
# slerm-0zd9
title: Early terminal architecture refactor
status: completed
type: milestone
priority: high
tags:
- architecture
- early-refactor
created_at: 2026-06-09T04:28:59.691757Z
updated_at: 2026-06-09T04:52:12.735894Z
---

## Context

Source of truth: `.local/03-early-refactor.md` (Slerm Interface Implementation Plan). Future workers should read that plan for the overarching architectural direction before starting child work.

This milestone groups the early refactor that separates persisted terminal configuration from live runtime state while keeping the current single-crate, mostly single-`Entity<WorkspaceState>` GPUI shape.

## Work

Complete the smallest coherent sequence from the plan:

1. Clean up vocabulary from generic “items” to terminals.
2. Make terminal persistence explicit with `TerminalSpec`, `TerminalId`, `ProcessSpec`, and `TerminalExtensionSpec`.
3. Remove task runtime status from persisted specs.
4. Add a runtime-state home and derived status/attention model.
5. Add the first runtime service/backend seams without full libghostty or PTY implementation.

## Verification

- All child ishes are completed.
- `ish check` passes.
- `cargo fmt` and `cargo check` pass for the repository.
- The current GPUI prototype still builds and preserves existing visible behavior unless a child ish explicitly updates labels/badges.

## Completion Notes

- Completed milestone by verifying all child epics/tasks are completed: persisted terminal spec refactor (`slerm-oid4`) and runtime/status foundation (`slerm-x5su`).
- Persisted workspace state now uses terminal-oriented vocabulary and explicit specs: `TerminalSpec`, `TerminalId`, `ProcessSpec`, and `TerminalExtensionSpec`.
- Runtime-only process/session, agent, task, status, attention, and backend seam types live under `crates/slerm/src/runtime` and are intentionally not serialized.
- `SlermApp` keeps persisted `WorkspaceState` separate from `TerminalRuntimeService`; current UI builds with runtime-derived attention and no persisted task status.

## Final Verification

- `cargo fmt -- --check`
- `cargo check`
- `cargo test`
- `ish check`
