# CLAUDE.md - autumn-datatable-plugin

The data table v2 widget pack for autumn-web (autumn#3414). It follows the
widget contract in autumn `docs/guide/widget-packs.md` and ADR 0030.

## Rules

- **No JavaScript needed.** Each feature works as links and forms. The
  script only syncs the select-all state, the count and the confirm dialog.
- **The contract.** Classes are `autumn-datatable-*` or core classes. CSS
  uses only public tokens. No inline script or style, no `on*`, no
  `style` attribute, no arrow-key code.
- **The state round-trips.** A change to `TableState` keeps
  `decode(encode(s)) == s`. The proptests in `tests/state.rs` check it.
- **Safe default.** `TableState::checked` drops what the table does not
  declare. Do not remove a check. Add an explicit opt-in for an unsafe path.
- **Each id starts with the table id.**
- **Comments stand alone.** No review rounds or findings labels.
  Write in ASD-STE100: short sentences, active voice.

## Commands

- Test: `cargo test`
- Lint: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`
- Browser: `AUTUMN_CHROMIUM=/path/to/chrome cargo test --features system-tests --test browser -- --include-ignored`
- Coverage: `cargo llvm-cov --fail-under-lines 85`

## Layout

| Path | What |
|---|---|
| `src/state.rs` | `TableState`, `Density`: decode, encode, links, `checked`. |
| `src/selection.rs` | `SelectAll`, `BulkSelection`. |
| `src/table.rs` | `Column`, `Filter`, `BulkAction`, `DataTable`. |
| `src/stories.rs` | One story for each feature. |
| `src/datatable.css`, `src/datatable.js` | The pack assets. |
| `tests/` | `state`, `selection`, `render`, `conformance`, `browser`. |
