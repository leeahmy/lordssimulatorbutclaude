//! The place the player and those who live with them have made their own, what they have
//! built there, and what they have learned.

use serde::{Deserialize, Serialize};

use crate::item::Item;
use crate::people::PersonId;
use crate::politics::Law;
use crate::time::Timestamp;
use crate::world::{StructureKind, ThingId};

/// How many hours of work one builder puts in in a day.
pub const WORKDAY: u32 = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settlement {
    pub name: String,
    /// The pile of stones the settlement was founded with, which holds its common store.
    pub cairn: ThingId,
    pub founded: Timestamp,
    /// What has been learned, in the order it was.
    #[serde(default)]
    pub advances: Vec<Advance>,
    /// What is being worked towards.
    #[serde(default)]
    pub pursuit: Option<Advance>,
    /// How far the pursuit has got, in days of one thinker's work.
    #[serde(default)]
    pub insight: u32,
    /// The grain growing in the fields, to be had at harvest.
    #[serde(default)]
    pub growing: u32,
    /// Who leads, which is nobody only until the first moot after a leader is lost.
    #[serde(default)]
    pub leader: Option<PersonId>,
    #[serde(default)]
    pub laws: Vec<Law>,
    /// Whether the leader leaves the running of things to custom: who does what, what is
    /// built, and what is worked out. A leader who is not the player character always does.
    #[serde(default)]
    pub steward: bool,
    /// The year in which a leader was last chosen.
    #[serde(default)]
    pub moot_year: u64,
    /// The days, counted from the epoch, on which the player character last made each
    /// appeal, in the order of [`Appeal::ALL`].
    #[serde(default)]
    pub appealed_on: [Option<u64>; 3],
    /// What the player character has brought to the common store over what they have
    /// taken out of it, in kilocalories and their worth.
    #[serde(default)]
    pub credit: i64,
    /// How much of `credit` people have taken note of already.
    #[serde(default)]
    pub noted_credit: i64,
}

impl Settlement {
    pub fn new(name: String, cairn: ThingId, founded: Timestamp) -> Self {
        Self {
            name,
            cairn,
            founded,
            advances: Vec::new(),
            pursuit: None,
            insight: 0,
            growing: 0,
            leader: None,
            laws: Vec::new(),
            steward: false,
            moot_year: 0,
            appealed_on: [None; 3],
            credit: 0,
            noted_credit: 0,
        }
    }

    pub fn lives_by(&self, law: Law) -> bool {
        self.laws.contains(&law)
    }

    pub fn knows(&self, advance: Advance) -> bool {
        self.advances.contains(&advance)
    }

    /// Returns whether `advance` is neither known nor out of reach for want of another.
    pub fn can_pursue(&self, advance: Advance) -> bool {
        !self.knows(advance) && advance.requires().is_none_or(|needed| self.knows(needed))
    }
}

/// A better way of doing things that a settlement can work out.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Advance {
    Smoking,
    Pottery,
    Boats,
    Herbalism,
    Weaving,
    Farming,
    Masonry,
    Ironworking,
    Writing,
}

impl Advance {
    pub const ALL: [Self; 9] = [
        Self::Smoking,
        Self::Pottery,
        Self::Boats,
        Self::Herbalism,
        Self::Weaving,
        Self::Farming,
        Self::Masonry,
        Self::Ironworking,
        Self::Writing,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Smoking => "Smoking",
            Self::Pottery => "Pottery",
            Self::Boats => "Boats",
            Self::Herbalism => "Herbalism",
            Self::Weaving => "Weaving",
            Self::Farming => "Farming",
            Self::Masonry => "Masonry",
            Self::Ironworking => "Ironworking",
            Self::Writing => "Writing",
        }
    }

    /// Returns what the advance is good for.
    pub fn use_of(self) -> &'static str {
        match self {
            Self::Smoking => "Smokehouses can be built, with which catches are kept for good",
            Self::Pottery => {
                "Clay can be fired in a kiln into pots, and food in the store keeps twice \
                 as long in jars"
            }
            Self::Boats => "Docks can be built, from which fishers go out and catch twice as much",
            Self::Herbalism => "Far fewer die of sickness",
            Self::Weaving => "Everyone is clothed for the winter, pelts or no pelts",
            Self::Farming => "Fields can be cleared, on which farmers grow rye",
            Self::Masonry => "Wells can be dug, for clean water at home",
            Self::Ironworking => {
                "Bog ore can be raked from the lake and smithies built to forge it in, and \
                 iron tools make all work yield a quarter more"
            }
            Self::Writing => "Thinkers build on what was written down, and get half again as far",
        }
    }

    /// Returns how many days of one thinker's work the advance takes.
    pub const fn cost(self) -> u32 {
        match self {
            Self::Smoking => 15,
            Self::Pottery => 25,
            Self::Boats | Self::Herbalism => 30,
            Self::Weaving => 35,
            Self::Farming => 40,
            Self::Masonry => 50,
            Self::Ironworking => 80,
            Self::Writing => 100,
        }
    }

    /// Returns the advance that has to be known first, if any has to.
    pub const fn requires(self) -> Option<Self> {
        match self {
            Self::Masonry | Self::Writing => Some(Self::Pottery),
            Self::Ironworking => Some(Self::Masonry),
            _ => None,
        }
    }
}

/// What it takes to put up a building of a settlement.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub kind: StructureKind,
    pub materials: &'static [(Item, u32)],
    /// How many hours of work it takes.
    pub hours: u32,
    pub needs: Option<Advance>,
    /// How many people it houses.
    pub houses: u32,
    pub use_of: &'static str,
}

