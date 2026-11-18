use super::*;
use crate::body::{Ailment, Body};
use crate::climate::{Climate, Light};
use crate::object::{Name, Sex};
use crate::simulation::Command;
use crate::space::Length;
use crate::state::{FIRST_MORNING, State};
use crate::world::{Area, AreaKind, Plant, World};

const HOUR: u64 = SECONDS_PER_HOUR;
const DAY: u64 = SECONDS_PER_DAY;

fn tree(species: PlantSpecies, x: i64, y: i64) -> Thing {
    Thing::Plant(Plant::new(
        Object::new(Location::new(x, y), Timestamp::EPOCH),
        species,
        Length::from_meters(15),
        Length::from_millimeters(300),
    ))
}

fn bilberries(x: i64, y: i64) -> Thing {
    Thing::Plant(Plant::new(
        Object::new(Location::new(x, y), Timestamp::EPOCH),
        PlantSpecies::Bilberry,
        Length::from_millimeters(300),
        Length::from_millimeters(2_000),
    ))
}

fn lake(south: i64, north: i64) -> Area {
    let corners = [
        (-20_000, south),
        (20_000, south),
        (20_000, north),
        (-20_000, north),
    ];
    Area::new(
        AreaKind::Lake,
        corners
            .into_iter()
            .map(|(x, y)| Location::new(x, y))
            .collect(),
    )
}

fn simulation(world: World) -> Simulation {
    Simulation::from_state(State {
        world,
        ..State::new(Name::default(), Sex::Female)
    })
}

/// A simulation of a player in `world` where there is weather, `days` and `hours` after
/// [`FIRST_MORNING`], which is eight in the morning on 1 May.
fn boreal(world: World, days: u64, hours: u64) -> Simulation {
    let start = FIRST_MORNING.as_seconds() + days * DAY + hours * HOUR;

    Simulation::from_state(State {
        world,
        climate: Climate::Boreal { seed: 4 },
        time: Timestamp::from_seconds(start),
        started: Timestamp::from_seconds(start),
        ..State::new(Name::default(), Sex::Female)
    })
}

fn carry(sim: &mut Simulation, item: Item, count: u32) {
    sim.state.player.carried_mut().add(item, count);
}

fn carried(sim: &Simulation, item: Item) -> u32 {
    sim.state.player.carried().count(item)
}

fn run(sim: &mut Simulation, ticks: u64) {
    for _ in 0..ticks {
        sim.tick();
    }
}

/// Ticks until the player is doing nothing, and returns how many ticks that took.
fn run_until_idle(sim: &mut Simulation) -> u64 {
    let mut ticks = 0;
    while sim.state.player.activity().is_some() {
        assert!(ticks < 200_000, "the player never came to rest");
        sim.tick();
        ticks += 1;
    }
    ticks
}

fn last_note(sim: &Simulation) -> &str {
    sim.state
        .journal
        .entries()
        .last()
        .map_or("", |entry| entry.text.as_str())
}

fn noted(sim: &Simulation, text: &str) -> bool {
    let entries = sim.state.journal.entries();
    entries.iter().any(|entry| entry.text.contains(text))
}

/// Puts a fire that burns for `seconds` next to the player.
fn light_fire(sim: &mut Simulation, seconds: u64) -> Subject {
    let now = sim.state.time;
    let at = sim.site().expect("there is dry ground to build on");
    let mut fire = Structure::new(Object::new(at, now), StructureKind::Campfire);
    fire.stoke(seconds, now);
    sim.add_thing(Thing::Structure(fire));
    sim.look();

    within_reach(sim, Kind::Structure(StructureKind::Campfire)).expect("the fire is at hand")
}

fn within_reach(sim: &Simulation, kind: Kind) -> Option<Subject> {
    sim.surroundings
        .within_reach
        .iter()
        .find(|sighting| sighting.kind == kind)
        .map(|sighting| sighting.subject)
}

/// Does `job` over and over, with enough time between to do it in, until `done` or the
/// luck it takes has failed implausibly many times.
fn keep_trying(sim: &mut Simulation, command: &Command, mut done: impl FnMut(&Simulation) -> bool) {
    for _ in 0..40 {
        sim.apply(command.clone());
        run_until_idle(sim);
        if done(sim) {
            return;
        }
    }
    panic!("{command:?} never worked");
}

