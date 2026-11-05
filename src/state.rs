//! The table state in the query string.

use std::collections::BTreeMap;
use std::fmt;

use autumn_web::pagination::{DEFAULT_PAGE_SIZE, ListQuery, MAX_PAGE_SIZE, PageRequest, SortDir};
use autumn_web::query_string::QueryError;
use serde::Deserialize;

use crate::table::{Column, Filter, FilterKind};

/// The longest search or text filter value that [`TableState::checked`]
/// keeps, in characters.
pub const MAX_TEXT_LEN: usize = 200;

/// The row height of the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    /// Normal row height. The default.
    #[default]
    Comfortable,
    /// Less space in each cell.
    Compact,
}

impl Density {
    /// The `density=` value.
    #[must_use]
    pub const fn param_value(self) -> &'static str {
        match self {
            Self::Comfortable => "comfortable",
            Self::Compact => "compact",
        }
    }

    /// Parse a `density=` value. An unknown value is
    /// [`Density::Comfortable`].
    #[must_use]
    pub const fn from_param(value: &str) -> Self {
        if value.eq_ignore_ascii_case("compact") {
            Self::Compact
        } else {
            Self::Comfortable
        }
    }
}

/// The state of a table: search, sort, page, filters, columns and density.
///
/// It decodes from the query string and encodes back to the same string.
///
/// | Parameter | Meaning |
/// |---|---|
/// | `q` | The search text. |
/// | `sort`, `dir` | The sort key, and `asc` or `desc`. |
/// | `page`, `size` | The page (from 1) and the page size, as [`PageRequest`]. |
/// | `filter[<key>]` | A filter value. Repeat the key for more values. |
/// | `cols` | The optional columns to show. `cols=` with no value is a marker. |
/// | `density` | `comfortable` or `compact`. |
///
/// A bad `dir`, `page`, `size` or `density` value gives the default. Two
/// values for one scalar (`q=a&q=b`) are an error. An empty value is
/// absent. Unknown parameters are ignored.
///
/// The state holds what the client sent. Call [`checked`](Self::checked)
/// before you use it in a query.
///
/// ```rust
/// use autumn_datatable_plugin::TableState;
///
/// let state = TableState::from_query("sort=title&filter[status]=open&page=3").unwrap();
/// let next = state.sorted_by("title");
/// assert_eq!(next.to_query(), "filter[status]=open&sort=title&dir=desc");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableState {
    search: Option<String>,
    sort: Option<(String, SortDir)>,
    page: u32,
    size: Option<u32>,
    filters: BTreeMap<String, Vec<String>>,
    columns: Option<Vec<String>>,
    density: Density,
}

/// The wire shape. Every value is text, so a bad value never fails the
/// decode. `Vec` takes the repeated keys.
#[derive(Deserialize)]
struct Raw {
    q: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    page: Option<String>,
    size: Option<String>,
    #[serde(default)]
    filter: BTreeMap<String, Vec<String>>,
    cols: Option<Vec<String>>,
    density: Option<String>,
}

impl From<Raw> for TableState {
    fn from(raw: Raw) -> Self {
        let mut state = Self::new();
        if let Some(q) = raw.q {
            state = state.with_search(&q);
        }
        if let Some(sort) = raw.sort.filter(|s| !s.is_empty()) {
            let dir = raw.dir.as_deref().map_or(SortDir::Asc, SortDir::from_param);
            state.sort = Some((sort, dir));
        }
        for (key, values) in raw.filter {
            state = state.with_filter(&key, values.iter().map(String::as_str));
        }
        if let Some(cols) = raw.cols {
            state = state.with_columns(cols.iter().map(String::as_str));
        }
        state.page = parse_positive(raw.page.as_deref()).unwrap_or(1);
        state.size = parse_positive(raw.size.as_deref());
        state.density = raw
            .density
            .as_deref()
            .map_or_else(Density::default, Density::from_param);
        state
    }
}

impl<'de> Deserialize<'de> for TableState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Raw::deserialize(deserializer).map(Self::from)
    }
}

