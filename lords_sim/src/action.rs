//! What a person can set about doing.

use serde::{Deserialize, Serialize};

use crate::craft::Recipe;
use crate::item::{Inventory, Item};
use crate::people::Bond;
use crate::politics::Appeal;
use crate::settlement::Advance;
use crate::skill::Skill;
use crate::time::Timestamp;
use crate::world::{Kind, PlantSpecies, StructureKind, Subject, ThingId};

/// Something to do to a subject within reach.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Verb {
    GatherSticks,
    GatherBoughs,
    StripBark,
    DigRoots,
    PickBerries,
    Fell,
    PickUp,
    Drink,
    FillWater,
    Fish,
    AddStick,
    AddLog,
    Light,
    Collect,
    HangFood,
    TakeDown,
    /// Putting all the wood on a fire that it takes, logs first.
    Stoke,
    Talk,
    /// Giving the best food carried.
    Gift,
    /// Asking someone to come and live with the player character.
    Invite,
    /// Asking someone to share the player character's life.
    Court,
    TryForChild,
    /// Putting everything carried but tools, clothes, and water in the common store.
    Deposit,
    /// Working on a building site.
    Build,
    DigClay,
    /// Raking bog ore off the bottom of the shallows.
    RakeOre,
    /// Paying someone in food to think for a while.
    Hire,
}

impl Verb {
    /// Returns what there is to do to something of `kind`.
    pub const fn for_kind(kind: Kind) -> &'static [Self] {
        match kind {
            Kind::Plant(PlantSpecies::Spruce) => &[
                Self::GatherSticks,
                Self::GatherBoughs,
                Self::DigRoots,
                Self::Fell,
            ],
            Kind::Plant(PlantSpecies::Pine | PlantSpecies::Birch) => {
                &[Self::GatherSticks, Self::StripBark, Self::Fell]
            }
            Kind::Plant(PlantSpecies::Bilberry | PlantSpecies::Lingonberry) => &[Self::PickBerries],
            Kind::Rock => &[],
            Kind::Lake => &[
                Self::Drink,
                Self::FillWater,
                Self::Fish,
                Self::DigClay,
                Self::RakeOre,
            ],
            Kind::Pile(_) => &[Self::PickUp],
            Kind::Structure(StructureKind::Campfire) => {
                &[Self::AddStick, Self::AddLog, Self::Stoke]
            }
            Kind::Structure(StructureKind::Fireplace) => &[Self::Light],
            Kind::Structure(StructureKind::Snare) => &[Self::TakeDown],
            Kind::Structure(StructureKind::SprungSnare) => &[Self::Collect],
            Kind::Structure(StructureKind::FishTrap) => &[Self::Collect, Self::TakeDown],
            Kind::Structure(StructureKind::DryingRack) => &[Self::HangFood, Self::Collect],
            Kind::Structure(StructureKind::Cairn | StructureKind::Storehouse) => &[Self::Deposit],
            Kind::Structure(StructureKind::Site) => &[Self::Build],
            Kind::Structure(StructureKind::Well) => &[Self::Drink, Self::FillWater],
            Kind::Structure(_) => &[],
            Kind::Person(Bond::Stranger | Bond::Child) => &[Self::Talk, Self::Gift],
            Kind::Person(Bond::Friend) => &[Self::Talk, Self::Gift, Self::Invite, Self::Court],
            Kind::Person(Bond::Companion) => &[Self::Talk, Self::Gift, Self::Court, Self::Hire],
            Kind::Person(Bond::Partner) => &[Self::Talk, Self::Gift, Self::TryForChild],
        }
    }

    /// Returns what the verb is called when done to something of `kind`.
    pub fn name(self, kind: Kind) -> &'static str {
        match self {
            Self::GatherSticks => "Gather sticks",
            Self::GatherBoughs => "Cut boughs",
            Self::StripBark if kind == Kind::Plant(PlantSpecies::Pine) => "Strip inner bark",
            Self::StripBark => "Strip bark",
            Self::DigRoots => "Dig up roots",
            Self::PickBerries => "Pick berries",
            Self::Fell => "Fell",
            Self::PickUp => "Pick up",
            Self::Drink => "Drink",
            Self::FillWater => "Fill pots",
            Self::Fish => "Fish",
            Self::AddStick => "Add a stick",
            Self::AddLog => "Add a log",
            Self::Light => "Light",
            Self::Collect if matches!(kind, Kind::Structure(StructureKind::DryingRack)) => {
                "Take the food"
            }
            Self::Collect => "Take the catch",
            Self::HangFood => "Hang food to dry",
            Self::TakeDown => "Take down",
            Self::Stoke => "Bank up with all the wood",
            Self::Talk => "Talk",
            Self::Gift => "Give food",
            Self::Invite => "Ask to stay with you",
            Self::Court => "Ask to share your life",
            Self::TryForChild => "Try for a child",
            Self::Deposit => "Put your load in the store",
            Self::Build => "Work on it",
            Self::DigClay => "Dig clay",
            Self::RakeOre => "Rake bog ore",
            Self::Hire => "Hire to think for you",
        }
    }

    /// Returns what the one doing it is said to be doing.
    pub fn doing(self) -> &'static str {
        match self {
            Self::GatherSticks => "Gathering sticks",
            Self::GatherBoughs => "Cutting boughs",
            Self::StripBark => "Stripping bark",
            Self::DigRoots => "Digging up roots",
            Self::PickBerries => "Picking berries",
            Self::Fell => "Felling a tree",
            Self::PickUp => "Picking things up",
            Self::Drink => "Drinking",
            Self::FillWater => "Filling pots",
            Self::Fish => "Fishing",
            Self::AddStick | Self::AddLog | Self::Stoke => "Feeding the fire",
            Self::Light => "Lighting the fire",
            Self::Collect => "Emptying it",
            Self::HangFood => "Hanging food to dry",
            Self::TakeDown => "Taking it down",
            Self::Talk | Self::Invite | Self::Court => "Talking",
            Self::Gift => "Giving food",
            Self::TryForChild => "Together",
            Self::Deposit => "Storing your load",
            Self::Build => "Building",
            Self::DigClay => "Digging clay",
            Self::RakeOre => "Raking bog ore",
            Self::Hire => "Striking a bargain",
        }
    }

    /// Returns the tool the verb takes, if it takes one.
    pub const fn tool(self) -> Option<Item> {
        match self {
            Self::GatherBoughs | Self::StripBark => Some(Item::Knife),
            Self::Fell => Some(Item::StoneAxe),
            Self::Fish => Some(Item::FishingRod),
            Self::FillWater => Some(Item::BarkPot),
            Self::Light => Some(Item::FireSteel),
            _ => None,
        }
    }

    /// Returns the skill the verb takes and adds to, if it takes any.
    pub const fn skill(self) -> Option<Skill> {
        match self {
            Self::GatherSticks
            | Self::GatherBoughs
            | Self::StripBark
            | Self::DigRoots
            | Self::PickBerries => Some(Skill::Foraging),
            Self::Fell => Some(Skill::Woodcraft),
            Self::Fish => Some(Skill::Fishing),
            Self::Light => Some(Skill::Firemaking),
            Self::Collect | Self::TakeDown => Some(Skill::Trapping),
            Self::Build => Some(Skill::Building),
            Self::DigClay | Self::RakeOre => Some(Skill::Foraging),
            Self::HangFood => Some(Skill::Cooking),
            Self::PickUp
            | Self::Drink
            | Self::FillWater
            | Self::AddStick
            | Self::AddLog
            | Self::Stoke
            | Self::Talk
            | Self::Gift
            | Self::Invite
            | Self::Court
            | Self::TryForChild
            | Self::Deposit
            | Self::Hire => None,
        }
    }

    /// Returns whether the verb is done again and again until there is no more to do or
    /// to do it with.
    pub const fn repeats(self) -> bool {
        matches!(
            self,
            Self::GatherSticks | Self::GatherBoughs | Self::PickBerries | Self::Fish | Self::Build
        )
    }
}

