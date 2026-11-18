//! What a person gets better at by doing it.

use serde::{Deserialize, Serialize};

/// The highest level of any skill.
pub const TOP_LEVEL: u32 = 10;

/// How much experience the first level takes. Each further level takes more than the last.
const FIRST_LEVEL: u32 = 50;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Skill {
    Foraging,
    Woodcraft,
    Crafting,
    Firemaking,
    Fishing,
    Trapping,
    Cooking,
    Building,
}

impl Skill {
    pub const ALL: [Self; 8] = [
        Self::Foraging,
        Self::Woodcraft,
        Self::Crafting,
        Self::Firemaking,
        Self::Fishing,
        Self::Trapping,
        Self::Cooking,
        Self::Building,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Foraging => "Foraging",
            Self::Woodcraft => "Woodcraft",
            Self::Crafting => "Crafting",
            Self::Firemaking => "Firemaking",
            Self::Fishing => "Fishing",
            Self::Trapping => "Trapping",
            Self::Cooking => "Cooking",
            Self::Building => "Building",
        }
    }
}

/// The experience a person has of each skill.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Skills {
    /// In the order of [`Skill::ALL`], with none for the skills past its end.
    experience: Vec<u32>,
}

impl Skills {
    pub fn experience(&self, skill: Skill) -> u32 {
        self.experience.get(skill as usize).copied().unwrap_or(0)
    }

    /// Returns the level of `skill`, from zero to [`TOP_LEVEL`].
    pub fn level(&self, skill: Skill) -> u32 {
        level_at(self.experience(skill))
    }

    /// Returns how far `skill` is from its level to the next, in hundredths, which is a
    /// hundred at the top level.
    pub fn progress_percent(&self, skill: Skill) -> u32 {
        let experience = self.experience(skill);
        let level = level_at(experience);
        if level == TOP_LEVEL {
            return 100;
        }

        let (from, to) = (experience_for(level), experience_for(level + 1));
        (experience - from) * 100 / (to - from)
    }

    /// Adds `experience` of `skill`, and returns the level reached by it if it is a new one.
    pub(crate) fn gain(&mut self, skill: Skill, experience: u32) -> Option<u32> {
        let index = skill as usize;
        if self.experience.len() <= index {
            self.experience.resize(index + 1, 0);
        }

        let before = level_at(self.experience[index]);
        self.experience[index] = self.experience[index].saturating_add(experience);
        let after = level_at(self.experience[index]);

        (after > before).then_some(after)
    }
}

fn level_at(experience: u32) -> u32 {
    (experience / FIRST_LEVEL).isqrt().min(TOP_LEVEL)
}

/// Returns the least experience that `level` takes.
fn experience_for(level: u32) -> u32 {
    level * level * FIRST_LEVEL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_level_takes_more_experience_than_the_last() {
        assert_eq!(level_at(0), 0);
        assert_eq!(level_at(49), 0);
        assert_eq!(level_at(50), 1);
        assert_eq!(level_at(199), 1);
        assert_eq!(level_at(200), 2);
        assert_eq!(level_at(5_000), TOP_LEVEL);
        assert_eq!(level_at(u32::MAX), TOP_LEVEL);
    }

    #[test]
    fn gaining_tells_of_a_new_level_only() {
        let mut skills = Skills::default();

        assert_eq!(skills.gain(Skill::Fishing, 49), None);
        assert_eq!(skills.gain(Skill::Fishing, 1), Some(1));
        assert_eq!(skills.gain(Skill::Fishing, 10), None);
        assert_eq!(skills.gain(Skill::Fishing, 1_000), Some(4));

        assert_eq!(skills.level(Skill::Fishing), 4);
        assert_eq!(skills.level(Skill::Building), 0);
        assert_eq!(skills.experience(Skill::Foraging), 0);
    }

    #[test]
    fn progress_is_counted_from_one_level_to_the_next() {
        let mut skills = Skills::default();
        assert_eq!(skills.progress_percent(Skill::Cooking), 0);

        skills.gain(Skill::Cooking, 25);
        assert_eq!(skills.progress_percent(Skill::Cooking), 50);

        skills.gain(Skill::Cooking, 25);
        assert_eq!(skills.progress_percent(Skill::Cooking), 0);

        skills.gain(Skill::Cooking, 100_000);
        assert_eq!(skills.progress_percent(Skill::Cooking), 100);
    }

    #[test]
    fn skills_are_listed_in_the_order_they_are_kept_in() {
        for (index, skill) in Skill::ALL.into_iter().enumerate() {
            assert_eq!(skill as usize, index);
            assert!(!skill.name().is_empty());
        }
    }
}
