//! What a settlement builds: houses, workplaces, and everything a town is run from.

use crate::advance::Advance;
use crate::item::Item;
use crate::people::Role;
use crate::world::StructureKind;

/// What it takes to put up a building of a settlement, and what it is good for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub kind: StructureKind,
    pub materials: &'static [(Item, u32)],
    /// How many hours of work it takes.
    pub hours: u32,
    pub needs: Option<Advance>,
    /// How many people it houses.
    pub houses: u32,
    /// The work it gives, and to how many.
    pub jobs: Option<(Role, u32)>,
    /// What it costs the treasury, once there is coin.
    pub coins: u32,
    /// Whether it is put at the edge of the settlement instead of in the middle of it.
    pub outskirts: bool,
    pub use_of: &'static str,
}

/// The buildings that can be planned, roughly in the order they come within reach.
pub const BUILDINGS: [Building; 64] = [
    Building {
        kind: StructureKind::Hut,
        materials: &[(Item::Log, 12), (Item::SpruceBough, 20)],
        hours: 48,
        needs: None,
        houses: 4,
        jobs: None,
        coins: 0,
        outskirts: false,
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
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Houses twelve",
    },
    Building {
        kind: StructureKind::Storehouse,
        materials: &[(Item::Log, 16), (Item::BirchBark, 10)],
        hours: 64,
        needs: None,
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Food in the store keeps twice as long",
    },
    Building {
        kind: StructureKind::Workshop,
        materials: &[(Item::Log, 16), (Item::Stone, 6)],
        hours: 64,
        needs: None,
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Crafters make twice as much, and thinkers get half again as far",
    },
    Building {
        kind: StructureKind::Smokehouse,
        materials: &[(Item::Log, 8), (Item::Stone, 10)],
        hours: 40,
        needs: Some(Advance::Smoking),
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Fish and meat brought in are smoked, and keep for good",
    },
    Building {
        kind: StructureKind::Field,
        materials: &[(Item::Stick, 20)],
        hours: 80,
        needs: Some(Advance::Farming),
        houses: 0,
        jobs: Some((Role::Farmer, 3)),
        coins: 0,
        outskirts: true,
        use_of: "Three farmers can work it",
    },
    Building {
        kind: StructureKind::Hall,
        materials: &[(Item::Log, 30), (Item::SpruceBough, 30), (Item::Stone, 10)],
        hours: 120,
        needs: None,
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "A place to meet and to be heard. A town cannot be without one",
    },
    Building {
        kind: StructureKind::Kiln,
        materials: &[(Item::Stone, 20), (Item::Clay, 10)],
        hours: 40,
        needs: Some(Advance::Pottery),
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Clay pots and bricks can be fired by it",
    },
    Building {
        kind: StructureKind::Dock,
        materials: &[(Item::Log, 12), (Item::Cordage, 4)],
        hours: 48,
        needs: Some(Advance::Boats),
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Fishers go out in boats, and catch twice as much",
    },
    Building {
        kind: StructureKind::Smithy,
        materials: &[(Item::Stone, 20), (Item::Log, 10)],
        hours: 80,
        needs: Some(Advance::Bronze),
        houses: 0,
        jobs: Some((Role::Smith, 2)),
        coins: 0,
        outskirts: false,
        use_of: "Two smiths smelt ore and forge tools. Metal can be worked in it by hand",
    },
    Building {
        kind: StructureKind::Well,
        materials: &[(Item::Stone, 30), (Item::Log, 4)],
        hours: 80,
        needs: Some(Advance::Masonry),
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Clean water that never freezes over",
    },
    Building {
        kind: StructureKind::Sauna,
        materials: &[(Item::Log, 10), (Item::Stone, 12)],
        hours: 40,
        needs: None,
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "People are cleaner and more content, and fewer sicken",
    },
    Building {
        kind: StructureKind::Shrine,
        materials: &[(Item::Stone, 20), (Item::Log, 6)],
        hours: 60,
        needs: None,
        houses: 0,
        jobs: Some((Role::Priest, 1)),
        coins: 0,
        outskirts: false,
        use_of: "A priest keeps the rites, and people are more content",
    },
    Building {
        kind: StructureKind::Temple,
        materials: &[(Item::Stone, 150), (Item::Log, 40), (Item::Glass, 10)],
        hours: 500,
        needs: Some(Advance::Masonry),
        houses: 0,
        jobs: Some((Role::Priest, 3)),
        coins: 1_000,
        outskirts: false,
        use_of: "Three priests keep the rites of a town, and people are more content still",
    },
    Building {
        kind: StructureKind::Theatre,
        materials: &[(Item::Plank, 60), (Item::Stone, 30), (Item::Cloth, 10)],
        hours: 400,
        needs: Some(Advance::Papermaking),
        houses: 0,
        jobs: Some((Role::Performer, 4)),
        coins: 1_500,
        outskirts: false,
        use_of: "Four performers put on plays, and people are more content",
    },
    Building {
        kind: StructureKind::Park,
        materials: &[(Item::Stone, 30), (Item::Log, 10)],
        hours: 150,
        needs: Some(Advance::Sanitation),
        houses: 0,
        jobs: None,
        coins: 500,
        outskirts: false,
        use_of: "Somewhere green to walk. People are more content",
    },
    Building {
        kind: StructureKind::Museum,
        materials: &[(Item::Brick, 150), (Item::Glass, 30), (Item::Book, 20)],
        hours: 600,
        needs: Some(Advance::Science),
        houses: 0,
        jobs: None,
        coins: 3_000,
        outskirts: false,
        use_of: "What is known is kept and shown: thinkers and scholars get a twentieth further",
    },
    Building {
        kind: StructureKind::Cinema,
        materials: &[(Item::Brick, 100), (Item::Wire, 20), (Item::Glass, 10)],
        hours: 400,
        needs: Some(Advance::Electricity),
        houses: 0,
        jobs: Some((Role::Performer, 3)),
        coins: 4_000,
        outskirts: false,
        use_of: "Moving pictures. People are more content",
    },
    Building {
        kind: StructureKind::Stadium,
        materials: &[(Item::Cement, 300), (Item::Steel, 60)],
        hours: 1_500,
        needs: Some(Advance::Concrete),
        houses: 0,
        jobs: Some((Role::Performer, 6)),
        coins: 8_000,
        outskirts: false,
        use_of: "Games that the whole town turns out for. People are more content",
    },
    Building {
        kind: StructureKind::Palisade,
        materials: &[(Item::Log, 60), (Item::Cordage, 10)],
        hours: 150,
        needs: None,
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: true,
        use_of: "Worth six guards against raiders",
    },
    Building {
        kind: StructureKind::Barracks,
        materials: &[(Item::Log, 30), (Item::Stone, 20), (Item::Bronze, 6)],
        hours: 200,
        needs: Some(Advance::Bronze),
        houses: 0,
        jobs: Some((Role::Guard, 6)),
        coins: 400,
        outskirts: false,
        use_of: "Six guards keep watch, and stand against raiders",
    },
    Building {
        kind: StructureKind::Wall,
        materials: &[(Item::Stone, 300), (Item::Log, 40)],
        hours: 800,
        needs: Some(Advance::Masonry),
        houses: 0,
        jobs: None,
        coins: 1_500,
        outskirts: true,
        use_of: "Worth twenty guards against raiders",
    },
    Building {
        kind: StructureKind::Pasture,
        materials: &[(Item::Stick, 30), (Item::Log, 6)],
        hours: 60,
        needs: Some(Advance::Husbandry),
        houses: 0,
        jobs: Some((Role::Herder, 3)),
        coins: 0,
        outskirts: true,
        use_of: "Three herders keep animals on it for meat, cheese, wool, and hides",
    },
    Building {
        kind: StructureKind::Brewery,
        materials: &[(Item::Log, 14), (Item::Stone, 8), (Item::Clay, 10)],
        hours: 70,
        needs: Some(Advance::Brewing),
        houses: 0,
        jobs: Some((Role::Brewer, 2)),
        coins: 0,
        outskirts: false,
        use_of: "Two brewers make ale of rye",
    },
    Building {
        kind: StructureKind::Tavern,
        materials: &[(Item::Log, 20), (Item::Stone, 6)],
        hours: 80,
        needs: Some(Advance::Brewing),
        houses: 0,
        jobs: Some((Role::Innkeeper, 1)),
        coins: 0,
        outskirts: false,
        use_of: "Where there is a tavern with ale, people are more content",
    },
    Building {
        kind: StructureKind::Quarry,
        materials: &[(Item::Log, 10), (Item::Cordage, 6)],
        hours: 100,
        needs: Some(Advance::Masonry),
        houses: 0,
        jobs: Some((Role::Quarrier, 4)),
        coins: 0,
        outskirts: true,
        use_of: "Four quarriers break stone",
    },
    Building {
        kind: StructureKind::Mine,
        materials: &[(Item::Log, 30), (Item::Stone, 10)],
        hours: 200,
        needs: Some(Advance::Mining),
        houses: 0,
        jobs: Some((Role::Miner, 6)),
        coins: 0,
        outskirts: true,
        use_of: "Six miners bring up ore, and coal once it is known what it is good for",
    },
    Building {
        kind: StructureKind::CharcoalKiln,
        materials: &[(Item::Stone, 15), (Item::Clay, 10)],
        hours: 50,
        needs: Some(Advance::Charcoal),
        houses: 0,
        jobs: Some((Role::Burner, 2)),
        coins: 0,
        outskirts: false,
        use_of: "Two burners char logs. Charcoal can be burned at it by hand",
    },
    Building {
        kind: StructureKind::Weavery,
        materials: &[(Item::Log, 14), (Item::Cordage, 6)],
        hours: 70,
        needs: Some(Advance::Weaving),
        houses: 0,
        jobs: Some((Role::Weaver, 3)),
        coins: 0,
        outskirts: false,
        use_of: "Three weavers make cloth of wool. Cloth can be woven in it by hand",
    },
    Building {
        kind: StructureKind::Bathhouse,
        materials: &[(Item::Stone, 40), (Item::Log, 10)],
        hours: 150,
        needs: Some(Advance::Masonry),
        houses: 0,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Fewer sicken where people can wash",
    },
    Building {
        kind: StructureKind::Market,
        materials: &[(Item::Log, 24), (Item::Stone, 10)],
        hours: 120,
        needs: Some(Advance::Coinage),
        houses: 0,
        jobs: Some((Role::Merchant, 3)),
        coins: 300,
        outskirts: false,
        use_of: "Three merchants trade with those who pass through. Goods are bought and sold here",
    },
    Building {
        kind: StructureKind::Courthouse,
        materials: &[(Item::Stone, 50), (Item::Log, 30)],
        hours: 250,
        needs: Some(Advance::Law),
        houses: 0,
        jobs: Some((Role::Clerk, 2)),
        coins: 750,
        outskirts: false,
        use_of: "Two clerks keep the peace and the tax rolls",
    },
    Building {
        kind: StructureKind::Infirmary,
        materials: &[(Item::Log, 24), (Item::Stone, 10), (Item::Cloth, 6)],
        hours: 150,
        needs: Some(Advance::Medicine),
        houses: 0,
        jobs: Some((Role::Healer, 3)),
        coins: 500,
        outskirts: false,
        use_of: "Three healers tend the sick",
    },
    Building {
        kind: StructureKind::Harbour,
        materials: &[(Item::Log, 60), (Item::Cordage, 20), (Item::Cloth, 10)],
        hours: 300,
        needs: Some(Advance::Sailing),
        houses: 0,
        jobs: Some((Role::Sailor, 6)),
        coins: 1_000,
        outskirts: false,
        use_of: "Six sailors man trading ships",
    },
    Building {
        kind: StructureKind::School,
        materials: &[(Item::Log, 24), (Item::Stone, 8)],
        hours: 140,
        needs: Some(Advance::Teaching),
        houses: 0,
        jobs: Some((Role::Teacher, 2)),
        coins: 400,
        outskirts: false,
        use_of: "Two teachers teach children to read",
    },
    Building {
        kind: StructureKind::Mill,
        materials: &[(Item::Stone, 30), (Item::Log, 20)],
        hours: 180,
        needs: Some(Advance::Mills),
        houses: 0,
        jobs: Some((Role::Miller, 2)),
        coins: 500,
        outskirts: false,
        use_of: "With millers at work, a tenth more comes of every harvest",
    },
    Building {
        kind: StructureKind::Sawmill,
        materials: &[(Item::Log, 30), (Item::Stone, 10), (Item::Tools, 2)],
        hours: 180,
        needs: Some(Advance::Mills),
        houses: 0,
        jobs: Some((Role::Sawyer, 3)),
        coins: 500,
        outskirts: false,
        use_of: "Three sawyers cut planks. Planks can be sawn here by hand",
    },
    Building {
        kind: StructureKind::Cottage,
        materials: &[(Item::Plank, 24), (Item::Stone, 12)],
        hours: 120,
        needs: Some(Advance::Mills),
        houses: 8,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Houses eight",
    },
    Building {
        kind: StructureKind::Brickworks,
        materials: &[(Item::Stone, 30), (Item::Clay, 30), (Item::Log, 10)],
        hours: 160,
        needs: Some(Advance::Brickmaking),
        houses: 0,
        jobs: Some((Role::Brickmaker, 4)),
        coins: 500,
        outskirts: false,
        use_of: "Four brickmakers fire bricks",
    },
    Building {
        kind: StructureKind::Glassworks,
        materials: &[(Item::Brick, 40), (Item::Stone, 10)],
        hours: 180,
        needs: Some(Advance::Glassmaking),
        houses: 0,
        jobs: Some((Role::Glassmaker, 2)),
        coins: 600,
        outskirts: false,
        use_of: "Two glassmakers make panes. Glass can be blown here by hand",
    },
    Building {
        kind: StructureKind::Guildhall,
        materials: &[(Item::Plank, 40), (Item::Stone, 30)],
        hours: 300,
        needs: Some(Advance::Guilds),
        houses: 0,
        jobs: None,
        coins: 1_500,
        outskirts: false,
        use_of: "Every craft yields a quarter more",
    },
    Building {
        kind: StructureKind::Library,
        materials: &[(Item::Plank, 30), (Item::Stone, 20), (Item::Paper, 40)],
        hours: 240,
        needs: Some(Advance::Papermaking),
        houses: 0,
        jobs: Some((Role::Scholar, 2)),
        coins: 1_000,
        outskirts: false,
        use_of: "Two scholars study here, and the books kept in the store help everyone who thinks",
    },
    Building {
        kind: StructureKind::Townhouse,
        materials: &[(Item::Brick, 60), (Item::Plank, 30), (Item::Glass, 6)],
        hours: 260,
        needs: Some(Advance::Brickmaking),
        houses: 16,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Houses sixteen",
    },
    Building {
        kind: StructureKind::TownHall,
        materials: &[(Item::Brick, 120), (Item::Plank, 60), (Item::Glass, 20)],
        hours: 700,
        needs: Some(Advance::Law),
        houses: 0,
        jobs: Some((Role::Clerk, 4)),
        coins: 3_000,
        outskirts: false,
        use_of: "Four clerks run the town from it. A city cannot be without one",
    },
    Building {
        kind: StructureKind::Bank,
        materials: &[(Item::Brick, 80), (Item::Plank, 20), (Item::Iron, 10)],
        hours: 300,
        needs: Some(Advance::Banking),
        houses: 0,
        jobs: Some((Role::Clerk, 2)),
        coins: 2_000,
        outskirts: false,
        use_of: "What lies in the treasury earns interest",
    },
    Building {
        kind: StructureKind::PrintingHouse,
        materials: &[(Item::Brick, 40), (Item::Plank, 30), (Item::Bronze, 10)],
        hours: 260,
        needs: Some(Advance::Printing),
        houses: 0,
        jobs: Some((Role::Printer, 3)),
        coins: 1_500,
        outskirts: false,
        use_of: "Three printers make books of paper, and print the newspaper",
    },
    Building {
        kind: StructureKind::University,
        materials: &[
            (Item::Brick, 200),
            (Item::Plank, 60),
            (Item::Glass, 30),
            (Item::Book, 20),
        ],
        hours: 900,
        needs: Some(Advance::Science),
        houses: 0,
        jobs: Some((Role::Scholar, 10)),
        coins: 6_000,
        outskirts: false,
        use_of: "Ten scholars work here",
    },
    Building {
        kind: StructureKind::Waterworks,
        materials: &[(Item::Brick, 150), (Item::Iron, 20), (Item::Stone, 60)],
        hours: 600,
        needs: Some(Advance::Sanitation),
        houses: 0,
        jobs: None,
        coins: 4_000,
        outskirts: false,
        use_of: "Clean water in, and foul water out: crowded towns sicken half as much",
    },
    Building {
        kind: StructureKind::Steelworks,
        materials: &[(Item::Brick, 150), (Item::Iron, 30), (Item::Stone, 40)],
        hours: 600,
        needs: Some(Advance::Steel),
        houses: 0,
        jobs: Some((Role::Steelworker, 6)),
        coins: 3_000,
        outskirts: false,
        use_of: "Six steelworkers make steel of iron and coal",
    },
    Building {
        kind: StructureKind::Factory,
        materials: &[
            (Item::Brick, 200),
            (Item::Steel, 40),
            (Item::Plank, 40),
            (Item::Glass, 20),
        ],
        hours: 900,
        needs: Some(Advance::SteamPower),
        houses: 0,
        jobs: Some((Role::FactoryHand, 15)),
        coins: 7_500,
        outskirts: false,
        use_of: "Fifteen hands make tools, machine parts, and wire",
    },
    Building {
        kind: StructureKind::TextileMill,
        materials: &[
            (Item::Brick, 120),
            (Item::Plank, 40),
            (Item::Iron, 20),
            (Item::MachineParts, 4),
        ],
        hours: 700,
        needs: Some(Advance::Mechanization),
        houses: 0,
        jobs: Some((Role::MillHand, 12)),
        coins: 5_000,
        outskirts: false,
        use_of: "Twelve hands weave four times what weavers do",
    },
    Building {
        kind: StructureKind::Tenement,
        materials: &[
            (Item::Brick, 150),
            (Item::Plank, 60),
            (Item::Glass, 20),
            (Item::Iron, 10),
        ],
        hours: 600,
        needs: Some(Advance::SteamPower),
        houses: 40,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Houses forty",
    },
    Building {
        kind: StructureKind::RailwayStation,
        materials: &[(Item::Brick, 150), (Item::Steel, 80), (Item::Plank, 60)],
        hours: 1_200,
        needs: Some(Advance::Railways),
        houses: 0,
        jobs: Some((Role::Carrier, 6)),
        coins: 15_000,
        outskirts: false,
        use_of: "Twice the trade, and twice as many come to settle",
    },
    Building {
        kind: StructureKind::Cannery,
        materials: &[
            (Item::Brick, 100),
            (Item::Steel, 20),
            (Item::MachineParts, 2),
        ],
        hours: 500,
        needs: Some(Advance::Canning),
        houses: 0,
        jobs: Some((Role::Canner, 6)),
        coins: 4_000,
        outskirts: false,
        use_of: "Six canners put up fish and meat in cans that never go off",
    },
    Building {
        kind: StructureKind::PowerStation,
        materials: &[
            (Item::Brick, 250),
            (Item::Steel, 60),
            (Item::MachineParts, 10),
            (Item::Wire, 20),
        ],
        hours: 1_200,
        needs: Some(Advance::Electricity),
        houses: 0,
        jobs: Some((Role::Engineer, 4)),
        coins: 20_000,
        outskirts: false,
        use_of: "With engineers and coal, factories and mills yield half again as much, and houses are heated",
    },
    Building {
        kind: StructureKind::TelegraphOffice,
        materials: &[(Item::Brick, 60), (Item::Wire, 30), (Item::Plank, 20)],
        hours: 300,
        needs: Some(Advance::Telegraph),
        houses: 0,
        jobs: Some((Role::Operator, 2)),
        coins: 4_000,
        outskirts: false,
        use_of: "With news of prices, trade pays a fifth more",
    },
    Building {
        kind: StructureKind::Hospital,
        materials: &[
            (Item::Brick, 200),
            (Item::Glass, 40),
            (Item::Steel, 20),
            (Item::Cloth, 30),
        ],
        hours: 1_000,
        needs: Some(Advance::Antibiotics),
        houses: 0,
        jobs: Some((Role::Healer, 12)),
        coins: 15_000,
        outskirts: false,
        use_of: "Twelve healers work here",
    },
    Building {
        kind: StructureKind::ColdStore,
        materials: &[
            (Item::Brick, 120),
            (Item::Steel, 30),
            (Item::MachineParts, 6),
            (Item::Wire, 10),
        ],
        hours: 600,
        needs: Some(Advance::Refrigeration),
        houses: 0,
        jobs: None,
        coins: 7_500,
        outskirts: false,
        use_of: "Food in the store does not go off",
    },
    Building {
        kind: StructureKind::MachineDepot,
        materials: &[
            (Item::Brick, 100),
            (Item::Steel, 40),
            (Item::MachineParts, 10),
        ],
        hours: 600,
        needs: Some(Advance::Combustion),
        houses: 0,
        jobs: Some((Role::Mechanic, 4)),
        coins: 10_000,
        outskirts: false,
        use_of: "With mechanics and oil, tractors work the fields",
    },
    Building {
        kind: StructureKind::BroadcastingHouse,
        materials: &[
            (Item::Brick, 100),
            (Item::Wire, 60),
            (Item::Glass, 20),
            (Item::MachineParts, 4),
        ],
        hours: 700,
        needs: Some(Advance::Radio),
        houses: 0,
        jobs: Some((Role::Operator, 4)),
        coins: 12_500,
        outskirts: false,
        use_of: "From here you can speak to everyone at once",
    },
    Building {
        kind: StructureKind::ApartmentBlock,
        materials: &[
            (Item::Cement, 200),
            (Item::Steel, 60),
            (Item::Glass, 60),
            (Item::Wire, 20),
        ],
        hours: 1_500,
        needs: Some(Advance::Concrete),
        houses: 100,
        jobs: None,
        coins: 0,
        outskirts: false,
        use_of: "Houses a hundred",
    },
    Building {
        kind: StructureKind::Airport,
        materials: &[
            (Item::Cement, 400),
            (Item::Steel, 100),
            (Item::Glass, 40),
            (Item::Wire, 40),
        ],
        hours: 2_500,
        needs: Some(Advance::Aviation),
        houses: 0,
        jobs: Some((Role::Carrier, 10)),
        coins: 50_000,
        outskirts: true,
        use_of: "Trade and people from everywhere",
    },
    Building {
        kind: StructureKind::Institute,
        materials: &[
            (Item::Cement, 200),
            (Item::Steel, 50),
            (Item::Glass, 60),
            (Item::Wire, 30),
            (Item::Book, 40),
        ],
        hours: 1_500,
        needs: Some(Advance::Electronics),
        houses: 0,
        jobs: Some((Role::Scholar, 20)),
        coins: 30_000,
        outskirts: false,
        use_of: "Twenty scholars work here",
    },
    Building {
        kind: StructureKind::ComputingCentre,
        materials: &[
            (Item::Cement, 150),
            (Item::Steel, 40),
            (Item::Wire, 100),
            (Item::MachineParts, 20),
        ],
        hours: 1_200,
        needs: Some(Advance::Computers),
        houses: 0,
        jobs: Some((Role::Scholar, 10)),
        coins: 40_000,
        outskirts: false,
        use_of: "Ten scholars work here, and every scholar gets twice as far",
    },
];