#[test]
fn sticks_are_gathered_from_a_tree_walked_to_until_it_has_no_more() {
    let mut world = World::default();
    let spruce = Subject::Thing(world.add_thing(tree(PlantSpecies::Spruce, 0, 5_000)));
    let mut sim = simulation(world);
    let job = Job::Act {
        verb: Verb::GatherSticks,
        subject: spruce,
    };

    sim.apply(Command::Act {
        verb: Verb::GatherSticks,
        subject: spruce,
    });
    assert_eq!(sim.view().walk.and_then(|walk| walk.intent), Some(job));

    sim.tick();
    sim.tick();
    sim.tick();
    sim.tick();
    let work = sim.view().work.expect("the player has got to work");
    assert_eq!(work.job, job);
    assert_eq!(work.subject, Some(Kind::Plant(PlantSpecies::Spruce)));
    assert_eq!(work.needed, 120);

    run_until_idle(&mut sim);

    assert_eq!(sim.state.player.location(), Location::new(0, 4_450));
    assert_eq!(carried(&sim, Item::Stick), 12);
    assert_eq!(
        last_note(&sim),
        "There is no dead wood left on this spruce."
    );
    assert_eq!(sim.state.player.skills().experience(Skill::Foraging), 8);
}

#[test]
fn gathering_goes_on_only_until_the_load_is_heavy() {
    let mut world = World::default();
    let spruce = Subject::Thing(world.add_thing(tree(PlantSpecies::Spruce, 0, 500)));
    let mut sim = simulation(world);
    carry(&mut sim, Item::Knife, 1);
    let cut = Command::Act {
        verb: Verb::GatherBoughs,
        subject: spruce,
    };

    sim.apply(cut.clone());
    run_until_idle(&mut sim);

    let load = sim.state.player.carried().weight();
    assert!(load > crate::item::HEAVY_LOAD && load < crate::item::HEAVY_LOAD + 2_000);
    assert_eq!(last_note(&sim), "You stop, as your load is getting heavy.");

    let before = carried(&sim, Item::SpruceBough);
    sim.apply(cut);
    run_until_idle(&mut sim);
    assert_eq!(carried(&sim, Item::SpruceBough), before + 3);
}

#[test]
fn nothing_is_done_without_the_tool_it_takes() {
    let mut world = World::default();
    let birch = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 0, 500)));
    let mut sim = simulation(world);

    sim.apply(Command::Act {
        verb: Verb::Fell,
        subject: birch,
    });
    assert_eq!(sim.state.player.activity(), None);
    assert_eq!(last_note(&sim), "You need a stone axe for that.");

    sim.apply(Command::Act {
        verb: Verb::GatherBoughs,
        subject: birch,
    });
    assert_eq!(last_note(&sim), "That cannot be done to a birch.");
}

#[test]
fn felled_tree_is_a_pile_of_logs_of_which_only_some_can_be_carried() {
    let mut world = World::default();
    let birch = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 0, 5_000)));
    let mut sim = simulation(world);
    carry(&mut sim, Item::StoneAxe, 1);

    sim.apply(Command::Act {
        verb: Verb::Fell,
        subject: birch,
    });
    let ticks = run_until_idle(&mut sim);

    assert!(ticks > 1_800);
    assert_eq!(sim.state.world.kind(birch), None);
    assert_eq!(sim.at_hand.loose.count(Item::Log), 3);
    assert_eq!(sim.at_hand.loose.count(Item::Stick), 11);
    assert!(
        sim.view()
            .map
            .things
            .iter()
            .all(|mark| mark.subject != birch)
    );
    assert_eq!(sim.state.player.skills().experience(Skill::Woodcraft), 30);

    let pile = within_reach(&sim, Kind::Pile(Item::Log)).expect("the logs lie at hand");
    sim.apply(Command::Act {
        verb: Verb::PickUp,
        subject: pile,
    });
    run_until_idle(&mut sim);

    assert_eq!(carried(&sim, Item::Stick), 11);
    assert_eq!(carried(&sim, Item::Log), 2);
    assert_eq!(sim.at_hand.loose.count(Item::Log), 1);
    assert!(sim.state.player.carried().weight() <= crate::item::CARRY_LIMIT);

    sim.apply(Command::Act {
        verb: Verb::PickUp,
        subject: pile,
    });
    assert_eq!(last_note(&sim), "You cannot carry any more.");
}

