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
    Sand,
    Plank,
    Brick,
    Glass,
    Charcoal,
    Coal,
    CopperOre,
    TinOre,
    IronOre,
    Bronze,
    Iron,
    Steel,
    Wool,
    Hide,
    Leather,
    Cloth,
    Flour,
    Paper,
    Book,
    Cement,
    MachineParts,
    Wire,
    Oil,
    Medicine,
    Tools,
    Knife,
    FireSteel,
    StoneAxe,
    BronzeAxe,
    IronAxe,
    SteelAxe,
    Pickaxe,
    Saw,
    Quern,
    BoneNeedle,
    FishingRod,
    BarkPot,
    ClayPot,
    Snare,
    FishTrap,
    Bow,
    Arrow,
    BarkPack,
    Backpack,
    Skis,
    FurHat,
    FurCoat,
    FurBoots,
    WoolHat,
    WoolCoat,
    LeatherBoots,
    Chanterelles,
    Nettles,
    Greens,
    RyeBread,
    Bilberries,
    Lingonberries,
    InnerBark,
    Grain,
    Turnip,
    Potato,
    Cheese,
    HareCarcass,
    DeerCarcass,
    RawMeat,
    CookedMeat,
    DriedMeat,
    RawFish,
    CookedFish,
    DriedFish,
    CannedFood,
    Ale,
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

/// Where on the body something is worn.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Garment {
    Head,
    Body,
    Feet,
}

impl Garment {
    pub const ALL: [Self; 3] = [Self::Head, Self::Body, Self::Feet];
}

