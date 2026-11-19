use super::*;
use crate::action::Verb;
use crate::craft::Recipe;
use crate::object::{Name, Sex};
use crate::simulation::Command;
use crate::simulation::society::tests::{
    carry, in_store, last_note, member, noted, pass_days, put_up, run_until_idle, settle,
    simulation, stock,
};
use crate::world::{Kind, World};

/// A settlement the player founded and leads, with the `count` grown-ups asked for, who
/// each think of the player what is given.
fn settled(regards: &[u8]) -> (Simulation, Vec<PersonId>) {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    let members = regards
        .iter()
        .enumerate()
        .map(|(index, regard)| {
            let sex = if index % 2 == 0 {
                Sex::Male
            } else {
                Sex::Female
            };
            let id = member(&mut sim, &format!("Eero{index}"), sex, 30, Role::Idle);
            sim.state.world.person_mut(id).expect("just added").regard = *regard;
            id
        })
        .collect();
    stock(&mut sim, Item::DriedMeat, 5_000);
    sim.refresh_society();

    (sim, members)
}

fn person_mut(sim: &mut Simulation, id: PersonId) -> &mut Person {
    sim.state.world.person_mut(id).expect("the person is there")
}

fn regard(sim: &Simulation, id: PersonId) -> u8 {
    sim.state
        .world
        .person(id)
        .expect("the person is there")
        .regard
}

fn lead(sim: &mut Simulation, leader: PersonId) {
    sim.state.settlement.as_mut().expect("founded").leader = Some(leader);
    sim.refresh_society();
}

fn politics(sim: &Simulation) -> PoliticsView {
    sim.politics_view().expect("founded")
}

/// Puts up something built right by the player.
fn put_near(sim: &mut Simulation, kind: StructureKind) {
    let at = sim.site().expect("there is dry ground by the player");
    let structure =
        crate::world::Structure::new(crate::object::Object::new(at, sim.state.time), kind);
    sim.add_thing(crate::world::Thing::Structure(structure));
    sim.look();
}

#[test]
fn founder_leads_and_newcomers_to_a_settlement_have_no_say() {
    let (mut sim, members) = settled(&[10, 10]);
    assert!(sim.leads());
    assert!(politics(&sim).leads);

    lead(&mut sim, members[0]);
    assert_eq!(politics(&sim).leader.as_deref(), Some("Eero0 Koski"));

    sim.apply(Command::Assign(members[1], Role::Fisher));
    assert_eq!(
        last_note(&sim),
        "You have no say in that. Eero0 leads Lordstead."
    );
    assert_eq!(
        sim.state.world.person(members[1]).map(|p| p.role),
        Some(Role::Idle)
    );

    stock(&mut sim, Item::Log, 12);
    stock(&mut sim, Item::SpruceBough, 20);
    for command in [
        Command::Plan(StructureKind::Hut),
        Command::Legislate(Law::Schooling, true),
        Command::SetSteward(true),
    ] {
        sim.apply(command);
        assert_eq!(
            last_note(&sim),
            "You have no say in that. Eero0 leads Lordstead."
        );
    }
    assert_eq!(in_store(&sim, Item::Log), 12);
}

#[test]
fn moot_chooses_whoever_most_would_follow() {
    let (mut sim, members) = settled(&[10, 10, 10]);
    let me = sim.state.world.protagonist();
    person_mut(&mut sim, members[0]).esteem = 60;
    lead(&mut sim, members[0]);

    // Nobody thinks much of the newcomer: only their own voice is for them.
    let view = politics(&sim);
    assert_eq!(view.voices, 4);
    assert_eq!(view.candidates[0].name, "Eero0 Koski");
    assert_eq!(view.candidates[0].voices, 3);
    assert!(
        view.candidates
            .iter()
            .any(|candidate| candidate.you && candidate.voices == 1)
    );
    sim.refresh_society();
    assert!(sim.view().people.iter().all(|person| !person.backs_you));

    sim.hold_moot();
    assert_eq!(
        sim.state.settlement.as_ref().and_then(|s| s.leader),
        Some(members[0])
    );
    assert!(noted(
        &sim,
        "The moot keeps Eero0 Koski as leader of Lordstead, with 3 of 4"
    ));

    // Once they have won the others over, the moot goes their way.
    for id in &members[1..] {
        person_mut(&mut sim, *id).regard = 90;
    }
    sim.refresh_society();
    assert_eq!(
        sim.view()
            .people
            .iter()
            .filter(|person| person.backs_you)
            .count(),
        2
    );

    sim.hold_moot();
    assert_eq!(sim.state.settlement.as_ref().and_then(|s| s.leader), me);
    assert_eq!(
        last_note(&sim),
        "The moot chooses you as leader of Lordstead, with 3 of 4 voices."
    );
    assert!(sim.take_alert());
}