impl Building {
    /// Returns what keeping the building up costs the treasury in a hundred days, once
    /// there is coin: a fifth of what putting it up did. Houses cost the treasury nothing,
    /// to put up or to keep up: they are the business of those who live in them.
    pub const fn upkeep(&self) -> u32 {
        self.coins / 5
    }
}

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

/// Returns how many people of `role` the buildings of `kinds` give work to, or `None` if
/// the role takes no building to work at.
pub fn places(role: Role, kinds: &[StructureKind]) -> Option<u32> {
    let mut any = false;
    let mut places = 0;
    for plan in &BUILDINGS {
        if let Some((worked, each)) = plan.jobs
            && worked == role
        {
            any = true;
            places += each * kinds.iter().filter(|kind| **kind == plan.kind).count() as u32;
        }
    }
    any.then_some(places)
}

/// How many farmers one field takes.
pub const FARMERS_PER_FIELD: u32 = 3;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::craft::{Product, Recipe};

    #[test]
    fn buildings_take_materials_and_work_and_some_house_people() {
        for (index, plan) in BUILDINGS.iter().enumerate() {
            assert!(!plan.materials.is_empty() && plan.hours > 0, "{plan:?}");
            assert_eq!(building(plan.kind), Some(plan));
            assert_eq!(houses(plan.kind), plan.houses);
            assert!(!plan.use_of.is_empty() && !plan.use_of.ends_with('.'));
            assert!(
                BUILDINGS[..index]
                    .iter()
                    .all(|other| other.kind != plan.kind)
            );
        }
        assert_eq!(houses(StructureKind::Cabin), 4);
        assert_eq!(houses(StructureKind::Campfire), 0);
        assert_eq!(building(StructureKind::Cabin), None);
    }

    #[test]
    fn work_that_takes_a_building_is_counted_by_the_buildings_there_are() {
        let kinds = [
            StructureKind::Field,
            StructureKind::Field,
            StructureKind::Hut,
            StructureKind::University,
            StructureKind::Library,
        ];

        assert_eq!(places(Role::Farmer, &kinds), Some(2 * FARMERS_PER_FIELD));
        assert_eq!(places(Role::Scholar, &kinds), Some(12));
        assert_eq!(places(Role::Miner, &kinds), Some(0));
        assert_eq!(places(Role::Fisher, &kinds), None);
        assert_eq!(places(Role::Thinker, &kinds), None);
    }

    /// Nothing is built of what neither a trade of the settlement nor anyone by hand
    /// can make.
    #[test]
    fn nothing_is_built_of_what_cannot_be_come_by() {
        let made_by_hand = |item: Item| {
            Recipe::ALL
                .into_iter()
                .any(|recipe| match recipe.product() {
                    Product::Items(items) => items.iter().any(|(made, _)| *made == item),
                    Product::Structure(_) => false,
                })
        };
        let made_by_trade = |item: Item| {
            Role::ALL
                .into_iter()
                .any(|role| role.makes().iter().any(|(made, _)| *made == item))
        };

        for plan in &BUILDINGS {
            for (item, _) in plan.materials {
                assert!(
                    made_by_hand(*item) || made_by_trade(*item),
                    "{:?} takes {item:?}",
                    plan.kind
                );
            }
        }
    }

    #[test]
    fn every_trade_that_takes_a_building_has_one_and_what_it_needs_can_be_known() {
        for plan in &BUILDINGS {
            if let Some(needed) = plan.needs {
                assert!(Advance::ALL.contains(&needed));
            }
        }
        for role in Role::ALL {
            if role.takes_a_workplace() {
                assert!(places(role, &[]).is_some(), "{role:?}");
            } else {
                assert_eq!(places(role, &[]), None, "{role:?}");
            }
        }
    }
}
