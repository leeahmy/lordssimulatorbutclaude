//! Things small enough to be carried, and the carrying of them.

use serde::{Deserialize, Serialize};

use crate::time::gregorian::SECONDS_PER_DAY;

/// The most a person can carry, in grams.
pub const CARRY_LIMIT: u32 = 40_000;

/// The weight from which carrying slows a person down, in grams.
pub const HEAVY_LOAD: u32 = 25_000;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Item {
    Stick,
    Log,
    SpruceBough,
    BirchBark,
    SpruceRoot,
    Cordage,
    Stone,
    HarePelt,
    Bone,
    Clay,
    BogOre,

    Knife,
    FireSteel,
    StoneAxe,
    IronAxe,
    BoneNeedle,
    FishingRod,
    BarkPot,
    ClayPot,
    Snare,
    FishTrap,

    FurHat,
    FurCoat,
    FurBoots,

    RyeBread,
    Bilberries,
    Lingonberries,
    InnerBark,
    Grain,
    HareCarcass,
    RawMeat,
    CookedMeat,
    DriedMeat,
    RawFish,
    CookedFish,
    DriedFish,
    Water,
    BoiledWater,
}

/// What eating or drinking one of an item does.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Food {
    /// The energy it gives, in kilocalories.
    pub energy: u32,
    /// The water it gives, in milliliters.
    pub water: u32,
    /// The chance in a thousand that it makes the eater sick.
    pub risk: u32,
    /// How many in-game seconds it takes to get down.
    pub seconds: u32,
}

impl Item {
    pub const ALL: [Self; 38] = [
        Self::Stick,
        Self::Log,
        Self::SpruceBough,
        Self::BirchBark,
        Self::SpruceRoot,
        Self::Cordage,
        Self::Stone,
        Self::HarePelt,
        Self::Bone,
        Self::Clay,
        Self::BogOre,
        Self::Knife,
        Self::FireSteel,
        Self::StoneAxe,
        Self::IronAxe,
        Self::BoneNeedle,
        Self::FishingRod,
        Self::BarkPot,
        Self::ClayPot,
        Self::Snare,
        Self::FishTrap,
        Self::FurHat,
        Self::FurCoat,
        Self::FurBoots,
        Self::RyeBread,
        Self::Bilberries,
        Self::Lingonberries,
        Self::InnerBark,
        Self::Grain,
        Self::HareCarcass,
        Self::RawMeat,
        Self::CookedMeat,
        Self::DriedMeat,
        Self::RawFish,
        Self::CookedFish,
        Self::DriedFish,
        Self::Water,
        Self::BoiledWater,
    ];

