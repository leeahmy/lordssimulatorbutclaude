//! Life among other people: getting to know them, living together, and what a settlement
//! does with its days.

use std::sync::Arc;

use crate::action::{Job, Verb};
use crate::body::Ailment;
use crate::item::{Inventory, Item, Stack};
use crate::journal::Tone;
use crate::object::{Human, Living, Name, Object, Sex};
use crate::people::{ADULT_AGE, DEVOTED, FRIENDLY, Person, PersonId, Role, Trait, given_name};
use crate::politics::{Law, inherited_views};
use crate::settlement::{Advance, BUILDINGS, Settlement, building, houses, places};
use crate::space::{Length, Location};
use crate::time::Timestamp;
use crate::time::gregorian::{Date, SECONDS_PER_DAY, TimeOfDay};
use crate::view::{PersonView, SettlementView, SiteView};
use crate::world::{Structure, StructureKind, Subject, Thing, ThingId};

use super::Simulation;

/// How much a first talk of the day, any further talk, and a gift of food add to someone's
/// regard.
const TALK_WARMTH: u8 = 6;
const SMALL_TALK_WARMTH: u8 = 1;
const GIFT_WARMTH: u8 = 10;

/// How many days longer a wanderer stays for each day they are talked with.
const LINGERING_DAYS: u64 = 3;

/// How long a child is carried, and how long after a birth before the next is conceived.
const PREGNANCY_DAYS: u64 = 280;
const RECOVERY_DAYS: u64 = 300;

/// The chance in a hundred that trying for a child leads to one.
const CONCEPTION_CHANCE: u64 = 30;

/// The kilocalories a grown-up and a child eat in a day.
const ADULT_FARE: u32 = 2_500;
const CHILD_FARE: u32 = 1_500;

/// How many days without enough to eat before someone who is not family leaves, and before
/// anyone starves.
const DAYS_TO_LEAVE: u16 = 7;
const DAYS_TO_STARVE: u16 = 15;

/// How many wanderers camp in the forest at most, and the chance in a hundred that another
/// comes on a day there is room for one.
const MOST_WANDERERS: usize = 2;
const WANDERER_CHANCE: u64 = 4;

/// The size of a band at which half as many of those who pass through ask to stay as
/// would with nobody there, and how many come to settle in a hundred thousand days for
/// each one who lives in a settlement already: one or two in a hundred in a year.
const SMALL_BAND: u64 = 12;

/// The number of people beyond which no more come to settle for there being more.
const LARGE_TOWN: usize = 1_500;
const SETTLERS_PER_HEAD: u64 = 3;

/// How many people a settlement has before what happens to those the player character
/// is not close to is told by the month and not one by one.
pub(super) const CROWD: usize = 40;

/// The contentment below which people leave for somewhere better.
const RESTLESS: u8 = 25;

/// How far from home wanderers make camp, in millimeters, and for how many days.
const CAMP_DISTANCE: (i64, i64) = (70_000, 160_000);
const CAMP_DAYS: (i64, i64) = (8, 20);

/// How many people a settlement must have before others come asking to join it, and how
/// many days of food it must have in store.
const DRAW_PEOPLE: usize = 3;
const DRAW_FOOD_DAYS: u32 = 10;

/// How far apart the buildings of a settlement stand, in millimeters, how many plots
/// out from the cairn it reaches, and how many plots out its outskirts begin.
const PLOT: i64 = 9_000;
const REACH: i64 = 16;
const OUTSKIRTS: i64 = 5;

/// How many things stand built before new houses go up in the place of old ones.
const BUILT_UP: usize = 100;

/// The chance in a hundred each month that a sickness sweeps through a settlement of
/// [`CROWD`] people or more that knows nothing of vaccination, and the chance in a
/// hundred that it takes each one it reaches.
const EPIDEMIC_CHANCE: u64 = 2;
const EPIDEMIC_TOLL: u64 = 6;

/// The chance in a thousand that someone left out in the frost dies of it in a day.
const EXPOSURE_RISK: u64 = 20;

/// The chance in ten thousand that someone dies of a sickness in a day.
const SICKNESS_RISK: u64 = 2;

/// Returns whether the player character keeps `item` on them when they put the rest of
/// their load in the common store: tools, clothes, and water.
fn kept_on(item: Item) -> bool {
    item.garment().is_some()
        || item.axe().is_some()
        || matches!(
            item,
            Item::Knife
                | Item::FireSteel
                | Item::Pickaxe
                | Item::Saw
                | Item::BoneNeedle
                | Item::FishingRod
                | Item::BarkPot
                | Item::ClayPot
                | Item::Medicine
                | Item::Water
                | Item::BoiledWater
        )
}

impl Simulation {
    /// Makes sure the player character is one of the people of the world, so that others
    /// can be their partner and their children.
    pub(super) fn ensure_protagonist(&mut self) {
        if self.state.world.protagonist().is_some() {
            return;
        }

        let player = &self.state.player;
        let mut record = Person::new(
            player.name().clone(),
            player.sex(),
            player.birth(),
            player.location(),
        );
        record.member = true;
        let id = self.state.world.add_person(record);
        self.state.world.set_protagonist(id);
    }

    /// Returns whether the player knows of `subject`, and so can set out for it.
    pub(super) fn knows(&self, subject: Subject) -> bool {
        match subject {
            Subject::Person(_) => self.state.world.kind(subject).is_some(),
            Subject::Thing(_) | Subject::Area(_) => self.state.memory.knows(subject),
        }
    }

    fn me(&self) -> Option<&Person> {
        let world = &self.state.world;
        world.protagonist().and_then(|id| world.person(id))
    }

    /// Returns why the player cannot do `verb` with the person as things are, if they
    /// cannot.
    pub(super) fn check_person(&self, verb: Verb, id: PersonId) -> Result<(), String> {
        let now = self.state.time;
        let world = &self.state.world;
        let Some(person) = world.person(id).filter(|person| person.is_present()) else {
            return Err("They are no longer here.".to_owned());
        };
        let Some(my_id) = world.protagonist() else {
            return Ok(());
        };
        let Some(me) = self.me() else {
            return Ok(());
        };
        let first = person.name.first();

        match verb {
            Verb::Gift if self.best_food(self.state.player.carried(), false).is_none() => {
                Err("You have no food to give.".to_owned())
            }
            Verb::Invite if person.member => Err(format!("{first} lives with you already.")),
            Verb::Invite if person.regard < FRIENDLY => {
                Err(format!("{first} does not know you well enough to stay."))
            }
            Verb::Court => {
                if me.partner.is_some() {
                    Err("You share your life with someone already.".to_owned())
                } else if person.partner.is_some() {
                    Err(format!("{first} shares their life with someone already."))
                } else if !person.is_adult(now) || self.age() < ADULT_AGE {
                    Err("That is for grown-ups.".to_owned())
                } else if me.is_close_kin(my_id, person, id) {
                    Err(format!("{first} is your close kin."))
                } else if person.regard < DEVOTED {
                    Err(format!("{first} is not that fond of you yet."))
                } else {
                    Ok(())
                }
            }
            Verb::TryForChild => {
                let (mother, _) = if person.sex == Sex::Female {
                    (person, me)
                } else {
                    (me, person)
                };
                let recovered = mother.last_birth.is_none_or(|birth| {
                    now.seconds_since(birth) >= RECOVERY_DAYS * SECONDS_PER_DAY
                });

                if me.partner != Some(id) {
                    Err(format!("{first} is not your partner."))
                } else if person.sex == me.sex {
                    Err("The two of you cannot have a child together.".to_owned())
                } else if mother.expecting.is_some() {
                    Err("A child is on the way already.".to_owned())
                } else if !mother.is_fertile(now) || !recovered {
                    Err("This is not the time for a child.".to_owned())
                } else if self.at_hand.shelter.is_none() {
                    Err("You need a roof over your heads for that.".to_owned())
                } else {
                    Ok(())
                }
            }
            Verb::Hire => self.check_hire(id),
            _ => Ok(()),
        }
    }