#[test]
fn like_minded_people_and_kin_lean_towards_one_another() {
    let (mut sim, members) = settled(&[40, 40, 40]);
    let me = sim.state.world.protagonist();
    let rival = members[0];
    person_mut(&mut sim, rival).esteem = 45;
    person_mut(&mut sim, members[2]).views = [-2, -2, -2, -2];
    lead(&mut sim, rival);
    let backs = |sim: &Simulation, id| sim.backers().contains(&id);

    // The one who leads is held in higher esteem than the player by both.
    assert!(!backs(&sim, members[1]) && !backs(&sim, members[2]));

    // Once the two stand for different things, those who think like the player back them.
    person_mut(&mut sim, rival).views = [2, 2, 2, 2];
    for issue in Issue::ALL {
        sim.apply(Command::SetView(issue, -2));
    }
    assert_eq!(politics(&sim).views, [-2; 4]);
    assert!(!backs(&sim, members[1]));
    assert!(backs(&sim, members[2]));

    sim.apply(Command::SetView(Issue::Rule, 9));
    assert_eq!(politics(&sim).views[Issue::Rule as usize], 2);

    // Kinship counts for more than a little esteem.
    for issue in Issue::ALL {
        sim.apply(Command::SetView(issue, 0));
    }
    person_mut(&mut sim, rival).views = [0; 4];
    assert!(!backs(&sim, members[1]));
    person_mut(&mut sim, members[1]).parents = [me, None];
    assert!(backs(&sim, members[1]));
}

#[test]
fn leader_of_a_band_makes_laws_alone_and_a_village_votes_on_them() {
    let (mut sim, members) = settled(&[50, 50, 50]);
    person_mut(&mut sim, members[0]).views = [0, 2, 0, 0];
    person_mut(&mut sim, members[1]).views = [0, -2, 0, 0];
    person_mut(&mut sim, members[2]).views = [0, -1, 0, 0];
    assert_eq!(sim.era(), Era::Band);
    assert!(politics(&sim).decides_alone);
    assert_eq!(politics(&sim).ayes[Law::Schooling as usize], 1);

    sim.apply(Command::Legislate(Law::Schooling, true));
    assert!(sim.lives_by(Law::Schooling));
    assert!(last_note(&sim).starts_with("Schooling is now the law."));
    assert_eq!(regard(&sim, members[0]), 53);
    assert_eq!(regard(&sim, members[1]), 47);
    assert_eq!(sim.working_age(), 14);

    // The opposite law takes its place.
    sim.apply(Command::Legislate(Law::OldCustoms, true));
    assert_eq!(politics(&sim).laws, [Law::OldCustoms]);

    // Grown to a village, the law is put to the grown-ups, most of whom like it.
    for index in 3..8 {
        let id = member(
            &mut sim,
            &format!("Aino{index}"),
            Sex::Female,
            30,
            Role::Idle,
        );
        person_mut(&mut sim, id).views = [0, -1, 0, 0];
    }
    assert_eq!(sim.era(), Era::Village);
    assert!(!politics(&sim).decides_alone);

    sim.apply(Command::Legislate(Law::Schooling, true));
    assert_eq!(
        last_note(&sim),
        "The moot will not have schooling: 1 of 8 voices for it."
    );
    sim.apply(Command::Legislate(Law::OldCustoms, false));
    assert!(last_note(&sim).starts_with("The moot will not give up the old customs"));
    assert!(sim.lives_by(Law::OldCustoms));

    // Under the elder's word, nobody is asked.
    person_mut(&mut sim, members[0]).views = [0, 2, -2, 0];
    for id in sim.members() {
        person_mut(&mut sim, id).views[Issue::Rule as usize] = -1;
    }
    sim.apply(Command::Legislate(Law::EldersWord, true));
    sim.apply(Command::Legislate(Law::Schooling, true));
    assert!(sim.lives_by(Law::EldersWord) && sim.lives_by(Law::Schooling));
    assert_eq!(politics(&sim).next_moot, None);
}

