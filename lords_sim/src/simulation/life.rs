//! Everything there is to living apart from walking: what the body goes through as time
//! passes, and what the player sets about doing.

use crate::action::{Job, Verb};
use crate::body::{Cause, Effort};
use crate::climate::{Sky, Temperature, Wind};
use crate::craft::{Product, Recipe};
use crate::item::{HEAVY_LOAD, Item, Stack};
use crate::journal::Tone;
use crate::object::{Activity, Living, Object};
use crate::skill::Skill;
use crate::space::Location;
use crate::state::Death;
use crate::time::Timestamp;
use crate::time::gregorian::{Date, SECONDS_PER_DAY, SECONDS_PER_HOUR};
use crate::world::{Kind, Pile, PlantSpecies, Structure, StructureKind, Subject, Thing, ThingId};

use super::Simulation;

/// How many hundredths faster each level of a skill makes what it is the skill of.
const SPEEDUP_PER_LEVEL: u32 = 4;

/// The experience that each minute of doing something gives of its skill.
const EXPERIENCE_PER_MINUTE: u32 = 1;

/// How many sticks, boughs, and roots, pieces of bark, and helpings of berries are had
/// for each time they are gathered.
const STICKS_PER_ROUND: u32 = 3;
const BOUGHS_PER_ROUND: u32 = 3;
const ROOTS_PER_ROUND: u32 = 2;
const BARK_PER_TREE: u32 = 3;

/// How tall a spruce has to be to have boughs worth cutting, in millimeters.
const BOUGH_HEIGHT: i64 = 2_000;

/// How many sticks lighting a cold fireplace takes besides the tinder.
const KINDLING: u32 = 3;

/// The chance in a hundred that a fire catches for someone unskilled on a calm, dry day,
/// and how much each level of the skill adds to it.
const FIRE_CHANCE: i64 = 75;
const FIRE_CHANCE_PER_LEVEL: i64 = 3;

/// The chance in a hundred that a round of fishing lands a fish for someone unskilled,
/// and how much each level of the skill adds to it.
const BITE_CHANCE: i64 = 40;
const BITE_CHANCE_PER_LEVEL: i64 = 4;

/// The chance in a thousand that a snare or a fish trap makes a catch in an hour for
/// someone unskilled, and how much each level of the skill adds to it.
const SNARE_CHANCE: u64 = 12;
const SNARE_CHANCE_PER_LEVEL: u64 = 2;
const FISH_TRAP_CHANCE: u64 = 40;
const FISH_TRAP_CHANCE_PER_LEVEL: u64 = 4;
const FISH_TRAP_HOLDS: u32 = 3;

/// The chance in a hundred that a stone axe breaks on a tree.
const AXE_BREAKING_CHANCE: u64 = 8;

/// How many in-game seconds food takes to dry on a rack.
const DRYING_TIME: u64 = 2 * SECONDS_PER_DAY;

/// The shortest and the longest a stomach sickness lasts, in in-game hours.
const SICKNESS_HOURS: (i64, i64) = (12, 36);

/// The water in a mouthful of snow in milliliters, and how many thousandths of a degree
/// eating it cools the body.
const SNOW_WATER: u32 = 200;
const SNOW_CHILL: i32 = 100;

/// The age from which each day may be the last, and how fast the chance of that grows.
pub(super) const OLD_AGE: u64 = 60;
pub(super) const OLD_AGE_ODDS: u64 = 200_000;

impl Simulation {
    /// Returns how hard the player is working at what they are doing.
    pub(super) fn effort(&self) -> Effort {
        match self.state.player.activity() {
            None => Effort::Resting,
            Some(Activity::Sleeping) => Effort::Sleeping,
            Some(Activity::Walking { .. }) => Effort::Walking,
            Some(Activity::Doing { job, .. }) => match job {
                Job::Consume(_) | Job::EatSnow | Job::Appeal(_) => Effort::Resting,
                Job::Act { verb, .. } => match verb {
                    Verb::Drink
                    | Verb::Fish
                    | Verb::AddStick
                    | Verb::AddLog
                    | Verb::Stoke
                    | Verb::Light
                    | Verb::Talk
                    | Verb::Gift
                    | Verb::Invite
                    | Verb::Court
                    | Verb::TryForChild
                    | Verb::Hire
                    | Verb::Deposit => Effort::Resting,
                    _ => Effort::Working,
                },
                Job::Craft(recipe) => match recipe.skill() {
                    Skill::Building => Effort::Working,
                    _ => Effort::Resting,
                },
            },
        }
    }

    /// Returns how warm it feels to the player where they are, dressed as they are, before
    /// what they are doing warms them.
    pub(super) fn felt(&self) -> Temperature {
        let conditions = &self.conditions;
        let at_hand = &self.at_hand;
        let player = &self.state.player;
        let mut tenths = player.clothing_warmth();

        if at_hand.shelter.is_none() {
            tenths -= match conditions.wind {
                Wind::Calm => 0,
                Wind::Breeze => 15,
                Wind::Windy => 40,
            };
            tenths -= match conditions.sky {
                Sky::Clear | Sky::Overcast => 0,
                Sky::Snow => 15,
                Sky::Rain => 40,
            };
        }
        tenths += match at_hand.shelter {
            Some(kind) if kind.is_house() => 100,
            Some(_) => 30,
            None => 0,
        };
        if at_hand.fire {
            tenths += if at_hand.shelter.is_some_and(StructureKind::is_house) {
                200
            } else {
                150
            };
        }
        if at_hand.bed && player.activity() == Some(Activity::Sleeping) {
            tenths += 30;
        }

        conditions.temperature.warmer_by(tenths)
    }

    /// Returns how many in-game seconds `job` takes the player as they are.
    pub(super) fn needed(&self, job: Job) -> u32 {
        let player = &self.state.player;
        let (seconds, skill) = match job {
            Job::Consume(item) => return item.food().map_or(1, |food| food.seconds),
            Job::EatSnow => return 120,
            Job::Appeal(appeal) => return appeal.seconds(),
            Job::Craft(recipe) => (recipe.seconds(), Some(recipe.skill())),
            Job::Act { verb, subject } => (self.verb_seconds(verb, subject), verb.skill()),
        };

        let level = skill.map_or(0, |skill| player.skills().level(skill));
        let skilled = seconds * (100 - SPEEDUP_PER_LEVEL * level) / 100;
        (skilled * 100 / player.body().pace_percent().max(1)).max(1)
    }

