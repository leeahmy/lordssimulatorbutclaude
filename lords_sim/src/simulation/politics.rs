//! Who leads a settlement and how that is decided, the laws it lives by, how the player
//! wins people over, and how a settlement runs itself when the player does not run it.

use crate::item::{Item, Stack};
use crate::journal::Tone;
use crate::object::Living;
use crate::people::{ADULT_AGE, Person, PersonId, Role, Trait};
use crate::politics::{Appeal, Civics, Era, FIRMEST, Issue, Law, Office, Views, agreement};
use crate::settlement::Advance;
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
pub(super) const HOUR_OF_WORK: i64 = 1_500;

/// The kilocalories of food it takes to feast one person.
const FEAST_FARE: u32 = 1_500;

/// The kilocalories of food someone asks for thinking for the player, and for how many
/// days they do.
pub(super) const HIRE_FARE: u32 = 12_500;
pub(super) const HIRE_DAYS: u64 = 10;

/// The coins someone asks for it instead where there is coin.
pub(super) const HIRE_COINS: i64 = 60;

/// How many days of food there must be in the store for whoever leads to be well thought
/// of.
const LEAN_DAYS: u32 = 5;

/// The esteem someone has to be held in to be named at a moot.
const NAMEABLE: u8 = 35;

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
        let town_hall = self.has_built(StructureKind::TownHall);

        Era::of(Civics {
            people: self.members().len() as u32 + 1,
            writing: settlement.knows(Advance::Writing),
            hall: town_hall || self.has_built(StructureKind::Hall),
            charter: town_hall && settlement.knows(Advance::Law),
            power: self.has_built(StructureKind::PowerStation),
        })
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
    fn grown_ups(&self) -> Vec<(PersonId, &Person)> {
        let now = self.state.time;
        self.state
            .world
            .people()
            .filter(|(_, person)| person.member && person.is_adult(now))
            .collect()
    }

    /// Returns who could lead: the player character, whoever leads now, and the two held
    /// in the highest esteem.
    fn candidates(&self, grown_ups: &[(PersonId, &Person)]) -> Vec<PersonId> {
        let me = self.state.world.protagonist();
        let leader = self.state.settlement.as_ref().and_then(|s| s.leader);
        let mut candidates = Vec::new();

        if let Some(me) = me.filter(|_| self.can_stand()) {
            candidates.push(me);
        }
        if let Some(leader) = leader.filter(|leader| grown_ups.iter().any(|(id, _)| id == leader)) {
            candidates.push(leader);
        }
        let mut esteemed: Vec<&(PersonId, &Person)> = grown_ups.iter().collect();
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
                // Whoever leads speaks for themselves. Anyone else who is named thinks
                // well of themselves, but stands aside for someone they think more of.
                let choice = if Some(*id) == leader && candidates.contains(id) {
                    *id
                } else {
                    candidates
                        .iter()
                        .copied()
                        .max_by_key(|candidate| {
                            // The first named wins a tie.
                            let order = candidates.iter().position(|c| c == candidate);
                            let regard = if candidate == id {
                                i32::from(voter.esteem) + KINSHIP
                            } else {
                                standing(voter, *id, *candidate)
                            };
                            (regard, std::cmp::Reverse(order))
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
        let era = self.era();
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
        // What the one chosen is called: a leader, a headman, or a mayor.
        let leader = era.title();
        self.society_stale = true;

        let Some((chosen, won)) = tally.first().copied() else {
            settlement.leader = None;
            return;
        };
        settlement.leader = Some(chosen);
        // Custom keeps running things under a new leader until they say otherwise.
        if Some(chosen) == me && before != me {
            settlement.steward = true;
        }
        if voices <= 1 {
            return;
        }

        if before != Some(chosen) {
            let name = self.state.world.person(chosen).map(|p| p.name.full());
            self.chronicle(format!(
                "{} was chosen as {leader}.",
                name.unwrap_or_default()
            ));
        }
        if Some(chosen) == me {
            let words = if before == me {
                format!("The moot keeps you as {leader} of {place}, with {won} of {voices} voices.")
            } else {
                format!(
                    "The moot chooses you as {leader} of {place}, with {won} of {voices} voices."
                )
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
                    "The moot {kept} {name} as {leader} of {place}, with {won} of {voices} \
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
        let term = self.term(era);
        let due = date.year() >= settlement.moot_year + term && !for_life;
        let place = settlement.name.clone();
        let heir = if leader_gone && settlement.lives_by(Law::Birthright) {
            settlement.leader.and_then(|late| self.heir_of(late))
        } else {
            None
        };

        // A settlement that has grown into something more remembers the day.
        let reached = era as u8;
        if let Some(settlement) = &mut self.state.settlement
            && reached > settlement.era_reached
        {
            settlement.era_reached = reached;
            let place = settlement.name.clone();
            self.alert = true;
            let words = format!("{place} has become a {}.", era.name().to_lowercase());
            self.note(Tone::Good, words.clone());
            self.chronicle(words);
        }
        self.esteem_day(date.ordinal());
        if let Some(heir) = heir {
            self.succeed(heir);
        } else if leader_gone || (due && date.ordinal() == MOOT_DAY) {
            self.hold_moot();
        } else if due && era.campaigns() && date.ordinal() == CALLING_DAY {
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
            self.fill_offices();
        }
    }

    /// Returns how much the office does for the settlement: nothing if nobody holds it,
    /// one if someone does, and two if they are of the kind that holds it best.
    pub(super) fn office(&self, office: Office) -> u32 {
        let now = self.state.time;
        let holder = self.state.settlement.as_ref().and_then(|settlement| {
            let held = settlement.offices.iter().find(|(held, _)| *held == office);
            held.map(|(_, holder)| *holder)
        });
        let holder = holder
            .and_then(|holder| self.state.world.person(holder))
            .filter(|holder| holder.is_present() && holder.member && holder.is_adult(now));

        match holder {
            None => 0,
            Some(holder) if holder.has(office.suits()) => 2,
            Some(_) => 1,
        }
    }

    /// Returns the grown-ups held in the highest esteem, who could be given an office.
    fn trusted(&self) -> Vec<(PersonId, &Person)> {
        let mut grown_ups = self.grown_ups();
        grown_ups.sort_by_key(|(id, person)| (std::cmp::Reverse(person.esteem), *id));
        grown_ups.truncate(20);
        grown_ups
    }

    /// Gives `office` to the person, or takes it from whoever holds it, if the player
    /// leads.
    pub(super) fn appoint(&mut self, office: Office, holder: Option<PersonId>) {
        if let Err(reason) = self.authority() {
            self.note(Tone::Plain, reason);
            return;
        }
        let eligible =
            holder.is_none_or(|holder| self.trusted().iter().any(|(id, _)| *id == holder));
        if !eligible {
            self.note(
                Tone::Plain,
                "An office goes to someone grown whom people think well of.",
            );
            return;
        }
        self.install(office, holder, true);
    }

    fn install(&mut self, office: Office, holder: Option<PersonId>, by_player: bool) {
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };
        // Nobody holds two offices.
        settlement
            .offices
            .retain(|(held, other)| *held != office && Some(*other) != holder);
        self.society_stale = true;
        let Some(holder) = holder else {
            return;
        };
        settlement.offices.push((office, holder));
        let Some(person) = self.state.world.person_mut(holder) else {
            return;
        };
        person.esteem = person.esteem.saturating_add(5).min(100);
        if by_player {
            person.warm_to(10);
        }
        let name = person.name.full();
        let title = office.name().to_lowercase();
        self.note(Tone::Plain, format!("{name} is made {title}."));
        self.chronicle(format!("{name} was made {title}."));
    }

    /// Has a leader who is not the player give the offices nobody holds to those who
    /// suit them best among those people think well of.
    fn fill_offices(&mut self) {
        if self.era() == Era::Band {
            return;
        }
        for office in Office::ALL {
            if self.office(office) > 0 {
                continue;
            }
            let held: Vec<PersonId> = self
                .state
                .settlement
                .as_ref()
                .map_or_else(Vec::new, |s| s.offices.iter().map(|(_, id)| *id).collect());
            let trusted = self.trusted();
            let free = || trusted.iter().filter(|(id, _)| !held.contains(id));
            let pick = free()
                .find(|(_, person)| person.has(office.suits()))
                .or_else(|| free().next())
                .map(|(id, _)| *id);
            if pick.is_some() {
                self.install(office, pick, false);
            }
        }
    }

    /// Returns how many years pass between the choosing of leaders in a settlement of
    /// `era`, by the laws it lives by.
    fn term(&self, era: Era) -> u64 {
        if self.lives_by(Law::OpenMoot) {
            1
        } else if self.lives_by(Law::Assembly) {
            2
        } else {
            era.term()
        }
    }

    /// Returns the eldest grown child of `late` who lives in the settlement.
    fn heir_of(&self, late: PersonId) -> Option<PersonId> {
        let now = self.state.time;
        let world = &self.state.world;
        let me = world.protagonist();
        let alive = self.state.death.is_none();

        world
            .everyone()
            .filter(|(id, person)| {
                let here = person.is_present() && (person.member || Some(*id) == me);
                here && (Some(*id) != me || alive)
            })
            .filter(|(_, person)| person.parents.contains(&Some(late)) && person.is_adult(now))
            .min_by_key(|(id, person)| (person.born, *id))
            .map(|(id, _)| id)
    }

    /// Has `heir` lead after their parent, as is their birthright.
    fn succeed(&mut self, heir: PersonId) {
        let me = self.state.world.protagonist();
        let year = Date::from_timestamp(self.state.time).year();
        let name = self.state.world.person(heir).map(|heir| heir.name.full());
        let Some(settlement) = &mut self.state.settlement else {
            return;
        };
        settlement.leader = Some(heir);
        settlement.moot_year = year;
        let place = settlement.name.clone();
        self.society_stale = true;

        if Some(heir) == me {
            self.alert = true;
            self.note(
                Tone::Good,
                format!("By birthright, you lead {place} after your parent."),
            );
        } else {
            self.note(
                Tone::Plain,
                format!(
                    "By birthright, {} leads {place} after their parent.",
                    name.unwrap_or_default()
                ),
            );
        }
    }

    /// Lets the esteem people are held in drift towards what their age, their work, and
    /// the state of the settlement they lead make of them.
    fn esteem_day(&mut self, day_of_year: u16) {
        let now = self.state.time;
        let leader = self.state.settlement.as_ref().and_then(|s| s.leader);
        let members = self.members();
        let fed = self.food_days(members.len() + 1) >= LEAN_DAYS;
        let campaigning = self.era().campaigns() && (CALLING_DAY..MOOT_DAY).contains(&day_of_year);
        // Whoever leads is thought of by how content people are with their lot.
        let mood = self
            .state
            .settlement
            .as_ref()
            .map_or(0, |settlement| (i32::from(settlement.contentment) - 50) / 4);

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
                Role::Builder | Role::Crafter | Role::Thinker | Role::Smith => 5,
                Role::Healer | Role::Teacher | Role::Clerk | Role::Merchant => 8,
                Role::Scholar => 10,
                _ => 0,
            };
            if person.has(Trait::Kind) || person.has(Trait::Easygoing) {
                due += 8;
            }
            if person.has(Trait::Stern) {
                due -= 4;
            }
            due += i32::from(person.skill / 10);
            if Some(id) == leader {
                due += if fed { 10 } else { -10 };
                due += mood;
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
        let leads = self.leads();
        let ayes = grown_ups
            .iter()
            .filter(|(_, person)| {
                let lean = person.views[issue] * law.side();
                lean > 0 || (lean == 0 && leads && person.regard >= LOYAL)
            })
            .count();

        (ayes as u32, grown_ups.len() as u32)
    }

    /// Returns how many voices out of `voices` a law takes to be made or given up.
    fn needed_voices(&self, voices: u32) -> u32 {
        if self.lives_by(Law::Assembly) {
            (voices * 2).div_ceil(3)
        } else {
            voices.div_ceil(2)
        }
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
        if let Some(needed) = law.needs().filter(|needed| !self.knows_advance(*needed)) {
            self.note(
                Tone::Plain,
                format!(
                    "Nobody can make sense of such a law before {} is known.",
                    needed.name().to_lowercase()
                ),
            );
            return;
        }

        let (ayes, voices) = self.ayes(law);
        let ayes = if wanted { ayes } else { voices - ayes };
        if !self.decides_alone() && ayes < self.needed_voices(voices) {
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
        self.chronicle(if wanted {
            format!("{} became the law.", law.name())
        } else {
            format!("{} was the law no longer.", law.name())
        });
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
            let thinkable = law.needs().is_none_or(|needed| self.knows_advance(needed));
            if !firm || !thinkable || self.lives_by(law) || self.lives_by(law.opposite()) {
                continue;
            }
            let (ayes, voices) = self.ayes(law);
            if self.decides_alone() || ayes >= self.needed_voices(voices) {
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
            .and_then(|settlement| settlement.appealed_on.get(appeal as usize).copied())
            .flatten();

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
        let coins = appeal.coins();
        if coins > self.state.purse {
            return Err(format!(
                "That costs {coins} coins, and you have {}.",
                self.state.purse.max(0)
            ));
        }
        match appeal {
            Appeal::Speech => Ok(()),
            Appeal::Feast => {
                let guests = members.min(Appeal::TABLE);
                let (needed, had) = (FEAST_FARE * guests, self.own_food());
                if had < needed {
                    Err(format!(
                        "A feast for {guests} takes {needed} kcal of your own food, carried \
                         or piled up here. You have {had}."
                    ))
                } else {
                    Ok(())
                }
            }
            Appeal::Notices if !settlement.knows(Advance::Writing) => {
                Err("Nobody here can read or write yet.".to_owned())
            }
            Appeal::Notices if self.has(Item::BirchBark) < 5 && self.has(Item::Paper) < 5 => Err(
                "Notices take 5 pieces of birch bark or sheets of paper to write on.".to_owned(),
            ),
            Appeal::Notices => Ok(()),
            Appeal::Newspaper if !self.has_built(StructureKind::PrintingHouse) => {
                Err("There is no printing house to print a newspaper.".to_owned())
            }
            Appeal::Broadcast if !self.has_built(StructureKind::BroadcastingHouse) => {
                Err("There is no broadcasting house to speak from.".to_owned())
            }
            Appeal::Newspaper | Appeal::Broadcast => Ok(()),
        }
    }

    /// Brings about what `appeal` comes to. Returns whether it was made.
    pub(super) fn finish_appeal(&mut self, appeal: Appeal) -> bool {
        let today = self.state.time.as_seconds() / SECONDS_PER_DAY;
        let now = self.state.time;
        let mine = self.my_views();
        let members = self.members();
        let literacy = self
            .state
            .settlement
            .as_ref()
            .map_or(0, |settlement| u32::from(settlement.literacy));
        let televised = self.knows_advance(Advance::Television);
        if let Some(settlement) = &mut self.state.settlement {
            let slot = appeal as usize;
            if settlement.appealed_on.len() <= slot {
                settlement.appealed_on.resize(slot + 1, None);
            }
            settlement.appealed_on[slot] = Some(today);
        }
        if appeal.coins() > 0 {
            self.charge(appeal.coins());
        }
        // Who of many is reached is settled by the day, so that it is not always the same.
        let reached = |id: PersonId, most: u32| {
            let every = (members.len() as u32).div_ceil(most).max(1);
            (u64::from(id.0) + today).is_multiple_of(u64::from(every))
        };
        // Who can read is as good as settled by birth, and the more can the more learn to.
        let reads = |id: PersonId| id.0 * 37 % 100 < literacy.max(10);

        match appeal {
            Appeal::Speech => {
                let (mut won, mut lost) = (0, 0);
                for id in &members {
                    let heard = reached(*id, Appeal::CROWD);
                    let Some(person) = self.state.world.person_mut(*id) else {
                        continue;
                    };
                    if !person.is_adult(now) || !heard {
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
                let guests = (members.len() as u32).min(Appeal::TABLE);
                self.spend_own_food(FEAST_FARE * guests);
                for id in &members {
                    let fed = reached(*id, Appeal::TABLE);
                    if let Some(person) = self.state.world.person_mut(*id)
                        && fed
                    {
                        person.warm_to(8);
                        person.hungry_days = 0;
                    }
                }
                self.note(
                    Tone::Good,
                    "You lay out a feast. Everyone at it eats their fill, and remembers who \
                     fed them.",
                );
            }
            Appeal::Notices => {
                if self.has(Item::BirchBark) >= 5 {
                    self.use_up(Item::BirchBark, 5);
                } else {
                    self.use_up(Item::Paper, 5);
                }
                for id in &members {
                    if let Some(person) = self.state.world.person_mut(*id)
                        && person.is_adult(now)
                        && reads(*id)
                    {
                        person.warm_to(2);
                    }
                }
                self.note(
                    Tone::Plain,
                    "You put up notices of what you stand for where everyone passes.",
                );
            }
            Appeal::Newspaper => {
                for id in &members {
                    if let Some(person) = self.state.world.person_mut(*id)
                        && person.is_adult(now)
                        && reads(*id)
                    {
                        let like_minded = agreement(&person.views, &mine) >= 2;
                        person.warm_to(if like_minded { 4 } else { 2 });
                    }
                }
                self.note(
                    Tone::Plain,
                    "What you stand for is in the newspaper for everyone who reads to see.",
                );
            }
            Appeal::Broadcast => {
                for id in &members {
                    if let Some(person) = self.state.world.person_mut(*id)
                        && person.is_adult(now)
                    {
                        person.warm_to(if televised { 4 } else { 2 });
                    }
                }
                self.note(
                    Tone::Plain,
                    if televised {
                        "You speak to everyone at once, and they see you as you do."
                    } else {
                        "You speak over the air, and are heard in every house."
                    },
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
        } else if self.monetized() {
            if self.state.purse < HIRE_COINS {
                Err(format!(
                    "{first} asks {HIRE_COINS} coins for {HIRE_DAYS} days of it. You have {}.",
                    self.state.purse.max(0)
                ))
            } else {
                Ok(())
            }
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
        let paid_in_coin = self.monetized();
        if paid_in_coin {
            self.state.purse -= HIRE_COINS;
        } else {
            super::society::eat(self.state.player.carried_mut(), HIRE_FARE);
        }
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
            if paid_in_coin {
                format!("{first} takes your coins, and will think for you for {HIRE_DAYS} days.")
            } else {
                format!("{first} takes your food, and will think for you for {HIRE_DAYS} days.")
            },
        );
        true
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
        let term = self.term(era);
        let now = self.state.time;
        let mut leanings = [[0u32; 3]; 4];
        for (_, person) in world.people().filter(|(_, p)| p.member && p.is_adult(now)) {
            for (issue, lean) in person.views.iter().enumerate() {
                leanings[issue][(lean.signum() + 1) as usize] += 1;
            }
        }
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
            ayes: Law::ALL.iter().map(|law| self.ayes(*law).0).collect(),
            needed: self.needed_voices(self.grown_ups().len() as u32),
            leanings,
            offices: Office::ALL
                .into_iter()
                .map(|office| {
                    let held = settlement.offices.iter().find(|(held, _)| *held == office);
                    let holder = held
                        .and_then(|(_, holder)| world.person(*holder))
                        .filter(|holder| holder.is_present() && holder.member)
                        .map(|holder| (holder.name.full(), holder.has(office.suits())));
                    (office, holder)
                })
                .collect(),
            trusted: self
                .trusted()
                .into_iter()
                .map(|(id, person)| {
                    let traits: Vec<&str> = person.traits.iter().map(|t| t.name()).collect();
                    let words = if traits.is_empty() {
                        person.name.full()
                    } else {
                        format!(
                            "{} ({})",
                            person.name.full(),
                            traits.join(", ").to_lowercase()
                        )
                    };
                    (id, words)
                })
                .collect(),
            campaigning: era.campaigns()
                && (CALLING_DAY..MOOT_DAY).contains(&date.ordinal())
                && next_moot.is_some_and(|days| days < 40),
            grown_ups: self.grown_ups().len() as u32,
            decides_alone: self.decides_alone(),
            next_moot,
            steward: settlement.steward,
            credit: settlement.credit,
            views: self.my_views(),
            appeal_rest: Appeal::ALL
                .iter()
                .map(|appeal| self.appeal_rest(*appeal))
                .collect(),
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
