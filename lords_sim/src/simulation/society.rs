//! Life among other people: getting to know them, living together, and what a settlement
//! does with its days.

use std::sync::Arc;

use crate::action::{Job, Verb};
use crate::body::Ailment;
use crate::item::{Inventory, Item, Stack};
use crate::journal::Tone;
use crate::object::{Human, Living, Name, Object, Sex};
use crate::people::{ADULT_AGE, DEVOTED, FRIENDLY, Person, PersonId, Role, given_name};
use crate::politics::{Law, inherited_views};
use crate::settlement::{
    Advance, BUILDINGS, FARMERS_PER_FIELD, Settlement, WORKDAY, building, houses,
};
use crate::space::{Length, Location};
use crate::time::Timestamp;
use crate::time::gregorian::{Date, SECONDS_PER_DAY, TimeOfDay};
use crate::view::{PersonView, SettlementView, SiteView};
use crate::world::{PlantSpecies, Structure, StructureKind, Subject, Thing, ThingId};

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

/// How far from home wanderers make camp, in millimeters, and for how many days.
const CAMP_DISTANCE: (i64, i64) = (70_000, 160_000);
const CAMP_DAYS: (i64, i64) = (8, 20);

/// How many people a settlement must have before others come asking to join it, and how
/// many days of food it must have in store.
const DRAW_PEOPLE: usize = 3;
const DRAW_FOOD_DAYS: u32 = 10;

/// How far apart the buildings of a settlement stand, in millimeters.
const PLOT: i64 = 9_000;

/// The days of the year between which rye grows, and the day it is harvested.
const GROWING_SEASON: (u16, u16) = (121, 258);
const HARVEST_DAY: u16 = 259;
const GRAIN_PER_FARMER_DAY: u32 = 30;

/// How many pelts a set of winter clothes takes.
const PELTS_PER_OUTFIT: u32 = 4;

/// The chance in a thousand that someone left out in the frost dies of it in a day.
const EXPOSURE_RISK: u64 = 20;

/// The chance in ten thousand that someone dies of a sickness in a day.
const SICKNESS_RISK: u64 = 2;

