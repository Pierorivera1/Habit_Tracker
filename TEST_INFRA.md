# Habitodo Test Infrastructure Specification (`TEST_INFRA.md`)

## 1. Overview & Testing Philosophy

**Habitodo** is a native dark-mode habit and to-do tracking application implemented in Rust with SQLite (`rusqlite`) and the `iced` GUI framework.
The test infrastructure provides **100% headless, opaque-box integration verification** decoupled from presentation side-effects.

### Core Testing Invariants
1. **Progressive Testability**: Tests verify observable interface contracts and public invariants without relying on uncompleted milestones.
2. **Headless & Deterministic**: Full persistence verification runs via in-memory SQLite instances (`Database::new_in_memory()`) and isolated temporary files (`tempfile::tempdir()`).
3. **Zero-Panic Guarantee**: Empty states, missing records, invalid inputs, duplicate keys, and boundary conditions must return structured `Result<T, rusqlite::Error>` without aborting or panicking.
4. **Data Integrity**: WAL mode, atomic UPSERTs, cascading foreign keys, and transactional export/import schemas are thoroughly verified under high concurrency and stress.

---

## 2. Test Architecture & Tier Hierarchy

The integration test suite is located in `tests/` and structured into four progressive tiers plus shared fixtures:

```
tests/
├── common/
│   └── mod.rs                 # Shared TestHarness, seeding fixtures & date assertions
├── test_tier1_features.rs     # Core feature verification (>=5 tests per core feature)
├── test_tier2_boundaries.rs   # Edge conditions, extremes, date boundaries, SQL injection
├── test_tier3_interactions.rs # Pairwise feature combinations and cross-module lifecycles
├── test_tier4_workloads.rs    # Real-world multi-day simulations and sync scenarios
└── empirical_challenge.rs     # Multi-threaded stress tests, volume benchmarks, WAL concurrency
```

### Tier 1: Core Feature Verification (`tests/test_tier1_features.rs`)
Focuses on individual feature correctness with >= 5 dedicated tests per core feature group (45 tests total):
- **Core Feature 1: Database Engine & Schema (Features 1, 2, 3)**:
  - In-memory lifecycle and performance PRAGMAs (`WAL`, `foreign_keys = ON`, `busy_timeout = 5000`).
  - Recursive parent directory creation for disk databases.
  - Schema creation idempotency.
  - Verification of tables (`habits`, `habit_completions`, `todos`, `settings`) and indexes (`idx_completions_date`, `idx_todos_completed`, `idx_todos_due`).
  - Settings key-value store upsert and retrieval.
  - Cascading deletion of completions on habit removal.
- **Core Feature 2: Domain Models & Serde (Features 4, 5)**:
  - `Habit` serialization/deserialization roundtrip.
  - `Todo` optional fields handling and minimal JSON parsing.
  - `HabitCompletion` default completed values.
  - `DailyStats::percentage` numerical safety (0/0 -> 0.0 without NaN/Inf).
  - Exact fraction calculations (0%, 25%, 50%, 75%, 100%, 33.3%).
  - Full `ExportData` roundtrip with nested entities.
  - Backward compatibility via `completions` serde alias.
- **Core Feature 3: Daily Habits & Atomic UPSERT (Features 6, 7)**:
  - `LEFT JOIN` unrecorded habit default (`completed = false`).
  - Consistent ordering by position and ID.
  - Atomic UPSERT initial insertion (`completed = true`).
  - Atomic UPSERT toggle inversion (`true -> false`).
  - Rapid multi-toggle single row invariant.
  - Date isolation across consecutive days.
  - Exclusion of soft-deleted (`archived = 1`) habits.
- **Core Feature 4: Analytics & Metrics Queries (Features 8, 9, 10)**:
  - Empty database stats safety.
  - 30-day sliding window date range filtering.
  - Daily completion rollup aggregation accuracy.
  - Habit 30-day success adherence rate calculation.
  - Zero adherence vs 100% adherence boundaries.
  - Non-existent habit rate safety (returns 0.0).
  - Monthly heatmap boundary strictness (month isolation).
  - Multi-habit daily completion summation in heatmap.
- **Core Feature 5: To-Dos CRUD & Timestamping (Feature 11)**:
  - To-do insertion with unique auto-incrementing IDs.
  - Insertion with optional notes and due dates.
  - Completion toggling with automatic `completed_at` timestamping and clearing.
  - In-place updates to title, notes, and deadlines.
  - Item deletion and ID cleanup.
  - Sorting: uncompleted items precede completed items.
