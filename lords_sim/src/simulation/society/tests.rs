use super::*;
use crate::body::{Body, Cause};
use crate::craft::Recipe;
use crate::people::Bond;
use crate::simulation::Command;
use crate::state::State;
use crate::world::{Area, AreaKind, Kind, World};

pub(crate) const DAY: u64 = SECONDS_PER_DAY;

/// Long enough after the epoch for there to be grown-ups born after it.
pub(crate) const NOW: Timestamp = Timestamp::from_seconds(50 * 365 * DAY);

pub(crate) fn simulation(world: World) -> Simulation {
    let name = Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned());
    let mut state = State {
        world,
        time: NOW,
        ..State::new(name, Sex::Female)
    };
    state.player = Human::new(
        Location::default(),
        Timestamp::from_seconds(25 * 365 * DAY),
        state.player.name().clone(),
        Sex::Female,
    );

    Simulation::from_state(state)
}

pub(crate) fn person(
    sim: &mut Simulation,
    first: &str,
    sex: Sex,
    age: u64,
    x: i64,
    y: i64,
) -> PersonId {
    let born = NOW.as_seconds() - (age * 365 + age / 4 + 2) * DAY;
    let name = Name::new(first.to_owned(), Vec::new(), "Koski".to_owned());
    let id = sim.state.world.add_person(Person::new(
        name,
        sex,
        Timestamp::from_seconds(born),
        Location::new(x, y),
    ));
    sim.look();
    sim.refresh_society();
    id
}

/// Adds someone who lives with the player and does `role`.
pub(crate) fn member(
    sim: &mut Simulation,
    first: &str,
    sex: Sex,
    age: u64,
    role: Role,
) -> PersonId {
    let id = person(sim, first, sex, age, 0, 900);
    let record = sim.state.world.person_mut(id).expect("just added");
    record.member = true;
    record.role = role;
    record.regard = 50;
    sim.refresh_society();
    id
}

pub(crate) fn carry(sim: &mut Simulation, item: Item, count: u32) {
    sim.state.player.carried_mut().add(item, count);
}

pub(crate) fn run_until_idle(sim: &mut Simulation) {
    let mut ticks = 0;
    while sim.state.player.activity().is_some() {
        assert!(ticks < 200_000, "the player never came to rest");
        sim.tick();
        ticks += 1;
    }
}

pub(crate) fn act(sim: &mut Simulation, verb: Verb, id: PersonId) {
    sim.apply(Command::Act {
        verb,
        subject: Subject::Person(id),
    });
    run_until_idle(sim);
}

pub(crate) fn last_note(sim: &Simulation) -> &str {
    let entries = sim.state.journal.entries();
    entries.last().map_or("", |entry| entry.text.as_str())
}

pub(crate) fn noted(sim: &Simulation, text: &str) -> bool {
    let entries = sim.state.journal.entries();
    entries.iter().any(|entry| entry.text.contains(text))
}

/// Founds a settlement where the player stands, and returns its cairn.
pub(crate) fn settle(sim: &mut Simulation) -> ThingId {
    carry(sim, Item::Stone, 10);
    sim.apply(Command::Craft(Recipe::Cairn));
    run_until_idle(sim);
    sim.state.settlement.as_ref().expect("founded").cairn
}

pub(crate) fn stock(sim: &mut Simulation, item: Item, count: u32) {
    sim.stock(item, count);
    sim.look();
    sim.refresh_society();
}

pub(crate) fn in_store(sim: &Simulation, item: Item) -> u32 {
    sim.store().map_or(0, |store| store.count(item))
}

/// Lets `days` days pass for the others, without the player living through them.
pub(crate) fn pass_days(sim: &mut Simulation, days: u64) {
    for _ in 0..days {
        sim.state.time = Timestamp::from_seconds(sim.state.time.as_seconds() + DAY);
        sim.live_a_day();
    }
    sim.look();
    sim.refresh_society();
}

pub(crate) fn put_up(sim: &mut Simulation, kind: StructureKind) {
    let at = sim
        .free_plot(false)
        .unwrap_or_else(|| Location::new(0, 700));
    let structure = Structure::new(Object::new(at, sim.state.time), kind);
    sim.add_thing(Thing::Structure(structure));
    sim.look();
}

