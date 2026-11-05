# Plan: data table v2 (autumn#3414)

Part of autumn#3393. The pack ships through the widget plugin seam
(autumn ADR 0030).

## Goal

A data table that the server drives with query parameters. It works with
JavaScript off. It adds selection, a filter bar, column visibility,
density, sticky headers, and empty and loading states. Core `data_table`
stays the light, sort-only table.

## Brainstorm

- `TableState`: the table state in the query string. `q`, `sort`, `dir`,
  `page`, `size`, `filter[<key>]` (many values), `cols`, `density`.
- `TableState` decodes with `Query<T>` and with `TableState::from_query`. It
  encodes back to the same query string.
- Links change one part of the state. A sort or a filter change goes back
  to page 1.
- Selection column: one native checkbox for each row, and a select-all
  checkbox. It has three states: off, on and `aria-checked="mixed"`.
- Bulk actions: a POST form around the table. It sends `action` and
  `ids`. Without JavaScript, select-all sends `all` and the page ids.
- `BulkSelection` decodes the POST body and gives the ids to act on.
- Filter bar: a GET form. It has a text, a select or a checkbox group for
  each filter. Active filters show as chips with a remove link.
- Column visibility: a `<details>` with a GET form of checkboxes.
- Density: two links, comfortable and compact.
- Sticky header: CSS `position: sticky` in a scroll region.
- Empty state: the core `autumn-empty-state`. With an active filter, it
  shows "No rows match" and a link that clears the filters.
- Loading state: skeleton rows with `aria-busy="true"`. The root has
  `data-autumn-busy`, so an htmx request sets `aria-busy`.

## Reverse brainstorm (how can it fail?)

| Fault | Prevention |
|---|---|
| A sort link loses the filters. | Each link starts from the full state. Test each link. |
| A filter change keeps page 7 of 2. | A sort, filter, search or size change resets the page. |
| A filter value injects markup or a URL part. | maud escapes values. The encoder percent-encodes. Test with `"><script>`. |
| The app puts an unknown `sort` into SQL. | `TableState::checked` drops unknown sort, filter and column keys. |
| `size=1000000` loads all rows. | `checked` clamps `size` to a maximum. |
| The user hides all columns. | The table shows the default columns. |
| A form with all boxes clear sends no `cols`. | The form sends `cols=` as a marker. |
| The select-all state is wrong after an htmx swap. | The script sets up on `autumn:init` and reads the DOM. |
| `aria-checked` and the native state do not agree. | The script sets `indeterminate` to agree with `aria-checked`. Axe checks it. |
| Two tables on one page have the same ids. | Each id starts with the table id. |
| A class fails the pack class check. | Each class is `autumn-datatable-*` or a core class. |
| Nested forms break submit. | GET forms are outside the POST form. |
| A bulk submit with no rows does a no-op. | The script stops it and shows a message. The server also checks. |

## Six thinking hats

- **White (facts).** Core `data_table` sorts only. The admin list page
  uses `q`, `sort`, `dir`, `ids` and `action`. `autumn-web` 0.10 has the
  widget seam, multi-value query decoding and the conformance checks.
  The admin plugin is on crates.io. It cannot use a git dependency.
- **Red (feelings).** Users want a table that works as links and forms.
  A heavy script is a reason not to use it.
- **Black (risks).** The admin migration needs this crate on crates.io
  first. Mixed state is easy to get wrong for screen readers. Browser
  tests can be slow or flaky. Long URLs.
- **Yellow (benefits).** One table for the admin and for apps. The admin
  can delete its select-all script. Typed state. htmx works with
  `hx-boost` on the root.
- **Green (ideas).** `checked` as the safe default. Filter chips. The
  `cols=` marker. Range selection is a later issue.
- **Blue (process).** Red, then green, then refactor, for each part:
  state, selection, render, pack, script and browser tests, then docs.
  The admin migration is a follow-up in the autumn repository.

## Steps

1. RED: tests for the state round-trip, links, `checked`, select-all
   state, `BulkSelection`, render and static conformance.
2. GREEN: the code that makes the tests pass.
3. Browser tests: the selection keyboard model, and axe through
   `run_widget_browser_conformance`.
4. REFACTOR: clean up with all tests green. Docs and CI.
5. Review from several angles. Fix the findings.

## Decision after the plan

The admin plugin keeps its own table. This crate is in its own
repository. If the admin used it, each `autumn-web` release must first
release this crate. The admin migration is not part of this work.
