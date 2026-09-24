# Project: Habitodo (HabitTracker)

## Architecture
Habitodo is a native desktop habit and to-do tracking application implemented in Rust using `iced` (0.13.1) and SQLite via `rusqlite` (0.32 bundled).
The project follows Elm Architecture principles (unidirectional data flow: Model-View-Update) with a strictly decoupled four-tier structure ensuring 100% headless testability:

1. **Domain & Persistence Layer (`src/domain/`, `src/db/`)**:
   - Pure domain data structures (`Habit`, `Todo`, `HabitCompletion`, `DailyStats`, `ExportData`) with serde serialization.
   - SQLite persistence engine with WAL mode, foreign keys, schema initialization, and high-performance SQL query catalog (LEFT JOIN for daily habit status, atomic UPSERT for toggling, 30-day stats aggregation, JSON export/import).
2. **State & Keybinding Layer (`src/app/`, `src/keybindings/`)**:
   - Central application state (`AppState`, `Screen`, `InputMode`) and typed `Message` enum.
   - Write-through persistence on every mutation; non-blocking database errors routed via `Message::DbError(String)`.
   - Pure keybinding mapper translating raw iced key events into domain `Message` actions while strictly isolating text input focus.
3. **Presentation Layer (`src/ui/`)**:
   - Dark mode design system (`#0f0f0f` background, `#1a1a1a` cards, `#2d2d2d` borders, `#f3f4f6`/`#9ca3af` text, `#4ade80`/`#60a5fa` accents).
   - Component tree: Top bar, Main content (Daily Habits list with completed styles & To-Dos list), Collapsible Metrics Sidebar (Today %, 14/30-day Canvas trend line chart, monthly heatmap grid, per-habit success rates), Settings screen (habit reordering via J/K, archiving, renaming, JSON export), and dynamic bottom shortcut bar.
4. **CLI & Execution Harness (`src/main.rs`)**:
   - Command-line argument handling (`--help`, `--version`, `--verify`).
   - Self-verification diagnostics runner executing against an in-memory SQLite database before GUI initialization.

