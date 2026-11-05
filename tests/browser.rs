//! Browser tests: the plugin conformance gate (axe, console, keyboard) and
//! the selection keyboard model.
//!
//! Run:
//!
//! ```text
//! AUTUMN_CHROMIUM=/path/to/chrome \
//!   cargo test --features system-tests --test browser -- --include-ignored
//! ```

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on a panic"
)]
#![cfg(feature = "system-tests")]

use std::time::Duration;

use autumn_datatable_plugin::{ASSETS, BulkAction, BulkSelection, Column, DataTable, pack};
use autumn_web::plugin_conformance::widgets::browser::run_widget_browser_conformance;
use autumn_web::plugin_conformance::widgets::run_widget_conformance;
use autumn_web::prelude::*;
use autumn_web::system_test::{Page, SystemTest, SystemTestRunner};
use maud::{DOCTYPE, Markup, html};

struct Row {
    id: u32,
    name: &'static str,
}

const ROWS: [Row; 3] = [
    Row { id: 1, name: "Ada" },
    Row {
        id: 2,
        name: "Grace",
    },
    Row {
        id: 3,
        name: "Linus",
    },
];

fn table(selected: &[String]) -> Markup {
    let columns = vec![
        Column::new("name", "Name", |r: &Row| html! { (r.name) }).row_header(),
        Column::new("id", "ID", |r: &Row| html! { (r.id) }),
    ];
    let actions = [
        BulkAction::new("archive", "Archive"),
        BulkAction::new("delete", "Delete").confirm("Delete the selected people?"),
    ];
    DataTable::new("people", "People", &ROWS, &columns)
        .selectable(|r| r.id.to_string())
        .row_label(|r| r.name.to_owned())
        .selected(selected)
        .bulk_actions("/people/actions", &actions, html! {})
        .render()
}

fn layout(body: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { "People" }
                link rel="stylesheet" href="/static/css/autumn-widgets.css";
                script src="/static/js/htmx.min.js" defer {}
                script src="/static/js/autumn-widgets.js" defer {}
                (pack().head())
            }
            body {
                main {
                    button id="start" type="button" { "Start" }
                    div id="slot" { (body) }
                    button id="swap" type="button" hx-get="/people/fragment"
                        hx-target="#slot" hx-swap="innerHTML" { "Swap" }
                }
            }
        }
    }
}

#[get("/people")]
async fn people() -> Markup {
    layout(&table(&[]))
}

#[get("/people/fragment")]
async fn people_fragment() -> Markup {
    table(&["2".to_owned()])
}

#[post("/people/actions")]
async fn people_actions(body: String) -> Markup {
    let bulk = BulkSelection::from_form(body.as_bytes()).expect("bulk body");
    let text = format!("{}: {}", bulk.action(), bulk.ids().join(","));
    html! { (DOCTYPE) html lang="en" { head { title { "Done" } } body { p id="result" { (text) } } } }
}

async fn runner() -> SystemTestRunner {
    let mut config = autumn_web::config::AutumnConfig::default();
    config.security.csrf.enabled = false;
    let state = autumn_web::AppState::for_test().with_profile("test");
    state.insert_extension(config);
    Box::pin(
        SystemTest::new()
            .state(state)
            .routes(routes![people, people_fragment, people_actions])
            .plugin_assets(&ASSETS)
            .build(),
    )
    .await
    .expect("start runner")
}

async fn eval<T: serde::de::DeserializeOwned + Default>(page: &Page, js: &str) -> T {
    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.into_value::<T>().ok())
        .unwrap_or_default()
}

