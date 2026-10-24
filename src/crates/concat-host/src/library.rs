// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
//! Local usage ordering shared by asset libraries. Previewing never records a use.

use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap};

/// Successful applications of one library item.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Usage {
    /// Lifetime application count.
    pub count: u64,
    /// Monotonic order of its most recent application, independent of wall time.
    pub last: u64,
}
/// Small persistent history; stores IDs and numbers, never provider media URLs.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct History {
    /// Changes after each successful application, also invalidating shelf caches.
    pub clock: u64,
    /// Usage by stable package ID or `giphy:<provider-id>`.
    pub items: BTreeMap<String, Usage>,
}
impl History {
    /// Records a successful user application. Undo/redo and previews do not call this.
    pub fn record(&mut self, id: &str) {
        if id.is_empty() {
            return;
        }
        self.clock = self
            .clock
            .max(self.items.values().map(|v| v.last).max().unwrap_or(0))
            .saturating_add(1);
        let entry = self.items.entry(id.to_owned()).or_default();
        entry.count = entry.count.saturating_add(1);
        entry.last = self.clock;
    }
    /// Whether an item has actually been used, for Most used and Recent views.
    pub fn used(&self, id: &str) -> bool {
        self.items.get(id).is_some_and(|v| v.count > 0)
    }
    /// Stable sort: Most used uses frequency, then recency; Recent uses recency.
    pub fn compare(&self, mode: i32, a: &str, b: &str) -> Ordering {
        let a_use = self.items.get(a).cloned().unwrap_or_default();
        let b_use = self.items.get(b).cloned().unwrap_or_default();
        (if mode == 2 {
            b_use.count.cmp(&a_use.count)
        } else {
            Ordering::Equal
        })
        .then_with(|| b_use.last.cmp(&a_use.last))
        .then_with(|| a.cmp(b))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frequency_recency_and_persistence_are_distinct() {
        let mut history = History::default();
        history.record("effect.a");
        history.record("effect.a");
        history.record("giphy:B");
        assert_eq!(history.compare(2, "effect.a", "giphy:B"), Ordering::Less);
        assert_eq!(history.compare(3, "effect.a", "giphy:B"), Ordering::Greater);
        assert!(!history.used("never-used"));
        let mut restored: History =
            serde_json::from_str(&serde_json::to_string(&history).unwrap()).unwrap();
        restored.record("effect.a");
        assert_eq!(restored.items["effect.a"].count, 3);
        assert_eq!(restored.compare(3, "effect.a", "giphy:B"), Ordering::Less);
        assert_eq!(restored.items["giphy:B"].count, 1);
    }
}
