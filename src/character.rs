//! Character level and experience. Levels give skill points; artifact
//! capacity grows with level (design §7).

pub const MAX_LEVEL: u32 = 50;
/// Skill points at level 1; each level after that adds one.
pub const STARTING_POINTS: u32 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Character {
    pub level: u32,
    /// Experience into the current level.
    pub xp: u32,
}

impl Default for Character {
    fn default() -> Self {
        Character { level: 1, xp: 0 }
    }
}

impl Character {
    /// Experience needed to go from `level` to the next one.
    pub fn xp_to_next(level: u32) -> u32 {
        (400.0 * (level as f32).powf(1.1)).round() as u32
    }

    /// Adds experience; returns how many levels were gained.
    pub fn add_xp(&mut self, amount: u32) -> u32 {
        let before = self.level;
        self.xp += amount;
        while self.level < MAX_LEVEL && self.xp >= Self::xp_to_next(self.level) {
            self.xp -= Self::xp_to_next(self.level);
            self.level += 1;
        }
        if self.level == MAX_LEVEL {
            self.xp = 0;
        }
        self.level - before
    }

    pub fn skill_points(&self) -> u32 {
        STARTING_POINTS + self.level - 1
    }

    /// Artifact capacity by level: 2 at level 1, then 3, 4, 5, 6 and 7 at
    /// levels 5, 10, 18, 28 and 40.
    pub fn artifact_capacity(&self) -> u8 {
        let steps = [5, 10, 18, 28, 40];
        2 + steps.iter().filter(|&&l| self.level >= l).count() as u8
    }

    /// Progress through the current level, 0 to 1.
    pub fn level_fraction(&self) -> f32 {
        if self.level >= MAX_LEVEL {
            1.0
        } else {
            self.xp as f32 / Self::xp_to_next(self.level) as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_level_one_with_seven_points() {
        let c = Character::default();
        assert_eq!(c.skill_points(), 7);
        assert_eq!(c.artifact_capacity(), 2);
    }

    #[test]
    fn experience_carries_over_levels() {
        let mut c = Character::default();
        let gained = c.add_xp(Character::xp_to_next(1) + Character::xp_to_next(2) + 3);
        assert_eq!(gained, 2);
        assert_eq!(c.level, 3);
        assert_eq!(c.xp, 3);
        assert_eq!(c.skill_points(), 9);
    }

    #[test]
    fn level_caps_at_fifty() {
        let mut c = Character::default();
        c.add_xp(u32::MAX / 2);
        assert_eq!(c.level, MAX_LEVEL);
        assert_eq!(c.skill_points(), 56);
        assert_eq!(c.artifact_capacity(), 7);
    }
}