#[test]
pub(crate) fn player_character_is_one_of_the_people_but_not_among_the_others() {
    let sim = simulation(World::default());
    let world = &sim.state.world;

    let me = world
        .protagonist()
        .expect("the player is one of the people");
    assert_eq!(
        world.person(me).map(|me| me.name.full()),
        Some("Ada Lord".to_owned())
    );
    assert_eq!(world.people().count(), 0);
    assert!(sim.view().people.is_empty());
    assert_eq!(
        Simulation::from_state(sim.state().clone()).state(),
        sim.state()
    );
}

#[test]
pub(crate) fn someone_in_sight_is_seen_and_on_the_map() {
    let mut sim = simulation(World::default());
    let far = person(&mut sim, "Eero", Sex::Male, 30, 0, 20_000);
    let near = person(&mut sim, "Aino", Sex::Female, 30, 0, 900);
    let view = sim.view();

    assert_eq!(view.surroundings.people.len(), 1);
    assert_eq!(view.surroundings.people[0].subject, Subject::Person(far));
    assert_eq!(
        view.surroundings.people[0].kind,
        Kind::Person(Bond::Stranger)
    );
    assert_eq!(view.surroundings.within_reach.len(), 1);
    assert_eq!(
        view.surroundings.within_reach[0].subject,
        Subject::Person(near)
    );
    assert_eq!(view.map.people.len(), 2);
    assert_eq!(view.people.len(), 2);
    assert!(!view.people[0].met);
    assert_eq!(view.person(Subject::Person(far)).map(|p| p.age), Some(30));
}

#[test]
pub(crate) fn stranger_talked_with_and_given_food_becomes_a_friend_who_stays() {
    let mut sim = simulation(World::default());
    let eero = person(&mut sim, "Eero", Sex::Male, 30, 0, 20_000);
    carry(&mut sim, Item::RyeBread, 3);

    act(&mut sim, Verb::Invite, eero);
    assert_eq!(last_note(&sim), "That cannot be done to a stranger.");

    act(&mut sim, Verb::Talk, eero);
    assert!(noted(&sim, "You meet Eero Koski, who is 30."));
    assert!(sim.is_within_reach(Subject::Person(eero)));
    act(&mut sim, Verb::Talk, eero);
    assert_eq!(sim.state.world.person(eero).map(|p| p.regard), Some(7));

    for _ in 0..3 {
        act(&mut sim, Verb::Gift, eero);
    }
    assert_eq!(sim.state.world.bond(eero), Some(Bond::Friend));
    assert!(noted(&sim, "You give Eero rye bread."));
    act(&mut sim, Verb::Gift, eero);
    assert_eq!(last_note(&sim), "You have no food to give.");

    act(&mut sim, Verb::Invite, eero);
    assert_eq!(last_note(&sim), "Eero will stay with you from now on.");
    assert_eq!(sim.state.world.bond(eero), Some(Bond::Companion));
    assert!(sim.view().people[0].member);
}

#[test]
pub(crate) fn companion_without_a_home_keeps_by_the_player() {
    let mut sim = simulation(World::default());
    let eero = member(&mut sim, "Eero", Sex::Male, 30, Role::Forager);
    sim.state
        .player
        .animal_mut()
        .move_to(Location::new(50_000, 0));

    sim.move_people();

    let at = sim.state.world.person(eero).expect("still there").location;
    assert!(at.distance_to(Location::new(50_000, 0)) <= Length::from_meters(2));
}