    fn verb_seconds(&self, verb: Verb, subject: Subject) -> u32 {
        let plant = match subject {
            Subject::Thing(id) => self.state.world.plant(id),
            Subject::Area(_) | Subject::Person(_) => None,
        };

        match verb {
            Verb::GatherSticks => 120,
            Verb::GatherBoughs => 150,
            Verb::StripBark => match plant.map(|plant| plant.species()) {
                Some(PlantSpecies::Pine) => 600,
                _ => 300,
            },
            Verb::DigRoots => 420,
            Verb::PickBerries => 180,
            Verb::Fell => {
                // Iron cuts twice as fast as stone.
                let per_millimeter = if self.has(Item::IronAxe) > 0 { 3 } else { 6 };
                plant.map_or(300, |plant| {
                    (plant.trunk_diameter().as_millimeters() as u32 * per_millimeter).max(300)
                })
            }
            Verb::DigClay => 600,
            Verb::RakeOre => 900,
            Verb::Hire => 300,
            Verb::PickUp | Verb::AddLog => 20,
            Verb::Drink | Verb::FillWater | Verb::TakeDown | Verb::Stoke => 60,
            Verb::Fish => 600,
            Verb::AddStick => 10,
            Verb::Light | Verb::Collect | Verb::HangFood => 120,
            Verb::Talk => 600,
            Verb::Gift | Verb::Deposit => 60,
            Verb::Invite => 300,
            Verb::Court => 900,
            Verb::TryForChild => 1_800,
            Verb::Build => 3_600,
        }
    }

    /// Moves the job under way on by a tick.
    pub(super) fn work(&mut self) {
        let Some(Activity::Doing { job, done }) = self.state.player.activity() else {
            return;
        };

        let done = done + 1;
        if done < self.needed(job) {
            let activity = Activity::Doing { job, done };
            self.state.player.animal_mut().set_activity(Some(activity));
            return;
        }

        self.state.player.animal_mut().set_activity(None);
        self.state.player.set_unfinished(None);
        let finished = self.finish(job);
        if self.stale {
            self.look();
        }

        let repeats = match job {
            Job::Act { verb, .. } => verb.repeats(),
            Job::Craft(recipe) => recipe.repeats(),
            Job::Consume(_) | Job::EatSnow | Job::Appeal(_) => false,
        };
        if finished && repeats {
            let gathering = matches!(job, Job::Act { .. });
            if gathering && self.state.player.carried().weight() > HEAVY_LOAD {
                self.note(Tone::Plain, "You stop, as your load is getting heavy.");
                return;
            }
            match self.check(job) {
                Ok(()) => self.begin(job),
                Err(reason) if matches!(job, Job::Act { .. }) => self.note(Tone::Plain, reason),
                Err(_) => {}
            }
        }
    }

    /// Has the player set about `job`, walking to what it is done to first if need be.
    pub(super) fn take_up(&mut self, job: Job) {
        if self.state.player.activity() == Some(Activity::Sleeping) {
            self.wake("You get up.");
        }
        if let Err(reason) = self.check(job) {
            self.note(Tone::Plain, reason);
            return;
        }

        self.set_aside();
        match job {
            Job::Act { subject, .. } if !self.is_within_reach(subject) => {
                self.walk_to(subject, Some(job));
            }
            _ => self.begin(job),
        }
    }

    /// Stops whatever the player is doing, keeping in mind how far a job had got.
    pub(super) fn set_aside(&mut self) {
        match self.state.player.activity() {
            Some(Activity::Doing { job, done }) => {
                self.state.player.set_unfinished(Some((job, done)));
            }
            Some(Activity::Sleeping) => self.wake("You get up."),
            Some(Activity::Walking { .. }) | None => {}
        }

        self.state.player.animal_mut().set_activity(None);
        self.state.player.set_intent(None);
        self.route.clear();
    }

    /// Has the player set about what they walked here for, if it was for something.
    pub(super) fn arrive(&mut self) {
        let Some(job) = self.state.player.intent() else {
            return;
        };
        self.state.player.set_intent(None);

        let there = match job {
            Job::Act { subject, .. } => self.is_within_reach(subject),
            Job::Craft(_) | Job::Consume(_) | Job::EatSnow | Job::Appeal(_) => true,
        };
        if !there {
            return;
        }
        match self.check(job) {
            Ok(()) => self.begin(job),
            Err(reason) => self.note(Tone::Plain, reason),
        }
    }

    /// Starts on `job`, from where it was left if it is the job last left unfinished.
    pub(super) fn begin(&mut self, job: Job) {
        let done = match self.state.player.unfinished() {
            Some((unfinished, done)) if unfinished == job => done,
            _ => 0,
        };

        self.state.player.set_unfinished(None);
        self.state
            .player
            .animal_mut()
            .set_activity(Some(Activity::Doing { job, done }));
    }

    pub(super) fn lie_down(&mut self) {
        let body = self.state.player.body();
        if self.state.player.activity() == Some(Activity::Sleeping) {
            return;
        }
        if body.is_too_cold_to_sleep() {
            self.note(Tone::Plain, "You are too cold to fall asleep.");
            return;
        }
        if body.is_wide_awake() {
            self.note(Tone::Plain, "You are not tired enough to fall asleep.");
            return;
        }

        self.set_aside();
        self.state
            .player
            .animal_mut()
            .set_activity(Some(Activity::Sleeping));
        let place = match (self.at_hand.shelter, self.at_hand.bed) {
            (Some(kind), _) if kind.is_house() => "indoors",
            (Some(_), _) => "under the lean-to",
            (None, true) => "on the bough bed",
            (None, false) => "on the bare ground",
        };
        self.note(Tone::Plain, format!("You lie down to sleep {place}."));
    }