fn parse_positive(value: Option<&str>) -> Option<u32> {
    value.and_then(|v| v.parse::<u32>().ok()).filter(|n| *n > 0)
}

impl TableState {
    /// The default state: page 1, no search, no sort, no filters.
    #[must_use]
    pub fn new() -> Self {
        Self {
            page: 1,
            ..Self::default()
        }
    }

    /// Decode a raw query string, with or without the `?`.
    ///
    /// # Errors
    ///
    /// [`QueryError`] when a scalar parameter occurs two times, or a key is
    /// both a scalar and a container (`filter=x&filter[a]=y`).
    pub fn from_query(query: &str) -> Result<Self, QueryError> {
        let query = query.strip_prefix('?').unwrap_or(query);
        autumn_web::query_string::from_query_str(query)
    }

    /// The search text.
    #[must_use]
    pub fn search(&self) -> Option<&str> {
        self.search.as_deref()
    }

    /// The sort key and direction.
    #[must_use]
    pub fn sort(&self) -> Option<(&str, SortDir)> {
        self.sort.as_ref().map(|(key, dir)| (key.as_str(), *dir))
    }

    /// The page, from 1.
    #[must_use]
    pub const fn page(&self) -> u32 {
        if self.page == 0 { 1 } else { self.page }
    }

    /// The page size that the client asked for.
    #[must_use]
    pub const fn size(&self) -> Option<u32> {
        self.size
    }

    /// Each filter key and its values.
    #[must_use]
    pub const fn filters(&self) -> &BTreeMap<String, Vec<String>> {
        &self.filters
    }

    /// The values of one filter. Empty when the filter is not set.
    #[must_use]
    pub fn filter(&self, key: &str) -> &[String] {
        self.filters.get(key).map_or(&[], Vec::as_slice)
    }

    /// The optional columns to show. `None` means the defaults.
    #[must_use]
    pub fn columns(&self) -> Option<&[String]> {
        self.columns.as_deref()
    }

    /// The density.
    #[must_use]
    pub const fn density(&self) -> Density {
        self.density
    }

    /// `true` when a search or a filter is set.
    #[must_use]
    pub fn has_filters(&self) -> bool {
        self.search.is_some() || !self.filters.is_empty()
    }

    // ── Changes ─────────────────────────────────────────────────────────

    /// Set the search text. An empty text removes it. Resets the page.
    #[must_use]
    pub fn with_search(&self, q: &str) -> Self {
        let mut next = self.first_page();
        next.search = (!q.is_empty()).then(|| q.to_owned());
        next
    }

    /// Sort by `key`. If `key` is the sort now, flip the direction. Else sort
    /// ascending. Resets the page.
    #[must_use]
    pub fn sorted_by(&self, key: &str) -> Self {
        let dir = match self.sort() {
            Some((now, dir)) if now == key => dir.toggled(),
            _ => SortDir::Asc,
        };
        self.with_sort(key, dir)
    }

    /// Sort by `key` in `dir`. An empty key removes the sort. Resets the
    /// page.
    #[must_use]
    pub fn with_sort(&self, key: &str, dir: SortDir) -> Self {
        let mut next = self.first_page();
        next.sort = (!key.is_empty()).then(|| (key.to_owned(), dir));
        next
    }

    /// Go to `page`. Page 0 is page 1.
    #[must_use]
    pub fn with_page(&self, page: u32) -> Self {
        let mut next = self.clone();
        next.page = page.max(1);
        next
    }

    /// Set the page size. 0 removes it. Resets the page.
    #[must_use]
    pub fn with_size(&self, size: u32) -> Self {
        let mut next = self.first_page();
        next.size = (size > 0).then_some(size);
        next
    }

