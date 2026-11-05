//! The `DataTable` widget and its parts.

use std::borrow::Cow;
use std::collections::BTreeSet;

use autumn_web::form::FieldOptions;
use autumn_web::widget_pack::element;
use autumn_web::widgets::{SkeletonShape, empty_state, skeleton};
use maud::{Markup, Render, html};

use crate::selection::SelectAll;
use crate::state::{Density, TableState};

/// The message when a bulk submit has no selected row.
const NONE_SELECTED: &str = "Select at least one row.";

/// `true` for `[a-z0-9_-]+`.
fn is_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// `true` for `[a-z][a-z0-9-]*`.
fn is_table_id(id: &str) -> bool {
    id.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

// ── Column ──────────────────────────────────────────────────────────────

/// One column of a [`DataTable`].
///
/// ```rust
/// use autumn_datatable_plugin::Column;
///
/// struct Post { title: String }
///
/// let title = Column::new("title", "Title", |p: &Post| maud::html! { (p.title) })
///     .sortable()
///     .row_header();
/// assert_eq!(title.key(), "title");
/// ```
pub struct Column<'a, T> {
    key: &'a str,
    header: &'a str,
    sort_key: Option<&'a str>,
    hideable: bool,
    default_visible: bool,
    row_header: bool,
    cell: Box<dyn Fn(&T) -> Markup + 'a>,
}

impl<'a, T> Column<'a, T> {
    /// A column that always shows and does not sort.
    ///
    /// # Panics
    ///
    /// Panics when `key` is not `[a-z0-9_-]+`. The key goes into `cols=`
    /// and into ids.
    #[must_use]
    pub fn new(key: &'a str, header: &'a str, cell: impl Fn(&T) -> Markup + 'a) -> Self {
        assert!(
            is_key(key),
            "column key {key:?} is not valid: use [a-z0-9_-]+"
        );
        Self {
            key,
            header,
            sort_key: None,
            hideable: false,
            default_visible: true,
            row_header: false,
            cell: Box::new(cell),
        }
    }

    /// Sort by the column key.
    #[must_use]
    pub const fn sortable(self) -> Self {
        let key = self.key;
        self.sort_key(key)
    }

    /// Sort by `sort_key`. Use it when the sort key is not the column key.
    #[must_use]
    pub const fn sort_key(mut self, sort_key: &'a str) -> Self {
        self.sort_key = Some(sort_key);
        self
    }

    /// The user can hide the column. It shows by default.
    #[must_use]
    pub const fn hideable(mut self) -> Self {
        self.hideable = true;
        self
    }

    /// The user can show the column. It is hidden by default.
    #[must_use]
    pub const fn hidden(mut self) -> Self {
        self.hideable = true;
        self.default_visible = false;
        self
    }

    /// The cells are row headers (`<th scope="row">`). Use it on the column
    /// that names the row.
    #[must_use]
    pub const fn row_header(mut self) -> Self {
        self.row_header = true;
        self
    }

    /// The column key.
    #[must_use]
    pub const fn key(&self) -> &'a str {
        self.key
    }

    /// The header text.
    #[must_use]
    pub const fn header(&self) -> &'a str {
        self.header
    }

    /// `true` when the user can hide or show the column.
    #[must_use]
    pub const fn is_hideable(&self) -> bool {
        self.hideable
    }

    pub(crate) const fn sort_key_str(&self) -> Option<&'a str> {
        self.sort_key
    }
}

// ── Filter ──────────────────────────────────────────────────────────────

/// The control of a [`Filter`].
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum FilterKind<'a> {
    /// A text input. One value.
    Text,
    /// A `<select>` of `(value, label)` options. One value.
    Select(&'a [(&'a str, &'a str)]),
    /// A checkbox group of `(value, label)` options. Many values.
    Multi(&'a [(&'a str, &'a str)]),
}

/// One filter in the filter bar. It writes `filter[<key>]`.
#[derive(Debug, Clone, Copy)]
pub struct Filter<'a> {
    key: &'a str,
    label: &'a str,
    kind: FilterKind<'a>,
}