#[test]
pub(crate) fn partners_are_grown_up_fond_and_not_kin() {
    let mut sim = simulation(World::default());
    let me = sim.state.world.protagonist();
    let eero = person(&mut sim, "Eero", Sex::Male, 30, 0, 900);
    let boy = person(&mut sim, "Otso", Sex::Male, 14, 900, 0);
    let son = person(&mut sim, "Kalle", Sex::Male, 20, -900, 0);
    for id in [eero, boy, son] {
        sim.state.world.person_mut(id).expect("there").regard = 80;
    }
    sim.state.world.person_mut(son).expect("there").parents = [me, None];
    sim.state.world.person_mut(eero).expect("there").regard = 40;
    sim.look();

    act(&mut sim, Verb::Court, eero);
    assert_eq!(last_note(&sim), "Eero is not that fond of you yet.");
    act(&mut sim, Verb::Court, boy);
    assert_eq!(last_note(&sim), "That is for grown-ups.");
    assert_eq!(sim.state.world.bond(son), Some(Bond::Child));

    sim.state.world.person_mut(eero).expect("there").regard = 60;
    sim.look();
    act(&mut sim, Verb::Court, eero);
    assert_eq!(sim.state.world.bond(eero), Some(Bond::Partner));
    assert!(sim.state.world.person(eero).is_some_and(|eero| eero.member));

    act(&mut sim, Verb::TryForChild, boy);
    assert_eq!(last_note(&sim), "That cannot be done to a friend.");
}

#[test]
pub(crate) fn child_is_conceived_under_a_roof_and_born_in_time() {
    let mut sim = simulation(World::default());
    let me = sim.state.world.protagonist().expect("the player");
    let eero = person(&mut sim, "Eero", Sex::Male, 30, 0, 900);
    sim.state.world.person_mut(eero).expect("there").regard = 90;
    sim.look();
    act(&mut sim, Verb::Court, eero);

    act(&mut sim, Verb::TryForChild, eero);
    assert_eq!(last_note(&sim), "You need a roof over your heads for that.");

    put_up(&mut sim, StructureKind::LeanTo);
    let expecting = |sim: &Simulation| sim.state.world.person(me).and_then(|me| me.expecting);
    for _ in 0..40 {
        if expecting(&sim).is_some() {
            break;
        }
        *sim.state.player.body_mut() = Body::default();
        act(&mut sim, Verb::TryForChild, eero);
    }
    let due = expecting(&sim).expect("a child is on the way");
    assert!(noted(&sim, "You are with child."));
    act(&mut sim, Verb::TryForChild, eero);
    assert_eq!(last_note(&sim), "A child is on the way already.");

    sim.state.time = due;
    carry(&mut sim, Item::RyeBread, 10);
    sim.live_a_day();
    sim.look();
    sim.refresh_society();

    let view = sim.view();
    let child = view
        .people
        .iter()
        .find(|person| person.bond == Bond::Child)
        .expect("the child is born");
    assert_eq!(child.age, 0);
    assert!(child.member && !child.works);
    assert!(child.name.ends_with("Koski"));
    assert!(noted(&sim, "You give birth to a"));
    assert_eq!(expecting(&sim), None);
    assert!(
        sim.state.player.carried().count(Item::RyeBread) < 10,
        "the child ate"
    );
}

#[test]
pub(crate) fn settlement_is_founded_once_and_keeps_a_common_store() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::Knife, 1);
    carry(&mut sim, Item::Log, 2);
    carry(&mut sim, Item::RyeBread, 1);

    let cairn = settle(&mut sim);
    assert_eq!(
        sim.view().settlement.map(|town| town.name.clone()),
        Some("Lordstead".to_owned())
    );
    assert!(noted(&sim, "You name the place Lordstead."));
    assert_eq!(sim.at_hand.store, Some(cairn));

    carry(&mut sim, Item::Stone, 10);
    sim.apply(Command::Craft(Recipe::Cairn));
    assert_eq!(last_note(&sim), "You have founded a settlement already.");

    sim.apply(Command::Act {
        verb: Verb::Deposit,
        subject: Subject::Thing(cairn),
    });
    run_until_idle(&mut sim);
    assert_eq!(in_store(&sim, Item::Log), 2);
    assert_eq!(in_store(&sim, Item::Stone), 10);
    assert!(sim.state.player.carried().has(Item::Knife));
    assert_eq!(sim.state.player.carried().count(Item::Log), 0);
    assert_eq!(sim.at_hand.loose.count(Item::Log), 2);

    // What is in the store is at hand: to eat, to take, and to make things of.
    sim.apply(Command::Take(Item::Log, 1));
    assert_eq!(sim.state.player.carried().count(Item::Log), 1);
    assert_eq!(in_store(&sim, Item::Log), 1);
    sim.apply(Command::Consume(Item::RyeBread));
    run_until_idle(&mut sim);
    assert_eq!(in_store(&sim, Item::RyeBread), 0);

    sim.apply(Command::NameSettlement("  Lakeside ".to_owned()));
    assert_eq!(
        sim.view().settlement.map(|town| town.name.clone()),
        Some("Lakeside".to_owned())
    );
}

