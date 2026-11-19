use super::*;
use crate::Command;
use crate::object::Sex;
use crate::simulation::society::tests::{
    in_store, member, noted, put_up, settle, simulation, stock,
};
use crate::time::Timestamp;
use crate::world::World;

/// A settlement led by the player with a hostile neighbour of 120 people and a friendly
/// one of 60 that knows a great deal.
fn settled() -> Simulation {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    sim.state.neighbours = Neighbour::generate(&mut crate::rng::Rng::new(4));
    sim.state.neighbours[0].people = 120;
    sim.state.neighbours[0].regard = -60;
    sim.state.neighbours[1].people = 60;
    sim.state.neighbours[1].regard = 20;
    sim.state.neighbours[1].advances = 30;
    stock(&mut sim, Item::DriedFish, 20_000);
    sim
}

fn last_note(sim: &Simulation) -> &str {
    let entries = sim.state.journal.entries();
    entries.last().map_or("", |entry| &entry.text)
}

fn guards(sim: &mut Simulation, many: usize) {
    for index in 0..many {
        member(sim, &format!("Otso{index}"), Sex::Male, 30, Role::Guard);
    }
}

#[test]
fn defence_is_what_guards_a_palisade_and_a_wall_are_worth() {
    let mut sim = settled();
    guards(&mut sim, 3);
    // Guards with no barracks to keep watch from are no guards.
    assert_eq!(sim.defence(), 0);

    put_up(&mut sim, StructureKind::Barracks);
    assert_eq!(sim.defence(), 3);
    let bold = sim.members()[0];
    sim.state.world.person_mut(bold).expect("there").traits = vec![Trait::Bold];
    assert_eq!(sim.defence(), 4);

    put_up(&mut sim, StructureKind::Palisade);
    put_up(&mut sim, StructureKind::Wall);
    assert_eq!(sim.defence(), 4 + 6 + 20);
    sim.refresh_society();
    assert_eq!(sim.view().settlement.expect("founded").defence, 30);
}

#[test]
fn raiders_carry_off_what_is_not_defended_and_are_driven_off_from_what_is() {
    let mut sim = settled();
    guards(&mut sim, 16);
    sim.state.settlement.as_mut().expect("founded").treasury = 1_000;
    let members = sim.members();
    let fish = in_store(&sim, Item::DriedFish);

    // Six raiders against nothing. The dice are thrown until they come.
    for _ in 0..40 {
        sim.be_raided(&members);
    }
    assert!(noted(&sim, "Raiders from"));
    assert!(in_store(&sim, Item::DriedFish) < fish);
    assert!(sim.state.settlement.as_ref().expect("founded").treasury < 1_000);
    assert!(sim.members().len() < 16);

    put_up(&mut sim, StructureKind::Wall);
    let (members, fish) = (sim.members(), in_store(&sim, Item::DriedFish));
    for _ in 0..40 {
        sim.be_raided(&members);
    }
    assert_eq!(in_store(&sim, Item::DriedFish), fish);
    assert_eq!(sim.members().len(), members.len());
    assert!(last_note(&sim).ends_with("were driven off at the edge of the settlement."));
}

#[test]
fn envoys_bring_gifts_make_pacts_and_ask_to_be_taught() {
    let mut sim = settled();
    let friend = 1;
    let regard = |sim: &Simulation| sim.state.neighbours[friend].regard;

    sim.apply(Command::Envoy(friend, Mission::Pact));
    assert_eq!(
        last_note(&sim),
        "There is nothing to trade without a market."
    );

    // Where there is no coin, a gift is of food.
    let fish = in_store(&sim, Item::DriedFish);
    sim.apply(Command::Envoy(friend, Mission::Gift));
    assert_eq!(regard(&sim), 30);
    assert_eq!(in_store(&sim, Item::DriedFish), fish - 100);
    sim.apply(Command::Envoy(friend, Mission::Gift));
    assert!(last_note(&sim).ends_with("Wait 30 more days."));

    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement.advances.push(Advance::Coinage);
    settlement.treasury = 500;
    put_up(&mut sim, StructureKind::Market);
    let later = sim.state.time.as_seconds() + 31 * SECONDS_PER_DAY;
    sim.state.time = Timestamp::from_seconds(later);
    sim.apply(Command::Envoy(friend, Mission::Pact));
    assert!(sim.state.neighbours[friend].pact);
    assert_eq!(regard(&sim), 35);
    assert_eq!(sim.neighbourly(), (1, 2));

    sim.state.time = Timestamp::from_seconds(later + 31 * SECONDS_PER_DAY);
    sim.apply(Command::Envoy(friend, Mission::Learn));
    assert!(last_note(&sim).ends_with("is not close enough to you to teach you."));
    sim.state.neighbours[friend].regard = 45;
    sim.apply(Command::Envoy(friend, Mission::Learn));
    assert_eq!(
        last_note(&sim),
        "Nothing is being worked out for them to help with."
    );
    sim.apply(Command::Pursue(Advance::Pottery));
    sim.apply(Command::Envoy(friend, Mission::Learn));
    let settlement = sim.state.settlement.as_ref().expect("founded");
    assert_eq!(settlement.insight, Advance::Pottery.cost() * 2);
    assert_eq!(regard(&sim), 40);
    assert_eq!(sim.view().neighbours.len(), 3);
}

