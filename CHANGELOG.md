# Changelog

## [Unreleased]

### Added

- `DataTable`: data table v2 as a widget pack (autumn#3414). The server
  drives it with query parameters. It works with JavaScript off.
- Selection column with a select-all box (`aria-checked="mixed"`) and a bulk
  action form. A confirm action opens a `<dialog>`. With JavaScript off,
  `BulkSelection::needs_confirm` tells the server to ask.
- Filter bar: text, select and checkbox filters as `filter[<key>]`, with
  removable chips.
- Column visibility (`cols`), density (`density`) and a sticky header.
- Empty and loading states.
- `TableState`: decode, encode and link methods. `checked` keeps only
  declared keys.
- `BulkSelection`: decodes the bulk form body.
- `DatatablePlugin` and `pack()`: install the assets and the stories.