#[test]
fn bark_is_stripped_off_a_tree_once() {
    let mut world = World::default();
    let birch = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 0, 500)));
    let pine = Subject::Thing(world.add_thing(tree(PlantSpecies::Pine, 500, 0)));
    let mut sim = simulation(world);
    carry(&mut sim, Item::Knife, 1);
    let strip = |subject| Command::Act {
        verb: Verb::StripBark,
        subject,
    };

    sim.apply(strip(birch));
    assert_eq!(run_until_idle(&mut sim), 300);
    sim.apply(strip(pine));
    assert_eq!(run_until_idle(&mut sim), 600);

    assert_eq!(carried(&sim, Item::BirchBark), 3);
    assert_eq!(carried(&sim, Item::InnerBark), 3);

    sim.apply(strip(birch));
    assert_eq!(sim.state.player.activity(), None);
    assert_eq!(
        last_note(&sim),
        "The bark of this birch has been taken already."
    );
}

#[test]
fn campfire_burns_down_and_is_lit_again_and_fed() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::FireSteel, 1);
    carry(&mut sim, Item::BirchBark, 30);
    carry(&mut sim, Item::Stick, 9);
    carry(&mut sim, Item::Log, 1);

    keep_trying(&mut sim, &Command::Craft(Recipe::Campfire), |sim| {
        sim.at_hand.fire
    });
    assert_eq!(carried(&sim, Item::Stick), 4);
    assert!(sim.state.player.skills().experience(Skill::Firemaking) > 0);
    let fire = within_reach(&sim, Kind::Structure(StructureKind::Campfire));
    let fire = fire.expect("the fire burns within reach");

    run(&mut sim, 76 * 60);
    assert!(!sim.at_hand.fire);
    assert_eq!(
        sim.state.world.kind(fire),
        Some(Kind::Structure(StructureKind::Fireplace))
    );
    assert!(noted(&sim, "The fire has burned out."));

    sim.apply(Command::Act {
        verb: Verb::AddLog,
        subject: fire,
    });
    assert_eq!(last_note(&sim), "That cannot be done to a cold fireplace.");

    let light = Command::Act {
        verb: Verb::Light,
        subject: fire,
    };
    keep_trying(&mut sim, &light, |sim| sim.at_hand.fire);
    assert_eq!(carried(&sim, Item::Stick), 1);

    let burning_until = |sim: &Simulation| match fire {
        Subject::Thing(id) => sim.state.world.structure(id).map(Structure::until),
        Subject::Area(_) => None,
    };
    let before = burning_until(&sim).expect("the fire is there");
    sim.apply(Command::Act {
        verb: Verb::AddLog,
        subject: fire,
    });
    run_until_idle(&mut sim);
    let after = burning_until(&sim).expect("the fire is there");

    assert_eq!(after.seconds_since(before), 2 * HOUR);
    assert_eq!(carried(&sim, Item::Log), 0);
    assert_eq!(sim.at_hand.fire_until, Some(after));
}

#[test]
fn fire_is_banked_up_with_logs_before_sticks_and_takes_only_so_much() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::Stick, 40);
    sim.apply(Command::Drop(Item::Stick, 40));
    carry(&mut sim, Item::Log, 2);
    let fire = light_fire(&mut sim, HOUR);
    let bank_up = Command::Act {
        verb: Verb::Stoke,
        subject: fire,
    };

    sim.apply(bank_up.clone());
    assert_eq!(run_until_idle(&mut sim), 60);

    // An hour was left, two logs made it five, and sticks filled it up to twelve.
    assert_eq!(carried(&sim, Item::Log), 0);
    assert_eq!(sim.at_hand.loose.count(Item::Stick), 12);
    assert_eq!(
        last_note(&sim),
        "You bank up the fire. It will burn for about 12 hours."
    );

    sim.apply(bank_up);
    assert_eq!(last_note(&sim), "The fire has all the wood it can take.");
    sim.apply(Command::Act {
        verb: Verb::AddLog,
        subject: fire,
    });
    assert_eq!(last_note(&sim), "You have no log to add.");
}