impl<'a> Filter<'a> {
    /// # Panics
    ///
    /// Panics when `key` is not `[a-z0-9_-]+`.
    fn new(key: &'a str, label: &'a str, kind: FilterKind<'a>) -> Self {
        assert!(
            is_key(key),
            "filter key {key:?} is not valid: use [a-z0-9_-]+"
        );
        Self { key, label, kind }
    }

    /// A text filter.
    ///
    /// # Panics
    ///
    /// Panics when `key` is not `[a-z0-9_-]+`.
    #[must_use]
    pub fn text(key: &'a str, label: &'a str) -> Self {
        Self::new(key, label, FilterKind::Text)
    }

    /// A filter with one value from `options`.
    ///
    /// # Panics
    ///
    /// Panics when `key` is not `[a-z0-9_-]+`.
    #[must_use]
    pub fn select(key: &'a str, label: &'a str, options: &'a [(&'a str, &'a str)]) -> Self {
        Self::new(key, label, FilterKind::Select(options))
    }

    /// A filter with many values from `options`.
    ///
    /// # Panics
    ///
    /// Panics when `key` is not `[a-z0-9_-]+`.
    #[must_use]
    pub fn multi(key: &'a str, label: &'a str, options: &'a [(&'a str, &'a str)]) -> Self {
        Self::new(key, label, FilterKind::Multi(options))
    }

    /// The filter key.
    #[must_use]
    pub const fn key(&self) -> &'a str {
        self.key
    }

    /// The label.
    #[must_use]
    pub const fn label(&self) -> &'a str {
        self.label
    }

    /// The control.
    #[must_use]
    pub const fn kind(&self) -> FilterKind<'a> {
        self.kind
    }

    /// The label of `value`: the option label, or the value for a text
    /// filter.
    fn value_label<'v>(&self, value: &'v str) -> &'v str
    where
        'a: 'v,
    {
        match self.kind {
            FilterKind::Text => value,
            FilterKind::Select(options) | FilterKind::Multi(options) => options
                .iter()
                .find(|(v, _)| *v == value)
                .map_or(value, |(_, label)| label),
        }
    }
}

// ── BulkAction ──────────────────────────────────────────────────────────

/// One action in the bulk action bar.
#[derive(Debug, Clone, Copy)]
pub struct BulkAction<'a> {
    name: &'a str,
    label: &'a str,
    confirm: Option<&'a str>,
}

impl<'a> BulkAction<'a> {
    /// An action that runs at once.
    #[must_use]
    pub const fn new(name: &'a str, label: &'a str) -> Self {
        Self {
            name,
            label,
            confirm: None,
        }
    }

    /// Ask the user with `text` before the form posts. Without JavaScript
    /// the form posts at once: also check on the server.
    #[must_use]
    pub const fn confirm(mut self, text: &'a str) -> Self {
        self.confirm = Some(text);
        self
    }

    /// The name. The form posts it as `action`.
    #[must_use]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// The label.
    #[must_use]
    pub const fn label(&self) -> &'a str {
        self.label
    }

    /// The confirm text, if the action asks first.
    #[must_use]
    pub const fn confirm_text(&self) -> Option<&'a str> {
        self.confirm
    }
}

// ── DataTable ───────────────────────────────────────────────────────────

struct Bulk<'a> {
    action: &'a str,
    actions: &'a [BulkAction<'a>],
    fields: Markup,
}

struct Empty<'a> {
    title: &'a str,
    body: &'a str,
    action: Option<Markup>,
}

type RowText<'a, T> = Box<dyn Fn(&T) -> String + 'a>;

