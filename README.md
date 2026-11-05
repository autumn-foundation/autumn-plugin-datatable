# autumn-datatable-plugin

Data table v2 for [autumn-web](https://github.com/autumn-foundation/autumn).
It is a first-party widget pack (autumn#3414). Core `data_table` stays the
light table that sorts only.

The server drives the table with query parameters. It works with
JavaScript off.

| Feature | How it works |
|---|---|
| Sort | Header links with `aria-sort`. `sort`, `dir`. |
| Selection | A checkbox for each row and a select-all box. The select-all box has three states: off, on and mixed. The script sets `indeterminate` and `aria-checked="mixed"` together. |
| Bulk actions | A POST form: `action`, `ids`. Without JavaScript, select-all sends `all` and the page ids. A confirm action opens a `<dialog>` and sends `confirmed=1`. |
| Filter bar | A GET search form. Text, select and checkbox filters as `filter[<key>]`, with many values. Each active filter is a chip with a remove link. |
| Column visibility | A `<details>` with a GET form: `cols`. |
| Density | Links: `density=compact`. |
| Sticky header | `.sticky_header()`. The header row stays on screen in the scroll region. |
| Empty state | The core empty state. With an active filter: "No rows match" and a link that clears the filters. |
| Loading state | Skeleton rows and `aria-busy="true"`. The root has `data-autumn-busy`. With `hx-indicator="this"` on the root, an htmx request from the table sets `aria-busy`. |

## Install

```toml
[dependencies]
autumn-datatable-plugin = { git = "https://github.com/autumn-foundation/autumn-plugin-datatable" }
```

```rust
use autumn_datatable_plugin::DatatablePlugin;

autumn_web::app().plugin(DatatablePlugin)
```

The plugin serves the pack CSS and JS and adds the stories to `/_stories`.
In the layout `<head>`, after the core widget assets, add the pack links:

```rust
head {
    link rel="stylesheet" href=(autumn_web::ui::WIDGETS_CSS_PATH);
    script src="/static/js/autumn-widgets.js" defer {}
    (autumn_datatable_plugin::pack().head())
}
```

## Use

This is a sketch. `Post`, `PgPostRepository`, `layout` and `csrf_field` are
the app's own. Import `html`, `get`, `post`, `Markup`, `Redirect`,
`AutumnError` and `AutumnResult` from `autumn_web::prelude`, and
`pagination_nav` and `PagerOptions` from `autumn_web::ui::pagination`.

```rust
use autumn_datatable_plugin::{BulkAction, BulkSelection, Column, DataTable, Filter, TableState};
use autumn_web::extract::Query;

const STATUS: &[(&str, &str)] = &[("draft", "Draft"), ("live", "Live")];

fn columns() -> Vec<Column<'static, Post>> {
    vec![
        Column::new("title", "Title", |p: &Post| html! { (p.title) }).sortable().row_header(),
        Column::new("status", "Status", |p: &Post| html! { (p.status) }).sortable().hideable(),
        Column::new("id", "ID", |p: &Post| html! { (p.id) }).hidden(),
    ]
}

#[get("/posts")]
async fn index(Query(raw): Query<TableState>, repo: PgPostRepository) -> AutumnResult<Markup> {
    let columns = columns();
    let filters = [Filter::multi("status", "Status", STATUS)];
    // The safe default: only declared keys, `size` clamped.
    let state = raw.checked(&columns, &filters);
    // `list_query` has the single-value filters only. Apply a filter with
    // two or more values yourself, with `eq_any(state.filter("status"))`.
    let page = repo.list(&state.list_query(), &state.page_request()).await?;
    let actions = [BulkAction::new("delete", "Delete").confirm("Delete the selected posts?")];
    // The root passthrough: htmx boost, and `aria-busy` during a request.
    let options = FieldOptions::new().hx("boost", "true").hx("indicator", "this");
    Ok(layout(html! {
        (DataTable::new("posts", "Posts", &page.content, &columns)
            .options(&options)
            .state(&state)
            .base_path("/posts")
            .search("Search posts")
            .filters(&filters)
            .density_switch()
            .selectable(|p| p.id.to_string())
            .row_label(|p| p.title.clone())
            // Put the CSRF input in the form. The form is a POST.
            .bulk_actions("/posts/actions", &actions, csrf_field())
            .empty("No posts yet", "Write the first post.", None))
        (pagination_nav(&page, &PagerOptions::new("/posts").query(&state.to_query())))
    }))
}

#[post("/posts/actions")]
async fn actions(body: String) -> AutumnResult<Redirect> {
    let bulk = BulkSelection::from_form(body.as_bytes())
        .map_err(|err| AutumnError::bad_request_msg(err.to_string()))?;
    let actions = [BulkAction::new("delete", "Delete").confirm("Delete the selected posts?")];
    // With JavaScript off, the dialog does not open. Ask on a server page
    // that posts the same body with `confirmed=1`.
    if bulk.needs_confirm(&actions) {
        return Ok(Redirect::to("/posts/actions/confirm"));
    }
    if bulk.action_in(&actions).map(BulkAction::name) == Some("delete") {
        // Check access to each id in `bulk.ids()`, then delete.
    }
    Ok(Redirect::to("/posts"))
}
```

## The state

`TableState` decodes with `Query<TableState>` or `TableState::from_query`.
`to_query` encodes it back to the same string. Each link method changes one
part:

| Method | Page |
|---|---|
| `sorted_by(key)`: it flips the direction when `key` is the sort now. If not, it sorts ascending | back to 1 |
| `with_sort`, `with_search`, `with_filter`, `with_filter_value`, `without_filter_value`, `without_filter`, `cleared`, `with_size` | back to 1 |
| `with_page`, `with_density`, `with_columns`, `default_columns` | kept |

Keys are `[a-z0-9_]+`. The decoder keeps at most 32 filters, 64 values for
each filter and 64 columns.

`checked(&columns, &filters)` is the safe default:

- It drops an unknown sort key.
- It drops an undeclared filter and a value that is not an option.
- It drops a column that is not hideable.
- It cuts text to 200 characters and clamps `size` to 100.

The raw state stays available for an app that needs more.

## Keyboard

The table uses native controls. It has no arrow-key code.

| Key | Effect |
|---|---|
| Tab | Moves to the scroll region, the select-all box, then each row box. |
| Space on select-all | Off or mixed: selects each row. On: clears each row. With JavaScript off, the box only sends `all`. |
| Space on a row box | Toggles the row. Select-all shows the mixed state. |
| Enter on "Apply action" | Posts the action. With no row, the status says "Select at least one row." |
| Escape in the confirm dialog | Closes the dialog. Focus goes back to "Apply action". |

The bulk `<form>` holds no cell. The boxes, the select and the button join
it with the `form` attribute. Then a form in a cell does not nest.

## Tests

```bash
cargo test
```

The browser tests need Chromium. They run the plugin conformance gate (axe in
the light and the dark scheme, console, keyboard) and the selection keyboard
tests:

```bash
AUTUMN_CHROMIUM=/path/to/chrome cargo test --features system-tests --test browser -- --include-ignored
```

`tests/fixtures/axe-core/` is a copy of axe-core 4.x (MPL-2.0). It is not in
the published crate.

## Licence

MIT or Apache-2.0.