#[test]
fn laws_change_what_is_brought_in_eaten_and_worked_out() {
    let (mut sim, members) = settled(&[50, 50]);
    person_mut(&mut sim, members[0]).role = Role::Fisher;
    person_mut(&mut sim, members[1]).role = Role::Thinker;
    put_up(&mut sim, StructureKind::Smokehouse);
    sim.apply(Command::Pursue(Advance::Writing));
    sim.apply(Command::Pursue(Advance::Pottery));
    let meat = in_store(&sim, Item::DriedMeat);

    pass_days(&mut sim, 1);
    assert_eq!(in_store(&sim, Item::DriedFish), 24);
    assert_eq!(politics(&sim).era, Era::Band);
    let insight = sim.state.settlement.as_ref().expect("founded").insight;
    // A thinker, a little that comes by itself, and a tenth more for the two
    // neighbours who know more.
    assert_eq!(insight, 14);
    // A fisher eats 2,500 kcal and a thinker half as much again: 15 helpings of meat.
    assert_eq!(meat - in_store(&sim, Item::DriedMeat), 15);

    sim.apply(Command::Legislate(Law::OwnHearths, true));
    sim.apply(Command::Legislate(Law::Schooling, true));
    pass_days(&mut sim, 1);
    assert_eq!(in_store(&sim, Item::DriedFish), 24 + 27);
    let insight = sim.state.settlement.as_ref().expect("founded").insight - insight;
    assert_eq!(insight, 21);
}

#[test]
fn people_take_note_of_what_the_player_brings_to_the_store_and_takes_out_of_it() {
    let (mut sim, members) = settled(&[20, 20]);
    let cairn = sim.state.settlement.as_ref().expect("founded").cairn;
    carry(&mut sim, Item::DriedFish, 60);

    sim.apply(Command::Act {
        verb: Verb::Deposit,
        subject: crate::world::Subject::Thing(cairn),
    });
    run_until_idle(&mut sim);

    // Sixty dried perch are 12,000 kcal: taken note of twice over.
    assert_eq!(politics(&sim).credit, 12_000);
    assert_eq!(regard(&sim, members[0]), 22);
    assert_eq!(
        last_note(&sim),
        "People take note of what you bring to the common store."
    );

    sim.apply(Command::Take(Item::DriedMeat, 60));
    assert_eq!(politics(&sim).credit, 12_000 - 60 * 430);
    assert_eq!(regard(&sim, members[0]), 17);
    assert!(noted(
        &sim,
        "People take note that you take more out of the store than you bring to it."
    ));
}

#[test]
fn work_on_a_building_of_the_settlement_counts_for_the_player() {
    let (mut sim, members) = settled(&[20]);
    stock(&mut sim, Item::Log, 12);
    stock(&mut sim, Item::SpruceBough, 20);
    sim.apply(Command::Plan(StructureKind::Hut));
    let site = sim
        .view()
        .map
        .things
        .iter()
        .find(|mark| mark.kind == Kind::Structure(StructureKind::Site))
        .map(|mark| mark.subject)
        .expect("the site is on the map");

    sim.apply(Command::Act {
        verb: Verb::Build,
        subject: site,
    });
    for _ in 0..4 * 3_650 {
        sim.tick();
    }
    sim.apply(Command::Stop);

    assert_eq!(politics(&sim).credit, 4 * HOUR_OF_WORK);
    assert_eq!(regard(&sim, members[0]), 21);
}

