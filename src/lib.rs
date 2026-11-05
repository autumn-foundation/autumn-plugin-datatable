//! Data table v2: a widget pack for `autumn-web` (autumn#3414).
//!
//! The server drives the table with query parameters. It works with
//! JavaScript off. The pack adds to the core `data_table`:
//!
//! - a selection column with a select-all box (`aria-checked="mixed"`)
//!   and a bulk action form,
//! - a filter bar (`filter[<key>]`, many values) with removable chips,
//! - column visibility, density and a sticky header,
//! - empty and loading states.
//!
//! [`TableState`] is the state. It decodes with `Query<TableState>` and
//! encodes to the same query string. [`TableState::checked`] keeps only
//! what the table declares.
//!
//! # Install
//!
//! ```rust
//! use autumn_datatable_plugin::DatatablePlugin;
//!
//! fn app() -> autumn_web::app::AppBuilder {
//!     autumn_web::app().plugin(DatatablePlugin)
//! }
//! ```
//!
//! In the layout `<head>`, after the core widget assets:
//!
//! ```rust
//! let head = autumn_datatable_plugin::pack().head().into_string();
//! assert!(head.contains("datatable"));
//! ```
//!
//! # Use
//!
//! ```rust
//! use autumn_datatable_plugin::{BulkAction, Column, DataTable, Filter, TableState};
//!
//! struct Post { id: i64, title: String, status: String }
//!
//! const STATUS: &[(&str, &str)] = &[("draft", "Draft"), ("live", "Live")];
//!
//! # fn handler(query: &str, rows: Vec<Post>) -> maud::Markup {
//! let columns = vec![
//!     Column::new("title", "Title", |p: &Post| maud::html! { (p.title) }).sortable().row_header(),
//!     Column::new("status", "Status", |p: &Post| maud::html! { (p.status) }).hideable(),
//! ];
//! let filters = [Filter::multi("status", "Status", STATUS)];
//! let state = TableState::from_query(query).unwrap_or_default().checked(&columns, &filters);
//! // Load `rows` with `state.list_query()` and `state.page_request()`.
//! let actions = [BulkAction::new("delete", "Delete").confirm("Delete the selected posts?")];
//! DataTable::new("posts", "Posts", &rows, &columns)
//!     .state(&state)
//!     .base_path("/posts")
//!     .search("Search posts")
//!     .filters(&filters)
//!     .selectable(|p| p.id.to_string())
//!     .bulk_actions("/posts/actions", &actions, maud::html! {})
//!     .render()
//! # }
//! ```

mod selection;
mod state;
mod stories;
mod table;

use autumn_web::app::AppBuilder;
use autumn_web::assets::PluginAssets;
use autumn_web::plugin::Plugin;
use autumn_web::widget_pack::WidgetPack;

pub use autumn_web::pagination::SortDir;
pub use selection::{BulkSelection, SelectAll};
pub use state::{Density, MAX_TEXT_LEN, TableState};
pub use table::{BulkAction, Column, DataTable, Filter, FilterKind};

/// The pack CSS and JS.
pub static ASSETS: PluginAssets = PluginAssets::from_files(
    "datatable",
    &[
        ("datatable.css", include_bytes!("datatable.css")),
        ("datatable.js", include_bytes!("datatable.js")),
    ],
);

/// The pack: its stories and its assets.
#[must_use]
pub fn pack() -> WidgetPack {
    WidgetPack::new("datatable")
        .extend(stories::all())
        .assets(&ASSETS)
}

/// The plugin. It installs [`pack`]: the assets and the stories.
#[derive(Debug, Default, Clone, Copy)]
pub struct DatatablePlugin;

impl Plugin for DatatablePlugin {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(env!("CARGO_PKG_NAME"))
    }

    fn build(self, app: AppBuilder) -> AppBuilder {
        app.widget_pack(pack())
    }
}