#[test]
fn raid_on_a_neighbour_pays_only_those_who_are_the_stronger() {
    let mut sim = settled();
    let foe = 0;
    sim.apply(Command::Envoy(foe, Mission::Raid));
    assert_eq!(last_note(&sim), "You have no guards to send.");

    guards(&mut sim, 6);
    put_up(&mut sim, StructureKind::Barracks);
    sim.apply(Command::Envoy(foe, Mission::Raid));
    assert_eq!(sim.members().len(), 4);
    assert!(noted(&sim, "Your guards are beaten back from"));

    sim.state.neighbours[foe].people = 36;
    sim.state.neighbours[foe].envoy_on = None;
    sim.apply(Command::Envoy(foe, Mission::Raid));
    assert_eq!(
        sim.state.settlement.as_ref().expect("founded").treasury,
        180
    );
    assert_eq!(in_store(&sim, Item::Grain), 720);
    assert_eq!(sim.state.neighbours[foe].regard, -100);
}

#[test]
fn neighbours_grow_and_old_grudges_fade_unless_the_laws_feed_them() {
    let mut sim = settled();
    let people = sim.state.neighbours[0].people;

    sim.reckon_with_neighbours();
    assert!(sim.state.neighbours[0].people > people);
    assert_eq!(sim.state.neighbours[0].regard, -59);
    assert_eq!(sim.state.neighbours[1].regard, 19);

    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement.laws.push(Law::ClosedGates);
    sim.reckon_with_neighbours();
    assert_eq!(sim.state.neighbours[0].regard, -60);
    assert_eq!(sim.state.neighbours[1].regard, 16);
}

#[test]
fn midsummer_is_kept_where_there_is_food_and_ale() {
    let mut sim = settled();
    for index in 0..6 {
        member(
            &mut sim,
            &format!("Aino{index}"),
            Sex::Female,
            30,
            Role::Idle,
        );
    }
    let mood = |sim: &Simulation| sim.state.settlement.as_ref().expect("founded").contentment;

    sim.keep_midsummer(7);
    assert_eq!(mood(&sim), 50);

    stock(&mut sim, Item::Ale, 10);
    sim.keep_midsummer(7);
    assert_eq!(mood(&sim), 56);
    assert_eq!(in_store(&sim, Item::Ale), 7);
    assert!(last_note(&sim).starts_with("Midsummer is kept"));
}

#[test]
fn skill_at_a_trade_tells_in_what_it_yields() {
    let mut sim = settled();
    let smith = member(&mut sim, "Eero", Sex::Male, 40, Role::Quarrier);
    put_up(&mut sim, StructureKind::Quarry);
    sim.state
        .settlement
        .as_mut()
        .expect("founded")
        .advances
        .push(Advance::Masonry);

    sim.work_day(&sim.members());
    assert_eq!(in_store(&sim, Item::Stone), 10);

    sim.state.world.person_mut(smith).expect("there").skill = 100;
    sim.work_day(&sim.members());
    assert_eq!(in_store(&sim, Item::Stone), 23);
    sim.refresh_society();
    assert_eq!(sim.view().people[0].skill, 100);
}

#[test]
fn hard_winter_takes_more_wood_and_a_flood_spoils_food() {
    let mut sim = settled();
    for index in 0..8 {
        member(
            &mut sim,
            &format!("Aino{index}"),
            Sex::Female,
            30,
            Role::Idle,
        );
    }
    put_up(&mut sim, StructureKind::Longhouse);
    stock(&mut sim, Item::Log, 100);
    let members = sim.members();
    sim.conditions.temperature = crate::climate::Temperature::from_tenths(-100);

    sim.winter(&members);
    assert_eq!(in_store(&sim, Item::Log), 98);

    let today = sim.state.time.as_seconds() / SECONDS_PER_DAY;
    sim.state
        .settlement
        .as_mut()
        .expect("founded")
        .hard_winter_until = today + 10;
    sim.winter(&members);
    assert_eq!(in_store(&sim, Item::Log), 95);

    let fish = in_store(&sim, Item::DriedFish);
    sim.flood();
    assert_eq!(in_store(&sim, Item::DriedFish), fish - fish / 10);
    assert!(noted(&sim, "The thaw has come all at once"));
}

