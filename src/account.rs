//! Account-wide progress that outlives characters: the highest map tier
//! unlocked and the collection of supports and artifacts found. Saved as a
//! small text file in the home directory.
use crate::content::{ArtifactKind, Loot, SupportKind, MAX_ARTIFACT_TIER};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    /// Highest map tier you may start. Beating a tier's boss unlocks the next.
    pub max_tier: u32,
    /// Supports are unlocks: found once, usable by every character.
    pub supports: Vec<SupportKind>,
    /// Artifacts found, each with its worth in tier-1 copies. Copies fuse on
    /// their own: two of a tier make one of the next, so a worth of 5 is a
    /// tier-3 copy plus a tier-1 copy, and the artifact is tier 3.
    pub artifacts: Vec<(ArtifactKind, u32)>,
}

impl Default for Account {
    fn default() -> Self {
        Account { max_tier: 1, supports: Vec::new(), artifacts: Vec::new() }
    }
}

impl Account {
    fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".arpg-machine").join("account.txt"))
    }

    pub fn parse(text: &str) -> Self {
        let mut account = Account::default();
        for line in text.lines() {
            let Some((key, value)) = line.trim().split_once('=') else { continue };
            let value = value.trim();
            match key {
                "max_tier" => {
                    if let Ok(tier) = value.parse::<u32>() {
                        account.max_tier = tier.max(1);
                    }
                }
                "support" => {
                    if let Some(s) = SupportKind::ALL.into_iter().find(|s| s.name() == value) {
                        if !account.supports.contains(&s) {
                            account.supports.push(s);
                        }
                    }
                }
                "artifact" => {
                    let (name, count) = value.rsplit_once(':').unwrap_or((value, "1"));
                    if let Some(a) = ArtifactKind::ALL.into_iter().find(|a| a.name() == name) {
                        account.add(Loot::Artifact(a, 1));
                        if let Some(entry) = account.artifacts.iter_mut().find(|(k, _)| *k == a) {
                            entry.1 = count.parse().unwrap_or(1).clamp(1, MAX_WORTH);
                        }
                    }
                }
                _ => {}
            }
        }
        account
    }

    pub fn to_text(&self) -> String {
        let mut out = format!("max_tier={}\n", self.max_tier);
        for s in &self.supports {
            out += &format!("support={}\n", s.name());
        }
        for (a, n) in &self.artifacts {
            out += &format!("artifact={}:{}\n", a.name(), n);
        }
        out
    }

    /// Loads the saved account, or a fresh one.
    pub fn load() -> Self {
        Self::path().and_then(|p| std::fs::read_to_string(p).ok()).map(|t| Self::parse(&t)).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::path() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(path, self.to_text());
        }
    }

    /// Records a boss kill at `tier`. Returns the newly unlocked tier, if any.
    pub fn record_victory(&mut self, tier: u32) -> Option<u32> {
        if tier >= self.max_tier {
            self.max_tier = tier + 1;
            Some(self.max_tier)
        } else {
            None
        }
    }

    pub fn has_support(&self, support: SupportKind) -> bool {
        self.supports.contains(&support)
    }

    /// The artifact's worth in tier-1 copies (0 if never found).
    pub fn artifact_worth(&self, artifact: ArtifactKind) -> u32 {
        self.artifacts.iter().find(|(a, _)| *a == artifact).map_or(0, |(_, n)| *n)
    }

    /// Best tier you own, 0 if none.
    pub fn artifact_tier(&self, artifact: ArtifactKind) -> u8 {
        tier_of(self.artifact_worth(artifact))
    }

    /// Progress toward the next tier: (worth, worth needed). None at the top.
    pub fn artifact_progress(&self, artifact: ArtifactKind) -> Option<(u32, u32)> {
        let tier = self.artifact_tier(artifact);
        (tier > 0 && tier < MAX_ARTIFACT_TIER).then(|| (self.artifact_worth(artifact), 1 << tier))
    }

    /// Artifacts already legendary: more copies would be wasted, so they no
    /// longer drop.
    pub fn maxed_artifacts(&self) -> Vec<ArtifactKind> {
        self.artifacts.iter().filter(|&&(_, n)| n >= MAX_WORTH).map(|&(a, _)| a).collect()
    }

    /// Adds a drop. Returns the artifact's new tier if the copy fused it up.
    pub fn add(&mut self, loot: Loot) -> Option<u8> {
        match loot {
            Loot::Support(s) => {
                if !self.supports.contains(&s) {
                    self.supports.push(s);
                }
                None
            }
            Loot::Artifact(a, tier) => {
                let before = self.artifact_tier(a);
                let worth = 1u32 << (tier.clamp(1, MAX_ARTIFACT_TIER) - 1);
                match self.artifacts.iter_mut().find(|(k, _)| *k == a) {
                    Some(entry) => entry.1 = (entry.1 + worth).min(MAX_WORTH),
                    None => self.artifacts.push((a, worth)),
                }
                let after = self.artifact_tier(a);
                (before > 0 && after > before).then_some(after)
            }
        }
    }
}