    /// Brings about what `verb` done with the person comes to. Returns whether it came to
    /// what it was meant to.
    pub(super) fn finish_with_person(&mut self, verb: Verb, id: PersonId) -> bool {
        let now = self.state.time;
        let today = now.as_seconds() / SECONDS_PER_DAY;
        let my_id = self.state.world.protagonist();
        let my_sex = self.state.player.sex();
        let Some(person) = self.state.world.person_mut(id) else {
            return false;
        };
        let name = person.name.full();
        let first = person.name.first().to_owned();
        let was_friendly = person.regard >= FRIENDLY;
        self.society_stale = true;

        match verb {
            Verb::Hire => return self.finish_hire(id),
            Verb::Talk => {
                let met = person.talked_on.is_some();
                if person.talked_on == Some(today) {
                    person.warm_to(SMALL_TALK_WARMTH);
                } else {
                    person.warm_to(TALK_WARMTH);
                    person.talked_on = Some(today);
                    if let Some(leaves) = person.leaves {
                        let later = leaves.as_seconds() + LINGERING_DAYS * SECONDS_PER_DAY;
                        person.leaves = Some(Timestamp::from_seconds(later));
                    }
                }
                let friendly = person.regard >= FRIENDLY;
                let age = person.age(now);

                if met {
                    self.note(Tone::Plain, format!("You talk with {first} for a while."));
                } else {
                    self.note(Tone::Good, format!("You meet {name}, who is {age}."));
                }
                if friendly && !was_friendly {
                    self.note(Tone::Good, format!("{first} has come to trust you."));
                }
            }
            Verb::Gift => {
                let carried = self.state.player.carried().clone();
                let Some(food) = self.best_food(&carried, false) else {
                    return false;
                };
                self.state.player.carried_mut().take(food, 1);
                if let Some(person) = self.state.world.person_mut(id) {
                    person.warm_to(GIFT_WARMTH);
                    person.hungry_days = 0;
                }
                self.note(
                    Tone::Plain,
                    format!("You give {first} {}.", food.name().to_lowercase()),
                );
            }
            Verb::Invite => {
                person.member = true;
                person.leaves = None;
                let words = match &self.state.settlement {
                    Some(settlement) => format!("{first} has joined {}.", settlement.name),
                    None => format!("{first} will stay with you from now on."),
                };
                self.note(Tone::Good, words);
            }
            Verb::Court => {
                person.partner = my_id;
                person.member = true;
                person.leaves = None;
                if let Some(me) = my_id.and_then(|me| self.state.world.person_mut(me)) {
                    me.partner = Some(id);
                }
                self.note(
                    Tone::Good,
                    format!("{first} will share your life. You are partners now."),
                );
            }
            Verb::TryForChild => {
                if !self.state.rng.chance(CONCEPTION_CHANCE) {
                    self.note(Tone::Plain, format!("You spend the night with {first}."));
                    return true;
                }

                let due =
                    Timestamp::from_seconds(now.as_seconds() + PREGNANCY_DAYS * SECONDS_PER_DAY);
                let mother = if my_sex == Sex::Female {
                    my_id
                } else {
                    Some(id)
                };
                if let Some(mother) = mother.and_then(|id| self.state.world.person_mut(id)) {
                    mother.expecting = Some(due);
                }
                if my_sex == Sex::Female {
                    self.note(Tone::Good, "You are with child.");
                } else {
                    self.note(Tone::Good, format!("{first} is with child."));
                }
            }
            _ => return false,
        }

        self.stale = true;
        true
    }

    /// Returns the age of the player character in full years.
    fn age(&self) -> u64 {
        let born = Date::from_timestamp(self.state.player.birth());
        Date::from_timestamp(self.state.time).years_since(born)
    }

    /// Returns the most filling food in `inventory`, leaving out what may make the eater
    /// sick unless `any` will do.
    fn best_food(&self, inventory: &Inventory, any: bool) -> Option<Item> {
        inventory
            .stacks()
            .iter()
            .filter_map(|stack| Some((stack.item, stack.item.food()?)))
            .filter(|(item, food)| !item.is_drink() && food.energy > 0 && (any || food.risk == 0))
            .max_by_key(|(_, food)| food.energy)
            .map(|(item, _)| item)
    }

    /// Returns the common store of the settlement.
    pub(super) fn store_mut(&mut self) -> Option<&mut Inventory> {
        let cairn = self.state.settlement.as_ref()?.cairn;
        self.state
            .world
            .structure_mut(cairn)
            .map(Structure::hold_mut)
    }

    pub(super) fn store(&self) -> Option<&Inventory> {
        let cairn = self.state.settlement.as_ref()?.cairn;
        self.state.world.structure(cairn).map(Structure::hold)
    }

    /// Puts `count` of `item` in the common store.
    pub(super) fn stock(&mut self, item: Item, count: u32) {
        if let Some(store) = self.store_mut() {
            store.add(item, count);
        }
    }

    /// Returns the kinds of everything that stands built.
    pub(super) fn structures(&self) -> Vec<(ThingId, StructureKind)> {
        self.kept
            .iter()
            .filter_map(|id| Some((*id, self.state.world.structure(*id)?.kind())))
            .collect()
    }

    pub(super) fn members(&self) -> Vec<PersonId> {
        self.state
            .world
            .people()
            .filter(|(_, person)| person.member)
            .map(|(id, _)| id)
            .collect()
    }

    /// Founds the settlement around the cairn just raised.
    pub(super) fn found(&mut self, cairn: ThingId) {
        let family = self.state.player.name().last().to_owned();
        let name = if family.is_empty() {
            "The Stead".to_owned()
        } else {
            format!("{family}stead")
        };

        self.note(
            Tone::Good,
            format!("You name the place {name}. Those who join you will live here."),
        );
        let mut settlement = Settlement::new(name, cairn, self.state.time);
        settlement.leader = self.state.world.protagonist();
        settlement.moot_year = Date::from_timestamp(self.state.time).year();
        self.state.settlement = Some(settlement);
        self.society_stale = true;
        let founder = self.state.player.name().full();
        self.chronicle(format!("{founder} raised a cairn and founded the place."));
    }

    pub(super) fn rename_settlement(&mut self, name: String) {
        let name = name.trim();
        if let Some(settlement) = &mut self.state.settlement
            && !name.is_empty()
        {
            settlement.name = name.chars().take(40).collect();
            self.society_stale = true;
        }
    }

    /// Puts everything the player carries but tools, clothes, and water in the store.
    pub(super) fn deposit(&mut self) -> bool {
        let carried = self.state.player.carried_mut();
        let load: Vec<Stack> = carried
            .stacks()
            .iter()
            .filter(|stack| !kept_on(stack.item))
            .copied()
            .collect();
        for stack in &load {
            carried.take(stack.item, stack.count);
        }

        let Some(store) = self.store_mut() else {
            return false;
        };
        for stack in &load {
            store.add_stack(*stack);
        }
        self.stale = true;
        self.society_stale = true;
        if !self.monetized() {
            self.note(Tone::Plain, "You put your load in the common store.");
            self.credit(load.iter().map(Self::worth).sum());
            return true;
        }

        // Where there is coin, the store buys what it is brought, as far as the
        // treasury goes. What it cannot pay for is remembered.
        let asked: i64 = load.iter().map(Self::selling_price).sum();
        let paid = self.pay(asked);
        if paid < asked {
            let unpaid: i64 = load.iter().map(Self::worth).sum();
            self.credit(unpaid * (asked - paid) / asked.max(1));
        }
        self.note(
            Tone::Plain,
            format!("You sell your load to the store for {paid} coins."),
        );
        true
    }

    /// Returns why the player has nothing to put in the store, if they have not.
    pub(super) fn check_deposit(&self) -> Result<(), String> {
        let carried = self.state.player.carried();
        let any = carried.stacks().iter().any(|stack| !kept_on(stack.item));

        if self.state.settlement.is_none() {
            Err("There is no settlement to keep a store for.".to_owned())
        } else if any {
            Ok(())
        } else {
            Err("You carry nothing to put in the store.".to_owned())
        }
    }

