use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Direction of recent change for a leaderboard entry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Trend {
    Rising,
    Stable,
    Falling,
}

/// Categories for the leaderboard – celebrating growth and collaboration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeaderboardCategory {
    TotalXP,
    Conservation,
    BuildCount,
    AgentAccuracy,
    FarmYield,
    Collaboration,
    Exploration,
    Teaching,
    Creativity,
}

/// A single entry on the leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub player: String,
    pub score: f64,
    pub category: LeaderboardCategory,
    pub updated_tick: u64,
    pub trend: Trend,
}

impl LeaderboardEntry {
    pub fn new(player: &str, score: f64, category: LeaderboardCategory, tick: u64) -> Self {
        Self {
            player: player.to_string(),
            score,
            category,
            updated_tick: tick,
            trend: Trend::Stable,
        }
    }
}

/// The leaderboard, organised by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    pub entries: HashMap<LeaderboardCategory, Vec<LeaderboardEntry>>,
}

impl Leaderboard {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Upsert an entry for a player in the given category.
    /// Computes the trend based on score change.
    pub fn update(&mut self, player: &str, category: LeaderboardCategory, score: f64, tick: u64) {
        let entries = self.entries.entry(category).or_default();

        if let Some(existing) = entries.iter_mut().find(|e| e.player == player) {
            // Compute trend
            if score > existing.score {
                existing.trend = Trend::Rising;
            } else if (score - existing.score).abs() < f64::EPSILON {
                existing.trend = Trend::Stable;
            } else {
                existing.trend = Trend::Falling;
            }
            existing.score = score;
            existing.updated_tick = tick;
        } else {
            entries.push(LeaderboardEntry::new(player, score, category, tick));
        }

        // Keep sorted descending by score
        entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Return the top N entries in a category.
    pub fn top_n(&self, category: &LeaderboardCategory, n: usize) -> Vec<&LeaderboardEntry> {
        self.entries
            .get(category)
            .map(|entries| entries.iter().take(n).collect())
            .unwrap_or_default()
    }

    /// Return the 1-based rank of a player in a category.
    pub fn player_rank(&self, player: &str, category: &LeaderboardCategory) -> Option<usize> {
        self.entries.get(category).and_then(|entries| {
            entries
                .iter()
                .position(|e| e.player == player)
                .map(|idx| idx + 1)
        })
    }

    /// Return all leaderboard entries for a player across all categories.
    pub fn player_entries(&self, player: &str) -> Vec<&LeaderboardEntry> {
        let mut result: Vec<&LeaderboardEntry> = Vec::new();
        for entries in self.entries.values() {
            for entry in entries {
                if entry.player == player {
                    result.push(entry);
                }
            }
        }
        result
    }

    /// Return all entries whose trend is Rising.
    pub fn rising_stars(&self) -> Vec<&LeaderboardEntry> {
        let mut result: Vec<&LeaderboardEntry> = Vec::new();
        for entries in self.entries.values() {
            for entry in entries {
                if matches!(entry.trend, Trend::Rising) {
                    result.push(entry);
                }
            }
        }
        result
    }

    /// Return entries with the biggest score gains in the last `window` ticks.
    /// Only entries that have been updated within the window and have Rising trend qualify.
    /// Sorted descending by score.
    pub fn most_improved(&self, window: u64) -> Vec<&LeaderboardEntry> {
        let current_tick = self
            .entries
            .values()
            .flatten()
            .map(|e| e.updated_tick)
            .max()
            .unwrap_or(0);
        let threshold = current_tick.saturating_sub(window);

        let mut candidates: Vec<&LeaderboardEntry> = self
            .entries
            .values()
            .flatten()
            .filter(|e| e.updated_tick >= threshold && matches!(e.trend, Trend::Rising))
            .collect();

        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }
}

impl Default for Leaderboard {
    fn default() -> Self {
        Self::new()
    }
}

/// A weighted collaboration score for a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationScore {
    pub player: String,
    pub collabs: u32,
    pub unique_partners: u32,
    pub successful_trades: u32,
    pub teaching_moments: u32,
}

impl CollaborationScore {
    pub fn new(player: &str) -> Self {
        Self {
            player: player.to_string(),
            collabs: 0,
            unique_partners: 0,
            successful_trades: 0,
            teaching_moments: 0,
        }
    }

