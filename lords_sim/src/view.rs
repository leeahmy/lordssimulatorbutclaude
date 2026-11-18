use std::sync::Arc;

use crate::action::{AtHand, Job};
use crate::climate::{Conditions, Temperature};
use crate::item::Inventory;
use crate::journal::Entry;
use crate::map::Map;
use crate::object::Sex;
use crate::people::{Bond, PersonId, Role, WORKING_AGE};
use crate::perception::Surroundings;
use crate::politics::{Era, Law, Views};
use crate::settlement::Advance;
use crate::space::{Length, Location};
use crate::state::Death;
use crate::world::{Kind, StructureKind, Subject};
use crate::{Human, Timestamp};

/// What the player is shown of the simulation.
///
/// A view holds only what the player character knows or perceives, so its size follows the
/// character's surroundings and never the size of the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub time: Timestamp,
    pub player: Human,
    pub surroundings: Arc<Surroundings>,
    pub map: Arc<Map>,
    /// `None` unless the player character is walking somewhere.
    pub walk: Option<Walk>,
    /// `None` unless the player character is busy with something that takes a while.
    pub work: Option<Work>,
    pub sleeping: bool,
    pub conditions: Conditions,
    /// How warm it feels to the player character where they are, dressed as they are.
    pub felt: Temperature,
    pub at_hand: Arc<AtHand>,
    /// The latest things that have happened, oldest first.
    pub journal: Arc<[Entry]>,
    /// When the life being lived began.
    pub started: Timestamp,
    /// How far the player character has walked.
    pub walked: Length,
    pub death: Option<Death>,
    /// Everyone else alive and in the forest, those closest to the player character at
    /// heart first.
    pub people: Arc<[PersonView]>,
    pub settlement: Option<Arc<SettlementView>>,
    /// Whether the player character sees to eating, drinking, and sleeping by themselves.
    pub routine: bool,
    /// How many lives the player has lived before this one.
    pub past_lives: u32,
}

impl View {
    pub fn person(&self, subject: Subject) -> Option<&PersonView> {
        self.people.iter().find(|person| person.subject == subject)
    }

    /// Returns those the player could live on as if their character died now.
    pub fn heirs(&self) -> impl Iterator<Item = &PersonView> {
        self.people
            .iter()
            .filter(|person| person.member && person.age >= WORKING_AGE)
    }
}

/// What the player is shown of someone else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonView {
    pub id: PersonId,
    pub subject: Subject,
    pub name: String,
    pub sex: Sex,
    /// In full years.
    pub age: u64,
    pub bond: Bond,
    /// Whether they live with the player character.
    pub member: bool,
    /// Whether the player character has talked with them.
    pub met: bool,
    pub role: Role,
    /// Whether they are old enough to work.
    pub works: bool,
    /// How well they think of the player character, out of a hundred.
    pub regard: u8,
    pub expecting: bool,
    pub hungry: bool,
    /// The first name of their partner, unless that is the player character.
    pub partner: Option<String>,
    pub views: Views,
    /// How well the others of the settlement think of them, out of a hundred.
    pub esteem: u8,
    /// Whether the player character has hired them to think.
    pub hired: bool,
    /// Whether they would have the player character lead, if a leader were chosen now.
    pub backs_you: bool,
}

/// What the player is shown of their settlement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettlementView {
    pub name: String,
    pub founded: Timestamp,
    /// How many live there, the player character included.
    pub people: u32,
    /// How many the houses hold.
    pub housing: u32,
    /// How many days the food in the store lasts everyone.
    pub food_days: u32,
    pub store: Inventory,
    pub advances: Vec<Advance>,
    pub pursuit: Option<Advance>,
    /// How far the pursuit has got, in days of one thinker's work.
    pub insight: u32,
    /// The grain growing in the fields.
    pub growing: u32,
    /// What stands built, and how many of each.
    pub buildings: Vec<(StructureKind, u32)>,
    pub sites: Vec<SiteView>,
    pub politics: PoliticsView,
}

/// What the player is shown of how their settlement is led.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoliticsView {
    pub era: Era,
    /// The name of whoever leads.
    pub leader: Option<String>,
    /// Whether the player character leads.
    pub leads: bool,
    /// Those who would be named at a moot held now, the one with the most voices first.
    pub candidates: Vec<Candidate>,
    /// How many voices there are in all.
    pub voices: u32,
    pub laws: Vec<Law>,
    /// How many of the grown-ups are for each law, in the order of [`Law::ALL`].
    pub ayes: [u32; 8],
    /// How many grown-ups there are to have a law, the player character not counted.
    pub grown_ups: u32,
    /// Whether whoever leads makes the laws without asking.
    pub decides_alone: bool,
    /// In how many days a leader is next chosen, unless whoever leads does so for life.
    pub next_moot: Option<u64>,
    /// Whether the running of things is left to custom.
    pub steward: bool,
    /// What the player character has brought to the common store over what they have
    /// taken out of it, in kilocalories and their worth.
    pub credit: i64,
    /// The views of the player character.
    pub views: Views,
    /// In how many days each appeal can be made again, in the order of [`Appeal::ALL`].
    ///
    /// [`Appeal::ALL`]: crate::politics::Appeal::ALL
    pub appeal_rest: [u64; 3],
}

/// Someone who would be named at a moot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub name: String,
    pub voices: u32,
    /// Whether this is the player character.
    pub you: bool,
}

/// A building being built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SiteView {
    pub kind: StructureKind,
    /// How many hours of work have been put in, and how many it takes.
    pub work: u32,
    pub hours: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walk {
    pub toward: Kind,
    pub destination: Location,
    /// The places the walk turns at on the way to `destination` in order.
    pub via: Vec<Location>,
    pub remaining: Length,
    /// What the walk is for, if it is for more than getting there.
    pub intent: Option<Job>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Work {
    pub job: Job,
    /// What the job is done to, if it is done to something.
    pub subject: Option<Kind>,
    /// How many in-game seconds' worth of the job is done, and how many it takes.
    pub done: u32,
    pub needed: u32,
}
