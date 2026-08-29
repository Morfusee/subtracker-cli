use ratatui::text::Span;

use super::theme::Theme;

#[derive(Clone, Copy, Debug)]
pub struct QuotaBar {
    remaining_percent: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BarGeometry {
    pub filled: u16,
    pub empty: u16,
}

impl QuotaBar {
    pub const fn new(remaining_percent: f64) -> Self {
        Self { remaining_percent }
    }

    pub fn geometry(self, width: u16) -> BarGeometry {
        let remaining = self.remaining_percent.clamp(0.0, 100.0);
        let filled = ((f64::from(width) * remaining / 100.0).round() as u16).min(width);

        BarGeometry {
            filled,
            empty: width.saturating_sub(filled),
        }
    }

    pub fn spans(self, width: u16, theme: Theme) -> Vec<Span<'static>> {
        let remaining = self.remaining_percent.clamp(0.0, 100.0);
        let geometry = self.geometry(width);
        let health_style = theme.quota(remaining);

        vec![
            Span::styled("█".repeat(usize::from(geometry.filled)), health_style),
            Span::styled("░".repeat(usize::from(geometry.empty)), theme.empty_bar()),
            Span::raw("  "),
            Span::styled(format!("{remaining:>3.0}%"), health_style),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::{ColorMode, Theme};

    #[test]
    fn bar_geometry_matches_percentage_at_known_width() {
        assert_eq!(
            QuotaBar::new(0.0).geometry(20),
            BarGeometry {
                filled: 0,
                empty: 20
            }
        );
        assert_eq!(
            QuotaBar::new(40.0).geometry(20),
            BarGeometry {
                filled: 8,
                empty: 12
            }
        );
        assert_eq!(
            QuotaBar::new(75.0).geometry(20),
            BarGeometry {
                filled: 15,
                empty: 5
            }
        );
        assert_eq!(
            QuotaBar::new(100.0).geometry(20),
            BarGeometry {
                filled: 20,
                empty: 0
            }
        );
    }

    #[test]
    fn geometry_clamps_malformed_display_values_without_panicking() {
        assert_eq!(
            QuotaBar::new(-5.0).geometry(10),
            BarGeometry {
                filled: 0,
                empty: 10
            }
        );
        assert_eq!(
            QuotaBar::new(120.0).geometry(10),
            BarGeometry {
                filled: 10,
                empty: 0
            }
        );
    }

    #[test]
    fn spans_keep_percentage_text_even_without_color() {
        let spans = QuotaBar::new(65.0).spans(10, Theme::new(ColorMode::None));

        let plain = spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(plain, "███████░░░   65%");
    }
}
