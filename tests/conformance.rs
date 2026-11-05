//! The pack passes the static widget conformance checks.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on a panic"
)]

use autumn_datatable_plugin::{ASSETS, DatatablePlugin, pack};
use autumn_web::plugin::Plugin;
use autumn_web::plugin_conformance::widgets::run_widget_conformance;

#[test]
fn the_pack_conforms() {
    let report = run_widget_conformance(&pack());
    assert!(report.passed(), "{}", report.to_text_report());
}

#[test]
fn the_pack_has_a_story_for_each_feature() {
    let pack = pack();
    let slugs: Vec<&str> = pack
        .stories()
        .iter()
        .map(autumn_web::stories::Story::slug)
        .collect();
    for slug in [
        "datatable-sorting",
        "datatable-selection",
        "datatable-filters",
        "datatable-columns-and-density",
        "datatable-sticky-header",
        "datatable-empty",
        "datatable-no-match",
        "datatable-loading",
        "datatable-bulk-actions",
    ] {
        assert!(slugs.contains(&slug), "no story `{slug}` in {slugs:?}");
    }
}

#[test]
fn the_pack_links_its_css_and_js() {
    let pack = pack();
    assert_eq!(pack.name(), "datatable");
    assert_eq!(ASSETS.namespace(), "datatable");
    let head = pack.head().into_string();
    assert!(head.contains("datatable"), "{head}");
    assert!(head.contains("<link"));
    assert!(head.contains("<script"));
    assert!(head.contains("integrity="));
}

#[test]
fn the_plugin_has_a_name() {
    assert_eq!(DatatablePlugin.name(), "autumn-datatable-plugin");
}

#[test]
fn the_plugin_installs_on_an_app() {
    let _app = autumn_web::app().plugin(DatatablePlugin);
}