#[test]
fn food_is_cooked_over_a_fire_for_as_long_as_there_is_any() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::RawMeat, 2);

    sim.apply(Command::Craft(Recipe::CookMeat));
    assert_eq!(sim.state.player.activity(), None);
    assert_eq!(
        last_note(&sim),
        "To cook meat you still need a burning fire within reach."
    );

    light_fire(&mut sim, 3 * HOUR);
    sim.apply(Command::Craft(Recipe::CookMeat));
    assert_eq!(run_until_idle(&mut sim), 1_200);

    assert_eq!(carried(&sim, Item::RawMeat), 0);
    assert_eq!(carried(&sim, Item::CookedMeat), 2);
    assert_eq!(sim.state.player.skills().experience(Skill::Cooking), 20);
}

#[test]
fn what_is_made_takes_what_it_is_made_of_from_the_ground_too() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::SpruceBough, 15);
    sim.apply(Command::Drop(Item::SpruceBough, 10));
    assert_eq!(carried(&sim, Item::SpruceBough), 5);
    assert_eq!(sim.at_hand.loose.count(Item::SpruceBough), 10);

    sim.apply(Command::Craft(Recipe::Bed));
    assert_eq!(run_until_idle(&mut sim), 900);

    assert!(sim.at_hand.bed);
    assert_eq!(carried(&sim, Item::SpruceBough), 0);
    assert!(sim.at_hand.loose.is_empty());
    assert_eq!(within_reach(&sim, Kind::Pile(Item::SpruceBough)), None);
    assert_eq!(last_note(&sim), "You lay the boughs into a springy bed.");
}

#[test]
fn nothing_is_made_of_too_little() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::SpruceRoot, 1);

    sim.apply(Command::Craft(Recipe::Cordage));

    assert_eq!(sim.state.player.activity(), None);
    assert_eq!(
        last_note(&sim),
        "To twist cordage you still need 1 × spruce root."
    );
}

#[test]
fn job_set_aside_is_taken_up_from_where_it_was_left() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::SpruceBough, 15);

    sim.apply(Command::Craft(Recipe::Bed));
    run(&mut sim, 300);
    sim.apply(Command::Stop);
    assert_eq!(sim.view().work, None);
    run(&mut sim, 50);

    sim.apply(Command::Craft(Recipe::Bed));
    assert_eq!(sim.view().work.map(|work| work.done), Some(300));
    assert_eq!(run_until_idle(&mut sim), 600);
    assert!(sim.at_hand.bed);

    carry(&mut sim, Item::SpruceBough, 15);
    sim.apply(Command::Craft(Recipe::Bed));
    assert_eq!(sim.view().work.map(|work| work.done), Some(0));
}

#[test]
fn job_under_way_carries_on_from_a_state() {
    let mut world = World::default();
    let spruce = Subject::Thing(world.add_thing(tree(PlantSpecies::Spruce, 0, 5_000)));
    let mut sim = simulation(world);
    sim.apply(Command::Act {
        verb: Verb::GatherSticks,
        subject: spruce,
    });
    run(&mut sim, 100);

    let mut resumed = Simulation::from_state(sim.state().clone());
    assert_eq!(resumed.view(), sim.view());

    assert_eq!(run_until_idle(&mut resumed), run_until_idle(&mut sim));
    assert_eq!(resumed.state().player, sim.state().player);
    assert_eq!(resumed.state().world, sim.state().world);
    assert_eq!(resumed.state().journal, sim.state().journal);
}

#[test]
fn lake_is_drunk_from_and_pots_are_filled_at_it() {
    let mut world = World::default();
    let water = Subject::Area(world.add_area(lake(5_000, 30_000)));
    let mut sim = simulation(world);
    let fill = Command::Act {
        verb: Verb::FillWater,
        subject: water,
    };

    sim.apply(fill.clone());
    assert_eq!(last_note(&sim), "You need a birch-bark pot for that.");

    sim.apply(Command::Act {
        verb: Verb::Drink,
        subject: water,
    });
    run_until_idle(&mut sim);
    assert!(sim.at_hand.water);
    assert!(sim.state.player.body().is_quenched());
    assert!(noted(&sim, "You drink your fill from the lake."));

    carry(&mut sim, Item::BarkPot, 2);
    carry(&mut sim, Item::BoiledWater, 1);
    sim.apply(fill.clone());
    run_until_idle(&mut sim);
    assert_eq!(carried(&sim, Item::Water), 1);

    sim.apply(fill);
    assert_eq!(last_note(&sim), "All your pots are full.");

    sim.apply(Command::Consume(Item::Water));
    assert_eq!(last_note(&sim), "You are not thirsty.");
}

