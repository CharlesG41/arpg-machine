//! Spells, supports and the spell bar.

/// Damage/visual family. The renderer picks colors from this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Arcane,
    Frost,
    Fire,
    Poison,
    Physical,
    Lightning,
}

/// How a spell reaches its targets; this also decides its default targeting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Nearest enemy.
    Projectile,
    /// Nearest enemy, then jumps.
    Chain,
    /// Toward the nearest enemy in range.
    Cone,
    /// Densest pack in range.
    GroundZone,
    /// Around the player, no targeting.
    Nova,
    /// Always around the player.
    Orbit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpellKind {
    ArcaneBolt,
    ChainLightning,
    FrostNova,
    FlameCone,
    VenomPool,
    OrbitBlades,
}

impl SpellKind {
    pub const ALL: [SpellKind; 6] = [
        SpellKind::ArcaneBolt,
        SpellKind::ChainLightning,
        SpellKind::FrostNova,
        SpellKind::FlameCone,
        SpellKind::VenomPool,
        SpellKind::OrbitBlades,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SpellKind::ArcaneBolt => "Arcane Bolt",
            SpellKind::ChainLightning => "Chain Lightning",
            SpellKind::FrostNova => "Frost Nova",
            SpellKind::FlameCone => "Flame Breath",
            SpellKind::VenomPool => "Venom Pool",
            SpellKind::OrbitBlades => "Orbiting Blades",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            SpellKind::ArcaneBolt => "Projectile at the nearest enemy.",
            SpellKind::ChainLightning => "Strikes the nearest enemy, then jumps to 3 more.",
            SpellKind::FrostNova => "Burst around you. Applies Chill (slows).",
            SpellKind::FlameCone => "Cone of fire toward the nearest enemy. Applies 4 Burn.",
            SpellKind::VenomPool => "Pool on the densest pack. Applies Poison.",
            SpellKind::OrbitBlades => "Three blades circle you at all times.",
        }
    }

    /// Slots taken on the bar (God Tower sizes).
    pub fn size(self) -> u8 {
        match self {
            SpellKind::FlameCone | SpellKind::VenomPool => 2,
            _ => 1,
        }
    }

    pub fn shape(self) -> Shape {
        match self {
            SpellKind::ArcaneBolt => Shape::Projectile,
            SpellKind::ChainLightning => Shape::Chain,
            SpellKind::FrostNova => Shape::Nova,
            SpellKind::FlameCone => Shape::Cone,
            SpellKind::VenomPool => Shape::GroundZone,
            SpellKind::OrbitBlades => Shape::Orbit,
        }
    }

    pub fn element(self) -> Element {
        match self {
            SpellKind::ArcaneBolt => Element::Arcane,
            SpellKind::ChainLightning => Element::Lightning,
            SpellKind::FrostNova => Element::Frost,
            SpellKind::FlameCone => Element::Fire,
            SpellKind::VenomPool => Element::Poison,
            SpellKind::OrbitBlades => Element::Physical,
        }
    }

    /// Seconds between casts at 100% cooldown rate.
    pub fn cooldown(self) -> f32 {
        match self {
            SpellKind::ArcaneBolt => 0.55,
            SpellKind::ChainLightning => 1.1,
            SpellKind::FrostNova => 2.2,
            SpellKind::FlameCone => 1.0,
            SpellKind::VenomPool => 3.0,
            SpellKind::OrbitBlades => 0.0,
        }
    }

    pub fn damage(self) -> f32 {
        match self {
            SpellKind::ArcaneBolt => 14.0,
            SpellKind::ChainLightning => 16.0,
            SpellKind::FrostNova => 12.0,
            SpellKind::FlameCone => 9.0,
            SpellKind::VenomPool => 3.0,
            SpellKind::OrbitBlades => 8.0,
        }
    }

    /// Whether the spell picks a target (and so can use targeting supports).
    pub fn targets(self) -> bool {
        !matches!(self.shape(), Shape::Nova | Shape::Orbit)
    }

    pub fn has_area(self) -> bool {
        matches!(self.shape(), Shape::Cone | Shape::GroundZone | Shape::Nova | Shape::Orbit)
    }

    /// Whether the spell casts on a cooldown (Orbit is always on).
    pub fn casts(self) -> bool {
        self.shape() != Shape::Orbit
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SupportKind {
    Multishot,
    Pierce,
    Amplify,
    Contagion,
    Combustion,
    Frenzy,
    Hunter,
    Echo,
}

impl SupportKind {
    pub const ALL: [SupportKind; 8] = [
        SupportKind::Multishot,
        SupportKind::Pierce,
        SupportKind::Amplify,
        SupportKind::Contagion,
        SupportKind::Combustion,
        SupportKind::Frenzy,
        SupportKind::Hunter,
        SupportKind::Echo,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SupportKind::Multishot => "Multishot",
            SupportKind::Pierce => "Pierce",
            SupportKind::Amplify => "Amplify",
            SupportKind::Contagion => "Contagion",
            SupportKind::Combustion => "Combustion",
            SupportKind::Frenzy => "Frenzy",
            SupportKind::Hunter => "Hunter",
            SupportKind::Echo => "Echo",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            SupportKind::Multishot => "+2 projectiles (or +2 jumps), each aimed at a different enemy. −20% damage.",
            SupportKind::Pierce => "Projectiles pass through 3 more enemies.",
            SupportKind::Amplify => "+50% area.",
            SupportKind::Contagion => "This spell applies 3 Poison. A Poisoned enemy that dies spreads its Poison around it.",
            SupportKind::Combustion => "This spell applies 3 Burn. An enemy that dies while Burning explodes.",
            SupportKind::Frenzy => "Kills by this spell grant momentum: +3% cooldown rate per stack (max 20). It all drops after 2 s without a kill.",
            SupportKind::Hunter => "Targeting: bosses first, then rares, then the nearest enemy.",
            SupportKind::Echo => "Also casts whenever the spell to its left casts.",
        }
    }

    /// Whether this support does something on this spell.
    pub fn fits(self, spell: SpellKind) -> bool {
        match self {
            SupportKind::Multishot => matches!(spell.shape(), Shape::Projectile | Shape::Chain),
            SupportKind::Pierce => spell.shape() == Shape::Projectile,
            SupportKind::Amplify => spell.has_area(),
            SupportKind::Hunter => spell.targets(),
            SupportKind::Echo => spell.casts(),
            SupportKind::Contagion | SupportKind::Combustion | SupportKind::Frenzy => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub spell: SpellKind,
    pub support: Option<SupportKind>,
}

/// The spell bar: ordered left to right. Adjacency reads the neighbors in this order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    pub slots: Vec<Slot>,
    pub capacity: u8,
}

impl Build {
    pub const DEFAULT_CAPACITY: u8 = 6;

    pub fn new(slots: Vec<Slot>) -> Self {
        Build { slots, capacity: Self::DEFAULT_CAPACITY }
    }

    pub fn used(&self) -> u8 {
        self.slots.iter().map(|s| s.spell.size()).sum()
    }

    pub fn contains(&self, spell: SpellKind) -> bool {
        self.slots.iter().any(|s| s.spell == spell)
    }

    pub fn support_used(&self, support: SupportKind) -> bool {
        self.slots.iter().any(|s| s.support == Some(support))
    }

    pub fn can_add(&self, spell: SpellKind) -> bool {
        !self.contains(spell) && self.used() + spell.size() <= self.capacity
    }

    pub fn add(&mut self, spell: SpellKind) -> bool {
        if !self.can_add(spell) {
            return false;
        }
        self.slots.push(Slot { spell, support: None });
        true
    }

    /// Sockets a support; each support may appear once per build, so it moves
    /// from wherever it was.
    pub fn socket(&mut self, index: usize, support: SupportKind) -> bool {
        if index >= self.slots.len() || !support.fits(self.slots[index].spell) {
            return false;
        }
        for slot in &mut self.slots {
            if slot.support == Some(support) {
                slot.support = None;
            }
        }
        self.slots[index].support = Some(support);
        true
    }

    pub fn is_valid(&self) -> bool {
        let unique_spells = self.slots.iter().enumerate().all(|(i, a)| self.slots[i + 1..].iter().all(|b| b.spell != a.spell));
        let unique_supports = self
            .slots
            .iter()
            .enumerate()
            .all(|(i, a)| a.support.is_none() || self.slots[i + 1..].iter().all(|b| b.support != a.support));
        let fitting = self.slots.iter().all(|s| s.support.map_or(true, |sup| sup.fits(s.spell)));
        self.used() <= self.capacity && unique_spells && unique_supports && fitting
    }
}

/// Ready-made builds to compare how different machines clear.
pub fn presets() -> Vec<(&'static str, Build)> {
    use SpellKind::*;
    use SupportKind::*;
    let slot = |spell, support| Slot { spell, support };
    vec![
        (
            "Starter",
            Build::new(vec![slot(ArcaneBolt, None), slot(FrostNova, None)]),
        ),
        (
            "Contagion",
            Build::new(vec![
                slot(VenomPool, Some(Amplify)),
                slot(ArcaneBolt, Some(Contagion)),
                slot(ChainLightning, Some(Hunter)),
                slot(FrostNova, Some(Echo)),
            ]),
        ),
        (
            "Combustion",
            Build::new(vec![
                slot(FlameCone, Some(Combustion)),
                slot(OrbitBlades, Some(Amplify)),
                slot(ArcaneBolt, Some(Multishot)),
                slot(ChainLightning, Some(Hunter)),
            ]),
        ),
        (
            "Frenzy",
            Build::new(vec![
                slot(ArcaneBolt, Some(Pierce)),
                slot(ChainLightning, Some(Multishot)),
                slot(FrostNova, Some(Frenzy)),
                slot(OrbitBlades, Some(Combustion)),
                slot(FlameCone, Some(Hunter)),
            ]),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_valid() {
        for (name, build) in presets() {
            assert!(build.is_valid(), "preset {name} is invalid");
        }
    }

    #[test]
    fn support_moves_instead_of_duplicating() {
        let mut build = Build::new(vec![]);
        build.add(SpellKind::ArcaneBolt);
        build.add(SpellKind::ChainLightning);
        assert!(build.socket(0, SupportKind::Hunter));
        assert!(build.socket(1, SupportKind::Hunter));
        assert_eq!(build.slots[0].support, None);
        assert_eq!(build.slots[1].support, Some(SupportKind::Hunter));
        assert!(build.is_valid());
    }

    #[test]
    fn capacity_counts_spell_sizes() {
        let mut build = Build::new(vec![]);
        assert!(build.add(SpellKind::FlameCone));
        assert!(build.add(SpellKind::VenomPool));
        assert!(build.add(SpellKind::ArcaneBolt));
        assert!(build.add(SpellKind::FrostNova));
        assert_eq!(build.used(), 6);
        assert!(!build.add(SpellKind::ChainLightning));
    }

    #[test]
    fn supports_only_fit_matching_spells() {
        assert!(!SupportKind::Pierce.fits(SpellKind::FrostNova));
        assert!(!SupportKind::Hunter.fits(SpellKind::OrbitBlades));
        assert!(SupportKind::Amplify.fits(SpellKind::VenomPool));
    }
}