    /// Takes up to `count` of `item` from what lies within reach, the common store
    /// included, for the player to carry.
    pub(super) fn take(&mut self, item: Item, count: u32) {
        let room = self.state.player.carried().room_for(item);
        let count = count.min(room).min(self.at_hand.loose.count(item));
        if count == 0 {
            self.note(Tone::Plain, "You cannot carry any more.");
            return;
        }
        // What lies in piles is taken first, and is free. The rest is bought.
        let stored = self.store().map_or(0, |store| store.count(item));
        let piled = self.at_hand.loose.count(item).saturating_sub(stored);
        let cost = Self::buying_price(item, count.saturating_sub(piled));
        if self.monetized() && self.at_hand.store.is_some() && cost > self.state.purse.max(0) {
            self.note(
                Tone::Plain,
                format!(
                    "That costs {cost} coins at the market, and you have {}.",
                    self.state.purse
                ),
            );
            return;
        }

        let taken = self.take_loose(item, count);
        self.state.player.carried_mut().add_stack(taken);
        self.society_stale = true;
        self.note(
            Tone::Plain,
            format!("You take {} × {}.", taken.count, item.name().to_lowercase()),
        );
    }

    /// Puts an hour of work into the building site. Returns whether the hour was put in.
    pub(super) fn build(&mut self, id: ThingId) -> bool {
        let Some(site) = self.state.world.structure_mut(id) else {
            return false;
        };
        let Some(plan) = site.plan().and_then(building) else {
            return false;
        };

        if site.build(1, plan.hours) {
            self.finished_building(plan.kind);
        } else {
            self.note(Tone::Plain, "You work on the building.");
        }
        // An hour's building is paid a coin where there is coin, and remembered where
        // there is none.
        if !self.monetized() || self.pay(1) == 0 {
            self.credit_work();
        }
        self.society_stale = true;
        true
    }

    pub(super) fn finished_building(&mut self, kind: StructureKind) {
        let name = crate::world::Kind::Structure(kind).name().to_lowercase();
        self.stale = true;
        self.note(Tone::Good, format!("The {name} is finished."));
        // The first of anything is remembered.
        if self.built().iter().filter(|built| **built == kind).count() == 1 {
            self.chronicle(format!("The first {name} was built."));
        }
    }

    /// Plans a building of `kind`, if the player has the say to.
    pub(super) fn plan(&mut self, kind: StructureKind) {
        let planned = self.authority().and_then(|()| self.mark_out(kind));
        if let Err(reason) = planned {
            self.note(Tone::Plain, reason);
        }
    }

    /// Takes what a building of `kind` is made of out of the store and marks out a site
    /// for it by the cairn. Returns why not, if that cannot be done.
    pub(super) fn mark_out(&mut self, kind: StructureKind) -> Result<(), String> {
        let Some(plan) = building(kind) else {
            return Err("That is not something a settlement builds.".to_owned());
        };
        let Some(settlement) = &self.state.settlement else {
            return Err("There is no settlement to build for.".to_owned());
        };
        if let Some(needed) = plan.needs.filter(|needed| !settlement.knows(*needed)) {
            return Err(format!(
                "Nobody knows {} yet.",
                needed.name().to_lowercase()
            ));
        }

        let store = self.store().cloned().unwrap_or_default();
        let lacking: Vec<String> = plan
            .materials
            .iter()
            .filter(|(item, count)| store.count(*item) < *count)
            .map(|(item, count)| {
                let short = count - store.count(*item);
                format!("{short} × {}", item.name().to_lowercase())
            })
            .collect();
        if !lacking.is_empty() {
            return Err(format!(
                "The store is short of {} for that.",
                lacking.join(", ")
            ));
        }
        let coins = if settlement.knows(Advance::Coinage) && !settlement.lives_by(Law::PublicWorks)
        {
            i64::from(plan.coins)
        } else {
            0
        };
        if coins > settlement.treasury {
            return Err(format!(
                "That costs {coins} coins, and the treasury holds {}.",
                settlement.treasury.max(0)
            ));
        }
        // Once a place is built up, a bigger house goes where the smallest stood.
        let replaced = self.lesser_house(plan.houses);
        let plot = self.free_plot(plan.outskirts);
        if replaced.is_none() && plot.is_none() {
            return Err("There is no room left to build on.".to_owned());
        }

        if let Some(store) = self.store_mut() {
            for (item, count) in plan.materials {
                store.take(*item, *count);
            }
        }
        if let Some(settlement) = &mut self.state.settlement {
            settlement.treasury -= coins;
        }
        if let Some(old) = replaced.and_then(|id| self.state.world.structure_mut(id)) {
            old.rebuild(kind);
            self.stale = true;
        } else if let Some(plot) = plot {
            let site = Structure::site(Object::new(plot, self.state.time), kind);
            self.add_thing(Thing::Structure(site));
        }
        self.society_stale = true;
        let name = crate::world::Kind::Structure(kind).name().to_lowercase();
        self.note(
            Tone::Plain,
            format!(
                "The site of a {name} is marked out. It takes {} hours of work.",
                plan.hours
            ),
        );
        Ok(())
    }

    /// Returns the smallest house there is that holds no more than half as many as a
    /// house for `people`, once so much stands that new houses are better put in the
    /// place of old ones.
    fn lesser_house(&self, people: u32) -> Option<ThingId> {
        let structures = self.structures();
        if people == 0 || structures.len() < BUILT_UP {
            return None;
        }

        structures
            .into_iter()
            .filter(|(_, kind)| building(*kind).is_some())
            .map(|(id, kind)| (houses(kind), id))
            .filter(|(holds, _)| *holds > 0 && holds * 2 <= people)
            .min()
            .map(|(_, id)| id)
    }

    /// Returns the nearest plot to the cairn that is dry and has nothing built on it, of
    /// those on the `outskirts` of the settlement if that is where it is to be.
    fn free_plot(&self, outskirts: bool) -> Option<Location> {
        let world = &self.state.world;
        let cairn = self.state.settlement.as_ref()?.cairn;
        let center = world.structure(cairn)?.location();
        let taken: std::collections::HashSet<(i64, i64)> = self
            .structures()
            .iter()
            .filter_map(|(id, _)| Some(world.structure(*id)?.location()))
            .map(|at| {
                let east = (at.x() - center.x() + PLOT / 2).div_euclid(PLOT);
                let north = (at.y() - center.y() + PLOT / 2).div_euclid(PLOT);
                (east, north)
            })
            .collect();

        let mut plots: Vec<(i64, i64)> = (-REACH..=REACH)
            .flat_map(|east| (-REACH..=REACH).map(move |north| (east, north)))
            .filter(|plot| *plot != (0, 0) && !taken.contains(plot))
            .collect();
        plots.sort_by_key(|(east, north)| {
            let away = east * east + north * north;
            let inside = away < OUTSKIRTS * OUTSKIRTS;
            (outskirts && inside, away, *north, *east)
        });

        plots
            .into_iter()
            .map(|(east, north)| Location::new(center.x() + east * PLOT, center.y() + north * PLOT))
            .find(|plot| !world.is_water(*plot))
    }

