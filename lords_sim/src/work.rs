//! What the trades of a settlement turn into what in a day.

use crate::advance::Advance;
use crate::item::Item;
use crate::people::Role;

/// Something someone of a trade does with a day.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Process {
    pub role: Role,
    /// What the settlement has to know for it to be done.
    pub needs: Option<Advance>,
    /// What is taken out of the store for it.
    pub takes: &'static [(Item, u32)],
    /// What is put in the store by it, the thing it is done for first.
    pub makes: &'static [(Item, u32)],
    /// Whether it is done on the side, on top of the main work of the day.
    pub aside: bool,
}

const fn main(
    role: Role,
    needs: Option<Advance>,
    takes: &'static [(Item, u32)],
    makes: &'static [(Item, u32)],
) -> Process {
    Process {
        role,
        needs,
        takes,
        makes,
        aside: false,
    }
}

const fn aside(
    role: Role,
    needs: Option<Advance>,
    takes: &'static [(Item, u32)],
    makes: &'static [(Item, u32)],
) -> Process {
    Process {
        role,
        needs,
        takes,
        makes,
        aside: true,
    }
}

/// Everything the trades do, apart from what follows the seasons. Of the main work of a
/// trade, the first that can be done is done.
pub const PROCESSES: [Process; 40] = [
    main(
        Role::Gatherer,
        None,
        &[],
        &[
            (Item::SpruceBough, 8),
            (Item::Stick, 6),
            (Item::SpruceRoot, 4),
            (Item::BirchBark, 3),
            (Item::Stone, 2),
        ],
    ),
    aside(
        Role::Gatherer,
        Some(Advance::Pottery),
        &[],
        &[(Item::Clay, 4)],
    ),
    aside(
        Role::Gatherer,
        Some(Advance::Glassmaking),
        &[],
        &[(Item::Sand, 4)],
    ),
    aside(
        Role::Gatherer,
        Some(Advance::Ironworking),
        &[],
        &[(Item::BogOre, 2)],
    ),
    main(
        Role::Woodcutter,
        None,
        &[],
        &[(Item::Log, 4), (Item::Stick, 8)],
    ),
    main(
        Role::Trapper,
        None,
        &[],
        &[(Item::CookedMeat, 10), (Item::HarePelt, 1)],
    ),
    main(Role::Fisher, None, &[], &[(Item::CookedFish, 24)]),
    main(
        Role::Hunter,
        None,
        &[],
        &[(Item::CookedMeat, 9), (Item::Hide, 1)],
    ),
    main(
        Role::Herder,
        None,
        &[],
        &[
            (Item::CookedMeat, 6),
            (Item::Cheese, 6),
            (Item::Wool, 2),
            (Item::Hide, 1),
        ],
    ),
    main(Role::Brewer, None, &[(Item::Grain, 12)], &[(Item::Ale, 24)]),
    main(Role::Innkeeper, None, &[(Item::Ale, 10)], &[]),
    main(Role::Quarrier, None, &[], &[(Item::Stone, 10)]),
    main(
        Role::Miner,
        None,
        &[],
        &[(Item::CopperOre, 3), (Item::TinOre, 1)],
    ),
    aside(
        Role::Miner,
        Some(Advance::Ironworking),
        &[],
        &[(Item::IronOre, 3)],
    ),
    aside(Role::Miner, Some(Advance::Steel), &[], &[(Item::Coal, 5)]),
    main(
        Role::Burner,
        None,
        &[(Item::Log, 2)],
        &[(Item::Charcoal, 8)],
    ),
    main(
        Role::Smith,
        Some(Advance::Ironworking),
        &[(Item::Iron, 1)],
        &[(Item::Tools, 2)],
    ),
    main(Role::Smith, None, &[(Item::Bronze, 1)], &[(Item::Tools, 2)]),
    main(
        Role::Smith,
        Some(Advance::Ironworking),
        &[(Item::IronOre, 3), (Item::Charcoal, 2)],
        &[(Item::Iron, 2)],
    ),
    main(
        Role::Smith,
        Some(Advance::Ironworking),
        &[(Item::BogOre, 3), (Item::Charcoal, 2)],
        &[(Item::Iron, 2)],
    ),
    main(
        Role::Smith,
        None,
        &[(Item::CopperOre, 2), (Item::TinOre, 1), (Item::Charcoal, 2)],
        &[(Item::Bronze, 2)],
    ),
    main(Role::Weaver, None, &[(Item::Wool, 3)], &[(Item::Cloth, 2)]),
    main(Role::Sawyer, None, &[(Item::Log, 2)], &[(Item::Plank, 8)]),
    aside(
        Role::Sawyer,
        Some(Advance::Papermaking),
        &[(Item::Log, 1)],
        &[(Item::Paper, 10)],
    ),
    main(
        Role::Brickmaker,
        None,
        &[(Item::Log, 1)],
        &[(Item::Brick, 10)],
    ),
    main(
        Role::Brickmaker,
        Some(Advance::Concrete),
        &[(Item::Coal, 1)],
        &[(Item::Cement, 10)],
    ),
    main(
        Role::Glassmaker,
        None,
        &[(Item::Charcoal, 2)],
        &[(Item::Glass, 4)],
    ),
    main(Role::Printer, None, &[(Item::Paper, 4)], &[(Item::Book, 1)]),
    main(
        Role::Steelworker,
        None,
        &[(Item::IronOre, 4), (Item::Coal, 4)],
        &[(Item::Steel, 3)],
    ),
    main(
        Role::Steelworker,
        None,
        &[(Item::Iron, 3), (Item::Coal, 3)],
        &[(Item::Steel, 3)],
    ),
    main(
        Role::FactoryHand,
        None,
        &[(Item::Steel, 1)],
        &[(Item::Tools, 3)],
    ),
    main(
        Role::FactoryHand,
        Some(Advance::Electricity),
        &[(Item::CopperOre, 2)],
        &[(Item::Wire, 3)],
    ),
    main(
        Role::FactoryHand,
        None,
        &[(Item::Steel, 2)],
        &[(Item::MachineParts, 1)],
    ),
    main(
        Role::MillHand,
        None,
        &[(Item::Wool, 8)],
        &[(Item::Cloth, 8)],
    ),
    main(
        Role::Canner,
        None,
        &[(Item::DriedFish, 12), (Item::Iron, 1)],
        &[(Item::CannedFood, 6)],
    ),
    main(
        Role::Canner,
        None,
        &[(Item::CookedFish, 12), (Item::Iron, 1)],
        &[(Item::CannedFood, 6)],
    ),
    main(Role::Engineer, Some(Advance::CleanPower), &[], &[]),
    main(Role::Engineer, None, &[(Item::Coal, 6)], &[]),
    main(Role::Mechanic, None, &[(Item::Oil, 2)], &[]),
    aside(Role::Healer, None, &[], &[(Item::Medicine, 1)]),
];

