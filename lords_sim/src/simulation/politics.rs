//! Who leads a settlement and how that is decided, the laws it lives by, how the player
//! wins people over, and how a settlement runs itself when the player does not run it.

use crate::item::{Item, Stack};
use crate::journal::Tone;
use crate::object::Living;
use crate::people::{ADULT_AGE, Person, PersonId, Role};
use crate::politics::{Appeal, Era, FIRMEST, Issue, Law, Views, agreement};
use crate::settlement::{Advance, BUILDINGS, FARMERS_PER_FIELD};
use crate::time::Timestamp;
use crate::time::gregorian::{Date, SECONDS_PER_DAY, is_leap_year};
use crate::view::{Candidate, PoliticsView};
use crate::world::StructureKind;

use super::Simulation;

/// The day of the year on which leaders are chosen, and the day on which an election to
/// come is called.
const MOOT_DAY: u16 = 80;
const CALLING_DAY: u16 = 50;

/// The day of the year on which a leader who is not the player sees to the laws.
const LAWGIVING_DAY: u16 = 100;

/// How much more a voice is worth to its own kin, and to the one who leads already.
const KINSHIP: i32 = 20;
const INCUMBENCY: i32 = 5;

/// How much each point of agreement between two people's views adds to what one thinks of
/// the other as a leader.
const LIKE_MINDEDNESS: i32 = 4;

/// The regard from which someone with no view on a matter sides with the player.
const LOYAL: u8 = 40;

/// How many kilocalories of food, or as much as is worth that, the player brings to the
/// store or takes out of it before people take note.
const NOTICEABLE: i64 = 5_000;

/// What an hour of work on a building of the settlement is worth.
const HOUR_OF_WORK: i64 = 1_500;

/// The kilocalories of food it takes to feast one person.
const FEAST_FARE: u32 = 1_500;

/// The kilocalories of food someone asks for thinking for the player, and for how many
/// days they do.
pub(super) const HIRE_FARE: u32 = 12_500;
pub(super) const HIRE_DAYS: u64 = 10;

/// How many days of food there must be in the store for whoever leads to be well thought
/// of.
const LEAN_DAYS: u32 = 5;

/// How many days of food there must be in the store before anyone is spared from finding
/// more of it, in the half of the year when food is easily found and in the half when
/// the winter has to be laid in for, and how many more before anyone is set to thinking.
const SPARE_DAYS: (u32, u32) = (15, 45);
const THINKING_DAYS: u32 = 30;

/// The esteem someone has to be held in to be named at a moot.
const NAMEABLE: u8 = 35;

/// The order in which a settlement left to itself works things out.
const CUSTOMARY_PURSUITS: [Advance; 9] = [
    Advance::Smoking,
    Advance::Pottery,
    Advance::Farming,
    Advance::Boats,
    Advance::Weaving,
    Advance::Herbalism,
    Advance::Masonry,
    Advance::Writing,
    Advance::Ironworking,
];

impl Simulation {
    /// Returns whether the player character leads the settlement.
    pub(super) fn leads(&self) -> bool {
        let me = self.state.world.protagonist();
        let settlement = self.state.settlement.as_ref();
        me.is_some() && settlement.is_some_and(|settlement| settlement.leader == me)
    }

    /// Returns why the player has no say over how the settlement is run, if they have not.
    pub(super) fn authority(&self) -> Result<(), String> {
        let Some(settlement) = &self.state.settlement else {
            return Err("There is no settlement to run.".to_owned());
        };
        if self.leads() {
            return Ok(());
        }

        let leader = settlement
            .leader
            .and_then(|leader| self.state.world.person(leader))
            .map_or("Nobody", |leader| leader.name.first());
        Err(format!(
            "You have no say in that. {leader} leads {}.",
            settlement.name
        ))
    }

    pub(super) fn lives_by(&self, law: Law) -> bool {
        let settlement = self.state.settlement.as_ref();
        settlement.is_some_and(|settlement| settlement.lives_by(law))
    }

    /// Returns the age from which the people of the settlement work.
    pub(super) fn working_age(&self) -> u64 {
        if self.lives_by(Law::Schooling) {
            crate::people::WORKING_AGE + 2
        } else {
            crate::people::WORKING_AGE
        }
    }