    /// Weighted score: collabs*1 + partners*2 + trades*1 + teaching*3
    pub fn score(&self) -> f64 {
        (self.collabs as f64 * 1.0)
            + (self.unique_partners as f64 * 2.0)
            + (self.successful_trades as f64 * 1.0)
            + (self.teaching_moments as f64 * 3.0)
    }
}

/// Conditions that trigger a milestone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MilestoneCondition {
    ScoreAbove(LeaderboardCategory, f64),
    RankAbove(LeaderboardCategory, usize),
    TotalCategoriesAbove(f64),
    CollaborationScoreAbove(f64),
}

/// A milestone that a player can earn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub description: String,
    pub condition: MilestoneCondition,
    pub reward: String,
}

impl Milestone {
    pub fn new(name: &str, description: &str, condition: MilestoneCondition, reward: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            condition,
            reward: reward.to_string(),
        }
    }
}

/// Tracks which players have earned which milestones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneTracker {
    pub milestones: Vec<Milestone>,
    pub earned: HashMap<String, Vec<String>>,
}

impl MilestoneTracker {
    pub fn new(milestones: Vec<Milestone>) -> Self {
        Self {
            milestones,
            earned: HashMap::new(),
        }
    }

    /// Check all milestones for a player and return newly-earned ones.
    pub fn check(&mut self, player: &str, leaderboard: &Leaderboard) -> Vec<&Milestone> {
        let newly_earned: Vec<&Milestone> = self
            .milestones
            .iter()
            .filter(|milestone| {
                // Skip if already earned
                if let Some(earned) = self.earned.get(player) {
                    if earned.contains(&milestone.name) {
                        return false;
                    }
                }
                self.condition_met(player, &milestone.condition, leaderboard)
            })
            .collect();

        for milestone in &newly_earned {
            self.earned
                .entry(player.to_string())
                .or_default()
                .push(milestone.name.clone());
        }

        newly_earned
    }