/// The buildings that can be planned.
pub const BUILDINGS: [Building; 11] = [
    Building {
        kind: StructureKind::Hut,
        materials: &[(Item::Log, 12), (Item::SpruceBough, 20)],
        hours: 48,
        needs: None,
        houses: 4,
        use_of: "Houses four",
    },
    Building {
        kind: StructureKind::Longhouse,
        materials: &[
            (Item::Log, 40),
            (Item::SpruceBough, 40),
            (Item::BirchBark, 20),
        ],
        hours: 160,
        needs: None,
        houses: 12,
        use_of: "Houses twelve",
    },
    Building {
        kind: StructureKind::Storehouse,
        materials: &[(Item::Log, 16), (Item::BirchBark, 10)],
        hours: 64,
        needs: None,
        houses: 0,
        use_of: "Food in the store keeps twice as long",
    },
    Building {
        kind: StructureKind::Workshop,
        materials: &[(Item::Log, 16), (Item::Stone, 6)],
        hours: 64,
        needs: None,
        houses: 0,
        use_of: "Crafters make twice as much, and thinkers get half again as far",
    },
    Building {
        kind: StructureKind::Smokehouse,
        materials: &[(Item::Log, 8), (Item::Stone, 10)],
        hours: 40,
        needs: Some(Advance::Smoking),
        houses: 0,
        use_of: "Fish and meat brought in are smoked, and keep for good",
    },
    Building {
        kind: StructureKind::Field,
        materials: &[(Item::Stick, 20)],
        hours: 80,
        needs: Some(Advance::Farming),
        houses: 0,
        use_of: "Three farmers can work it",
    },
    Building {
        kind: StructureKind::Hall,
        materials: &[(Item::Log, 30), (Item::SpruceBough, 30), (Item::Stone, 10)],
        hours: 120,
        needs: None,
        houses: 0,
        use_of: "A place to meet and to be heard. A town cannot be without one",
    },
    Building {
        kind: StructureKind::Kiln,
        materials: &[(Item::Stone, 20), (Item::Clay, 10)],
        hours: 40,
        needs: Some(Advance::Pottery),
        houses: 0,
        use_of: "Clay pots can be fired by it",
    },
    Building {
        kind: StructureKind::Dock,
        materials: &[(Item::Log, 12), (Item::Cordage, 4)],
        hours: 48,
        needs: Some(Advance::Boats),
        houses: 0,
        use_of: "Fishers go out in boats, and catch twice as much",
    },
    Building {
        kind: StructureKind::Smithy,
        materials: &[(Item::Stone, 20), (Item::Log, 10)],
        hours: 80,
        needs: Some(Advance::Ironworking),
        houses: 0,
        use_of: "Iron tools can be forged in it",
    },
    Building {
        kind: StructureKind::Well,
        materials: &[(Item::Stone, 30), (Item::Log, 4)],
        hours: 80,
        needs: Some(Advance::Masonry),
        houses: 0,
        use_of: "Clean water that never freezes over",
    },
];

/// Returns what it takes to put up a building of `kind`, if it is something that is planned
/// and built by a settlement.
pub fn building(kind: StructureKind) -> Option<&'static Building> {
    BUILDINGS.iter().find(|building| building.kind == kind)
}

/// Returns how many people a structure of `kind` houses.
pub fn houses(kind: StructureKind) -> u32 {
    match kind {
        StructureKind::Cabin => 4,
        StructureKind::LeanTo => 2,
        kind => building(kind).map_or(0, |building| building.houses),
    }
}

/// How many farmers one field takes.
pub const FARMERS_PER_FIELD: u32 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    fn settlement() -> Settlement {
        let mut world = crate::World::default();
        let cairn = world.add_thing(crate::world::Thing::Structure(
            crate::world::Structure::new(Default::default(), StructureKind::Cairn),
        ));

        Settlement::new("Lordstead".to_owned(), cairn, Timestamp::EPOCH)
    }

    #[test]
    fn advances_build_on_one_another() {
        let mut settlement = settlement();
        assert!(settlement.can_pursue(Advance::Pottery));
        assert!(!settlement.can_pursue(Advance::Masonry));
        assert!(!settlement.can_pursue(Advance::Ironworking));

        settlement.advances.push(Advance::Pottery);
        assert!(!settlement.can_pursue(Advance::Pottery));
        assert!(settlement.can_pursue(Advance::Masonry));
        assert!(settlement.can_pursue(Advance::Writing));
    }

    #[test]
    fn every_advance_can_be_reached_and_says_what_it_is_good_for() {
        let mut settlement = settlement();

        while settlement.advances.len() < Advance::ALL.len() {
            let next = Advance::ALL
                .into_iter()
                .find(|advance| settlement.can_pursue(*advance))
                .expect("something can always be pursued until all is known");
            settlement.advances.push(next);
        }

        for advance in Advance::ALL {
            assert!(advance.cost() > 0);
            assert!(!advance.name().is_empty() && !advance.use_of().is_empty());
        }
    }

    #[test]
    fn buildings_take_materials_and_work_and_some_house_people() {
        for plan in &BUILDINGS {
            assert!(!plan.materials.is_empty() && plan.hours > 0, "{plan:?}");
            assert_eq!(building(plan.kind), Some(plan));
            assert_eq!(houses(plan.kind), plan.houses);
        }
        assert_eq!(houses(StructureKind::Cabin), 4);
        assert_eq!(houses(StructureKind::Campfire), 0);
        assert_eq!(building(StructureKind::Cabin), None);
    }
}