/// A data table that the server drives with query parameters.
///
/// It works with JavaScript off. Links and GET forms carry the full
/// [`TableState`]. The pack script only syncs the select-all state, the
/// count and the confirm dialog.
///
/// ```rust
/// use autumn_datatable_plugin::{Column, DataTable, TableState};
///
/// struct Post { id: i64, title: String }
///
/// let rows = vec![Post { id: 1, title: "Hello".into() }];
/// let columns = vec![
///     Column::new("title", "Title", |p: &Post| maud::html! { (p.title) }).sortable().row_header(),
/// ];
/// let state = TableState::from_query("sort=title").unwrap().checked(&columns, &[]);
/// let html = DataTable::new("posts", "Posts", &rows, &columns)
///     .state(&state)
///     .base_path("/posts")
///     .selectable(|p| p.id.to_string())
///     .render()
///     .into_string();
/// assert!(html.contains(r#"aria-sort="ascending""#));
/// ```
pub struct DataTable<'a, T> {
    id: &'a str,
    caption: &'a str,
    rows: &'a [T],
    columns: &'a [Column<'a, T>],
    state: Cow<'a, TableState>,
    base_path: &'a str,
    search: Option<&'a str>,
    filters: &'a [Filter<'a>],
    density_switch: bool,
    sticky: bool,
    row_id: Option<RowText<'a, T>>,
    row_label: Option<RowText<'a, T>>,
    selected: &'a [String],
    bulk: Option<Bulk<'a>>,
    empty: Option<Empty<'a>>,
    loading: Option<usize>,
    options: Option<&'a FieldOptions<'a>>,
}

impl<'a, T> DataTable<'a, T> {
    /// A table with an id, a caption, rows and columns.
    ///
    /// Each element id starts with `id`. The caption names the table.
    ///
    /// # Panics
    ///
    /// Panics when `id` is not `[a-z][a-z0-9-]*`.
    #[must_use]
    pub fn new(id: &'a str, caption: &'a str, rows: &'a [T], columns: &'a [Column<'a, T>]) -> Self {
        assert!(
            is_table_id(id),
            "table id {id:?} is not valid: use [a-z][a-z0-9-]*"
        );
        Self {
            id,
            caption,
            rows,
            columns,
            state: Cow::Owned(TableState::new()),
            base_path: "",
            search: None,
            filters: &[],
            density_switch: false,
            sticky: false,
            row_id: None,
            row_label: None,
            selected: &[],
            bulk: None,
            empty: None,
            loading: None,
            options: None,
        }
    }

    /// The current state. Give the [`TableState::checked`] state.
    #[must_use]
    pub fn state(mut self, state: &'a TableState) -> Self {
        self.state = Cow::Borrowed(state);
        self
    }

    /// The path of the links and the GET forms, for example `"/posts"`.
    #[must_use]
    pub const fn base_path(mut self, path: &'a str) -> Self {
        self.base_path = path;
        self
    }

    /// Show a search input with this label. It writes `q`.
    #[must_use]
    pub const fn search(mut self, label: &'a str) -> Self {
        self.search = Some(label);
        self
    }

    /// Show the filter bar with these filters.
    #[must_use]
    pub const fn filters(mut self, filters: &'a [Filter<'a>]) -> Self {
        self.filters = filters;
        self
    }

    /// Show the density links.
    #[must_use]
    pub const fn density_switch(mut self) -> Self {
        self.density_switch = true;
        self
    }

    /// Keep the header row on screen when the table scrolls.
    #[must_use]
    pub const fn sticky_header(mut self) -> Self {
        self.sticky = true;
        self
    }

    /// Add a selection column. `id` gives the row id that the form posts.
    #[must_use]
    pub fn selectable(mut self, id: impl Fn(&T) -> String + 'a) -> Self {
        self.row_id = Some(Box::new(id));
        self
    }

    /// The row name for the checkbox label, "Select {name}". The default is
    /// "Select row {id}".
    #[must_use]
    pub fn row_label(mut self, label: impl Fn(&T) -> String + 'a) -> Self {
        self.row_label = Some(Box::new(label));
        self
    }

