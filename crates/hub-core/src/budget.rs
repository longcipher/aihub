use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};

use chrono::{Datelike, Utc};
use scc::HashMap;

use crate::types::virtual_key::BudgetMode;

struct Tracker {
    spent_cents: AtomicI64,
    month: AtomicU32,
    year: AtomicI64,
}

impl Tracker {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            spent_cents: AtomicI64::new(0),
            month: AtomicU32::new(now.month()),
            year: AtomicI64::new(now.year() as i64),
        }
    }

    fn check(&self, limit_cents: Option<i64>, mode: &BudgetMode) -> (bool, Option<i64>) {
        let limit = match limit_cents {
            Some(l) => l,
            None => return (true, None),
        };
        let now = Utc::now();
        if now.month() != self.month.load(Ordering::Relaxed) ||
            now.year() as i64 != self.year.load(Ordering::Relaxed)
        {
            self.spent_cents.store(0, Ordering::Relaxed);
            self.month.store(now.month(), Ordering::Relaxed);
            self.year.store(now.year() as i64, Ordering::Relaxed);
        }
        let spent = self.spent_cents.load(Ordering::Relaxed);
        let remaining = limit - spent;
        match mode {
            BudgetMode::Hard => (spent < limit, Some(remaining.max(0))),
            BudgetMode::Soft => {
                if spent >= limit {
                    tracing::warn!("Budget exceeded: spent {spent} cents, limit {limit} cents");
                }
                (true, Some(remaining))
            }
        }
    }

    fn add_spent(&self, cents: i64) {
        self.spent_cents.fetch_add(cents, Ordering::Relaxed);
    }
}

pub struct BudgetEnforcer {
    trackers: HashMap<String, std::sync::Arc<Tracker>>,
}

impl BudgetEnforcer {
    pub fn new() -> Self {
        Self { trackers: HashMap::new() }
    }

    pub fn check_budget(
        &self,
        key: &str,
        limit_cents: Option<i64>,
        mode: &BudgetMode,
    ) -> (bool, Option<i64>) {
        self.get_tracker(key).check(limit_cents, mode)
    }

    pub fn record_spending(&self, key: &str, cents: i64) {
        self.get_tracker(key).add_spent(cents);
    }

    fn get_tracker(&self, key: &str) -> std::sync::Arc<Tracker> {
        if let Some(t) = self.trackers.read_sync(key, |_, v| v.clone()) {
            return t;
        }
        let t = std::sync::Arc::new(Tracker::new());
        let _ = self.trackers.insert_sync(key.to_string(), t.clone());
        t
    }
}

impl Default for BudgetEnforcer {
    fn default() -> Self {
        Self::new()
    }
}
