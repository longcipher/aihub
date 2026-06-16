use std::sync::{
    Arc,
    atomic::{AtomicI64, AtomicU32, Ordering},
};

use chrono::{Datelike, Utc};
use hub_core::types::virtual_key::BudgetMode;
use scc::HashMap;

/// Budget tracker for a single key
struct BudgetTracker {
    spent_cents: AtomicI64,
    month: AtomicU32,
    year: AtomicI64,
    monthly_limit_cents: Option<i64>,
    budget_mode: BudgetMode,
}

impl BudgetTracker {
    fn new(monthly_limit_cents: Option<i64>, budget_mode: BudgetMode) -> Self {
        let now = Utc::now();
        Self {
            spent_cents: AtomicI64::new(0),
            month: AtomicU32::new(now.month()),
            year: AtomicI64::new(now.year() as i64),
            monthly_limit_cents,
            budget_mode,
        }
    }

    /// Check if a request is allowed under the budget.
    /// Returns (allowed, remaining_cents)
    fn check_budget(&self) -> (bool, Option<i64>) {
        // If no limit, always allow
        let limit = match self.monthly_limit_cents {
            Some(l) => l,
            None => return (true, None),
        };

        // Check if month has changed (reset budget)
        let now = Utc::now();
        let current_month = now.month();
        let current_year = now.year() as i64;

        let stored_month = self.month.load(Ordering::Relaxed);
        let stored_year = self.year.load(Ordering::Relaxed);

        if current_month != stored_month || current_year != stored_year {
            // New month - reset budget
            self.spent_cents.store(0, Ordering::Relaxed);
            self.month.store(current_month, Ordering::Relaxed);
            self.year.store(current_year, Ordering::Relaxed);
        }

        let spent = self.spent_cents.load(Ordering::Relaxed);
        let remaining = limit - spent;

        match self.budget_mode {
            BudgetMode::Hard => (spent < limit, Some(remaining.max(0))),
            BudgetMode::Soft => {
                if spent >= limit {
                    tracing::warn!(
                        "Budget exceeded for key: spent {} cents, limit {} cents",
                        spent,
                        limit
                    );
                }
                (true, Some(remaining))
            }
        }
    }

    /// Add spent amount in cents
    fn add_spent(&self, cents: i64) {
        self.spent_cents.fetch_add(cents, Ordering::Relaxed);
    }

    fn spent(&self) -> i64 {
        self.spent_cents.load(Ordering::Relaxed)
    }
}

/// Budget enforcement middleware
pub struct BudgetEnforcer {
    trackers: HashMap<String, Arc<BudgetTracker>>,
}

impl BudgetEnforcer {
    pub fn new() -> Self {
        Self { trackers: HashMap::new() }
    }

    /// Get or create a tracker for the given key
    fn get_or_create_tracker(
        &self,
        key: &str,
        monthly_limit_cents: Option<i64>,
        budget_mode: &BudgetMode,
    ) -> Arc<BudgetTracker> {
        if let Some(tracker) = self.trackers.read_sync(key, |_, v| v.clone()) {
            return tracker;
        }
        let tracker = Arc::new(BudgetTracker::new(monthly_limit_cents, budget_mode.clone()));
        let _ = self.trackers.insert_sync(key.to_string(), tracker.clone());
        tracker
    }

    /// Check if a request is allowed under the budget.
    /// Returns (allowed, remaining_cents)
    pub fn check_budget(
        &self,
        key: &str,
        monthly_limit_cents: Option<i64>,
        budget_mode: &BudgetMode,
    ) -> (bool, Option<i64>) {
        self.get_or_create_tracker(key, monthly_limit_cents, budget_mode).check_budget()
    }

    /// Record spending for a key (uses default budget mode if key doesn't exist)
    pub fn record_spending(&self, key: &str, cents: i64) {
        let tracker = self.get_or_create_tracker(key, None, &BudgetMode::default());
        tracker.add_spent(cents);
    }

    /// Get remaining budget for a key
    pub fn remaining(&self, key: &str) -> Option<i64> {
        self.trackers.read_sync(key, |_, v| {
            let (_, remaining) = v.check_budget();
            remaining
        })?
    }

    /// Get spent amount for a key
    pub fn spent(&self, key: &str) -> Option<i64> {
        self.trackers.read_sync(key, |_, v| v.spent())
    }
}

impl Default for BudgetEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_no_limit() {
        let enforcer = BudgetEnforcer::new();
        let (allowed, remaining) = enforcer.check_budget("key1", None, &BudgetMode::Hard);
        assert!(allowed);
        assert_eq!(remaining, None);
    }

    #[test]
    fn test_budget_hard_mode_under_limit() {
        let enforcer = BudgetEnforcer::new();
        let (allowed, remaining) = enforcer.check_budget("key1", Some(100), &BudgetMode::Hard);
        assert!(allowed);
        assert_eq!(remaining, Some(100));
    }

    #[test]
    fn test_budget_hard_mode_over_limit() {
        let enforcer = BudgetEnforcer::new();
        // Create the tracker with limit first
        enforcer.check_budget("key1", Some(100), &BudgetMode::Hard);
        // Spend the budget
        enforcer.record_spending("key1", 100);
        let (allowed, remaining) = enforcer.check_budget("key1", Some(100), &BudgetMode::Hard);
        assert!(!allowed);
        assert_eq!(remaining, Some(0));
    }

    #[test]
    fn test_budget_soft_mode_over_limit() {
        let enforcer = BudgetEnforcer::new();
        // Create the tracker with limit first
        enforcer.check_budget("key1", Some(100), &BudgetMode::Soft);
        // Spend the budget
        enforcer.record_spending("key1", 100);
        let (allowed, remaining) = enforcer.check_budget("key1", Some(100), &BudgetMode::Soft);
        assert!(allowed); // Soft mode allows
        assert_eq!(remaining, Some(0));
    }

    #[test]
    fn test_budget_separate_keys() {
        let enforcer = BudgetEnforcer::new();
        enforcer.record_spending("key1", 100);
        // key2 should still be allowed
        let (allowed, _) = enforcer.check_budget("key2", Some(100), &BudgetMode::Hard);
        assert!(allowed);
    }

    #[test]
    fn test_budget_spent_tracking() {
        let enforcer = BudgetEnforcer::new();
        // Initialize tracker first
        enforcer.check_budget("key1", Some(200), &BudgetMode::Hard);
        enforcer.record_spending("key1", 50);
        enforcer.record_spending("key1", 30);
        assert_eq!(enforcer.spent("key1"), Some(80));
    }

    #[test]
    fn test_budget_remaining_unknown_key() {
        let enforcer = BudgetEnforcer::new();
        assert_eq!(enforcer.remaining("unknown"), None);
    }
}