#[test]
fn eating_fills_and_nobody_eats_when_full() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::RyeBread, 2);
    let hungry = sim.state.player.body().energy_percent();

    sim.apply(Command::Consume(Item::RyeBread));
    assert_eq!(run_until_idle(&mut sim), 180);

    assert!(sim.state.player.body().energy_percent() > hungry);
    assert!(sim.state.player.body().is_full());
    assert_eq!(carried(&sim, Item::RyeBread), 1);

    sim.apply(Command::Consume(Item::RyeBread));
    assert_eq!(last_note(&sim), "You are too full to eat.");
    sim.apply(Command::Consume(Item::Stone));
    assert_eq!(last_note(&sim), "Stone is not for eating.");
    sim.apply(Command::Consume(Item::Bilberries));
    assert_eq!(last_note(&sim), "You have no bilberries.");
}

#[test]
fn sleep_comes_only_to_the_tired_and_ends_when_rested() {
    let mut sim = simulation(World::default());

    sim.apply(Command::Sleep);
    assert!(!sim.view().sleeping);
    assert_eq!(last_note(&sim), "You are not tired enough to fall asleep.");

    run(&mut sim, 16 * HOUR);
    carry(&mut sim, Item::RyeBread, 1);
    sim.apply(Command::Sleep);
    assert!(sim.view().sleeping);
    assert_eq!(last_note(&sim), "You lie down to sleep on the bare ground.");

    let slept = run_until_idle(&mut sim);
    assert!((7 * HOUR..10 * HOUR).contains(&slept), "slept {slept} s");
    assert!(sim.state.player.body().is_rested());
    assert_eq!(last_note(&sim), "You wake up rested.");
}

#[test]
fn sleeper_is_woken_by_anything_asked_of_them() {
    let mut sim = simulation(World::default());
    run(&mut sim, 16 * HOUR);
    carry(&mut sim, Item::RyeBread, 1);
    sim.apply(Command::Sleep);
    run(&mut sim, HOUR);
    assert!(sim.view().sleeping);

    sim.apply(Command::Consume(Item::RyeBread));

    assert!(!sim.view().sleeping);
    assert!(noted(&sim, "You get up."));
    assert!(sim.view().work.is_some());
}

#[test]
fn life_left_alone_ends_in_thirst_and_then_nothing_moves_on() {
    let mut sim = simulation(World::default());
    let mut alerted = false;

    for _ in 0..7 * DAY {
        sim.tick();
        alerted |= sim.take_alert();
        if sim.state.death.is_some() {
            break;
        }
    }

    let death = sim.state.death.expect("nobody lives a week without water");
    assert_eq!(death.cause, Cause::Thirst);
    assert!(alerted);
    assert!(death.time.seconds_since(Timestamp::EPOCH) > 3 * DAY);
    assert_eq!(last_note(&sim), "You have died of thirst.");
    assert!(noted(&sim, "You are getting thirsty."));
    assert!(noted(&sim, "You are parched. You must drink."));
    assert_eq!(sim.view().death, Some(death));
    assert!(!sim.state.player.body().is_alive());

    let state = sim.state().clone();
    sim.tick();
    sim.apply(Command::Sleep);
    sim.apply(Command::EatSnow);
    assert_eq!(*sim.state(), state);
}