/// Something that takes a while to do.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Job {
    Act {
        verb: Verb,
        subject: Subject,
    },
    Craft(Recipe),
    /// Eating or drinking one of the item.
    Consume(Item),
    EatSnow,
    /// Winning over the people of the settlement.
    Appeal(Appeal),
}

/// What is within reach of a person apart from what they carry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AtHand {
    /// Everything in the piles within reach.
    pub loose: Inventory,
    /// The piles within reach, nearest first.
    pub(crate) piles: Vec<ThingId>,
    /// The cairn of the settlement, if its common store is within reach, in which case
    /// what is in the store counts as lying within reach.
    pub store: Option<ThingId>,
    /// Whether a fire burns within reach.
    pub fire: bool,
    /// When the fire within reach that burns the longest burns out.
    pub fire_until: Option<Timestamp>,
    /// The best shelter within reach.
    pub shelter: Option<StructureKind>,
    pub bed: bool,
    pub water: bool,
    /// What the settlement knows.
    pub advances: Vec<Advance>,
    /// The buildings within reach at which things are made.
    pub workplaces: Vec<StructureKind>,
}

impl AtHand {
    /// Returns how many of `item` there are to use between what is `carried` and what
    /// lies within reach.
    pub fn count(&self, carried: &Inventory, item: Item) -> u32 {
        carried.count(item) + self.loose.count(item)
    }

