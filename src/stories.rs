//! One story for each feature of the table.

use autumn_web::stories::{Story, story};
use maud::{Markup, html};

use crate::{BulkAction, Column, DataTable, Filter, TableState};

struct Person {
    id: u32,
    name: &'static str,
    role: &'static str,
    team: &'static str,
}

const PEOPLE: [Person; 4] = [
    Person {
        id: 1,
        name: "Ada Lovelace",
        role: "admin",
        team: "Core",
    },
    Person {
        id: 2,
        name: "Grace Hopper",
        role: "editor",
        team: "Compilers",
    },
    Person {
        id: 3,
        name: "Alan Turing",
        role: "viewer",
        team: "Theory",
    },
    Person {
        id: 4,
        name: "Edsger Dijkstra",
        role: "editor",
        team: "Theory",
    },
];

const ROLES: &[(&str, &str)] = &[
    ("admin", "Admin"),
    ("editor", "Editor"),
    ("viewer", "Viewer"),
];
const TEAMS: &[(&str, &str)] = &[
    ("Core", "Core"),
    ("Compilers", "Compilers"),
    ("Theory", "Theory"),
];

fn columns() -> Vec<Column<'static, Person>> {
    vec![
        Column::new("name", "Name", |p: &Person| html! { (p.name) })
            .sortable()
            .row_header(),
        Column::new("role", "Role", |p: &Person| html! { (p.role) })
            .sortable()
            .hideable(),
        Column::new("team", "Team", |p: &Person| html! { (p.team) }).hideable(),
        Column::new("id", "ID", |p: &Person| html! { (p.id) }).hidden(),
    ]
}

fn state(query: &str) -> TableState {
    TableState::from_query(query).unwrap_or_default()
}

fn sorting() -> Markup {
    let cols = columns();
    let st = state("sort=name&dir=desc");
    DataTable::new("datatable-sorting", "People", &PEOPLE, &cols)
        .state(&st)
        .render()
}

fn selection() -> Markup {
    let cols = columns();
    let selected = ["2".to_owned()];
    DataTable::new("datatable-selection", "People", &PEOPLE, &cols)
        .selectable(|p| p.id.to_string())
        .row_label(|p| p.name.to_owned())
        .selected(&selected)
        .render()
}

fn bulk_actions() -> Markup {
    let cols = columns();
    let actions = [
        BulkAction::new("archive", "Archive"),
        BulkAction::new("delete", "Delete").confirm("Delete the selected people?"),
    ];
    DataTable::new("datatable-bulk", "People", &PEOPLE, &cols)
        .selectable(|p| p.id.to_string())
        .row_label(|p| p.name.to_owned())
        .bulk_actions("/people/actions", &actions, html! {})
        .render()
}

fn filters() -> Markup {
    let cols = columns();
    let filters = [
        Filter::multi("role", "Role", ROLES),
        Filter::select("team", "Team", TEAMS),
        Filter::text("name", "Name"),
    ];
    let st = state("q=a&filter[role]=editor&filter[team]=Theory").checked(&cols, &filters);
    let rows = [Person {
        id: 4,
        name: "Edsger Dijkstra",
        role: "editor",
        team: "Theory",
    }];
    DataTable::new("datatable-filters", "People", &rows, &cols)
        .state(&st)
        .search("Search people")
        .filters(&filters)
        .render()
}

fn columns_and_density() -> Markup {
    let cols = columns();
    let st = state("cols=role&cols=id&density=compact");
    DataTable::new("datatable-columns", "People", &PEOPLE, &cols)
        .state(&st)
        .density_switch()
        .render()
}

fn sticky_header() -> Markup {
    let cols = columns();
    DataTable::new("datatable-sticky", "People", &PEOPLE, &cols)
        .sticky_header()
        .render()
}

fn empty() -> Markup {
    let cols = columns();
    let rows: [Person; 0] = [];
    DataTable::new("datatable-empty", "People", &rows, &cols)
        .empty(
            "No people yet",
            "Invite the first person.",
            Some(html! { a href="#invite" { "Invite a person" } }),
        )
        .render()
}

fn no_match() -> Markup {
    let cols = columns();
    let filters = [Filter::multi("role", "Role", ROLES)];
    let st = state("filter[role]=admin&filter[role]=viewer").checked(&cols, &filters);
    let rows: [Person; 0] = [];
    DataTable::new("datatable-no-match", "People", &rows, &cols)
        .state(&st)
        .filters(&filters)
        .render()
}

fn loading() -> Markup {
    let cols = columns();
    let rows: [Person; 0] = [];
    DataTable::new("datatable-loading", "People", &rows, &cols)
        .selectable(|p| p.id.to_string())
        .loading(3)
        .render()
}

/// The pack stories.
pub fn all() -> Vec<Story> {
    vec![
        story! { "Datatable", "Datatable sorting", { sorting() } },
        story! { "Datatable", "Datatable selection", { selection() } },
        story! { "Datatable", "Datatable bulk actions", { bulk_actions() } },
        story! { "Datatable", "Datatable filters", { filters() } },
        story! { "Datatable", "Datatable columns and density", { columns_and_density() } },
        story! { "Datatable", "Datatable sticky header", { sticky_header() } },
        story! { "Datatable", "Datatable empty", { empty() } },
        story! { "Datatable", "Datatable no match", { no_match() } },
        story! { "Datatable", "Datatable loading", { loading() } },
    ]
}
