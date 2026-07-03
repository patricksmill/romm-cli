use std::collections::HashMap;

use crate::types::achievements::{
    AchievementRow, EarnedAchievement, MergedRaMetadata, RaAchievement, RaUserGameProgression,
    RaUserProgression,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AchievementLoadResult {
    Loaded {
        rows: Vec<AchievementRow>,
        summary: (usize, usize),
    },
    Empty(String),
}

fn badge_key(achievement: &RaAchievement) -> Option<&str> {
    achievement
        .badge_id
        .as_deref()
        .or(achievement.badge_name.as_deref())
        .filter(|s| !s.is_empty())
}

pub fn merge_achievements(
    metadata: &MergedRaMetadata,
    progression: Option<&RaUserGameProgression>,
) -> Vec<AchievementRow> {
    let earned_by_badge: HashMap<&str, &EarnedAchievement> = progression
        .map(|p| {
            p.earned_achievements
                .iter()
                .map(|e| (e.id.as_str(), e))
                .collect()
        })
        .unwrap_or_default();

    let mut indexed: Vec<(i64, AchievementRow)> = metadata
        .achievements
        .iter()
        .map(|a| {
            let order = a.display_order.unwrap_or(i64::MAX);
            let earned = badge_key(a).and_then(|k| earned_by_badge.get(k).copied());
            (
                order,
                AchievementRow {
                    title: a.title.clone(),
                    description: a.description.clone(),
                    points: a.points,
                    earned: earned.is_some(),
                    earned_at: earned.and_then(|e| e.date.clone()),
                },
            )
        })
        .collect();

    indexed.sort_by_key(|(order, _)| *order);
    indexed.into_iter().map(|(_, row)| row).collect()
}

pub fn summary(rows: &[AchievementRow]) -> (usize, usize) {
    let earned = rows.iter().filter(|r| r.earned).count();
    (earned, rows.len())
}

pub fn achievement_empty_message(
    ra_id: Option<i64>,
    ra_username: Option<&str>,
    metadata: Option<&MergedRaMetadata>,
) -> Option<&'static str> {
    if ra_id.is_none() {
        return Some("Not matched to RetroAchievements");
    }
    if ra_username
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_none()
    {
        return Some("Set RA username in RomM profile (web UI)");
    }
    if metadata.map(|m| m.achievements.is_empty()).unwrap_or(true) {
        return Some("No achievement metadata on server");
    }
    None
}

pub fn prepare_achievements(
    ra_id: Option<i64>,
    metadata: Option<&MergedRaMetadata>,
    ra_username: Option<&str>,
    progression: Option<&RaUserProgression>,
) -> AchievementLoadResult {
    if let Some(msg) = achievement_empty_message(ra_id, ra_username, metadata) {
        return AchievementLoadResult::Empty(msg.to_string());
    }
    let ra_id = ra_id.expect("checked above");
    let metadata = metadata.expect("checked above");
    let game_progression =
        progression.and_then(|p| p.results.iter().find(|r| r.rom_ra_id == Some(ra_id)));
    let rows = merge_achievements(metadata, game_progression);
    AchievementLoadResult::Loaded {
        summary: summary(&rows),
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::achievements::{EarnedAchievement, RaAchievement, RaUserGameProgression};

    fn sample_metadata() -> MergedRaMetadata {
        MergedRaMetadata {
            achievements: vec![
                RaAchievement {
                    title: "First steps".into(),
                    description: Some("Begin the adventure".into()),
                    points: Some(5),
                    badge_id: Some("85541".into()),
                    badge_name: None,
                    display_order: Some(0),
                },
                RaAchievement {
                    title: "Speed run".into(),
                    description: None,
                    points: Some(10),
                    badge_id: Some("85542".into()),
                    badge_name: None,
                    display_order: Some(1),
                },
            ],
        }
    }

    #[test]
    fn merge_marks_earned_by_badge_id() {
        let progression = RaUserGameProgression {
            rom_ra_id: Some(14402),
            num_awarded: Some(1),
            max_possible: Some(2),
            earned_achievements: vec![EarnedAchievement {
                id: "85541".into(),
                date: Some("2022-08-23 22:56:38".into()),
                date_hardcore: None,
            }],
        };
        let rows = merge_achievements(&sample_metadata(), Some(&progression));
        assert_eq!(rows.len(), 2);
        assert!(rows[0].earned);
        assert!(!rows[1].earned);
        assert_eq!(summary(&rows), (1, 2));
    }

    #[test]
    fn prepare_returns_empty_when_ra_id_missing() {
        let result = prepare_achievements(None, None, Some("player"), None);
        assert_eq!(
            result,
            AchievementLoadResult::Empty("Not matched to RetroAchievements".into())
        );
    }

    #[test]
    fn current_user_fixture_deserializes() {
        let json = r#"{"id":1,"ra_username":"player1","ra_progression":{"total":1,"results":[]}}"#;
        let user: crate::types::CurrentUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.ra_username.as_deref(), Some("player1"));
    }
}