#[test]
pub(crate) fn building_is_planned_out_of_the_store_and_built_by_builders_and_the_player() {
    let mut sim = simulation(World::default());
    settle(&mut sim);

    sim.apply(Command::Plan(StructureKind::Hut));
    assert_eq!(
        last_note(&sim),
        "The store is short of 12 × log, 20 × spruce bough for that."
    );
    sim.apply(Command::Plan(StructureKind::Well));
    assert_eq!(last_note(&sim), "Nobody knows masonry yet.");

    stock(&mut sim, Item::Log, 12);
    stock(&mut sim, Item::SpruceBough, 20);
    sim.apply(Command::Plan(StructureKind::Hut));
    assert_eq!(in_store(&sim, Item::Log), 0);
    let town = sim.view().settlement.expect("founded");
    assert_eq!(town.sites.len(), 1);
    assert_eq!((town.sites[0].work, town.sites[0].hours), (0, 48));
    assert_eq!(town.housing, 0);

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
    for _ in 0..2 * 3_700 {
        sim.tick();
    }
    sim.apply(Command::Stop);
    assert_eq!(sim.view().settlement.expect("founded").sites[0].work, 2);

    member(&mut sim, "Eero", Sex::Male, 30, Role::Builder);
    pass_days(&mut sim, 5);
    assert_eq!(sim.view().settlement.expect("founded").sites[0].work, 42);
    pass_days(&mut sim, 1);

    let town = sim.view().settlement.expect("founded");
    assert!(town.sites.is_empty());
    assert_eq!(town.buildings, [(StructureKind::Hut, 1)]);
    assert_eq!(town.housing, 4);
    assert!(noted(&sim, "The hut is finished."));
}

#[test]
pub(crate) fn those_who_work_fill_the_store_and_everyone_eats_out_of_it() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    member(&mut sim, "Eero", Sex::Male, 30, Role::Fisher);
    member(&mut sim, "Aino", Sex::Female, 28, Role::Woodcutter);
    member(&mut sim, "Otso", Sex::Male, 5, Role::Forager);

    pass_days(&mut sim, 1);

    // Twelve perch came in. Two grown-ups and a child ate 6,500 kcal of them.
    assert_eq!(in_store(&sim, Item::CookedFish), 0);
    assert_eq!(in_store(&sim, Item::Log), 4);
    assert_eq!(in_store(&sim, Item::Stick), 8);
    assert!(noted(
        &sim,
        "There is not enough food in the store for everyone."
    ));

    sim.apply(Command::Assign(sim.view().people[1].id, Role::Fisher));
    stock(&mut sim, Item::DriedMeat, 100);
    pass_days(&mut sim, 3);
    let town = sim.view().settlement.expect("founded");
    assert!(town.food_days >= 3, "{} days of food", town.food_days);
    assert_eq!(town.people, 4);
    assert!(sim.view().people.iter().all(|person| !person.hungry));
}

#[test]
pub(crate) fn hungry_people_leave_and_in_the_end_starve() {
    let mut sim = simulation(World::default());
    let me = sim.state.world.protagonist();
    settle(&mut sim);
    let eero = member(&mut sim, "Eero", Sex::Male, 30, Role::Idle);
    let child = member(&mut sim, "Otso", Sex::Male, 5, Role::Idle);
    sim.state.world.person_mut(child).expect("there").parents = [me, None];

    pass_days(&mut sim, u64::from(DAYS_TO_LEAVE));
    assert!(sim.state.world.person(eero).is_some_and(|eero| eero.gone));
    assert!(noted(&sim, "Eero Koski has left to find food elsewhere."));
    assert!(
        sim.state
            .world
            .person(child)
            .is_some_and(Person::is_present)
    );

    pass_days(&mut sim, u64::from(DAYS_TO_STARVE - DAYS_TO_LEAVE));
    assert!(
        sim.state
            .world
            .person(child)
            .is_some_and(|child| child.died.is_some())
    );
    assert!(noted(&sim, "Otso Koski has died of hunger."));
    assert!(sim.members().is_empty());
}

