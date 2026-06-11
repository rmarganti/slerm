# Terminal Performance Instrumentation

Slerm keeps terminal performance logging disabled by default. Set `SLERM_TERMINAL_PERF=1` before launching Slerm to emit one `stderr` line per terminal prepaint:

```sh
SLERM_TERMINAL_PERF=1 cargo run --all-features
```

Each line includes:

- total terminal prepaint duration
- PTY drain duration, bytes read, and changed terminal count
- libghostty snapshot/update duration
- rows and cells considered for rendering
- render items produced by the current row/run snapshot path
- GPUI `shape_line` call count

Manual smoke scenarios for baseline/after-change comparisons:

1. Open a normal shell and type/edit commands at the prompt.
2. Run a dense TUI such as Neovim.
3. Run `ish tui` or another full-screen TUI.
4. Run bounded high output, for example `yes | head -100000`.
5. Keep one noisy terminal hidden while interacting with the active terminal.

With instrumentation disabled, Slerm should not emit these performance lines.
