use ratatui::style::{Color, Modifier, Style};

use crate::model::ProviderId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorMode {
    TrueColor,
    Named,
    None,
}

impl ColorMode {
    pub fn detect_from(no_color: bool, colorterm: Option<&str>, windows_terminal: bool) -> Self {
        if no_color {
            return Self::None;
        }

        let colorterm = colorterm.unwrap_or_default().to_ascii_lowercase();
        if colorterm.contains("truecolor") || colorterm.contains("24bit") || windows_terminal {
            Self::TrueColor
        } else {
            Self::Named
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaHealth {
    Healthy,
    Moderate,
    Low,
    Critical,
}

impl QuotaHealth {
    pub fn from_remaining(percent: f64) -> Self {
        if percent >= 75.0 {
            Self::Healthy
        } else if percent >= 40.0 {
            Self::Moderate
        } else if percent >= 15.0 {
            Self::Low
        } else {
            Self::Critical
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    mode: ColorMode,
}

impl Theme {
    pub const fn new(mode: ColorMode) -> Self {
        Self { mode }
    }

    pub fn detect() -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some();
        let colorterm = std::env::var("COLORTERM").ok();
        let windows_terminal = std::env::var_os("WT_SESSION").is_some();

        Self::new(ColorMode::detect_from(
            no_color,
            colorterm.as_deref(),
            windows_terminal,
        ))
    }

    pub fn provider_title(self, provider: ProviderId) -> Style {
        self.provider_border(provider).add_modifier(Modifier::BOLD)
    }

    pub fn provider_border(self, provider: ProviderId) -> Style {
        let rgb = match provider {
            ProviderId::Codex => (110, 160, 205),    // Dark pastel sky blue
            ProviderId::OpenCode => (100, 185, 165), // Dark pastel sage / teal
            ProviderId::Antigravity => (175, 145, 210), // Dark pastel mauve / lavender
        };

        let named = match provider {
            ProviderId::Codex => Color::Cyan,
            ProviderId::OpenCode => Color::Green,
            ProviderId::Antigravity => Color::Magenta,
        };

        self.style(rgb, named)
    }

    pub fn quota(self, remaining_percent: f64) -> Style {
        match QuotaHealth::from_remaining(remaining_percent) {
            QuotaHealth::Healthy => self.style((120, 180, 135), Color::Green), // Soft pastel sage
            QuotaHealth::Moderate => self.style((220, 180, 110), Color::Yellow), // Soft pastel amber
            QuotaHealth::Low => self.style((215, 135, 105), Color::DarkGray), // Soft pastel terracotta
            QuotaHealth::Critical => self.style((215, 110, 115), Color::Red), // Soft pastel dusty rose
        }
    }

    pub fn primary(self) -> Style {
        self.style((210, 218, 225), Color::White) // Soft pastel chalk
    }

    pub fn secondary(self) -> Style {
        self.style((130, 145, 160), Color::DarkGray) // Muted pastel slate
    }

    pub fn empty_bar(self) -> Style {
        self.style((45, 55, 70), Color::DarkGray) // Deep muted slate
    }

    pub fn warning(self) -> Style {
        self.style((220, 180, 110), Color::Yellow)
    }

    pub fn error(self) -> Style {
        self.style((215, 110, 115), Color::Red)
    }

    pub fn backdrop(self) -> Style {
        match self.mode {
            ColorMode::TrueColor => Style::default()
                .fg(Color::Rgb(55, 65, 78))
                .bg(Color::Rgb(8, 12, 18)),
            ColorMode::Named => Style::default().fg(Color::DarkGray).bg(Color::Black),
            ColorMode::None => Style::default(),
        }
    }

    fn style(self, rgb: (u8, u8, u8), named: Color) -> Style {
        match self.mode {
            ColorMode::TrueColor => Style::default().fg(Color::Rgb(rgb.0, rgb.1, rgb.2)),
            ColorMode::Named => Style::default().fg(named),
            ColorMode::None => Style::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn quota_health_boundaries_are_exact() {
        assert_eq!(QuotaHealth::from_remaining(100.0), QuotaHealth::Healthy);
        assert_eq!(QuotaHealth::from_remaining(75.0), QuotaHealth::Healthy);
        assert_eq!(QuotaHealth::from_remaining(74.0), QuotaHealth::Moderate);
        assert_eq!(QuotaHealth::from_remaining(40.0), QuotaHealth::Moderate);
        assert_eq!(QuotaHealth::from_remaining(39.0), QuotaHealth::Low);
        assert_eq!(QuotaHealth::from_remaining(15.0), QuotaHealth::Low);
        assert_eq!(QuotaHealth::from_remaining(14.0), QuotaHealth::Critical);
        assert_eq!(QuotaHealth::from_remaining(0.0), QuotaHealth::Critical);
    }

    #[test]
    fn provider_accents_are_distinct_in_true_color_mode() {
        let theme = Theme::new(ColorMode::TrueColor);

        assert_eq!(
            theme.provider_border(ProviderId::Codex).fg,
            Some(Color::Rgb(110, 160, 205))
        );
        assert_eq!(
            theme.provider_border(ProviderId::OpenCode).fg,
            Some(Color::Rgb(100, 185, 165))
        );
        assert_eq!(
            theme.provider_border(ProviderId::Antigravity).fg,
            Some(Color::Rgb(175, 145, 210))
        );
    }

    #[test]
    fn no_color_always_wins_capability_detection() {
        assert_eq!(
            ColorMode::detect_from(true, Some("truecolor"), true),
            ColorMode::None
        );
    }

    #[test]
    fn true_color_is_selected_for_known_true_color_environment() {
        assert_eq!(
            ColorMode::detect_from(false, Some("24bit"), false),
            ColorMode::TrueColor
        );
        assert_eq!(
            ColorMode::detect_from(false, None, true),
            ColorMode::TrueColor
        );
    }

    #[test]
    fn unknown_environment_uses_named_color_fallback() {
        assert_eq!(ColorMode::detect_from(false, None, false), ColorMode::Named);
    }
}