#[test]
pub(crate) fn thinkers_work_out_an_advance_which_opens_up_more() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    member(&mut sim, "Eero", Sex::Male, 30, Role::Thinker);
    member(&mut sim, "Aino", Sex::Female, 30, Role::Thinker);
    stock(&mut sim, Item::DriedMeat, 500);

    sim.apply(Command::Pursue(Advance::Masonry));
    assert_eq!(sim.view().settlement.expect("founded").pursuit, None);
    sim.apply(Command::Pursue(Advance::Smoking));
    assert_eq!(last_note(&sim), "Your people set their minds to smoking.");

    pass_days(&mut sim, 6);
    let town = sim.view().settlement.expect("founded");
    assert_eq!((town.pursuit, town.insight), (Some(Advance::Smoking), 15));

    // Two thinkers get 2.5 days further in a day. The rest is skipped over here.
    let left = Advance::Smoking.cost() * 10 - 15 * 10 - 20;
    sim.state.settlement.as_mut().expect("founded").insight += left;
    pass_days(&mut sim, 1);
    let town = sim.view().settlement.expect("founded");
    assert_eq!(town.advances, [Advance::Smoking]);
    assert_eq!(town.pursuit, None);
    assert!(noted(&sim, "Your people have worked out smoking."));

    put_up(&mut sim, StructureKind::Smokehouse);
    sim.apply(Command::Assign(sim.view().people[0].id, Role::Fisher));
    pass_days(&mut sim, 1);
    assert_eq!(in_store(&sim, Item::DriedFish), 24);
}

#[test]
pub(crate) fn rye_grows_through_the_summer_and_comes_in_at_harvest() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    for name in ["Eero", "Aino", "Otso", "Ilta"] {
        member(&mut sim, name, Sex::Male, 30, Role::Farmer);
    }
    stock(&mut sim, Item::DriedMeat, 2_000);
    put_up(&mut sim, StructureKind::Field);
    // The first of September of some year.
    let year_start = (sim.state.time.as_seconds() / DAY / 1_461) * 1_461 * DAY;
    sim.state.time = Timestamp::from_seconds(year_start + 243 * DAY);

    pass_days(&mut sim, 5);
    // A field takes three farmers, however many there are.
    assert_eq!(sim.view().settlement.expect("founded").growing, 5 * 3 * 30);

    pass_days(&mut sim, 30);
    let town = sim.view().settlement.expect("founded");
    assert_eq!(town.growing, 0);
    // Some of it has been eaten since.
    assert!(
        in_store(&sim, Item::Grain) > 400,
        "{}",
        in_store(&sim, Item::Grain)
    );
    assert!(noted(&sim, "The rye is brought in"));
}

#[test]
pub(crate) fn crafters_clothe_people_out_of_pelts() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    let crafter = member(&mut sim, "Aino", Sex::Female, 30, Role::Crafter);
    stock(&mut sim, Item::DriedMeat, 100);
    stock(&mut sim, Item::HarePelt, 4);
    stock(&mut sim, Item::SpruceRoot, 6);

    pass_days(&mut sim, 1);
    assert!(
        sim.state
            .world
            .person(crafter)
            .is_some_and(|crafter| crafter.clothed)
    );
    assert_eq!(in_store(&sim, Item::HarePelt), 0);

    pass_days(&mut sim, 1);
    assert_eq!(in_store(&sim, Item::Cordage), 3);
}

