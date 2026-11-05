//! `DataTable` markup.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on a panic"
)]

use autumn_datatable_plugin::{BulkAction, Column, DataTable, Density, Filter, TableState};
use autumn_web::form::FieldOptions;
use maud::html;

struct Post {
    id: i64,
    title: &'static str,
    status: &'static str,
}

fn posts() -> Vec<Post> {
    vec![
        Post {
            id: 1,
            title: "Alpha",
            status: "open",
        },
        Post {
            id: 2,
            title: "Beta",
            status: "closed",
        },
        Post {
            id: 3,
            title: "Gamma",
            status: "open",
        },
    ]
}

fn columns() -> Vec<Column<'static, Post>> {
    vec![
        Column::new("title", "Title", |p: &Post| html! { (p.title) })
            .sortable()
            .row_header(),
        Column::new("status", "Status", |p: &Post| html! { (p.status) })
            .sortable()
            .hideable(),
        Column::new("id", "ID", |p: &Post| html! { (p.id) }).hidden(),
    ]
}

const STATUS: &[(&str, &str)] = &[("open", "Open"), ("closed", "Closed")];
const KIND: &[(&str, &str)] = &[("bug", "Bug"), ("idea", "Idea")];

fn filters() -> Vec<Filter<'static>> {
    vec![
        Filter::multi("status", "Status", STATUS),
        Filter::select("kind", "Kind", KIND),
        Filter::text("author", "Author"),
    ]
}

fn state(query: &str) -> TableState {
    TableState::from_query(query).expect("decodes")
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

/// The text of the first element that opens with `open`.
fn element<'h>(html: &'h str, open: &str) -> &'h str {
    let start = html
        .find(open)
        .unwrap_or_else(|| panic!("no `{open}` in {html}"));
    let rest = &html[start..];
    &rest[..rest.find('>').map_or(rest.len(), |end| end + 1)]
}

// ── Structure ───────────────────────────────────────────────────────────

