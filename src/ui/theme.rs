//! Dark mode palette tokens, colors, and iced widget styling helpers.

use iced::border::Radius;
use iced::widget::{button, container, progress_bar, text_input};
use iced::{Background, Border, Color, Shadow, Theme};

// -----------------------------------------------------------------------------
// Dark Mode Palette Tokens
// -----------------------------------------------------------------------------

/// Canvas / Window root background: `#0f0f0f`
pub const BG_MAIN: Color = Color::from_rgb(15.0 / 255.0, 15.0 / 255.0, 15.0 / 255.0);

/// Card / Panel surface background: `#1a1a1a`
pub const SURFACE_CARD: Color = Color::from_rgb(26.0 / 255.0, 26.0 / 255.0, 26.0 / 255.0);

/// Hovered or elevated card background: `#222222`
pub const SURFACE_HOVER: Color = Color::from_rgb(34.0 / 255.0, 34.0 / 255.0, 34.0 / 255.0);

/// Shortcut pill / badge background: `#262626`
pub const SURFACE_PILL: Color = Color::from_rgb(38.0 / 255.0, 38.0 / 255.0, 38.0 / 255.0);

/// Subtle container & card border: `#2d2d2d`
pub const BORDER_SUBTLE: Color = Color::from_rgb(45.0 / 255.0, 45.0 / 255.0, 45.0 / 255.0);

/// Pill badge border: `#3f3f46`
pub const BORDER_PILL: Color = Color::from_rgb(63.0 / 255.0, 63.0 / 255.0, 70.0 / 255.0);

/// Primary high-contrast text: `#f3f4f6`
pub const TEXT_PRIMARY: Color = Color::from_rgb(243.0 / 255.0, 244.0 / 255.0, 246.0 / 255.0);

/// Secondary muted text: `#9ca3af`
pub const TEXT_SECONDARY: Color = Color::from_rgb(156.0 / 255.0, 163.0 / 255.0, 175.0 / 255.0);

/// Muted label / grid text: `#4b5563`
pub const TEXT_MUTED: Color = Color::from_rgb(75.0 / 255.0, 85.0 / 255.0, 99.0 / 255.0);

/// Success green accent: `#4ade80`
pub const ACCENT_GREEN: Color = Color::from_rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0);

/// Secondary blue accent (trend chart, tags): `#60a5fa`
pub const ACCENT_BLUE: Color = Color::from_rgb(96.0 / 255.0, 165.0 / 255.0, 250.0 / 255.0);

/// Focus ring / cursor selection cyan: `#38bdf8`
pub const ACCENT_CYAN: Color = Color::from_rgb(56.0 / 255.0, 189.0 / 255.0, 248.0 / 255.0);

/// Danger red accent (deletion confirmation): `#ef4444`
pub const ACCENT_DANGER: Color = Color::from_rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0);

/// Monthly heatmap opacity tiers (based on #4ade80)
pub const HEATMAP_L1: Color = Color::from_rgba(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0, 0.25);
pub const HEATMAP_L2: Color = Color::from_rgba(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0, 0.55);
pub const HEATMAP_L3: Color = Color::from_rgba(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0, 0.85);
pub const HEATMAP_L4: Color = ACCENT_GREEN;
pub const HEATMAP_EMPTY: Color = SURFACE_HOVER;

// -----------------------------------------------------------------------------
// Container Styles
// -----------------------------------------------------------------------------

/// Root window style: solid #0f0f0f background
pub fn root_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_MAIN)),
        text_color: Some(TEXT_PRIMARY),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

/// Standard card container style: #1a1a1a background, #2d2d2d border, 8px radius
pub fn card_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_CARD)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: Shadow::default(),
    }
}

/// Selected item card style: elevated #222222 background, 3px cyan border
pub fn selected_card_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_HOVER)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: ACCENT_CYAN,
            width: 3.0,
            radius: Radius::from(8.0),
        },
        shadow: Shadow::default(),
    }
}

/// Top bar container: #0f0f0f background with bottom border #2d2d2d
pub fn top_bar_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_MAIN)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        shadow: Shadow::default(),
    }
}