#[test]
pub(crate) fn frost_takes_those_without_a_fire_or_a_roof() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    for index in 0..40 {
        member(&mut sim, &format!("Eero{index}"), Sex::Male, 30, Role::Idle);
    }
    stock(&mut sim, Item::DriedMeat, 20_000);
    sim.conditions.temperature = crate::climate::Temperature::from_degrees(-20);

    for _ in 0..30 {
        sim.state.time = Timestamp::from_seconds(sim.state.time.as_seconds() + DAY);
        let members = sim.members();
        sim.winter(&members);
    }

    let left = sim.members().len();
    assert!((10..36).contains(&left), "{left} of 40 are left");
    assert!(noted(&sim, "has died of the cold."));
    assert!(noted(&sim, "There is nothing in the store to heat"));
}

#[test]
pub(crate) fn people_of_a_settlement_pair_up_and_have_children() {
    // Sickness may take either of them first, so more than one life is given a try.
    let lived = |seed: u64| {
        let mut sim = simulation(World::default());
        sim.state.rng = crate::rng::Rng::new(seed);
        settle(&mut sim);
        put_up(&mut sim, StructureKind::Longhouse);
        let aino = member(&mut sim, "Aino", Sex::Female, 25, Role::Idle);
        let eero = member(&mut sim, "Eero", Sex::Male, 27, Role::Idle);
        stock(&mut sim, Item::DriedMeat, 100_000);

        for _ in 0..500 {
            let born = sim
                .state
                .world
                .everyone()
                .any(|(_, person)| person.parents[0] == Some(aino));
            if born {
                return Some((sim, aino, eero));
            }
            pass_days(&mut sim, 10);
        }
        None
    };
    let (sim, aino, eero) = (1..=6)
        .find_map(lived)
        .expect("a child was born in one life out of six");
    let child_of = |sim: &Simulation| {
        let mut everyone = sim.state.world.everyone();
        everyone
            .find(|(_, person)| person.parents[0] == Some(aino))
            .map(|(_, child)| child.clone())
    };

    let child = child_of(&sim).expect("the child was born");
    assert!(noted(&sim, "have become partners."));
    assert!(noted(&sim, "is born to Aino Koski."));
    assert!(child.parents[1].is_some() && child.member);
    let _ = eero;
}

#[test]
pub(crate) fn wanderers_come_and_go_and_are_given_away_by_their_smoke() {
    let mut sim = simulation(World::default());

    pass_days(&mut sim, 300);

    let came = sim.state.world.everyone().count() - 1;
    let here = sim.state.world.people().count();
    assert!(came >= 3, "{came} wanderers came in three hundred days");
    assert!(here <= MOST_WANDERERS && here < came);
    assert!(noted(&sim, "Smoke rises to the"));
    for (_, wanderer) in sim.state.world.people() {
        let away = wanderer.location.distance_to(Location::default());
        assert!(away >= Length::from_meters(70) && away <= Length::from_meters(160));
    }
}

#[test]
pub(crate) fn wanderers_are_drawn_to_a_settlement_that_does_well() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    put_up(&mut sim, StructureKind::Longhouse);
    for name in ["Eero", "Aino", "Otso"] {
        member(&mut sim, name, Sex::Male, 30, Role::Idle);
    }
    stock(&mut sim, Item::DriedMeat, 100_000);

    pass_days(&mut sim, 400);

    assert!(sim.members().len() > 3);
    assert!(noted(&sim, "has come to Lordstead asking to stay"));
}