    pub(super) fn era(&self) -> Era {
        let Some(settlement) = &self.state.settlement else {
            return Era::Band;
        };
        let hall = self
            .structures()
            .iter()
            .any(|(_, kind)| *kind == StructureKind::Hall);

        Era::of(
            self.members().len() as u32 + 1,
            settlement.knows(Advance::Writing),
            hall,
        )
    }

    fn my_views(&self) -> Views {
        let world = &self.state.world;
        let me = world.protagonist().and_then(|me| world.person(me));
        me.map_or([0; 4], |me| me.views)
    }

    /// Returns whether the player character is alive and old enough to lead.
    fn can_stand(&self) -> bool {
        let born = Date::from_timestamp(self.state.player.birth());
        let age = Date::from_timestamp(self.state.time).years_since(born);
        self.state.death.is_none() && age >= ADULT_AGE
    }

    /// Returns the grown-ups of the settlement apart from the player character.
    fn grown_ups(&self) -> Vec<(PersonId, Person)> {
        let now = self.state.time;
        self.state
            .world
            .people()
            .filter(|(_, person)| person.member && person.is_adult(now))
            .map(|(id, person)| (id, person.clone()))
            .collect()
    }

    /// Returns who could lead: the player character, whoever leads now, and the two held
    /// in the highest esteem.
    fn candidates(&self, grown_ups: &[(PersonId, Person)]) -> Vec<PersonId> {
        let me = self.state.world.protagonist();
        let leader = self.state.settlement.as_ref().and_then(|s| s.leader);
        let mut candidates = Vec::new();

        if let Some(me) = me.filter(|_| self.can_stand()) {
            candidates.push(me);
        }
        if let Some(leader) = leader.filter(|leader| grown_ups.iter().any(|(id, _)| id == leader)) {
            candidates.push(leader);
        }
        let mut esteemed: Vec<&(PersonId, Person)> = grown_ups.iter().collect();
        esteemed.sort_by_key(|(id, person)| (std::cmp::Reverse(person.esteem), *id));
        for (id, person) in esteemed.into_iter().take(2) {
            if !candidates.contains(id) && person.esteem >= NAMEABLE {
                candidates.push(*id);
            }
        }
        candidates
    }

    /// Returns whom each grown-up of the settlement would have lead it, the player
    /// character and the other candidates each speaking for themselves.
    fn ballots(&self) -> Vec<(PersonId, PersonId)> {
        let world = &self.state.world;
        let me = world.protagonist();
        let my_record = me.and_then(|me| world.person(me));
        let my_views = self.my_views();
        let leader = self.state.settlement.as_ref().and_then(|s| s.leader);
        let grown_ups = self.grown_ups();
        let candidates = self.candidates(&grown_ups);
        if candidates.is_empty() {
            return Vec::new();
        }

        let akin = |voter: &Person, voter_id: PersonId, other: &Person, other_id: PersonId| {
            voter.partner == Some(other_id)
                || voter.parents.contains(&Some(other_id))
                || other.parents.contains(&Some(voter_id))
        };
        let standing = |voter: &Person, voter_id: PersonId, candidate: PersonId| {
            let (base, views, kin) = if Some(candidate) == me {
                let kin = my_record.is_some_and(|mine| akin(voter, voter_id, mine, candidate));
                (i32::from(voter.regard), my_views, kin)
            } else {
                let Some(other) = world.person(candidate) else {
                    return i32::MIN;
                };
                let kin = akin(voter, voter_id, other, candidate);
                (i32::from(other.esteem), other.views, kin)
            };

            base + LIKE_MINDEDNESS * agreement(&voter.views, &views)
                + if kin { KINSHIP } else { 0 }
                + if Some(candidate) == leader {
                    INCUMBENCY
                } else {
                    0
                }
        };

        let mut ballots: Vec<(PersonId, PersonId)> = grown_ups
            .iter()
            .map(|(id, voter)| {
                let choice = if candidates.contains(id) {
                    *id
                } else {
                    candidates
                        .iter()
                        .copied()
                        .max_by_key(|candidate| {
                            // The first named wins a tie.
                            let order = candidates.iter().position(|c| c == candidate);
                            (standing(voter, *id, *candidate), std::cmp::Reverse(order))
                        })
                        .unwrap_or(*id)
                };
                (*id, choice)
            })
            .collect();
        if let Some(me) = me.filter(|me| candidates.contains(me)) {
            ballots.push((me, me));
        }
        ballots
    }