    pub(super) fn wake(&mut self, words: &str) {
        if self.state.player.activity() == Some(Activity::Sleeping) {
            self.state.player.animal_mut().set_activity(None);
            self.note(Tone::Plain, words);
        }
    }

    pub(super) fn drop(&mut self, item: Item, count: u32) {
        let dropped = self.state.player.carried_mut().take(item, count);
        if dropped.count == 0 {
            return;
        }

        self.put_down(dropped);
        self.note(
            Tone::Plain,
            format!("You put down {}.", counted(dropped.item, dropped.count)),
        );
    }

    pub(super) fn is_within_reach(&self, subject: Subject) -> bool {
        self.surroundings
            .within_reach
            .iter()
            .any(|sighting| sighting.subject == subject)
    }

    /// Returns how many of `item` the player has to use, carried or lying within reach.
    pub(super) fn has_axe(&self) -> bool {
        self.at_hand
            .has_tool(self.state.player.carried(), Item::StoneAxe)
    }

    pub(super) fn has(&self, item: Item) -> u32 {
        self.at_hand.count(self.state.player.carried(), item)
    }

    /// Returns why the player cannot do `job` as things are, if they cannot.
    pub(super) fn check(&self, job: Job) -> Result<(), String> {
        let player = &self.state.player;
        let body = player.body();
        let carried = player.carried();

        match job {
            Job::Act { verb, subject } => self.check_act(verb, subject),
            Job::Appeal(appeal) => self.check_appeal(appeal),
            Job::Craft(recipe) => {
                let lacks = self.at_hand.lacks(carried, recipe);
                if recipe == Recipe::Cairn && self.state.settlement.is_some() {
                    return Err("You have founded a settlement already.".to_owned());
                }
                if lacks.is_empty() {
                    return match recipe.product() {
                        Product::Structure(_) if self.site().is_none() => {
                            Err("There is no room to build anything here.".to_owned())
                        }
                        _ => Ok(()),
                    };
                }

                let lacks: Vec<String> = lacks.into_iter().map(|lack| lack.describe()).collect();
                Err(format!(
                    "To {} you still need {}.",
                    recipe.name().to_lowercase(),
                    lacks.join(", ")
                ))
            }
            Job::Consume(item) => {
                let Some(_) = item.food() else {
                    return Err(format!("{} is not for eating.", item.name()));
                };
                if self.has(item) == 0 {
                    Err(format!("You have no {}.", item.name().to_lowercase()))
                } else if item.is_drink() && body.is_quenched() {
                    Err("You are not thirsty.".to_owned())
                } else if !item.is_drink() && body.is_full() {
                    Err("You are too full to eat.".to_owned())
                } else {
                    Ok(())
                }
            }
            Job::EatSnow => {
                if !self.conditions.snow {
                    Err("There is no snow on the ground.".to_owned())
                } else if body.is_quenched() {
                    Err("You are not thirsty.".to_owned())
                } else {
                    Ok(())
                }
            }
        }
    }