/// Metrics sidebar container: #1a1a1a background with left border #2d2d2d
pub fn sidebar_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_CARD)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        shadow: Shadow::default(),
    }
}

/// Bottom shortcut bar container: #0f0f0f background with top border #2d2d2d
pub fn bottom_bar_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_MAIN)),
        text_color: Some(TEXT_SECONDARY),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        shadow: Shadow::default(),
    }
}

/// Shortcut key badge pill container: #262626 background, #3f3f46 border, 4px radius
pub fn pill_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_PILL)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_PILL,
            width: 1.0,
            radius: Radius::from(4.0),
        },
        shadow: Shadow::default(),
    }
}

/// Modal backdrop style: semi-transparent dark overlay
pub fn modal_backdrop(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
        text_color: Some(TEXT_PRIMARY),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

/// Modal card container style: #1a1a1a with #38bdf8 cyan border
pub fn modal_card_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_CARD)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: ACCENT_CYAN,
            width: 1.5,
            radius: Radius::from(10.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 16.0,
        },
    }
}

// -----------------------------------------------------------------------------
// Button Styles
// -----------------------------------------------------------------------------

/// Primary button style
pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base_bg = match status {
        button::Status::Hovered => Color::from_rgb(86.0 / 255.0, 232.0 / 255.0, 138.0 / 255.0),
        button::Status::Pressed => Color::from_rgb(64.0 / 255.0, 212.0 / 255.0, 118.0 / 255.0),
        button::Status::Disabled => Color::from_rgb(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0),
        button::Status::Active => ACCENT_GREEN,
    };

    button::Style {
        background: Some(Background::Color(base_bg)),
        text_color: BG_MAIN,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow::default(),
    }
}

/// Secondary / Ghost button style
pub fn secondary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, border_color) = match status {
        button::Status::Hovered => (Some(Background::Color(SURFACE_HOVER)), ACCENT_CYAN),
        button::Status::Pressed => (Some(Background::Color(SURFACE_PILL)), ACCENT_BLUE),
        button::Status::Disabled => (None, Color::TRANSPARENT),
        button::Status::Active => (Some(Background::Color(SURFACE_CARD)), BORDER_SUBTLE),
    };

    button::Style {
        background: bg,
        text_color: TEXT_PRIMARY,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow::default(),
    }
}

/// Danger button style (for delete confirmation)
pub fn danger_button(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb(245.0 / 255.0, 85.0 / 255.0, 85.0 / 255.0),
        button::Status::Pressed => Color::from_rgb(220.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0),
        button::Status::Disabled => Color::from_rgb(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0),
        button::Status::Active => ACCENT_DANGER,
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow::default(),
    }
}

// -----------------------------------------------------------------------------
// TextInput Styles
// -----------------------------------------------------------------------------

/// Custom text input style matching dark theme
pub fn custom_text_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused => ACCENT_CYAN,
        text_input::Status::Hovered => TEXT_MUTED,
        text_input::Status::Disabled => BORDER_SUBTLE,
        text_input::Status::Active => BORDER_SUBTLE,
    };

    text_input::Style {
        background: Background::Color(SURFACE_CARD),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        icon: TEXT_SECONDARY,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: ACCENT_BLUE,
    }
}

// -----------------------------------------------------------------------------
// Checkbox & Rule Styles
// -----------------------------------------------------------------------------

use iced::widget::{checkbox, rule};

