use serde::{Deserialize, Serialize};

/// RetroAchievements catalog attached to a ROM (`merged_ra_metadata` from RomM).
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct MergedRaMetadata {
    #[serde(default)]
    pub achievements: Vec<RaAchievement>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RaAchievement {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub points: Option<i64>,
    #[serde(default)]
    pub badge_id: Option<String>,
    #[serde(default, alias = "badge_name")]
    pub badge_name: Option<String>,
    #[serde(default)]
    pub display_order: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct RaUserProgression {
    #[serde(default)]
    pub results: Vec<RaUserGameProgression>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RaUserGameProgression {
    #[serde(default)]
    pub rom_ra_id: Option<i64>,
    #[serde(default)]
    pub num_awarded: Option<i64>,
    #[serde(default)]
    pub max_possible: Option<i64>,
    #[serde(default)]
    pub earned_achievements: Vec<EarnedAchievement>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EarnedAchievement {
    pub id: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub date_hardcore: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementRow {
    pub title: String,
    pub points: Option<i64>,
    pub earned: bool,
    pub earned_at: Option<String>,
}