/// What the player character does not put in the common store with the rest of their load.
const KEPT_ON: [Item; 11] = [
    Item::Knife,
    Item::FireSteel,
    Item::StoneAxe,
    Item::BoneNeedle,
    Item::FishingRod,
    Item::BarkPot,
    Item::FurHat,
    Item::FurCoat,
    Item::FurBoots,
    Item::Water,
    Item::BoiledWater,
];

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
    fn store_mut(&mut self) -> Option<&mut Inventory> {
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
    fn stock(&mut self, item: Item, count: u32) {
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
            .filter(|stack| !KEPT_ON.contains(&stack.item))
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
        self.note(Tone::Plain, "You put your load in the common store.");
        self.credit(load.iter().map(Self::worth).sum());
        true
    }

    /// Returns why the player has nothing to put in the store, if they have not.
    pub(super) fn check_deposit(&self) -> Result<(), String> {
        let carried = self.state.player.carried();
        let any = carried
            .stacks()
            .iter()
            .any(|stack| !KEPT_ON.contains(&stack.item));

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
        self.credit_work();
        self.society_stale = true;
        true
    }

    fn finished_building(&mut self, kind: StructureKind) {
        let name = crate::world::Kind::Structure(kind).name().to_lowercase();
        self.stale = true;
        self.note(Tone::Good, format!("The {name} is finished."));
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
        let Some(plot) = self.free_plot() else {
            return Err("There is no room left to build on.".to_owned());
        };

        if let Some(store) = self.store_mut() {
            for (item, count) in plan.materials {
                store.take(*item, *count);
            }
        }
        let site = Structure::site(Object::new(plot, self.state.time), kind);
        self.add_thing(Thing::Structure(site));
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

    /// Returns the nearest plot to the cairn that is dry and has nothing built on it.
    fn free_plot(&self) -> Option<Location> {
        let world = &self.state.world;
        let cairn = self.state.settlement.as_ref()?.cairn;
        let center = world.structure(cairn)?.location();
        let built: Vec<Location> = self
            .structures()
            .iter()
            .filter_map(|(id, _)| Some(world.structure(*id)?.location()))
            .collect();

        let mut plots: Vec<(i64, i64)> = (-6..=6)
            .flat_map(|east| (-6..=6).map(move |north| (east, north)))
            .filter(|plot| *plot != (0, 0))
            .collect();
        plots.sort_by_key(|(east, north)| (east * east + north * north, *north, *east));

        plots
            .into_iter()
            .map(|(east, north)| Location::new(center.x() + east * PLOT, center.y() + north * PLOT))
            .find(|plot| {
                let free = built
                    .iter()
                    .all(|at| at.distance_to(*plot) > Length::from_millimeters(PLOT / 2));
                free && !world.is_water(*plot)
            })
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
            person.role = role;
            self.society_stale = true;
        }
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
            if let Some(settlement) = &mut self.state.settlement
                && settlement.leader == my_id
            {
                // The next day's moot finds another.
                settlement.leader = None;
            }
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

        self.state.world.set_protagonist(id);
        if let Some(settlement) = &mut self.state.settlement {
            // What the dead brought to the store and said by the cairn is theirs, not
            // their heir's.
            settlement.credit = 0;
            settlement.noted_credit = 0;
            settlement.appealed_on = [None; 3];
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
                let spot = match person.role {
                    _ if !working => at_home,
                    Role::Idle => at_home,
                    Role::Fisher => {
                        shore.map_or(at_home, |shore| shore.toward(home, Length::from_meters(1)))
                    }
                    Role::Forager | Role::Gatherer | Role::Trapper | Role::Woodcutter => {
                        offset(home, 35_000.0 + f64::from(id.0 % 7) * 5_000.0)
                    }
                    Role::Farmer | Role::Builder | Role::Crafter | Role::Thinker => {
                        offset(home, 12_000.0)
                    }
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
        if self.state.settlement.is_some() {
            self.labor(&members);
            self.feed(&members);
            self.winter(&members);
            self.pair_up(&members);
            self.politics_day();
        } else {
            self.feed_children(&members);
        }
        self.mortality(&members);
    }

    pub(super) fn person_dies(&mut self, id: PersonId, of: &str) {
        let now = self.state.time;
        let Some(person) = self.state.world.person_mut(id) else {
            return;
        };
        person.died = Some(now);
        let name = person.name.full();
        let partner = person.partner.take();
        if let Some(partner) = partner.and_then(|id| self.state.world.person_mut(id)) {
            partner.partner = None;
        }

        self.stale = true;
        self.note(Tone::Bad, format!("{name} has died of {of}."));
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
            child.role = Role::Idle;
            self.state.world.add_person(child);

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
            self.note(Tone::Good, words);
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
        if camping >= MOST_WANDERERS || !self.state.rng.chance(WANDERER_CHANCE) {
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

        let members = self.members().len();
        let open = self.lives_by(Law::OpenGates);
        let drawn = members >= DRAW_PEOPLE
            && !self.lives_by(Law::ClosedGates)
            && self.food_days(members) >= DRAW_FOOD_DAYS
            && (open || (members as u32) < self.housing())
            && self.state.rng.chance(if open { 100 } else { 50 });
        if drawn {
            let name = wanderer.name.full();
            wanderer.member = true;
            wanderer.leaves = None;
            wanderer.regard = FRIENDLY;
            wanderer.location = home;
            self.state.world.add_person(wanderer);
            let place = self
                .state
                .settlement
                .as_ref()
                .map_or_else(String::new, |settlement| settlement.name.clone());
            self.note(
                Tone::Good,
                format!("{name} has come to {place} asking to stay, and is made welcome."),
            );
        } else {
            let way = self.state.player.location().bearing_to(camp).name();
            self.state.world.add_person(wanderer);
            self.note(
                Tone::Plain,
                format!("Smoke rises to the {way}. Someone has made camp there."),
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
        let energy: u32 = self.store().map_or(0, |store| {
            store
                .stacks()
                .iter()
                .filter_map(|stack| Some(stack.item.food()?.energy * stack.count))
                .sum()
        });

        energy / (ADULT_FARE * members.max(1) as u32)
    }

    /// Has those who work do a day of it.
    fn labor(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let date = Date::from_timestamp(now);
        let Some(settlement) = &self.state.settlement else {
            return;
        };
        let kinds: Vec<StructureKind> = self.structures().into_iter().map(|(_, k)| k).collect();
        let count = |kind| kinds.iter().filter(|built| **built == kind).count() as u32;
        let tools = if settlement.knows(Advance::Ironworking) {
            125
        } else {
            100
        };
        let boats = if count(StructureKind::Dock) > 0 { 2 } else { 1 };
        let food_share = if settlement.lives_by(Law::OwnHearths) {
            115
        } else if settlement.lives_by(Law::SharedTable) {
            95
        } else {
            100
        };
        let thinking = if settlement.lives_by(Law::Schooling) {
            150
        } else if settlement.lives_by(Law::OldCustoms) {
            70
        } else {
            100
        };
        let working_age = self.working_age();
        let written = settlement.knows(Advance::Writing);
        let smoked = count(StructureKind::Smokehouse) > 0;
        let workshop = count(StructureKind::Workshop) > 0;
        let field_room = count(StructureKind::Field) * FARMERS_PER_FIELD;
        let growing_season = (GROWING_SEASON.0..=GROWING_SEASON.1).contains(&date.ordinal());
        let (fish, meat) = if smoked {
            (Item::DriedFish, Item::DriedMeat)
        } else {
            (Item::CookedFish, Item::CookedMeat)
        };

        let mut yields: Vec<(Item, u32)> = Vec::new();
        let (mut builders, mut crafters, mut thinkers, mut farmers) = (0, 0, 0, 0);
        for id in members {
            let Some(person) = self.state.world.person(*id) else {
                continue;
            };
            if person.age(now) < working_age {
                continue;
            }
            // Someone hired by the player thinks, whatever else they are set to.
            if person.hired_until.is_some_and(|until| until > now) {
                thinkers += 1;
                continue;
            }

            match person.role {
                Role::Forager => yields.push(if PlantSpecies::Bilberry.is_ripe(date) {
                    (Item::Bilberries, 160)
                } else if PlantSpecies::Lingonberry.is_ripe(date) {
                    (Item::Lingonberries, 180)
                } else {
                    (Item::InnerBark, 40)
                }),
                Role::Gatherer => yields.extend([
                    (Item::SpruceBough, 8),
                    (Item::Stick, 6),
                    (Item::SpruceRoot, 4),
                    (Item::BirchBark, 3),
                    (Item::Stone, 2),
                ]),
                Role::Fisher => {
                    let catch = if self.conditions.frozen { 12 } else { 24 };
                    yields.push((fish, catch * boats));
                }
                Role::Trapper => yields.extend([(meat, 10), (Item::HarePelt, 1)]),
                Role::Woodcutter => yields.extend([(Item::Log, 4), (Item::Stick, 8)]),
                Role::Farmer => farmers += 1,
                Role::Builder => builders += 1,
                Role::Crafter => crafters += 1,
                Role::Thinker => thinkers += 1,
                Role::Idle => {}
            }
        }

        for (item, count) in yields {
            let share = if item.food().is_some() {
                food_share
            } else {
                100
            };
            self.stock(item, count * tools / 100 * share / 100);
        }

        if growing_season && let Some(settlement) = &mut self.state.settlement {
            settlement.growing += farmers.min(field_room) * GRAIN_PER_FARMER_DAY * tools / 100;
        }
        let harvest = self.state.settlement.as_mut().map_or(0, |settlement| {
            if date.ordinal() == HARVEST_DAY {
                std::mem::take(&mut settlement.growing)
            } else {
                0
            }
        });
        if harvest > 0 {
            self.stock(Item::Grain, harvest);
            self.note(
                Tone::Good,
                format!("The rye is brought in: {harvest} measures of grain."),
            );
        }

        let mut hours = builders * WORKDAY * tools / 100;
        for (id, kind) in self.structures() {
            if hours == 0 {
                break;
            }
            if kind != StructureKind::Site {
                continue;
            }
            let Some(site) = self.state.world.structure_mut(id) else {
                continue;
            };
            let Some(plan) = site.plan().and_then(building) else {
                continue;
            };
            let put_in = hours.min(plan.hours.saturating_sub(site.work()).max(1));
            hours -= put_in;
            if site.build(put_in, plan.hours) {
                self.finished_building(plan.kind);
            }
        }

        let outfits = crafters * if workshop { 2 } else { 1 };
        for _ in 0..outfits {
            let unclothed = members.iter().copied().find(|id| {
                let person = self.state.world.person(*id);
                person.is_some_and(|person| !person.clothed)
            });
            let pelts = self.store().map_or(0, |store| store.count(Item::HarePelt));
            let roots = self
                .store()
                .map_or(0, |store| store.count(Item::SpruceRoot));

            if let Some(unclothed) = unclothed.filter(|_| pelts >= PELTS_PER_OUTFIT) {
                if let Some(store) = self.store_mut() {
                    store.take(Item::HarePelt, PELTS_PER_OUTFIT);
                }
                if let Some(person) = self.state.world.person_mut(unclothed) {
                    person.clothed = true;
                }
            } else if roots >= 2
                && let Some(store) = self.store_mut()
            {
                let cordage = (roots / 2).min(4);
                store.take(Item::SpruceRoot, cordage * 2);
                store.add(Item::Cordage, cordage);
            }
        }

        // Insight is counted in tenths of a thinker's day. A little comes to any band of
        // people by itself.
        let mut insight = thinkers * 10;
        if workshop {
            insight = insight * 3 / 2;
        }
        if written {
            insight = insight * 3 / 2;
        }
        if members.len() >= 2 {
            insight += 3;
        }
        insight = insight * thinking / 100;
        let learned = self.state.settlement.as_mut().and_then(|settlement| {
            let pursuit = settlement.pursuit?;
            settlement.insight += insight;
            (settlement.insight >= pursuit.cost() * 10).then(|| {
                settlement.advances.push(pursuit);
                settlement.pursuit = None;
                settlement.insight = 0;
                pursuit
            })
        });
        if let Some(advance) = learned {
            self.note(
                Tone::Good,
                format!(
                    "Your people have worked out {}. {}.",
                    advance.name().to_lowercase(),
                    advance.use_of()
                ),
            );
        }
    }

    /// Has everyone who lives in the settlement eat out of the store, and deals with those
    /// who went without for too long.
    fn feed(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let me = self.state.world.protagonist();
        let mut store = self.store().cloned().unwrap_or_default();
        let mut starved = Vec::new();
        let mut left = Vec::new();
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
            if person.role == Role::Thinker && person.is_adult(now) {
                fare = fare * 3 / 2;
            }
            fare = fare * table / 100;

            if eat(&mut store, fare) >= fare * 4 / 5 {
                person.hungry_days = 0;
                if person.regard < 80 {
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
        for id in left {
            if let Some(person) = self.state.world.person_mut(id) {
                person.gone = true;
                let name = person.name.full();
                self.stale = true;
                self.note(
                    Tone::Bad,
                    format!("{name} has left to find food elsewhere."),
                );
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
    fn winter(&mut self, members: &[PersonId]) {
        if self.conditions.temperature.as_tenths() >= 0 || members.is_empty() {
            return;
        }

        let needed = members.len().div_ceil(4) as u32;
        let burned = self
            .store_mut()
            .map_or(0, |store| store.take(Item::Log, needed).count);
        let housed = self.housing() as usize;
        let woven = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.knows(Advance::Weaving));
        let exposed = if burned < needed {
            self.note(
                Tone::Bad,
                "There are no logs in the store to heat the houses with.",
            );
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
                && self.state.rng.one_in(1, 500)
                && let Some(woman) = self.state.world.person_mut(*id)
            {
                woman.expecting = Some(due);
            }
        }
    }

    /// Lets sickness and old age take whom they take.
    fn mortality(&mut self, members: &[PersonId]) {
        let now = self.state.time;
        let healed = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.knows(Advance::Herbalism));
        let mut sickness = if healed {
            SICKNESS_RISK / 2
        } else {
            SICKNESS_RISK
        };
        if self.lives_by(Law::ClosedGates) && self.state.rng.chance(50) {
            sickness = 0;
        }

        for id in members {
            let Some(age) = self.state.world.person(*id).map(|person| person.age(now)) else {
                continue;
            };
            let years_old = age.saturating_sub(super::life::OLD_AGE - 1);

            if self.state.rng.one_in(sickness, 10_000) {
                self.person_dies(*id, "a sickness");
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
                })
            })
            .collect();
        census.sort_by_key(|person| (std::cmp::Reverse(person.bond), person.id));
        self.census = census.into();

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
                    } else if BUILDINGS.iter().any(|plan| plan.kind == *kind)
                        || matches!(kind, StructureKind::Cabin | StructureKind::LeanTo)
                    {
                        match buildings.iter_mut().find(|(built, _)| built == kind) {
                            Some((_, count)) => *count += 1,
                            None => buildings.push((*kind, 1)),
                        }
                    }
                }

                Arc::new(SettlementView {
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