    fn check_act(&self, verb: Verb, subject: Subject) -> Result<(), String> {
        let world = &self.state.world;
        let player = &self.state.player;
        let carried = player.carried();
        let now = self.state.time;

        let Some(kind) = world.kind(subject) else {
            return Err("That is no longer there.".to_owned());
        };
        if !self.knows(subject) {
            return Err("You do not know of such a thing.".to_owned());
        }
        if !Verb::for_kind(kind).contains(&verb) {
            return Err(format!(
                "That cannot be done to a {}.",
                kind.name().to_lowercase()
            ));
        }
        if let Some(tool) = verb.tool()
            && !self.at_hand.has_tool(carried, tool)
        {
            return Err(format!(
                "You need a {} for that.",
                tool.name().to_lowercase()
            ));
        }

        let name = kind.name().to_lowercase();
        let (plant, structure) = match subject {
            Subject::Thing(id) => (world.plant(id), world.structure(id)),
            Subject::Area(_) | Subject::Person(_) => (None, None),
        };
        let no_room = || Err("You cannot carry any more.".to_owned());
        // A well never freezes over.
        let iced_over = self.conditions.frozen && !self.has_axe() && structure.is_none();
        let ice =
            || Err("The lake is frozen over. You need an axe to cut a hole in the ice.".to_owned());

        match verb {
            Verb::GatherSticks => {
                if plant.is_some_and(|plant| plant.stock(now) == 0) {
                    Err(format!("There is no dead wood left on this {name}."))
                } else if carried.room_for(Item::Stick) == 0 {
                    no_room()
                } else {
                    Ok(())
                }
            }
            Verb::GatherBoughs => {
                if plant.is_some_and(|plant| plant.height().as_millimeters() < BOUGH_HEIGHT) {
                    Err(format!(
                        "This {name} is too small to have boughs worth cutting."
                    ))
                } else if carried.room_for(Item::SpruceBough) == 0 {
                    no_room()
                } else {
                    Ok(())
                }
            }
            Verb::StripBark => {
                if plant.is_some_and(|plant| !plant.is_grown()) {
                    Err(format!(
                        "This {name} is too young to have bark worth taking."
                    ))
                } else if plant.is_some_and(|plant| plant.is_stripped()) {
                    Err(format!("The bark of this {name} has been taken already."))
                } else {
                    Ok(())
                }
            }
            Verb::DigRoots => {
                if carried.room_for(Item::SpruceRoot) == 0 {
                    no_room()
                } else {
                    Ok(())
                }
            }
            Verb::PickBerries => {
                let Some(plant) = plant else {
                    return Ok(());
                };
                if !self.is_ripe(plant.species()) {
                    Err("There are no ripe berries here at this time of the year.".to_owned())
                } else if plant.stock(now) == 0 {
                    Err("This patch has been picked clean.".to_owned())
                } else {
                    Ok(())
                }
            }
            Verb::PickUp => {
                let fits = match subject {
                    Subject::Thing(id) => world.pile(id).is_some_and(|pile| {
                        let stacks = pile.items().stacks();
                        stacks.iter().any(|stack| carried.room_for(stack.item) > 0)
                    }),
                    Subject::Area(_) | Subject::Person(_) => false,
                };
                if fits { Ok(()) } else { no_room() }
            }
            Verb::Drink => {
                if iced_over {
                    ice()
                } else if player.body().is_quenched() {
                    Err("You are not thirsty.".to_owned())
                } else {
                    Ok(())
                }
            }
            Verb::FillWater => {
                if iced_over {
                    ice()
                } else if self.empty_pots() == 0 {
                    Err("All your pots are full.".to_owned())
                } else {
                    Ok(())
                }
            }
            Verb::Fish => {
                if iced_over {
                    ice()
                } else {
                    Ok(())
                }
            }
            Verb::AddStick | Verb::AddLog => {
                let fuel = if verb == Verb::AddStick {
                    Item::Stick
                } else {
                    Item::Log
                };
                let room = structure.map_or(0, |fire| fire.room(now));
                if self.has(fuel) == 0 {
                    Err(format!(
                        "You have no {} to add.",
                        fuel.name().to_lowercase()
                    ))
                } else if room < u64::from(fuel.burn_time()) {
                    Err("The fire has all the wood it can take.".to_owned())
                } else {
                    Ok(())
                }
            }
            Verb::Stoke => {
                let room = structure.map_or(0, |fire| fire.room(now));
                if self.has(Item::Stick) + self.has(Item::Log) == 0 {
                    Err("You have no wood to put on the fire.".to_owned())
                } else if room < u64::from(Item::Stick.burn_time()) {
                    Err("The fire has all the wood it can take.".to_owned())
                } else {
                    Ok(())
                }
            }
            Verb::Light => {
                if self.has(Item::BirchBark) == 0 || self.has(Item::Stick) < KINDLING {
                    Err(format!(
                        "To light the fire you need a piece of birch bark for tinder and \
                         {KINDLING} sticks."
                    ))
                } else {
                    Ok(())
                }
            }
            Verb::Collect => {
                if structure.is_some_and(|structure| structure.hold().is_empty()) {
                    Err(format!("There is nothing in the {name}."))
                } else {
                    Ok(())
                }
            }
            Verb::HangFood => {
                if carried.has(Item::RawMeat) || carried.has(Item::RawFish) {
                    Ok(())
                } else {
                    Err("You have no raw meat or fish to hang.".to_owned())
                }
            }
            Verb::Fell | Verb::TakeDown | Verb::Build => Ok(()),
            Verb::DigClay | Verb::RakeOre => {
                let knows_iron = self.state.settlement.as_ref().is_some_and(|settlement| {
                    settlement.knows(crate::settlement::Advance::Ironworking)
                });
                let item = if verb == Verb::DigClay {
                    Item::Clay
                } else {
                    Item::BogOre
                };
                if self.conditions.frozen {
                    Err("The shore is frozen hard.".to_owned())
                } else if verb == Verb::RakeOre && !knows_iron {
                    Err("Nobody here knows what to look for, or what to do with it.".to_owned())
                } else if carried.room_for(item) == 0 {
                    no_room()
                } else {
                    Ok(())
                }
            }
            Verb::Hire => match subject {
                Subject::Person(id) => self.check_hire(id),
                Subject::Thing(_) | Subject::Area(_) => Ok(()),
            },
            Verb::Deposit => self.check_deposit(),
            Verb::Talk | Verb::Gift | Verb::Invite | Verb::Court | Verb::TryForChild => {
                match subject {
                    Subject::Person(id) => self.check_person(verb, id),
                    Subject::Thing(_) | Subject::Area(_) => Ok(()),
                }
            }
        }
    }

    /// Returns whether the berries of `species` are ripe, which they always are where
    /// there are no seasons.
    fn is_ripe(&self, species: PlantSpecies) -> bool {
        match self.state.climate {
            crate::climate::Climate::Still => species.berries().is_some(),
            crate::climate::Climate::Boreal { .. } => {
                species.is_ripe(Date::from_timestamp(self.state.time))
            }
        }
    }

    /// Returns how many of the pots the player carries have no water in them.
    pub(super) fn empty_pots(&self) -> u32 {
        let carried = self.state.player.carried();
        let full = carried.count(Item::Water) + carried.count(Item::BoiledWater);

        let room: u32 = carried
            .stacks()
            .iter()
            .map(|stack| stack.item.holds() * stack.count)
            .sum();
        room.saturating_sub(full)
    }

    /// Returns where something built by the player where they stand goes: next to them,
    /// on dry ground.
    pub(super) fn site(&self) -> Option<Location> {
        const STEP: i64 = 700;
        let at = self.state.player.location();

        [(0, 1), (1, 0), (0, -1), (-1, 0)]
            .into_iter()
            .map(|(east, north)| Location::new(at.x() + east * STEP, at.y() + north * STEP))
            .find(|site| !self.state.world.is_water(*site))
    }

    /// Brings about what `job` comes to now that the time it takes has been put in.
    /// Returns whether it came to what it was meant to.
    fn finish(&mut self, job: Job) -> bool {
        if let Err(reason) = self.check(job) {
            self.note(Tone::Plain, reason);
            return false;
        }
        let seconds = self.needed(job);

        let (finished, skill) = match job {
            Job::Act { verb, subject } => (self.finish_act(verb, subject), verb.skill()),
            Job::Craft(recipe) => (self.finish_craft(recipe), Some(recipe.skill())),
            Job::Consume(item) => (self.consume(item), None),
            Job::Appeal(appeal) => (self.finish_appeal(appeal), None),
            Job::EatSnow => {
                let body = self.state.player.body_mut();
                body.drink(SNOW_WATER);
                body.cool(SNOW_CHILL);
                self.note(Tone::Plain, "You eat snow. It chills you to the bone.");
                (true, None)
            }
        };

        if let Some(skill) = skill {
            let experience = (seconds / 60 * EXPERIENCE_PER_MINUTE).max(1);
            if let Some(level) = self.state.player.skills_mut().gain(skill, experience) {
                self.note(
                    Tone::Good,
                    format!(
                        "You are getting better at {}: level {level}.",
                        skill.name().to_lowercase()
                    ),
                );
            }
        }
        finished
    }