#[test]
fn speech_feast_and_notices_win_people_over_each_in_its_time() {
    let (mut sim, members) = settled(&[20, 20]);
    person_mut(&mut sim, members[0]).views = [2, 2, 2, 2];
    person_mut(&mut sim, members[1]).views = [-2, -2, -2, -2];
    for issue in Issue::ALL {
        sim.apply(Command::SetView(issue, 2));
    }

    sim.apply(Command::Appeal(Appeal::Speech));
    assert_eq!(sim.view().work.map(|work| work.needed), Some(3_600));
    run_until_idle(&mut sim);
    assert_eq!(regard(&sim, members[0]), 23);
    assert_eq!(regard(&sim, members[1]), 19);
    assert!(last_note(&sim).ends_with("1 nod along, and 1 turn away."));
    assert_eq!(politics(&sim).appeal_rest, [1, 0, 0, 0, 0]);
    sim.apply(Command::Appeal(Appeal::Speech));
    assert_eq!(
        last_note(&sim),
        "People have had enough of that for now. Wait 1 more days."
    );

    sim.apply(Command::Appeal(Appeal::Feast));
    assert_eq!(
        last_note(&sim),
        "A feast for 2 takes 3000 kcal of your own food, carried or piled up here. You have 0."
    );
    carry(&mut sim, Item::DriedMeat, 10);
    sim.apply(Command::Appeal(Appeal::Feast));
    run_until_idle(&mut sim);
    assert_eq!(regard(&sim, members[1]), 27);
    assert_eq!(sim.state.player.carried().count(Item::DriedMeat), 3);
    assert_eq!(politics(&sim).appeal_rest[1], 30);

    sim.apply(Command::Appeal(Appeal::Notices));
    assert_eq!(last_note(&sim), "Nobody here can read or write yet.");
    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement.advances.push(Advance::Writing);
    settlement.literacy = 100;
    carry(&mut sim, Item::BirchBark, 5);
    sim.apply(Command::Appeal(Appeal::Notices));
    run_until_idle(&mut sim);
    assert_eq!(regard(&sim, members[1]), 29);
    assert_eq!(sim.state.player.carried().count(Item::BirchBark), 0);

    sim.state
        .player
        .animal_mut()
        .move_to(crate::Location::new(90_000, 0));
    sim.look();
    pass_days(&mut sim, 2);
    sim.state
        .player
        .animal_mut()
        .move_to(crate::Location::new(90_000, 0));
    sim.look();
    sim.apply(Command::Appeal(Appeal::Speech));
    assert_eq!(
        last_note(&sim),
        "You have to be by the cairn or a storehouse to be heard."
    );
}

#[test]
fn anyone_can_pay_someone_to_think_and_set_them_to_what_nobody_is_working_out() {
    let (mut sim, members) = settled(&[20, 20]);
    lead(&mut sim, members[1]);
    let hire = Command::Act {
        verb: Verb::Hire,
        subject: crate::world::Subject::Person(members[0]),
    };

    sim.apply(hire.clone());
    assert_eq!(
        last_note(&sim),
        "Eero0 asks 10 days' keep for it: 12500 kcal of food out of what you carry. You carry 0."
    );

    carry(&mut sim, Item::DriedMeat, 30);
    sim.apply(hire.clone());
    run_until_idle(&mut sim);
    assert_eq!(sim.state.player.carried().count(Item::DriedMeat), 0);
    assert!(sim.view().people.iter().any(|person| person.hired));
    sim.apply(hire);
    assert_eq!(last_note(&sim), "Eero0 is thinking for you already.");

    sim.apply(Command::Pursue(Advance::Pottery));
    assert!(!politics(&sim).leads);
    let pursuit = |sim: &Simulation| sim.state.settlement.as_ref().and_then(|s| s.pursuit);
    assert_eq!(pursuit(&sim), Some(Advance::Pottery));
    sim.apply(Command::Pursue(Advance::Smoking));
    assert_eq!(pursuit(&sim), Some(Advance::Pottery));

    // The one who leads is not the player, so the settlement runs itself and may set
    // the hired one to other work, which they leave undone for the ten days.
    let before = sim.state.settlement.as_ref().expect("founded").insight;
    pass_days(&mut sim, 5);
    let after = sim.state.settlement.as_ref().expect("founded").insight;
    assert!(after - before >= 5 * 13, "{before} to {after}");

    pass_days(&mut sim, 10);
    assert!(sim.view().people.iter().all(|person| !person.hired));
}

