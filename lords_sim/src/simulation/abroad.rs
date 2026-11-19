//! What comes of there being other settlements in the world, and of the mishaps and
//! feast days that befall any settlement.

use crate::item::Item;
use crate::journal::Tone;
use crate::neighbour::{GENEROUS, Mission, Neighbour, TRUSTING};
use crate::people::{PersonId, Role, Trait};
use crate::politics::Law;
use crate::settlement::Advance;
use crate::time::gregorian::{Date, SECONDS_PER_DAY};
use crate::world::StructureKind;

use super::Simulation;

/// The day of the year on which neighbours take stock of one another, the day on which
/// raiders set out, and midsummer.
const RECKONING_DAY: u16 = 1;
const RAIDING_DAY: u16 = 210;
const MIDSUMMER: u16 = 172;

/// The day of the year on which the autumn fair is held, and the fewest people a place
/// has before anyone comes to it.
const FAIR_DAY: u16 = 250;
const FAIR_PEOPLE: usize = 25;

/// The day of the year on which it is plain what kind of winter it will be, and the day
/// on which the spring flood comes, in the years it does.
const FIRST_FROSTS: u16 = 320;
const THAW: u16 = 105;

/// The chance in a hundred of a hard winter, and of a spring flood, in a year.
const HARD_WINTER_CHANCE: u64 = 12;
const FLOOD_CHANCE: u64 = 8;

/// The fewest people a settlement has before it is worth raiding, and the chance in a
/// hundred that a hostile neighbour raids in a year.
const WORTH_RAIDING: usize = 25;
const RAID_CHANCE: u64 = 20;

/// What a palisade and a town wall are worth in guards.
const PALISADE: u32 = 6;
const WALL: u32 = 20;

/// How many days in a hundred thousand something that is built of wood catches fire.
const FIRE_RISK: u64 = 2;

/// What is built of wood, and burns.
const WOODEN: [StructureKind; 12] = [
    StructureKind::Hut,
    StructureKind::Longhouse,
    StructureKind::Cabin,
    StructureKind::Cottage,
    StructureKind::Storehouse,
    StructureKind::Workshop,
    StructureKind::Smokehouse,
    StructureKind::Weavery,
    StructureKind::Tavern,
    StructureKind::Brewery,
    StructureKind::Sawmill,
    StructureKind::Hall,
];

impl Simulation {
    /// Returns how many raiders the settlement can stand against: its guards, the bold
    /// among them counted twice, and what its palisade and its wall are worth.
    pub(super) fn defence(&self) -> u32 {
        let now = self.state.time;
        let barracks = self
            .built()
            .iter()
            .filter(|kind| **kind == StructureKind::Barracks)
            .count() as u32;
        let mut places = barracks * 6;
        let mut guards = 0;
        for (_, person) in self.state.world.people() {
            if places == 0 {
                break;
            }
            if person.member && person.role == Role::Guard && person.works(now) {
                places -= 1;
                guards += if person.has(Trait::Bold) { 2 } else { 1 };
            }
        }

        let mut defence = guards + 2 * self.office(crate::politics::Office::Marshal);
        if self.has_built(StructureKind::Palisade) {
            defence += PALISADE;
        }
        if self.has_built(StructureKind::Wall) {
            defence += WALL;
        }
        defence
    }

    /// Returns how many of the neighbours there is a pact of trade with, and how many
    /// that are not hostile know more than the settlement does.
    pub(super) fn neighbourly(&self) -> (u32, u32) {
        let known = self
            .state
            .settlement
            .as_ref()
            .map_or(0, |settlement| settlement.advances.len());
        let pacts = self.state.neighbours.iter().filter(|n| n.pact).count();
        let ahead = self
            .state
            .neighbours
            .iter()
            .filter(|n| !n.is_hostile() && usize::from(n.advances) > known)
            .count();
        (pacts as u32, ahead as u32)
    }

