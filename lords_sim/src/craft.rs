//! What a person can make, and of what.

use serde::{Deserialize, Serialize};

use crate::item::Item;
use crate::settlement::Advance;
use crate::skill::Skill;
use crate::world::StructureKind;

/// Something that can be made, built, set up, or prepared.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Recipe {
    Cordage,
    StoneAxe,
    BoneNeedle,
    FishingRod,
    BarkPot,
    Snare,
    FishTrap,
    ClayPot,
    IronAxe,
    FurHat,
    FurBoots,
    FurCoat,

    Campfire,
    Bed,
    LeanTo,
    DryingRack,
    Cabin,
    SetSnare,
    SetFishTrap,
    Cairn,

    Butcher,
    CookMeat,
    CookFish,
    BoilWater,
}

/// What a recipe comes to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Product {
    Items(&'static [(Item, u32)]),
    Structure(StructureKind),
}

/// Which list a recipe belongs in.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Trade {
    Tools,
    Clothing,
    Camp,
    Food,
}

impl Trade {
    pub const ALL: [Self; 4] = [Self::Tools, Self::Clothing, Self::Camp, Self::Food];

    pub fn name(self) -> &'static str {
        match self {
            Self::Tools => "Tools",
            Self::Clothing => "Clothing",
            Self::Camp => "Camp",
            Self::Food => "Food and drink",
        }
    }
}