#[test]
fn crime_grows_where_many_are_idle_and_guards_keep_it_down() {
    let mut sim = settled();
    for index in 0..30 {
        member(
            &mut sim,
            &format!("Aino{index}"),
            Sex::Female,
            30,
            Role::Idle,
        );
    }
    for _ in 0..8 {
        put_up(&mut sim, StructureKind::Hut);
    }
    let crime = |sim: &Simulation| sim.state.settlement.as_ref().expect("founded").crime;

    for _ in 0..60 {
        sim.work_day(&sim.members());
    }
    // Everyone idle: as bad as it gets for a place this small.
    assert_eq!(crime(&sim), 35);

    put_up(&mut sim, StructureKind::Barracks);
    for id in sim.members().into_iter().take(2) {
        sim.state.world.person_mut(id).expect("there").role = Role::Guard;
    }
    for _ in 0..60 {
        sim.work_day(&sim.members());
    }
    assert!(crime(&sim) <= 20, "{}", crime(&sim));
    sim.refresh_society();
    assert_eq!(sim.view().settlement.expect("founded").crime, crime(&sim));
}

#[test]
fn priests_and_performers_make_for_a_more_content_town() {
    let mut sim = settled();
    for index in 0..10 {
        member(
            &mut sim,
            &format!("Aino{index}"),
            Sex::Female,
            30,
            Role::Fisher,
        );
    }
    for _ in 0..3 {
        put_up(&mut sim, StructureKind::Hut);
    }
    let mood = |sim: &mut Simulation| {
        for _ in 0..80 {
            sim.work_day(&sim.members());
        }
        sim.state.settlement.as_ref().expect("founded").contentment
    };
    let plain = mood(&mut sim);

    put_up(&mut sim, StructureKind::Shrine);
    put_up(&mut sim, StructureKind::Sauna);
    put_up(&mut sim, StructureKind::Theatre);
    let ids = sim.members();
    sim.state.world.person_mut(ids[0]).expect("there").role = Role::Priest;
    sim.state.world.person_mut(ids[1]).expect("there").role = Role::Performer;
    let cultured = mood(&mut sim);

    assert!(cultured >= plain + 9, "{plain} to {cultured}");
}

#[test]
fn own_child_is_apprenticed_to_a_trade_and_learns_it_before_working_at_it() {
    let mut sim = settled();
    let me = sim.state.world.protagonist();
    let child = member(&mut sim, "Aino", Sex::Female, 9, Role::Idle);
    let other = member(&mut sim, "Eero", Sex::Male, 9, Role::Idle);
    sim.state.world.person_mut(child).expect("there").parents = [me, None];

    sim.apply(Command::Apprentice(other, Role::Smith));
    assert_eq!(
        last_note(&sim),
        "Only your own child is yours to apprentice."
    );
    sim.apply(Command::Apprentice(child, Role::Smith));
    assert!(last_note(&sim).starts_with("Aino is to learn the trade of a smith"));

    let mut day = sim.state.time.as_seconds() / SECONDS_PER_DAY;
    for _ in 0..40 {
        day += 1;
        sim.state.time = Timestamp::from_seconds(day * SECONDS_PER_DAY);
        sim.practise(&sim.members());
    }
    let learned = |sim: &Simulation, id| sim.state.world.person(id).expect("there").skill;
    assert!(learned(&sim, child) >= 2);
    assert_eq!(learned(&sim, other), 0);
    sim.refresh_society();
    assert!(sim.view().people.iter().any(|person| person.apprenticed));
}

#[test]
fn industry_fouls_the_air_and_the_autumn_fair_fills_the_treasury() {
    let mut sim = settled();
    for kind in [
        StructureKind::Factory,
        StructureKind::Factory,
        StructureKind::Steelworks,
        StructureKind::PowerStation,
    ] {
        put_up(&mut sim, kind);
    }
    let healthy = sim.sickness_risk(1);
    sim.work_day(&sim.members());
    let running = sim.state.settlement.as_ref().expect("founded").running;
    assert_eq!(running.pollution, 19);
    assert!(sim.sickness_risk(1) > healthy);

    put_up(&mut sim, StructureKind::Waterworks);
    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement.advances.push(Advance::CleanPower);
    sim.work_day(&sim.members());
    let running = sim.state.settlement.as_ref().expect("founded").running;
    assert_eq!(running.pollution, 1);

    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement.running.trade = 100;
    settlement.advances.push(Advance::Coinage);
    sim.hold_fair();
    assert_eq!(sim.state.settlement.as_ref().expect("founded").treasury, 70);
    assert!(last_note(&sim).starts_with("The autumn fair fills the market"));
}