    /// Lets what happens between settlements, and what befalls this one, happen for the
    /// day.
    pub(super) fn world_day(&mut self, members: &[PersonId]) {
        let date = Date::from_timestamp(self.state.time);
        if self.state.neighbours.is_empty() {
            self.state.neighbours = Neighbour::generate(&mut self.state.rng);
        }
        if self.state.settlement.is_none() {
            return;
        }

        if date.ordinal() == RECKONING_DAY {
            self.reckon_with_neighbours();
        }
        if date.ordinal() == RAIDING_DAY && members.len() >= WORTH_RAIDING {
            self.be_raided(members);
        }
        if date.ordinal() == MIDSUMMER {
            self.keep_midsummer(members.len() as u32 + 1);
        }
        if date.ordinal() == FAIR_DAY && members.len() >= FAIR_PEOPLE && self.monetized() {
            self.hold_fair();
        }
        if date.ordinal() == FIRST_FROSTS && self.state.rng.chance(HARD_WINTER_CHANCE) {
            let today = self.state.time.as_seconds() / SECONDS_PER_DAY;
            if let Some(settlement) = &mut self.state.settlement {
                settlement.hard_winter_until = today + 120;
            }
            self.alert = true;
            self.note(
                Tone::Bad,
                "The frost has come early and bites deep. This will be a hard winter: the \
                 houses will take half as much wood again.",
            );
            self.chronicle("A hard winter set in.");
        }
        if date.ordinal() == THAW && self.state.rng.chance(FLOOD_CHANCE) {
            self.flood();
        }
        self.catch_fire();
    }

    /// Has the neighbours grow, learn, and think again of the settlement, as they do
    /// once a year.
    fn reckon_with_neighbours(&mut self) {
        let lives_by = |law| self.lives_by(law);
        let mut drift = 0;
        if lives_by(Law::OpenGates) || lives_by(Law::FreePort) {
            drift += 2;
        }
        if lives_by(Law::ClosedGates) || lives_by(Law::Tariffs) {
            drift -= 2;
        }

        for index in 0..self.state.neighbours.len() {
            let grows = self.state.rng.between((0, 3)) as u32;
            let learns = self.state.rng.chance(22);
            let neighbour = &mut self.state.neighbours[index];
            neighbour.people += neighbour.people * grows / 100 + 1;
            if learns && usize::from(neighbour.advances) < Advance::ALL.len() {
                neighbour.advances += 1;
            }
            // Old grudges fade, and so does old goodwill, unless there is a pact.
            let fading = -i32::from(neighbour.regard.signum());
            neighbour.warm_by(drift + if neighbour.pact { 2 } else { fading });
        }
        self.society_stale = true;
    }

    /// Has every hostile neighbour that feels like it fall on the settlement.
    fn be_raided(&mut self, members: &[PersonId]) {
        let defence = self.defence();
        for index in 0..self.state.neighbours.len() {
            let neighbour = &self.state.neighbours[index];
            if !neighbour.is_hostile() || !self.state.rng.chance(RAID_CHANCE) {
                continue;
            }
            let (name, raiders) = (neighbour.name.clone(), neighbour.raiders());
            self.alert = true;

            if defence >= raiders {
                self.state.neighbours[index].warm_by(5);
                self.note(
                    Tone::Good,
                    format!(
                        "Raiders from {name} came, {raiders} of them, and were driven off \
                         at the edge of the settlement."
                    ),
                );
                continue;
            }

            // What cannot be defended is carried off: a fifth of the food, a fifth of
            // the treasury, and the lives of some who stood in the way.
            if let Some(store) = self.store_mut() {
                for stack in store.stacks().to_vec() {
                    if stack.item.food().is_some() {
                        store.take(stack.item, stack.count / 5);
                    }
                }
            }
            if let Some(settlement) = &mut self.state.settlement {
                settlement.treasury -= settlement.treasury.max(0) / 5;
                settlement.contentment = settlement.contentment.saturating_sub(10);
            }
            let fallen = (raiders - defence).div_ceil(4).min(2) as usize;
            let now = self.state.time;
            let victims: Vec<PersonId> = members
                .iter()
                .copied()
                .filter(|id| {
                    let person = self.state.world.person(*id);
                    person.is_some_and(|person| person.is_present() && person.is_adult(now))
                })
                .rev()
                .take(fallen)
                .collect();
            self.chronicle(format!("Raiders from {name} plundered the settlement."));
            self.note(
                Tone::Bad,
                format!(
                    "Raiders from {name} fell on the settlement, {raiders} of them against \
                     a defence worth {defence}. They carried off a fifth of the food and \
                     of the treasury."
                ),
            );
            for id in victims {
                self.person_dies(id, "the raid");
            }
        }
        self.stale = true;
        self.society_stale = true;
    }

