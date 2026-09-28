//! The current character, saved as a small text file next to the account:
//! level, experience, skill tree (seed and owned nodes), loadout and tier.
//! Spells, supports and artifacts are stored by name, so a save survives
//! catalogue reordering; unknown names are skipped.
use crate::character::Character;
use crate::content::{ArtifactKind, Build, Slot, SpellId, SupportKind};
use crate::tree::SkillTree;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterSave {
    pub character: Character,
    pub tree_seed: u64,
    pub acquired: Vec<usize>,
    pub slots: Vec<Slot>,
    pub artifacts: Vec<ArtifactKind>,
    pub sandbox: bool,
    pub tier: u32,
}

impl CharacterSave {
    pub fn capture(character: Character, tree: &SkillTree, build: &Build, sandbox: bool, tier: u32) -> Self {
        CharacterSave {
            character,
            tree_seed: tree.seed,
            acquired: (0..tree.nodes.len()).filter(|&i| tree.acquired[i] && i != crate::tree::ORIGIN).collect(),
            slots: build.slots.clone(),
            artifacts: build.artifacts.clone(),
            sandbox,
            tier,
        }
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out += &format!("level={}\nxp={}\n", self.character.level, self.character.xp);
        out += &format!("tree_seed={}\n", self.tree_seed);
        let acquired: Vec<String> = self.acquired.iter().map(|i| i.to_string()).collect();
        out += &format!("acquired={}\n", acquired.join(","));
        for slot in &self.slots {
            out += &format!("spell={}|{}\n", slot.spell.name(), slot.support.map_or("", |s| s.name()));
        }
        for artifact in &self.artifacts {
            out += &format!("artifact={}\n", artifact.name());
        }
        out += &format!("sandbox={}\ntier={}\n", u8::from(self.sandbox), self.tier);
        out
    }

    pub fn parse(text: &str) -> Option<Self> {
        let mut save = CharacterSave {
            character: Character::default(),
            tree_seed: 1,
            acquired: Vec::new(),
            slots: Vec::new(),
            artifacts: Vec::new(),
            sandbox: false,
            tier: 1,
        };
        let mut seen_seed = false;
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else { continue };
            match key.trim() {
                "level" => save.character.level = value.trim().parse().ok()?,
                "xp" => save.character.xp = value.trim().parse().ok()?,
                "tree_seed" => {
                    save.tree_seed = value.trim().parse().ok()?;
                    seen_seed = true;
                }
                "acquired" => save.acquired = value.split(',').filter_map(|v| v.trim().parse().ok()).collect(),
                "spell" => {
                    let (name, support) = value.split_once('|').unwrap_or((value, ""));
                    if let Some(spell) = SpellId::all().find(|s| s.name() == name.trim()) {
                        let support = SupportKind::ALL.into_iter().find(|s| s.name() == support.trim());
                        save.slots.push(Slot { spell, support });
                    }
                }
                "artifact" => {
                    if let Some(a) = ArtifactKind::ALL.into_iter().find(|a| a.name() == value.trim()) {
                        save.artifacts.push(a);
                    }
                }
                "sandbox" => save.sandbox = value.trim() == "1",
                "tier" => save.tier = value.trim().parse().unwrap_or(1),
                _ => {}
            }
        }
        seen_seed.then_some(save)
    }

    fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".arpg-machine").join("character.txt"))
    }

    pub fn load() -> Option<Self> {
        Self::parse(&std::fs::read_to_string(Self::path()?).ok()?)
    }

    pub fn save_text(text: &str) {
        if let Some(path) = Self::path() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(path, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::presets;

    #[test]
    fn round_trips_through_text() {
        let mut tree = SkillTree::generate(4, 12);
        let exit = tree.nodes[crate::tree::ORIGIN].neighbors[0];
        tree.acquire(exit);
        let build = presets()[1].1.clone();
        let mut character = Character::default();
        character.add_xp(900);
        let save = CharacterSave::capture(character, &tree, &build, true, 3);
        let parsed = CharacterSave::parse(&save.to_text()).unwrap();
        assert_eq!(parsed, save);
    }

    #[test]
    fn saves_to_disk_and_loads_back() {
        let dir = std::env::temp_dir().join(format!("arpg-save-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Only this test reads HOME in this crate's tests.
        std::env::set_var("HOME", &dir);
        let tree = SkillTree::generate(6, 7);
        let save = CharacterSave::capture(Character { level: 4, xp: 17 }, &tree, &Build::empty(), false, 2);
        CharacterSave::save_text(&save.to_text());
        assert_eq!(CharacterSave::load(), Some(save));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn unknown_names_are_skipped() {
        let text = "tree_seed=2\nspell=Not A Spell|Hunter\nspell=Arcane Bolt|Nope\nartifact=Nothing\n";
        let save = CharacterSave::parse(text).unwrap();
        assert_eq!(save.slots.len(), 1);
        assert_eq!(save.slots[0].support, None);
        assert!(save.artifacts.is_empty());
    }
}