/// Worth of a legendary copy: 2⁴ tier-1 copies.
pub const MAX_WORTH: u32 = 1 << (MAX_ARTIFACT_TIER - 1);

fn tier_of(worth: u32) -> u8 {
    if worth == 0 {
        0
    } else {
        ((u32::BITS - worth.leading_zeros()) as u8).min(MAX_ARTIFACT_TIER)
    }
}

/// Highest artifact tier that drops on a map tier: 1 on tiers 1–2, 2 on 3–4,
/// up to legendary from tier 9.
pub fn max_drop_tier(map_tier: u32) -> u8 {
    (1 + (map_tier.max(1) - 1) / 2).min(MAX_ARTIFACT_TIER as u32) as u8
}

/// How much harder a tier is than tier 1, and how much more experience it gives.
pub struct TierScaling {
    pub health: f32,
    pub damage: f32,
    pub xp: f32,
}

pub fn tier_scaling(tier: u32) -> TierScaling {
    let steps = tier.max(1) - 1;
    TierScaling {
        health: 1.3f32.powi(steps as i32),
        damage: 1.2f32.powi(steps as i32),
        xp: 1.0 + 0.25 * steps as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beating_the_top_tier_unlocks_the_next() {
        let mut account = Account::default();
        assert_eq!(account.record_victory(1), Some(2));
        assert_eq!(account.record_victory(1), None);
        assert_eq!(account.record_victory(2), Some(3));
        assert_eq!(account.max_tier, 3);
    }

    #[test]
    fn saves_and_parses() {
        let mut account = Account { max_tier: 7, ..Account::default() };
        account.add(Loot::Support(SupportKind::Hunter));
        account.add(Loot::Artifact(ArtifactKind::SagesRing, 1));
        account.add(Loot::Artifact(ArtifactKind::SagesRing, 1));
        assert_eq!(account.artifact_worth(ArtifactKind::SagesRing), 2);
        assert_eq!(Account::parse(&account.to_text()), account);
        assert_eq!(Account::parse("garbage"), Account::default());
    }

    #[test]
    fn supports_are_unlocked_once() {
        let mut account = Account::default();
        account.add(Loot::Support(SupportKind::Echo));
        account.add(Loot::Support(SupportKind::Echo));
        assert_eq!(account.supports, vec![SupportKind::Echo]);
    }

    #[test]
    fn copies_fuse_up_to_legendary() {
        let mut account = Account::default();
        let ring = ArtifactKind::SagesRing;
        assert_eq!(account.add(Loot::Artifact(ring, 1)), None, "a first copy is not a fusion");
        assert_eq!(account.artifact_tier(ring), 1);
        assert_eq!(account.add(Loot::Artifact(ring, 1)), Some(2));
        assert_eq!(account.add(Loot::Artifact(ring, 1)), None);
        assert_eq!(account.artifact_progress(ring), Some((3, 4)));
        assert_eq!(account.add(Loot::Artifact(ring, 1)), Some(3));
        assert_eq!(account.add(Loot::Artifact(ring, 4)), Some(4), "a tier-4 drop is worth 8 copies");
        assert_eq!(account.artifact_tier(ring), 4);
        account.add(Loot::Artifact(ring, 5));
        assert_eq!(account.artifact_tier(ring), MAX_ARTIFACT_TIER);
        assert_eq!(account.artifact_progress(ring), None);
        assert_eq!(account.maxed_artifacts(), vec![ring]);
    }

    #[test]
    fn harder_maps_drop_better_artifacts() {
        assert_eq!((1..=10).map(max_drop_tier).collect::<Vec<_>>(), vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn tier_one_is_the_baseline() {
        let t = tier_scaling(1);
        assert_eq!((t.health, t.damage, t.xp), (1.0, 1.0, 1.0));
        let t4 = tier_scaling(4);
        assert!((t4.health - 2.197).abs() < 1e-3 && (t4.xp - 1.75).abs() < 1e-6);
    }
}