    fn consume(&mut self, item: Item) -> bool {
        let Some(food) = item.food() else {
            return false;
        };
        if self.has(item) == 0 {
            return false;
        }
        self.use_up(item, 1);

        self.state.player.body_mut().take_in(food);
        let name = item.name().to_lowercase();
        if item.is_drink() {
            self.note(Tone::Plain, format!("You drink the {name}."));
        } else {
            self.note(Tone::Plain, format!("You eat the {name}."));
        }
        self.risk_sickness(food.risk, "It does not sit well. You are sick.");
        true
    }

    fn risk_sickness(&mut self, risk: u32, words: &str) {
        if !self.state.rng.one_in(u64::from(risk), 1_000) {
            return;
        }

        let hours = self.state.rng.between(SICKNESS_HOURS) as u64;
        let until =
            Timestamp::from_seconds(self.state.time.as_seconds() + hours * SECONDS_PER_HOUR);
        self.state.player.body_mut().sicken(until);
        self.note(Tone::Bad, words);
    }

    fn finish_act(&mut self, verb: Verb, subject: Subject) -> bool {
        let now = self.state.time;
        let id = match subject {
            Subject::Thing(id) => id,
            Subject::Area(_) => return self.finish_at_water(verb, false),
            Subject::Person(id) => return self.finish_with_person(verb, id),
        };
        let Some(kind) = self.state.world.kind(subject) else {
            return false;
        };
        let name = kind.name().to_lowercase();

        match verb {
            Verb::GatherSticks => {
                let Some(plant) = self.state.world.plant_mut(id) else {
                    return false;
                };
                let count = STICKS_PER_ROUND.min(plant.stock(now));
                plant.take(count, now);
                self.give(Stack::new(Item::Stick, count));
                self.note(Tone::Plain, "You gather dead branches for sticks.");
            }
            Verb::GatherBoughs => {
                self.give(Stack::new(Item::SpruceBough, BOUGHS_PER_ROUND));
                self.note(Tone::Plain, "You cut spruce boughs.");
            }
            Verb::DigRoots => {
                self.give(Stack::new(Item::SpruceRoot, ROOTS_PER_ROUND));
                self.note(Tone::Plain, "You dig up long, tough spruce roots.");
            }
            Verb::StripBark => {
                let Some(plant) = self.state.world.plant_mut(id) else {
                    return false;
                };
                plant.strip();
                let bark = if plant.species() == PlantSpecies::Pine {
                    Item::InnerBark
                } else {
                    Item::BirchBark
                };
                self.give(Stack::new(bark, BARK_PER_TREE));
                self.note(Tone::Plain, format!("You strip bark off the {name}."));
            }
            Verb::PickBerries => {
                let foraging = self.state.player.skills().level(Skill::Foraging);
                let Some(plant) = self.state.world.plant_mut(id) else {
                    return false;
                };
                let Some(berries) = plant.species().berries() else {
                    return false;
                };
                let count = (1 + foraging / 5).min(plant.stock(now));
                plant.take(count, now);
                self.give(Stack::new(berries, count));
                self.note(
                    Tone::Plain,
                    format!("You pick {}.", berries.name().to_lowercase()),
                );
            }
            Verb::Fell => {
                let Some(plant) = self.state.world.plant(id) else {
                    return false;
                };
                let (logs, sticks) = plant.timber();
                let at = plant.location();
                self.state.world.remove_thing(id);
                self.add_thing(Thing::Pile(Pile::new(
                    Object::new(at, now),
                    [Stack::new(Item::Log, logs), Stack::new(Item::Stick, sticks)],
                )));

                if self.has(Item::IronAxe) == 0 && self.state.rng.chance(AXE_BREAKING_CHANCE) {
                    self.use_up(Item::StoneAxe, 1);
                    self.note(
                        Tone::Bad,
                        format!("The {name} comes down, and the head of your axe cracks in two."),
                    );
                } else {
                    self.note(Tone::Good, format!("The {name} comes crashing down."));
                }
            }
            Verb::PickUp => {
                let mut taken = Vec::new();
                let carried = self.state.player.carried().clone();
                let Some(pile) = self.state.world.pile_mut(id) else {
                    return false;
                };
                let mut load = carried;
                for stack in pile.items().stacks().to_vec() {
                    let count = stack.count.min(load.room_for(stack.item));
                    let stack = pile.items_mut().take(stack.item, count);
                    if stack.count > 0 {
                        load.add_stack(stack);
                        taken.push(counted(stack.item, stack.count));
                    }
                }
                if pile.items().is_empty() {
                    self.state.world.remove_thing(id);
                }
                *self.state.player.carried_mut() = load;
                self.stale = true;
                self.note(Tone::Plain, format!("You pick up {}.", taken.join(", ")));
            }
            Verb::AddStick | Verb::AddLog => {
                let fuel = if verb == Verb::AddStick {
                    Item::Stick
                } else {
                    Item::Log
                };
                self.use_up(fuel, 1);
                if let Some(fire) = self.state.world.structure_mut(id) {
                    fire.stoke(u64::from(fuel.burn_time()), now);
                }
                self.stale = true;
                self.note(
                    Tone::Plain,
                    format!("You put a {} on the fire.", fuel.name().to_lowercase()),
                );
            }
            Verb::Stoke => {
                for fuel in [Item::Log, Item::Stick] {
                    let burn_time = u64::from(fuel.burn_time());
                    while self.has(fuel) > 0 {
                        let Some(fire) = self.state.world.structure_mut(id) else {
                            return false;
                        };
                        if fire.room(now) < burn_time {
                            break;
                        }
                        fire.stoke(burn_time, now);
                        self.use_up(fuel, 1);
                        if self.stale {
                            self.look();
                        }
                    }
                }

                self.stale = true;
                let left = self
                    .state
                    .world
                    .structure(id)
                    .map_or(0, |fire| fire.until().seconds_since(now));
                self.note(
                    Tone::Plain,
                    format!(
                        "You bank up the fire. It will burn for about {} hours.",
                        left.div_ceil(SECONDS_PER_HOUR)
                    ),
                );
            }
            Verb::Light => {
                self.use_up(Item::BirchBark, 1);
                if !self.fire_catches() {
                    self.note(
                        Tone::Bad,
                        "The tinder burns out before the kindling catches.",
                    );
                    return false;
                }

                self.use_up(Item::Stick, KINDLING);
                if let Some(fire) = self.state.world.structure_mut(id) {
                    fire.set_kind(StructureKind::Campfire);
                    fire.stoke(u64::from(KINDLING * Item::Stick.burn_time()), now);
                }
                self.stale = true;
                self.note(Tone::Good, "The fire burns again.");
            }
            Verb::Collect => return self.collect(id, kind),
            Verb::HangFood => {
                let carried = self.state.player.carried_mut();
                let hung = [Item::RawMeat, Item::RawFish].map(|item| carried.take(item, u32::MAX));
                if let Some(rack) = self.state.world.structure_mut(id) {
                    for stack in hung {
                        rack.hold_mut().add_stack(stack);
                    }
                    rack.set_since(now);
                }
                self.note(Tone::Plain, "You hang the raw food on the rack to dry.");
            }
            Verb::TakeDown => {
                let Some(structure) = self.state.world.structure(id) else {
                    return false;
                };
                let hold = structure.hold().stacks().to_vec();
                let parts: &[Item] = match structure.kind() {
                    StructureKind::FishTrap => &[Item::FishTrap],
                    _ => &[Item::Snare, Item::Stick],
                };
                self.state.world.remove_thing(id);
                self.stale = true;
                for stack in hold {
                    self.give(Stack::new(stack.item, stack.count));
                }
                for part in parts {
                    self.give(Stack::new(*part, 1));
                }
                self.note(Tone::Plain, format!("You take down the {name}."));
            }
            Verb::Drink | Verb::FillWater => return self.finish_at_water(verb, true),
            Verb::Deposit => return self.deposit(),
            Verb::Build => return self.build(id),
            Verb::Fish
            | Verb::Talk
            | Verb::Gift
            | Verb::Invite
            | Verb::Court
            | Verb::TryForChild
            | Verb::DigClay
            | Verb::RakeOre
            | Verb::Hire => return false,
        }
        true
    }