#[test]
fn snare_catches_a_hare_in_time_which_is_butchered_for_meat_and_a_pelt() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::Knife, 1);
    carry(&mut sim, Item::Snare, 1);
    carry(&mut sim, Item::Stick, 1);

    sim.apply(Command::Craft(Recipe::SetSnare));
    run_until_idle(&mut sim);
    assert_eq!(carried(&sim, Item::Snare), 0);
    let snare = within_reach(&sim, Kind::Structure(StructureKind::Snare));
    let snare = snare.expect("the snare is set within reach");

    let sprung = Kind::Structure(StructureKind::SprungSnare);
    let mut hours = 0;
    while sim.state.world.kind(snare) != Some(sprung) {
        assert!(hours < 1_500, "the snare never caught anything");
        *sim.state.player.body_mut() = Body::default();
        run(&mut sim, HOUR);
        hours += 1;
    }
    assert_eq!(within_reach(&sim, sprung), Some(snare));

    sim.apply(Command::Act {
        verb: Verb::Collect,
        subject: snare,
    });
    run_until_idle(&mut sim);
    assert_eq!(carried(&sim, Item::HareCarcass), 1);
    assert_eq!(
        sim.state.world.kind(snare),
        Some(Kind::Structure(StructureKind::Snare))
    );

    sim.apply(Command::Craft(Recipe::Butcher));
    run_until_idle(&mut sim);
    assert_eq!(carried(&sim, Item::HareCarcass), 0);
    assert_eq!(carried(&sim, Item::RawMeat), 3);
    assert_eq!(carried(&sim, Item::HarePelt), 1);
    assert_eq!(carried(&sim, Item::Bone), 2);

    sim.apply(Command::Act {
        verb: Verb::TakeDown,
        subject: snare,
    });
    run_until_idle(&mut sim);
    assert_eq!(carried(&sim, Item::Snare), 1);
    assert_eq!(sim.state.world.kind(snare), None);
}

#[test]
fn berries_are_picked_only_when_they_are_ripe() {
    let patch = |world: &mut World| Subject::Thing(world.add_thing(bilberries(0, 1_500)));
    let pick = |subject| Command::Act {
        verb: Verb::PickBerries,
        subject,
    };

    let mut world = World::default();
    let in_may = patch(&mut world);
    let mut sim = boreal(world, 0, 0);
    sim.apply(pick(in_may));
    assert_eq!(
        last_note(&sim),
        "There are no ripe berries here at this time of the year."
    );

    let mut world = World::default();
    let in_august = patch(&mut world);
    let mut sim = boreal(world, 92, 0);
    sim.apply(pick(in_august));
    let ticks = run_until_idle(&mut sim);
    assert!((3_400..=3_600).contains(&ticks), "picking took {ticks} s");
    assert_eq!(carried(&sim, Item::Bilberries), 20);
    assert_eq!(last_note(&sim), "This patch has been picked clean.");
}

#[test]
fn things_put_down_lie_in_one_pile_until_picked_up() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::Stone, 3);
    carry(&mut sim, Item::Stick, 1);

    sim.apply(Command::Drop(Item::Stone, 2));
    sim.apply(Command::Drop(Item::Stick, 5));
    sim.apply(Command::Drop(Item::Log, 1));

    assert_eq!(carried(&sim, Item::Stone), 1);
    assert_eq!(carried(&sim, Item::Stick), 0);
    assert_eq!(sim.at_hand.loose.count(Item::Stone), 2);
    assert_eq!(sim.at_hand.loose.count(Item::Stick), 1);
    assert_eq!(sim.surroundings.within_reach.len(), 1);
    assert_eq!(last_note(&sim), "You put down stick.");

    let pile = within_reach(&sim, Kind::Pile(Item::Stone)).expect("the pile is at hand");
    sim.apply(Command::Act {
        verb: Verb::PickUp,
        subject: pile,
    });
    run_until_idle(&mut sim);

    assert_eq!(carried(&sim, Item::Stone), 3);
    assert_eq!(carried(&sim, Item::Stick), 1);
    assert!(sim.surroundings.is_empty());
    assert_eq!(last_note(&sim), "You pick up stick, 2 × stone.");
}

#[test]
fn raw_food_goes_off_in_the_warm_wherever_it_is_kept() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::RawFish, 2);
    carry(&mut sim, Item::DriedFish, 1);
    sim.apply(Command::Drop(Item::RawFish, 1));
    assert_eq!(sim.at_hand.loose.count(Item::RawFish), 1);

    run(&mut sim, 15 * HOUR);
    assert_eq!(carried(&sim, Item::RawFish), 1);

    run(&mut sim, 2 * HOUR);
    assert_eq!(carried(&sim, Item::RawFish), 0);
    assert_eq!(carried(&sim, Item::DriedFish), 1);
    assert!(noted(&sim, "Your raw perch has gone off."));
    assert!(sim.at_hand.loose.is_empty());
    assert!(sim.surroundings.is_empty());
}