    pub(super) fn pursue(&mut self, advance: Advance) {
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };
        if !settlement.can_pursue(advance) || settlement.pursuit == Some(advance) {
            return;
        }
        // Anyone may set people to working something out while nothing is being.
        if settlement.pursuit.is_some()
            && let Err(reason) = self.authority()
        {
            self.note(Tone::Plain, reason);
            return;
        }
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };

        settlement.pursuit = Some(advance);
        settlement.insight = 0;
        self.society_stale = true;
        self.note(
            Tone::Plain,
            format!(
                "Your people set their minds to {}.",
                advance.name().to_lowercase()
            ),
        );
    }

    pub(super) fn assign(&mut self, id: PersonId, role: Role) {
        if self.state.settlement.is_some()
            && let Err(reason) = self.authority()
        {
            self.note(Tone::Plain, reason);
            return;
        }
        if let Some(person) = self.state.world.person_mut(id)
            && person.member
            && person.role != role
        {
            person.take_up(role);
            self.society_stale = true;
        }
    }

    /// Apprentices a child of the player character to the trade of `role`, which they
    /// learn until they are old enough to work at it.
    pub(super) fn apprentice(&mut self, id: PersonId, role: Role) {
        let now = self.state.time;
        let me = self.state.world.protagonist();
        let Some(child) = self.state.world.person_mut(id) else {
            return;
        };
        let first = child.name.first().to_owned();
        let refusal = if me.is_none() || !child.parents.contains(&me) {
            Some("Only your own child is yours to apprentice.".to_owned())
        } else if child.works(now) {
            Some(format!("{first} is old enough to work already."))
        } else if child.age(now) < crate::people::APPRENTICE_AGE {
            Some(format!("{first} is too young to learn a trade yet."))
        } else {
            None
        };
        if let Some(refusal) = refusal {
            self.note(Tone::Plain, refusal);
            return;
        }

        child.take_up(role);
        child.apprenticed = role != Role::Idle;
        self.society_stale = true;
        self.note(
            Tone::Good,
            format!(
                "{first} is to learn the trade of a {} until old enough to work at it.",
                role.name().to_lowercase()
            ),
        );
    }

    /// Writes the death of the player character into what is kept of them as one of the
    /// people, and leaves what they carried where they fell.
    pub(super) fn record_death(&mut self) {
        let now = self.state.time;
        let at = self.state.player.location();
        let my_id = self.state.world.protagonist();

        if let Some(me) = my_id.and_then(|id| self.state.world.person_mut(id)) {
            me.died = Some(now);
            me.location = at;
            let partner = me.partner.take();
            if let Some(partner) = partner.and_then(|id| self.state.world.person_mut(id)) {
                partner.partner = None;
            }
        }
        self.society_stale = true;
    }

    /// Has the player live on as one of those who lived with the character that died.
    pub(super) fn live_on(&mut self, id: PersonId) {
        let now = self.state.time;
        if self.state.death.is_none() {
            return;
        }
        let Some(heir) = self.state.world.person(id).cloned() else {
            return;
        };
        if !heir.is_present() || !heir.member || !heir.works(now) {
            return;
        }

        let fallen = std::mem::replace(
            &mut self.state.player,
            Human::new(heir.location, heir.born, heir.name.clone(), heir.sex),
        );
        // What the dead carried goes to the living: the tools to the one who lives on,
        // and the rest to where they fell.
        let mut left = Vec::new();
        for stack in fallen.carried().stacks() {
            if matches!(stack.item, Item::Knife | Item::FireSteel) {
                self.state.player.carried_mut().add_stack(*stack);
            } else {
                left.push(*stack);
            }
        }
        if !left.is_empty() {
            let pile = crate::world::Pile::new(Object::new(fallen.location(), now), left);
            self.add_thing(Thing::Pile(pile));
        }

        // What the dead had in their purse goes to their partner or their child, and to
        // the treasury if it is someone else who lives on.
        let dead = self.state.world.protagonist();
        let kin = dead.is_some() && (heir.partner == dead || heir.parents.contains(&dead));
        let purse = std::mem::take(&mut self.state.purse);
        if kin {
            self.state.purse = purse.max(0);
        }
        // A child has learned half of what their parent knew at their side.
        if heir.parents.contains(&dead) && dead.is_some() {
            for skill in crate::skill::Skill::ALL {
                let taught = fallen.skills().experience(skill) / 2;
                self.state.player.skills_mut().gain(skill, taught);
            }
        }
        self.state.world.set_protagonist(id);
        if let Some(settlement) = &mut self.state.settlement {
            // What the dead brought to the store and said by the cairn is theirs, not
            // their heir's.
            settlement.credit = 0;
            settlement.noted_credit = 0;
            settlement.appealed_on.clear();
            if !kin {
                settlement.treasury += purse.max(0);
            }
        }
        self.state.death = None;
        self.state.started = now;
        self.state.walked = 0;
        self.state.past_lives += 1;
        self.route.clear();
        self.stale = true;
        self.society_stale = true;

        let parent = heir
            .parents
            .iter()
            .flatten()
            .filter_map(|parent| self.state.world.person(*parent))
            .map(|parent| parent.name.first().to_owned())
            .next();
        let words = match parent {
            Some(parent) => format!(
                "Life goes on. You are {}, child of {parent}, {} years old.",
                heir.name.full(),
                heir.age(now)
            ),
            None => format!(
                "Life goes on. You are {}, {} years old.",
                heir.name.full(),
                heir.age(now)
            ),
        };
        self.note(Tone::Good, words);
    }

    /// Has the player character see to what their body needs, if they are doing nothing.
    pub(super) fn follow_routine(&mut self) {
        if !self.state.routine || self.state.player.activity().is_some() {
            return;
        }
        let body = self.state.player.body();
        let ailments = body.ailments();
        let any = |these: [Ailment; 2]| these.iter().any(|ailment| ailments.contains(ailment));
        let sleepy = !body.is_wide_awake() && self.conditions.light == crate::climate::Light::Dark;
        let can_sleep = !body.is_too_cold_to_sleep() && !body.is_wide_awake();

        if any([Ailment::Thirsty, Ailment::Parched]) && self.seek_water() {
            return;
        }
        if any([Ailment::Hungry, Ailment::Starving]) && self.seek_food() {
            return;
        }
        if (any([Ailment::Tired, Ailment::Exhausted]) || sleepy) && can_sleep {
            self.lie_down();
        }
    }

    fn seek_water(&mut self) -> bool {
        for water in [Item::BoiledWater, Item::Water] {
            if self.has(water) > 0 {
                self.take_up(Job::Consume(water));
                return true;
            }
        }

        let world = &self.state.world;
        let at = self.state.player.location();
        let wells = self
            .structures()
            .into_iter()
            .filter(|(_, kind)| *kind == StructureKind::Well)
            .map(|(id, _)| Subject::Thing(id));
        let lakes = world.areas().map(|(id, _)| Subject::Area(id));
        let nearest = wells
            .chain(lakes)
            .filter(|subject| self.knows(*subject))
            .filter_map(|subject| {
                let spot = world.approach(subject, at, Human::REACH)?;
                Some((at.distance_to(spot), subject))
            })
            .min()
            .map(|(_, subject)| subject);

        let drink = nearest.map(|subject| Job::Act {
            verb: Verb::Drink,
            subject,
        });
        match drink {
            Some(drink) if self.check(drink).is_ok() => self.take_up(drink),
            _ if self.check(Job::EatSnow).is_ok() => self.take_up(Job::EatSnow),
            _ => return false,
        }
        true
    }

    fn seek_food(&mut self) -> bool {
        let mut within_reach = self.state.player.carried().clone();
        for stack in self.at_hand.loose.stacks() {
            within_reach.add_stack(*stack);
        }
        let starving = self.state.player.body().energy_percent() == 0;
        let food = self
            .best_food(&within_reach, false)
            .or_else(|| self.best_food(&within_reach, starving));
        if let Some(food) = food {
            self.take_up(Job::Consume(food));
            return true;
        }

        // Home is where the food is, if there is any.
        let stocked = self
            .store()
            .is_some_and(|store| self.best_food(store, true).is_some());
        let cairn = self
            .state
            .settlement
            .as_ref()
            .map(|settlement| settlement.cairn);
        match cairn {
            Some(cairn) if stocked && self.at_hand.store.is_none() => {
                self.walk_to(Subject::Thing(cairn), None);
                self.state.player.activity().is_some()
            }
            _ => false,
        }
    }

    /// Moves the others to where they are at this hour: at work by day, and at home, or by
    /// the player if there is no home, for the rest of it.
    pub(super) fn move_people(&mut self) {
        let hour = TimeOfDay::from_timestamp(self.state.time).hour();
        let now = self.state.time;
        let at = self.state.player.location();
        let world = &self.state.world;
        let home = self
            .state
            .settlement
            .as_ref()
            .and_then(|settlement| world.structure(settlement.cairn))
            .map(Structure::location);
        let shore = home.and_then(|home| {
            let shores = world.areas().map(|(_, area)| area.nearest_shore(home));
            shores.min_by_key(|shore| home.distance_to(*shore))
        });

        // Where each trade that takes a workplace has its workplaces.
        let mut workplaces: Vec<Vec<Location>> = vec![Vec::new(); Role::ALL.len()];
        for (id, kind) in self.structures() {
            let jobs = building(kind).and_then(|plan| plan.jobs);
            if let (Some((role, _)), Some(structure)) = (jobs, world.structure(id)) {
                workplaces[role as usize].push(structure.location());
            }
        }

        let moves: Vec<(PersonId, Location)> = world
            .people()
            .filter(|(_, person)| person.member)
            .map(|(id, person)| {
                let turn = f64::from(id.0) * 2.4;
                let offset = |from: Location, away: f64| {
                    Location::new(
                        from.x() + (away * turn.cos()) as i64,
                        from.y() + (away * turn.sin()) as i64,
                    )
                };
                let Some(home) = home else {
                    return (id, offset(at, 1_500.0));
                };

                let at_home = offset(home, 4_000.0 + f64::from(id.0 % 5) * 1_500.0);
                let working = (8..18).contains(&hour) && person.works(now);
                let places = &workplaces[person.role as usize];
                let spot = match person.role {
                    _ if !working => at_home,
                    Role::Idle => at_home,
                    Role::Fisher => {
                        shore.map_or(at_home, |shore| shore.toward(home, Length::from_meters(1)))
                    }
                    Role::Forager | Role::Gatherer | Role::Trapper | Role::Woodcutter => {
                        offset(home, 35_000.0 + f64::from(id.0 % 7) * 5_000.0)
                    }
                    _ if !places.is_empty() => {
                        offset(places[id.0 as usize % places.len()], 2_500.0)
                    }
                    _ => offset(home, 12_000.0),
                };
                let spot = if world.is_water(spot) { at_home } else { spot };
                (id, spot)
            })
            .collect();

        for (id, spot) in moves {
            if let Some(person) = self.state.world.person_mut(id)
                && person.location != spot
            {
                person.location = spot;
                self.stale = true;
            }
        }
    }

    /// Lets what happens to the others by the day happen.
    pub(super) fn live_a_day(&mut self) {
        self.society_stale = true;
        self.births();
        self.wanderers();

        let members = self.members();
        self.world_day(&members);
        if self.state.settlement.is_some() {
            self.practise(&members);
            self.work_day(&members);
            self.feed(&members);
            self.winter(&members);
            self.pair_up(&members);
            self.politics_day();
            self.unrest(&members);
            self.tell_the_month(members.len());
        } else {
            self.feed_children(&members);
        }
        self.mortality(&members);
    }

    /// Has everyone who works get a month better at their trade on the first of the
    /// month.
    pub(super) fn practise(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        if Date::from_timestamp(now).day() != 1 {
            return;
        }
        for id in members {
            if let Some(person) = self.state.world.person_mut(*id)
                && (person.works(now) || person.apprenticed)
            {
                person.practise();
            }
        }
    }

    /// Returns whether the person is close enough to the player character for what
    /// happens to them to be told however many people there are.
    fn is_close(&self, id: PersonId) -> bool {
        let world = &self.state.world;
        let me = world.protagonist();
        let leader = self.state.settlement.as_ref().and_then(|s| s.leader);
        world.person(id).is_some_and(|person| {
            person.partner == me
                || person.parents.contains(&me)
                || person.regard >= DEVOTED
                || Some(id) == leader
                || me
                    .and_then(|me| world.person(me))
                    .is_some_and(|mine| mine.parents.contains(&Some(id)))
        })
    }

    /// Returns whether there are so many people that what happens to those the player
    /// character is not close to is told by the month.
    fn crowded(&self) -> bool {
        self.state.settlement.is_some() && self.members().len() >= CROWD
    }

    /// Counts one more who came to settle, was born, or died, in that order.
    fn count_towards_the_month(&mut self, which: usize) {
        if let Some(settlement) = &mut self.state.settlement {
            settlement.tally[which] += 1;
        }
    }

    pub(super) fn person_dies(&mut self, id: PersonId, of: &str) {
        let now = self.state.time;
        let told = !self.crowded() || self.is_close(id);
        let Some(person) = self.state.world.person_mut(id) else {
            return;
        };
        person.died = Some(now);
        let name = person.name.full();
        let member = person.member;
        let partner = person.partner.take();
        if let Some(partner) = partner.and_then(|id| self.state.world.person_mut(id)) {
            partner.partner = None;
        }

        self.stale = true;
        if member {
            self.count_towards_the_month(2);
        }
        if told {
            self.note(Tone::Bad, format!("{name} has died of {of}."));
        }
    }

    /// Tells on the first of the month how many came, were born, and died in the month
    /// before, where there are too many people to tell of one by one.
    fn tell_the_month(&mut self, members: usize) {
        let date = Date::from_timestamp(self.state.time);
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };
        if date.day() != 1 {
            return;
        }
        let [came, born, died] = std::mem::take(&mut settlement.tally);
        let place = settlement.name.clone();
        if members >= CROWD && came + born + died > 0 {
            self.note(
                Tone::Plain,
                format!(
                    "In the past month {came} settled in {place}, {born} were born, and \
                     {died} died. {} live there now.",
                    members + 1
                ),
            );
        }
    }

    /// Has someone leave for somewhere better when people are not content with their lot.
    fn unrest(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let me = self.state.world.protagonist();
        let restless = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.contentment < RESTLESS);
        if !restless || members.len() < 6 || !self.state.rng.chance(5) {
            return;
        }

        let pick = self.state.rng.below(members.len() as u64) as usize;
        let id = members[pick];
        let told = !self.crowded() || self.is_close(id);
        let Some(person) = self.state.world.person_mut(id) else {
            return;
        };
        let family = person.partner == me || person.parents.contains(&me);
        if family || !person.is_adult(now) {
            return;
        }
        person.gone = true;
        let name = person.name.full();
        let partner = person.partner.take();
        if let Some(partner) = partner.and_then(|id| self.state.world.person_mut(id)) {
            partner.partner = None;
        }
        self.stale = true;
        if told {
            self.note(
                Tone::Bad,
                format!("{name} has had enough, and has left to seek a better life elsewhere."),
            );
        }
    }

    fn births(&mut self) {
        let now = self.state.time;
        let world = &self.state.world;
        let me = world.protagonist();
        let alive = self.state.death.is_none();
        let due: Vec<PersonId> = world
            .everyone()
            .filter(|(id, person)| person.is_present() && (alive || Some(*id) != me))
            .filter(|(_, person)| person.expecting.is_some_and(|due| due <= now))
            .map(|(id, _)| id)
            .collect();

        for mother_id in due {
            let sex = if self.state.rng.chance(50) {
                Sex::Female
            } else {
                Sex::Male
            };
            let given = given_name(&mut self.state.rng, sex).to_owned();
            let Some(mother) = self.state.world.person_mut(mother_id) else {
                continue;
            };
            mother.expecting = None;
            mother.last_birth = Some(now);
            let mother = mother.clone();
            let father_id = mother.partner;
            let father = father_id
                .and_then(|id| self.state.world.person(id))
                .cloned();

            let mine = Some(mother_id) == me || (father_id.is_some() && father_id == me);
            let at = if Some(mother_id) == me {
                self.state.player.location()
            } else {
                mother.location
            };
            let family = father
                .as_ref()
                .map_or(mother.name.last(), |father| father.name.last());
            let mut child = Person::new(
                Name::new(given.clone(), Vec::new(), family.to_owned()),
                sex,
                now,
                at,
            );
            child.parents = [Some(mother_id), father_id];
            child.member = mother.member || mine;
            child.regard = if mine { 80 } else { 40 };
            let fathers = father.as_ref().map_or(mother.views, |father| father.views);
            child.views = inherited_views(&mut self.state.rng, [&mother.views, &fathers]);
            let mut inherited = mother.traits.clone();
            if let Some(father) = &father {
                inherited.extend(&father.traits);
            }
            child.traits = crate::people::random_traits(&mut self.state.rng, &inherited);
            child.role = Role::Idle;
            let told = mine || !self.crowded() || self.is_close(mother_id);
            self.state.world.add_person(child);
            if mother.member {
                self.count_towards_the_month(1);
            }

            let what = if sex == Sex::Female {
                "daughter"
            } else {
                "son"
            };
            let words = if Some(mother_id) == me {
                format!("You give birth to a {what}. You name the child {given}.")
            } else if mine {
                format!(
                    "{} gives birth to your {what}. You name the child {given}.",
                    mother.name.first()
                )
            } else {
                format!("A {what}, {given}, is born to {}.", mother.name.full())
            };
            if told {
                self.note(Tone::Good, words);
            }
            self.stale = true;
        }
    }

    fn wanderers(&mut self) {
        let now = self.state.time;
        let world = &self.state.world;

        let leaving: Vec<PersonId> = world
            .people()
            .filter(|(_, person)| !person.member)
            .filter(|(_, person)| person.leaves.is_some_and(|leaves| leaves <= now))
            .map(|(id, _)| id)
            .collect();
        for id in leaving {
            if let Some(person) = self.state.world.person_mut(id) {
                person.gone = true;
                let (name, met) = (person.name.first().to_owned(), person.talked_on.is_some());
                self.stale = true;
                if met {
                    self.note(Tone::Plain, format!("{name} has broken camp and moved on."));
                }
            }
        }

        let camping = self
            .state
            .world
            .people()
            .filter(|(_, person)| !person.member)
            .count();
        let members = self.members().len();
        let open = self.lives_by(Law::OpenGates);
        let (content, settled) = match &self.state.settlement {
            Some(settlement) => (settlement.contentment, true),
            None => (50, false),
        };
        let can_draw = settled
            && members >= DRAW_PEOPLE
            && !self.lives_by(Law::ClosedGates)
            && content >= RESTLESS
            && self.food_days(members) >= DRAW_FOOD_DAYS
            && (open || (members as u32) < self.housing());
        // The more people there are, and the easier it is to get there, the more hear
        // of the place and come to settle: so many in a hundred thousand days for each
        // one who lives there already.
        let mut stream = members.min(LARGE_TOWN) as u64 * SETTLERS_PER_HEAD;
        if open {
            stream *= 2;
        }
        if self.lives_by(Law::FreePort) {
            stream = stream * 3 / 2;
        }
        if self.has_built(StructureKind::RailwayStation) {
            stream *= 2;
        }
        if self.has_built(StructureKind::Airport) {
            stream *= 2;
        }
        stream = match content {
            0..40 => stream / 2,
            40..65 => stream,
            _ => stream * 5 / 4,
        };
        let settling = can_draw && self.state.rng.one_in(stream, 100_000);
        // Whoever passes through and makes camp may ask to stay, the sooner the fewer
        // there are to get used to.
        let passing = self.state.rng.chance(WANDERER_CHANCE);
        let asking = if open { 100 } else { 50 } * SMALL_BAND / (SMALL_BAND + members as u64);
        if !settling && !(passing && (camping < MOST_WANDERERS || can_draw)) {
            return;
        }

        let home = self
            .state
            .settlement
            .as_ref()
            .and_then(|settlement| self.state.world.structure(settlement.cairn))
            .map_or(self.state.player.location(), Structure::location);
        let camp = (0..8).find_map(|_| {
            let way = self.state.rng.unit() * std::f64::consts::TAU;
            let away = self.state.rng.between(CAMP_DISTANCE) as f64;
            let camp = Location::new(
                home.x() + (away * way.cos()) as i64,
                home.y() + (away * way.sin()) as i64,
            );
            (!self.state.world.is_water(camp)).then_some(camp)
        });
        let Some(camp) = camp else {
            return;
        };

        let mut wanderer = Person::wanderer(&mut self.state.rng, now, camp);
        let days = self.state.rng.between(CAMP_DAYS) as u64;
        wanderer.leaves = Some(Timestamp::from_seconds(
            now.as_seconds() + days * SECONDS_PER_DAY,
        ));

        let drawn = settling || (can_draw && self.state.rng.chance(asking));
        if !drawn {
            if camping >= MOST_WANDERERS {
                return;
            }
            let way = self.state.player.location().bearing_to(camp).name();
            self.state.world.add_person(wanderer);
            self.note(
                Tone::Plain,
                format!("Smoke rises to the {way}. Someone has made camp there."),
            );
            self.stale = true;
            return;
        }

        // To a place of many, people come with their own: a partner, and a child.
        let told = members < CROWD;
        let name = wanderer.name.full();
        let family = wanderer.name.last().to_owned();
        let settle = |mut newcomer: Person| {
            newcomer.member = true;
            newcomer.leaves = None;
            newcomer.regard = FRIENDLY;
            newcomer.location = home;
            newcomer
        };
        let first = settle(wanderer);
        let (sex, views) = (first.sex, first.views);
        let first = self.state.world.add_person(first);
        self.count_towards_the_month(0);

        let room = open || (members as u32 + 2) < self.housing();
        if settling && members >= 12 && room && self.state.rng.chance(40) {
            let mut partner = settle(Person::wanderer(&mut self.state.rng, now, home));
            partner.sex = if sex == Sex::Female {
                Sex::Male
            } else {
                Sex::Female
            };
            let given = given_name(&mut self.state.rng, partner.sex).to_owned();
            partner.name = Name::new(given, Vec::new(), family.clone());
            partner.partner = Some(first);
            let partner_views = partner.views;
            let partner = self.state.world.add_person(partner);
            if let Some(first) = self.state.world.person_mut(first) {
                first.partner = Some(partner);
            }
            self.count_towards_the_month(0);

            if self.state.rng.chance(50) {
                let childs = if self.state.rng.chance(50) {
                    Sex::Female
                } else {
                    Sex::Male
                };
                let given = given_name(&mut self.state.rng, childs).to_owned();
                let age_in_days = self.state.rng.between((200, 10 * 365)) as u64;
                let born = now
                    .as_seconds()
                    .saturating_sub(age_in_days * SECONDS_PER_DAY);
                let mut child = Person::new(
                    Name::new(given, Vec::new(), family),
                    childs,
                    Timestamp::from_seconds(born),
                    home,
                );
                child.views = inherited_views(&mut self.state.rng, [&views, &partner_views]);
                child.parents = if sex == Sex::Female {
                    [Some(first), Some(partner)]
                } else {
                    [Some(partner), Some(first)]
                };
                child.role = Role::Idle;
                child.regard = FRIENDLY;
                child.member = true;
                self.state.world.add_person(child);
                self.count_towards_the_month(0);
            }
        }

        if told {
            let place = self
                .state
                .settlement
                .as_ref()
                .map_or_else(String::new, |settlement| settlement.name.clone());
            self.note(
                Tone::Good,
                format!("{name} has come to {place} asking to stay, and is made welcome."),
            );
        }
        self.stale = true;
    }

    /// Returns how many people the houses of the settlement hold.
    pub(super) fn housing(&self) -> u32 {
        self.structures()
            .into_iter()
            .map(|(_, kind)| houses(kind))
            .sum()
    }

    /// Returns how many days the food in the store lasts `members` people.
    pub(super) fn food_days(&self, members: usize) -> u32 {
        let energy: u64 = self.store().map_or(0, |store| {
            store
                .stacks()
                .iter()
                .filter_map(|stack| {
                    Some(u64::from(stack.item.food()?.energy) * u64::from(stack.count))
                })
                .sum()
        });

        let days = energy / (u64::from(ADULT_FARE) * members.max(1) as u64);
        u32::try_from(days).unwrap_or(u32::MAX)
    }

    /// Has everyone who lives in the settlement eat out of the store, and deals with those
    /// who went without for too long.
    fn feed(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let me = self.state.world.protagonist();
        let mut store = self.store().cloned().unwrap_or_default();
        let mut starved = Vec::new();
        let mut left = Vec::new();
        let leads = self.leads();
        let today = now.as_seconds() / SECONDS_PER_DAY;
        let table = if self.lives_by(Law::SharedTable) {
            90
        } else if self.lives_by(Law::OwnHearths) {
            110
        } else {
            100
        };
        let patience = if self.lives_by(Law::OldCustoms) {
            DAYS_TO_LEAVE + 5
        } else {
            DAYS_TO_LEAVE
        };

        for id in members {
            let Some(person) = self.state.world.person_mut(*id) else {
                continue;
            };
            let mut fare = if person.is_adult(now) {
                ADULT_FARE
            } else {
                CHILD_FARE
            };
            // Thinkers are kept on half as much again for their trouble.
            if matches!(person.role, Role::Thinker | Role::Scholar) && person.is_adult(now) {
                fare = fare * 3 / 2;
            }
            fare = fare * table / 100;

            if eat(&mut store, fare) >= fare * 4 / 5 {
                person.hungry_days = 0;
                // Those who are fed come to think well of whoever leads, in time.
                if leads && person.regard < 60 && (today + u64::from(id.0)).is_multiple_of(10) {
                    person.warm_to(1);
                }
                continue;
            }
            person.hungry_days += 1;
            person.cool_to(3);
            let family = person.partner == me || person.parents.contains(&me);
            if person.hungry_days >= DAYS_TO_STARVE {
                starved.push(*id);
            } else if person.hungry_days >= patience && !family && person.is_adult(now) {
                left.push(*id);
            }
        }

        if let Some(kept) = self.store_mut() {
            *kept = store;
        }
        if members.iter().any(|id| {
            let person = self.state.world.person(*id);
            person.is_some_and(|person| person.hungry_days == 1)
        }) {
            self.note(
                Tone::Bad,
                "There is not enough food in the store for everyone.",
            );
        }
        for id in starved {
            self.person_dies(id, "hunger");
        }
        let crowded = self.crowded();
        for id in left {
            let told = !crowded || self.is_close(id);
            if let Some(person) = self.state.world.person_mut(id) {
                person.gone = true;
                let name = person.name.full();
                self.stale = true;
                if told {
                    self.note(
                        Tone::Bad,
                        format!("{name} has left to find food elsewhere."),
                    );
                }
            }
        }
    }

    /// Feeds the children of a player who has no settlement out of what the player has
    /// within reach.
    fn feed_children(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let children: Vec<PersonId> = members
            .iter()
            .copied()
            .filter(|id| {
                let person = self.state.world.person(*id);
                person.is_some_and(|person| !person.works(now))
            })
            .collect();

        for id in children {
            let mut eaten = eat(self.state.player.carried_mut(), CHILD_FARE);
            for pile in self.at_hand.piles.clone() {
                if eaten >= CHILD_FARE {
                    break;
                }
                if let Some(pile) = self.state.world.pile_mut(pile) {
                    eaten += eat(pile.items_mut(), CHILD_FARE - eaten);
                    self.stale = true;
                }
            }

            let Some(child) = self.state.world.person_mut(id) else {
                continue;
            };
            if eaten >= CHILD_FARE * 4 / 5 {
                child.hungry_days = 0;
                continue;
            }
            child.hungry_days += 1;
            let name = child.name.first().to_owned();
            if child.hungry_days >= DAYS_TO_STARVE {
                self.person_dies(id, "hunger");
            } else {
                self.note(
                    Tone::Bad,
                    format!("{name} goes hungry. You have no food to share."),
                );
            }
        }
    }

    /// Burns logs to heat the houses when it is freezing, and lets the frost take its toll
    /// of those it reaches.
    pub(super) fn winter(&mut self, members: &[PersonId]) {
        if self.conditions.temperature.as_tenths() >= 0 || members.is_empty() {
            return;
        }

        // The houses are heated by a power station where one runs, and with coal or logs
        // where none does.
        let powered = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.running.power);
        // A hard winter takes half as much wood again.
        let today = self.state.time.as_seconds() / SECONDS_PER_DAY;
        let hard = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.hard_winter_until > today);
        let usual = members.len().div_ceil(4) as u32;
        let needed = if hard {
            usual + usual.div_ceil(2)
        } else {
            usual
        };
        let burned = if powered {
            needed
        } else {
            self.store_mut().map_or(0, |store| {
                let coal = store.take(Item::Coal, needed).count;
                coal + store.take(Item::Log, needed - coal).count
            })
        };
        let housed = self.housing() as usize;
        let woven = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.knows(Advance::Weaving));
        let was_unheated = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.running.unheated);
        if let Some(settlement) = &mut self.state.settlement {
            settlement.running.unheated = burned < needed.min(usual);
            if burned < needed {
                settlement.contentment = settlement.contentment.saturating_sub(1).max(20);
            }
        }
        // With the usual wood in a winter that takes more, the houses are cold, but
        // nobody under a roof dies of it.
        let exposed = if burned < needed.min(usual) {
            if !was_unheated {
                self.alert = true;
                self.note(
                    Tone::Bad,
                    "There is nothing in the store to heat the houses with.",
                );
            }
            members.len()
        } else {
            members.len().saturating_sub(housed)
        };

        for id in members.iter().rev().take(exposed) {
            let clothed = self
                .state
                .world
                .person(*id)
                .is_some_and(|person| person.clothed);
            let risk = if clothed || woven {
                EXPOSURE_RISK / 2
            } else {
                EXPOSURE_RISK
            };
            if self.state.rng.one_in(risk, 1_000) {
                self.person_dies(*id, "the cold");
            }
        }
    }

    /// Lets those who live in the settlement find each other, and have children.
    fn pair_up(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let me = self.state.world.protagonist();
        let room = (members.len() as u32) < self.housing();
        let single = |sex| {
            let world = &self.state.world;
            members
                .iter()
                .copied()
                .filter(move |id| {
                    world.person(*id).is_some_and(|person| {
                        person.sex == sex && person.partner.is_none() && person.is_adult(now)
                    })
                })
                .collect::<Vec<_>>()
        };

        let men = single(Sex::Male);
        for woman_id in single(Sex::Female) {
            if !self.state.rng.chance(2) {
                continue;
            }
            let world = &self.state.world;
            let Some(woman) = world.person(woman_id) else {
                continue;
            };
            let man_id = men.iter().copied().find(|man_id| {
                world.person(*man_id).is_some_and(|man| {
                    man.partner.is_none()
                        && !woman.is_close_kin(woman_id, man, *man_id)
                        && woman.age(now).abs_diff(man.age(now)) <= 15
                })
            });
            let Some(man_id) = man_id else {
                continue;
            };

            let names = [woman_id, man_id].map(|id| {
                let person = self.state.world.person(id);
                person.map_or_else(String::new, |person| person.name.first().to_owned())
            });
            if let Some(woman) = self.state.world.person_mut(woman_id) {
                woman.partner = Some(man_id);
            }
            if let Some(man) = self.state.world.person_mut(man_id) {
                man.partner = Some(woman_id);
            }
            self.note(
                Tone::Good,
                format!("{} and {} have become partners.", names[0], names[1]),
            );
        }

        // Where most can read, and where few children die, fewer are born.
        let days_to_conceive = match &self.state.settlement {
            Some(settlement) if settlement.knows(Advance::Antibiotics) => 5_000,
            Some(settlement) if settlement.knows(Advance::Vaccination) => 3_500,
            Some(settlement) if settlement.literacy >= 50 => 2_200,
            _ => 800,
        };
        let due = Timestamp::from_seconds(now.as_seconds() + PREGNANCY_DAYS * SECONDS_PER_DAY);
        for id in members {
            let world = &self.state.world;
            let Some(woman) = world.person(*id) else {
                continue;
            };
            let recovered = woman
                .last_birth
                .is_none_or(|birth| now.seconds_since(birth) >= 400 * SECONDS_PER_DAY);
            let partnered = woman.partner.is_some_and(|partner| {
                Some(partner) != me && world.person(partner).is_some_and(Person::is_present)
            });

            if room
                && partnered
                && recovered
                && woman.is_fertile(now)
                && self.state.rng.one_in(1, days_to_conceive)
                && let Some(woman) = self.state.world.person_mut(*id)
            {
                woman.expecting = Some(due);
            }
        }
    }

    /// Returns how many in a hundred thousand die of a sickness in a day, as the
    /// settlement is.
    pub(super) fn sickness_risk(&self, members: usize) -> u64 {
        let people = members as u32 + 1;
        let Some(settlement) = &self.state.settlement else {
            return SICKNESS_RISK * 10;
        };
        let knows = |advance| settlement.knows(advance);
        let running = settlement.running;
        let mut risk = SICKNESS_RISK * 10;

        if knows(Advance::Herbalism) {
            risk /= 2;
        }
        // Healers save the more the more of the people they can see to.
        let seen_to = u64::from(
            (running.healers * super::economy::PEOPLE_PER_HEALER * 100 / people).min(100),
        );
        let saved = if knows(Advance::Chemistry) { 60 } else { 35 };
        risk = risk * (100 - saved * seen_to / 100) / 100;
        if knows(Advance::Antibiotics) {
            risk /= 3;
        }
        if people > self.housing() {
            risk = risk * 3 / 2;
        }
        if self.has_built(StructureKind::Waterworks) {
            risk = risk * 3 / 4;
        } else if members >= 60 {
            risk = risk * 5 / 4;
        }
        if self.has_built(StructureKind::Bathhouse) {
            risk = risk * 9 / 10;
        }
        if self.has_built(StructureKind::Sauna) {
            risk = risk * 19 / 20;
        }
        // Foul air and foul water shorten lives.
        risk = risk * (100 + u64::from(running.pollution.min(60))) / 100;
        let ale = self.store().is_some_and(|store| store.has(Item::Ale));
        if ale {
            risk = risk * 17 / 20;
        }
        if settlement.lives_by(Law::FreePort) {
            risk = risk * 6 / 5;
        }
        risk
    }

    /// Returns the age from which each day may be someone's last.
    pub(super) fn old_age(&self) -> u64 {
        let mut age = super::life::OLD_AGE;
        if self.knows_advance(Advance::Antibiotics) {
            age += 5;
        }
        if self.knows_advance(Advance::ModernMedicine) {
            age += 10;
        }
        age
    }

    /// Lets sickness and old age take whom they take.
    fn mortality(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let date = Date::from_timestamp(now);
        let mut sickness = self.sickness_risk(members.len());
        if self.lives_by(Law::ClosedGates) && self.state.rng.chance(50) {
            sickness = 0;
        }
        let vaccinated = self.knows_advance(Advance::Vaccination);
        let chemistry = self.knows_advance(Advance::Chemistry);
        let old_age = self.old_age();

        // Now and then a sickness sweeps through a place where many live close together.
        let epidemic = members.len() >= CROWD
            && !vaccinated
            && date.day() == 15
            && self.state.rng.chance(EPIDEMIC_CHANCE);
        if epidemic {
            self.alert = true;
            self.note(
                Tone::Bad,
                "A sickness is going from house to house. Many have taken to their beds.",
            );
            self.chronicle("A sickness went from house to house.");
        }
        let toll = EPIDEMIC_TOLL * sickness / (SICKNESS_RISK * 10);

        for id in members {
            let Some(person) = self.state.world.person(*id) else {
                continue;
            };
            let age = person.age(now);
            let (hardy, frail) = (person.has(Trait::Hardy), person.has(Trait::Frail));
            let years_old = age.saturating_sub(old_age - 1);
            // Small children are the first to go.
            let risk = match age {
                0..5 if vaccinated => sickness * 3 / 2,
                0..5 => sickness * 4,
                _ => sickness,
            };
            let risk = if hardy {
                risk / 2
            } else if frail {
                risk * 2
            } else {
                risk
            };

            let sickened = self.state.rng.one_in(risk, 100_000)
                || (epidemic && self.state.rng.chance(toll.max(1)));
            if sickened {
                // A dose of medicine saves every other one it is given to.
                let dosed = self
                    .store_mut()
                    .is_some_and(|store| store.take(Item::Medicine, 1).count > 0);
                let cured = dosed && self.state.rng.chance(if chemistry { 75 } else { 50 });
                if !cured {
                    self.person_dies(*id, "a sickness");
                }
            } else if self
                .state
                .rng
                .one_in(years_old * years_old, super::life::OLD_AGE_ODDS)
            {
                self.person_dies(*id, "old age");
            }
        }
    }

    /// Works out anew what the player is shown of the others and of the settlement.
    pub(super) fn refresh_society(&mut self) {
        self.society_stale = false;
        let now = self.state.time;
        let world = &self.state.world;
        let me = world.protagonist();

        let backers = self.backers();
        let mut census: Vec<PersonView> = world
            .people()
            .filter_map(|(id, person)| {
                let bond = world.bond(id)?;
                let partner = person
                    .partner
                    .filter(|partner| Some(*partner) != me)
                    .and_then(|partner| world.person(partner))
                    .map(|partner| partner.name.first().to_owned());

                Some(PersonView {
                    id,
                    subject: Subject::Person(id),
                    name: person.name.full(),
                    sex: person.sex,
                    age: person.age(now),
                    bond,
                    member: person.member,
                    met: person.talked_on.is_some() || person.member,
                    role: person.role,
                    works: person.works(now),
                    regard: person.regard,
                    expecting: person.expecting.is_some(),
                    hungry: person.hungry_days > 0,
                    partner,
                    views: person.views,
                    esteem: person.esteem,
                    hired: person.hired_until.is_some_and(|until| until > now),
                    backs_you: backers.contains(&id),
                    traits: person.traits.clone(),
                    skill: person.skill,
                    apprenticed: person.apprenticed,
                })
            })
            .collect();
        census.sort_by_key(|person| (std::cmp::Reverse(person.bond), person.id));
        self.census = census.into();
        self.kin = self.kin().into();

        let politics = self.politics_view();
        self.town = self
            .state
            .settlement
            .as_ref()
            .zip(politics)
            .map(|(settlement, politics)| {
                let members = self.members().len();
                let structures = self.structures();
                let mut buildings: Vec<(StructureKind, u32)> = Vec::new();
                let mut sites = Vec::new();
                for (id, kind) in &structures {
                    if *kind == StructureKind::Site {
                        let site = world.structure(*id);
                        let plan = site.and_then(Structure::plan).and_then(building);
                        if let (Some(site), Some(plan)) = (site, plan) {
                            sites.push(SiteView {
                                kind: plan.kind,
                                work: site.work(),
                                hours: plan.hours,
                            });
                        }
                    } else if building(*kind).is_some()
                        || matches!(kind, StructureKind::Cabin | StructureKind::LeanTo)
                    {
                        match buildings.iter_mut().find(|(built, _)| built == kind) {
                            Some((_, count)) => *count += 1,
                            None => buildings.push((*kind, 1)),
                        }
                    }
                }

                // The order of the buildings is that in which they come within reach.
                buildings
                    .sort_by_key(|(kind, _)| BUILDINGS.iter().position(|plan| plan.kind == *kind));
                let kinds = self.built();
                let ids = self.members();
                let hands = self.hands(&ids, &kinds);
                let jobs = Role::ALL
                    .into_iter()
                    .map(|role| (role, hands.of(role), places(role, &kinds)))
                    .filter(|(_, working, places)| *working > 0 || places.is_some_and(|p| p > 0))
                    .collect();

                Arc::new(SettlementView {
                    age: settlement.age(),
                    monetized: self.monetized(),
                    coined: settlement.knows(Advance::Coinage),
                    treasury: settlement.treasury,
                    tax: self.tax_rate(),
                    ledger: settlement.ledger,
                    orders: settlement.orders.clone(),
                    literacy: settlement.literacy,
                    contentment: settlement.contentment,
                    running: settlement.running,
                    tools: self.tools_percent(),
                    defence: self.defence(),
                    crime: settlement.crime,
                    pollution: settlement.running.pollution,
                    chronicle: settlement.chronicle.clone(),
                    jobs,
                    name: settlement.name.clone(),
                    founded: settlement.founded,
                    // The player character lives there too.
                    people: members as u32 + 1,
                    housing: self.housing(),
                    food_days: self.food_days(members + 1),
                    store: self.store().cloned().unwrap_or_default(),
                    advances: settlement.advances.clone(),
                    pursuit: settlement.pursuit,
                    insight: settlement.insight / 10,
                    growing: settlement.growing,
                    buildings,
                    sites,
                    politics,
                })
            });
    }
}

/// Eats up to `fare` kilocalories' worth out of `larder`, what goes off the soonest first,
/// and returns how many kilocalories were had.
pub(super) fn eat(larder: &mut Inventory, fare: u32) -> u32 {
    let mut foods: Vec<(Item, u32, u32)> = larder
        .stacks()
        .iter()
        .filter_map(|stack| {
            let food = stack.item.food()?;
            (food.energy > 0).then_some((stack.item, stack.count, food.energy))
        })
        .collect();
    foods.sort_by_key(|(item, _, _)| item.shelf_life().unwrap_or(u32::MAX));

    let mut eaten = 0;
    for (item, count, energy) in foods {
        if eaten >= fare {
            break;
        }
        let helpings = (fare - eaten).div_ceil(energy).min(count);
        larder.take(item, helpings);
        eaten += helpings * energy;
    }
    eaten
}

#[cfg(test)]
pub(super) mod tests;