---

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | SQLite Engine & WAL Mode | SQLite initialization at `~/.local/share/habitodo/habitodo.db` with `PRAGMA journal_mode = WAL` and foreign keys | M1 | R1, DB Spec §1 |
| 2 | Full v1 DDL Schema & Indexes | Tables `habits`, `habit_completions`, `todos`, `settings` and indexes `idx_completions_date`, `idx_todos_completed`, `idx_todos_due` | M1 | R1, DB Spec §2 |
| 3 | In-Memory SQLite Support | Headless and isolated test database support via `Database::new_in_memory()` | M1 | AC, Env Report §2.2 |
| 4 | Domain Models & Serde | Strongly-typed structs `Habit`, `HabitCompletion`, `Todo`, `DailyStats`, `ExportData` with serde derive and defaults | M1 | R2, Data Model §1 |
| 5 | DailyStats Percentage Method | `DailyStats::percentage(&self) -> f32` with safe handling of zero habits (returns `0.0`) | M1 | R2, Data Model §2 |
| 6 | LEFT JOIN Daily Habits Query | Query daily habits with `LEFT JOIN` and `COALESCE(c.completed, 0)` defaulting unrecorded habits to false | M1 | R1, AC, DB Spec §3 |
| 7 | Atomic UPSERT Habit Toggle | Atomic toggling on composite key `(habit_id, date)` inserting or inverting `completed` | M1 | R1, AC, DB Spec §3 |
| 8 | 30-Day Completion Rate Rollups | SQL group-by query computing daily completion percentages for past 30 days | M1 | R1, AC, DB Spec §3 |
| 9 | Per-Habit 30-Day Success Rate | Query computing individual habit consistency percentages over 30 days | M1 | R3, DB Spec §3 |
| 10 | Monthly Heatmap Query | Query aggregating completion counts grouped by date for current month | M1 | R3, DB Spec §3 |
| 11 | To-Dos CRUD & Timestamping | Complete To-Do management (add, toggle completion with `completed_at`, edit notes, reorder, delete) | M1 | R1, DB Spec §3 |
| 12 | Habits Reordering & Archiving | Habits management: soft-delete archiving (`archived = 1`) and transactional position swapping | M1 | R3, R4, DB Spec §3 |
| 13 | Full JSON Data Export & Import | Exporting and importing entire database state to/from JSON schema | M1 | R1, AC, DB Spec §4 |
| 14 | Central Elm App State | State struct managing `habits`, `todos`, `selected_date`, `selected_index`, `screen`, `sidebar_open`, `input_mode` | M2 | R2, Arch Spec §1 |
| 15 | Typed Message Enum | Exhaustive message enum covering navigation, habit actions, todo actions, settings, and db lifecycle | M2 | R2, Arch Spec §2 |
| 16 | Write-Through Persistence | Immediate SQLite write-through on state transitions in `App::update` | M2 | Arch Spec §3 |
| 17 | Non-Blocking DbError Handling | Capturing all SQLite errors into `Message::DbError(String)` and displaying as status/toast without panic | M2 | R2, AC, Arch Spec §4 |
| 18 | Vim Navigation Keybindings | `j`/`k` (move up/down), `h`/`l` (prev/next day), `t` (today), `g`/`G` (top/bottom) | M2 | R4, Keybindings Spec §2 |
| 19 | Action Keybindings | `Space` (toggle item), `x` (complete/dismiss), `a` (add), `i`/`Enter` (edit), `d` (delete with confirm), `n` (notes) | M2 | R4, Keybindings Spec §3 |
| 20 | Settings Keybindings | `j`/`k` (navigate), `a` (add habit), `d` (archive), `r` (rename), `J`/`K` (reorder), `e` (export JSON), `Esc` (return) | M2 | R4, Keybindings Spec §4 |
| 21 | Text Input Focus Isolation | Disabling single-key shortcuts while typing in inline `TextInput` fields; restoring on `Enter`/`Esc` | M2 | Keybindings Spec §5, UI Spec §7 |
| 22 | Dark Mode Styling Tokens | Custom theme palette (`#0f0f0f` canvas, `#1a1a1a` cards, `#2d2d2d` borders, high contrast text, green/blue accents) | M3 | R3, UI Spec §2 |
| 23 | Top Bar Component | Application title, active date display, quick settings toggle button | M3 | R3, UI Spec §3 |
| 24 | Main Content View | Two distinct lists: Non-negotiable Daily Habits (with strikethrough/opacity completion states) and To-Dos | M3 | R3, UI Spec §4 |
| 25 | Selection Highlight Cursor | Visual indicator (cyan left border `#38bdf8` and background elevation `#222222`) for active row selection | M3 | R3, UI Spec §4 |
| 26 | Collapsible Metrics Sidebar | 340px sidebar, toggleable via `s` or `Tab`, auto-collapse under 900px | M3 | R3, UI Spec §5 |
| 27 | Today's Completion Widget | Big numeric percentage display with styled progress bar | M3 | R3, UI Spec §5 |
| 28 | 14/30-Day Trend Chart Canvas | Custom Canvas `Program` drawing historical completion rate trend curve with gridlines and data points | M3 | R3, UI Spec §5 |
| 29 | Monthly Heatmap Grid Widget | 7-column calendar grid showing habit completion intensity with tiered opacity | M3 | R3, UI Spec §5 |
| 30 | Per-Habit Success Rate Table | Compact list of active habits with 30-day percentage completion bars | M3 | R3, UI Spec §5 |
| 31 | Settings View Component | Dedicated screen for habit reordering (J/K), adding, archiving, renaming, and JSON export button | M3 | R3, UI Spec §6 |
| 32 | Bottom Shortcut Bar | Dynamic contextual footer showing active available keybindings | M3 | R3, UI Spec §8 |
| 33 | Modal & Confirmation Dialogs | Modals for Add To-Do, Add Habit, and `[y/n]` deletion confirmation | M3 | UI Spec §7 |
| 34 | CLI Flags (`--help`, `--version`) | Command-line flags displaying usage and version info before GUI launch | M4 | AC, Env Report §1.4 |
| 35 | CLI Self-Verification (`--verify`) | Headless self-diagnostics flag testing schema, queries, serde, and reporting status | M4 | AC, Env Report §1.4 |
| 36 | Application Startup & Zero-Panic | Clean startup on missing/empty DB files without crashing or panicking | M4 | AC, Arch Spec §4 |
| 37 | Automated Test Suite (Tiers 1-4) | Comprehensive test suite covering schema, left join, upsert, rollups, serde, transitions | M5 | AC, Test Spec |
| 38 | Adversarial Coverage Hardening | White-box stress testing, boundary fuzzing, and invariant coverage (Tier 5) | M5 | Dual Track Phase 2 |