    /// Takes what a trap has caught, or what hangs on a rack.
    fn collect(&mut self, id: ThingId, kind: Kind) -> bool {
        let now = self.state.time;
        let Some(structure) = self.state.world.structure_mut(id) else {
            return false;
        };
        let dried = now.seconds_since(structure.since()) >= DRYING_TIME;
        let hold = std::mem::take(structure.hold_mut());
        structure.set_since(now);

        let words = match kind {
            Kind::Structure(StructureKind::SprungSnare) => {
                structure.set_kind(StructureKind::Snare);
                self.stale = true;
                "You take a hare out of the snare and set it again."
            }
            Kind::Structure(StructureKind::FishTrap) => "You empty the fish trap.",
            _ if dried => "You take the dried food off the rack.",
            _ => "You take the food off the rack before it has dried.",
        };

        for stack in hold.stacks() {
            let stack = match (kind, stack.item) {
                (Kind::Structure(StructureKind::DryingRack), Item::RawMeat) if dried => {
                    Stack::new(Item::DriedMeat, stack.count)
                }
                (Kind::Structure(StructureKind::DryingRack), Item::RawFish) if dried => {
                    Stack::new(Item::DriedFish, stack.count)
                }
                (Kind::Structure(StructureKind::FishTrap), item) => Stack::new(item, stack.count),
                _ => *stack,
            };
            self.give(stack);
        }
        self.note(Tone::Good, words);
        true
    }

    /// Brings about what `verb` done at water comes to, the water being `clean` if it is
    /// that of a well.
    fn finish_at_water(&mut self, verb: Verb, clean: bool) -> bool {
        match verb {
            Verb::Drink if clean => {
                let body = self.state.player.body_mut();
                body.drink(body.thirst());
                self.note(Tone::Plain, "You drink your fill from the well.");
            }
            Verb::FillWater if clean => {
                let pots = self.empty_pots();
                self.state.player.carried_mut().add(Item::BoiledWater, pots);
                self.note(Tone::Plain, "You fill your pots from the well.");
            }
            Verb::Drink => {
                let body = self.state.player.body_mut();
                body.drink(body.thirst());
                self.note(Tone::Plain, "You drink your fill from the lake.");
                let risk = Item::Water.food().map_or(0, |food| food.risk);
                self.risk_sickness(risk, "The water does not sit well. You are sick.");
            }
            Verb::FillWater => {
                let pots = self.empty_pots();
                self.state.player.carried_mut().add(Item::Water, pots);
                self.note(Tone::Plain, "You fill your pots from the lake.");
            }
            Verb::DigClay => {
                self.give(Stack::new(Item::Clay, 2));
                self.note(Tone::Plain, "You dig grey clay out of the shore.");
            }
            Verb::RakeOre => {
                self.give(Stack::new(Item::BogOre, 2));
                self.note(
                    Tone::Plain,
                    "You rake rusty lumps of ore out of the shallows.",
                );
            }
            Verb::Fish => {
                let level = i64::from(self.state.player.skills().level(Skill::Fishing));
                let mut chance = BITE_CHANCE + BITE_CHANCE_PER_LEVEL * level;
                if self.conditions.light == crate::climate::Light::Twilight {
                    chance += 15;
                }
                if self.conditions.frozen {
                    chance -= 10;
                }

                if self.state.rng.chance(chance.clamp(5, 95) as u64) {
                    self.give(Stack::new(Item::RawFish, 1));
                    self.note(Tone::Good, "You land a perch.");
                } else {
                    self.note(Tone::Plain, "Nothing bites.");
                }
            }
            _ => return false,
        }
        true
    }