async fn poll(page: &Page, js: &str) -> bool {
    for _ in 0..100 {
        if eval::<bool>(page, js).await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

async fn open(page: &Page, path: &str) {
    page.visit(path).await.expect("visit");
    assert!(
        poll(
            page,
            "document.readyState === 'complete' && window.__autumnWidgets === true"
        )
        .await,
        "page did not load"
    );
    assert!(
        poll(
            page,
            "!!document.querySelector('[data-autumn-datatable-ready]')"
        )
        .await,
        "the pack script did not set up the table"
    );
}

/// `[checked, indeterminate, aria-checked]` of the select-all box, and the
/// checked row ids, and the status text.
const STATE_JS: &str = r#"(() => {
  const all = document.getElementById('people-all');
  const rows = [...document.querySelectorAll('[data-autumn-datatable-row]')]
    .filter(c => c.checked).map(c => c.value);
  const status = document.getElementById('people-status').textContent.trim();
  return JSON.stringify([all.checked, all.indeterminate, all.getAttribute('aria-checked'), rows, status]);
})()"#;

async fn state(page: &Page) -> String {
    eval::<String>(page, STATE_JS).await
}

async fn active_id(page: &Page) -> String {
    eval::<String>(
        page,
        "document.activeElement && document.activeElement.id || ''",
    )
    .await
}

async fn active_value(page: &Page) -> String {
    eval::<String>(
        page,
        "document.activeElement && document.activeElement.value || ''",
    )
    .await
}

fn axe_source() -> String {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/axe-core/axe.min.js");
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

// ── The conformance gate ────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_pack_passes_the_browser_gate() {
    let pack = pack();
    let report = run_widget_conformance(&pack);
    assert!(report.passed(), "{}", report.to_text_report());
    let report = run_widget_browser_conformance(&pack, &axe_source())
        .await
        .expect("browser conformance runs");
    assert!(report.passed(), "{}", report.to_text_report());
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_mixed_state_page_has_no_axe_violation() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    open(&page, "/people").await;
    page.click("#swap").await.expect("swap");
    assert!(
        poll(
            &page,
            "document.getElementById('people-all').indeterminate === true"
        )
        .await,
        "the swapped table is not set up"
    );
    let violations = page.axe_violations(&axe_source()).await.expect("axe");
    assert!(violations.is_empty(), "{violations:#?}");
}

