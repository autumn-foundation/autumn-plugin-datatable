# ADR 0001: The query string holds the table state

- Status: Accepted
- Date: 2026-11-05
- Issue: autumn#3414

## Context

The table needs sort, filters, column visibility, density and selection.
It must work with JavaScript off. The admin plugin and apps must read the
same state on the server.

## Options

1. Keep the state in the browser (local storage, script).
2. Keep the state in the session.
3. Keep the state in the query string. Selection goes in the POST body.

## Decision

Option 3.

- `TableState` is the state. It decodes with `Query<TableState>` and
  encodes to the same query string. Filters use `filter[<key>]` with many
  values, through the autumn multi-value decoder.
- Each link and each GET form writes the full state. A sort, filter,
  search or page-size change resets `page`.
- Selection is not in the URL. The bulk form posts `action`, `ids`, and
  for select-all without JavaScript, `all` and `visible`.
- `TableState::checked` keeps only declared keys and clamps `size`.
  This is the safe default. The raw state stays available.
- Column visibility sends `cols=` as a marker. Then "all optional columns off"
  is different from "no choice".

- Keys are `[a-z0-9_]+`. Ids join their parts with `-`, so two ids cannot
  be equal. The decoder caps filters, values and columns.
- The bulk `<form>` holds no cell. Its controls join it with the `form`
  attribute. Then a form in a cell does not nest in it.
- With JavaScript off, select-all does not change the row boxes. The form
  sends `all_shown` when the server rendered select-all on. Then `all`
  that the user turns on acts on the page, and a row that the user clears
  stays out.
- The script writes `aria-checked="mixed"` and `indeterminate` together.
  The server writes neither. On a native box, `aria-checked` must agree
  with the native state.
- The confirm dialog sends `confirmed=1`. With JavaScript off, the server
  asks: `BulkSelection::needs_confirm`.
- The script listens for `submit` in the capture phase. Then `hx-boost`
  cannot post before the dialog.

## Consequences

- A URL holds the full view. The user can share it and go back to it.
- The script is small. It only syncs the select-all state, the count and
  the confirm dialog.
- A long filter set makes a long URL.
- The admin URL format changes from `filter.<key>` to `filter[<key>]`
  when the admin moves to this table.
- `list_query` has only the single-value filters. The app applies a
  multi-value filter with `eq_any`.