- **Core Feature 6: Habit Setup & Reordering (Feature 12)**:
  - Automatic position increment on habit addition.
  - Unique habit name constraint enforcement (`rusqlite::Error`).
  - Transactional position swapping (`reorder_habit` up/down).
  - Boundary reordering no-ops (top up, bottom down).
  - Soft-delete archiving (`archived = 1`) and daily list exclusion.
  - In-place renaming preserving ID and completions.
- **Core Feature 7: JSON Export & Import (Feature 13)**:
  - Comprehensive database capture across all four tables.
  - Complete state restoration into clean database instances.
  - Transactional rollback on foreign key or schema violation.
  - Unicode, emoji, and quote character preservation.
  - Idempotent updates on matching primary keys.

### Tier 2: Boundary & Edge Cases (`tests/test_tier2_boundaries.rs`)
Focuses on extreme inputs, adversarial payloads, and system limits (20 tests total):
- Empty database operations across all queries without panic.
- String boundaries: 1-character, 500-character, 2048-character habit names.
- Whitespace formatting: leading/trailing whitespace, embedded `\n`, `\r\n`, and `\t`.
- Internationalization: Emojis, CJK characters, Arabic script, mathematical symbols.
- SQL injection immunity: verification against malicious injection payloads.
- Extreme note sizes: 70 KB text payloads in `todos.notes`.
- Minimal/empty to-do titles.
- Calendar boundaries:
  - Leap year February 29 (e.g. 2024, 2028).
  - Non-leap year February 28 (e.g. 2025, 2026).
  - Year rollover: December 31 to January 1 across 30-day rollups.
  - Month length transitions: 30-day (Apr 30 -> May 1) and 31-day (Jul 31 -> Aug 1).
  - Epoch extremes: dates in 1970 and 2099.
- Foreign key and non-existent ID error handling.
- Reordering edge cases (empty list, 1 habit).
- Idempotent archiving on already-archived records.
- 100-cycle rapid toggle state consistency.
- Settings boundary handling (empty keys and empty values).
- `DailyStats` extreme numerical limits (`usize::MAX`).

### Tier 3: Interactions & Combinations (`tests/test_tier3_interactions.rs`)
Focuses on cross-module lifecycles and feature interactions (10 tests total):
- Habit creation -> Daily toggle -> Archiving -> Verification of heatmap and historical adherence retention.
- Habit reordering -> Daily checklist display order verification.
- To-Do lifecycle: Add -> Complete (timestamp) -> Update -> Complete -> Uncomplete (clear timestamp) -> Delete -> Verify active sort.
- Daily habit completion matrix -> 30-day stats calculation -> `DailyStats::percentage` correlation.
- Monthly heatmap isolation across month crossover dates (Sept 28 - Oct 2).
- Habit rename preserving completions and positional index.
- Cascade deletion updating 30-day stats rollups.
- Full backup export -> clean DB restore -> seamless tracking continuation.
- Key-value application settings surviving full export/import cycle.
- Multi-habit multi-day completion states under concurrent reordering.

### Tier 4: Real-World Workloads (`tests/test_tier4_workloads.rs`)
Simulates authentic multi-day user journeys (5 tests total):
- **30-Day New Habit Challenge**: 3 initial habits (100% adherence, weekday-only adherence, alternating adherence) plus 4th habit added on Day 16. Verifies 30-day trend lines, per-habit rates, and monthly heatmap density.
- **Busy Two-Week Sprint**: 14 days of realistic task management. Daily habit tracking, dynamic to-do generation, mid-sprint habit archiving, reordering, and todo sorting.
- **Multi-Device Synchronization Simulation**: Device A logs 45 days of activity and exports to JSON; Device B imports state, asserts byte-for-byte domain equivalence, and continues tracking on Day 46 and Day 47 without conflict.
- **Relapse & Recovery Habit Cycle**: 30-day tracking of a challenging routine across 4 distinct phases (10-day streak -> 7-day relapse -> 10-day recovery -> 3-day maintenance) with rolling success rate assertions.
- **Quarterly Multi-Habit Scale**: 90 simulated days (July 1 to Sept 28) across 6 active habits with varied recurrence intervals (>500 completion records). Verifies quarterly heatmap isolation and instant query response.

---

## 3. Shared Test Infrastructure (`tests/common/mod.rs`)

