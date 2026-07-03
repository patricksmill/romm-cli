use ratatui::text::Line;

use romm_api::core::utils::truncate;
use romm_api::types::AchievementRow;

use super::types::AchievementListState;

pub fn achievement_lines(state: &AchievementListState) -> Vec<Line<'static>> {
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
            lines.extend(rows.iter().map(format_row));
            lines
        }
    }
}

fn format_row(row: &AchievementRow) -> Line<'static> {
    let marker = if row.earned { "[✓]" } else { "[ ]" };
    let points = row
        .points
        .map(|p| format!(" — {p} pts"))
        .unwrap_or_default();
    Line::from(format!(
        "  {marker} {}{}",
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
                points: Some(5),
                earned: true,
                earned_at: None,
            }],
            summary: (1, 1),
        };
        assert!(achievement_lines(&state)[1].to_string().contains('✓'));
    }
}