    fn fire_catches(&mut self) -> bool {
        let level = i64::from(self.state.player.skills().level(Skill::Firemaking));
        let mut chance = FIRE_CHANCE + FIRE_CHANCE_PER_LEVEL * level;
        if self.at_hand.shelter.is_none() {
            chance -= match self.conditions.sky {
                Sky::Clear | Sky::Overcast => 0,
                Sky::Snow => 20,
                Sky::Rain => 40,
            };
            if self.conditions.wind == Wind::Windy {
                chance -= 15;
            }
        }

        self.state.rng.chance(chance.clamp(10, 98) as u64)
    }

    fn finish_craft(&mut self, recipe: Recipe) -> bool {
        let now = self.state.time;

        if recipe == Recipe::Campfire && !self.fire_catches() {
            self.use_up(Item::BirchBark, 1);
            self.note(
                Tone::Bad,
                "The tinder burns out before the kindling catches.",
            );
            return false;
        }

        let site = self.site();
        for (item, count) in recipe.materials() {
            self.use_up(*item, *count);
        }
        match recipe.product() {
            Product::Items(items) => {
                for (item, count) in items {
                    self.give(Stack::new(*item, *count));
                }
            }
            Product::Structure(kind) => {
                let Some(site) = site else {
                    return false;
                };
                let mut structure = Structure::new(Object::new(site, now), kind);
                if kind == StructureKind::Campfire {
                    let sticks = recipe
                        .materials()
                        .iter()
                        .find(|(item, _)| *item == Item::Stick)
                        .map_or(0, |(_, count)| *count);
                    structure.stoke(u64::from(sticks * Item::Stick.burn_time()), now);
                }
                let id = self.add_thing(Thing::Structure(structure));
                if kind == StructureKind::Cairn {
                    self.note(Tone::Good, recipe.done());
                    self.found(id);
                    return true;
                }
            }
        }

        self.note(Tone::Good, recipe.done());
        true
    }

    /// Hands `stack` to the player, who puts down what of it they cannot carry.
    pub(super) fn give(&mut self, stack: Stack) {
        let carried = self.state.player.carried_mut();
        let fits = stack.count.min(carried.room_for(stack.item));

        carried.add_stack(Stack {
            count: fits,
            ..stack
        });
        if fits < stack.count {
            self.put_down(Stack {
                count: stack.count - fits,
                ..stack
            });
        }
    }

    /// Puts `stack` on the nearest pile within reach, or starts a pile where the player
    /// stands if there is none.
    fn put_down(&mut self, stack: Stack) {
        let pile = self.at_hand.piles.first().copied();
        if let Some(pile) = pile.and_then(|id| self.state.world.pile_mut(id)) {
            pile.items_mut().add_stack(stack);
            self.stale = true;
        } else {
            let at = self.state.player.location();
            let pile = Pile::new(Object::new(at, self.state.time), [stack]);
            self.add_thing(Thing::Pile(pile));
            // Looked at now, so that the next thing put down in the same breath joins it.
            self.look();
        }
    }

    /// Uses up `count` of `item`, of those carried first and then of those lying within
    /// reach.
    pub(super) fn use_up(&mut self, item: Item, count: u32) {
        let left = count - self.state.player.carried_mut().take(item, count).count;
        self.take_loose(item, left);
    }

    /// Takes up to `count` of `item` from what lies within reach, from the piles first
    /// and then from the common store if that is within reach, and returns what was had.
    pub(super) fn take_loose(&mut self, item: Item, count: u32) -> Stack {
        let mut taken = Stack {
            count: 0,
            ..Stack::new(item, 0)
        };
        let add = |taken: &mut Stack, stack: Stack| {
            if stack.count > 0 {
                taken.fresh = if taken.count == 0 {
                    stack.fresh
                } else {
                    taken.fresh.min(stack.fresh)
                };
                taken.count += stack.count;
            }
        };

        for id in self.at_hand.piles.clone() {
            let Some(pile) = self.state.world.pile_mut(id) else {
                continue;
            };
            let stack = pile.items_mut().take(item, count - taken.count);
            add(&mut taken, stack);
            if pile.items().is_empty() {
                self.state.world.remove_thing(id);
            }
        }
        let before = taken.count;
        if let Some(store) = self.at_hand.store
            && let Some(cairn) = self.state.world.structure_mut(store)
        {
            let stack = cairn.hold_mut().take(item, count - taken.count);
            add(&mut taken, stack);
        }
        let out_of_store = Stack {
            count: taken.count - before,
            ..taken
        };

        if taken.count > 0 {
            self.stale = true;
            self.society_stale = true;
        }
        if out_of_store.count > 0 {
            self.credit(-Self::worth(&out_of_store));
        }
        taken
    }

    pub(super) fn add_thing(&mut self, thing: Thing) -> ThingId {
        let (at, footprint) = (thing.location(), thing.footprint());
        let id = self.state.world.add_thing(thing);

        self.grid.insert(id, at, footprint);
        self.kept.push(id);
        self.stale = true;
        id
    }

    /// Lets what happens by the minute happen: the weather turns, fires burn down, and the
    /// body uses itself up.
    pub(super) fn pass_minute(&mut self) {
        let now = self.state.time;
        self.turn_weather();
        self.burn_fires();

        let sleeping = self.state.player.activity() == Some(Activity::Sleeping);
        let effort = self.effort();
        let felt = self.felt();
        let bed = self.at_hand.bed || self.at_hand.shelter.is_some_and(StructureKind::is_house);
        let body = self.state.player.body_mut();
        let before = body.ailments();
        let death = body.pass_minute(effort, felt, bed, now);
        let after = body.ailments();

        if let Some(cause) = death {
            self.die(cause);
            return;
        }
        for ailment in after {
            if !before.contains(&ailment) {
                self.alert |= ailment.is_grave();
                self.note(Tone::Bad, ailment.onset());
            }
        }

        let body = self.state.player.body();
        if sleeping {
            if body.is_too_cold_to_sleep() {
                self.alert = true;
                self.wake("The cold wakes you.");
            } else if body.is_rested() {
                self.wake("You wake up rested.");
            }
        } else if body.must_sleep() {
            self.set_aside();
            self.state
                .player
                .animal_mut()
                .set_activity(Some(Activity::Sleeping));
            self.alert = true;
            self.note(Tone::Bad, "You collapse where you stand and fall asleep.");
        }

        self.follow_routine();
        if now.as_seconds().is_multiple_of(SECONDS_PER_HOUR) {
            self.pass_hour();
            self.move_people();
        }
        if now.as_seconds().is_multiple_of(SECONDS_PER_DAY) {
            self.pass_day();
        }
    }