#[test]
fn settlement_led_by_another_runs_itself() {
    let (mut sim, members) = settled(&[20, 20, 20, 20, 20, 20]);
    lead(&mut sim, members[0]);
    stock(&mut sim, Item::Log, 30);
    stock(&mut sim, Item::SpruceBough, 40);

    pass_days(&mut sim, 1);

    let roles: Vec<Role> = sim.view().people.iter().map(|person| person.role).collect();
    assert!(roles.contains(&Role::Woodcutter), "{roles:?}");
    assert!(
        roles.contains(&Role::Trapper) && roles.contains(&Role::Fisher),
        "{roles:?}"
    );
    assert!(!roles.contains(&Role::Idle), "{roles:?}");
    let town = sim.view().settlement.expect("founded");
    assert_eq!(town.pursuit, Some(Advance::Smoking));
    assert_eq!(town.sites.len(), 1);
    assert_eq!(town.sites[0].kind, StructureKind::Hut);
    assert!(noted(&sim, "The site of a hut is marked out."));

    // A handful of people have nobody to spare for thinking, but get somewhere all the
    // same.
    pass_days(&mut sim, 30);
    let town = sim.view().settlement.expect("founded");
    assert!(town.housing >= 4, "houses for {}", town.housing);
    assert!(town.food_days >= 60, "food for {} days", town.food_days);
    let insight = sim.state.settlement.as_ref().expect("founded").insight;
    assert!(insight >= 90, "{insight}");
}

#[test]
fn leader_who_holds_something_firmly_makes_it_law_in_time() {
    let (mut sim, members) = settled(&[20, 20]);
    person_mut(&mut sim, members[0]).views = [0, 0, 0, -2];
    lead(&mut sim, members[0]);
    let year_start = (sim.state.time.as_seconds() / SECONDS_PER_DAY / 1_461) * 1_461;
    sim.state.time = Timestamp::from_seconds((year_start + 90) * SECONDS_PER_DAY);

    pass_days(&mut sim, 15);

    assert!(sim.lives_by(Law::ClosedGates));
    assert!(noted(&sim, "Eero0 makes closed gates the law."));
}

#[test]
fn new_leader_is_chosen_when_the_old_one_is_gone_and_at_the_spring_moot() {
    let (mut sim, members) = settled(&[20, 20, 20]);
    person_mut(&mut sim, members[0]).esteem = 70;
    person_mut(&mut sim, members[1]).esteem = 50;
    lead(&mut sim, members[0]);
    let leader = |sim: &Simulation| sim.state.settlement.as_ref().and_then(|s| s.leader);

    sim.person_dies(members[0], "old age");
    pass_days(&mut sim, 1);
    assert_eq!(leader(&sim), Some(members[1]));
    assert!(noted(
        &sim,
        "The moot chooses Eero1 Koski as leader of Lordstead"
    ));

    // The next moot is in the spring of the year after.
    let due = politics(&sim).next_moot.expect("a moot is due");
    assert!((1..=366).contains(&due), "in {due} days");
    for id in sim.members() {
        person_mut(&mut sim, id).regard = 95;
    }
    pass_days(&mut sim, due - 1);
    assert_eq!(leader(&sim), Some(members[1]));
    pass_days(&mut sim, 2);
    assert_eq!(leader(&sim), sim.state.world.protagonist());
}

#[test]
fn pots_of_clay_and_axes_of_iron_take_knowing_how_and_a_place_to_make_them() {
    let (mut sim, _) = settled(&[20]);
    carry(&mut sim, Item::Clay, 2);
    carry(&mut sim, Item::Stick, 4);

    sim.apply(Command::Craft(Recipe::ClayPot));
    assert_eq!(
        last_note(&sim),
        "To fire a clay pot you still need your people to know pottery, a kiln within reach."
    );

    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement
        .advances
        .extend([Advance::Pottery, Advance::Ironworking]);
    put_near(&mut sim, StructureKind::Kiln);
    sim.apply(Command::Craft(Recipe::ClayPot));
    run_until_idle(&mut sim);
    assert_eq!(sim.state.player.carried().count(Item::ClayPot), 1);
    assert_eq!(sim.empty_pots(), 3);

    put_near(&mut sim, StructureKind::Smithy);
    carry(&mut sim, Item::BogOre, 4);
    carry(&mut sim, Item::Log, 2);
    carry(&mut sim, Item::Stick, 1);
    sim.apply(Command::Craft(Recipe::IronAxe));
    run_until_idle(&mut sim);
    assert!(sim.state.player.carried().has(Item::IronAxe));
    assert!(sim.has_axe());
}