impl Item {
    pub const ALL: [Self; 85] = [
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
        Self::Sand,
        Self::Plank,
        Self::Brick,
        Self::Glass,
        Self::Charcoal,
        Self::Coal,
        Self::CopperOre,
        Self::TinOre,
        Self::IronOre,
        Self::Bronze,
        Self::Iron,
        Self::Steel,
        Self::Wool,
        Self::Hide,
        Self::Leather,
        Self::Cloth,
        Self::Flour,
        Self::Paper,
        Self::Book,
        Self::Cement,
        Self::MachineParts,
        Self::Wire,
        Self::Oil,
        Self::Medicine,
        Self::Tools,
        Self::Knife,
        Self::FireSteel,
        Self::StoneAxe,
        Self::BronzeAxe,
        Self::IronAxe,
        Self::SteelAxe,
        Self::Pickaxe,
        Self::Saw,
        Self::Quern,
        Self::BoneNeedle,
        Self::FishingRod,
        Self::BarkPot,
        Self::ClayPot,
        Self::Snare,
        Self::FishTrap,
        Self::Bow,
        Self::Arrow,
        Self::BarkPack,
        Self::Backpack,
        Self::Skis,
        Self::FurHat,
        Self::FurCoat,
        Self::FurBoots,
        Self::WoolHat,
        Self::WoolCoat,
        Self::LeatherBoots,
        Self::Chanterelles,
        Self::Nettles,
        Self::Greens,
        Self::RyeBread,
        Self::Bilberries,
        Self::Lingonberries,
        Self::InnerBark,
        Self::Grain,
        Self::Turnip,
        Self::Potato,
        Self::Cheese,
        Self::HareCarcass,
        Self::DeerCarcass,
        Self::RawMeat,
        Self::CookedMeat,
        Self::DriedMeat,
        Self::RawFish,
        Self::CookedFish,
        Self::DriedFish,
        Self::CannedFood,
        Self::Ale,
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
            Self::Sand => "Sand",
            Self::Plank => "Plank",
            Self::Brick => "Brick",
            Self::Glass => "Pane of glass",
            Self::Charcoal => "Charcoal",
            Self::Coal => "Coal",
            Self::CopperOre => "Copper ore",
            Self::TinOre => "Tin ore",
            Self::IronOre => "Iron ore",
            Self::Bronze => "Bronze bar",
            Self::Iron => "Iron bar",
            Self::Steel => "Steel bar",
            Self::Wool => "Wool",
            Self::Hide => "Hide",
            Self::Leather => "Leather",
            Self::Cloth => "Cloth",
            Self::Flour => "Rye flour",
            Self::Paper => "Paper",
            Self::Book => "Book",
            Self::Cement => "Cement",
            Self::MachineParts => "Machine parts",
            Self::Wire => "Copper wire",
            Self::Oil => "Oil",
            Self::Medicine => "Medicine",
            Self::Tools => "Tools",
            Self::Knife => "Knife",
            Self::FireSteel => "Fire steel",
            Self::StoneAxe => "Stone axe",
            Self::BronzeAxe => "Bronze axe",
            Self::IronAxe => "Iron axe",
            Self::SteelAxe => "Steel axe",
            Self::Pickaxe => "Pickaxe",
            Self::Saw => "Saw",
            Self::Quern => "Quern",
            Self::BoneNeedle => "Bone needle",
            Self::FishingRod => "Fishing rod",
            Self::BarkPot => "Birch-bark pot",
            Self::ClayPot => "Clay pot",
            Self::Snare => "Snare",
            Self::FishTrap => "Fish trap",
            Self::Bow => "Bow",
            Self::Arrow => "Arrow",
            Self::BarkPack => "Birch-bark pack",
            Self::Backpack => "Leather backpack",
            Self::Skis => "Skis",
            Self::FurHat => "Fur hat and mittens",
            Self::FurCoat => "Fur coat",
            Self::FurBoots => "Fur boots",
            Self::WoolHat => "Wool hat and mittens",
            Self::WoolCoat => "Wool coat",
            Self::LeatherBoots => "Leather boots",
            Self::Chanterelles => "Chanterelles",
            Self::Nettles => "Nettles",
            Self::Greens => "Nettle greens",
            Self::RyeBread => "Rye bread",
            Self::Bilberries => "Bilberries",
            Self::Lingonberries => "Lingonberries",
            Self::InnerBark => "Pine inner bark",
            Self::Grain => "Rye",
            Self::Turnip => "Turnip",
            Self::Potato => "Potato",
            Self::Cheese => "Cheese",
            Self::HareCarcass => "Hare",
            Self::DeerCarcass => "Roe deer",
            Self::RawMeat => "Raw meat",
            Self::CookedMeat => "Cooked meat",
            Self::DriedMeat => "Dried meat",
            Self::RawFish => "Raw perch",
            Self::CookedFish => "Cooked perch",
            Self::DriedFish => "Dried perch",
            Self::CannedFood => "Canned food",
            Self::Ale => "Ale",
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
            Self::BirchBark => 100,
            Self::SpruceRoot => 100,
            Self::Cordage => 80,
            Self::Stone => 1_000,
            Self::HarePelt => 300,
            Self::Bone => 100,
            Self::Clay => 1_000,
            Self::BogOre => 1_000,
            Self::Sand => 1_000,
            Self::Plank => 3_000,
            Self::Brick => 2_500,
            Self::Glass => 1_000,
            Self::Charcoal => 500,
            Self::Coal => 1_000,
            Self::CopperOre => 1_000,
            Self::TinOre => 1_000,
            Self::IronOre => 1_000,
            Self::Bronze => 1_000,
            Self::Iron => 1_000,
            Self::Steel => 1_000,
            Self::Wool => 300,
            Self::Hide => 1_500,
            Self::Leather => 800,
            Self::Cloth => 400,
            Self::Flour => 100,
            Self::Paper => 50,
            Self::Book => 800,
            Self::Cement => 1_000,
            Self::MachineParts => 5_000,
            Self::Wire => 500,
            Self::Oil => 1_000,
            Self::Medicine => 50,
            Self::Tools => 2_000,
            Self::Knife => 150,
            Self::FireSteel => 50,
            Self::StoneAxe => 1_500,
            Self::BronzeAxe => 1_300,
            Self::IronAxe => 1_200,
            Self::SteelAxe => 1_100,
            Self::Pickaxe => 2_500,
            Self::Saw => 800,
            Self::Quern => 6_000,
            Self::BoneNeedle => 10,
            Self::FishingRod => 500,
            Self::BarkPot => 200,
            Self::ClayPot => 800,
            Self::Snare => 80,
            Self::FishTrap => 3_000,
            Self::Bow => 700,
            Self::Arrow => 40,
            Self::BarkPack => 600,
            Self::Backpack => 1_200,
            Self::Skis => 3_000,
            Self::FurHat => 400,
            Self::FurCoat => 2_000,
            Self::FurBoots => 800,
            Self::WoolHat => 250,
            Self::WoolCoat => 1_200,
            Self::LeatherBoots => 700,
            Self::Chanterelles => 60,
            Self::Nettles => 50,
            Self::Greens => 200,
            Self::RyeBread => 250,
            Self::Bilberries => 60,
            Self::Lingonberries => 60,
            Self::InnerBark => 100,
            Self::Grain => 100,
            Self::Turnip => 300,
            Self::Potato => 300,
            Self::Cheese => 100,
            Self::HareCarcass => 3_000,
            Self::DeerCarcass => 22_000,
            Self::RawMeat => 400,
            Self::CookedMeat => 300,
            Self::DriedMeat => 150,
            Self::RawFish => 250,
            Self::CookedFish => 200,
            Self::DriedFish => 100,
            Self::CannedFood => 400,
            Self::Ale => 500,
            Self::Water => 1_000,
            Self::BoiledWater => 1_000,
        }
    }

    /// Returns what one of the item is worth at a market, in coins, before what there is
    /// of it and what is wanted of it are taken into account.
    pub const fn value(self) -> u32 {
        match self {
            Self::Stick => 1,
            Self::Log => 6,
            Self::SpruceBough => 1,
            Self::BirchBark => 1,
            Self::SpruceRoot => 1,
            Self::Cordage => 3,
            Self::Stone => 2,
            Self::HarePelt => 8,
            Self::Bone => 1,
            Self::Clay => 1,
            Self::BogOre => 4,
            Self::Sand => 1,
            Self::Plank => 4,
            Self::Brick => 3,
            Self::Glass => 12,
            Self::Charcoal => 2,
            Self::Coal => 3,
            Self::CopperOre => 5,
            Self::TinOre => 9,
            Self::IronOre => 4,
            Self::Bronze => 30,
            Self::Iron => 24,
            Self::Steel => 40,
            Self::Wool => 5,
            Self::Hide => 8,
            Self::Leather => 18,
            Self::Cloth => 20,
            Self::Flour => 2,
            Self::Paper => 2,
            Self::Book => 60,
            Self::Cement => 4,
            Self::MachineParts => 150,
            Self::Wire => 25,
            Self::Oil => 10,
            Self::Medicine => 40,
            Self::Tools => 50,
            Self::Knife => 25,
            Self::FireSteel => 20,
            Self::StoneAxe => 6,
            Self::BronzeAxe => 45,
            Self::IronAxe => 40,
            Self::SteelAxe => 60,
            Self::Pickaxe => 45,
            Self::Saw => 45,
            Self::Quern => 8,
            Self::BoneNeedle => 2,
            Self::FishingRod => 5,
            Self::BarkPot => 4,
            Self::ClayPot => 6,
            Self::Snare => 4,
            Self::FishTrap => 10,
            Self::Bow => 15,
            Self::Arrow => 1,
            Self::BarkPack => 8,
            Self::Backpack => 45,
            Self::Skis => 20,
            Self::FurHat => 25,
            Self::FurCoat => 60,
            Self::FurBoots => 25,
            Self::WoolHat => 30,
            Self::WoolCoat => 100,
            Self::LeatherBoots => 50,
            Self::Chanterelles => 2,
            Self::Nettles => 1,
            Self::Greens => 1,
            Self::RyeBread => 4,
            Self::Bilberries => 1,
            Self::Lingonberries => 1,
            Self::InnerBark => 1,
            Self::Grain => 2,
            Self::Turnip => 1,
            Self::Potato => 2,
            Self::Cheese => 4,
            Self::HareCarcass => 10,
            Self::DeerCarcass => 60,
            Self::RawMeat => 4,
            Self::CookedMeat => 5,
            Self::DriedMeat => 6,
            Self::RawFish => 2,
            Self::CookedFish => 3,
            Self::DriedFish => 3,
            Self::CannedFood => 8,
            Self::Ale => 2,
            Self::Water => 1,
            Self::BoiledWater => 1,
        }
    }

    /// Returns what eating or drinking one of the item does, or `None` if it is neither
    /// food nor drink.
    pub const fn food(self) -> Option<Food> {
        let (energy, water, risk, seconds) = match self {
            Self::Chanterelles => (30, 50, 0, 60),
            Self::Greens => (120, 150, 0, 120),
            Self::RyeBread => (600, 0, 0, 180),
            Self::Bilberries => (35, 50, 0, 60),
            Self::Lingonberries => (30, 50, 0, 60),
            Self::InnerBark => (60, 0, 0, 180),
            Self::Grain => (340, 0, 0, 180),
            Self::Turnip => (80, 200, 0, 120),
            Self::Potato => (230, 150, 0, 180),
            Self::Cheese => (350, 0, 0, 120),
            Self::RawMeat => (450, 100, 300, 240),
            Self::CookedMeat => (450, 50, 0, 240),
            Self::DriedMeat => (430, 0, 0, 240),
            Self::RawFish => (220, 100, 150, 180),
            Self::CookedFish => (220, 50, 0, 180),
            Self::DriedFish => (200, 0, 0, 180),
            Self::CannedFood => (500, 100, 0, 180),
            Self::Ale => (180, 450, 0, 60),
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
        matches!(self, Self::Water | Self::BoiledWater | Self::Ale)
    }

    /// Returns how many in-game seconds of a mild day the item keeps for, or `None` if it
    /// never goes off.
    pub const fn shelf_life(self) -> Option<u32> {
        let days = match self {
            Self::Chanterelles => 4,
            Self::Nettles => 3,
            Self::Greens => 3,
            Self::RyeBread => 30,
            Self::Bilberries => 5,
            Self::Lingonberries => 30,
            Self::Turnip => 150,
            Self::Potato => 180,
            Self::Cheese => 120,
            Self::HareCarcass => 2,
            Self::DeerCarcass => 2,
            Self::RawMeat => 2,
            Self::CookedMeat => 4,
            Self::RawFish => 1,
            Self::CookedFish => 3,
            _ => return None,
        };

        Some(days * SECONDS_PER_DAY as u32)
    }

    /// Returns where the item is worn, if it is something to wear.
    pub const fn garment(self) -> Option<Garment> {
        match self {
            Self::FurHat | Self::WoolHat => Some(Garment::Head),
            Self::FurCoat | Self::WoolCoat => Some(Garment::Body),
            Self::FurBoots | Self::LeatherBoots => Some(Garment::Feet),
            _ => None,
        }
    }

    /// Returns how many tenths of a degree warmer wearing the item keeps its wearer.
    pub const fn warmth(self) -> i32 {
        match self {
            Self::FurHat => 50,
            Self::FurCoat => 90,
            Self::FurBoots => 40,
            Self::WoolHat => 45,
            Self::WoolCoat => 80,
            Self::LeatherBoots => 30,
            _ => 0,
        }
    }

    /// Returns how many grams more someone carrying the item on their back can carry,
    /// and carry lightly.
    pub const fn pack(self) -> u32 {
        match self {
            Self::BarkPack => 8_000,
            Self::Backpack => 15_000,
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

    /// Returns the items that do for this one wherever this one is called for as a tool,
    /// the least good first.
    pub const fn stand_ins(self) -> &'static [Self] {
        match self {
            Self::StoneAxe => &[Self::BronzeAxe, Self::IronAxe, Self::SteelAxe],
            Self::BarkPot => &[Self::ClayPot],
            _ => &[],
        }
    }

    /// Returns how many hundredths as long felling a tree takes with the item as it does
    /// with a stone axe, and the chance in a hundred that the item breaks on the tree.
    /// Returns `None` if the item is no axe.
    pub const fn axe(self) -> Option<(u32, u64)> {
        match self {
            Self::StoneAxe => Some((100, 8)),
            Self::BronzeAxe => Some((70, 1)),
            Self::IronAxe => Some((50, 0)),
            Self::SteelAxe => Some((35, 0)),
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

    /// Returns how many grams more than without the best pack there is in here lets
    /// someone carry.
    fn pack(&self) -> u32 {
        let packs = self.stacks.iter().map(|stack| stack.item.pack());
        packs.max().unwrap_or(0)
    }

    /// Returns the most someone carrying this can carry, in grams.
    pub fn limit(&self) -> u32 {
        CARRY_LIMIT + self.pack()
    }

    /// Returns whether this is a load heavy enough to slow whoever carries it down.
    pub fn is_heavy(&self) -> bool {
        self.weight() > HEAVY_LOAD + self.pack()
    }

    /// Returns how many more of `item` fit before the whole weighs all that can be
    /// carried.
    pub fn room_for(&self, item: Item) -> u32 {
        self.limit().saturating_sub(self.weight()) / item.weight()
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
        assert!(Item::ALL.iter().all(|item| item.value() > 0));
    }

    #[test]
    fn better_axes_cut_faster_and_stand_in_for_a_stone_one() {
        let mut pace = 101;
        for axe in [
            Item::StoneAxe,
            Item::BronzeAxe,
            Item::IronAxe,
            Item::SteelAxe,
        ] {
            let (percent, _) = axe.axe().expect("it is an axe");
            assert!(percent < pace, "{axe:?}");
            pace = percent;
            assert!(axe == Item::StoneAxe || Item::StoneAxe.stand_ins().contains(&axe));
        }
        assert_eq!(Item::Knife.axe(), None);
        assert_eq!(Item::WoolCoat.garment(), Item::FurCoat.garment());
        assert!(Item::Ale.is_drink());
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
    fn pack_lets_more_be_carried_and_carried_lightly() {
        let mut inventory = Inventory::default();
        inventory.add(Item::Log, 2);
        assert!(inventory.is_heavy());
        assert_eq!(inventory.room_for(Item::Stone), 10);

        inventory.add(Item::BarkPack, 1);
        assert_eq!(inventory.limit(), CARRY_LIMIT + 8_000);
        assert!(!inventory.is_heavy());
        inventory.add(Item::Backpack, 1);
        assert_eq!(inventory.limit(), CARRY_LIMIT + 15_000);
        assert_eq!(inventory.room_for(Item::Stone), 23);
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
