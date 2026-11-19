//! What a person can make with their own hands, and of what.

use serde::{Deserialize, Serialize};

use crate::advance::Advance;
use crate::item::Item;
use crate::skill::Skill;
use crate::world::StructureKind;

/// Something that can be made, built, set up, or prepared.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Recipe {
    Cordage,
    NettleCordage,
    StoneAxe,
    BoneNeedle,
    FishingRod,
    BarkPot,
    Snare,
    FishTrap,
    Bow,
    Arrows,
    BarkPack,
    Backpack,
    Skis,
    ClayPot,
    IronAxe,
    Quern,
    BronzeAxe,
    Pickaxe,
    Saw,
    SteelAxe,
    BronzeTools,
    IronTools,
    Charcoal,
    SmeltBronze,
    SmeltIron,
    SmeltBogIron,
    MakeSteel,
    Bricks,
    Glass,
    Planks,
    Leather,
    Cloth,
    Paper,
    Book,
    FurHat,
    FurBoots,
    FurCoat,
    WoolHat,
    WoolCoat,
    LeatherBoots,
    Campfire,
    Bed,
    LeanTo,
    DryingRack,
    Cabin,
    SetSnare,
    SetFishTrap,
    Cairn,
    Butcher,
    ButcherDeer,
    CookMeat,
    CookFish,
    CookGreens,
    BoilWater,
    Flour,
    Bread,
    Ale,
    Medicine,
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
    Materials,
    Clothing,
    Camp,
    Food,
}

impl Trade {
    pub const ALL: [Self; 5] = [
        Self::Tools,
        Self::Materials,
        Self::Clothing,
        Self::Camp,
        Self::Food,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Tools => "Tools",
            Self::Materials => "Materials",
            Self::Clothing => "Clothing",
            Self::Camp => "Camp",
            Self::Food => "Food and drink",
        }
    }
}

