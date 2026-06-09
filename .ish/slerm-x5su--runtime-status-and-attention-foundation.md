---
# slerm-x5su
title: Runtime status and attention foundation
status: completed
type: epic
priority: high
tags:
- architecture
- runtime
- early-refactor
created_at: 2026-06-09T04:29:13.714178Z
updated_at: 2026-06-09T04:51:15.436427Z
parent: slerm-0zd9
blocked_by:
- slerm-i8ug
---

## Context

Source of truth: `.local/03-early-refactor.md`, especially “Runtime State”, “Derived Status and Attention”, migration phases 4–6, and “Adapt Instead of Applying One-to-One”.

After persisted specs stop carrying runtime status, Slerm needs an explicit home for live process/session, agent, and task state plus a derived status layer for UI attention.

## Dependencies

- Parent milestone: `slerm-0zd9`.
- Blocked by completion of the persisted terminal spec refactor; runtime state should be based on `TerminalId`, `TerminalSpec`, `ProcessSpec`, and `TerminalExtensionSpec` names.

## Work

Deliver the near-term runtime foundation without implementing full PTY/libghostty behavior:

- `runtime/` module with terminal session/runtime structs and `TerminalRuntimeService`.
- Separate `AgentRuntime`/`AgentStatus` and `TaskRuntime`/`TaskStatus`.
- Derived `TerminalStatus`/attention model usable by UI.
- Runtime entity/backend seams introduced only where useful now.

## Verification

- Runtime types are not serialized as workspace config.
- `cargo fmt` and `cargo check` pass.
- UI remains functional with mocked/default runtime state.


## Completion Notes

- Reviewed child runtime tasks `slerm-q6fh`, `slerm-ptbm`, `slerm-ye7d`, and `slerm-nk40`; all are completed and together deliver the runtime/status foundation.
- Runtime-only state now lives in `crates/slerm/src/runtime/mod.rs` with `TerminalRuntimeService`, session/run state, separate agent/task runtime state, derived terminal/project attention, and a minimal PTY backend seam.
- `SlermApp` owns separate persisted workspace and runtime service GPUI entities; add/close terminal flows keep runtime state in sync without serializing it.
- Runtime structs intentionally have no Serde derives, and task status remains runtime-only.

## Final Verification

- `cargo fmt -- --check`
- `cargo check`
- `cargo test`
- `rg "Serialize|Deserialize" crates/slerm/src/runtime` (no matches)
- `rg "TaskStatus::" crates/slerm/src` (matches only runtime model/derivation/tests)
- `ish check`