    /// Returns how many voices each candidate would have at a moot held now, the one with
    /// the most first, and whoever leads already first among equals.
    fn tally(&self) -> Vec<(PersonId, u32)> {
        let leader = self.state.settlement.as_ref().and_then(|s| s.leader);
        let mut tally: Vec<(PersonId, u32)> = Vec::new();

        for (_, choice) in self.ballots() {
            match tally.iter_mut().find(|(candidate, _)| *candidate == choice) {
                Some((_, voices)) => *voices += 1,
                None => tally.push((choice, 1)),
            }
        }
        tally.sort_by_key(|(candidate, voices)| {
            (
                std::cmp::Reverse(*voices),
                Some(*candidate) != leader,
                *candidate,
            )
        });
        tally
    }

    /// Has the grown-ups of the settlement choose who leads it.
    pub(super) fn hold_moot(&mut self) {
        let me = self.state.world.protagonist();
        let year = Date::from_timestamp(self.state.time).year();
        let tally = self.tally();
        let voices: u32 = tally.iter().map(|(_, voices)| voices).sum();
        let mine = tally
            .iter()
            .find(|(candidate, _)| Some(*candidate) == me)
            .map_or(0, |(_, voices)| *voices);
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };
        let before = settlement.leader;
        let place = settlement.name.clone();
        settlement.moot_year = year;
        self.society_stale = true;

        let Some((chosen, won)) = tally.first().copied() else {
            settlement.leader = None;
            return;
        };
        settlement.leader = Some(chosen);
        if voices <= 1 {
            return;
        }