#[test]
fn food_hung_on_a_rack_dries_in_two_days() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::Knife, 1);
    carry(&mut sim, Item::Stick, 8);
    carry(&mut sim, Item::Cordage, 2);
    carry(&mut sim, Item::RawMeat, 2);
    carry(&mut sim, Item::RawFish, 1);

    sim.apply(Command::Craft(Recipe::DryingRack));
    run_until_idle(&mut sim);
    let rack = within_reach(&sim, Kind::Structure(StructureKind::DryingRack));
    let rack = rack.expect("the rack stands within reach");
    let take = Command::Act {
        verb: Verb::Collect,
        subject: rack,
    };

    sim.apply(take.clone());
    assert_eq!(last_note(&sim), "There is nothing in the drying rack.");

    sim.apply(Command::Act {
        verb: Verb::HangFood,
        subject: rack,
    });
    run_until_idle(&mut sim);
    assert_eq!(carried(&sim, Item::RawMeat), 0);

    for _ in 0..2 * 24 {
        *sim.state.player.body_mut() = Body::default();
        run(&mut sim, HOUR);
    }
    sim.apply(take);
    run_until_idle(&mut sim);

    assert_eq!(carried(&sim, Item::DriedMeat), 2);
    assert_eq!(carried(&sim, Item::DriedFish), 1);
    assert_eq!(last_note(&sim), "You take the dried food off the rack.");
}

#[test]
fn fish_bite_for_those_with_a_rod_and_patience() {
    let mut world = World::default();
    let water = Subject::Area(world.add_area(lake(5_000, 30_000)));
    let mut sim = simulation(world);
    let fish = Command::Act {
        verb: Verb::Fish,
        subject: water,
    };

    sim.apply(fish.clone());
    assert_eq!(last_note(&sim), "You need a fishing rod for that.");

    carry(&mut sim, Item::FishingRod, 1);
    sim.apply(fish);
    run(&mut sim, 6 * HOUR);

    assert!(sim.view().work.is_some(), "fishing goes on until stopped");
    assert!(carried(&sim, Item::RawFish) > 0);
    assert!(noted(&sim, "You land a perch."));
    assert!(sim.state.player.skills().level(Skill::Fishing) > 0);
    assert!(noted(&sim, "You are getting better at fishing: level 1."));
}

#[test]
fn lean_to_is_built_and_slept_under() {
    let mut sim = simulation(World::default());
    carry(&mut sim, Item::Stick, 15);
    carry(&mut sim, Item::SpruceBough, 25);
    carry(&mut sim, Item::Cordage, 2);

    sim.apply(Command::Craft(Recipe::LeanTo));
    run_until_idle(&mut sim);
    assert_eq!(sim.at_hand.shelter, Some(StructureKind::LeanTo));
    assert!(sim.state.player.carried().is_empty());

    run(&mut sim, 14 * HOUR);
    sim.apply(Command::Sleep);
    assert_eq!(last_note(&sim), "You lie down to sleep under the lean-to.");
}

#[test]
fn nothing_is_built_where_there_is_only_water_around() {
    let mut world = World::default();
    world.add_area(lake(-30_000, 30_000));
    let mut sim = simulation(world);
    carry(&mut sim, Item::SpruceBough, 15);

    sim.apply(Command::Craft(Recipe::Bed));

    assert_eq!(sim.state.player.activity(), None);
    assert_eq!(last_note(&sim), "There is no room to build anything here.");
}

#[test]
fn winter_night_is_dark_but_for_the_light_of_a_fire() {
    let mut world = World::default();
    let near = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 0, 8_000)));
    let far = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 9_000, 20_000)));
    // Midnight in the middle of December.
    let mut sim = boreal(world, 229, 16);
    assert_eq!(sim.conditions.light, Light::Dark);

    assert!(sim.surroundings.sighting(near).is_some());
    assert!(sim.surroundings.sighting(far).is_none());

    light_fire(&mut sim, HOUR);
    assert!(sim.surroundings.sighting(far).is_some());

    run(&mut sim, HOUR + 60);
    assert!(sim.surroundings.sighting(far).is_none());
}

#[test]
fn day_breaks_on_what_the_night_hid() {
    let mut world = World::default();
    let far = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 9_000, 60_000)));
    // Midnight at the end of September.
    let mut sim = boreal(world, 152, 16);
    assert!(sim.surroundings.sighting(far).is_none());

    run(&mut sim, 12 * HOUR);

    assert_eq!(sim.conditions.light, Light::Day);
    assert!(sim.surroundings.sighting(far).is_some());
}

