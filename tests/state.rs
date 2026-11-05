//! `TableState`: decode, encode, links and `checked`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on a panic"
)]

use autumn_datatable_plugin::{Column, Density, Filter, SortDir, TableState};
use autumn_web::pagination::DEFAULT_PAGE_SIZE;
use maud::html;
use proptest::prelude::*;

struct Post;

fn columns() -> Vec<Column<'static, Post>> {
    vec![
        Column::new("title", "Title", |_: &Post| html! {}).sortable(),
        Column::new("status", "Status", |_: &Post| html! {})
            .sortable()
            .hideable(),
        Column::new("author", "Author", |_: &Post| html! {}).hidden(),
        Column::new("id", "ID", |_: &Post| html! {}),
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

fn parse(query: &str) -> TableState {
    TableState::from_query(query).expect("query decodes")
}

// ── Decode ──────────────────────────────────────────────────────────────

#[test]
fn an_empty_query_is_the_default_state() {
    let state = parse("");
    assert_eq!(state, TableState::new());
    assert_eq!(state.search(), None);
    assert_eq!(state.sort(), None);
    assert_eq!(state.page(), 1);
    assert_eq!(state.size(), None);
    assert!(state.filters().is_empty());
    assert_eq!(state.columns(), None);
    assert_eq!(state.density(), Density::Comfortable);
    assert!(!state.has_filters());
}

#[test]
fn a_full_query_decodes_each_part() {
    let state = parse(
        "q=rust&sort=title&dir=desc&page=3&size=50&filter[status]=open\
         &filter[status]=closed&filter[kind]=bug&cols=status&cols=author&density=compact",
    );
    assert_eq!(state.search(), Some("rust"));
    assert_eq!(state.sort(), Some(("title", SortDir::Desc)));
    assert_eq!(state.page(), 3);
    assert_eq!(state.size(), Some(50));
    assert_eq!(state.filter("status"), ["open", "closed"]);
    assert_eq!(state.filter("kind"), ["bug"]);
    assert_eq!(state.filter("nope"), [] as [String; 0]);
    assert_eq!(
        state.columns(),
        Some(&["status".to_owned(), "author".to_owned()][..])
    );
    assert_eq!(state.density(), Density::Compact);
    assert!(state.has_filters());
}

#[test]
fn percent_encoded_brackets_decode_as_brackets() {
    let state = parse("filter%5Bstatus%5D=open");
    assert_eq!(state.filter("status"), ["open"]);
}

#[test]
fn the_cols_marker_alone_means_no_optional_column() {
    let state = parse("cols=");
    assert_eq!(state.columns(), Some(&[][..]));
    assert_eq!(state.to_query(), "cols=");
}

#[test]
fn bad_values_fall_back_to_the_default() {
    let state = parse("dir=sideways&sort=title&page=abc&size=x&density=huge");
    assert_eq!(state.sort(), Some(("title", SortDir::Asc)));
    assert_eq!(state.page(), 1);
    assert_eq!(state.size(), None);
    assert_eq!(state.density(), Density::Comfortable);
    assert_eq!(parse("page=0").page(), 1);
    assert_eq!(parse("size=0").size(), None);
}

#[test]
fn a_direction_without_a_sort_is_ignored() {
    assert_eq!(parse("dir=desc"), TableState::new());
}

#[test]
fn empty_values_are_absent() {
    let state = parse("q=&sort=&filter[status]=&filter[kind]=bug&filter[kind]=");
    assert_eq!(state.search(), None);
    assert_eq!(state.sort(), None);
    assert_eq!(state.filter("status"), [] as [String; 0]);
    assert_eq!(state.filter("kind"), ["bug"]);
    assert_eq!(state.filters().len(), 1);
}

#[test]
fn a_repeated_value_of_one_filter_is_kept_once() {
    assert_eq!(
        parse("filter[status]=open&filter[status]=open").filter("status"),
        ["open"]
    );
}

#[test]
fn a_duplicate_scalar_is_an_error() {
    assert!(TableState::from_query("q=a&q=b").is_err());
    assert!(TableState::from_query("sort=a&sort=b").is_err());
}

#[test]
fn unknown_parameters_are_ignored() {
    assert_eq!(parse("utm_source=mail&x[y]=z"), TableState::new());
}

#[test]
fn the_state_decodes_through_the_autumn_query_decoder() {
    let state: TableState =
        autumn_web::query_string::from_query_str("sort=title&filter[status]=open")
            .expect("decodes");
    assert_eq!(state.sort(), Some(("title", SortDir::Asc)));
    assert_eq!(state.filter("status"), ["open"]);
}

// ── Encode ──────────────────────────────────────────────────────────────

#[test]
fn the_default_state_encodes_to_nothing() {
    assert_eq!(TableState::new().to_query(), "");
    assert_eq!(TableState::new().href("/posts"), "/posts");
    // An empty `href` keeps the current query. `?` clears it.
    assert_eq!(TableState::new().href(""), "?");
    assert_eq!(parse("page=2").href(""), "?page=2");
}

#[test]
fn default_is_new() {
    assert_eq!(TableState::default(), TableState::new());
    assert_eq!(TableState::default().page(), 1);
    assert_eq!(
        parse(&TableState::default().to_query()),
        TableState::default()
    );
}

#[test]
fn a_key_that_cannot_round_trip_is_dropped() {
    for key in ["a]b", "a[b", "", "A", "a-b", "a b"] {
        assert!(
            TableState::new()
                .with_filter(key, ["v"])
                .filters()
                .is_empty(),
            "{key:?}"
        );
        assert_eq!(
            TableState::new().with_columns([key]).columns(),
            Some(&[][..]),
            "{key:?}"
        );
    }
    assert_eq!(
        parse("cols=Bad&cols=ok").columns(),
        Some(&["ok".to_owned()][..])
    );
}

#[test]
fn the_decoder_caps_filters_values_and_columns() {
    use autumn_datatable_plugin::{MAX_COLUMNS, MAX_FILTER_VALUES, MAX_FILTERS};
    let many_keys: String = (0..500)
        .map(|i| format!("filter[k{i}]=1"))
        .collect::<Vec<_>>()
        .join("&");
    assert_eq!(parse(&many_keys).filters().len(), MAX_FILTERS);
    let many_values: String = (0..500)
        .map(|i| format!("filter[k]=v{i}"))
        .collect::<Vec<_>>()
        .join("&");
    assert_eq!(parse(&many_values).filter("k").len(), MAX_FILTER_VALUES);
    let many_cols: String = (0..500)
        .map(|i| format!("cols=c{i}"))
        .collect::<Vec<_>>()
        .join("&");
    assert_eq!(
        parse(&many_cols).columns().map(<[String]>::len),
        Some(MAX_COLUMNS)
    );
}

#[test]
fn the_encoder_writes_a_fixed_order_and_omits_defaults() {
    let state =
        parse("density=compact&cols=status&filter[status]=open&page=2&dir=desc&sort=title&q=a");
    assert_eq!(
        state.to_query(),
        "q=a&filter[status]=open&cols=status&sort=title&dir=desc&page=2&density=compact"
    );
    assert_eq!(
        state.href("/posts"),
        "/posts?q=a&filter[status]=open&cols=status&sort=title&dir=desc&page=2&density=compact"
    );
    assert_eq!(state.to_string(), state.to_query());
}

#[test]
fn the_encoder_percent_encodes_values() {
    let state = TableState::new()
        .with_search("a&b=c d+e%\"><script>")
        .with_filter("author", ["Zoë [x]"]);
    let query = state.to_query();
    assert_eq!(
        query,
        "q=a%26b%3Dc%20d%2Be%25%22%3E%3Cscript%3E&filter[author]=Zo%C3%AB%20%5Bx%5D"
    );
    assert_eq!(parse(&query), state);
}

// ── Links ───────────────────────────────────────────────────────────────

#[test]
fn sorted_by_toggles_the_direction_and_resets_the_page() {
    let state = parse("sort=title&page=4&filter[status]=open&density=compact");
    let flipped = state.sorted_by("title");
    assert_eq!(flipped.sort(), Some(("title", SortDir::Desc)));
    assert_eq!(flipped.page(), 1);
    assert_eq!(flipped.filter("status"), ["open"]);
    assert_eq!(flipped.density(), Density::Compact);
    assert_eq!(
        flipped.sorted_by("title").sort(),
        Some(("title", SortDir::Asc))
    );
    let other = flipped.sorted_by("status");
    assert_eq!(other.sort(), Some(("status", SortDir::Asc)));
}

#[test]
fn a_filter_change_resets_the_page_and_keeps_the_rest() {
    let state = parse("sort=title&dir=desc&page=7&cols=status&density=compact&size=50");
    for next in [
        state.with_filter("status", ["open"]),
        state.with_filter_value("status", "closed"),
        state.with_search("x"),
        state.with_size(10),
    ] {
        assert_eq!(next.page(), 1);
        assert_eq!(next.sort(), Some(("title", SortDir::Desc)));
        assert_eq!(next.columns(), Some(&["status".to_owned()][..]));
        assert_eq!(next.density(), Density::Compact);
    }
}

#[test]
fn filter_values_add_and_remove() {
    let state = TableState::new()
        .with_filter_value("status", "open")
        .with_filter_value("status", "closed")
        .with_filter_value("status", "open");
    assert_eq!(state.filter("status"), ["open", "closed"]);
    let less = state.without_filter_value("status", "open");
    assert_eq!(less.filter("status"), ["closed"]);
    let none = less.without_filter_value("status", "closed");
    assert!(none.filters().is_empty());
    assert!(state.without_filter("status").filters().is_empty());
}

#[test]
fn cleared_drops_the_search_and_filters_only() {
    let state = parse("q=a&filter[status]=open&sort=title&cols=status&density=compact&page=3");
    let cleared = state.cleared();
    assert_eq!(cleared.search(), None);
    assert!(cleared.filters().is_empty());
    assert_eq!(cleared.page(), 1);
    assert_eq!(cleared.sort(), Some(("title", SortDir::Asc)));
    assert_eq!(cleared.columns(), Some(&["status".to_owned()][..]));
    assert_eq!(cleared.density(), Density::Compact);
}

#[test]
fn density_and_columns_keep_the_page() {
    let state = parse("page=3");
    assert_eq!(state.with_density(Density::Compact).page(), 3);
    assert_eq!(state.with_columns(["status"]).page(), 3);
    assert_eq!(
        state.with_columns(["status"]).default_columns().columns(),
        None
    );
}

#[test]
fn with_page_sets_the_page() {
    assert_eq!(TableState::new().with_page(5).page(), 5);
    assert_eq!(TableState::new().with_page(0).page(), 1);
}

// ── checked ─────────────────────────────────────────────────────────────

#[test]
fn checked_drops_what_the_table_does_not_declare() {
    let state = parse(
        "sort=password&filter[status]=open&filter[status]=evil&filter[secret]=1\
         &filter[kind]=idea&filter[kind]=bug&cols=status&cols=id&cols=nope&size=5000",
    )
    .checked(&columns(), &filters());
    assert_eq!(state.sort(), None);
    assert_eq!(state.filter("status"), ["open"]);
    assert_eq!(state.filter("secret"), [] as [String; 0]);
    // A select filter takes one value: the first valid one.
    assert_eq!(state.filter("kind"), ["idea"]);
    // `id` is not hideable, so it is not a choice.
    assert_eq!(state.columns(), Some(&["status".to_owned()][..]));
    assert_eq!(state.size(), Some(autumn_web::pagination::MAX_PAGE_SIZE));
}

#[test]
fn checked_keeps_a_declared_sort_and_a_text_filter() {
    let state = parse("sort=status&dir=desc&filter[author]=%20ann%20&q=%20hi%20")
        .checked(&columns(), &filters());
    assert_eq!(state.sort(), Some(("status", SortDir::Desc)));
    assert_eq!(state.filter("author"), ["ann"]);
    assert_eq!(state.search(), Some("hi"));
}

#[test]
fn checked_limits_the_length_of_text() {
    let long = "a".repeat(500);
    let state = TableState::new()
        .with_search(&long)
        .with_filter("author", [long.as_str()])
        .checked(&columns(), &filters());
    assert_eq!(state.search().map(str::len), Some(200));
    assert_eq!(state.filter("author")[0].len(), 200);
}

#[test]
fn a_sort_key_can_differ_from_the_column_key() {
    let cols = vec![Column::new("name", "Name", |_: &Post| html! {}).sort_key("name_lower")];
    let state = parse("sort=name_lower").checked(&cols, &[]);
    assert_eq!(state.sort(), Some(("name_lower", SortDir::Asc)));
    assert_eq!(parse("sort=name").checked(&cols, &[]).sort(), None);
}

// ── Core interop ────────────────────────────────────────────────────────

#[test]
fn the_state_gives_a_page_request() {
    let req = parse("page=3&size=10").page_request();
    assert_eq!((req.page(), req.size()), (3, 10));
    let req = TableState::new().page_request();
    assert_eq!((req.page(), req.size()), (1, DEFAULT_PAGE_SIZE));
}

#[test]
fn the_state_gives_a_list_query() {
    let query = parse("sort=title&dir=desc&filter[kind]=bug").list_query();
    assert_eq!(query.sort(), Some("title"));
    assert_eq!(query.direction(), SortDir::Desc);
    let pairs: Vec<(&str, &str)> = query.filters().collect();
    assert_eq!(pairs, [("kind", "bug")]);
}

#[test]
fn a_multi_value_filter_is_not_in_the_list_query() {
    // The repository joins list filters with AND: two values of one
    // column match no row.
    let state = parse("filter[status]=open&filter[status]=closed&filter[kind]=bug");
    let query = state.list_query();
    let pairs: Vec<(&str, &str)> = query.filters().collect();
    assert_eq!(pairs, [("kind", "bug")]);
    assert_eq!(state.filter("status"), ["open", "closed"]);
}

// ── Round trip ──────────────────────────────────────────────────────────

fn text() -> impl Strategy<Value = String> {
    // Any text, also the characters that the query syntax uses.
    proptest::string::string_regex("[ -~é中\\[\\]&=+%#]{1,12}").expect("regex")
}

fn key() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-z0-9_]{1,9}").expect("regex")
}