---

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Foundation & Database Layer | `Cargo.toml`, domain models, SQLite engine, schema init, queries, JSON export/import | none | DONE |
| M2 | State Machine & Keybindings | `AppState`, `Message`, write-through transitions, non-blocking DbError, key mapping & focus isolation | M1 | DONE |
| M3 | UI Design System & Widgets | Dark theme, Top bar, Habit/Todo lists, Collapsible Sidebar, Canvas trend chart, Heatmap, Settings, Shortcut bar | M2 | DONE |
| M4 | CLI & Application Integration | `main.rs`, CLI flags (`--help`, `--version`, `--verify`), zero-panic startup, Wayland/X11 entry point | M3 | DONE |
| M5 | E2E Verification & Hardening | Pass 100% of E2E test suite (`TEST_READY.md`) and adversarial coverage hardening (Tier 5) | M4, E2E Track | DONE |

---

## Interface Contracts

### `src/domain` ↔ `src/db`
```rust
pub struct Habit {
    pub id: i64,
    pub name: String,
    pub position: i64,
    pub created_at: String,
    pub archived: bool,
}

pub struct HabitCompletion {
    pub habit_id: i64,
    pub date: chrono::NaiveDate,
    pub completed: bool,
}

pub struct Todo {
    pub id: i64,
    pub title: String,
    pub notes: Option<String>,
    pub completed: bool,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub position: i64,
}

pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub total_habits: usize,
    pub completed_habits: usize,
}
impl DailyStats {
    pub fn percentage(&self) -> f32 {
        if self.total_habits == 0 { 0.0 } else { (self.completed_habits as f32 / self.total_habits as f32) * 100.0 }
    }
}

pub struct ExportData {
    pub version: u32,
    pub exported_at: String,
    pub habits: Vec<Habit>,
    pub habit_completions: Vec<HabitCompletion>,
    pub todos: Vec<Todo>,
    pub settings: std::collections::HashMap<String, String>,
}

pub struct Database {
    conn: rusqlite::Connection,
}
impl Database {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> rusqlite::Result<Self>;
    pub fn new_in_memory() -> rusqlite::Result<Self>;
    pub fn init_schema(&self) -> rusqlite::Result<()>;
    pub fn get_habits_for_date(&self, date: chrono::NaiveDate) -> rusqlite::Result<Vec<(Habit, bool)>>;
    pub fn toggle_habit_completion(&self, habit_id: i64, date: chrono::NaiveDate) -> rusqlite::Result<bool>;
    pub fn get_30_day_completion_stats(&self, end_date: chrono::NaiveDate) -> rusqlite::Result<Vec<DailyStats>>;
    pub fn get_habit_success_rate_30_days(&self, habit_id: i64, end_date: chrono::NaiveDate) -> rusqlite::Result<f32>;
    pub fn get_monthly_heatmap(&self, year: i32, month: u32) -> rusqlite::Result<std::collections::HashMap<chrono::NaiveDate, usize>>;
    pub fn get_all_todos(&self) -> rusqlite::Result<Vec<Todo>>;
    pub fn add_todo(&self, title: &str, notes: Option<&str>, due_date: Option<chrono::NaiveDate>) -> rusqlite::Result<i64>;
    pub fn toggle_todo(&self, todo_id: i64) -> rusqlite::Result<bool>;
    pub fn delete_todo(&self, todo_id: i64) -> rusqlite::Result<()>;
    pub fn add_habit(&self, name: &str) -> rusqlite::Result<i64>;
    pub fn archive_habit(&self, habit_id: i64) -> rusqlite::Result<()>;
    pub fn rename_habit(&self, habit_id: i64, new_name: &str) -> rusqlite::Result<()>;
    pub fn reorder_habit(&self, habit_id: i64, move_up: bool) -> rusqlite::Result<()>;
    pub fn export_data(&self) -> rusqlite::Result<ExportData>;
    pub fn import_data(&self, data: &ExportData) -> rusqlite::Result<()>;
}
```

