# lau-leaderboard

A category-based leaderboard system with milestone tracking, trend analysis, and weighted collaboration scoring — built in Rust.

## What This Does

`lau-leaderboard` provides a complete leaderboard data structure for multi-category competitive systems. It tracks player scores across nine categories (Total XP, Conservation, Build Count, Agent Accuracy, Farm Yield, Collaboration, Exploration, Teaching, Creativity), automatically sorts entries, detects score trends, and manages milestone achievements with configurable conditions.

This is designed as a game/simulation leaderboard backend — something you'd use to power scoreboards, achievement pop-ups, and "most improved" widgets.

## Key Idea

The library treats the leaderboard as a **category-partitioned sorted map**. Each `LeaderboardCategory` maintains its own descending-sorted list of `LeaderboardEntry` values. Upserting a score automatically computes a `Trend` (Rising / Stable / Falling) and re-sorts. On top of this, a `MilestoneTracker` evaluates threshold conditions against the live leaderboard to award one-time achievements.

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lau-leaderboard = { git = "https://github.com/SuperInstance/lau-leaderboard" }
```

Or, once published:

```toml
[dependencies]
lau-leaderboard = "0.1"
```

Requires `serde` (already re-exported as a dependency).

## Quick Start

```rust
use lau_leaderboard::{Leaderboard, LeaderboardCategory, MilestoneTracker, default_milestones};

let mut lb = Leaderboard::new();

// Post scores
lb.update("alice", LeaderboardCategory::TotalXP, 100.0, 1);
lb.update("bob", LeaderboardCategory::TotalXP, 200.0, 2);
lb.update("alice", LeaderboardCategory::Conservation, 0.95, 3);

// Query
let top = lb.top_n(&LeaderboardCategory::TotalXP, 10);
let rank = lb.player_rank("alice", &LeaderboardCategory::TotalXP); // Some(2)

// Milestones
let mut tracker = MilestoneTracker::new(default_milestones());
let earned = tracker.check("alice", &lb); // returns newly earned milestones
```

## API Reference

### `Leaderboard`

| Method | Description |
|---|---|
| `new()` | Create an empty leaderboard. |
| `update(player, category, score, tick)` | Upsert a score; computes trend & re-sorts. |
| `top_n(category, n)` | Top *n* entries in a category (descending). |
| `player_rank(player, category)` | 1-based rank, or `None`. |
| `player_entries(player)` | All entries across all categories for a player. |
| `rising_stars()` | All entries with `Trend::Rising`. |
| `most_improved(window)` | Rising entries updated within the last `window` ticks. |

### `LeaderboardEntry`

```rust
pub struct LeaderboardEntry {
    pub player: String,
    pub score: f64,
    pub category: LeaderboardCategory,
    pub updated_tick: u64,
    pub trend: Trend, // Rising | Stable | Falling
}
```

### `CollaborationScore`

Weighted scoring: `collabs×1 + unique_partners×2 + successful_trades×1 + teaching_moments×3`.

```rust
let mut cs = CollaborationScore::new("alice");
cs.collabs = 10;
cs.unique_partners = 5;
cs.teaching_moments = 2;
assert_eq!(cs.score(), 26.0);
```

### `MilestoneTracker`

| Method | Description |
|---|---|
| `new(milestones)` | Create tracker with a list of milestones. |
| `check(player, leaderboard)` | Returns references to newly earned milestones. |

**Pre-built milestones:** `first_steps_milestone()`, `balance_keeper_milestone()`, `social_butterfly_milestone()`, `renaissance_kid_milestone()` — or build your own with `MilestoneCondition`.

### Serialization

All types derive `Serialize` / `Deserialize` via serde. Round-trip through JSON is tested.

## How It Works

1. **Storage:** A `HashMap<LeaderboardCategory, Vec<LeaderboardEntry>>` partitions entries by category.
2. **Upsert:** On `update()`, existing entries are found by player name. If found, trend is computed (`score > old` → Rising, equal → Stable, less → Falling) and the score is replaced. Otherwise a new entry is pushed. The category list is then sorted descending by score.
3. **Milestones:** `MilestoneTracker` stores a `Vec<Milestone>` and a `HashMap<String, Vec<String>>` mapping player → earned milestone names. `check()` iterates milestones, skips already-earned ones, evaluates the `MilestoneCondition` against the leaderboard, and returns references to any newly earned milestones.
4. **Most Improved:** Finds the maximum `updated_tick` across all entries, computes a threshold (`max_tick - window`), and filters for Rising entries above that threshold.

## The Math

**Trend detection** is a simple three-way comparison against ε = `f64::EPSILON`:

```
trend = if score > old_score       → Rising
        else if |score - old| < ε  → Stable
        else                       → Falling
```

**Collaboration scoring** uses a weighted linear combination:

$$S_{\text{collab}} = 1 \cdot c + 2 \cdot u + 1 \cdot t + 3 \cdot m$$

where $c$ = collaborations, $u$ = unique partners, $t$ = successful trades, $m$ = teaching moments.

**Ranking** is 1-based index into the descending-sorted category list.

## Tests

**32 unit tests** covering: empty state, upsert, sorting, trend detection, rank lookup, player entries, rising stars, most improved windowing, collaboration scoring, all four milestone conditions, milestone idempotency, multi-player isolation, serde round-trips, and large-scale (100-player) top-*n*.

## License

No license file present. Contact the repository owner for licensing terms.
