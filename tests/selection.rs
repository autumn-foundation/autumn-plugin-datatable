//! Select-all state and the bulk form body.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on a panic"
)]

use autumn_datatable_plugin::{BulkAction, BulkSelection, SelectAll};

#[test]
fn select_all_has_three_states() {
    assert_eq!(SelectAll::from_counts(0, 3), SelectAll::None);
    assert_eq!(SelectAll::from_counts(1, 3), SelectAll::Mixed);
    assert_eq!(SelectAll::from_counts(2, 3), SelectAll::Mixed);
    assert_eq!(SelectAll::from_counts(3, 3), SelectAll::All);
}

#[test]
fn select_all_with_no_rows_is_none() {
    assert_eq!(SelectAll::from_counts(0, 0), SelectAll::None);
}

#[test]
fn select_all_clamps_a_count_above_the_total() {
    assert_eq!(SelectAll::from_counts(9, 3), SelectAll::All);
}

#[test]
fn select_all_gives_the_aria_and_native_state() {
    assert_eq!(SelectAll::None.aria_checked(), "false");
    assert_eq!(SelectAll::Mixed.aria_checked(), "mixed");
    assert_eq!(SelectAll::All.aria_checked(), "true");
    assert!(SelectAll::All.is_checked());
    assert!(!SelectAll::Mixed.is_checked());
    assert!(!SelectAll::None.is_checked());
}

#[test]
fn select_all_counts_only_the_rows_on_the_page() {
    let page = ["1", "2", "3"];
    assert_eq!(SelectAll::for_page(&page, &["2", "9"]), SelectAll::Mixed);
    assert_eq!(SelectAll::for_page(&page, &["9"]), SelectAll::None);
    assert_eq!(SelectAll::for_page(&page, &["3", "1", "2"]), SelectAll::All);
    assert_eq!(
        SelectAll::for_page(&page, &["1", "1", "1"]),
        SelectAll::Mixed
    );
}

#[test]
fn a_bulk_body_gives_the_action_and_ids() {
    let bulk = BulkSelection::from_form(b"action=delete&ids=1&ids=2&visible=1&visible=2&visible=3")
        .expect("decodes");
    assert_eq!(bulk.action(), "delete");
    assert_eq!(bulk.ids(), ["1", "2"]);
    assert!(!bulk.is_all());
    assert!(!bulk.is_empty());
}

#[test]
fn select_all_without_javascript_gives_the_page_ids() {
    let bulk = BulkSelection::from_form(b"action=delete&all=1&visible=1&visible=2&visible=3")
        .expect("decodes");
    assert!(bulk.is_all());
    assert_eq!(bulk.ids(), ["1", "2", "3"]);
}

#[test]
fn ids_are_unique_and_not_empty() {
    let bulk = BulkSelection::from_form(b"action=x&ids=2&ids=&ids=2&ids=1").expect("decodes");
    assert_eq!(bulk.ids(), ["2", "1"]);
}

#[test]
fn an_empty_body_selects_nothing() {
    let bulk = BulkSelection::from_form(b"").expect("decodes");
    assert!(bulk.is_empty());
    assert_eq!(bulk.action(), "");
    assert_eq!(bulk.ids(), [] as [&str; 0]);
}

#[test]
fn a_duplicate_action_is_an_error() {
    assert!(BulkSelection::from_form(b"action=a&action=b").is_err());
}

#[test]
fn the_action_must_be_declared() {
    let actions = [
        BulkAction::new("archive", "Archive"),
        BulkAction::new("delete", "Delete").confirm("Delete the rows?"),
    ];
    let bulk = BulkSelection::from_form(b"action=delete&ids=1").expect("decodes");
    let action = bulk.action_in(&actions).expect("declared");
    assert_eq!(action.name(), "delete");
    assert_eq!(action.label(), "Delete");
    assert_eq!(action.confirm_text(), Some("Delete the rows?"));
    let bulk = BulkSelection::from_form(b"action=drop_db&ids=1").expect("decodes");
    assert!(bulk.action_in(&actions).is_none());
}

#[test]
fn the_selection_decodes_as_a_serde_type() {
    let bulk: BulkSelection =
        autumn_web::form::from_urlencoded(b"action=a&ids=7").expect("decodes");
    assert_eq!(bulk.ids(), ["7"]);
}