impl Role {
    /// Returns everything those of the role may put in the store, and how many of each
    /// in a day at the most.
    pub fn makes(self) -> Vec<(Item, u32)> {
        let mut made: Vec<(Item, u32)> = Vec::new();
        for process in PROCESSES.iter().filter(|process| process.role == self) {
            for (item, count) in process.makes {
                match made.iter_mut().find(|(known, _)| known == item) {
                    Some((_, most)) => *most = (*most).max(*count),
                    None => made.push((*item, *count)),
                }
            }
        }
        made
    }
}

/// Returns how many of `item` a settlement of `people` keeps in store at the most. What
/// is made beyond that is not made, so that nothing piles up without end. There is no
/// such thing as too much food.
pub fn ceiling(item: Item, people: u32) -> u32 {
    if item.food().is_some() && item != Item::Ale {
        return u32::MAX;
    }

    let (per_ten, floor) = match item {
        // Enough logs to see everyone through a winter.
        Item::Log => (150, 100),
        Item::Stone | Item::Plank => (30, 80),
        Item::Stick => (20, 60),
        Item::SpruceBough => (10, 60),
        Item::BirchBark | Item::Sand => (5, 30),
        Item::SpruceRoot | Item::BogOre => (5, 20),
        Item::Clay => (10, 40),
        Item::CopperOre | Item::TinOre | Item::IronOre => (10, 30),
        Item::Charcoal => (15, 40),
        Item::Coal => (25, 60),
        Item::Bronze | Item::Iron | Item::Steel => (8, 30),
        Item::Brick => (60, 200),
        Item::Cement => (60, 300),
        Item::Glass => (15, 60),
        Item::Cloth | Item::Wool => (6, 20),
        Item::Hide | Item::HarePelt | Item::Cordage => (3, 12),
        Item::Paper => (10, 60),
        Item::Book => (2, 40),
        Item::MachineParts => (2, 20),
        Item::Wire => (8, 60),
        Item::Tools => (3, 4),
        Item::Medicine => (2, 5),
        Item::Ale => (30, 40),
        Item::Oil => (5, 20),
        _ => (5, 20),
    };
    per_ten * people / 10 + floor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_process_is_of_a_trade_and_does_something() {
        for process in &PROCESSES {
            assert!(
                !process.takes.is_empty() || !process.makes.is_empty() || process.needs.is_some(),
                "{process:?}"
            );
            assert!(process.takes.iter().all(|(_, count)| *count > 0));
            assert!(process.makes.iter().all(|(_, count)| *count > 0));
        }
        assert!(Role::Smith.makes().contains(&(Item::Tools, 2)));
        assert!(Role::Thinker.makes().is_empty());
    }

    /// What a trade takes is something another trade makes, or food that is grown or
    /// caught.
    #[test]
    fn nothing_is_taken_that_nobody_makes() {
        let made = |item: Item| {
            Role::ALL
                .into_iter()
                .any(|role| role.makes().iter().any(|(made, _)| *made == item))
        };

        for process in &PROCESSES {
            for (item, _) in process.takes {
                let grown = matches!(item, Item::Grain | Item::DriedFish | Item::Oil);
                assert!(grown || made(*item), "{:?} takes {item:?}", process.role);
            }
        }
    }

    #[test]
    fn stores_are_kept_to_what_a_settlement_can_use() {
        assert_eq!(ceiling(Item::Grain, 10), u32::MAX);
        assert_eq!(ceiling(Item::Log, 10), 250);
        assert_eq!(ceiling(Item::Stone, 10), 110);
        assert_eq!(ceiling(Item::Tools, 30), 13);
        assert!(ceiling(Item::Ale, 10) < u32::MAX);
        assert!(ceiling(Item::Brick, 200) > ceiling(Item::Brick, 20));
    }
}