    /// Set the values of one filter. Empty values are dropped and a value
    /// is kept one time. No values removes the filter. Resets the page.
    #[must_use]
    pub fn with_filter<'v>(&self, key: &str, values: impl IntoIterator<Item = &'v str>) -> Self {
        let mut next = self.first_page();
        let mut kept: Vec<String> = Vec::new();
        for value in values {
            if !value.is_empty() && !kept.iter().any(|v| v == value) {
                kept.push(value.to_owned());
            }
        }
        if key.is_empty() || kept.is_empty() {
            next.filters.remove(key);
        } else {
            next.filters.insert(key.to_owned(), kept);
        }
        next
    }

    /// Add one value to a filter. Resets the page.
    #[must_use]
    pub fn with_filter_value(&self, key: &str, value: &str) -> Self {
        let values = self.filter(key).iter().map(String::as_str).chain([value]);
        self.with_filter(key, values)
    }

    /// Remove one value from a filter. Resets the page.
    #[must_use]
    pub fn without_filter_value(&self, key: &str, value: &str) -> Self {
        let values = self
            .filter(key)
            .iter()
            .map(String::as_str)
            .filter(|v| *v != value);
        self.with_filter(key, values)
    }

    /// Remove a filter. Resets the page.
    #[must_use]
    pub fn without_filter(&self, key: &str) -> Self {
        self.with_filter(key, [])
    }

    /// Remove the search and each filter. Keeps the sort, the columns and
    /// the density. Resets the page.
    #[must_use]
    pub fn cleared(&self) -> Self {
        let mut next = self.first_page();
        next.search = None;
        next.filters.clear();
        next
    }

    /// Show these optional columns. Empty keys are dropped. Keeps the page.
    #[must_use]
    pub fn with_columns<'k>(&self, keys: impl IntoIterator<Item = &'k str>) -> Self {
        let mut next = self.clone();
        let mut kept: Vec<String> = Vec::new();
        for key in keys {
            if !key.is_empty() && !kept.iter().any(|k| k == key) {
                kept.push(key.to_owned());
            }
        }
        next.columns = Some(kept);
        next
    }

    /// Show the default columns. Keeps the page.
    #[must_use]
    pub fn default_columns(&self) -> Self {
        let mut next = self.clone();
        next.columns = None;
        next
    }

    /// Set the density. Keeps the page.
    #[must_use]
    pub fn with_density(&self, density: Density) -> Self {
        let mut next = self.clone();
        next.density = density;
        next
    }

    fn first_page(&self) -> Self {
        let mut next = self.clone();
        next.page = 1;
        next
    }

    // ── Safe default ────────────────────────────────────────────────────

    /// Keep only what the table declares. Use this state for the query.
    ///
    /// - A sort key that no sortable column has is dropped.
    /// - A filter key that `filters` does not declare is dropped.
    /// - A select or checkbox filter keeps only its option values. A select
    ///   keeps one value.
    /// - A text filter and the search are trimmed and cut to
    ///   [`MAX_TEXT_LEN`] characters. A text filter keeps one value.
    /// - Only hideable column keys stay in `cols`.
    /// - `size` is clamped to [`MAX_PAGE_SIZE`].
    #[must_use]
    pub fn checked<T>(&self, columns: &[Column<'_, T>], filters: &[Filter<'_>]) -> Self {
        let mut next = self.clone();
        next.search = self
            .search
            .as_deref()
            .map(clean_text)
            .filter(|q| !q.is_empty());
        next.sort = self.sort.clone().filter(|(key, _)| {
            columns
                .iter()
                .any(|c| c.sort_key_str() == Some(key.as_str()))
        });
        next.filters = BTreeMap::new();
        for filter in filters {
            let values = self.filter(filter.key());
            let kept: Vec<String> = match filter.kind() {
                FilterKind::Text => values
                    .iter()
                    .map(|v| clean_text(v))
                    .filter(|v| !v.is_empty())
                    .take(1)
                    .collect(),
                FilterKind::Select(options) => values
                    .iter()
                    .filter(|v| has_option(options, v))
                    .take(1)
                    .cloned()
                    .collect(),
                FilterKind::Multi(options) => values
                    .iter()
                    .filter(|v| has_option(options, v))
                    .cloned()
                    .collect(),
            };
            if !kept.is_empty() {
                next.filters.insert(filter.key().to_owned(), kept);
            }
        }
        next.columns = self.columns.as_ref().map(|keys| {
            keys.iter()
                .filter(|key| {
                    columns
                        .iter()
                        .any(|c| c.is_hideable() && c.key() == key.as_str())
                })
                .cloned()
                .collect()
        });
        next.size = self.size.map(|s| s.min(MAX_PAGE_SIZE));
        next
    }

    // ── Core interop ────────────────────────────────────────────────────

    /// The page and size as a [`PageRequest`]. No size gives
    /// [`DEFAULT_PAGE_SIZE`].
    #[must_use]
    pub const fn page_request(&self) -> PageRequest {
        let size = match self.size {
            Some(size) => size,
            None => DEFAULT_PAGE_SIZE,
        };
        PageRequest::new(self.page(), size)
    }

    /// The sort and the filters as a [`ListQuery`], one pair for each
    /// filter value. Call [`checked`](Self::checked) first.
    #[must_use]
    pub fn list_query(&self) -> ListQuery {
        let pairs: Vec<(&str, &str)> = self
            .filters
            .iter()
            .flat_map(|(key, values)| values.iter().map(move |v| (key.as_str(), v.as_str())))
            .collect();
        match self.sort() {
            Some((key, dir)) => ListQuery::new(Some(key), dir, &pairs),
            None => ListQuery::new(None, SortDir::Asc, &pairs),
        }
    }

    // ── Encode ──────────────────────────────────────────────────────────

    /// The query string, without `?`. The order is fixed. Default values
    /// are not written.
    #[must_use]
    pub fn to_query(&self) -> String {
        self.pairs()
            .iter()
            .map(|(name, value)| format!("{}={}", encode_name(name), encode(value)))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// `base` with the query string, or `base` when the query is empty.
    #[must_use]
    pub fn href(&self, base: &str) -> String {
        let query = self.to_query();
        if query.is_empty() {
            base.to_owned()
        } else {
            format!("{base}?{query}")
        }
    }

    /// Each `(name, value)` pair, not encoded, in the order of
    /// [`to_query`](Self::to_query). A form writes these as inputs.
    pub(crate) fn pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(q) = &self.search {
            pairs.push(("q".to_owned(), q.clone()));
        }
        for (key, values) in &self.filters {
            let name = format!("filter[{key}]");
            pairs.extend(values.iter().map(|v| (name.clone(), v.clone())));
        }
        match &self.columns {
            Some(keys) if keys.is_empty() => pairs.push(("cols".to_owned(), String::new())),
            Some(keys) => pairs.extend(keys.iter().map(|k| ("cols".to_owned(), k.clone()))),
            None => {}
        }
        if let Some((key, dir)) = &self.sort {
            pairs.push(("sort".to_owned(), key.clone()));
            pairs.push(("dir".to_owned(), dir.param_value().to_owned()));
        }
        if self.page() > 1 {
            pairs.push(("page".to_owned(), self.page().to_string()));
        }
        if let Some(size) = self.size {
            pairs.push(("size".to_owned(), size.to_string()));
        }
        if self.density != Density::Comfortable {
            pairs.push(("density".to_owned(), self.density.param_value().to_owned()));
        }
        pairs
    }
}

impl fmt::Display for TableState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_query())
    }
}

fn has_option(options: &[(&str, &str)], value: &str) -> bool {
    options.iter().any(|(v, _)| *v == value)
}

fn clean_text(value: &str) -> String {
    value.trim().chars().take(MAX_TEXT_LEN).collect()
}

/// Encode a parameter name. The brackets of `filter[<key>]` stay.
fn encode_name(name: &str) -> String {
    name.split_inclusive(['[', ']'])
        .map(|part| {
            part.strip_suffix(['[', ']']).map_or_else(
                || encode(part),
                |text| format!("{}{}", encode(text), &part[text.len()..]),
            )
        })
        .collect()
}

/// Percent-encode all but the unreserved characters (RFC 3986).
fn encode(value: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}