#[test]
pub(crate) fn life_goes_on_as_someone_who_lived_with_the_dead() {
    let mut sim = simulation(World::default());
    let me = sim.state.world.protagonist().expect("the player");
    carry(&mut sim, Item::Knife, 1);
    carry(&mut sim, Item::Stick, 3);
    let daughter = member(&mut sim, "Tuuli", Sex::Female, 17, Role::Forager);
    let toddler = member(&mut sim, "Otso", Sex::Male, 3, Role::Idle);
    let stranger = person(&mut sim, "Eero", Sex::Male, 30, 0, 30_000);
    sim.state.world.person_mut(daughter).expect("there").parents = [Some(me), None];
    sim.refresh_society();
    let heirs: Vec<PersonId> = sim.view().heirs().map(|heir| heir.id).collect();
    assert_eq!(heirs, [daughter]);

    sim.apply(Command::LiveOn(daughter));
    assert_eq!(sim.state.player.name().first(), "Ada");

    sim.die(Cause::OldAge);
    assert!(
        sim.state
            .world
            .person(me)
            .is_some_and(|me| me.died.is_some())
    );
    sim.apply(Command::LiveOn(toddler));
    sim.apply(Command::LiveOn(stranger));
    assert!(sim.state.death.is_some());

    sim.apply(Command::LiveOn(daughter));

    assert_eq!(sim.state.death, None);
    assert_eq!(sim.state.player.name().full(), "Tuuli Koski");
    assert_eq!(sim.state.player.sex(), Sex::Female);
    assert_eq!(sim.state.world.protagonist(), Some(daughter));
    assert_eq!(sim.state.past_lives, 1);
    assert_eq!(sim.view().past_lives, 1);
    assert!(sim.state.player.carried().has(Item::Knife));
    assert_eq!(sim.at_hand.loose.count(Item::Stick), 3);
    assert_eq!(
        last_note(&sim),
        "Life goes on. You are Tuuli Koski, child of Ada, 17 years old."
    );
    assert!(sim.view().people.iter().all(|person| person.id != daughter));
    assert!(sim.state.player.body().is_alive());

    let before = sim.state.time;
    sim.tick();
    assert!(sim.state.time > before);
}

#[test]
pub(crate) fn routine_keeps_a_body_fed_watered_and_rested() {
    let mut world = World::default();
    let corners = [
        (-20_000, 8_000),
        (20_000, 8_000),
        (20_000, 40_000),
        (-20_000, 40_000),
    ];
    world.add_area(Area::new(
        AreaKind::Lake,
        corners
            .into_iter()
            .map(|(x, y)| Location::new(x, y))
            .collect(),
    ));
    let mut sim = simulation(world);
    carry(&mut sim, Item::DriedMeat, 30);

    sim.apply(Command::SetRoutine(true));
    assert!(sim.view().routine);
    for _ in 0..4 * DAY {
        sim.tick();
    }

    let body = sim.state.player.body();
    assert!(sim.state.death.is_none());
    assert_eq!(body.health_percent(), 100);
    assert!(body.ailments().iter().all(|ailment| !ailment.is_grave()));
    assert!(noted(&sim, "You drink your fill from the lake."));
    assert!(noted(&sim, "You eat the dried meat."));
    assert!(noted(&sim, "You wake up rested."));
    assert!(sim.state.player.carried().count(Item::DriedMeat) < 20);
}

#[test]
pub(crate) fn well_gives_clean_water_whatever_the_weather() {
    let mut sim = simulation(World::default());
    put_up(&mut sim, StructureKind::Well);
    sim.conditions.frozen = true;
    carry(&mut sim, Item::BarkPot, 1);
    let well = sim
        .surroundings
        .within_reach
        .iter()
        .find(|sighting| sighting.kind == Kind::Structure(StructureKind::Well))
        .map(|sighting| sighting.subject)
        .expect("the well is within reach");

    for verb in [Verb::Drink, Verb::FillWater] {
        sim.apply(Command::Act {
            verb,
            subject: well,
        });
        run_until_idle(&mut sim);
    }

    assert!(sim.state.player.body().is_quenched());
    assert_eq!(sim.state.player.carried().count(Item::BoiledWater), 1);
    assert!(sim.at_hand.water);
}

#[test]
pub(crate) fn state_with_people_and_a_settlement_reads_back_as_it_was_written() {
    let mut sim = simulation(World::default());
    settle(&mut sim);
    member(&mut sim, "Eero", Sex::Male, 30, Role::Thinker);
    sim.apply(Command::Pursue(Advance::Pottery));
    pass_days(&mut sim, 2);

    let written = ron::to_string(sim.state()).expect("the state encodes");
    let read: State = ron::from_str(&written).expect("the state decodes");

    assert_eq!(&read, sim.state());
    assert_eq!(
        Simulation::from_state(read).view().people,
        sim.view().people
    );
}
