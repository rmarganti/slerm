Slerm is a manager of terminals and agents.

It is a Rust application built upon GPUI (https://github.com/zed-industries/zed/tree/main/crates/gpui)
for UI and Libghostty for terminal emulation.

## Key Tenets

- Everything is a terminal. Agents and tasks are terminals with semantic extensions, not separate top-level concepts.
- Projects are the top-level organizational unit. Only one project is shown at a time, but inactive projects should surface attention when terminals need it.
- Persisted state describes intent/configuration: projects, terminal specs, process specs, and extension specs.
- Runtime state describes what is happening now: sessions, process status, agent status, task status, exits, and attention. Do not persist live runtime state as workspace config.
- Keep agent and task semantics separate. Agents report states like idle, working, awaiting review, or errored; tasks report lifecycle outcomes like pending, running, succeeded, failed, restarting, or stopped.
- Derive UI attention from runtime state instead of storing it as authoritative state.
- Keyboard navigation first, friendly to Vim users.
- Minimal UI; let the terminal be the star.
- Add GPUI entities, terminal-surface abstractions, and crate boundaries only when runtime/rendering pressure justifies them.
- Keep PTY/process control separate from terminal rendering and libghostty surface state.

## Verifying

Must be run before considering a task complete:

- `cargo fmt --all -- --check`
- `cargo test --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-features`