prop_compose! {
    fn any_state()(
        search in proptest::option::of(text()),
        sort in proptest::option::of((key(), any::<bool>())),
        page in 1u32..1000,
        size in proptest::option::of(1u32..500),
        filters in proptest::collection::btree_map(key(), proptest::collection::vec(text(), 1..4), 0..4),
        columns in proptest::option::of(proptest::collection::vec(key(), 0..4)),
        compact in any::<bool>(),
    ) -> TableState {
        let mut state = TableState::new();
        if let Some(search) = search {
            state = state.with_search(&search);
        }
        for (key, values) in &filters {
            state = state.with_filter(key, values.iter().map(String::as_str));
        }
        if let Some(columns) = &columns {
            state = state.with_columns(columns.iter().map(String::as_str));
        }
        if let Some((key, desc)) = sort {
            state = state.with_sort(&key, if desc { SortDir::Desc } else { SortDir::Asc });
        }
        if let Some(size) = size {
            state = state.with_size(size);
        }
        if compact {
            state = state.with_density(Density::Compact);
        }
        state.with_page(page)
    }
}

proptest! {
    #[test]
    fn encode_then_decode_gives_the_same_state(state in any_state()) {
        let query = state.to_query();
        let back = TableState::from_query(&query).expect("own output decodes");
        prop_assert_eq!(&back, &state, "query: {}", query);
        prop_assert_eq!(back.to_query(), query);
    }

    #[test]
    fn each_link_round_trips(state in any_state(), key in key(), value in text()) {
        for next in [
            state.sorted_by(&key),
            state.with_sort(&key, SortDir::Desc),
            state.with_search(&value),
            state.with_filter_value(&key, &value),
            state.without_filter_value(&key, &value),
            state.without_filter(&key),
            state.cleared(),
            state.with_size(7),
            state.with_page(3),
            state.with_columns([key.as_str()]),
            state.with_density(Density::Comfortable),
            state.default_columns(),
        ] {
            let back = TableState::from_query(&next.to_query()).expect("decodes");
            prop_assert_eq!(back, next);
        }
    }

    #[test]
    fn decode_never_panics(query in "[ -~]{0,60}") {
        let _ = TableState::from_query(&query);
    }
}
