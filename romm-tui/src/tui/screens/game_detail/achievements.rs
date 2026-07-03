use ratatui::text::Line;

use romm_api::core::utils::truncate;
use romm_api::types::AchievementRow;

use super::types::AchievementListState;

pub fn achievement_lines(
    state: &AchievementListState,
    selected_index: usize,
) -> Vec<Line<'static>> {
    match state {
        AchievementListState::Idle => vec![Line::from("  Loading soon...")],
        AchievementListState::Loading => vec![Line::from("  Loading achievements...")],
        AchievementListState::Unsupported(msg) => vec![Line::from(format!("  {msg}"))],
        AchievementListState::Failed(e) => {
            vec![Line::from(format!("  Error: {}", truncate(e, 90)))]
        }
        AchievementListState::Empty(msg) => vec![Line::from(format!("  {msg}"))],
        AchievementListState::Loaded { rows, summary } if rows.is_empty() => {
            vec![Line::from("  No achievements listed")]
        }
        AchievementListState::Loaded { rows, summary } => {
            let (earned, total) = *summary;
            let pct = if total == 0 {
                0
            } else {
                earned.saturating_mul(100).checked_div(total).unwrap_or(0)
            };
            let mut lines = vec![Line::from(format!("  {earned}/{total} ({pct}%)"))];
            lines.extend(
                rows.iter()
                    .enumerate()
                    .map(|(i, row)| format_row(row, i == selected_index)),
            );
            lines
        }
    }
}

fn format_row(row: &AchievementRow, selected: bool) -> Line<'static> {
    let cursor = if selected { ">" } else { " " };
    let marker = if row.earned { "[✓]" } else { "[ ]" };
    let points = row
        .points
        .map(|p| format!(" — {p} pts"))
        .unwrap_or_default();
    Line::from(format!(
        " {cursor} {marker} {}{}",
        truncate(&row.title, 100),
        points
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use romm_api::types::AchievementRow;

    #[test]
    fn achievement_lines_shows_earned_marker() {
        let state = AchievementListState::Loaded {
            rows: vec![AchievementRow {
                title: "First steps".into(),
                description: None,
                points: Some(5),
                earned: true,
                earned_at: None,
            }],
            summary: (1, 1),
        };
        let lines = achievement_lines(&state, 0);
        assert!(lines[1].to_string().contains('✓'));
        assert!(lines[1].to_string().contains('>'));
    }

    #[test]
    fn achievement_lines_unselected_has_no_cursor() {
        let state = AchievementListState::Loaded {
            rows: vec![
                AchievementRow {
                    title: "First".into(),
                    description: None,
                    points: Some(5),
                    earned: true,
                    earned_at: None,
                },
                AchievementRow {
                    title: "Second".into(),
                    description: None,
                    points: Some(10),
                    earned: false,
                    earned_at: None,
                },
            ],
            summary: (1, 2),
        };
        let lines = achievement_lines(&state, 0);
        assert!(lines[1].to_string().contains('>'));
        assert!(!lines[2].to_string().contains('>'));
    }
}
