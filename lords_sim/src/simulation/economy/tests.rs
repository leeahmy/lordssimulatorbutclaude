use super::*;
use crate::Command;
use crate::object::{Living, Name, Sex};
use crate::settlement::Age;
use crate::simulation::society::tests::{
    in_store, member, pass_days, put_up, settle, simulation, stock,
};
use crate::time::Timestamp;
use crate::time::gregorian::SECONDS_PER_DAY;
use crate::world::World;

fn know(sim: &mut Simulation, advances: &[Advance]) {
    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement.advances.extend(advances);
}

/// Lets `days` pass for the settlement without the player character living through
/// every second of them.
pub(crate) fn pass_days_coarsely(sim: &mut Simulation, days: u64) {
    for _ in 0..days {
        let next = sim.state.time.as_seconds() + SECONDS_PER_DAY;
        sim.state.time = Timestamp::from_seconds(next);
        sim.conditions = sim.state.climate.conditions(sim.state.time);
        sim.live_a_day();
        sim.kept.retain(|id| sim.state.world.thing(*id).is_some());
    }
    sim.look();
    sim.refresh_society();
}

fn summary(sim: &Simulation) -> String {
    let town = sim.view().settlement.expect("there is a settlement");
    let buildings: u32 = town.buildings.iter().map(|(_, count)| count).sum();
    format!(
        "{} people in {} buildings, {:?}, {:?}, {} advances, housing {}, food {} days, \
         treasury {}, tax {}, content {}, literacy {}, pursuing {:?}",
        town.people,
        buildings,
        town.politics.era,
        town.age,
        town.advances.len(),
        town.housing,
        town.food_days,
        town.treasury,
        town.tax,
        town.contentment,
        town.literacy,
        town.pursuit,
    )
}

/// How a hamlet fares over the generations with nobody but custom to run it. Run with
/// `--ignored --nocapture` to read how it went.
#[test]
#[ignore = "takes a while, and is for reading"]
fn settlement_left_to_itself_comes_of_age() {
    for seed in [1, 2] {
        let mut sim = Simulation::generate(Name::default(), Sex::Female, seed);
        let mut age = Age::Stone;
        let mut era = sim.era();
        println!("seed {seed}");

        for year in 0..400 {
            // Whoever leads, custom runs the place.
            sim.state.settlement.as_mut().expect("there").steward = true;
            let began = std::time::Instant::now();
            pass_days_coarsely(&mut sim, 365);
            let took = began.elapsed().as_micros() / 365;
            let town = sim.view().settlement.expect("the hamlet is still there");
            let reached = town.age != age || town.politics.era != era;
            age = town.age;
            era = town.politics.era;
            if reached || year % 10 == 9 {
                println!("  year {:3}: {} ({took} µs a day)", year + 1, summary(&sim));
            }
            if year % 50 == 49 {
                let built: Vec<String> = town
                    .buildings
                    .iter()
                    .map(|(kind, count)| format!("{kind:?}×{count}"))
                    .collect();
                println!("    built: {}", built.join(" "));
                let jobs: Vec<String> = town
                    .jobs
                    .iter()
                    .map(|(role, working, places)| match places {
                        Some(places) => format!("{role:?} {working}/{places}"),
                        None => format!("{role:?} {working}"),
                    })
                    .collect();
                println!("    jobs: {}", jobs.join(" "));
                let store: Vec<String> = town
                    .store
                    .stacks()
                    .iter()
                    .map(|stack| format!("{:?} {}", stack.item, stack.count))
                    .collect();
                println!("    store: {}", store.join(", "));
                println!("    ledger: {:?}", town.ledger);
            }
            if town.people < 4 {
                println!(
                    "  nearly everyone is gone in year {}: {}",
                    year + 1,
                    summary(&sim)
                );
                for entry in sim.state.journal.entries().iter().rev().take(40).rev() {
                    println!("    {}", entry.text);
                }
                break;
            }
            if town.advances.len() == Advance::ALL.len() {
                println!("  everything is known in year {}", year + 1);
                break;
            }
        }
    }
}