#[test]
fn the_table_has_a_caption_headers_and_rows() {
    let rows = posts();
    let cols = columns();
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .render()
        .into_string();
    assert!(html.contains(r#"<caption id="posts-caption">Posts</caption>"#));
    assert!(html.contains(r#"class="autumn-datatable-table""#));
    assert!(html.contains(r#"<th scope="row">Alpha</th>"#));
    assert_eq!(count(&html, r#"<th scope="col""#), 2);
    assert_eq!(count(&html, "<tr"), 4);
    assert!(html.contains("<td>open</td>"));
}

#[test]
fn the_root_is_a_labelled_region_that_scrolls() {
    let rows = posts();
    let cols = columns();
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .render()
        .into_string();
    let root = element(&html, "<div");
    assert!(root.contains(r#"class="autumn-datatable-root""#), "{root}");
    assert!(root.contains(r#"id="posts""#));
    assert!(root.contains("data-autumn-datatable"));
    assert!(root.contains("data-autumn-busy"));
    let scroll = element(&html, r#"<div class="autumn-datatable-scroll""#);
    assert!(scroll.contains(r#"role="region""#));
    assert!(scroll.contains(r#"aria-labelledby="posts-caption""#));
    assert!(scroll.contains(r#"tabindex="0""#));
}

#[test]
fn the_app_passthrough_goes_on_the_root() {
    let rows = posts();
    let cols = columns();
    let options = FieldOptions::new()
        .data("test-id", "posts")
        .hx("boost", "true")
        .hx("indicator", "this");
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .options(&options)
        .render()
        .into_string();
    let root = element(&html, "<div");
    assert!(root.contains(r#"data-test-id="posts""#), "{root}");
    assert!(root.contains(r#"hx-boost="true""#));
    assert!(root.contains(r#"hx-indicator="this""#));
}

#[test]
fn the_table_renders_through_maud_render() {
    let rows = posts();
    let cols = columns();
    let table = DataTable::new("posts", "Posts", &rows, &cols);
    let html = html! { (table) }.into_string();
    assert!(html.contains("autumn-datatable-table"));
}

#[test]
#[should_panic(expected = "table id")]
fn a_bad_table_id_panics() {
    let rows = posts();
    let cols = columns();
    let _ = DataTable::new("Posts table", "Posts", &rows, &cols);
}

#[test]
#[should_panic(expected = "column key")]
fn a_bad_column_key_panics() {
    let _ = Column::new("a b", "A", |_: &Post| html! {});
}

#[test]
fn two_tables_have_distinct_ids() {
    let rows = posts();
    let cols = columns();
    let a = DataTable::new("a", "A", &rows, &cols)
        .selectable(|p| p.id.to_string())
        .render()
        .into_string();
    let b = DataTable::new("b", "B", &rows, &cols)
        .selectable(|p| p.id.to_string())
        .render()
        .into_string();
    for (own, html) in [("a", &a), ("b", &b)] {
        for id in html
            .split(" id=\"")
            .skip(1)
            .map(|s| &s[..s.find('"').unwrap_or(0)])
        {
            assert!(id == own || id.starts_with(&format!("{own}-")), "id `{id}`");
        }
    }
}

// ── Sort ────────────────────────────────────────────────────────────────

#[test]
fn a_sortable_header_has_aria_sort_and_a_link_that_keeps_the_state() {
    let rows = posts();
    let cols = columns();
    let st = state("sort=title&page=3&filter[status]=open&density=compact");
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .state(&st)
        .base_path("/posts")
        .render()
        .into_string();
    assert!(
        html.contains(r#"<th scope="col" aria-sort="ascending">"#),
        "{html}"
    );
    assert!(html.contains(
        r#"href="/posts?filter[status]=open&amp;sort=title&amp;dir=desc&amp;density=compact""#
    ));
    assert!(html.contains(r#"<th scope="col" aria-sort="none">"#));
    assert!(html.contains(
        r#"href="/posts?filter[status]=open&amp;sort=status&amp;dir=asc&amp;density=compact""#
    ));
}

#[test]
fn a_header_that_is_not_sortable_has_no_link() {
    let rows = posts();
    let cols = vec![Column::new(
        "title",
        "Title",
        |p: &Post| html! { (p.title) },
    )];
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .render()
        .into_string();
    assert!(html.contains(r#"<th scope="col">Title</th>"#), "{html}");
    assert!(!html.contains("aria-sort"));
}

// ── Column visibility ───────────────────────────────────────────────────

#[test]
fn a_hidden_column_is_not_rendered_by_default() {
    let rows = posts();
    let cols = columns();
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .render()
        .into_string();
    assert!(!html.contains("<td>1</td>"));
    assert!(html.contains("<td>open</td>"));
}

#[test]
fn the_cols_parameter_picks_the_optional_columns() {
    let rows = posts();
    let cols = columns();
    let st = state("cols=id");
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .state(&st)
        .render()
        .into_string();
    assert!(html.contains("<td>1</td>"));
    assert!(!html.contains("<td>open</td>"));
    // `title` is not hideable: it stays.
    assert!(html.contains(r#"<th scope="row">Alpha</th>"#));
}

#[test]
fn all_columns_off_shows_the_default_columns() {
    let rows = posts();
    let cols = vec![
        Column::new("title", "Title", |p: &Post| html! { (p.title) }).hideable(),
        Column::new("status", "Status", |p: &Post| html! { (p.status) }).hideable(),
    ];
    let st = state("cols=");
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .state(&st)
        .render()
        .into_string();
    assert!(html.contains("<td>Alpha</td>"), "{html}");
    assert!(html.contains("<td>open</td>"));
}

#[test]
fn the_column_picker_is_a_get_form_with_the_cols_marker() {
    let rows = posts();
    let cols = columns();
    let st = state("q=x&sort=title&page=2&filter[status]=open");
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .state(&st)
        .base_path("/posts")
        .render()
        .into_string();
    assert!(
        html.contains(r#"<details class="autumn-datatable-columns">"#),
        "{html}"
    );
    let picker = &html[html.find("autumn-datatable-columns").expect("picker")..];
    let picker = &picker[..picker.find("</details>").expect("end")];
    assert!(picker.contains(r#"method="get""#));
    assert!(picker.contains(r#"action="/posts""#));
    assert!(picker.contains(r#"<input type="hidden" name="cols" value="">"#));
    assert!(
        picker.contains(r#"name="cols" value="status" checked"#),
        "{picker}"
    );
    assert!(picker.contains(r#"name="cols" value="id""#));
    assert!(!picker.contains(r#"name="cols" value="title""#));
    // The rest of the state goes along.
    for hidden in [
        r#"name="q" value="x""#,
        r#"name="filter[status]" value="open""#,
        r#"name="sort" value="title""#,
        r#"name="dir" value="asc""#,
        r#"name="page" value="2""#,
    ] {
        assert!(picker.contains(hidden), "no {hidden} in {picker}");
    }
}

#[test]
fn no_hideable_column_means_no_picker() {
    let rows = posts();
    let cols = vec![Column::new(
        "title",
        "Title",
        |p: &Post| html! { (p.title) },
    )];
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .render()
        .into_string();
    assert!(!html.contains("autumn-datatable-columns"));
}

// ── Density and sticky header ───────────────────────────────────────────

#[test]
fn the_density_switch_links_to_each_density() {
    let rows = posts();
    let cols = columns();
    let st = state("page=2&density=compact");
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .state(&st)
        .base_path("/posts")
        .density_switch()
        .render()
        .into_string();
    assert!(element(&html, "<div").contains("autumn-datatable-root--compact"));
    assert!(
        html.contains(r#"role="group" aria-label="Density""#),
        "{html}"
    );
    assert!(
        html.contains(r#"<a href="/posts?page=2">Comfortable</a>"#),
        "{html}"
    );
    assert!(html.contains(
        r#"<a href="/posts?page=2&amp;density=compact" aria-current="true">Compact</a>"#
    ));
    assert_eq!(Density::Compact.param_value(), "compact");
}

#[test]
fn a_sticky_header_sets_the_root_class() {
    let rows = posts();
    let cols = columns();
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .sticky_header()
        .render()
        .into_string();
    assert!(element(&html, "<div").contains("autumn-datatable-root--sticky"));
}

// ── Selection ───────────────────────────────────────────────────────────

fn selectable_html(selected: &[&str]) -> String {
    let rows = posts();
    let cols = columns();
    let ids: Vec<String> = selected.iter().map(|s| (*s).to_owned()).collect();
    DataTable::new("posts", "Posts", &rows, &cols)
        .selectable(|p| p.id.to_string())
        .row_label(|p| p.title.to_owned())
        .selected(&ids)
        .render()
        .into_string()
}

#[test]
fn each_row_has_a_named_checkbox() {
    let html = selectable_html(&[]);
    assert_eq!(count(&html, r#"name="ids""#), 3);
    assert!(html.contains(
        r#"<input type="checkbox" class="autumn-datatable-check" name="ids" value="1" aria-label="Select Alpha" data-autumn-datatable-row>"#
    ), "{html}");
}

#[test]
fn select_all_is_off_when_no_row_is_selected() {
    let html = selectable_html(&[]);
    let all = element(
        &html,
        r#"<input type="checkbox" class="autumn-datatable-check" id="posts-all""#,
    );
    assert!(all.contains(r#"name="all" value="1""#), "{all}");
    assert!(all.contains(r#"aria-label="Select all rows""#));
    assert!(!all.contains("checked"));
    assert!(html.contains("0 of 3 selected"));
}

#[test]
fn select_all_is_mixed_when_some_rows_are_selected() {
    let html = selectable_html(&["2"]);
    let all = element(&html, r#"id="posts-all""#);
    assert!(all.contains(r#"aria-checked="mixed""#), "{all}");
    assert!(!all.contains(" checked"));
    assert!(html.contains(r#"value="2" aria-label="Select Beta" checked"#));
    assert!(html.contains("1 of 3 selected"));
}

#[test]
fn select_all_is_on_when_each_row_is_selected() {
    let html = selectable_html(&["1", "2", "3"]);
    let all = element(&html, r#"id="posts-all""#);
    assert!(all.contains(" checked"), "{all}");
    assert!(!all.contains("aria-checked"));
    assert!(html.contains("3 of 3 selected"));
}

#[test]
fn the_selection_column_header_has_a_name() {
    let html = selectable_html(&[]);
    assert!(
        html.contains(r#"<th scope="col" class="autumn-datatable-select">"#),
        "{html}"
    );
    let status = element(&html, r#"<p id="posts-status""#);
    assert!(status.contains(r#"role="status""#), "{status}");
    assert!(status.contains("data-autumn-datatable-status"));
}

#[test]
fn the_default_row_label_is_the_id() {
    let rows = posts();
    let cols = columns();
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .selectable(|p| p.id.to_string())
        .render()
        .into_string();
    assert!(html.contains(r#"aria-label="Select row 1""#), "{html}");
}

// ── Bulk actions ────────────────────────────────────────────────────────

fn bulk_html(actions: &[BulkAction<'_>]) -> String {
    let rows = posts();
    let cols = columns();
    DataTable::new("posts", "Posts", &rows, &cols)
        .selectable(|p| p.id.to_string())
        .bulk_actions(
            "/posts/actions",
            actions,
            html! { input type="hidden" name="csrf" value="tok"; },
        )
        .render()
        .into_string()
}

#[test]
fn bulk_actions_wrap_the_table_in_a_post_form() {
    let html = bulk_html(&[BulkAction::new("archive", "Archive")]);
    let form = element(&html, "<form id=\"posts-bulk\"");
    assert!(form.contains(r#"method="post""#), "{form}");
    assert!(form.contains(r#"action="/posts/actions""#));
    assert!(html.contains(r#"<input type="hidden" name="csrf" value="tok">"#));
    assert_eq!(count(&html, r#"<input type="hidden" name="visible""#), 3);
    let form_start = html.find("<form id=\"posts-bulk\"").expect("form");
    let table_start = html.find("<table").expect("table");
    let form_end = html.rfind("</form>").expect("end");
    assert!(form_start < table_start && table_start < form_end);
    assert!(
        html.contains(r#"<select id="posts-action" name="action""#),
        "{html}"
    );
    assert!(html.contains(r#"<option value="archive">Archive</option>"#));
    assert!(html.contains(r#"<label for="posts-action">"#));
    assert!(html.contains(r#"type="submit""#));
    assert!(html.contains(r#"data-autumn-datatable-none="Select at least one row.""#));
    assert!(!html.contains("<dialog"));
}

#[test]
fn a_confirm_action_adds_the_dialog() {
    let html = bulk_html(&[
        BulkAction::new("archive", "Archive"),
        BulkAction::new("delete", "Delete").confirm("Delete the selected rows?"),
    ]);
    assert!(html.contains(
        r#"<option value="delete" data-autumn-datatable-confirm="Delete the selected rows?">Delete</option>"#
    ), "{html}");
    let dialog = element(&html, "<dialog");
    assert!(dialog.contains(r#"id="posts-confirm""#), "{dialog}");
    assert!(dialog.contains(r#"aria-labelledby="posts-confirm-title""#));
    assert!(html.contains("data-autumn-datatable-ok"));
    assert!(html.contains("data-autumn-datatable-cancel"));
    // The dialog is not in the form: its buttons do not submit it.
    assert!(html.find("<dialog").expect("dialog") > html.rfind("</form>").expect("form"));
}

#[test]
fn no_bulk_actions_means_no_form() {
    let html = selectable_html(&[]);
    assert!(!html.contains("<form id=\"posts-bulk\""));
    assert!(!html.contains(r#"name="visible""#));
}

// ── Filter bar ──────────────────────────────────────────────────────────

fn filter_html(query: &str) -> String {
    let rows = posts();
    let cols = columns();
    let filters = filters();
    let st = state(query);
    DataTable::new("posts", "Posts", &rows, &cols)
        .state(&st)
        .base_path("/posts")
        .search("Search posts")
        .filters(&filters)
        .render()
        .into_string()
}

#[test]
fn the_filter_bar_is_a_get_search_form() {
    let html = filter_html("sort=title&dir=desc&page=4&cols=status&density=compact&size=50");
    let form = element(&html, r#"<form id="posts-filters""#);
    assert!(form.contains(r#"method="get""#), "{form}");
    assert!(form.contains(r#"action="/posts""#));
    assert!(form.contains(r#"role="search""#));
    assert!(form.contains(r#"aria-label="Filter Posts""#));
    let bar = &html[html.find(r#"<form id="posts-filters""#).expect("bar")..];
    let bar = &bar[..bar.find("</form>").expect("end")];
    for hidden in [
        r#"name="sort" value="title""#,
        r#"name="dir" value="desc""#,
        r#"name="cols" value="status""#,
        r#"name="density" value="compact""#,
        r#"name="size" value="50""#,
    ] {
        assert!(bar.contains(hidden), "no {hidden} in {bar}");
    }
    assert!(
        !bar.contains(r#"name="page""#),
        "a filter change resets the page"
    );
    assert!(bar.contains(r#"<label for="posts-q">Search posts</label>"#));
    assert!(bar.contains(r#"<input type="search" id="posts-q" name="q""#));
}

#[test]
fn each_filter_kind_shows_the_current_value() {
    let html = filter_html("q=hi&filter[status]=closed&filter[kind]=idea&filter[author]=ann");
    assert!(html.contains(r#"name="q" value="hi""#));
    assert!(
        html.contains(r#"<fieldset class="autumn-datatable-filter">"#),
        "{html}"
    );
    assert!(html.contains("<legend>Status</legend>"));
    assert!(
        html.contains(r#"name="filter[status]" value="closed" checked"#),
        "{html}"
    );
    assert!(html.contains(
        r#"<input type="checkbox" id="posts-filter-status-0" name="filter[status]" value="open">"#
    ));
    assert!(html.contains(r#"<select id="posts-filter-kind" name="filter[kind]">"#));
    assert!(html.contains(r#"<option value="">Any</option>"#));
    assert!(html.contains(r#"<option value="idea" selected>Idea</option>"#));
    assert!(html.contains(r#"<label for="posts-filter-author">Author</label>"#));
    assert!(html.contains(
        r#"<input type="text" id="posts-filter-author" name="filter[author]" value="ann">"#
    ));
}

#[test]
fn active_filters_show_as_chips_that_remove_one_value() {
    let html = filter_html("q=hi&filter[status]=closed&filter[status]=open&sort=title");
    assert!(
        html.contains(r#"<ul class="autumn-datatable-chips" aria-label="Active filters">"#),
        "{html}"
    );
    assert!(html.contains(
        r#"href="/posts?q=hi&amp;filter[status]=open&amp;sort=title&amp;dir=asc" aria-label="Remove filter Status: Closed""#
    ), "{html}");
    assert!(html.contains(r#"aria-label="Remove search hi""#));
    assert!(html.contains(r#"<a class="autumn-datatable-clear" href="/posts?sort=title&amp;dir=asc">Clear filters</a>"#));
}

#[test]
fn no_active_filter_means_no_chips() {
    let html = filter_html("sort=title");
    assert!(!html.contains("autumn-datatable-chips"));
    assert!(!html.contains("Clear filters"));
}

// ── Empty and loading states ────────────────────────────────────────────

#[test]
fn an_empty_table_shows_the_empty_state() {
    let rows: Vec<Post> = Vec::new();
    let cols = columns();
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .selectable(|p| p.id.to_string())
        .empty(
            "No posts yet",
            "Write the first post.",
            Some(html! { a href="/posts/new" { "New post" } }),
        )
        .render()
        .into_string();
    assert!(html.contains(r#"<td colspan="3">"#), "{html}");
    assert!(html.contains("autumn-empty-state"));
    assert!(html.contains("No posts yet"));
    assert!(html.contains(r#"<a href="/posts/new">New post</a>"#));
    let all = element(&html, r#"id="posts-all""#);
    assert!(all.contains("disabled"), "{all}");
}

#[test]
fn an_empty_table_has_a_default_message() {
    let rows: Vec<Post> = Vec::new();
    let cols = columns();
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .render()
        .into_string();
    assert!(html.contains("No rows"), "{html}");
}

#[test]
fn an_empty_filtered_table_offers_to_clear_the_filters() {
    let rows: Vec<Post> = Vec::new();
    let cols = columns();
    let st = state("filter[status]=open&sort=title");
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .state(&st)
        .base_path("/posts")
        .empty("No posts yet", "", None)
        .render()
        .into_string();
    assert!(html.contains("No rows match"), "{html}");
    assert!(!html.contains("No posts yet"));
    assert!(
        html.contains(r#"<a href="/posts?sort=title&amp;dir=asc">Clear filters</a>"#),
        "{html}"
    );
}

#[test]
fn the_loading_state_has_skeleton_rows_and_aria_busy() {
    let rows: Vec<Post> = Vec::new();
    let cols = columns();
    let html = DataTable::new("posts", "Posts", &rows, &cols)
        .loading(4)
        .render()
        .into_string();
    let table = element(&html, "<table");
    assert!(table.contains(r#"aria-busy="true""#), "{table}");
    assert_eq!(count(&html, "autumn-skeleton--line"), 8);
    assert!(html.contains("Loading"));
    assert!(!html.contains("No rows"));
}

// ── Escaping ────────────────────────────────────────────────────────────

#[test]
fn state_values_are_escaped_in_markup_and_links() {
    let html = filter_html("q=%22%3E%3Cscript%3Ex&filter[author]=%3Cb%3E&sort=title");
    assert!(!html.contains("<script>"), "{html}");
    assert!(!html.contains("<b>"));
    assert!(html.contains("&quot;&gt;&lt;script&gt;x"));
    assert!(html.contains("q=%22%3E%3Cscript%3Ex"));
}