#[test]
fn hamlet_of_a_generated_life_gets_through_its_first_year_by_itself() {
    let mut sim = Simulation::generate(Name::default(), Sex::Female, 5);
    let place = sim
        .view()
        .settlement
        .expect("there is a hamlet")
        .name
        .clone();
    assert!(!sim.leads());
    assert_eq!(politics(&sim).era, Era::Band);
    assert_eq!(politics(&sim).voices, 6);

    for _ in 0..365 {
        sim.state.time = Timestamp::from_seconds(sim.state.time.as_seconds() + SECONDS_PER_DAY);
        sim.conditions = sim.state.climate.conditions(sim.state.time);
        sim.live_a_day();
    }
    sim.refresh_society();

    let town = sim.view().settlement.expect("the hamlet is still there");
    assert_eq!(town.name, place);
    assert!(town.people >= 6, "{} people are left", town.people);
    assert!(
        town.food_days >= 5,
        "{} days of food are left",
        town.food_days
    );
    assert!(!noted(&sim, "has died of hunger"));
    assert!(town.politics.leader.is_some());
}

fn know(sim: &mut Simulation, advances: &[Advance]) {
    let settlement = sim.state.settlement.as_mut().expect("founded");
    settlement.advances.extend(advances);
    sim.look();
}

fn built(sim: &Simulation, kind: StructureKind) -> crate::world::ThingId {
    let found = sim
        .structures()
        .into_iter()
        .find(|(_, built)| *built == kind);
    found.expect("it stands").0
}

#[test]
fn bronze_is_smelted_and_cast_by_hand_at_a_smithy() {
    let (mut sim, _) = settled(&[20]);
    know(
        &mut sim,
        &[Advance::Charcoal, Advance::Mining, Advance::Bronze],
    );
    carry(&mut sim, Item::CopperOre, 2);
    carry(&mut sim, Item::TinOre, 1);
    carry(&mut sim, Item::Charcoal, 2);
    carry(&mut sim, Item::Stick, 1);

    sim.apply(Command::Craft(Recipe::SmeltBronze));
    assert_eq!(
        last_note(&sim),
        "To smelt bronze you still need a smithy within reach."
    );

    put_near(&mut sim, StructureKind::Smithy);
    sim.apply(Command::Craft(Recipe::SmeltBronze));
    run_until_idle(&mut sim);
    assert_eq!(sim.state.player.carried().count(Item::Bronze), 1);
    assert!(!sim.has_axe());

    sim.apply(Command::Craft(Recipe::BronzeAxe));
    run_until_idle(&mut sim);
    assert!(sim.state.player.carried().has(Item::BronzeAxe));
    assert!(sim.has_axe());
}

#[test]
fn shift_at_a_workplace_adds_to_the_store_and_is_paid_once_there_is_coin() {
    let (mut sim, _) = settled(&[20]);
    know(&mut sim, &[Advance::Masonry]);
    put_near(&mut sim, StructureKind::Quarry);
    let quarry = crate::Subject::Thing(built(&sim, StructureKind::Quarry));
    let shift = Command::Act {
        verb: Verb::Labor,
        subject: quarry,
    };

    sim.apply(shift.clone());
    assert_eq!(sim.view().work.map(|work| work.needed), Some(4 * 3_600));
    run_until_idle(&mut sim);
    // Half of what a quarrier breaks in a day.
    assert_eq!(in_store(&sim, Item::Stone), 5);
    assert_eq!(politics(&sim).credit, 6_000);
    assert!(noted(&sim, "You work a shift as a quarrier."));

    know(&mut sim, &[Advance::Coinage]);
    put_up(&mut sim, StructureKind::Market);
    sim.state.settlement.as_mut().expect("founded").treasury = 100;
    sim.apply(shift);
    run_until_idle(&mut sim);
    assert_eq!(sim.state.purse, 2);
    assert!(noted(
        &sim,
        "You work a shift as a quarrier, and are paid 2 coins."
    ));

    let hut = StructureKind::Hut;
    put_near(&mut sim, hut);
    assert!(!Verb::for_kind(crate::world::Kind::Structure(hut)).contains(&Verb::Labor));
}

