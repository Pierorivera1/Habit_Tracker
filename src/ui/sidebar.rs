//! Collapsible Metrics Sidebar component.

use crate::app::message::Message;
use crate::app::state::AppState;
use crate::domain::DailyStats;
use crate::ui::theme;
use chrono::{Datelike, NaiveDate};
use iced::alignment::Horizontal;
use iced::border::Radius;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::widget::{column, container, progress_bar, row, scrollable, text, Space};
use iced::{mouse, Alignment, Border, Element, Length, Point, Rectangle, Renderer, Theme};

/// Collapsible Metrics Sidebar (340px width).
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let mut cards = column![]
        .spacing(16)
        .padding(16)
        .width(Length::Fixed(340.0));

    // -------------------------------------------------------------------------
    // CARD 1: Today's Progress / Completion %
    // -------------------------------------------------------------------------
    let total_habits = state.habits.len();
    let completed_habits = state.habits.iter().filter(|(_, c)| *c).count();
    let percentage = if total_habits == 0 {
        0.0
    } else {
        (completed_habits as f32 / total_habits as f32) * 100.0
    };

    let pct_label = format!("{:.0}%", percentage);
    let count_label = if total_habits == 0 {
        "No active habits".to_string()
    } else {
        format!("{} of {} habits completed", completed_habits, total_habits)
    };

    let card_progress = container(
        column![
            text("Today's Progress")
                .size(13)
                .color(theme::TEXT_SECONDARY),
            Space::with_height(4),
            text(pct_label).size(36).color(theme::TEXT_PRIMARY),
            text(count_label).size(12).color(theme::TEXT_MUTED),
            Space::with_height(8),
            progress_bar(0.0..=100.0, percentage)
                .style(theme::custom_progress_bar)
                .height(10.0),
        ]
        .width(Length::Fill),
    )
    .style(theme::card_container)
    .padding(16)
    .width(Length::Fill);

    cards = cards.push(card_progress);

    // -------------------------------------------------------------------------
    // CARD 2: 14/30-Day Trend Chart (Canvas)
    // -------------------------------------------------------------------------
    let chart_title = text("30-Day Completion Trend")
        .size(13)
        .color(theme::TEXT_SECONDARY);

    let chart_canvas = Canvas::new(TrendChart {
        stats: &state.daily_stats_30_days,
        selected_date: state.selected_date,
    })
    .width(Length::Fixed(300.0))
    .height(Length::Fixed(130.0));

    let card_trend =
        container(column![chart_title, Space::with_height(8), chart_canvas,].width(Length::Fill))
            .style(theme::card_container)
            .padding(16)
            .width(Length::Fill);

    cards = cards.push(card_trend);

    // -------------------------------------------------------------------------
    // CARD 3: Monthly Heatmap Grid
    // -------------------------------------------------------------------------
    let month_name = state.selected_date.format("%B %Y").to_string();
    let heatmap_title = text(format!("Consistency — {}", month_name))
        .size(13)
        .color(theme::TEXT_SECONDARY);

    let heatmap_grid = render_monthly_heatmap(state);

    let card_heatmap =
        container(column![heatmap_title, Space::with_height(8), heatmap_grid,].width(Length::Fill))
            .style(theme::card_container)
            .padding(16)
            .width(Length::Fill);

    cards = cards.push(card_heatmap);

    // -------------------------------------------------------------------------
    // CARD 4: Per-Habit Success Rate (30 Days)
    // -------------------------------------------------------------------------
    let success_rate_title = text("30-Day Habit Breakdown")
        .size(13)
        .color(theme::TEXT_SECONDARY);

    let mut habit_breakdown_col = column![success_rate_title].spacing(8).width(Length::Fill);

    if state.habits.is_empty() {
        habit_breakdown_col = habit_breakdown_col.push(
            text("No habits configured")
                .size(12)
                .color(theme::TEXT_MUTED),
        );
    } else {
        for (habit, _) in &state.habits {
            let rate = state
                .habit_success_rates
                .iter()
                .find(|(id, _)| *id == habit.id)
                .map(|(_, r)| *r)
                .unwrap_or(0.0);

            let habit_row = column![
                row![
                    text(&habit.name).size(13).color(theme::TEXT_PRIMARY),
                    Space::with_width(Length::Fill),
                    text(format!("{:.0}%", rate))
                        .size(12)
                        .color(theme::TEXT_SECONDARY),
                ]
                .align_y(Alignment::Center),
                Space::with_height(2),
                progress_bar(0.0..=100.0, rate)
                    .style(theme::custom_progress_bar)
                    .height(6.0),
            ]
            .width(Length::Fill);

            habit_breakdown_col = habit_breakdown_col.push(habit_row);
        }
    }

    let card_breakdown = container(habit_breakdown_col)
        .style(theme::card_container)
        .padding(16)
        .width(Length::Fill);

    cards = cards.push(card_breakdown);

    container(scrollable(cards).height(Length::Fill))
        .width(Length::Fixed(340.0))
        .height(Length::Fill)
        .style(theme::sidebar_container)
        .into()
}