    /// Returns whether there is `tool` to use, or something that does for it.
    pub fn has_tool(&self, carried: &Inventory, tool: Item) -> bool {
        let stand_in = tool.stand_in().map_or(0, |item| self.count(carried, item));
        self.count(carried, tool) + stand_in > 0
    }

    /// Returns everything that `recipe` takes and that someone carrying `carried` lacks.
    pub fn lacks(&self, carried: &Inventory, recipe: Recipe) -> Vec<Lack> {
        let mut lacks = Vec::new();

        if let Some((advance, workplace)) = recipe.craft() {
            if !self.advances.contains(&advance) {
                lacks.push(Lack::Knowledge(advance));
            }
            if !self.workplaces.contains(&workplace) {
                lacks.push(Lack::Workplace(workplace));
            }
        }
        for tool in recipe.tools() {
            if !self.has_tool(carried, *tool) {
                lacks.push(Lack::Tool(*tool));
            }
        }
        for (item, needed) in recipe.materials() {
            let had = self.count(carried, *item);
            if had < *needed {
                lacks.push(Lack::Material(*item, needed - had));
            }
        }
        if recipe.needs_fire() && !self.fire {
            lacks.push(Lack::Fire);
        }
        if recipe.needs_water() && !self.water {
            lacks.push(Lack::Water);
        }

        lacks
    }
}

/// Something missing for a recipe.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lack {
    Tool(Item),
    /// The item, and how many more of it are needed.
    Material(Item, u32),
    Fire,
    Water,
    /// Something the settlement has to have worked out.
    Knowledge(Advance),
    /// A building that has to be within reach.
    Workplace(StructureKind),
}

impl Lack {
    pub fn describe(self) -> String {
        match self {
            Self::Tool(item) => item.name().to_lowercase(),
            Self::Material(item, count) => format!("{count} × {}", item.name().to_lowercase()),
            Self::Fire => "a burning fire within reach".to_owned(),
            Self::Water => "the lake within reach".to_owned(),
            Self::Knowledge(advance) => {
                format!("your people to know {}", advance.name().to_lowercase())
            }
            Self::Workplace(kind) => {
                format!(
                    "a {} within reach",
                    Kind::Structure(kind).name().to_lowercase()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Stack;

    #[test]
    fn what_is_missing_for_a_recipe_is_told_by_how_much() {
        let mut carried = Inventory::default();
        carried.add(Item::Stick, 3);
        let mut at_hand = AtHand::default();

        assert_eq!(
            at_hand.lacks(&carried, Recipe::Campfire),
            [
                Lack::Tool(Item::FireSteel),
                Lack::Material(Item::Stick, 2),
                Lack::Material(Item::BirchBark, 1),
            ]
        );

        at_hand.loose.add_stack(Stack::new(Item::Stick, 2));
        carried.add(Item::BirchBark, 1);
        carried.add(Item::FireSteel, 1);
        assert_eq!(at_hand.lacks(&carried, Recipe::Campfire), []);
    }

    #[test]
    fn cooking_takes_a_fire_and_a_fish_trap_takes_water() {
        let mut carried = Inventory::default();
        carried.add(Item::RawMeat, 1);
        carried.add(Item::FishTrap, 1);
        let mut at_hand = AtHand::default();

        assert_eq!(at_hand.lacks(&carried, Recipe::CookMeat), [Lack::Fire]);
        assert_eq!(at_hand.lacks(&carried, Recipe::SetFishTrap), [Lack::Water]);

        at_hand.fire = true;
        at_hand.water = true;
        assert_eq!(at_hand.lacks(&carried, Recipe::CookMeat), []);
        assert_eq!(at_hand.lacks(&carried, Recipe::SetFishTrap), []);
    }

    #[test]
    fn lack_is_worded_for_a_list() {
        assert_eq!(Lack::Tool(Item::Knife).describe(), "knife");
        assert_eq!(
            Lack::Material(Item::SpruceBough, 4).describe(),
            "4 × spruce bough"
        );
    }

    #[test]
    fn there_is_something_to_do_to_most_kinds_of_things() {
        let lake = Verb::for_kind(Kind::Lake);
        let spruce = Verb::for_kind(Kind::Plant(PlantSpecies::Spruce));

        assert!(lake.contains(&Verb::Drink) && lake.contains(&Verb::Fish));
        assert!(spruce.contains(&Verb::GatherBoughs));
        assert!(!Verb::for_kind(Kind::Plant(PlantSpecies::Birch)).contains(&Verb::GatherBoughs));
        assert!(Verb::for_kind(Kind::Rock).is_empty());
        assert_eq!(
            Verb::StripBark.name(Kind::Plant(PlantSpecies::Pine)),
            "Strip inner bark"
        );
    }
}