    fn turn_weather(&mut self) {
        let before = self.conditions;
        let now = self.state.climate.conditions(self.state.time);
        self.conditions = now;

        if now.light != before.light {
            self.stale = true;
        }
        if now.sky != before.sky {
            match now.sky {
                Sky::Rain => self.note(Tone::Plain, "It starts to rain."),
                Sky::Snow => self.note(Tone::Plain, "It starts to snow."),
                Sky::Clear | Sky::Overcast => {
                    if matches!(before.sky, Sky::Rain) {
                        self.note(Tone::Plain, "The rain stops.");
                    } else if matches!(before.sky, Sky::Snow) {
                        self.note(Tone::Plain, "The snowfall stops.");
                    }
                }
            }
        }
        if now.snow != before.snow {
            self.note(
                Tone::Plain,
                if now.snow {
                    "Snow covers the ground."
                } else {
                    "The snow has melted."
                },
            );
        }
        if now.frozen != before.frozen {
            self.note(
                Tone::Plain,
                if now.frozen {
                    "The lake has frozen over."
                } else {
                    "The ice on the lake has broken up."
                },
            );
        }
    }

    fn burn_fires(&mut self) {
        let now = self.state.time;
        let mut burned_out = false;

        for id in &self.kept {
            if let Some(fire) = self.state.world.structure_mut(*id)
                && fire.kind() == StructureKind::Campfire
                && fire.until() <= now
            {
                fire.set_kind(StructureKind::Fireplace);
                burned_out = true;
            }
        }

        if burned_out {
            self.stale = true;
            if self.at_hand.fire {
                self.note(Tone::Bad, "The fire has burned out.");
            }
        }
    }

    /// Lets what happens by the hour happen: food goes off, and traps make their catches.
    fn pass_hour(&mut self) {
        let seconds = SECONDS_PER_HOUR as u32 * self.conditions.spoiling_percent() / 100;
        let trapping = u64::from(self.state.player.skills().level(Skill::Trapping));
        let fishing = u64::from(self.state.player.skills().level(Skill::Fishing));
        let frozen = self.conditions.frozen;
        let now = self.state.time;
        // How fast food goes off in the common store, in hundredths of how fast it does
        // in the open: half as fast in a storehouse, and half as fast again in jars.
        let stored = self.kept.iter().any(|id| {
            let structure = self.state.world.structure(*id);
            structure.is_some_and(|structure| structure.kind() == StructureKind::Storehouse)
        });
        let potted = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.knows(crate::settlement::Advance::Pottery));
        let keeping = if stored { 50 } else { 100 } * if potted { 50 } else { 100 } / 100;

        for stack in self.state.player.carried_mut().spoil(seconds) {
            self.note(
                Tone::Bad,
                format!("Your {} has gone off.", stack.item.name().to_lowercase()),
            );
        }

        self.kept.retain(|id| self.state.world.thing(*id).is_some());
        let mut emptied = Vec::new();
        for id in &self.kept {
            if let Some(pile) = self.state.world.pile_mut(*id) {
                if !pile.items_mut().spoil(seconds).is_empty() {
                    self.stale = true;
                }
                if pile.items().is_empty() {
                    emptied.push(*id);
                }
                continue;
            }
            let Some(structure) = self.state.world.structure_mut(*id) else {
                continue;
            };

            match structure.kind() {
                StructureKind::Snare => {
                    let chance = SNARE_CHANCE + SNARE_CHANCE_PER_LEVEL * trapping;
                    if self.state.rng.one_in(chance, 1_000) {
                        structure.set_kind(StructureKind::SprungSnare);
                        structure.set_since(now);
                        structure.hold_mut().add(Item::HareCarcass, 1);
                        self.stale = true;
                    }
                }
                StructureKind::SprungSnare => {
                    structure.hold_mut().spoil(seconds);
                }
                StructureKind::Cairn => {
                    structure.hold_mut().spoil(seconds * keeping / 100);
                }
                StructureKind::FishTrap => {
                    let chance = FISH_TRAP_CHANCE + FISH_TRAP_CHANCE_PER_LEVEL * fishing;
                    let room = structure.hold().count(Item::RawFish) < FISH_TRAP_HOLDS;
                    if !frozen && room && self.state.rng.one_in(chance, 1_000) {
                        structure.hold_mut().add(Item::RawFish, 1);
                    }
                }
                _ => {}
            }
        }
        for id in emptied {
            self.state.world.remove_thing(id);
        }
    }

    /// Lets what happens by the day happen: the others live their day, and the old may die.
    fn pass_day(&mut self) {
        self.live_a_day();

        let born = Date::from_timestamp(self.state.player.birth());
        let age = Date::from_timestamp(self.state.time).years_since(born);

        if age >= OLD_AGE {
            let years_past = age - OLD_AGE + 1;
            if self.state.rng.one_in(years_past * years_past, OLD_AGE_ODDS) {
                self.die(Cause::OldAge);
            }
        }
    }

    pub(super) fn die(&mut self, cause: Cause) {
        let player = &mut self.state.player;
        player.body_mut().die();
        player.animal_mut().set_activity(None);
        player.set_intent(None);
        self.route.clear();

        self.state.death = Some(Death {
            time: self.state.time,
            cause,
        });
        self.alert = true;
        self.note(Tone::Bad, format!("You have died of {}.", cause.name()));
        self.record_death();
    }
}

/// Returns the words for `count` of `item`, such as "3 × stick".
fn counted(item: Item, count: u32) -> String {
    let name = item.name().to_lowercase();
    if count == 1 {
        name
    } else {
        format!("{count} × {name}")
    }
}

#[cfg(test)]
mod tests;