#[test]
fn winter_chills_to_the_bone_those_without_a_fire() {
    // The middle of January.
    let mut cold = boreal(World::default(), 259, 0);
    let mut warm = boreal(World::default(), 259, 0);
    assert!(cold.conditions.snow && cold.conditions.frozen);
    light_fire(&mut warm, 6 * HOUR);
    assert!(warm.view().felt > cold.view().felt);

    run(&mut cold, 5 * HOUR);
    run(&mut warm, 5 * HOUR);

    let chill = |sim: &Simulation| sim.state.player.body().temperature();
    assert!(
        chill(&cold) < Temperature::from_degrees(36),
        "{}",
        chill(&cold)
    );
    assert!(chill(&warm) > chill(&cold).warmer_by(10));
    let ailments = cold.state.player.body().ailments();
    assert!(
        ailments.contains(&Ailment::Hypothermic) || ailments.contains(&Ailment::Freezing),
        "{ailments:?}"
    );
    assert!(cold.take_alert());
}

#[test]
fn snow_is_eaten_for_water_at_the_cost_of_warmth() {
    let mut winter = boreal(World::default(), 259, 0);
    let mut summer = boreal(World::default(), 60, 0);
    let before = winter.state.player.body().water_percent();

    summer.apply(Command::EatSnow);
    assert_eq!(last_note(&summer), "There is no snow on the ground.");

    winter.apply(Command::EatSnow);
    assert_eq!(run_until_idle(&mut winter), 120);
    assert!(winter.state.player.body().water_percent() > before);
    assert!(winter.state.player.body().temperature() < Temperature::from_degrees(37));
}

#[test]
fn frozen_lake_takes_an_axe_to_get_at() {
    let mut world = World::default();
    let water = Subject::Area(world.add_area(lake(1_000, 30_000)));
    let mut sim = boreal(world, 259, 0);
    let drink = Command::Act {
        verb: Verb::Drink,
        subject: water,
    };

    sim.apply(drink.clone());
    assert_eq!(
        last_note(&sim),
        "The lake is frozen over. You need an axe to cut a hole in the ice."
    );

    carry(&mut sim, Item::StoneAxe, 1);
    sim.apply(drink);
    run_until_idle(&mut sim);
    assert!(sim.state.player.body().is_quenched());
}

#[test]
fn walking_is_slower_through_snow_and_under_a_heavy_load() {
    let step = |sim: &mut Simulation| {
        let mut world_tree = None;
        for (id, _) in sim.state.world.things() {
            world_tree = Some(Subject::Thing(id));
        }
        sim.apply(Command::WalkTo(world_tree.expect("there is a tree")));
        sim.tick();
        sim.state.player.location().y()
    };
    let forest = || {
        let mut world = World::default();
        world.add_thing(tree(PlantSpecies::Birch, 0, 50_000));
        world
    };

    let mut light = simulation(forest());
    let mut loaded = simulation(forest());
    carry(&mut loaded, Item::Log, 2);
    let mut snowed_in = boreal(forest(), 259, 4);

    assert_eq!(step(&mut light), 1_400);
    assert_eq!(step(&mut loaded), 1_050);
    assert_eq!(step(&mut snowed_in), 1_120);
    assert_eq!(light.view().walked, Length::from_millimeters(1_400));
}

#[test]
fn generated_life_can_be_lived_off_the_land() {
    let mut sim = Simulation::generate(Name::default(), Sex::Male, 5);
    let kinds: Vec<Kind> = sim.view().map.things.iter().map(|mark| mark.kind).collect();

    assert!(kinds.iter().any(|kind| kind.is_tree()));
    assert_eq!(sim.view().journal.len(), 1);
    assert_eq!(sim.view().started, FIRST_MORNING);
    assert_eq!(carried(&sim, Item::RyeBread), 4);

    let spruce = sim
        .view()
        .map
        .things
        .iter()
        .find(|mark| mark.kind == Kind::Plant(PlantSpecies::Spruce))
        .map(|mark| mark.subject)
        .expect("a spruce is in sight");
    sim.apply(Command::Act {
        verb: Verb::DigRoots,
        subject: spruce,
    });
    run_until_idle(&mut sim);
    sim.apply(Command::Craft(Recipe::Cordage));
    run_until_idle(&mut sim);

    assert_eq!(carried(&sim, Item::Cordage), 1);
}