/// Custom checkbox style matching dark theme
pub fn custom_checkbox(_theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let (bg, icon_color, border_color) = match status {
        checkbox::Status::Active { is_checked } => {
            if is_checked {
                (Background::Color(ACCENT_GREEN), BG_MAIN, ACCENT_GREEN)
            } else {
                (
                    Background::Color(SURFACE_CARD),
                    Color::TRANSPARENT,
                    BORDER_PILL,
                )
            }
        }
        checkbox::Status::Hovered { is_checked } => {
            if is_checked {
                (Background::Color(ACCENT_GREEN), BG_MAIN, ACCENT_GREEN)
            } else {
                (
                    Background::Color(SURFACE_HOVER),
                    Color::TRANSPARENT,
                    ACCENT_CYAN,
                )
            }
        }
        checkbox::Status::Disabled { is_checked } => {
            if is_checked {
                (Background::Color(BORDER_PILL), TEXT_MUTED, BORDER_SUBTLE)
            } else {
                (
                    Background::Color(BG_MAIN),
                    Color::TRANSPARENT,
                    BORDER_SUBTLE,
                )
            }
        }
    };

    checkbox::Style {
        background: bg,
        icon_color,
        border: Border {
            color: border_color,
            width: 1.5,
            radius: Radius::from(4.0),
        },
        text_color: Some(TEXT_PRIMARY),
    }
}

/// Horizontal divider rule style: 1px #2d2d2d
pub fn custom_rule(_theme: &Theme) -> rule::Style {
    rule::Style {
        color: BORDER_SUBTLE,
        width: 1,
        radius: Radius::from(0.0),
        fill_mode: rule::FillMode::Full,
    }
}

// -----------------------------------------------------------------------------
// ProgressBar Styles
// -----------------------------------------------------------------------------

/// Styled progress bar: track #2d2d2d, fill #4ade80
pub fn custom_progress_bar(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(BORDER_SUBTLE),
        bar: Background::Color(ACCENT_GREEN),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(5.0),
        },
    }
}

/// Danger badge container style: semi-transparent red background
pub fn danger_badge_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            239.0 / 255.0,
            68.0 / 255.0,
            68.0 / 255.0,
            0.2,
        ))),
        text_color: Some(ACCENT_DANGER),
        border: Border {
            color: ACCENT_DANGER,
            width: 1.0,
            radius: Radius::from(4.0),
        },
        shadow: Shadow::default(),
    }
}

