# Move the admin list page to this table

The admin plugin (`autumn-admin-plugin`) has its own list table. This
guide maps it to `DataTable`. Do the move in the autumn repository, after
this crate is on crates.io. A crate on crates.io cannot have a git
dependency.

## Map

| Admin now (`templates.rs`, `admin.js`) | `DataTable` |
|---|---|
| `<table>` with `th class="checkbox-cell"` | `.selectable(\|r\| id)` |
| `#select-all` and the `.row-check` script | The select-all box and `datatable.js`. Delete the script. |
| `name="ids"`, `name="action"` | The same names. `BulkSelection::from_form`. |
| `option data-confirm="1"` and `#admin-bulk-confirm` | `BulkAction::confirm(text)` and the pack dialog. |
| "Select at least one row first." `alert` | The status text. No `alert`. |
| `?sort=<f>&dir=<d>` links with `▲`/`▼` | `Column::sortable()`. `aria-sort` on the header. |
| `q` search with `hx-get` | `.search(label)`. Put `hx-boost` on the root with `.options(...)`. |
| `filter.<key>=<value>` | `filter[<key>]=<value>`. The format changes. |
| "No records found." | `.empty("No records found.", "", None)`. |
| `View`, `Edit` links | A `Column` with header "Actions". |
| CSV links | Keep them in the card header. Use `state.to_query()` for the export URL. |

## Steps

1. Add `autumn-datatable-plugin` to `autumn-admin-plugin/Cargo.toml`.
2. Install `DatatablePlugin` from the admin plugin `build`.
3. In `model_list_page`, build `Column`s from the list fields and render
   `DataTable`.
4. Decode the list query with `TableState`. Change the filter decode from
   `filter.<key>` to `filter[<key>]`.
5. Decode the bulk body with `BulkSelection`.
6. Delete the select-all and bulk-confirm code in `admin.js`.
7. Change the admin tests that read the old markup (`select-all`,
   `row-check`, `filter.`). Run `cargo test -p autumn-admin-plugin`.