        if Some(chosen) == me {
            let words = if before == me {
                format!("The moot keeps you as leader of {place}, with {won} of {voices} voices.")
            } else {
                format!("The moot chooses you to lead {place}, with {won} of {voices} voices.")
            };
            self.alert = true;
            self.note(Tone::Good, words);
        } else {
            let name = self.state.world.person(chosen).map(|p| p.name.full());
            let name = name.unwrap_or_default();
            let kept = if before == Some(chosen) {
                "keeps"
            } else {
                "chooses"
            };
            let tone = if before == me { Tone::Bad } else { Tone::Plain };
            self.note(
                tone,
                format!(
                    "The moot {kept} {name} as leader of {place}, with {won} of {voices} \
                     voices. You had {mine}."
                ),
            );
        }
    }

    /// Lets what happens in the life of the settlement as a body happen for the day.
    pub(super) fn politics_day(&mut self) {
        let now = self.state.time;
        let date = Date::from_timestamp(now);
        let era = self.era();
        let Some(settlement) = &self.state.settlement else {
            return;
        };
        let leader_gone = settlement.leader.is_none_or(|leader| {
            let me = self.state.world.protagonist();
            let present = self
                .state
                .world
                .person(leader)
                .is_some_and(Person::is_present);
            !present || (Some(leader) == me && self.state.death.is_some())
        });
        let for_life = settlement.lives_by(Law::EldersWord);
        let term = if settlement.lives_by(Law::OpenMoot) {
            1
        } else {
            era.term()
        };
        let due = date.year() >= settlement.moot_year + term && !for_life;
        let place = settlement.name.clone();

        self.esteem_day(date.ordinal());
        if leader_gone || (due && date.ordinal() == MOOT_DAY) {
            self.hold_moot();
        } else if due && era == Era::Town && date.ordinal() == CALLING_DAY {
            self.alert = true;
            self.note(
                Tone::Plain,
                format!(
                    "{place} chooses its leader in thirty days. Now is the time to be seen \
                     and heard."
                ),
            );
        }

        let customary = self
            .state
            .settlement
            .as_ref()
            .is_some_and(|settlement| settlement.steward);
        if !self.leads() || customary {
            self.govern();
        }
        if !self.leads() && date.ordinal() == LAWGIVING_DAY {
            self.give_laws();
        }
    }

    /// Lets the esteem people are held in drift towards what their age, their work, and
    /// the state of the settlement they lead make of them.
    fn esteem_day(&mut self, day_of_year: u16) {
        let now = self.state.time;
        let leader = self.state.settlement.as_ref().and_then(|s| s.leader);
        let members = self.members();
        let fed = self.food_days(members.len() + 1) >= LEAN_DAYS;
        let campaigning = self.era() == Era::Town && (CALLING_DAY..MOOT_DAY).contains(&day_of_year);

        for id in members {
            let Some(person) = self.state.world.person_mut(id) else {
                continue;
            };
            if !person.is_adult(now) {
                continue;
            }

            let age = person.age(now).min(60) as i32;
            let mut due = 15 + age / 3 + (id.0 * 7 % 15) as i32;
            due += match person.role {
                Role::Builder | Role::Crafter | Role::Thinker => 5,
                _ => 0,
            };
            if Some(id) == leader {
                due += if fed { 10 } else { -10 };
            }
            if campaigning && day_of_year.is_multiple_of(2) {
                due += 10;
            }

            let esteem = i32::from(person.esteem);
            person.esteem = (esteem + (due - esteem).signum()).clamp(0, 100) as u8;
        }
    }

    /// Returns how many of the grown-ups are for `law`, and how many there are.
    fn ayes(&self, law: Law) -> (u32, u32) {
        let issue = law.issue() as usize;
        let grown_ups = self.grown_ups();
        let ayes = grown_ups
            .iter()
            .filter(|(_, person)| {
                let lean = person.views[issue] * law.side();
                lean > 0 || (lean == 0 && self.leads() && person.regard >= LOYAL)
            })
            .count();

        (ayes as u32, grown_ups.len() as u32)
    }

    /// Returns whether whoever leads makes the laws without asking anyone.
    fn decides_alone(&self) -> bool {
        self.lives_by(Law::EldersWord) || (self.era() == Era::Band && !self.lives_by(Law::OpenMoot))
    }

    /// Puts `law` in force, or takes it out of force if it is not `wanted`, if the player
    /// leads and the grown-ups have it.
    pub(super) fn legislate(&mut self, law: Law, wanted: bool) {
        if let Err(reason) = self.authority() {
            self.note(Tone::Plain, reason);
            return;
        }
        if self.lives_by(law) == wanted {
            return;
        }

        let (ayes, voices) = self.ayes(law);
        let ayes = if wanted { ayes } else { voices - ayes };
        if !self.decides_alone() && ayes * 2 < voices {
            let what = if wanted { "have" } else { "give up" };
            self.note(
                Tone::Bad,
                format!(
                    "The moot will not {what} {}: {ayes} of {voices} voices for it.",
                    law.name().to_lowercase()
                ),
            );
            return;
        }

        self.put_in_force(law, wanted);
        // Those the law suits think better of the one who made it, and the others worse.
        let side = if wanted { law.side() } else { -law.side() };
        for id in self.members() {
            if let Some(person) = self.state.world.person_mut(id) {
                match person.views[law.issue() as usize] * side {
                    1.. => person.warm_to(3),
                    0 => {}
                    _ => person.cool_to(3),
                }
            }
        }
        let words = if wanted {
            format!("{} is now the law. {}.", law.name(), law.effect())
        } else {
            format!("{} is the law no longer.", law.name())
        };
        self.note(Tone::Good, words);
    }

    fn put_in_force(&mut self, law: Law, wanted: bool) {
        if let Some(settlement) = &mut self.state.settlement {
            settlement
                .laws
                .retain(|other| *other != law && *other != law.opposite());
            if wanted {
                settlement.laws.push(law);
            }
        }
        self.society_stale = true;
    }

    /// Has a leader who is not the player make law of what they hold the most firmly, if
    /// the grown-ups have it.
    fn give_laws(&mut self) {
        let world = &self.state.world;
        let leader = self.state.settlement.as_ref().and_then(|s| s.leader);
        let Some(leader) = leader.and_then(|leader| world.person(leader)).cloned() else {
            return;
        };

        for law in Law::ALL {
            let firm = leader.views[law.issue() as usize] * law.side() == FIRMEST;
            if !firm || self.lives_by(law) {
                continue;
            }
            let (ayes, voices) = self.ayes(law);
            if self.decides_alone() || ayes * 2 >= voices {
                self.put_in_force(law, true);
                self.note(
                    Tone::Plain,
                    format!(
                        "{} makes {} the law. {}.",
                        leader.name.first(),
                        law.name().to_lowercase(),
                        law.effect()
                    ),
                );
                return;
            }
        }
    }

    pub(super) fn set_view(&mut self, issue: Issue, lean: i8) {
        let me = self.state.world.protagonist();
        if let Some(me) = me.and_then(|me| self.state.world.person_mut(me)) {
            me.views[issue as usize] = lean.clamp(-FIRMEST, FIRMEST);
            self.society_stale = true;
        }
    }

    pub(super) fn set_steward(&mut self, steward: bool) {
        if let Err(reason) = self.authority() {
            self.note(Tone::Plain, reason);
            return;
        }
        if let Some(settlement) = &mut self.state.settlement {
            settlement.steward = steward;
            self.society_stale = true;
        }
    }

    /// Returns how many kilocalories of food the player has of their own within reach:
    /// what they carry and what lies in piles, but not what is in the common store.
    fn own_food(&self) -> u32 {
        let energy = |stack: &Stack| stack.item.food().map_or(0, |food| food.energy) * stack.count;
        let carried: u32 = self
            .state
            .player
            .carried()
            .stacks()
            .iter()
            .map(energy)
            .sum();
        let piled: u32 = self
            .at_hand
            .piles
            .iter()
            .filter_map(|pile| self.state.world.pile(*pile))
            .flat_map(|pile| pile.items().stacks())
            .map(energy)
            .sum();

        carried + piled
    }

    /// Eats `fare` kilocalories of the player's own food, carried first.
    fn spend_own_food(&mut self, fare: u32) {
        let mut eaten = super::society::eat(self.state.player.carried_mut(), fare);
        for pile in self.at_hand.piles.clone() {
            if eaten >= fare {
                break;
            }
            if let Some(pile) = self.state.world.pile_mut(pile) {
                eaten += super::society::eat(pile.items_mut(), fare - eaten);
            }
        }
        self.stale = true;
    }

    /// Returns how many days are left before the player can make `appeal` again.
    fn appeal_rest(&self, appeal: Appeal) -> u64 {
        let today = self.state.time.as_seconds() / SECONDS_PER_DAY;
        let last = self
            .state
            .settlement
            .as_ref()
            .and_then(|settlement| settlement.appealed_on[appeal as usize]);

        last.map_or(0, |last| (last + appeal.rest_days()).saturating_sub(today))
    }

    /// Returns why the player cannot make `appeal` as things are, if they cannot.
    pub(super) fn check_appeal(&self, appeal: Appeal) -> Result<(), String> {
        let Some(settlement) = &self.state.settlement else {
            return Err("There is no settlement to win over.".to_owned());
        };
        let members = self.members().len() as u32;
        let rest = self.appeal_rest(appeal);

        if members == 0 {
            return Err("There is nobody here to win over.".to_owned());
        }
        if self.at_hand.store.is_none() {
            return Err("You have to be by the cairn or a storehouse to be heard.".to_owned());
        }
        if rest > 0 {
            return Err(format!(
                "People have had enough of that for now. Wait {rest} more days."
            ));
        }
        match appeal {
            Appeal::Speech => Ok(()),
            Appeal::Feast => {
                let (needed, had) = (FEAST_FARE * members, self.own_food());
                if had < needed {
                    Err(format!(
                        "A feast for {members} takes {needed} kcal of your own food, carried \
                         or piled up here. You have {had}."
                    ))
                } else {
                    Ok(())
                }
            }
            Appeal::Notices if !settlement.knows(Advance::Writing) => {
                Err("Nobody here can read or write yet.".to_owned())
            }
            Appeal::Notices if self.has(Item::BirchBark) < 5 => {
                Err("Notices take 5 pieces of birch bark to write on.".to_owned())
            }
            Appeal::Notices => Ok(()),
        }
    }

    /// Brings about what `appeal` comes to. Returns whether it was made.
    pub(super) fn finish_appeal(&mut self, appeal: Appeal) -> bool {
        let today = self.state.time.as_seconds() / SECONDS_PER_DAY;
        let now = self.state.time;
        let mine = self.my_views();
        let members = self.members();
        if let Some(settlement) = &mut self.state.settlement {
            settlement.appealed_on[appeal as usize] = Some(today);
        }

        match appeal {
            Appeal::Speech => {
                let (mut won, mut lost) = (0, 0);
                for id in &members {
                    let Some(person) = self.state.world.person_mut(*id) else {
                        continue;
                    };
                    if !person.is_adult(now) {
                        continue;
                    }
                    match agreement(&person.views, &mine) {
                        2.. => {
                            person.warm_to(3);
                            won += 1;
                        }
                        -1..=1 => person.warm_to(1),
                        _ => {
                            person.cool_to(1);
                            lost += 1;
                        }
                    }
                }
                self.note(
                    Tone::Plain,
                    format!(
                        "You speak of how you would have things. {won} nod along, and \
                         {lost} turn away."
                    ),
                );
            }
            Appeal::Feast => {
                self.spend_own_food(FEAST_FARE * members.len() as u32);
                for id in &members {
                    if let Some(person) = self.state.world.person_mut(*id) {
                        person.warm_to(8);
                        person.hungry_days = 0;
                    }
                }
                self.note(
                    Tone::Good,
                    "You lay out a feast. Everyone eats their fill, and remembers who fed them.",
                );
            }
            Appeal::Notices => {
                self.use_up(Item::BirchBark, 5);
                for id in &members {
                    if let Some(person) = self.state.world.person_mut(*id)
                        && person.is_adult(now)
                    {
                        person.warm_to(2);
                    }
                }
                self.note(
                    Tone::Plain,
                    "You put up notices of what you stand for where everyone passes.",
                );
            }
        }
        self.society_stale = true;
        true
    }

    /// Returns what `stack` is worth to the common store, in kilocalories for food and in
    /// as much as that for everything else.
    pub(super) fn worth(stack: &Stack) -> i64 {
        let each = match stack.item.food() {
            Some(food) => i64::from(food.energy),
            None => i64::from(stack.item.weight()) / 4,
        };
        each * i64::from(stack.count)
    }

    /// Takes note of the player bringing `worth` to the common store, or taking it out
    /// if it is less than nothing. People think better or worse of the player for it.
    pub(super) fn credit(&mut self, worth: i64) {
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };
        settlement.credit += worth;
        let level = settlement.credit.div_euclid(NOTICEABLE);
        let change = (level - settlement.noted_credit).clamp(-5, 5);
        let owing = settlement.credit < 0;
        if change == 0 {
            return;
        }
        settlement.noted_credit = level;

        for id in self.members() {
            if let Some(person) = self.state.world.person_mut(id) {
                if change > 0 {
                    person.warm_to(change as u8);
                } else {
                    person.cool_to((-change) as u8);
                }
            }
        }
        self.society_stale = true;
        if change > 0 {
            self.note(
                Tone::Good,
                "People take note of what you bring to the common store.",
            );
        } else if owing {
            self.note(
                Tone::Bad,
                "People take note that you take more out of the store than you bring to it.",
            );
        }
    }

    /// Takes note of the player putting an hour of work into a building of the settlement.
    pub(super) fn credit_work(&mut self) {
        self.credit(HOUR_OF_WORK);
    }

    /// Returns why the player cannot hire the person to think for them, if they cannot.
    pub(super) fn check_hire(&self, id: PersonId) -> Result<(), String> {
        let now = self.state.time;
        let Some(person) = self.state.world.person(id) else {
            return Err("They are no longer here.".to_owned());
        };
        let first = person.name.first();
        let energy = |stack: &Stack| stack.item.food().map_or(0, |food| food.energy) * stack.count;
        let carried: u32 = self
            .state
            .player
            .carried()
            .stacks()
            .iter()
            .map(energy)
            .sum();

        if self.state.settlement.is_none() {
            Err("There is nothing to work out without a settlement.".to_owned())
        } else if !person.is_adult(now) {
            Err(format!("{first} is too young for that."))
        } else if person.hired_until.is_some_and(|until| until > now) {
            Err(format!("{first} is thinking for you already."))
        } else if carried < HIRE_FARE {
            Err(format!(
                "{first} asks {HIRE_DAYS} days' keep for it: {HIRE_FARE} kcal of food out of \
                 what you carry. You carry {carried}."
            ))
        } else {
            Ok(())
        }
    }

    pub(super) fn finish_hire(&mut self, id: PersonId) -> bool {
        let now = self.state.time;
        super::society::eat(self.state.player.carried_mut(), HIRE_FARE);
        let Some(person) = self.state.world.person_mut(id) else {
            return false;
        };
        let until = now.as_seconds() + HIRE_DAYS * SECONDS_PER_DAY;
        person.hired_until = Some(Timestamp::from_seconds(until));
        person.warm_to(2);
        let first = person.name.first().to_owned();

        self.society_stale = true;
        self.note(
            Tone::Good,
            format!("{first} takes your food, and will think for you for {HIRE_DAYS} days."),
        );
        true
    }

    /// Runs the settlement the way it runs itself: sees to it that enough people find
    /// food, that what is needed gets built, and that something is being worked out.
    fn govern(&mut self) {
        let now = self.state.time;
        let date = Date::from_timestamp(now);
        let working_age = self.working_age();
        let members = self.members();
        let workers: Vec<PersonId> = members
            .iter()
            .copied()
            .filter(|id| {
                let person = self.state.world.person(*id);
                person.is_some_and(|person| person.age(now) >= working_age)
            })
            .collect();
        let hands = workers.len() as u32;
        let people = members.len() as u32 + 1;
        let food_days = self.food_days(people as usize);
        let kinds: Vec<StructureKind> = self.structures().into_iter().map(|(_, k)| k).collect();
        let count = |kind| kinds.iter().filter(|built| **built == kind).count() as u32;
        let store = self.store().cloned().unwrap_or_default();
        let Some(settlement) = &self.state.settlement else {
            return;
        };
        // From August to February the winter is laid in for.
        let spare = if (3..8).contains(&date.month()) {
            SPARE_DAYS.0
        } else {
            SPARE_DAYS.1
        };
        let lean = food_days < spare;
        let building = count(StructureKind::Site) > 0;
        let unclothed = members.iter().any(|id| {
            let person = self.state.world.person(*id);
            person.is_some_and(|person| !person.clothed)
        });
        let growing = (121..=258).contains(&date.ordinal());
        let berries = crate::world::PlantSpecies::Bilberry.is_ripe(date)
            || crate::world::PlantSpecies::Lingonberry.is_ripe(date);

        // Who does what, the most pressing first.
        let mut wanted: Vec<Role> = Vec::new();
        let mut want = |role: Role, many: u32| wanted.extend((0..many).map(|_| role));
        if hands >= 3 {
            want(Role::Woodcutter, 1);
        }
        if !lean {
            if building {
                want(Role::Builder, (hands / 5).max(1));
            }
            if unclothed && store.count(Item::HarePelt) >= 4 {
                want(Role::Crafter, 1);
            }
            if hands >= 5 {
                want(Role::Gatherer, hands / 8 + 1);
            }
            if food_days >= spare + THINKING_DAYS && settlement.pursuit.is_some() {
                want(Role::Thinker, hands / 6);
            }
        }
        if growing {
            let room = count(StructureKind::Field) * FARMERS_PER_FIELD;
            want(Role::Farmer, room.min(hands / 3));
        }
        let finders = [
            Role::Trapper,
            Role::Fisher,
            if berries { Role::Forager } else { Role::Fisher },
        ];
        for (index, id) in workers.iter().enumerate() {
            let role = wanted
                .get(index)
                .copied()
                .unwrap_or(finders[index % finders.len()]);
            if let Some(person) = self.state.world.person_mut(*id)
                && person.role != role
            {
                person.role = role;
                self.society_stale = true;
            }
        }

        // What to work out, if nothing is.
        let pursuit = CUSTOMARY_PURSUITS
            .into_iter()
            .find(|advance| settlement.can_pursue(*advance));
        if settlement.pursuit.is_none()
            && let Some(pursuit) = pursuit
            && let Some(settlement) = &mut self.state.settlement
        {
            settlement.pursuit = Some(pursuit);
            settlement.insight = 0;
            self.society_stale = true;
        }

        // What to build, one thing at a time.
        if building || lean {
            return;
        }
        let knows = |advance| {
            let settlement = self.state.settlement.as_ref();
            settlement.is_some_and(|settlement| settlement.knows(advance))
        };
        let housing = self.housing();
        let wants = |kind: StructureKind| match kind {
            StructureKind::Hut => housing < people,
            StructureKind::Longhouse => housing + 4 < people,
            StructureKind::Field => count(kind) * FARMERS_PER_FIELD < hands / 3,
            StructureKind::Hall => people >= Era::VILLAGE && count(kind) == 0,
            _ => count(kind) == 0,
        };
        let order = [
            StructureKind::Hut,
            StructureKind::Smokehouse,
            StructureKind::Storehouse,
            StructureKind::Field,
            StructureKind::Workshop,
            StructureKind::Dock,
            StructureKind::Kiln,
            StructureKind::Hall,
            StructureKind::Well,
            StructureKind::Smithy,
        ];
        let next = order.into_iter().find(|kind| {
            let plan = BUILDINGS.iter().find(|plan| plan.kind == *kind);
            plan.is_some_and(|plan| {
                let stocked = plan
                    .materials
                    .iter()
                    .all(|(item, count)| store.count(*item) >= *count);
                wants(*kind) && plan.needs.is_none_or(knows) && stocked
            })
        });
        if let Some(kind) = next {
            let _ = self.mark_out(kind);
        }
    }

    /// Returns what the player is shown of how the settlement is led.
    pub(super) fn politics_view(&self) -> Option<PoliticsView> {
        let settlement = self.state.settlement.as_ref()?;
        let world = &self.state.world;
        let me = world.protagonist();
        let date = Date::from_timestamp(self.state.time);
        let era = self.era();
        let tally = self.tally();
        let voices = tally.iter().map(|(_, voices)| voices).sum();

        let for_life = settlement.lives_by(Law::EldersWord);
        let term = if settlement.lives_by(Law::OpenMoot) {
            1
        } else {
            era.term()
        };
        let next_moot = (!for_life).then(|| {
            let mut year = (settlement.moot_year + term).max(date.year());
            if year == date.year() && date.ordinal() > MOOT_DAY {
                year += 1;
            }
            let years_between: u64 = (date.year()..year)
                .map(|year| if is_leap_year(year) { 366 } else { 365 })
                .sum();
            (years_between + u64::from(MOOT_DAY)).saturating_sub(u64::from(date.ordinal()))
        });

        Some(PoliticsView {
            era,
            leader: settlement
                .leader
                .and_then(|leader| world.person(leader))
                .map(|leader| leader.name.full()),
            leads: self.leads(),
            candidates: tally
                .into_iter()
                .filter_map(|(id, voices)| {
                    Some(Candidate {
                        name: world.person(id)?.name.full(),
                        voices,
                        you: Some(id) == me,
                    })
                })
                .collect(),
            voices,
            laws: settlement.laws.clone(),
            ayes: Law::ALL.map(|law| self.ayes(law).0),
            grown_ups: self.grown_ups().len() as u32,
            decides_alone: self.decides_alone(),
            next_moot,
            steward: settlement.steward,
            credit: settlement.credit,
            views: self.my_views(),
            appeal_rest: Appeal::ALL.map(|appeal| self.appeal_rest(appeal)),
        })
    }

    /// Returns those of the grown-ups who would have the player character lead.
    pub(super) fn backers(&self) -> Vec<PersonId> {
        let me = self.state.world.protagonist();
        self.ballots()
            .into_iter()
            .filter(|(voter, choice)| Some(*choice) == me && Some(*voter) != me)
            .map(|(voter, _)| voter)
            .collect()
    }
}

#[cfg(test)]
mod tests;