// ── The selection keyboard model ────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn tab_and_space_drive_the_selection() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    open(&page, "/people").await;
    assert_eq!(
        state(&page).await,
        r#"[false,false,null,[],"0 of 3 selected"]"#
    );

    // Tab reaches the scroll region, then select-all, then each row box.
    eval::<bool>(&page, "(document.getElementById('start').focus(), true)").await;
    page.press("Tab").await.expect("tab");
    assert!(
        eval::<bool>(
            &page,
            "document.activeElement.classList.contains('autumn-datatable-scroll')"
        )
        .await,
        "Tab does not reach the scroll region"
    );
    page.press("Tab").await.expect("tab");
    assert_eq!(active_id(&page).await, "people-all");

    // Space on select-all selects each row.
    page.press(" ").await.expect("space");
    assert_eq!(
        state(&page).await,
        r#"[true,false,null,["1","2","3"],"3 of 3 selected"]"#
    );

    // Space on a row clears it. Select-all is now mixed.
    page.press("Tab").await.expect("tab");
    assert_eq!(active_value(&page).await, "1");
    page.press(" ").await.expect("space");
    assert_eq!(
        state(&page).await,
        r#"[false,true,"mixed",["2","3"],"2 of 3 selected"]"#
    );

    page.press("Tab").await.expect("tab");
    assert_eq!(active_value(&page).await, "2");
    page.press(" ").await.expect("space");
    assert_eq!(
        state(&page).await,
        r#"[false,true,"mixed",["3"],"1 of 3 selected"]"#
    );

    // Space on a mixed select-all selects each row. Space again clears all.
    eval::<bool>(
        &page,
        "(document.getElementById('people-all').focus(), true)",
    )
    .await;
    page.press(" ").await.expect("space");
    assert_eq!(
        state(&page).await,
        r#"[true,false,null,["1","2","3"],"3 of 3 selected"]"#
    );
    page.press(" ").await.expect("space");
    assert_eq!(
        state(&page).await,
        r#"[false,false,null,[],"0 of 3 selected"]"#
    );

    // Space on each row in turn: the last one makes select-all on.
    for value in ["1", "2", "3"] {
        page.press("Tab").await.expect("tab");
        assert_eq!(active_value(&page).await, value);
        page.press(" ").await.expect("space");
    }
    assert_eq!(
        state(&page).await,
        r#"[true,false,null,["1","2","3"],"3 of 3 selected"]"#
    );
    page.expect_no_console_errors()
        .await
        .expect("no console error");
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn a_bulk_submit_with_no_row_stops_and_says_why() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    open(&page, "/people").await;
    eval::<bool>(
        &page,
        "(document.querySelector('#people-bulk [type=submit]').focus(), true)",
    )
    .await;
    page.press("Enter").await.expect("enter");
    assert!(
        poll(&page, "document.getElementById('people-status').textContent.includes('Select at least one row.')").await,
        "no message: {}",
        state(&page).await
    );
    assert!(eval::<bool>(&page, "location.pathname === '/people'").await);
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn a_confirm_action_opens_the_dialog_and_escape_cancels() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    open(&page, "/people").await;
    eval::<bool>(
        &page,
        "(() => { document.querySelector('[data-autumn-datatable-row][value=\"2\"]').click(); \
         document.getElementById('people-action').value = 'delete'; \
         document.querySelector('#people-bulk [type=submit]').focus(); return true; })()",
    )
    .await;
    page.press("Enter").await.expect("enter");
    assert!(
        poll(
            &page,
            "document.getElementById('people-confirm').open === true"
        )
        .await
    );
    assert!(
        eval::<bool>(&page, "document.querySelector('[data-autumn-datatable-confirm-text]').textContent === 'Delete the selected people?'").await
    );
    assert!(
        eval::<bool>(
            &page,
            "document.getElementById('people-confirm').contains(document.activeElement)"
        )
        .await
    );

    page.press("Escape").await.expect("escape");
    assert!(
        poll(
            &page,
            "document.getElementById('people-confirm').open === false"
        )
        .await
    );
    assert!(eval::<bool>(&page, "location.pathname === '/people'").await);
    assert!(
        poll(
            &page,
            "document.activeElement === document.querySelector('#people-bulk [type=submit]')"
        )
        .await,
        "focus did not go back to the submit button"
    );

    // Submit again and confirm: the form posts the selected id.
    page.press("Enter").await.expect("enter");
    assert!(
        poll(
            &page,
            "document.getElementById('people-confirm').open === true"
        )
        .await
    );
    eval::<bool>(
        &page,
        "(document.querySelector('[data-autumn-datatable-ok]').focus(), true)",
    )
    .await;
    page.press("Enter").await.expect("enter");
    assert!(
        poll(
            &page,
            "(document.getElementById('result') || {}).textContent === 'delete: 2'"
        )
        .await,
        "the form did not post"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn an_action_without_confirm_posts_at_once() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    open(&page, "/people").await;
    eval::<bool>(
        &page,
        "(document.getElementById('people-all').focus(), true)",
    )
    .await;
    page.press(" ").await.expect("space");
    eval::<bool>(
        &page,
        "(document.querySelector('#people-bulk [type=submit]').focus(), true)",
    )
    .await;
    page.press("Enter").await.expect("enter");
    assert!(
        poll(
            &page,
            "(document.getElementById('result') || {}).textContent === 'archive: 1,2,3'"
        )
        .await,
        "the form did not post"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn select_all_works_with_javascript_off() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    page.set_javascript_enabled(false).await.expect("js off");
    page.visit("/people").await.expect("visit");
    assert!(poll(&page, "document.readyState === 'complete'").await);
    eval::<bool>(
        &page,
        "(document.getElementById('people-all').focus(), true)",
    )
    .await;
    page.press(" ").await.expect("space");
    eval::<bool>(
        &page,
        "(document.querySelector('#people-bulk [type=submit]').focus(), true)",
    )
    .await;
    page.press("Enter").await.expect("enter");
    assert!(
        poll(
            &page,
            "(document.getElementById('result') || {}).textContent === 'archive: 1,2,3'"
        )
        .await,
        "select-all without JavaScript did not post the page ids"
    );
}