#[test]
fn trades_turn_what_is_in_the_store_into_what_they_make() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    know(
        &mut sim,
        &[Advance::Charcoal, Advance::Mining, Advance::Bronze],
    );
    put_up(&mut sim, StructureKind::CharcoalKiln);
    put_up(&mut sim, StructureKind::Smithy);
    put_up(&mut sim, StructureKind::Mine);
    member(&mut sim, "Aino", Sex::Female, 30, Role::Burner);
    member(&mut sim, "Eero", Sex::Male, 30, Role::Miner);
    member(&mut sim, "Otso", Sex::Male, 30, Role::Smith);
    stock(&mut sim, Item::Log, 10);
    stock(&mut sim, Item::DriedFish, 400);

    // The trades work in the order one hands on to the next: the burner and the miner
    // first, and the smith with what they brought in.
    pass_days_coarsely(&mut sim, 1);
    assert_eq!(in_store(&sim, Item::Log), 8);
    assert_eq!(in_store(&sim, Item::Charcoal), 6);
    assert_eq!(in_store(&sim, Item::CopperOre), 1);
    assert_eq!(in_store(&sim, Item::TinOre), 0);
    assert_eq!(in_store(&sim, Item::Bronze), 2);
    assert_eq!(sim.tools_percent(), 100);

    // With bars in the store, tools are forged before more ore is smelted.
    pass_days_coarsely(&mut sim, 1);
    assert_eq!(in_store(&sim, Item::Tools), 2);
    assert_eq!(in_store(&sim, Item::Bronze), 1);
    assert_eq!(sim.tools_percent(), 115);
}

#[test]
fn trade_that_takes_a_building_is_plied_only_by_as_many_as_it_has_room_for() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    know(&mut sim, &[Advance::Masonry]);
    for index in 0..6 {
        member(
            &mut sim,
            &format!("Eero{index}"),
            Sex::Male,
            30,
            Role::Quarrier,
        );
    }
    stock(&mut sim, Item::DriedFish, 2_000);

    pass_days_coarsely(&mut sim, 1);
    assert_eq!(in_store(&sim, Item::Stone), 0);

    put_up(&mut sim, StructureKind::Quarry);
    pass_days_coarsely(&mut sim, 1);
    assert_eq!(in_store(&sim, Item::Stone), 40);
    let town = sim.view().settlement.expect("founded");
    assert!(town.jobs.contains(&(Role::Quarrier, 4, Some(4))));
}

#[test]
fn nothing_is_made_beyond_what_a_settlement_can_use() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    member(&mut sim, "Aino", Sex::Female, 30, Role::Woodcutter);
    stock(&mut sim, Item::DriedFish, 5_000);

    pass_days_coarsely(&mut sim, 60);

    assert_eq!(in_store(&sim, Item::Log), ceiling(Item::Log, 2));
}

#[test]
fn lake_and_forest_feed_only_so_many() {
    let catch = |fishers: usize| {
        let mut sim = simulation(World::default());
        settle(&mut sim);
        for index in 0..fishers {
            member(
                &mut sim,
                &format!("Aino{index}"),
                Sex::Female,
                30,
                Role::Fisher,
            );
        }
        let before = in_store(&sim, Item::CookedFish);
        sim.work_day(&sim.members());
        in_store(&sim, Item::CookedFish) - before
    };

    assert_eq!(catch(1), 24);
    assert_eq!(catch(10), 240);
    // Each one beyond what the lake feeds brings in a quarter of what the others do.
    assert_eq!(catch(14), 264);
}

#[test]
fn coin_brings_taxes_wages_and_trade() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    know(&mut sim, &[Advance::Coinage]);
    put_up(&mut sim, StructureKind::Market);
    for index in 0..6 {
        member(
            &mut sim,
            &format!("Aino{index}"),
            Sex::Female,
            30,
            Role::Woodcutter,
        );
    }
    member(&mut sim, "Eero", Sex::Male, 30, Role::Merchant);
    member(&mut sim, "Otso", Sex::Male, 30, Role::Thinker);
    stock(&mut sim, Item::DriedFish, 5_000);
    stock(&mut sim, Item::HarePelt, 60);
    sim.state.settlement.as_mut().expect("founded").treasury = 1_000;
    assert!(sim.monetized());

    pass_days_coarsely(&mut sim, 2);

    let town = sim.view().settlement.expect("founded");
    // Six woodcutters and a merchant earn 24 coins a day between them, of which a
    // tenth is raised, and of that three fifths are had with no clerks to see to it. A
    // tenth of what is sold outside is raised on top.
    assert_eq!(town.ledger.taxes, 1 + town.ledger.sales / 10);
    assert!(town.ledger.sales > 0, "{:?}", town.ledger);
    // The thinker is paid out of the treasury, and so is whoever leads.
    assert_eq!(town.ledger.wages, i64::from(Role::Thinker.wage()) + 10);
    assert!(!town.ledger.unpaid);
    assert!(town.treasury > 0);
    assert!(in_store(&sim, Item::HarePelt) < 60);
    assert_eq!(town.running.trade, 30);

    // What whoever leads sends for is bought, as far as the treasury goes.
    sim.place_order(Item::Tools, 1);
    sim.state.settlement.as_mut().expect("founded").treasury = 500;
    pass_days_coarsely(&mut sim, 5);
    assert!(in_store(&sim, Item::Tools) >= 1);
    let town = sim.view().settlement.expect("founded");
    assert!(town.orders.is_empty());
}