    /// Returns what one of the item is called.
    pub fn name(self) -> &'static str {
        match self {
            Self::Stick => "Stick",
            Self::Log => "Log",
            Self::SpruceBough => "Spruce bough",
            Self::BirchBark => "Birch bark",
            Self::SpruceRoot => "Spruce root",
            Self::Cordage => "Cordage",
            Self::Stone => "Stone",
            Self::HarePelt => "Hare pelt",
            Self::Bone => "Bone",
            Self::Clay => "Clay",
            Self::BogOre => "Bog ore",
            Self::IronAxe => "Iron axe",
            Self::ClayPot => "Clay pot",
            Self::Knife => "Knife",
            Self::FireSteel => "Fire steel",
            Self::StoneAxe => "Stone axe",
            Self::BoneNeedle => "Bone needle",
            Self::FishingRod => "Fishing rod",
            Self::BarkPot => "Birch-bark pot",
            Self::Snare => "Snare",
            Self::FishTrap => "Fish trap",
            Self::FurHat => "Fur hat and mittens",
            Self::FurCoat => "Fur coat",
            Self::FurBoots => "Fur boots",
            Self::RyeBread => "Rye bread",
            Self::Bilberries => "Bilberries",
            Self::Lingonberries => "Lingonberries",
            Self::InnerBark => "Pine inner bark",
            Self::Grain => "Rye",
            Self::HareCarcass => "Hare",
            Self::RawMeat => "Raw meat",
            Self::CookedMeat => "Cooked meat",
            Self::DriedMeat => "Dried meat",
            Self::RawFish => "Raw perch",
            Self::CookedFish => "Cooked perch",
            Self::DriedFish => "Dried perch",
            Self::Water => "Lake water",
            Self::BoiledWater => "Boiled water",
        }
    }

    /// Returns the weight of one of the item in grams.
    pub const fn weight(self) -> u32 {
        match self {
            Self::Stick => 400,
            Self::Log => 15_000,
            Self::SpruceBough => 500,
            Self::BirchBark | Self::SpruceRoot | Self::Bone | Self::InnerBark | Self::Grain => 100,
            Self::Cordage | Self::Snare => 80,
            Self::Stone | Self::Clay | Self::BogOre => 1_000,
            Self::IronAxe => 1_200,
            Self::ClayPot => 800,
            Self::HarePelt => 300,
            Self::Knife => 150,
            Self::FireSteel => 50,
            Self::StoneAxe => 1_500,
            Self::BoneNeedle => 10,
            Self::FishingRod => 500,
            Self::BarkPot => 200,
            Self::FishTrap => 3_000,
            Self::FurHat => 400,
            Self::FurCoat => 2_000,
            Self::FurBoots => 800,
            Self::RyeBread | Self::RawFish => 250,
            Self::Bilberries | Self::Lingonberries => 60,
            Self::HareCarcass => 3_000,
            Self::RawMeat => 400,
            Self::CookedMeat => 300,
            Self::DriedMeat => 150,
            Self::CookedFish => 200,
            Self::DriedFish => 100,
            Self::Water | Self::BoiledWater => 1_000,
        }
    }

    /// Returns what eating or drinking one of the item does, or `None` if it is neither
    /// food nor drink.
    pub const fn food(self) -> Option<Food> {
        let (energy, water, risk, seconds) = match self {
            Self::RyeBread => (600, 0, 0, 180),
            Self::Bilberries => (35, 50, 0, 60),
            Self::Lingonberries => (30, 50, 0, 60),
            Self::InnerBark => (60, 0, 0, 180),
            Self::Grain => (340, 0, 0, 180),
            Self::RawMeat => (450, 100, 300, 240),
            Self::CookedMeat => (450, 50, 0, 240),
            Self::DriedMeat => (430, 0, 0, 240),
            Self::RawFish => (220, 100, 150, 180),
            Self::CookedFish => (220, 50, 0, 180),
            Self::DriedFish => (200, 0, 0, 180),
            Self::Water => (0, 1_000, 40, 45),
            Self::BoiledWater => (0, 1_000, 0, 45),
            _ => return None,
        };

        Some(Food {
            energy,
            water,
            risk,
            seconds,
        })
    }

    pub const fn is_drink(self) -> bool {
        matches!(self, Self::Water | Self::BoiledWater)
    }

    /// Returns how many in-game seconds of a mild day the item keeps for, or `None` if it
    /// never goes off.
    pub const fn shelf_life(self) -> Option<u32> {
        let days = match self {
            Self::RawFish => 1,
            Self::RawMeat | Self::HareCarcass => 2,
            Self::CookedFish => 3,
            Self::CookedMeat => 4,
            Self::Bilberries => 5,
            Self::Lingonberries | Self::RyeBread => 30,
            _ => return None,
        };

        Some(days * SECONDS_PER_DAY as u32)
    }

    /// Returns how many tenths of a degree warmer wearing the item keeps its wearer.
    pub const fn warmth(self) -> i32 {
        match self {
            Self::FurHat => 50,
            Self::FurCoat => 90,
            Self::FurBoots => 40,
            _ => 0,
        }
    }

    /// Returns how many liters of water the item holds, which is none unless it is a pot.
    pub const fn holds(self) -> u32 {
        match self {
            Self::BarkPot => 1,
            Self::ClayPot => 3,
            _ => 0,
        }
    }

    /// Returns the item that does for this one wherever this one is called for as a tool,
    /// if there is such an item.
    pub const fn stand_in(self) -> Option<Self> {
        match self {
            Self::StoneAxe => Some(Self::IronAxe),
            Self::BarkPot => Some(Self::ClayPot),
            _ => None,
        }
    }

    /// Returns how many in-game seconds the item keeps a fire burning for, or zero if it
    /// is no fuel.
    pub const fn burn_time(self) -> u32 {
        match self {
            Self::Stick => 15 * 60,
            Self::Log => 2 * 60 * 60,
            _ => 0,
        }
    }
}