### `src/db` ↔ `src/app`
- Database errors return `rusqlite::Result<T>` and are handled gracefully inside `AppState::update`:
```rust
match self.db.toggle_habit_completion(habit_id, date) {
    Ok(new_status) => { /* update in-memory state */ }
    Err(e) => { self.status_message = Some(format!("Database error: {}", e)); }
}
```

### `src/app` ↔ `src/ui`
```rust
pub struct AppState {
    pub screen: Screen,
    pub sidebar_open: bool,
    pub selected_date: chrono::NaiveDate,
    pub selected_index: usize,
    pub habits: Vec<(Habit, bool)>,
    pub todos: Vec<Todo>,
    pub daily_stats_30_days: Vec<DailyStats>,
    pub habit_success_rates: Vec<(i64, f32)>,
    pub monthly_heatmap: std::collections::HashMap<chrono::NaiveDate, usize>,
    pub input_mode: InputMode,
    pub status_message: Option<String>,
    pub db: Database,
}

pub fn view<'a>(state: &'a AppState) -> iced::Element<'a, Message>;
```

---

## Code Layout
```
/home/pierooo/Projects/Habit_Tracker/
├── Cargo.toml
├── src/
│   ├── lib.rs             # Public module exports and library root
│   ├── main.rs            # Binary entry point, CLI flags, iced application runner
│   ├── domain/            # Strongly-typed domain models & serialization
│   │   └── mod.rs
│   ├── db/                # SQLite persistence engine
│   │   ├── mod.rs
│   │   ├── schema.rs      # DDL, migrations, index definitions
│   │   └── queries.rs     # SQL queries (LEFT JOIN, UPSERT, rollups, export/import)
│   ├── app/               # Elm architecture state machine
│   │   ├── mod.rs
│   │   ├── state.rs       # AppState struct and update transitions
│   │   └── message.rs     # Typed Message enum
│   ├── keybindings/       # Pure keyboard shortcut mapping & focus handling
│   │   └── mod.rs
│   └── ui/                # Iced user interface and dark theme
│       ├── mod.rs
│       ├── theme.rs       # Colors, palette tokens, custom styles
│       ├── top_bar.rs     # Top navigation bar
│       ├── main_view.rs   # Daily Habits & To-Dos lists
│       ├── sidebar.rs     # Metrics sidebar & Canvas trend line chart & Heatmap
│       ├── settings_view.rs # Habit management and JSON export screen
│       └── components.rs  # Contextual shortcut footer & modal dialogs
└── tests/                 # Opaque-box E2E test suite (Tiers 1-4)
    ├── common/
    │   └── mod.rs
    ├── test_tier1_features.rs
    ├── test_tier2_boundaries.rs
    ├── test_tier3_interactions.rs
    └── test_tier4_workloads.rs
```