#[test]
fn eldest_grown_child_leads_after_a_leader_where_that_is_their_birthright() {
    let (mut sim, members) = settled(&[20, 20, 20]);
    person_mut(&mut sim, members[0]).esteem = 70;
    person_mut(&mut sim, members[2]).esteem = 90;
    person_mut(&mut sim, members[1]).parents = [Some(members[0]), None];
    lead(&mut sim, members[0]);
    sim.state
        .settlement
        .as_mut()
        .expect("founded")
        .laws
        .push(Law::Birthright);

    sim.person_dies(members[0], "old age");
    pass_days(&mut sim, 1);

    let leader = sim.state.settlement.as_ref().and_then(|s| s.leader);
    assert_eq!(leader, Some(members[1]));
    assert!(noted(
        &sim,
        "By birthright, Eero1 Koski leads Lordstead after their parent."
    ));
}

#[test]
fn assembly_takes_two_thirds_for_a_law() {
    let (mut sim, members) = settled(&[10; 8]);
    for id in &members[..5] {
        person_mut(&mut sim, *id).views = [0, 0, 0, 2];
    }
    for id in &members[5..] {
        person_mut(&mut sim, *id).views = [0, 0, 0, -2];
    }
    sim.state
        .settlement
        .as_mut()
        .expect("founded")
        .laws
        .push(Law::Assembly);
    sim.refresh_society();
    assert_eq!(politics(&sim).needed, 6);

    sim.apply(Command::Legislate(Law::OpenGates, true));
    assert!(!sim.lives_by(Law::OpenGates));
    assert_eq!(
        last_note(&sim),
        "The moot will not have open gates: 5 of 8 voices for it."
    );

    person_mut(&mut sim, members[5]).views = [0, 0, 0, 1];
    sim.apply(Command::Legislate(Law::OpenGates, true));
    assert!(sim.lives_by(Law::OpenGates));

    // A law that takes knowing more than is known cannot be thought of.
    sim.apply(Command::Legislate(Law::Tariffs, true));
    assert!(!sim.lives_by(Law::Tariffs));
    assert_eq!(
        last_note(&sim),
        "Nobody can make sense of such a law before coinage is known."
    );
}

#[test]
fn newspaper_and_broadcast_cost_coin_and_reach_those_who_read_and_everyone() {
    let (mut sim, members) = settled(&[20, 20]);
    person_mut(&mut sim, members[0]).views = [2, 2, 2, 2];
    for issue in Issue::ALL {
        sim.apply(Command::SetView(issue, 2));
    }
    sim.state.settlement.as_mut().expect("founded").literacy = 100;

    sim.apply(Command::Appeal(Appeal::Newspaper));
    assert_eq!(last_note(&sim), "That costs 30 coins, and you have 0.");
    sim.state.purse = 200;
    sim.apply(Command::Appeal(Appeal::Newspaper));
    assert_eq!(
        last_note(&sim),
        "There is no printing house to print a newspaper."
    );

    put_up(&mut sim, StructureKind::PrintingHouse);
    sim.apply(Command::Appeal(Appeal::Newspaper));
    run_until_idle(&mut sim);
    assert_eq!(sim.state.purse, 170);
    assert_eq!(regard(&sim, members[0]), 24);
    assert_eq!(regard(&sim, members[1]), 22);
    assert_eq!(politics(&sim).appeal_rest[Appeal::Newspaper as usize], 7);

    put_up(&mut sim, StructureKind::BroadcastingHouse);
    sim.apply(Command::Appeal(Appeal::Broadcast));
    run_until_idle(&mut sim);
    assert_eq!(sim.state.purse, 70);
    assert_eq!(regard(&sim, members[1]), 24);
    know(&mut sim, &[Advance::Television]);
    sim.state
        .settlement
        .as_mut()
        .expect("founded")
        .appealed_on
        .clear();
    sim.state.purse = 100;
    sim.apply(Command::Appeal(Appeal::Broadcast));
    run_until_idle(&mut sim);
    assert_eq!(regard(&sim, members[1]), 28);
}

#[test]
fn bigger_house_goes_up_in_the_place_of_the_smallest_once_a_place_is_built_up() {
    let (mut sim, _) = settled(&[20]);
    for _ in 0..100 {
        put_up(&mut sim, StructureKind::Hut);
    }
    stock(&mut sim, Item::Log, 40);
    stock(&mut sim, Item::SpruceBough, 40);
    stock(&mut sim, Item::BirchBark, 20);
    let (standing, housing) = (sim.structures().len(), sim.housing());

    sim.apply(Command::Plan(StructureKind::Longhouse));

    assert_eq!(sim.structures().len(), standing);
    assert_eq!(sim.housing(), housing - 4);
    let town = sim.view().settlement.expect("founded");
    assert_eq!(town.sites.len(), 1);
    assert_eq!(town.sites[0].kind, StructureKind::Longhouse);
}