    /// The ids that are selected now, for example after a failed action.
    #[must_use]
    pub const fn selected(mut self, ids: &'a [String]) -> Self {
        self.selected = ids;
        self
    }

    /// Put the table in a POST form to `action`, with a bulk action bar.
    /// `fields` goes in the form, for example the CSRF input.
    #[must_use]
    pub fn bulk_actions(
        mut self,
        action: &'a str,
        actions: &'a [BulkAction<'a>],
        fields: Markup,
    ) -> Self {
        self.bulk = Some(Bulk {
            action,
            actions,
            fields,
        });
        self
    }

    /// The empty state: a title, a text and an optional action. With an
    /// active filter, the table shows "No rows match" and a link that
    /// clears the filters.
    #[must_use]
    pub fn empty(mut self, title: &'a str, body: &'a str, action: Option<Markup>) -> Self {
        self.empty = Some(Empty {
            title,
            body,
            action,
        });
        self
    }

    /// Show `rows` skeleton rows and `aria-busy="true"` in place of the
    /// rows. Use it for a table that loads later.
    #[must_use]
    pub const fn loading(mut self, rows: usize) -> Self {
        self.loading = Some(rows);
        self
    }

    /// The app passthrough on the root: `data-*` and `hx-*`, for example
    /// `hx-boost`.
    #[must_use]
    pub const fn options(mut self, options: &'a FieldOptions<'a>) -> Self {
        self.options = Some(options);
        self
    }

    /// Render the table.
    #[must_use]
    pub fn render(&self) -> Markup {
        let mut class = String::from("autumn-datatable-root");
        if self.state.density() == Density::Compact {
            class.push_str(" autumn-datatable-root--compact");
        }
        if self.sticky {
            class.push_str(" autumn-datatable-root--sticky");
        }
        let attrs = [
            ("class", class.as_str()),
            ("id", self.id),
            ("data-autumn-datatable", ""),
            ("data-autumn-busy", ""),
        ];
        let none = FieldOptions::new();
        let body = html! {
            (self.toolbar())
            @if let Some(bulk) = &self.bulk {
                form id={ (self.id) "-bulk" } method="post" action=(bulk.action)
                    data-autumn-datatable-form data-autumn-datatable-none=(NONE_SELECTED) {
                    (bulk.fields)
                    @for id in self.row_ids() {
                        input type="hidden" name="visible" value=(id);
                    }
                    (self.scroll())
                    (self.bar())
                }
                @if bulk.actions.iter().any(|a| a.confirm.is_some()) {
                    (self.dialog())
                }
            } @else {
                (self.scroll())
                (self.bar())
            }
        };
        element("div", &attrs, self.options.unwrap_or(&none), body)
    }

    // ── Parts ───────────────────────────────────────────────────────────

    fn href(&self, state: &TableState) -> String {
        state.href(self.base_path)
    }

    fn row_ids(&self) -> Vec<String> {
        match (&self.row_id, self.loading) {
            (Some(id), None) => self.rows.iter().map(id).collect(),
            _ => Vec::new(),
        }
    }

    /// The columns to show.
    fn visible(&self) -> Vec<&Column<'a, T>> {
        let defaults: Vec<_> = self.columns.iter().filter(|c| c.default_visible).collect();
        let chosen: Vec<_> = self.state.columns().map_or_else(
            || defaults.clone(),
            |keys| {
                self.columns
                    .iter()
                    .filter(|c| !c.hideable || keys.iter().any(|k| k == c.key))
                    .collect()
            },
        );
        if !chosen.is_empty() {
            chosen
        } else if !defaults.is_empty() {
            defaults
        } else {
            self.columns.iter().collect()
        }
    }

    fn toolbar(&self) -> Markup {
        let picker = self.columns.iter().any(|c| c.hideable);
        let filter_bar = self.search.is_some() || !self.filters.is_empty();
        if !(picker || filter_bar || self.density_switch) {
            return html! {};
        }
        html! {
            div class="autumn-datatable-toolbar" {
                @if filter_bar {
                    (self.filter_bar())
                    (self.chips())
                }
                @if picker {
                    (self.column_picker())
                }
                @if self.density_switch {
                    (self.density_links())
                }
            }
        }
    }

    /// Hidden inputs for each state pair that `keep` accepts.
    fn hidden(&self, keep: impl Fn(&str) -> bool) -> Markup {
        html! {
            @for (name, value) in self.state.pairs() {
                @if keep(&name) {
                    input type="hidden" name=(name) value=(value);
                }
            }
        }
    }

    fn filter_bar(&self) -> Markup {
        let id = self.id;
        let action = (!self.base_path.is_empty()).then_some(self.base_path);
        html! {
            form id={ (id) "-filters" } class="autumn-datatable-filters" method="get" action=[action]
                role="search" aria-label={ "Filter " (self.caption) } {
                @if let Some(label) = self.search {
                    div class="autumn-datatable-filter" {
                        label for={ (id) "-q" } { (label) }
                        input type="search" id={ (id) "-q" } name="q" value=(self.state.search().unwrap_or(""));
                    }
                }
                @for filter in self.filters {
                    (self.filter_control(filter))
                }
                // A filter change goes back to page 1: no `page` input.
                (self.hidden(|name| matches!(name, "sort" | "dir" | "cols" | "density" | "size")))
                div class="autumn-datatable-filter-actions" {
                    button type="submit" class="autumn-button autumn-button--secondary autumn-button--sm" {
                        "Apply"
                    }
                    @if self.state.has_filters() {
                        a class="autumn-datatable-clear" href=(self.href(&self.state.cleared())) {
                            "Clear filters"
                        }
                    }
                }
            }
        }
    }

    fn filter_control(&self, filter: &Filter<'_>) -> Markup {
        let control_id = format!("{}-filter-{}", self.id, filter.key);
        let name = format!("filter[{}]", filter.key);
        let values = self.state.filter(filter.key);
        let has = |v: &str| values.iter().any(|x| x == v);
        match filter.kind {
            FilterKind::Text => html! {
                div class="autumn-datatable-filter" {
                    label for=(control_id) { (filter.label) }
                    input type="text" id=(control_id) name=(name)
                        value=(values.first().map_or("", String::as_str));
                }
            },
            FilterKind::Select(options) => html! {
                div class="autumn-datatable-filter" {
                    label for=(control_id) { (filter.label) }
                    select id=(control_id) name=(name) {
                        option value="" { "Any" }
                        @for (value, label) in options {
                            option value=(value) selected[has(value)] { (label) }
                        }
                    }
                }
            },
            FilterKind::Multi(options) => html! {
                fieldset class="autumn-datatable-filter" {
                    legend { (filter.label) }
                    @for (index, (value, label)) in options.iter().enumerate() {
                        @let option_id = format!("{control_id}-{index}");
                        div class="autumn-datatable-option" {
                            input type="checkbox" id=(option_id) name=(name) value=(value) checked[has(value)];
                            label for=(option_id) { (label) }
                        }
                    }
                }
            },
        }
    }

    fn chips(&self) -> Markup {
        let mut chips: Vec<(String, String, String)> = Vec::new();
        if let Some(q) = self.state.search() {
            chips.push((
                format!("Search: {q}"),
                format!("Remove search {q}"),
                self.href(&self.state.with_search("")),
            ));
        }
        for filter in self.filters {
            for value in self.state.filter(filter.key) {
                let text = format!("{}: {}", filter.label, filter.value_label(value));
                chips.push((
                    text.clone(),
                    format!("Remove filter {text}"),
                    self.href(&self.state.without_filter_value(filter.key, value)),
                ));
            }
        }
        if chips.is_empty() {
            return html! {};
        }
        html! {
            ul class="autumn-datatable-chips" aria-label="Active filters" {
                @for (text, label, href) in &chips {
                    li {
                        a class="autumn-datatable-chip" href=(href) aria-label=(label) {
                            (text) " "
                            span aria-hidden="true" { "×" }
                        }
                    }
                }
            }
        }
    }

    fn column_picker(&self) -> Markup {
        let visible: BTreeSet<&str> = self.visible().iter().map(|c| c.key).collect();
        let action = (!self.base_path.is_empty()).then_some(self.base_path);
        html! {
            details class="autumn-datatable-columns" {
                summary { "Columns" }
                form method="get" action=[action] {
                    fieldset {
                        legend { "Visible columns" }
                        // The marker: no box on still gives `cols`.
                        input type="hidden" name="cols" value="";
                        @for column in self.columns.iter().filter(|c| c.hideable) {
                            @let box_id = format!("{}-cols-{}", self.id, column.key);
                            div class="autumn-datatable-option" {
                                input type="checkbox" id=(box_id) name="cols" value=(column.key)
                                    checked[visible.contains(column.key)];
                                label for=(box_id) { (column.header) }
                            }
                        }
                    }
                    (self.hidden(|name| name != "cols"))
                    button type="submit" class="autumn-button autumn-button--secondary autumn-button--sm" {
                        "Apply"
                    }
                }
            }
        }
    }

    fn density_links(&self) -> Markup {
        let now = self.state.density();
        html! {
            div class="autumn-datatable-density" role="group" aria-label="Density" {
                @for (density, label) in [(Density::Comfortable, "Comfortable"), (Density::Compact, "Compact")] {
                    @let href = self.href(&self.state.with_density(density));
                    @if density == now {
                        a href=(href) aria-current="true" { (label) }
                    } @else {
                        a href=(href) { (label) }
                    }
                }
            }
        }
    }

    fn scroll(&self) -> Markup {
        html! {
            div class="autumn-datatable-scroll" role="region"
                aria-labelledby={ (self.id) "-caption" } tabindex="0" {
                @if self.loading.is_some() {
                    p class="autumn-visually-hidden" role="status" { "Loading rows…" }
                }
                (self.table())
            }
        }
    }

    fn table(&self) -> Markup {
        let columns = self.visible();
        let busy = self.loading.map(|_| "true");
        html! {
            table id={ (self.id) "-table" } class="autumn-datatable-table" aria-busy=[busy] {
                caption id={ (self.id) "-caption" } { (self.caption) }
                thead {
                    tr {
                        @if self.row_id.is_some() {
                            th scope="col" class="autumn-datatable-select" { (self.select_all()) }
                        }
                        @for column in &columns {
                            (self.header(column))
                        }
                    }
                }
                tbody { (self.body(&columns)) }
            }
        }
    }

    fn header(&self, column: &Column<'_, T>) -> Markup {
        let Some(sort_key) = column.sort_key else {
            return html! { th scope="col" { (column.header) } };
        };
        let aria_sort = match self.state.sort() {
            Some((key, dir)) if key == sort_key => dir.aria_value(),
            _ => "none",
        };
        html! {
            th scope="col" aria-sort=(aria_sort) {
                a class="autumn-datatable-sort" href=(self.href(&self.state.sorted_by(sort_key))) { (column.header) }
            }
        }
    }

    fn select_state(&self) -> SelectAll {
        SelectAll::for_page(&self.row_ids(), self.selected)
    }

    fn select_all(&self) -> Markup {
        let state = self.select_state();
        let mixed = (state == SelectAll::Mixed).then_some("mixed");
        let disabled = self.row_ids().is_empty();
        html! {
            input type="checkbox" class="autumn-datatable-check" id={ (self.id) "-all" } name="all" value="1"
                aria-label="Select all rows" checked[state.is_checked()] aria-checked=[mixed]
                disabled[disabled] data-autumn-datatable-all;
        }
    }

    fn body(&self, columns: &[&Column<'_, T>]) -> Markup {
        let span = columns.len() + usize::from(self.row_id.is_some());
        if let Some(count) = self.loading {
            return html! {
                @for _ in 0..count {
                    tr {
                        @if self.row_id.is_some() { td class="autumn-datatable-select" {} }
                        @for _ in columns { td { (skeleton(SkeletonShape::Line)) } }
                    }
                }
            };
        }
        if self.rows.is_empty() {
            return html! { tr { td colspan=(span) { (self.empty_state()) } } };
        }
        html! {
            @for row in self.rows {
                tr { (self.row_cells(row, columns)) }
            }
        }
    }

    fn row_cells(&self, row: &T, columns: &[&Column<'_, T>]) -> Markup {
        html! {
            @if let Some(id_of) = &self.row_id {
                @let id = id_of(row);
                @let name = self.row_label.as_ref().map_or_else(|| format!("row {id}"), |label| label(row));
                td class="autumn-datatable-select" {
                    input type="checkbox" class="autumn-datatable-check" name="ids" value=(id)
                        aria-label={ "Select " (name) } checked[self.selected.contains(&id)]
                        data-autumn-datatable-row;
                }
            }
            @for column in columns {
                @if column.row_header {
                    th scope="row" { ((column.cell)(row)) }
                } @else {
                    td { ((column.cell)(row)) }
                }
            }
        }
    }

    fn empty_state(&self) -> Markup {
        if self.state.has_filters() {
            let clear = html! { a href=(self.href(&self.state.cleared())) { "Clear filters" } };
            return empty_state(
                "No rows match",
                "Change or clear the filters.",
                Some(&clear),
            );
        }
        self.empty.as_ref().map_or_else(
            || empty_state("No rows", "", None),
            |empty| empty_state(empty.title, empty.body, empty.action.as_ref()),
        )
    }

    fn bar(&self) -> Markup {
        if self.row_id.is_none() {
            return html! {};
        }
        let ids = self.row_ids();
        let selected = ids.iter().filter(|id| self.selected.contains(id)).count();
        html! {
            div class="autumn-datatable-bulk" {
                p id={ (self.id) "-status" } class="autumn-datatable-status" role="status"
                    data-autumn-datatable-status {
                    (selected) " of " (ids.len()) " selected"
                }
                @if let Some(bulk) = &self.bulk {
                    @if !bulk.actions.is_empty() {
                        label for={ (self.id) "-action" } { "With selected" }
                        select id={ (self.id) "-action" } name="action" {
                            @for action in bulk.actions {
                                option value=(action.name) data-autumn-datatable-confirm=[action.confirm] {
                                    (action.label)
                                }
                            }
                        }
                        button type="submit" class="autumn-button autumn-button--secondary autumn-button--sm" {
                            "Apply"
                        }
                    }
                }
            }
        }
    }

    fn dialog(&self) -> Markup {
        let id = self.id;
        html! {
            dialog id={ (id) "-confirm" } class="autumn-datatable-confirm"
                aria-labelledby={ (id) "-confirm-title" } {
                h2 id={ (id) "-confirm-title" } class="autumn-datatable-confirm-title" { "Confirm the action" }
                p data-autumn-datatable-confirm-text {}
                div class="autumn-datatable-confirm-actions" {
                    button type="button" class="autumn-button autumn-button--secondary"
                        data-autumn-datatable-cancel { "Cancel" }
                    button type="button" class="autumn-button autumn-button--destructive"
                        data-autumn-datatable-ok { "Confirm" }
                }
            }
        }
    }
}

impl<T> Render for DataTable<'_, T> {
    fn render(&self) -> Markup {
        Self::render(self)
    }
}