/// Accent blue badge container style
pub fn blue_badge_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            96.0 / 255.0,
            165.0 / 255.0,
            250.0 / 255.0,
            0.2,
        ))),
        text_color: Some(ACCENT_BLUE),
        border: Border {
            color: ACCENT_BLUE,
            width: 1.0,
            radius: Radius::from(4.0),
        },
        shadow: Shadow::default(),
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_tokens_rgb_values() {
        // Main Background: #0f0f0f
        assert!((BG_MAIN.r - 15.0 / 255.0).abs() < 1e-4);
        assert!((BG_MAIN.g - 15.0 / 255.0).abs() < 1e-4);
        assert!((BG_MAIN.b - 15.0 / 255.0).abs() < 1e-4);

        // Surface Card: #1a1a1a
        assert!((SURFACE_CARD.r - 26.0 / 255.0).abs() < 1e-4);
        assert!((SURFACE_CARD.g - 26.0 / 255.0).abs() < 1e-4);
        assert!((SURFACE_CARD.b - 26.0 / 255.0).abs() < 1e-4);

        // Surface Hover: #222222
        assert!((SURFACE_HOVER.r - 34.0 / 255.0).abs() < 1e-4);

        // Border Subtle: #2d2d2d
        assert!((BORDER_SUBTLE.r - 45.0 / 255.0).abs() < 1e-4);

        // Primary Text: #f3f4f6
        assert!((TEXT_PRIMARY.r - 243.0 / 255.0).abs() < 1e-4);

        // Secondary Text: #9ca3af
        assert!((TEXT_SECONDARY.r - 156.0 / 255.0).abs() < 1e-4);

        // Accent Green: #4ade80
        assert!((ACCENT_GREEN.r - 74.0 / 255.0).abs() < 1e-4);
        assert!((ACCENT_GREEN.g - 222.0 / 255.0).abs() < 1e-4);
        assert!((ACCENT_GREEN.b - 128.0 / 255.0).abs() < 1e-4);

        // Accent Blue: #60a5fa
        assert!((ACCENT_BLUE.r - 96.0 / 255.0).abs() < 1e-4);
        assert!((ACCENT_BLUE.g - 165.0 / 255.0).abs() < 1e-4);
        assert!((ACCENT_BLUE.b - 250.0 / 255.0).abs() < 1e-4);

        // Accent Cyan: #38bdf8
        assert!((ACCENT_CYAN.r - 56.0 / 255.0).abs() < 1e-4);
        assert!((ACCENT_CYAN.g - 189.0 / 255.0).abs() < 1e-4);
        assert!((ACCENT_CYAN.b - 248.0 / 255.0).abs() < 1e-4);

        // Accent Danger: #ef4444
        assert!((ACCENT_DANGER.r - 239.0 / 255.0).abs() < 1e-4);
    }

    #[test]
    fn test_heatmap_tier_opacities() {
        assert!((HEATMAP_L1.a - 0.25).abs() < 1e-3);
        assert!((HEATMAP_L2.a - 0.55).abs() < 1e-3);
        assert!((HEATMAP_L3.a - 0.85).abs() < 1e-3);
        assert!((HEATMAP_L4.a - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_container_styles() {
        let theme = Theme::Dark;

        let root = root_container(&theme);
        assert!(root.background.is_some());

        let card = card_container(&theme);
        assert_eq!(card.border.color, BORDER_SUBTLE);
        assert_eq!(card.border.width, 1.0);

        let selected = selected_card_container(&theme);
        assert_eq!(selected.border.color, ACCENT_CYAN);
        assert_eq!(selected.border.width, 3.0);

        let top_bar = top_bar_container(&theme);
        assert_eq!(top_bar.border.color, BORDER_SUBTLE);

        let sidebar = sidebar_container(&theme);
        assert_eq!(sidebar.border.color, BORDER_SUBTLE);

        let bottom_bar = bottom_bar_container(&theme);
        assert_eq!(bottom_bar.border.color, BORDER_SUBTLE);

        let pill = pill_container(&theme);
        assert_eq!(pill.border.color, BORDER_PILL);

        let modal_back = modal_backdrop(&theme);
        assert!(modal_back.background.is_some());

        let modal_card = modal_card_container(&theme);
        assert_eq!(modal_card.border.color, ACCENT_CYAN);
        assert_eq!(modal_card.border.width, 1.5);
    }

    #[test]
    fn test_button_styles() {
        let theme = Theme::Dark;

        let primary_active = primary_button(&theme, button::Status::Active);
        assert_eq!(
            primary_active.background,
            Some(Background::Color(ACCENT_GREEN))
        );

        let sec_active = secondary_button(&theme, button::Status::Active);
        assert_eq!(sec_active.border.color, BORDER_SUBTLE);

        let sec_hovered = secondary_button(&theme, button::Status::Hovered);
        assert_eq!(sec_hovered.border.color, ACCENT_CYAN);

        let danger_active = danger_button(&theme, button::Status::Active);
        assert_eq!(
            danger_active.background,
            Some(Background::Color(ACCENT_DANGER))
        );
    }

    #[test]
    fn test_widget_styles_input_and_progress() {
        let theme = Theme::Dark;

        let input_active = custom_text_input(&theme, text_input::Status::Active);
        assert_eq!(input_active.border.color, BORDER_SUBTLE);

        let input_focused = custom_text_input(&theme, text_input::Status::Focused);
        assert_eq!(input_focused.border.color, ACCENT_CYAN);

        let progress = custom_progress_bar(&theme);
        assert_eq!(progress.bar, Background::Color(ACCENT_GREEN));
        assert_eq!(progress.background, Background::Color(BORDER_SUBTLE));

        let rule_style = custom_rule(&theme);
        assert_eq!(rule_style.color, BORDER_SUBTLE);
        assert_eq!(rule_style.width, 1);
    }

    #[test]
    fn test_checkbox_styles() {
        let theme = Theme::Dark;

        let active_checked = custom_checkbox(&theme, checkbox::Status::Active { is_checked: true });
        assert_eq!(active_checked.background, Background::Color(ACCENT_GREEN));

        let active_unchecked =
            custom_checkbox(&theme, checkbox::Status::Active { is_checked: false });
        assert_eq!(active_unchecked.icon_color, Color::TRANSPARENT);
    }
}