/// Some number of one item.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stack {
    pub item: Item,
    pub count: u32,
    /// How many in-game seconds of a mild day are left before the items go off, which is
    /// zero for items that never do.
    #[serde(default)]
    pub fresh: u32,
}

impl Stack {
    /// Returns `count` of `item` as fresh as they come.
    pub const fn new(item: Item, count: u32) -> Self {
        let fresh = match item.shelf_life() {
            Some(seconds) => seconds,
            None => 0,
        };

        Self { item, count, fresh }
    }

    pub const fn weight(&self) -> u32 {
        self.item.weight() * self.count
    }
}

/// What someone carries, or what lies in one place, with one stack for each item in the
/// order of the items.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Inventory {
    stacks: Vec<Stack>,
}

impl Inventory {
    pub fn stacks(&self) -> &[Stack] {
        &self.stacks
    }

    pub fn is_empty(&self) -> bool {
        self.stacks.is_empty()
    }

    pub fn count(&self, item: Item) -> u32 {
        self.stack(item).map_or(0, |stack| stack.count)
    }

    pub fn has(&self, item: Item) -> bool {
        self.count(item) > 0
    }

    pub fn stack(&self, item: Item) -> Option<&Stack> {
        self.stacks.iter().find(|stack| stack.item == item)
    }

    /// Returns the weight of everything in grams.
    pub fn weight(&self) -> u32 {
        self.stacks.iter().map(Stack::weight).sum()
    }

    /// Returns how many more of `item` fit before the whole weighs [`CARRY_LIMIT`].
    pub fn room_for(&self, item: Item) -> u32 {
        CARRY_LIMIT.saturating_sub(self.weight()) / item.weight()
    }

    /// Adds `count` fresh ones of `item`.
    pub fn add(&mut self, item: Item, count: u32) {
        self.add_stack(Stack::new(item, count));
    }

    /// Adds `added`, the whole stack of the item then being as fresh as its least fresh.
    pub fn add_stack(&mut self, added: Stack) {
        if added.count == 0 {
            return;
        }

        match self
            .stacks
            .binary_search_by_key(&added.item, |stack| stack.item)
        {
            Ok(index) => {
                let stack = &mut self.stacks[index];
                stack.count += added.count;
                stack.fresh = stack.fresh.min(added.fresh);
            }
            Err(index) => self.stacks.insert(index, added),
        }
    }

    /// Takes up to `count` of `item` out and returns them.
    pub fn take(&mut self, item: Item, count: u32) -> Stack {
        let Ok(index) = self.stacks.binary_search_by_key(&item, |stack| stack.item) else {
            return Stack {
                item,
                count: 0,
                fresh: 0,
            };
        };

        let stack = &mut self.stacks[index];
        let taken = Stack {
            count: count.min(stack.count),
            ..*stack
        };
        stack.count -= taken.count;
        if stack.count == 0 {
            self.stacks.remove(index);
        }
        taken
    }

    /// Lets `seconds` in-game seconds of a mild day pass, and returns the items that went
    /// off in them, which are no longer here.
    pub fn spoil(&mut self, seconds: u32) -> Vec<Stack> {
        let mut spoiled = Vec::new();
        if seconds == 0 {
            return spoiled;
        }

        self.stacks.retain_mut(|stack| {
            if stack.item.shelf_life().is_none() {
                return true;
            }

            stack.fresh = stack.fresh.saturating_sub(seconds);
            if stack.fresh == 0 {
                spoiled.push(*stack);
            }
            stack.fresh > 0
        });
        spoiled
    }
}