#[test]
fn player_buys_and_sells_at_the_market_and_is_paid_for_a_shift() {
    let mut sim = simulation(World::default());
    let cairn = settle(&mut sim);
    know(&mut sim, &[Advance::Coinage, Advance::Masonry]);
    put_up(&mut sim, StructureKind::Market);
    member(&mut sim, "Aino", Sex::Female, 30, Role::Woodcutter);
    stock(&mut sim, Item::DriedFish, 100);
    sim.state.settlement.as_mut().expect("founded").treasury = 100;
    sim.look();

    // Nothing is had from the store without coin.
    sim.apply(Command::Take(Item::DriedFish, 10));
    assert_eq!(sim.state.player.carried().count(Item::DriedFish), 0);
    assert!(
        sim.state
            .journal
            .entries()
            .last()
            .is_some_and(|entry| entry.text.starts_with("That costs 36 coins"))
    );

    // What is brought to the store is sold to it.
    sim.state.player.carried_mut().add(Item::HarePelt, 10);
    sim.apply(Command::Act {
        verb: crate::Verb::Deposit,
        subject: crate::Subject::Thing(cairn),
    });
    crate::simulation::society::tests::run_until_idle(&mut sim);
    assert_eq!(sim.state.purse, 64);
    assert_eq!(sim.state.settlement.as_ref().expect("founded").treasury, 36);

    sim.apply(Command::Take(Item::DriedFish, 10));
    assert_eq!(sim.state.player.carried().count(Item::DriedFish), 10);
    assert_eq!(sim.state.purse, 28);
    assert_eq!(sim.view().purse, 28);
}

#[test]
fn reading_spreads_where_there_are_teachers_and_speeds_what_is_worked_out() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    know(&mut sim, &[Advance::Writing, Advance::Teaching]);
    put_up(&mut sim, StructureKind::School);
    member(&mut sim, "Aino", Sex::Female, 30, Role::Teacher);
    member(&mut sim, "Eero", Sex::Male, 30, Role::Thinker);
    stock(&mut sim, Item::DriedFish, 20_000);
    sim.state.settlement.as_mut().expect("founded").pursuit = Some(Advance::Steel);

    pass_days_coarsely(&mut sim, 400);

    let town = sim.view().settlement.expect("founded");
    assert!((8..=12).contains(&town.literacy), "{}", town.literacy);
    assert_eq!(town.running.teachers, 1);
}

#[test]
fn contentment_follows_how_people_fare() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    member(&mut sim, "Aino", Sex::Female, 30, Role::Idle);
    member(&mut sim, "Eero", Sex::Male, 30, Role::Idle);
    let mood = |sim: &Simulation| sim.state.settlement.as_ref().expect("founded").contentment;
    assert_eq!(mood(&sim), 50);

    // Hungry and without a roof.
    pass_days_coarsely(&mut sim, 5);
    assert_eq!(mood(&sim), 45);

    stock(&mut sim, Item::DriedFish, 20_000);
    put_up(&mut sim, StructureKind::Hut);
    pass_days_coarsely(&mut sim, 30);
    assert!(mood(&sim) > 55, "{}", mood(&sim));
}

#[test]
fn age_is_come_into_with_what_is_worked_out() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    know(
        &mut sim,
        &[Advance::Pottery, Advance::Masonry, Advance::Mining],
    );
    sim.state.settlement.as_mut().expect("founded").pursuit = Some(Advance::Charcoal);

    sim.work_out(Advance::Charcoal.cost() * 10);

    let town = sim.state.settlement.as_ref().expect("founded");
    assert_eq!(town.age(), Age::Bronze);
    assert!(
        sim.state
            .journal
            .entries()
            .last()
            .is_some_and(|entry| entry.text.ends_with("has come into the Bronze Age."))
    );
    let _ = pass_days;
}

/// How long a day of a large town takes when every second of it is lived. Run with
/// `--ignored --nocapture` to read it.
#[test]
#[ignore = "takes a while, and is for reading"]
fn day_in_a_large_town_is_timed() {
    let mut sim = Simulation::generate(Name::default(), Sex::Female, 1);
    for years in [60, 60, 40] {
        sim.state.settlement.as_mut().expect("there").steward = true;
        pass_days_coarsely(&mut sim, 365 * years);
        // Someone young lives through the day, so that it is lived at all.
        let born = sim.state.time.as_seconds() - 25 * 365 * SECONDS_PER_DAY;
        sim.state.player = crate::object::Human::new(
            sim.state.player.location(),
            Timestamp::from_seconds(born),
            Name::default(),
            Sex::Female,
        );
        sim.state.death = None;
        let people = sim.view().people.len();
        let began = std::time::Instant::now();
        for _ in 0..SECONDS_PER_DAY {
            sim.tick();
        }
        println!(
            "{people} people: {} ms for a day lived second by second, dead: {}",
            began.elapsed().as_millis(),
            sim.state.death.is_some()
        );
    }
}