### `TestHarness` API
```rust
pub struct TestHarness {
    pub db: Database,
    _temp_dir: Option<TempDir>,
    pub db_path: Option<PathBuf>,
}

impl TestHarness {
    pub fn in_memory() -> Self;
    pub fn temp_file() -> Self;
    pub fn db(&self) -> &Database;
    pub fn seed_habits(&self, names: &[&str]) -> Vec<i64>;
    pub fn seed_standard_habits(&self) -> Vec<i64>;
    pub fn seed_todos(&self, titles: &[&str]) -> Vec<i64>;
    pub fn toggle_habit_on_dates(&self, habit_id: i64, dates: &[NaiveDate]);
    pub fn simulate_day(&self, date: NaiveDate, habit_ids: &[i64], completed_flags: &[bool]);
    pub fn assert_habit_status(&self, date: NaiveDate, habit_id: i64, expected: bool);
    pub fn assert_todo_status(&self, todo_id: i64, expected_completed: bool);
    pub fn get_habits_for_date(&self, date: NaiveDate) -> Vec<(Habit, bool)>;
    pub fn get_todos(&self) -> Vec<Todo>;
}
```

### Date & Assertion Utilities
- `date(year, month, day) -> NaiveDate`: Safe calendar date constructor.
- `date_range(start, count) -> Vec<NaiveDate>`: Sequence generator for consecutive calendar days.
- `days_ago(base, count) -> NaiveDate`: Backward calendar date navigation.
- `assert_approx_eq(actual, expected, tolerance)`: Floating-point precision comparator with actionable diff reporting.

---

## 4. Execution & Runner Commands

Run the entire integration test suite:
```bash
cargo test --test '*'
```

Run a specific tier:
```bash
cargo test --test test_tier1_features
cargo test --test test_tier2_boundaries
cargo test --test test_tier3_interactions
cargo test --test test_tier4_workloads
cargo test --test empirical_challenge
```

Run with standard output displayed:
```bash
cargo test --test '*' -- --nocapture
```

---

## 5. Feature Coverage Traceability Matrix

| Feature ID | Description | Primary Test File | Key Test Cases |
| :--- | :--- | :--- | :--- |
| **F1** | SQLite Engine & WAL Mode | `test_tier1_features.rs` | `test_f1_in_memory_db_creation_and_pragmas`, `test_f1_file_db_creation_with_nested_directories` |
| **F2** | Full v1 DDL Schema & Indexes | `test_tier1_features.rs` | `test_f1_all_tables_and_indexes_verification`, `test_f1_schema_idempotency_multiple_calls` |
| **F3** | In-Memory SQLite Support | `test_tier1_features.rs`, `common/` | `test_f1_in_memory_db_creation_and_pragmas`, `TestHarness::in_memory` |
| **F4** | Domain Models & Serde | `test_tier1_features.rs` | `test_f2_habit_serde_json_roundtrip`, `test_f2_todo_serde_with_optional_fields` |
| **F5** | DailyStats Percentage Method | `test_tier1_features.rs`, `test_tier2_boundaries.rs` | `test_f2_dailystats_percentage_zero_habits_safe`, `test_boundary_dailystats_percentage_edge_inputs` |
| **F6** | LEFT JOIN Daily Habits Query | `test_tier1_features.rs` | `test_f3_left_join_defaults_uncompleted_to_false`, `test_f3_daily_habits_ordered_by_position_and_id` |
| **F7** | Atomic UPSERT Habit Toggle | `test_tier1_features.rs`, `test_tier2_boundaries.rs` | `test_f3_atomic_upsert_first_toggle_to_true`, `test_boundary_rapid_toggle_state_integrity` |
| **F8** | 30-Day Completion Rate Rollups | `test_tier1_features.rs`, `test_tier3_interactions.rs` | `test_f4_30_day_completion_stats_daily_aggregation_accuracy`, `test_interaction_habit_completion_and_30_day_stats_percentage` |
| **F9** | Per-Habit 30-Day Success Rate | `test_tier1_features.rs`, `test_tier4_workloads.rs` | `test_f4_habit_success_rate_30_days_partial_adherence`, `test_workload_30_day_habit_challenge` |
| **F10** | Monthly Heatmap Query | `test_tier1_features.rs`, `test_tier3_interactions.rs` | `test_f4_monthly_heatmap_month_boundary_strictness`, `test_interaction_monthly_heatmap_across_month_boundary` |
| **F11** | To-Dos CRUD & Timestamping | `test_tier1_features.rs`, `test_tier3_interactions.rs` | `test_f5_todo_toggle_completion_sets_and_clears_completed_at`, `test_interaction_todo_lifecycle_states_and_ordering` |
| **F12** | Habits Reordering & Archiving | `test_tier1_features.rs`, `test_tier3_interactions.rs` | `test_f6_habits_reorder_up_and_down_swaps_positions`, `test_interaction_habit_creation_toggling_archiving_and_analytics` |
| **F13** | Full JSON Data Export & Import | `test_tier1_features.rs`, `test_tier4_workloads.rs` | `test_f7_export_captures_all_tables_completely`, `test_workload_multi_device_sync_and_portability` |