impl Recipe {
    pub const ALL: [Self; 58] = [
        Self::Cordage,
        Self::NettleCordage,
        Self::StoneAxe,
        Self::BoneNeedle,
        Self::FishingRod,
        Self::BarkPot,
        Self::Snare,
        Self::FishTrap,
        Self::Bow,
        Self::Arrows,
        Self::BarkPack,
        Self::Backpack,
        Self::Skis,
        Self::ClayPot,
        Self::IronAxe,
        Self::Quern,
        Self::BronzeAxe,
        Self::Pickaxe,
        Self::Saw,
        Self::SteelAxe,
        Self::BronzeTools,
        Self::IronTools,
        Self::Charcoal,
        Self::SmeltBronze,
        Self::SmeltIron,
        Self::SmeltBogIron,
        Self::MakeSteel,
        Self::Bricks,
        Self::Glass,
        Self::Planks,
        Self::Leather,
        Self::Cloth,
        Self::Paper,
        Self::Book,
        Self::FurHat,
        Self::FurBoots,
        Self::FurCoat,
        Self::WoolHat,
        Self::WoolCoat,
        Self::LeatherBoots,
        Self::Campfire,
        Self::Bed,
        Self::LeanTo,
        Self::DryingRack,
        Self::Cabin,
        Self::SetSnare,
        Self::SetFishTrap,
        Self::Cairn,
        Self::Butcher,
        Self::ButcherDeer,
        Self::CookMeat,
        Self::CookFish,
        Self::CookGreens,
        Self::BoilWater,
        Self::Flour,
        Self::Bread,
        Self::Ale,
        Self::Medicine,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Cordage => "Twist cordage",
            Self::NettleCordage => "Twist nettle cordage",
            Self::StoneAxe => "Make a stone axe",
            Self::BoneNeedle => "Carve a bone needle",
            Self::FishingRod => "Make a fishing rod",
            Self::BarkPot => "Fold a birch-bark pot",
            Self::Snare => "Tie a snare",
            Self::FishTrap => "Weave a fish trap",
            Self::Bow => "Make a bow",
            Self::Arrows => "Make arrows",
            Self::BarkPack => "Weave a birch-bark pack",
            Self::Backpack => "Sew a leather backpack",
            Self::Skis => "Carve skis",
            Self::ClayPot => "Fire a clay pot",
            Self::IronAxe => "Forge an iron axe",
            Self::Quern => "Peck a quern",
            Self::BronzeAxe => "Cast a bronze axe",
            Self::Pickaxe => "Cast a pickaxe",
            Self::Saw => "Cast a saw",
            Self::SteelAxe => "Forge a steel axe",
            Self::BronzeTools => "Cast bronze tools",
            Self::IronTools => "Forge iron tools",
            Self::Charcoal => "Burn charcoal",
            Self::SmeltBronze => "Smelt bronze",
            Self::SmeltIron => "Smelt iron",
            Self::SmeltBogIron => "Smelt bog iron",
            Self::MakeSteel => "Make steel",
            Self::Bricks => "Fire bricks",
            Self::Glass => "Make glass",
            Self::Planks => "Saw planks",
            Self::Leather => "Tan a hide",
            Self::Cloth => "Weave cloth",
            Self::Paper => "Make paper",
            Self::Book => "Write a book",
            Self::FurHat => "Sew a fur hat and mittens",
            Self::FurBoots => "Sew fur boots",
            Self::FurCoat => "Sew a fur coat",
            Self::WoolHat => "Sew a wool hat and mittens",
            Self::WoolCoat => "Sew a wool coat",
            Self::LeatherBoots => "Sew leather boots",
            Self::Campfire => "Light a campfire",
            Self::Bed => "Lay a bough bed",
            Self::LeanTo => "Build a lean-to",
            Self::DryingRack => "Build a drying rack",
            Self::Cabin => "Build a log cabin",
            Self::SetSnare => "Set a snare",
            Self::SetFishTrap => "Set a fish trap",
            Self::Cairn => "Found a settlement",
            Self::Butcher => "Butcher a hare",
            Self::ButcherDeer => "Butcher a deer",
            Self::CookMeat => "Cook meat",
            Self::CookFish => "Cook perch",
            Self::CookGreens => "Cook nettle greens",
            Self::BoilWater => "Boil water",
            Self::Flour => "Grind flour",
            Self::Bread => "Bake bread",
            Self::Ale => "Brew ale",
            Self::Medicine => "Boil medicine",
        }
    }

    /// Returns what the one who has made it is told.
    pub(crate) fn done(self) -> &'static str {
        match self {
            Self::Cordage => "You twist the roots into a length of cordage.",
            Self::NettleCordage => {
                "You strip the nettle stalks and twist the fibre into a length of cordage."
            }
            Self::StoneAxe => "You knap an edge on the stone and lash it to a handle: an axe.",
            Self::BoneNeedle => "You carve a needle out of the bone.",
            Self::FishingRod => "You carve a hook, tie it to a line, and the line to a rod.",
            Self::BarkPot => "You fold and stitch the bark into a pot that holds water.",
            Self::Snare => "You tie the cordage into a snare.",
            Self::FishTrap => "You weave the sticks into a fish trap.",
            Self::Bow => "You tiller a stave until it bends evenly, and string it.",
            Self::Arrows => "You straighten the shafts and bind on heads of knapped stone.",
            Self::BarkPack => "You weave strips of bark into a pack to carry on your back.",
            Self::Backpack => "You sew a pack of leather that sits well and holds a great deal.",
            Self::Skis => {
                "You split a log, carve two long boards, and bend up the tips over the fire."
            }
            Self::ClayPot => "You shape the clay and fire it into a pot that holds three liters.",
            Self::IronAxe => {
                "You smelt the ore, and hammer the bloom into an axe that will not break."
            }
            Self::Quern => "You peck two stones flat until one turns on the other: a quern.",
            Self::BronzeAxe => "You cast an axe head in clay and haft it.",
            Self::Pickaxe => "You cast a pick and haft it. Now boulders are stone to be had.",
            Self::Saw => "You cast a blade and file teeth into it: a saw.",
            Self::SteelAxe => "You draw out the steel and temper the edge. Nothing cuts like it.",
            Self::BronzeTools => "You cast hoes, adzes, and chisels for the store.",
            Self::IronTools => "You forge spades, hammers, and sickles for the store.",
            Self::Charcoal => "You stack the kiln, light it, and smother it. Charcoal.",
            Self::SmeltBronze => {
                "The ores run together in the crucible, and you pour a bar of bronze."
            }
            Self::SmeltIron => "You work the bloom on the anvil into a bar of iron.",
            Self::SmeltBogIron => "You work the bloom on the anvil into a bar of iron.",
            Self::MakeSteel => "You bake the iron in charcoal until it has turned to steel.",
            Self::Bricks => "You mould the clay and fire it into bricks.",
            Self::Glass => "You melt the sand and spin it out into panes.",
            Self::Planks => "You saw the log into planks.",
            Self::Leather => "You scrape the hide and steep it in bark liquor until it is leather.",
            Self::Cloth => "You spin the wool and weave it into a length of cloth.",
            Self::Paper => "You beat the rags to pulp and dry it in sheets.",
            Self::Book => "You write down what you know and bind it in leather.",
            Self::FurHat => "You sew a fur hat and a pair of mittens.",
            Self::FurBoots => "You sew a pair of fur boots.",
            Self::FurCoat => "You sew a fur coat.",
            Self::WoolHat => "You sew a wool hat and a pair of mittens.",
            Self::WoolCoat => "You cut and sew a wool coat, lighter than fur and nearly as warm.",
            Self::LeatherBoots => "You sew a pair of leather boots.",
            Self::Campfire => "The tinder catches, and soon the fire burns.",
            Self::Bed => "You lay the boughs into a springy bed.",
            Self::LeanTo => "You finish the lean-to. It keeps the rain and the wind off.",
            Self::DryingRack => "You lash together a rack to dry food on.",
            Self::Cabin => "You lay the last of the roof. The cabin is finished.",
            Self::SetSnare => "You set the snare on a hare trail.",
            Self::SetFishTrap => "You sink the fish trap by the shore.",
            Self::Cairn => "You raise a cairn of stones. This is where you will stay.",
            Self::Butcher => "You skin and butcher the hare.",
            Self::ButcherDeer => {
                "You skin and butcher the deer. There is meat for weeks, if it keeps."
            }
            Self::CookMeat => "You roast a piece of meat over the fire.",
            Self::CookFish => "You roast a perch over the fire.",
            Self::CookGreens => "You boil the nettles until they no longer sting.",
            Self::BoilWater => "You boil a potful of water with stones from the fire.",
            Self::Flour => "You turn the quern until the rye is flour.",
            Self::Bread => "You knead the dough and bake it on hot stones by the fire.",
            Self::Ale => "You mash the rye, boil it, and leave it to work into ale.",
            Self::Medicine => "You boil the berries and the bark down into a bitter draught.",
        }
    }

    pub const fn trade(self) -> Trade {
        match self {
            Self::Cordage => Trade::Tools,
            Self::NettleCordage => Trade::Tools,
            Self::StoneAxe => Trade::Tools,
            Self::BoneNeedle => Trade::Tools,
            Self::FishingRod => Trade::Tools,
            Self::BarkPot => Trade::Tools,
            Self::Snare => Trade::Tools,
            Self::FishTrap => Trade::Tools,
            Self::Bow => Trade::Tools,
            Self::Arrows => Trade::Tools,
            Self::BarkPack => Trade::Tools,
            Self::Backpack => Trade::Tools,
            Self::Skis => Trade::Tools,
            Self::ClayPot => Trade::Tools,
            Self::IronAxe => Trade::Tools,
            Self::Quern => Trade::Tools,
            Self::BronzeAxe => Trade::Tools,
            Self::Pickaxe => Trade::Tools,
            Self::Saw => Trade::Tools,
            Self::SteelAxe => Trade::Tools,
            Self::BronzeTools => Trade::Tools,
            Self::IronTools => Trade::Tools,
            Self::Charcoal => Trade::Materials,
            Self::SmeltBronze => Trade::Materials,
            Self::SmeltIron => Trade::Materials,
            Self::SmeltBogIron => Trade::Materials,
            Self::MakeSteel => Trade::Materials,
            Self::Bricks => Trade::Materials,
            Self::Glass => Trade::Materials,
            Self::Planks => Trade::Materials,
            Self::Leather => Trade::Materials,
            Self::Cloth => Trade::Materials,
            Self::Paper => Trade::Materials,
            Self::Book => Trade::Materials,
            Self::FurHat => Trade::Clothing,
            Self::FurBoots => Trade::Clothing,
            Self::FurCoat => Trade::Clothing,
            Self::WoolHat => Trade::Clothing,
            Self::WoolCoat => Trade::Clothing,
            Self::LeatherBoots => Trade::Clothing,
            Self::Campfire => Trade::Camp,
            Self::Bed => Trade::Camp,
            Self::LeanTo => Trade::Camp,
            Self::DryingRack => Trade::Camp,
            Self::Cabin => Trade::Camp,
            Self::SetSnare => Trade::Camp,
            Self::SetFishTrap => Trade::Camp,
            Self::Cairn => Trade::Camp,
            Self::Butcher => Trade::Food,
            Self::ButcherDeer => Trade::Food,
            Self::CookMeat => Trade::Food,
            Self::CookFish => Trade::Food,
            Self::CookGreens => Trade::Food,
            Self::BoilWater => Trade::Food,
            Self::Flour => Trade::Food,
            Self::Bread => Trade::Food,
            Self::Ale => Trade::Food,
            Self::Medicine => Trade::Food,
        }
    }

    /// Returns what is used up, and how many of each.
    pub const fn materials(self) -> &'static [(Item, u32)] {
        match self {
            Self::Cordage => &[(Item::SpruceRoot, 2)],
            Self::NettleCordage => &[(Item::Nettles, 6)],
            Self::StoneAxe => &[(Item::Stone, 2), (Item::Stick, 1), (Item::Cordage, 1)],
            Self::BoneNeedle => &[(Item::Bone, 1)],
            Self::FishingRod => &[(Item::Stick, 2), (Item::Cordage, 1)],
            Self::BarkPot => &[(Item::BirchBark, 2), (Item::Cordage, 1)],
            Self::Snare => &[(Item::Cordage, 1)],
            Self::FishTrap => &[(Item::Stick, 12), (Item::Cordage, 2)],
            Self::Bow => &[(Item::Stick, 2), (Item::Cordage, 2)],
            Self::Arrows => &[(Item::Stick, 2), (Item::Stone, 1), (Item::Cordage, 1)],
            Self::BarkPack => &[(Item::BirchBark, 6), (Item::Cordage, 3)],
            Self::Backpack => &[(Item::Leather, 2), (Item::Cordage, 2)],
            Self::Skis => &[(Item::Log, 1), (Item::Cordage, 2)],
            Self::ClayPot => &[(Item::Clay, 2), (Item::Stick, 4)],
            Self::IronAxe => &[(Item::BogOre, 4), (Item::Log, 2), (Item::Stick, 1)],
            Self::Quern => &[(Item::Stone, 3)],
            Self::BronzeAxe => &[(Item::Bronze, 1), (Item::Stick, 1)],
            Self::Pickaxe => &[(Item::Bronze, 1), (Item::Stick, 1)],
            Self::Saw => &[(Item::Bronze, 1), (Item::Stick, 1)],
            Self::SteelAxe => &[(Item::Steel, 1), (Item::Stick, 1)],
            Self::BronzeTools => &[(Item::Bronze, 1)],
            Self::IronTools => &[(Item::Iron, 1)],
            Self::Charcoal => &[(Item::Log, 2)],
            Self::SmeltBronze => &[(Item::CopperOre, 2), (Item::TinOre, 1), (Item::Charcoal, 2)],
            Self::SmeltIron => &[(Item::IronOre, 3), (Item::Charcoal, 2)],
            Self::SmeltBogIron => &[(Item::BogOre, 3), (Item::Charcoal, 2)],
            Self::MakeSteel => &[(Item::Iron, 2), (Item::Charcoal, 3)],
            Self::Bricks => &[(Item::Clay, 4), (Item::Stick, 4)],
            Self::Glass => &[(Item::Sand, 3), (Item::Charcoal, 2)],
            Self::Planks => &[(Item::Log, 1)],
            Self::Leather => &[(Item::Hide, 1), (Item::InnerBark, 2)],
            Self::Cloth => &[(Item::Wool, 3)],
            Self::Paper => &[(Item::Cloth, 1)],
            Self::Book => &[(Item::Paper, 6), (Item::Leather, 1)],
            Self::FurHat => &[(Item::HarePelt, 2), (Item::Cordage, 1)],
            Self::FurBoots => &[(Item::HarePelt, 2), (Item::Cordage, 1)],
            Self::FurCoat => &[(Item::HarePelt, 5), (Item::Cordage, 2)],
            Self::WoolHat => &[(Item::Cloth, 1), (Item::Wool, 1)],
            Self::WoolCoat => &[(Item::Cloth, 4)],
            Self::LeatherBoots => &[(Item::Leather, 2), (Item::Cordage, 1)],
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
            Self::ButcherDeer => &[(Item::DeerCarcass, 1)],
            Self::CookMeat => &[(Item::RawMeat, 1)],
            Self::CookFish => &[(Item::RawFish, 1)],
            Self::CookGreens => &[(Item::Nettles, 4), (Item::BoiledWater, 1)],
            Self::BoilWater => &[(Item::Water, 1)],
            Self::Flour => &[(Item::Grain, 4)],
            Self::Bread => &[(Item::Flour, 3), (Item::BoiledWater, 1)],
            Self::Ale => &[(Item::Grain, 4), (Item::BoiledWater, 2)],
            Self::Medicine => &[
                (Item::Lingonberries, 6),
                (Item::InnerBark, 2),
                (Item::BoiledWater, 1),
            ],
        }
    }

    /// Returns what has to be at hand but is not used up.
    pub const fn tools(self) -> &'static [Item] {
        match self {
            Self::Cordage => &[],
            Self::NettleCordage => &[],
            Self::StoneAxe => &[Item::Knife],
            Self::BoneNeedle => &[Item::Knife],
            Self::FishingRod => &[Item::Knife],
            Self::BarkPot => &[Item::Knife],
            Self::Snare => &[],
            Self::FishTrap => &[Item::Knife],
            Self::Bow => &[Item::Knife],
            Self::Arrows => &[Item::Knife],
            Self::BarkPack => &[Item::Knife],
            Self::Backpack => &[Item::Knife, Item::BoneNeedle],
            Self::Skis => &[Item::StoneAxe, Item::Knife],
            Self::ClayPot => &[],
            Self::IronAxe => &[],
            Self::Quern => &[],
            Self::BronzeAxe => &[],
            Self::Pickaxe => &[],
            Self::Saw => &[],
            Self::SteelAxe => &[],
            Self::BronzeTools => &[],
            Self::IronTools => &[],
            Self::Charcoal => &[],
            Self::SmeltBronze => &[],
            Self::SmeltIron => &[],
            Self::SmeltBogIron => &[],
            Self::MakeSteel => &[],
            Self::Bricks => &[],
            Self::Glass => &[],
            Self::Planks => &[Item::Saw],
            Self::Leather => &[Item::Knife],
            Self::Cloth => &[],
            Self::Paper => &[],
            Self::Book => &[],
            Self::FurHat => &[Item::Knife, Item::BoneNeedle],
            Self::FurBoots => &[Item::Knife, Item::BoneNeedle],
            Self::FurCoat => &[Item::Knife, Item::BoneNeedle],
            Self::WoolHat => &[Item::Knife, Item::BoneNeedle],
            Self::WoolCoat => &[Item::Knife, Item::BoneNeedle],
            Self::LeatherBoots => &[Item::Knife, Item::BoneNeedle],
            Self::Campfire => &[Item::FireSteel],
            Self::Bed => &[],
            Self::LeanTo => &[],
            Self::DryingRack => &[Item::Knife],
            Self::Cabin => &[Item::StoneAxe, Item::Knife],
            Self::SetSnare => &[],
            Self::SetFishTrap => &[],
            Self::Cairn => &[],
            Self::Butcher => &[Item::Knife],
            Self::ButcherDeer => &[Item::Knife],
            Self::CookMeat => &[],
            Self::CookFish => &[],
            Self::CookGreens => &[Item::BarkPot],
            Self::BoilWater => &[Item::BarkPot],
            Self::Flour => &[Item::Quern],
            Self::Bread => &[],
            Self::Ale => &[Item::ClayPot],
            Self::Medicine => &[Item::BarkPot],
        }
    }

    /// Returns what the settlement has to know for it to be made, if it takes knowing
    /// more than everyone does.
    pub const fn knowledge(self) -> Option<Advance> {
        match self {
            Self::ClayPot => Some(Advance::Pottery),
            Self::IronAxe => Some(Advance::Ironworking),
            Self::Quern => Some(Advance::Farming),
            Self::BronzeAxe => Some(Advance::Bronze),
            Self::Pickaxe => Some(Advance::Bronze),
            Self::Saw => Some(Advance::Bronze),
            Self::SteelAxe => Some(Advance::Steel),
            Self::BronzeTools => Some(Advance::Bronze),
            Self::IronTools => Some(Advance::Ironworking),
            Self::Charcoal => Some(Advance::Charcoal),
            Self::SmeltBronze => Some(Advance::Bronze),
            Self::SmeltIron => Some(Advance::Ironworking),
            Self::SmeltBogIron => Some(Advance::Ironworking),
            Self::MakeSteel => Some(Advance::Steel),
            Self::Bricks => Some(Advance::Brickmaking),
            Self::Glass => Some(Advance::Glassmaking),
            Self::Leather => Some(Advance::Husbandry),
            Self::Cloth => Some(Advance::Weaving),
            Self::Paper => Some(Advance::Papermaking),
            Self::Book => Some(Advance::Writing),
            Self::Ale => Some(Advance::Brewing),
            Self::Medicine => Some(Advance::Herbalism),
            _ => None,
        }
    }

    /// Returns the building it has to be made at, if it takes one.
    pub const fn workplace(self) -> Option<StructureKind> {
        match self {
            Self::ClayPot => Some(StructureKind::Kiln),
            Self::IronAxe => Some(StructureKind::Smithy),
            Self::BronzeAxe => Some(StructureKind::Smithy),
            Self::Pickaxe => Some(StructureKind::Smithy),
            Self::Saw => Some(StructureKind::Smithy),
            Self::SteelAxe => Some(StructureKind::Smithy),
            Self::BronzeTools => Some(StructureKind::Smithy),
            Self::IronTools => Some(StructureKind::Smithy),
            Self::Charcoal => Some(StructureKind::CharcoalKiln),
            Self::SmeltBronze => Some(StructureKind::Smithy),
            Self::SmeltIron => Some(StructureKind::Smithy),
            Self::SmeltBogIron => Some(StructureKind::Smithy),
            Self::MakeSteel => Some(StructureKind::Smithy),
            Self::Bricks => Some(StructureKind::Kiln),
            Self::Glass => Some(StructureKind::Glassworks),
            Self::Cloth => Some(StructureKind::Weavery),
            _ => None,
        }
    }

    /// Returns whether it takes a burning fire within reach.
    pub const fn needs_fire(self) -> bool {
        matches!(
            self,
            Self::Skis
                | Self::CookMeat
                | Self::CookFish
                | Self::CookGreens
                | Self::BoilWater
                | Self::Bread
                | Self::Ale
                | Self::Medicine
        )
    }

    /// Returns whether it takes water within reach.
    pub const fn needs_water(self) -> bool {
        matches!(self, Self::Leather | Self::Paper | Self::SetFishTrap)
    }

    pub const fn product(self) -> Product {
        match self {
            Self::Cordage => Product::Items(&[(Item::Cordage, 1)]),
            Self::NettleCordage => Product::Items(&[(Item::Cordage, 1)]),
            Self::StoneAxe => Product::Items(&[(Item::StoneAxe, 1)]),
            Self::BoneNeedle => Product::Items(&[(Item::BoneNeedle, 1)]),
            Self::FishingRod => Product::Items(&[(Item::FishingRod, 1)]),
            Self::BarkPot => Product::Items(&[(Item::BarkPot, 1)]),
            Self::Snare => Product::Items(&[(Item::Snare, 1)]),
            Self::FishTrap => Product::Items(&[(Item::FishTrap, 1)]),
            Self::Bow => Product::Items(&[(Item::Bow, 1)]),
            Self::Arrows => Product::Items(&[(Item::Arrow, 4)]),
            Self::BarkPack => Product::Items(&[(Item::BarkPack, 1)]),
            Self::Backpack => Product::Items(&[(Item::Backpack, 1)]),
            Self::Skis => Product::Items(&[(Item::Skis, 1)]),
            Self::ClayPot => Product::Items(&[(Item::ClayPot, 1)]),
            Self::IronAxe => Product::Items(&[(Item::IronAxe, 1)]),
            Self::Quern => Product::Items(&[(Item::Quern, 1)]),
            Self::BronzeAxe => Product::Items(&[(Item::BronzeAxe, 1)]),
            Self::Pickaxe => Product::Items(&[(Item::Pickaxe, 1)]),
            Self::Saw => Product::Items(&[(Item::Saw, 1)]),
            Self::SteelAxe => Product::Items(&[(Item::SteelAxe, 1)]),
            Self::BronzeTools => Product::Items(&[(Item::Tools, 2)]),
            Self::IronTools => Product::Items(&[(Item::Tools, 2)]),
            Self::Charcoal => Product::Items(&[(Item::Charcoal, 6)]),
            Self::SmeltBronze => Product::Items(&[(Item::Bronze, 1)]),
            Self::SmeltIron => Product::Items(&[(Item::Iron, 1)]),
            Self::SmeltBogIron => Product::Items(&[(Item::Iron, 1)]),
            Self::MakeSteel => Product::Items(&[(Item::Steel, 1)]),
            Self::Bricks => Product::Items(&[(Item::Brick, 4)]),
            Self::Glass => Product::Items(&[(Item::Glass, 2)]),
            Self::Planks => Product::Items(&[(Item::Plank, 3)]),
            Self::Leather => Product::Items(&[(Item::Leather, 1)]),
            Self::Cloth => Product::Items(&[(Item::Cloth, 1)]),
            Self::Paper => Product::Items(&[(Item::Paper, 8)]),
            Self::Book => Product::Items(&[(Item::Book, 1)]),
            Self::FurHat => Product::Items(&[(Item::FurHat, 1)]),
            Self::FurBoots => Product::Items(&[(Item::FurBoots, 1)]),
            Self::FurCoat => Product::Items(&[(Item::FurCoat, 1)]),
            Self::WoolHat => Product::Items(&[(Item::WoolHat, 1)]),
            Self::WoolCoat => Product::Items(&[(Item::WoolCoat, 1)]),
            Self::LeatherBoots => Product::Items(&[(Item::LeatherBoots, 1)]),
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
            Self::ButcherDeer => {
                Product::Items(&[(Item::RawMeat, 30), (Item::Hide, 1), (Item::Bone, 6)])
            }
            Self::CookMeat => Product::Items(&[(Item::CookedMeat, 1)]),
            Self::CookFish => Product::Items(&[(Item::CookedFish, 1)]),
            Self::CookGreens => Product::Items(&[(Item::Greens, 2)]),
            Self::BoilWater => Product::Items(&[(Item::BoiledWater, 1)]),
            Self::Flour => Product::Items(&[(Item::Flour, 4)]),
            Self::Bread => Product::Items(&[(Item::RyeBread, 2)]),
            Self::Ale => Product::Items(&[(Item::Ale, 4)]),
            Self::Medicine => Product::Items(&[(Item::Medicine, 1)]),
        }
    }

    /// Returns how many in-game seconds it takes someone unskilled and well.
    pub const fn seconds(self) -> u32 {
        const MINUTE: u32 = 60;
        const HOUR: u32 = 60 * MINUTE;

        match self {
            Self::Cordage => 5 * MINUTE,
            Self::NettleCordage => 10 * MINUTE,
            Self::StoneAxe => 30 * MINUTE,
            Self::BoneNeedle => 10 * MINUTE,
            Self::FishingRod => 15 * MINUTE,
            Self::BarkPot => 15 * MINUTE,
            Self::Snare => 2 * MINUTE,
            Self::FishTrap => 45 * MINUTE,
            Self::Bow => HOUR,
            Self::Arrows => 20 * MINUTE,
            Self::BarkPack => 90 * MINUTE,
            Self::Backpack => 3 * HOUR,
            Self::Skis => 4 * HOUR,
            Self::ClayPot => 40 * MINUTE,
            Self::IronAxe => 3 * HOUR,
            Self::Quern => 2 * HOUR,
            Self::BronzeAxe => 90 * MINUTE,
            Self::Pickaxe => 90 * MINUTE,
            Self::Saw => 2 * HOUR,
            Self::SteelAxe => 2 * HOUR,
            Self::BronzeTools => 2 * HOUR,
            Self::IronTools => 2 * HOUR,
            Self::Charcoal => 4 * HOUR,
            Self::SmeltBronze => 2 * HOUR,
            Self::SmeltIron => 3 * HOUR,
            Self::SmeltBogIron => 3 * HOUR,
            Self::MakeSteel => 4 * HOUR,
            Self::Bricks => 2 * HOUR,
            Self::Glass => 2 * HOUR,
            Self::Planks => 45 * MINUTE,
            Self::Leather => 2 * HOUR,
            Self::Cloth => 3 * HOUR,
            Self::Paper => 2 * HOUR,
            Self::Book => 8 * HOUR,
            Self::FurHat => 90 * MINUTE,
            Self::FurBoots => 2 * HOUR,
            Self::FurCoat => 4 * HOUR,
            Self::WoolHat => HOUR,
            Self::WoolCoat => 4 * HOUR,
            Self::LeatherBoots => 2 * HOUR,
            Self::Campfire => 3 * MINUTE,
            Self::Bed => 15 * MINUTE,
            Self::LeanTo => 90 * MINUTE,
            Self::DryingRack => 30 * MINUTE,
            Self::Cabin => 14 * HOUR,
            Self::SetSnare => 3 * MINUTE,
            Self::SetFishTrap => 5 * MINUTE,
            Self::Cairn => 90 * MINUTE,
            Self::Butcher => 15 * MINUTE,
            Self::ButcherDeer => 45 * MINUTE,
            Self::CookMeat => 10 * MINUTE,
            Self::CookFish => 7 * MINUTE,
            Self::CookGreens => 10 * MINUTE,
            Self::BoilWater => 5 * MINUTE,
            Self::Flour => 20 * MINUTE,
            Self::Bread => 30 * MINUTE,
            Self::Ale => HOUR,
            Self::Medicine => 45 * MINUTE,
        }
    }

    /// Returns the skill it takes, and adds to.
    pub const fn skill(self) -> Skill {
        match self {
            Self::Cordage => Skill::Crafting,
            Self::NettleCordage => Skill::Crafting,
            Self::StoneAxe => Skill::Crafting,
            Self::BoneNeedle => Skill::Crafting,
            Self::FishingRod => Skill::Crafting,
            Self::BarkPot => Skill::Crafting,
            Self::Snare => Skill::Trapping,
            Self::FishTrap => Skill::Fishing,
            Self::Bow => Skill::Crafting,
            Self::Arrows => Skill::Crafting,
            Self::BarkPack => Skill::Crafting,
            Self::Backpack => Skill::Crafting,
            Self::Skis => Skill::Woodcraft,
            Self::ClayPot => Skill::Crafting,
            Self::IronAxe => Skill::Crafting,
            Self::Quern => Skill::Crafting,
            Self::BronzeAxe => Skill::Crafting,
            Self::Pickaxe => Skill::Crafting,
            Self::Saw => Skill::Crafting,
            Self::SteelAxe => Skill::Crafting,
            Self::BronzeTools => Skill::Crafting,
            Self::IronTools => Skill::Crafting,
            Self::Charcoal => Skill::Firemaking,
            Self::SmeltBronze => Skill::Crafting,
            Self::SmeltIron => Skill::Crafting,
            Self::SmeltBogIron => Skill::Crafting,
            Self::MakeSteel => Skill::Crafting,
            Self::Bricks => Skill::Crafting,
            Self::Glass => Skill::Crafting,
            Self::Planks => Skill::Woodcraft,
            Self::Leather => Skill::Crafting,
            Self::Cloth => Skill::Crafting,
            Self::Paper => Skill::Crafting,
            Self::Book => Skill::Crafting,
            Self::FurHat => Skill::Crafting,
            Self::FurBoots => Skill::Crafting,
            Self::FurCoat => Skill::Crafting,
            Self::WoolHat => Skill::Crafting,
            Self::WoolCoat => Skill::Crafting,
            Self::LeatherBoots => Skill::Crafting,
            Self::Campfire => Skill::Firemaking,
            Self::Bed => Skill::Building,
            Self::LeanTo => Skill::Building,
            Self::DryingRack => Skill::Building,
            Self::Cabin => Skill::Building,
            Self::SetSnare => Skill::Trapping,
            Self::SetFishTrap => Skill::Fishing,
            Self::Cairn => Skill::Building,
            Self::Butcher => Skill::Trapping,
            Self::ButcherDeer => Skill::Trapping,
            Self::CookMeat => Skill::Cooking,
            Self::CookFish => Skill::Cooking,
            Self::CookGreens => Skill::Cooking,
            Self::BoilWater => Skill::Cooking,
            Self::Flour => Skill::Cooking,
            Self::Bread => Skill::Cooking,
            Self::Ale => Skill::Cooking,
            Self::Medicine => Skill::Cooking,
        }
    }

    /// Returns whether it is done again and again for as long as there is what it takes.
    pub const fn repeats(self) -> bool {
        matches!(
            self,
            Self::Cordage
                | Self::NettleCordage
                | Self::Planks
                | Self::CookMeat
                | Self::CookFish
                | Self::CookGreens
                | Self::BoilWater
                | Self::Flour
                | Self::Bread
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
        assert_eq!(Recipe::ALL.last(), Some(&Recipe::Medicine));
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

    #[test]
    fn what_is_made_at_a_building_takes_knowing_how() {
        for recipe in Recipe::ALL {
            if let Some(workplace) = recipe.workplace() {
                assert!(recipe.knowledge().is_some(), "{recipe:?}");
                assert!(crate::building::building(workplace).is_some(), "{recipe:?}");
            }
        }
        assert_eq!(Recipe::ClayPot.knowledge(), Some(Advance::Pottery));
        assert_eq!(Recipe::ClayPot.workplace(), Some(StructureKind::Kiln));
        assert_eq!(Recipe::Cordage.knowledge(), None);
    }

    /// Everything that is made of something can be made starting from what is gathered,
    /// caught, grown, dug, or had from the start.
    #[test]
    fn nothing_takes_what_cannot_be_come_by() {
        let mut had = vec![
            Item::Stick,
            Item::Log,
            Item::SpruceBough,
            Item::BirchBark,
            Item::InnerBark,
            Item::SpruceRoot,
            Item::Stone,
            Item::Clay,
            Item::Sand,
            Item::BogOre,
            Item::CopperOre,
            Item::TinOre,
            Item::IronOre,
            Item::Wool,
            Item::Hide,
            Item::Grain,
            Item::Lingonberries,
            Item::Nettles,
            Item::DeerCarcass,
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