impl Recipe {
    pub const ALL: [Self; 24] = [
        Self::Cordage,
        Self::StoneAxe,
        Self::BoneNeedle,
        Self::FishingRod,
        Self::BarkPot,
        Self::Snare,
        Self::FishTrap,
        Self::ClayPot,
        Self::IronAxe,
        Self::FurHat,
        Self::FurBoots,
        Self::FurCoat,
        Self::Campfire,
        Self::Bed,
        Self::LeanTo,
        Self::DryingRack,
        Self::Cabin,
        Self::SetSnare,
        Self::SetFishTrap,
        Self::Cairn,
        Self::Butcher,
        Self::CookMeat,
        Self::CookFish,
        Self::BoilWater,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Cordage => "Twist cordage",
            Self::StoneAxe => "Make a stone axe",
            Self::BoneNeedle => "Carve a bone needle",
            Self::FishingRod => "Make a fishing rod",
            Self::BarkPot => "Fold a birch-bark pot",
            Self::Snare => "Tie a snare",
            Self::FishTrap => "Weave a fish trap",
            Self::ClayPot => "Fire a clay pot",
            Self::IronAxe => "Forge an iron axe",
            Self::FurHat => "Sew a fur hat and mittens",
            Self::FurBoots => "Sew fur boots",
            Self::FurCoat => "Sew a fur coat",
            Self::Campfire => "Light a campfire",
            Self::Bed => "Lay a bough bed",
            Self::LeanTo => "Build a lean-to",
            Self::DryingRack => "Build a drying rack",
            Self::Cabin => "Build a log cabin",
            Self::SetSnare => "Set a snare",
            Self::SetFishTrap => "Set a fish trap",
            Self::Cairn => "Found a settlement",
            Self::Butcher => "Butcher a hare",
            Self::CookMeat => "Cook meat",
            Self::CookFish => "Cook perch",
            Self::BoilWater => "Boil water",
        }
    }

    /// Returns what the one who has made it is told.
    pub(crate) fn done(self) -> &'static str {
        match self {
            Self::Cordage => "You twist the roots into a length of cordage.",
            Self::StoneAxe => "You knap an edge on the stone and lash it to a handle: an axe.",
            Self::BoneNeedle => "You carve a needle out of the bone.",
            Self::FishingRod => "You carve a hook, tie it to a line, and the line to a rod.",
            Self::BarkPot => "You fold and stitch the bark into a pot that holds water.",
            Self::Snare => "You tie the cordage into a snare.",
            Self::FishTrap => "You weave the sticks into a fish trap.",
            Self::ClayPot => "You shape the clay and fire it into a pot that holds three liters.",
            Self::IronAxe => {
                "You smelt the ore, and hammer the bloom into an axe that will not break."
            }
            Self::FurHat => "You sew a fur hat and a pair of mittens.",
            Self::FurBoots => "You sew a pair of fur boots.",
            Self::FurCoat => "You sew a fur coat.",
            Self::Campfire => "The tinder catches, and soon the fire burns.",
            Self::Bed => "You lay the boughs into a springy bed.",
            Self::LeanTo => "You finish the lean-to. It keeps the rain and the wind off.",
            Self::DryingRack => "You lash together a rack to dry food on.",
            Self::Cabin => "You lay the last of the roof. The cabin is finished.",
            Self::SetSnare => "You set the snare on a hare trail.",
            Self::SetFishTrap => "You sink the fish trap by the shore.",
            Self::Cairn => "You raise a cairn of stones. This is where you will stay.",
            Self::Butcher => "You skin and butcher the hare.",
            Self::CookMeat => "You roast a piece of meat over the fire.",
            Self::CookFish => "You roast a perch over the fire.",
            Self::BoilWater => "You boil a potful of water with stones from the fire.",
        }
    }

    pub const fn trade(self) -> Trade {
        match self {
            Self::Cordage
            | Self::StoneAxe
            | Self::BoneNeedle
            | Self::FishingRod
            | Self::BarkPot
            | Self::Snare
            | Self::FishTrap
            | Self::ClayPot
            | Self::IronAxe => Trade::Tools,
            Self::FurHat | Self::FurBoots | Self::FurCoat => Trade::Clothing,
            Self::Campfire
            | Self::Bed
            | Self::LeanTo
            | Self::DryingRack
            | Self::Cabin
            | Self::SetSnare
            | Self::SetFishTrap
            | Self::Cairn => Trade::Camp,
            Self::Butcher | Self::CookMeat | Self::CookFish | Self::BoilWater => Trade::Food,
        }
    }

    /// Returns what is used up, and how many of each.
    pub const fn materials(self) -> &'static [(Item, u32)] {
        match self {
            Self::Cordage => &[(Item::SpruceRoot, 2)],
            Self::StoneAxe => &[(Item::Stone, 2), (Item::Stick, 1), (Item::Cordage, 1)],
            Self::BoneNeedle => &[(Item::Bone, 1)],
            Self::FishingRod => &[(Item::Stick, 2), (Item::Cordage, 1)],
            Self::BarkPot => &[(Item::BirchBark, 2), (Item::Cordage, 1)],
            Self::Snare => &[(Item::Cordage, 1)],
            Self::FishTrap => &[(Item::Stick, 12), (Item::Cordage, 2)],
            Self::ClayPot => &[(Item::Clay, 2), (Item::Stick, 4)],
            Self::IronAxe => &[(Item::BogOre, 4), (Item::Log, 2), (Item::Stick, 1)],
            Self::FurHat | Self::FurBoots => &[(Item::HarePelt, 2), (Item::Cordage, 1)],
            Self::FurCoat => &[(Item::HarePelt, 5), (Item::Cordage, 2)],
            Self::Campfire => &[(Item::Stick, 5), (Item::BirchBark, 1)],
            Self::Bed => &[(Item::SpruceBough, 15)],
            Self::LeanTo => &[
                (Item::Stick, 15),
                (Item::SpruceBough, 25),
                (Item::Cordage, 2),
            ],
            Self::DryingRack => &[(Item::Stick, 8), (Item::Cordage, 2)],
            Self::Cabin => &[
                (Item::Log, 30),
                (Item::SpruceBough, 40),
                (Item::BirchBark, 20),
            ],
            Self::SetSnare => &[(Item::Snare, 1), (Item::Stick, 1)],
            Self::SetFishTrap => &[(Item::FishTrap, 1)],
            Self::Cairn => &[(Item::Stone, 10)],
            Self::Butcher => &[(Item::HareCarcass, 1)],
            Self::CookMeat => &[(Item::RawMeat, 1)],
            Self::CookFish => &[(Item::RawFish, 1)],
            Self::BoilWater => &[(Item::Water, 1)],
        }
    }

    /// Returns what has to be at hand but is not used up.
    pub const fn tools(self) -> &'static [Item] {
        match self {
            Self::ClayPot | Self::IronAxe => &[],
            Self::StoneAxe
            | Self::BoneNeedle
            | Self::FishingRod
            | Self::BarkPot
            | Self::FishTrap
            | Self::DryingRack
            | Self::Butcher => &[Item::Knife],
            Self::FurHat | Self::FurBoots | Self::FurCoat => &[Item::Knife, Item::BoneNeedle],
            Self::Campfire => &[Item::FireSteel],
            Self::Cabin => &[Item::StoneAxe, Item::Knife],
            Self::BoilWater => &[Item::BarkPot],
            Self::Cordage
            | Self::Snare
            | Self::Bed
            | Self::LeanTo
            | Self::SetSnare
            | Self::SetFishTrap
            | Self::Cairn
            | Self::CookMeat
            | Self::CookFish => &[],
        }
    }

    /// Returns what the settlement has to know for it to be made, and the building it
    /// has to be made at, if it takes any such thing.
    pub const fn craft(self) -> Option<(Advance, StructureKind)> {
        match self {
            Self::ClayPot => Some((Advance::Pottery, StructureKind::Kiln)),
            Self::IronAxe => Some((Advance::Ironworking, StructureKind::Smithy)),
            _ => None,
        }
    }

    /// Returns whether it takes a burning fire within reach.
    pub const fn needs_fire(self) -> bool {
        matches!(self, Self::CookMeat | Self::CookFish | Self::BoilWater)
    }

    /// Returns whether it takes water within reach.
    pub const fn needs_water(self) -> bool {
        matches!(self, Self::SetFishTrap)
    }

    pub const fn product(self) -> Product {
        match self {
            Self::Cordage => Product::Items(&[(Item::Cordage, 1)]),
            Self::StoneAxe => Product::Items(&[(Item::StoneAxe, 1)]),
            Self::BoneNeedle => Product::Items(&[(Item::BoneNeedle, 1)]),
            Self::FishingRod => Product::Items(&[(Item::FishingRod, 1)]),
            Self::BarkPot => Product::Items(&[(Item::BarkPot, 1)]),
            Self::Snare => Product::Items(&[(Item::Snare, 1)]),
            Self::FishTrap => Product::Items(&[(Item::FishTrap, 1)]),
            Self::ClayPot => Product::Items(&[(Item::ClayPot, 1)]),
            Self::IronAxe => Product::Items(&[(Item::IronAxe, 1)]),
            Self::FurHat => Product::Items(&[(Item::FurHat, 1)]),
            Self::FurBoots => Product::Items(&[(Item::FurBoots, 1)]),
            Self::FurCoat => Product::Items(&[(Item::FurCoat, 1)]),
            Self::Campfire => Product::Structure(StructureKind::Campfire),
            Self::Bed => Product::Structure(StructureKind::Bed),
            Self::LeanTo => Product::Structure(StructureKind::LeanTo),
            Self::DryingRack => Product::Structure(StructureKind::DryingRack),
            Self::Cabin => Product::Structure(StructureKind::Cabin),
            Self::SetSnare => Product::Structure(StructureKind::Snare),
            Self::SetFishTrap => Product::Structure(StructureKind::FishTrap),
            Self::Cairn => Product::Structure(StructureKind::Cairn),
            Self::Butcher => {
                Product::Items(&[(Item::RawMeat, 3), (Item::HarePelt, 1), (Item::Bone, 2)])
            }
            Self::CookMeat => Product::Items(&[(Item::CookedMeat, 1)]),
            Self::CookFish => Product::Items(&[(Item::CookedFish, 1)]),
            Self::BoilWater => Product::Items(&[(Item::BoiledWater, 1)]),
        }
    }

    /// Returns how many in-game seconds it takes someone unskilled and well.
    pub const fn seconds(self) -> u32 {
        const MINUTE: u32 = 60;
        const HOUR: u32 = 60 * MINUTE;

        match self {
            Self::Cordage => 5 * MINUTE,
            Self::StoneAxe => 30 * MINUTE,
            Self::BoneNeedle => 10 * MINUTE,
            Self::FishingRod | Self::BarkPot | Self::Bed | Self::Butcher => 15 * MINUTE,
            Self::Snare => 2 * MINUTE,
            Self::FishTrap => 45 * MINUTE,
            Self::ClayPot => 40 * MINUTE,
            Self::IronAxe => 3 * HOUR,
            Self::FurHat => 90 * MINUTE,
            Self::FurBoots => 2 * HOUR,
            Self::FurCoat => 4 * HOUR,
            Self::Campfire | Self::SetSnare => 3 * MINUTE,
            Self::LeanTo | Self::Cairn => 90 * MINUTE,
            Self::DryingRack => 30 * MINUTE,
            Self::Cabin => 14 * HOUR,
            Self::SetFishTrap | Self::BoilWater => 5 * MINUTE,
            Self::CookMeat => 10 * MINUTE,
            Self::CookFish => 7 * MINUTE,
        }
    }

    /// Returns the skill it takes, and adds to.
    pub const fn skill(self) -> Skill {
        match self {
            Self::ClayPot | Self::IronAxe => Skill::Crafting,
            Self::Cordage
            | Self::StoneAxe
            | Self::BoneNeedle
            | Self::FishingRod
            | Self::BarkPot
            | Self::FurHat
            | Self::FurBoots
            | Self::FurCoat => Skill::Crafting,
            Self::Snare | Self::SetSnare | Self::Butcher => Skill::Trapping,
            Self::FishTrap | Self::SetFishTrap => Skill::Fishing,
            Self::Campfire => Skill::Firemaking,
            Self::Bed | Self::LeanTo | Self::DryingRack | Self::Cabin | Self::Cairn => {
                Skill::Building
            }
            Self::CookMeat | Self::CookFish | Self::BoilWater => Skill::Cooking,
        }
    }

    /// Returns whether it is done again and again for as long as there is what it takes.
    pub const fn repeats(self) -> bool {
        matches!(
            self,
            Self::Cordage | Self::CookMeat | Self::CookFish | Self::BoilWater
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_recipe_is_listed_once_in_order() {
        assert!(Recipe::ALL.is_sorted());
        assert!(Recipe::ALL.windows(2).all(|pair| pair[0] != pair[1]));
        assert_eq!(Recipe::ALL.last(), Some(&Recipe::BoilWater));
    }

    #[test]
    fn every_recipe_takes_something_and_time_and_comes_to_something() {
        for recipe in Recipe::ALL {
            assert!(!recipe.materials().is_empty(), "{recipe:?}");
            assert!(recipe.materials().iter().all(|(_, count)| *count > 0));
            assert!(recipe.seconds() >= 60, "{recipe:?}");
            assert!(!recipe.name().is_empty() && !recipe.done().is_empty());
            if let Product::Items(items) = recipe.product() {
                assert!(!items.is_empty(), "{recipe:?}");
            }
        }
    }

    /// Everything that is made of something can be made starting from what is gathered,
    /// caught, or had from the start.
    #[test]
    fn nothing_takes_what_cannot_be_come_by() {
        let mut had = vec![
            Item::Stick,
            Item::Log,
            Item::SpruceBough,
            Item::BirchBark,
            Item::SpruceRoot,
            Item::Stone,
            Item::Clay,
            Item::BogOre,
            Item::Knife,
            Item::FireSteel,
            Item::HareCarcass,
            Item::RawFish,
            Item::Water,
        ];

        let mut left: Vec<Recipe> = Recipe::ALL.to_vec();
        loop {
            let before = left.len();
            left.retain(|recipe| {
                let needs = recipe.materials().iter().map(|(item, _)| item);
                let makeable = needs.chain(recipe.tools()).all(|item| had.contains(item));
                if makeable && let Product::Items(items) = recipe.product() {
                    had.extend(items.iter().map(|(item, _)| *item));
                }
                !makeable
            });
            if left.len() == before {
                break;
            }
        }

        assert_eq!(left, []);
    }
}