// -----------------------------------------------------------------------------
// Canvas Trend Chart Program
// -----------------------------------------------------------------------------

/// Custom Canvas Program for rendering historical habit completion rates.
pub struct TrendChart<'a> {
    pub stats: &'a [DailyStats],
    pub selected_date: NaiveDate,
}

impl<'a> canvas::Program<Message, Theme, Renderer> for TrendChart<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let pad_left = 32.0;
        let pad_right = 10.0;
        let pad_top = 10.0;
        let pad_bottom = 20.0;

        let width = bounds.width - pad_left - pad_right;
        let height = bounds.height - pad_top - pad_bottom;

        if width <= 0.0 || height <= 0.0 {
            return vec![frame.into_geometry()];
        }

        // 1. Draw horizontal gridlines at 0%, 50%, 100%
        let grid_levels = [0.0, 50.0, 100.0];
        let grid_stroke = Stroke::default()
            .with_color(theme::BORDER_SUBTLE)
            .with_width(1.0);

        for level in &grid_levels {
            let y = pad_top + height - (level / 100.0 * height);

            let line = Path::line(Point::new(pad_left, y), Point::new(pad_left + width, y));
            frame.stroke(&line, grid_stroke);

            // Label
            let label = canvas::Text {
                content: format!("{:.0}%", level),
                position: Point::new(4.0, y - 6.0),
                color: theme::TEXT_MUTED,
                size: iced::Pixels(10.0),
                ..canvas::Text::default()
            };
            frame.fill_text(label);
        }

        // 2. Draw trend line and data points
        if !self.stats.is_empty() {
            let n = self.stats.len();
            let mut points = Vec::with_capacity(n);

            for (idx, stat) in self.stats.iter().enumerate() {
                let x = if n == 1 {
                    pad_left + width / 2.0
                } else {
                    pad_left + (idx as f32 / (n - 1) as f32) * width
                };

                let pct = stat.percentage().clamp(0.0, 100.0);
                let y = pad_top + height - (pct / 100.0 * height);

                points.push((Point::new(x, y), stat.date));
            }

            // Draw line path connecting points
            if points.len() > 1 {
                let trend_path = Path::new(|builder| {
                    builder.move_to(points[0].0);
                    for (pt, _) in &points[1..] {
                        builder.line_to(*pt);
                    }
                });

                let line_stroke = Stroke::default()
                    .with_color(theme::ACCENT_BLUE)
                    .with_width(2.0);
                frame.stroke(&trend_path, line_stroke);
            }

            // Draw data point dots
            for (pt, date) in &points {
                let is_selected = *date == self.selected_date;

                if is_selected {
                    // Outer cyan halo
                    frame.fill(&Path::circle(*pt, 5.5), theme::ACCENT_CYAN);
                    frame.fill(&Path::circle(*pt, 3.5), theme::BG_MAIN);
                } else {
                    frame.fill(&Path::circle(*pt, 2.5), theme::ACCENT_BLUE);
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

// -----------------------------------------------------------------------------
// Monthly Heatmap Grid
// -----------------------------------------------------------------------------

/// Renders a 7-column calendar heatmap grid for the active month.
fn render_monthly_heatmap<'a>(state: &'a AppState) -> Element<'a, Message> {
    let year = state.selected_date.year();
    let month = state.selected_date.month();

    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(state.selected_date);
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    let days_in_month = match next_month_date {
        Some(next) => (next - first_day).num_days() as u32,
        None => 31,
    };

    // Day of week: Monday is 0, Sunday is 6
    let start_offset = first_day.weekday().num_days_from_monday();

    // 1. Header row: Mon Tue Wed Thu Fri Sat Sun
    let day_headers = ["M", "T", "W", "T", "F", "S", "S"];
    let mut header_row = row![].spacing(4);
    for h in &day_headers {
        header_row = header_row.push(
            container(
                text(*h)
                    .size(11)
                    .color(theme::TEXT_MUTED)
                    .align_x(Horizontal::Center),
            )
            .width(Length::Fixed(36.0)),
        );
    }

    let mut grid_col = column![header_row].spacing(4);

    // 2. Build rows of 7 days
    let mut current_day = 1u32;
    let mut row_cells = row![].spacing(4);

    // Empty leading padding cells for the first week
    for _ in 0..start_offset {
        row_cells = row_cells
            .push(container(Space::with_width(Length::Fixed(36.0))).height(Length::Fixed(24.0)));
    }

    let mut current_weekday = start_offset;

    let total_habits = state.habits.len();

    while current_day <= days_in_month {
        let cell_date = NaiveDate::from_ymd_opt(year, month, current_day).unwrap_or(first_day);
        let completion_count = state.monthly_heatmap.get(&cell_date).copied().unwrap_or(0);

        let cell_color = if completion_count == 0 {
            theme::HEATMAP_EMPTY
        } else if total_habits == 0 {
            theme::HEATMAP_L1
        } else {
            let ratio = completion_count as f32 / total_habits as f32;
            if ratio >= 1.0 {
                theme::HEATMAP_L4
            } else if ratio >= 0.75 {
                theme::HEATMAP_L3
            } else if ratio >= 0.50 {
                theme::HEATMAP_L2
            } else {
                theme::HEATMAP_L1
            }
        };

        let is_selected = cell_date == state.selected_date;

        let cell_text = text(format!("{}", current_day))
            .size(10)
            .color(if completion_count > 0 {
                theme::BG_MAIN
            } else {
                theme::TEXT_MUTED
            });

        let cell_border = if is_selected {
            Border {
                color: theme::ACCENT_CYAN,
                width: 2.0,
                radius: Radius::from(4.0),
            }
        } else {
            Border {
                color: theme::BORDER_SUBTLE,
                width: 0.5,
                radius: Radius::from(4.0),
            }
        };

        let cell = container(cell_text)
            .width(Length::Fixed(36.0))
            .height(Length::Fixed(24.0))
            .align_x(Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(cell_color)),
                text_color: Some(theme::TEXT_PRIMARY),
                border: cell_border,
                shadow: iced::Shadow::default(),
            });

        row_cells = row_cells.push(cell);
        current_weekday += 1;
        current_day += 1;

        if current_weekday == 7 {
            grid_col = grid_col.push(row_cells);
            row_cells = row![].spacing(4);
            current_weekday = 0;
        }
    }

    // Trailing padding cells if final row is not complete
    if current_weekday > 0 {
        while current_weekday < 7 {
            row_cells = row_cells.push(
                container(Space::with_width(Length::Fixed(36.0))).height(Length::Fixed(24.0)),
            );
            current_weekday += 1;
        }
        grid_col = grid_col.push(row_cells);
    }

    grid_col.into()
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::{DailyStats, Habit};
    use chrono::NaiveDate;

    fn setup_test_state() -> AppState {
        let db = Database::new_in_memory().unwrap();
        AppState::new(db).unwrap()
    }

    #[test]
    fn test_sidebar_view_empty_state_zero_division_guard() {
        let state = setup_test_state();
        assert_eq!(state.habits.len(), 0);
        let _elem = view(&state);
    }

    #[test]
    fn test_sidebar_view_populated_state() {
        let mut state = setup_test_state();
        state.habits = vec![
            (
                Habit {
                    id: 1,
                    name: "Exercise".to_string(),
                    position: 1,
                    created_at: "2026-09-01".to_string(),
                    archived: false,
                },
                true,
            ),
            (
                Habit {
                    id: 2,
                    name: "Meditate".to_string(),
                    position: 2,
                    created_at: "2026-09-01".to_string(),
                    archived: false,
                },
                false,
            ),
        ];

        state.habit_success_rates = vec![(1, 85.0), (2, 40.0)];

        state.daily_stats_30_days = (0..30)
            .map(|i| DailyStats {
                date: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                total_habits: 2,
                completed_habits: (i % 3 == 0) as usize + 1,
            })
            .collect();

        let _elem = view(&state);
    }

    #[test]
    fn test_heatmap_month_boundaries_and_leap_years() {
        let mut state = setup_test_state();

        // Leap year: Feb 2024 (29 days)
        {
            state.selected_date = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
            let _grid = render_monthly_heatmap(&state);
        }

        // Non-leap year: Feb 2023 (28 days)
        {
            state.selected_date = NaiveDate::from_ymd_opt(2023, 2, 10).unwrap();
            let _grid = render_monthly_heatmap(&state);
        }

        // 31-day month: Dec 2026
        {
            state.selected_date = NaiveDate::from_ymd_opt(2026, 12, 25).unwrap();
            let _grid = render_monthly_heatmap(&state);
        }

        // 30-day month: Sep 2026
        {
            state.selected_date = NaiveDate::from_ymd_opt(2026, 9, 23).unwrap();
            let _grid = render_monthly_heatmap(&state);
        }
    }

    #[test]
    fn test_trend_chart_struct() {
        let empty_stats: Vec<DailyStats> = Vec::new();
        let chart = TrendChart {
            stats: &empty_stats,
            selected_date: NaiveDate::from_ymd_opt(2026, 9, 23).unwrap(),
        };
        assert_eq!(chart.stats.len(), 0);

        let single_stat = vec![DailyStats {
            date: NaiveDate::from_ymd_opt(2026, 9, 23).unwrap(),
            total_habits: 5,
            completed_habits: 4,
        }];
        let chart_single = TrendChart {
            stats: &single_stat,
            selected_date: NaiveDate::from_ymd_opt(2026, 9, 23).unwrap(),
        };
        assert_eq!(chart_single.stats.len(), 1);
    }
}