impl FromIterator<Stack> for Inventory {
    fn from_iter<I: IntoIterator<Item = Stack>>(stacks: I) -> Self {
        let mut inventory = Self::default();
        for stack in stacks {
            inventory.add_stack(stack);
        }
        inventory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_is_listed_once_in_order_and_has_a_name() {
        assert!(Item::ALL.is_sorted());
        assert!(Item::ALL.windows(2).all(|pair| pair[0] != pair[1]));
        assert_eq!(Item::ALL.last(), Some(&Item::BoiledWater));
        assert!(Item::ALL.iter().all(|item| !item.name().is_empty()));
        assert!(Item::ALL.iter().all(|item| item.weight() > 0));
    }

    #[test]
    fn stacks_of_the_same_item_are_kept_as_one() {
        let mut inventory = Inventory::default();

        inventory.add(Item::Stone, 2);
        inventory.add(Item::Stick, 3);
        inventory.add(Item::Stone, 1);
        inventory.add(Item::Log, 0);

        assert_eq!(
            inventory.stacks(),
            [Stack::new(Item::Stick, 3), Stack::new(Item::Stone, 3)]
        );
        assert_eq!(inventory.count(Item::Stone), 3);
        assert_eq!(inventory.count(Item::Log), 0);
        assert_eq!(inventory.weight(), 3 * 400 + 3 * 1_000);
    }

    #[test]
    fn no_more_is_taken_than_there_is() {
        let mut inventory = Inventory::default();
        inventory.add(Item::Stick, 3);

        assert_eq!(inventory.take(Item::Stick, 2).count, 2);
        assert_eq!(inventory.take(Item::Stick, 2).count, 1);
        assert_eq!(inventory.take(Item::Stick, 2).count, 0);
        assert_eq!(inventory.take(Item::Stone, 1).count, 0);
        assert!(inventory.is_empty());
    }

    #[test]
    fn room_is_what_is_left_of_the_carry_limit() {
        let mut inventory = Inventory::default();
        assert_eq!(inventory.room_for(Item::Log), 2);

        inventory.add(Item::Log, 2);
        assert_eq!(inventory.room_for(Item::Log), 0);
        assert_eq!(inventory.room_for(Item::Stone), 10);
    }

    #[test]
    fn food_goes_off_and_everything_else_keeps() {
        let mut inventory = Inventory::default();
        inventory.add(Item::RawFish, 2);
        inventory.add(Item::RawMeat, 1);
        inventory.add(Item::DriedMeat, 1);
        inventory.add(Item::Stick, 1);
        let day = SECONDS_PER_DAY as u32;

        assert!(inventory.spoil(day - 1).is_empty());
        assert_eq!(inventory.count(Item::RawFish), 2);

        let spoiled = inventory.spoil(1);
        assert_eq!(spoiled.len(), 1);
        assert_eq!((spoiled[0].item, spoiled[0].count), (Item::RawFish, 2));
        assert!(!inventory.has(Item::RawFish));

        inventory.spoil(100 * day);
        assert!(!inventory.has(Item::RawMeat));
        assert!(inventory.has(Item::DriedMeat));
        assert!(inventory.has(Item::Stick));
    }

    #[test]
    fn stack_is_as_fresh_as_the_least_fresh_put_in_it() {
        let mut inventory = Inventory::default();
        inventory.add(Item::RawMeat, 1);
        inventory.spoil(1_000);
        let older = inventory.stacks()[0].fresh;

        inventory.add(Item::RawMeat, 1);

        assert_eq!(inventory.stacks()[0].fresh, older);
        assert_eq!(inventory.count(Item::RawMeat), 2);
    }

    #[test]
    fn inventory_reads_back_as_it_was_written() {
        let mut inventory = Inventory::default();
        inventory.add(Item::RawMeat, 2);
        inventory.add(Item::Knife, 1);

        let written = ron::to_string(&inventory).expect("the inventory encodes");
        let read: Inventory = ron::from_str(&written).expect("the inventory decodes");

        assert_eq!(read, inventory);
    }
}
