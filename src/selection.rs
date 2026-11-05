//! Row selection: the select-all state and the bulk form body.

use std::collections::BTreeSet;

use autumn_web::form::FormDecodeError;
use serde::Deserialize;

use crate::table::BulkAction;

/// The state of the select-all checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectAll {
    /// No row is selected.
    None,
    /// Some rows are selected. `aria-checked="mixed"`.
    Mixed,
    /// Each row is selected.
    All,
}

impl SelectAll {
    /// The state for `selected` of `total` rows. A count above the total is
    /// the total. No rows is [`SelectAll::None`].
    #[must_use]
    pub const fn from_counts(selected: usize, total: usize) -> Self {
        if total == 0 || selected == 0 {
            Self::None
        } else if selected >= total {
            Self::All
        } else {
            Self::Mixed
        }
    }

    /// The state for the rows on the page. An id that is not on the page
    /// does not count. An id counts one time.
    #[must_use]
    pub fn for_page<P: AsRef<str>, S: AsRef<str>>(page_ids: &[P], selected: &[S]) -> Self {
        let selected: BTreeSet<&str> = selected.iter().map(AsRef::as_ref).collect();
        let page: BTreeSet<&str> = page_ids.iter().map(AsRef::as_ref).collect();
        Self::from_counts(page.intersection(&selected).count(), page.len())
    }

    /// The `aria-checked` value: `false`, `mixed` or `true`.
    #[must_use]
    pub const fn aria_checked(self) -> &'static str {
        match self {
            Self::None => "false",
            Self::Mixed => "mixed",
            Self::All => "true",
        }
    }

    /// `true` for [`SelectAll::All`]: the native `checked` state.
    #[must_use]
    pub const fn is_checked(self) -> bool {
        matches!(self, Self::All)
    }
}

/// The body of a bulk action form.
///
/// | Field | Meaning |
/// |---|---|
/// | `action` | The action name. |
/// | `ids` | The selected row ids. |
/// | `all` | Select-all is on. The form also sends it with JavaScript off. |
/// | `visible` | The ids of the rows on the page. |
/// | `all_shown` | The server rendered select-all on. |
/// | `confirmed` | The user confirmed the action in the dialog. |
///
/// The ids come from the client. Check that the user can change each row.
///
/// ```rust
/// use autumn_datatable_plugin::{BulkAction, BulkSelection};
///
/// let actions = [BulkAction::new("archive", "Archive")];
/// let bulk = BulkSelection::from_form(b"action=archive&ids=4&ids=7").unwrap();
/// assert_eq!(bulk.action_in(&actions).map(BulkAction::name), Some("archive"));
/// assert_eq!(bulk.ids(), ["4", "7"]);
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BulkSelection {
    #[serde(default)]
    action: String,
    #[serde(default)]
    ids: Vec<String>,
    #[serde(default)]
    all: Option<String>,
    #[serde(default)]
    visible: Vec<String>,
    #[serde(default)]
    all_shown: Option<String>,
    #[serde(default)]
    confirmed: Option<String>,
}

impl BulkSelection {
    /// Decode an `application/x-www-form-urlencoded` body.
    ///
    /// # Errors
    ///
    /// [`FormDecodeError`] when `action` or `all` occurs two times.
    pub fn from_form(body: &[u8]) -> Result<Self, FormDecodeError> {
        autumn_web::form::from_urlencoded(body)
    }

    /// The action name, as sent. Use [`action_in`](Self::action_in) to
    /// check it.
    #[must_use]
    pub fn action(&self) -> &str {
        &self.action
    }

    /// The declared action with this name, or `None`.
    #[must_use]
    pub fn action_in<'a, 'b>(&self, actions: &'a [BulkAction<'b>]) -> Option<&'a BulkAction<'b>> {
        actions.iter().find(|a| a.name() == self.action)
    }

    /// `true` when the user confirmed the action in the dialog.
    #[must_use]
    pub fn is_confirmed(&self) -> bool {
        self.confirmed.as_deref().is_some_and(|v| !v.is_empty())
    }

    /// `true` when the action asks for a confirm step and the user did not
    /// confirm. With JavaScript off, the form posts with no dialog. Then
    /// show a confirm page that posts the same body with `confirmed=1`.
    #[must_use]
    pub fn needs_confirm(&self, actions: &[BulkAction<'_>]) -> bool {
        self.action_in(actions)
            .is_some_and(|a| a.confirm_text().is_some())
            && !self.is_confirmed()
    }

    /// `true` when select-all is on.
    #[must_use]
    pub fn is_all(&self) -> bool {
        self.all.as_deref().is_some_and(|v| !v.is_empty())
    }

    /// The ids to act on, one time each, in order, with no empty id.
    ///
    /// With JavaScript off, a select-all box does not change the row boxes.
    /// The `all_shown` field tells the two cases apart:
    ///
    /// - The user turned select-all on: the page ids.
    /// - Select-all was on in the page: the selected ids. Then a row that
    ///   the user clears stays out.
    ///
    /// With JavaScript on, the row boxes follow select-all, so both are the
    /// same.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        let shown = self.all_shown.as_deref().is_some_and(|v| !v.is_empty());
        let source = if self.is_all() && !shown {
            &self.visible
        } else {
            &self.ids
        };
        let mut seen = BTreeSet::new();
        source
            .iter()
            .map(String::as_str)
            .filter(|id| !id.is_empty() && seen.insert(*id))
            .collect()
    }

    /// `true` when there is no id to act on.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids().is_empty()
    }
}
