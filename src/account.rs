//! Account-wide progress that outlives characters: the highest map tier
//! unlocked and the collection of supports and artifacts found. Saved as a
//! small text file in the home directory.
use crate::content::{ArtifactKind, Loot, SupportKind};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    /// Highest map tier you may start. Beating a tier's boss unlocks the next.
    pub max_tier: u32,
    /// Supports are unlocks: found once, usable by every character.
    pub supports: Vec<SupportKind>,
    /// Artifacts found, with how many copies (copies will feed fusion).
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
                        account.add(Loot::Artifact(a));
                        if let Some(entry) = account.artifacts.iter_mut().find(|(k, _)| *k == a) {
                            entry.1 = count.parse().unwrap_or(1).max(1);
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

    pub fn artifact_copies(&self, artifact: ArtifactKind) -> u32 {
        self.artifacts.iter().find(|(a, _)| *a == artifact).map_or(0, |(_, n)| *n)
    }

    pub fn add(&mut self, loot: Loot) {
        match loot {
            Loot::Support(s) => {
                if !self.supports.contains(&s) {
                    self.supports.push(s);
                }
            }
            Loot::Artifact(a) => match self.artifacts.iter_mut().find(|(k, _)| *k == a) {
                Some(entry) => entry.1 += 1,
                None => self.artifacts.push((a, 1)),
            },
        }
    }
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
        account.add(Loot::Artifact(ArtifactKind::SagesRing));
        account.add(Loot::Artifact(ArtifactKind::SagesRing));
        assert_eq!(account.artifact_copies(ArtifactKind::SagesRing), 2);
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
    fn tier_one_is_the_baseline() {
        let t = tier_scaling(1);
        assert_eq!((t.health, t.damage, t.xp), (1.0, 1.0, 1.0));
        let t4 = tier_scaling(4);
        assert!((t4.health - 2.197).abs() < 1e-3 && (t4.xp - 1.75).abs() < 1e-6);
    }
}