    /// Has the thaw flood the low ground, which spoils a tenth of the food in the store,
    /// and less where there is a storehouse to keep it dry.
    fn flood(&mut self) {
        let spoiled = if self.has_built(StructureKind::Storehouse) {
            20
        } else {
            10
        };
        if let Some(store) = self.store_mut() {
            for stack in store.stacks().to_vec() {
                if stack.item.food().is_some() {
                    store.take(stack.item, stack.count / spoiled);
                }
            }
        }
        self.alert = true;
        self.society_stale = true;
        self.note(
            Tone::Bad,
            "The thaw has come all at once, and the lake is over its banks. Food in the \
             store has been spoiled by the water.",
        );
        self.chronicle("The spring flood spoiled food in the store.");
    }

    /// Holds the autumn fair, to which people come from all around: a week's trade in a
    /// day, of which the treasury has its share, and a day to remember.
    fn hold_fair(&mut self) {
        let tax = i64::from(self.tax_rate());
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };
        let takings = i64::from(settlement.running.trade) * 7 * tax / 100;
        settlement.treasury += takings;
        settlement.contentment = (settlement.contentment + 3).min(100);
        let first = !settlement
            .chronicle
            .iter()
            .any(|entry| entry.text.starts_with("The first autumn fair"));
        for neighbour in &mut self.state.neighbours {
            if !neighbour.is_hostile() {
                neighbour.warm_by(1);
            }
        }
        self.note(
            Tone::Good,
            format!(
                "The autumn fair fills the market with people from all around. The \
                 treasury takes {takings} coins."
            ),
        );
        if first {
            self.chronicle("The first autumn fair was held.");
        }
    }

    /// Has the settlement keep midsummer, if it has the food and the ale for it.
    fn keep_midsummer(&mut self, people: u32) {
        let ale = self.store().map_or(0, |store| store.count(Item::Ale));
        if self.food_days(people as usize) < 60 || ale < people / 2 || people < 6 {
            return;
        }
        if let Some(store) = self.store_mut() {
            store.take(Item::Ale, people / 2);
        }
        if let Some(settlement) = &mut self.state.settlement {
            settlement.contentment = (settlement.contentment + 6).min(100);
        }
        self.note(
            Tone::Good,
            "Midsummer is kept with fires, dancing, and ale. People will remember it.",
        );
    }

    /// Lets something built of wood burn down, as now and then it does.
    fn catch_fire(&mut self) {
        let wooden: Vec<_> = self
            .structures()
            .into_iter()
            .filter(|(_, kind)| WOODEN.contains(kind))
            .collect();
        let mut risk = FIRE_RISK * wooden.len() as u64;
        if self.has_built(StructureKind::Well) || self.has_built(StructureKind::Waterworks) {
            risk /= 2;
        }
        if wooden.is_empty() || !self.state.rng.one_in(risk, 100_000) {
            return;
        }

        let (id, kind) = wooden[self.state.rng.below(wooden.len() as u64) as usize];
        self.state.world.remove_thing(id);
        self.kept.retain(|kept| *kept != id);
        self.stale = true;
        self.society_stale = true;
        self.alert = true;
        let name = crate::world::Kind::Structure(kind).name().to_lowercase();
        self.note(
            Tone::Bad,
            format!("Fire! A {name} has burned to the ground."),
        );
        self.chronicle(format!("A {name} burned down."));
    }

    /// Returns why no envoy can be sent to the neighbour on `mission` as things are, if
    /// none can.
    fn check_envoy(&self, index: usize, mission: Mission) -> Result<(), String> {
        self.authority()?;
        let today = self.state.time.as_seconds() / SECONDS_PER_DAY;
        let Some(neighbour) = self.state.neighbours.get(index) else {
            return Err("There is no such place.".to_owned());
        };
        let Some(settlement) = &self.state.settlement else {
            return Err("There is no settlement to speak for.".to_owned());
        };
        let name = &neighbour.name;
        let rest = neighbour.rest(today);
        if rest > 0 {
            return Err(format!(
                "{name} has had word from you lately. Wait {rest} more days."
            ));
        }

        match mission {
            Mission::Gift => {
                let coined = settlement.knows(Advance::Coinage);
                let food = self.food_days(self.members().len() + 1);
                if coined && settlement.treasury < Mission::GIFT_COINS {
                    Err("The treasury cannot spare 100 coins for gifts.".to_owned())
                } else if !coined && food < 30 {
                    Err("There is not enough food in the store to give any away.".to_owned())
                } else {
                    Ok(())
                }
            }
            Mission::Pact if neighbour.pact => Err(format!("There is a pact with {name} already.")),
            Mission::Pact if !self.monetized() => {
                Err("There is nothing to trade without a market.".to_owned())
            }
            Mission::Pact if neighbour.regard < TRUSTING => Err(format!(
                "{name} does not think well enough of you for a pact."
            )),
            Mission::Learn if neighbour.regard < GENEROUS => {
                Err(format!("{name} is not close enough to you to teach you."))
            }
            Mission::Learn if usize::from(neighbour.advances) <= settlement.advances.len() => {
                Err(format!("{name} knows nothing that you do not."))
            }
            Mission::Learn if settlement.pursuit.is_none() => {
                Err("Nothing is being worked out for them to help with.".to_owned())
            }
            Mission::Raid if self.defence() == 0 => Err("You have no guards to send.".to_owned()),
            Mission::Pact | Mission::Learn | Mission::Raid => Ok(()),
        }
    }

    /// Sends an envoy to the neighbour on `mission`, if the player leads and it can be.
    pub(super) fn send_envoy(&mut self, index: usize, mission: Mission) {
        if let Err(reason) = self.check_envoy(index, mission) {
            self.note(Tone::Plain, reason);
            return;
        }
        let today = self.state.time.as_seconds() / SECONDS_PER_DAY;
        let guards = self.defence();
        let neighbour = &mut self.state.neighbours[index];
        neighbour.envoy_on = Some(today);
        let name = neighbour.name.clone();
        self.society_stale = true;

        match mission {
            Mission::Gift => {
                neighbour.warm_by(10);
                let coined = self.knows_advance(Advance::Coinage);
                if coined && let Some(settlement) = &mut self.state.settlement {
                    settlement.treasury -= Mission::GIFT_COINS;
                } else if let Some(store) = self.store_mut() {
                    super::society::eat(store, Mission::GIFT_FOOD);
                }
                self.note(
                    Tone::Good,
                    format!("Your gifts are well received in {name}."),
                );
            }
            Mission::Pact => {
                neighbour.pact = true;
                neighbour.warm_by(5);
                self.chronicle(format!("A pact of trade was made with {name}."));
                self.note(
                    Tone::Good,
                    format!("A pact is made with {name}. Their traders will come and go freely."),
                );
            }
            Mission::Learn => {
                neighbour.warm_by(-5);
                let share = self
                    .state
                    .settlement
                    .as_ref()
                    .and_then(|settlement| settlement.pursuit)
                    .map_or(0, |pursuit| pursuit.cost() * 2);
                self.note(
                    Tone::Good,
                    format!("The people of {name} show yours how they do things."),
                );
                self.work_out(share);
            }
            Mission::Raid => {
                let theirs = neighbour.raiders();
                neighbour.pact = false;
                neighbour.warm_by(-50);
                let people = neighbour.people;
                if guards > theirs {
                    neighbour.people -= people / 20;
                    let loot = i64::from(people) * 5;
                    if let Some(settlement) = &mut self.state.settlement {
                        settlement.treasury += loot;
                    }
                    self.stock(Item::Grain, people * 20);
                    self.note(
                        Tone::Good,
                        format!(
                            "Your guards fall on {name} and come home with {loot} coins' \
                             worth and {} measures of grain.",
                            people * 20
                        ),
                    );
                } else {
                    self.note(
                        Tone::Bad,
                        format!("Your guards are beaten back from {name}."),
                    );
                    let now = self.state.time;
                    let fallen: Vec<PersonId> = self
                        .state
                        .world
                        .people()
                        .filter(|(_, person)| {
                            person.member && person.role == Role::Guard && person.works(now)
                        })
                        .map(|(id, _)| id)
                        .take(2)
                        .collect();
                    for id in fallen {
                        self.person_dies(id, "the raid");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