#[test]
fn life_started_alone_has_nobody_and_nothing_until_a_cairn_is_raised() {
    let name = Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned());
    let mut sim = Simulation::set_out(name, Sex::Female, 7, crate::state::Start::Alone);
    assert_eq!(sim.view().settlement, None);
    assert!(sim.view().people.is_empty());
    assert_eq!(sim.era(), Era::Band);

    carry(&mut sim, Item::Stone, 10);
    sim.apply(Command::Craft(Recipe::Cairn));
    run_until_idle(&mut sim);

    let town = sim.view().settlement.expect("founded");
    assert_eq!(town.name, "Lordstead");
    assert_eq!(town.people, 1);
    assert!(town.politics.leads);
    assert_eq!(town.age, crate::settlement::Age::Stone);
    assert!(!town.coined && !town.monetized);
}

#[test]
fn what_stands_built_is_kept_up_out_of_the_treasury_before_anyone_is_paid() {
    let (mut sim, _) = settled(&[20]);
    know(&mut sim, &[Advance::Coinage, Advance::Law]);
    put_up(&mut sim, StructureKind::Market);
    put_up(&mut sim, StructureKind::Courthouse);
    let ledger = |sim: &Simulation| sim.state.settlement.as_ref().expect("founded").ledger;

    // A market and a courthouse take 2 coins a day to keep up between them, and whoever
    // leads is paid 10.
    sim.state.settlement.as_mut().expect("founded").treasury = 5;
    sim.work_day(&sim.members());
    assert_eq!((ledger(&sim).upkeep, ledger(&sim).wages), (2, 3));
    assert!(ledger(&sim).unpaid);
    assert_eq!(sim.state.purse, 0);

    sim.state.settlement.as_mut().expect("founded").treasury = 500;
    sim.work_day(&sim.members());
    assert_eq!((ledger(&sim).upkeep, ledger(&sim).wages), (2, 10));
    assert!(!ledger(&sim).unpaid);
    assert_eq!(sim.state.purse, 10);

    sim.apply(Command::SetTax(80));
    assert_eq!(sim.view().settlement.expect("founded").tax, 50);
}

#[test]
fn offices_do_more_good_under_those_who_suit_them() {
    use crate::people::Trait;
    use crate::politics::Office;

    let (mut sim, members) = settled(&[20, 20, 20]);
    person_mut(&mut sim, members[0]).traits = vec![Trait::Bold];
    person_mut(&mut sim, members[1]).traits = vec![Trait::Kind];
    assert_eq!(sim.office(Office::Marshal), 0);
    let esteem = person_mut(&mut sim, members[0]).esteem;

    sim.apply(Command::Appoint(Office::Marshal, Some(members[0])));
    assert_eq!(sim.office(Office::Marshal), 2);
    assert_eq!(sim.defence(), 4);
    assert_eq!(regard(&sim, members[0]), 30);
    assert_eq!(person_mut(&mut sim, members[0]).esteem, esteem + 5);
    assert!(noted(&sim, "Eero0 Koski is made marshal."));

    // Nobody holds two offices, and an office is done no better by the wrong sort.
    sim.apply(Command::Appoint(Office::Almoner, Some(members[0])));
    assert_eq!(sim.office(Office::Marshal), 0);
    assert_eq!(sim.office(Office::Almoner), 1);
    sim.apply(Command::Appoint(Office::Almoner, Some(members[1])));
    assert_eq!(sim.office(Office::Almoner), 2);

    let view = politics(&sim);
    assert_eq!(view.offices.len(), 4);
    assert_eq!(view.offices[3].1, Some(("Eero1 Koski".to_owned(), true)));
    assert_eq!(view.trusted.len(), 3);

    sim.person_dies(members[1], "old age");
    assert_eq!(sim.office(Office::Almoner), 0);

    lead(&mut sim, members[2]);
    sim.apply(Command::Appoint(Office::Treasurer, Some(members[0])));
    assert!(last_note(&sim).starts_with("You have no say in that."));
}
