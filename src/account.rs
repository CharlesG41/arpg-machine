//! Account-wide progress that outlives characters: for now, the highest map
//! tier unlocked. Saved as a tiny text file in the home directory.
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Account {
    /// Highest map tier you may start. Beating a tier's boss unlocks the next.
    pub max_tier: u32,
}

impl Default for Account {
    fn default() -> Self {
        Account { max_tier: 1 }
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
            if let Some(value) = line.trim().strip_prefix("max_tier=") {
                if let Ok(tier) = value.trim().parse::<u32>() {
                    account.max_tier = tier.max(1);
                }
            }
        }
        account
    }

    pub fn to_text(self) -> String {
        format!("max_tier={}\n", self.max_tier)
    }

    /// Loads the saved account, or a fresh one.
    pub fn load() -> Self {
        Self::path().and_then(|p| std::fs::read_to_string(p).ok()).map(|t| Self::parse(&t)).unwrap_or_default()
    }

    pub fn save(self) {
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
        let account = Account { max_tier: 7 };
        assert_eq!(Account::parse(&account.to_text()), account);
        assert_eq!(Account::parse("garbage"), Account::default());
    }

    #[test]
    fn tier_one_is_the_baseline() {
        let t = tier_scaling(1);
        assert_eq!((t.health, t.damage, t.xp), (1.0, 1.0, 1.0));
        let t4 = tier_scaling(4);
        assert!((t4.health - 2.197).abs() < 1e-3 && (t4.xp - 1.75).abs() < 1e-6);
    }
}