    fn condition_met(&self, player: &str, condition: &MilestoneCondition, leaderboard: &Leaderboard) -> bool {
        match condition {
            MilestoneCondition::ScoreAbove(category, threshold) => {
                leaderboard
                    .entries
                    .get(category)
                    .and_then(|entries| entries.iter().find(|e| e.player == player))
                    .is_some_and(|e| e.score > *threshold)
            }
            MilestoneCondition::RankAbove(category, max_rank) => {
                leaderboard
                    .player_rank(player, category)
                    .is_some_and(|rank| rank <= *max_rank)
            }
            MilestoneCondition::TotalCategoriesAbove(threshold) => {
                let count = leaderboard
                    .entries
                    .values()
                    .flatten()
                    .filter(|e| e.player == player && e.score > 0.0)
                    .count();
                (count as f64) > *threshold
            }
            MilestoneCondition::CollaborationScoreAbove(threshold) => {
                // CollaborationScoreAbove is checked externally; for tracker
                // we treat it as unmet unless we find a Collaboration entry
                leaderboard
                    .entries
                    .get(&LeaderboardCategory::Collaboration)
                    .and_then(|entries| entries.iter().find(|e| e.player == player))
                    .is_some_and(|e| e.score > *threshold)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pre-built milestones
// ---------------------------------------------------------------------------

/// "First Steps" – awarded when a player has any score > 0 in any category.
pub fn first_steps_milestone() -> Milestone {
    Milestone::new(
        "First Steps",
        "Earn your first point in any category.",
        MilestoneCondition::ScoreAbove(LeaderboardCategory::TotalXP, 0.0),
        "Starter badge",
    )
}

/// "Balance Keeper" – awarded when a player's Conservation score exceeds 0.9.
pub fn balance_keeper_milestone() -> Milestone {
    Milestone::new(
        "Balance Keeper",
        "Achieve a Conservation score above 0.9.",
        MilestoneCondition::ScoreAbove(LeaderboardCategory::Conservation, 0.9),
        "Nature emblem",
    )
}

/// "Social Butterfly" – awarded when a player has 10+ unique partners (Collaboration score >= 20).
pub fn social_butterfly_milestone() -> Milestone {
    Milestone::new(
        "Social Butterfly",
        "Collaborate with 10 or more unique partners.",
        MilestoneCondition::ScoreAbove(LeaderboardCategory::Collaboration, 20.0),
        "Networking trophy",
    )
}

/// "Renaissance Kid" – awarded when a player has a score > 0 in 5+ categories.
pub fn renaissance_kid_milestone() -> Milestone {
    Milestone::new(
        "Renaissance Kid",
        "Earn a score above 0 in at least 5 categories.",
        MilestoneCondition::TotalCategoriesAbove(4.0),
        "Versatility medal",
    )
}

/// Return a default set of pre-built milestones.
pub fn default_milestones() -> Vec<Milestone> {
    vec![
        first_steps_milestone(),
        balance_keeper_milestone(),
        social_butterfly_milestone(),
        renaissance_kid_milestone(),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ======================== Leaderboard tests ========================

    #[test]
    fn test_new_leaderboard_is_empty() {
        let lb = Leaderboard::new();
        assert!(lb.entries.is_empty());
    }

    #[test]
    fn test_top_n_returns_empty_for_unknown_category() {
        let lb = Leaderboard::new();
        assert!(lb.top_n(&LeaderboardCategory::TotalXP, 5).is_empty());
    }

    #[test]
    fn test_update_adds_entry() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 1);
        let top = lb.top_n(&LeaderboardCategory::TotalXP, 10);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].player, "alice");
        assert_eq!(top[0].score, 100.0);
    }

    #[test]
    fn test_update_sorts_descending() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 50.0, 1);
        lb.update("bob", LeaderboardCategory::TotalXP, 200.0, 2);
        let top = lb.top_n(&LeaderboardCategory::TotalXP, 10);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].player, "bob");
        assert_eq!(top[1].player, "alice");
    }

    #[test]
    fn test_update_upserts_and_tracks_trend_rising() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 50.0, 1);
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 2);
        let entry = lb.top_n(&LeaderboardCategory::TotalXP, 10)[0];
        assert_eq!(entry.score, 100.0);
        assert!(matches!(entry.trend, Trend::Rising));
    }

    #[test]
    fn test_update_trend_falling() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 1);
        lb.update("alice", LeaderboardCategory::TotalXP, 30.0, 2);
        let entry = lb.top_n(&LeaderboardCategory::TotalXP, 10)[0];
        assert_eq!(entry.score, 30.0);
        assert!(matches!(entry.trend, Trend::Falling));
    }

    #[test]
    fn test_update_trend_stable() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 1);
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 2);
        let entry = lb.top_n(&LeaderboardCategory::TotalXP, 10)[0];
        assert!(matches!(entry.trend, Trend::Stable));
    }

    #[test]
    fn test_player_rank_found() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 50.0, 1);
        lb.update("bob", LeaderboardCategory::TotalXP, 200.0, 2);
        assert_eq!(lb.player_rank("alice", &LeaderboardCategory::TotalXP), Some(2));
        assert_eq!(lb.player_rank("bob", &LeaderboardCategory::TotalXP), Some(1));
    }

    #[test]
    fn test_player_rank_not_found() {
        let lb = Leaderboard::new();
        assert_eq!(lb.player_rank("nobody", &LeaderboardCategory::TotalXP), None);
    }

    #[test]
    fn test_player_entries_returns_all_categories() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 1);
        lb.update("alice", LeaderboardCategory::Conservation, 50.0, 2);
        lb.update("bob", LeaderboardCategory::TotalXP, 200.0, 3);
        let entries = lb.player_entries("alice");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_player_entries_returns_empty_for_unknown() {
        let lb = Leaderboard::new();
        assert!(lb.player_entries("nobody").is_empty());
    }

    #[test]
    fn test_rising_stars() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 50.0, 1);
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 2); // Rising
        lb.update("bob", LeaderboardCategory::TotalXP, 200.0, 3);
        lb.update("bob", LeaderboardCategory::TotalXP, 50.0, 4); // Falling
        let stars = lb.rising_stars();
        assert_eq!(stars.len(), 1);
        assert_eq!(stars[0].player, "alice");
    }

    #[test]
    fn test_most_improved_returns_rising_entries_in_window() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 50.0, 1);
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 5); // Rising at tick 5
        lb.update("bob", LeaderboardCategory::TotalXP, 200.0, 10);
        lb.update("bob", LeaderboardCategory::TotalXP, 50.0, 15); // Falling at tick 15

        // Window of 20 covers ticks [max(15)-20+1..15] = [0..15]
        let improved = lb.most_improved(20);
        assert!(improved.iter().any(|e| e.player == "alice"));
        // Bob is Falling so excluded
        assert!(!improved.iter().any(|e| e.player == "bob"));
    }

    #[test]
    fn test_most_improved_empty_window() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 50.0, 1);
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 100);
        // Window 10 around max tick 100 → threshold 91, alice updated at 100
        let improved = lb.most_improved(10);
        assert_eq!(improved.len(), 1);
    }

    // ======================== CollaborationScore tests ========================

    #[test]
    fn test_collaboration_score_default_is_zero() {
        let cs = CollaborationScore::new("alice");
        assert_eq!(cs.score(), 0.0);
    }

    #[test]
    fn test_collaboration_score_weighted() {
        let mut cs = CollaborationScore::new("alice");
        cs.collabs = 10;
        cs.unique_partners = 5;
        cs.successful_trades = 3;
        cs.teaching_moments = 2;
        // 10*1 + 5*2 + 3*1 + 2*3 = 10 + 10 + 3 + 6 = 29
        assert!((cs.score() - 29.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_collaboration_score_partial_weights() {
        let mut cs = CollaborationScore::new("bob");
        cs.collabs = 5;
        cs.teaching_moments = 1;
        // 5*1 + 0 + 0 + 1*3 = 8
        assert!((cs.score() - 8.0).abs() < f64::EPSILON);
    }

    // ======================== Milestone tests ========================

    #[test]
    fn test_first_steps_earned_when_score_above_zero() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 1.0, 1);

        let mut tracker = MilestoneTracker::new(default_milestones());
        let earned = tracker.check("alice", &lb);
        assert!(earned.iter().any(|m| m.name == "First Steps"));
    }

    #[test]
    fn test_first_steps_not_earned_at_zero() {
        let lb = Leaderboard::new();
        let mut tracker = MilestoneTracker::new(default_milestones());
        let earned = tracker.check("alice", &lb);
        assert!(!earned.iter().any(|m| m.name == "First Steps"));
    }

    #[test]
    fn test_balance_keeper_earned() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::Conservation, 1.0, 1);

        let mut tracker = MilestoneTracker::new(default_milestones());
        let earned = tracker.check("alice", &lb);
        assert!(earned.iter().any(|m| m.name == "Balance Keeper"));
    }

    #[test]
    fn test_balance_keeper_not_earned() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::Conservation, 0.5, 1);

        let mut tracker = MilestoneTracker::new(default_milestones());
        let earned = tracker.check("alice", &lb);
        assert!(!earned.iter().any(|m| m.name == "Balance Keeper"));
    }

    #[test]
    fn test_social_butterfly_earned() {
        let mut lb = Leaderboard::new();
        // Collaboration score of 20+ = 10 unique partners * 2
        lb.update("alice", LeaderboardCategory::Collaboration, 25.0, 1);

        let mut tracker = MilestoneTracker::new(default_milestones());
        let earned = tracker.check("alice", &lb);
        assert!(earned.iter().any(|m| m.name == "Social Butterfly"));
    }

    #[test]
    fn test_renaissance_kid_earned() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 1.0, 1);
        lb.update("alice", LeaderboardCategory::Conservation, 1.0, 2);
        lb.update("alice", LeaderboardCategory::BuildCount, 1.0, 3);
        lb.update("alice", LeaderboardCategory::AgentAccuracy, 1.0, 4);
        lb.update("alice", LeaderboardCategory::FarmYield, 1.0, 5);

        let mut tracker = MilestoneTracker::new(default_milestones());
        let earned = tracker.check("alice", &lb);
        assert!(earned.iter().any(|m| m.name == "Renaissance Kid"));
    }

    #[test]
    fn test_renaissance_kid_not_earned() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 1.0, 1);
        lb.update("alice", LeaderboardCategory::Conservation, 1.0, 2);
        lb.update("alice", LeaderboardCategory::BuildCount, 1.0, 3);
        // Only 3 categories with score > 0

        let mut tracker = MilestoneTracker::new(default_milestones());
        let earned = tracker.check("alice", &lb);
        assert!(!earned.iter().any(|m| m.name == "Renaissance Kid"));
    }

    #[test]
    fn test_milestones_not_earned_twice() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 1.0, 1);

        let mut tracker = MilestoneTracker::new(default_milestones());
        let first = tracker.check("alice", &lb);
        assert!(first.iter().any(|m| m.name == "First Steps"));

        let second = tracker.check("alice", &lb);
        assert!(!second.iter().any(|m| m.name == "First Steps"));
    }

    #[test]
    fn test_multiple_players_tracked_independently() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 1.0, 1);
        lb.update("bob", LeaderboardCategory::TotalXP, 1.0, 2);

        let mut tracker = MilestoneTracker::new(default_milestones());
        let alice_earned: Vec<String> = tracker
            .check("alice", &lb)
            .iter()
            .map(|m| m.name.clone())
            .collect();
        let bob_earned: Vec<String> = tracker
            .check("bob", &lb)
            .iter()
            .map(|m| m.name.clone())
            .collect();

        assert!(alice_earned.iter().any(|n| n == "First Steps"));
        assert!(bob_earned.iter().any(|n| n == "First Steps"));
    }

    #[test]
    fn test_rank_above_condition() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 50.0, 1);
        lb.update("bob", LeaderboardCategory::TotalXP, 200.0, 2);

        let milestone = Milestone::new(
            "Top Rank",
            "Reach rank 1 in TotalXP.",
            MilestoneCondition::RankAbove(LeaderboardCategory::TotalXP, 1),
            "Gold star",
        );
        let mut tracker = MilestoneTracker::new(vec![milestone]);

        let bob_earned = tracker.check("bob", &lb);
        assert!(bob_earned.iter().any(|m| m.name == "Top Rank"));

        let alice_earned = tracker.check("alice", &lb);
        assert!(!alice_earned.iter().any(|m| m.name == "Top Rank"));
    }

    #[test]
    fn test_large_top_n() {
        let mut lb = Leaderboard::new();
        for i in 0..100 {
            lb.update(
                &format!("player_{}", i),
                LeaderboardCategory::TotalXP,
                i as f64,
                i,
            );
        }
        let top = lb.top_n(&LeaderboardCategory::TotalXP, 10);
        assert_eq!(top.len(), 10);
        assert_eq!(top[0].player, "player_99");
        assert_eq!(top[9].player, "player_90");
    }

    #[test]
    fn test_serde_roundtrip_leaderboard() {
        let mut lb = Leaderboard::new();
        lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 1);
        lb.update("bob", LeaderboardCategory::Conservation, 50.0, 2);

        let json = serde_json::to_string(&lb).unwrap();
        let deserialized: Leaderboard = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized
                .player_rank("alice", &LeaderboardCategory::TotalXP),
            Some(1)
        );
        assert_eq!(
            deserialized
                .player_rank("bob", &LeaderboardCategory::Conservation),
            Some(1)
        );
    }

    #[test]
    fn test_serde_roundtrip_milestone_tracker() {
        let tracker = MilestoneTracker::new(default_milestones());
        let json = serde_json::to_string(&tracker).unwrap();
        let deserialized: MilestoneTracker = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.milestones.len(), 4);
        assert!(deserialized.earned.is_empty());
    }

    #[test]
    fn test_collaboration_score_serde() {
        let mut cs = CollaborationScore::new("alice");
        cs.collabs = 10;
        cs.unique_partners = 5;
        cs.teaching_moments = 3;

        let json = serde_json::to_string(&cs).unwrap();
        let deserialized: CollaborationScore = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.player, "alice");
        assert!((deserialized.score() - cs.score()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_leaderboard_entry_serde() {
        let entry = LeaderboardEntry::new("alice", 42.5, LeaderboardCategory::Exploration, 7);
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LeaderboardEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.player, "alice");
        assert!((deserialized.score - 42.5).abs() < f64::EPSILON);
        assert!(matches!(deserialized.category, LeaderboardCategory::Exploration));
    }
}
