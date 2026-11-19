use super::*;
use crate::Command;
use crate::item::Item;
use crate::object::Sex;
use crate::people::Role;
use crate::settlement::Advance;
use crate::simulation::society::tests::{member, put_up, settle, simulation, stock};
use crate::world::World;

fn market_town() -> Simulation {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement
        .advances
        .extend([Advance::Coinage, Advance::Masonry]);
    put_up(&mut sim, StructureKind::Market);
    put_up(&mut sim, StructureKind::Quarry);
    stock(&mut sim, Item::DriedFish, 20_000);
    sim
}

fn last_note(sim: &Simulation) -> &str {
    let entries = sim.state.journal.entries();
    entries.last().map_or("", |entry| &entry.text)
}

#[test]
fn share_in_a_workplace_pays_a_quarter_of_what_its_people_earn_less_taxes() {
    let mut sim = market_town();
    for index in 0..4 {
        member(
            &mut sim,
            &format!("Eero{index}"),
            Sex::Male,
            30,
            Role::Quarrier,
        );
    }

    sim.apply(Command::Invest(StructureKind::Quarry));
    assert_eq!(
        last_note(&sim),
        "A share in a quarry costs 200 coins, and you have 0."
    );
    sim.apply(Command::Invest(StructureKind::Hut));
    assert_eq!(last_note(&sim), "There is no owning a hut.");

    sim.state.purse = 500;
    sim.apply(Command::Invest(StructureKind::Quarry));
    assert_eq!(sim.state.purse, 300);
    assert_eq!(sim.view().holdings, [(StructureKind::Quarry, 1)]);
    sim.apply(Command::Invest(StructureKind::Quarry));
    assert_eq!(last_note(&sim), "There is no quarry left to buy into.");

    // Four quarriers earn 16 coins a day, of which a quarter is 4, less a tenth in
    // taxes, which is nothing of so little.
    sim.work_day(&sim.members());
    assert_eq!(sim.state.income, 4);
    assert_eq!(sim.view().income, 4);
}

#[test]
fn coin_given_to_the_treasury_is_thanked_and_coin_given_to_thinkers_is_thought_with() {
    let mut sim = market_town();
    let someone = member(&mut sim, "Aino", Sex::Female, 30, Role::Idle);
    let regard = |sim: &Simulation| sim.state.world.person(someone).expect("there").regard;
    let before = regard(&sim);

    sim.apply(Command::Donate(100));
    assert_eq!(last_note(&sim), "You have nothing to give.");

    sim.state.purse = 1_000;
    sim.apply(Command::Donate(600));
    assert_eq!(sim.state.purse, 400);
    assert_eq!(regard(&sim), before + 5);
    let settlement = sim.state.settlement.as_ref().expect("founded");
    assert_eq!(settlement.treasury, 600);
    assert!(
        settlement
            .chronicle
            .iter()
            .any(|entry| entry.text.ends_with("gave 600 coins to the treasury."))
    );

    sim.apply(Command::Endow(100));
    assert_eq!(
        last_note(&sim),
        "Nothing is being worked out to give towards."
    );
    sim.apply(Command::Pursue(Advance::Pottery));
    sim.apply(Command::Endow(100));
    assert_eq!(sim.state.purse, 300);
    assert_eq!(sim.state.settlement.as_ref().expect("founded").insight, 200);
}

#[test]
fn chronicle_keeps_what_is_worth_remembering() {
    let mut sim = market_town();
    let chronicle = |sim: &Simulation| -> Vec<String> {
        let town = sim.view().settlement.expect("founded");
        town.chronicle
            .iter()
            .map(|entry| entry.text.clone())
            .collect()
    };
    assert_eq!(
        chronicle(&sim),
        ["Ada Lord raised a cairn and founded the place."]
    );

    sim.apply(Command::Pursue(Advance::Pottery));
    sim.work_out(Advance::Pottery.cost() * 10);
    sim.apply(Command::Legislate(crate::politics::Law::OpenGates, true));
    sim.refresh_society();

    let told = chronicle(&sim);
    assert_eq!(told[1], "Pottery was worked out.");
    assert_eq!(told[2], "Open gates became the law.");

    for _ in 0..600 {
        sim.chronicle("Another year went by.");
    }
    sim.refresh_society();
    assert_eq!(chronicle(&sim).len(), LONGEST_CHRONICLE);
    assert_eq!(chronicle(&sim)[0], told[0]);
}

#[test]
fn kin_are_told_dead_or_alive() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    let me = sim.state.world.protagonist();
    let partner = member(&mut sim, "Eero", Sex::Male, 30, Role::Fisher);
    let child = member(&mut sim, "Aino", Sex::Female, 8, Role::Idle);
    let stranger = member(&mut sim, "Otso", Sex::Male, 40, Role::Fisher);
    let world = &mut sim.state.world;
    world.person_mut(partner).expect("there").partner = me;
    if let Some(mine) = me.and_then(|me| world.person_mut(me)) {
        mine.partner = Some(partner);
    }
    world.person_mut(child).expect("there").parents = [me, Some(partner)];
    let _ = stranger;

    sim.person_dies(child, "a sickness");
    sim.refresh_society();

    let kin = sim.view().kin;
    assert_eq!(kin.len(), 2);
    assert_eq!(
        (kin[0].relation, kin[0].alive, kin[0].here),
        ("Partner", true, true)
    );
    assert_eq!(
        (kin[1].relation, kin[1].alive, kin[1].age),
        ("Child", false, 8)
    );
    assert_eq!(kin[1].name, "Aino Koski");
}
