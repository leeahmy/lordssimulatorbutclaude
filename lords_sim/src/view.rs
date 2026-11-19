use std::sync::Arc;

use crate::action::{AtHand, Job};
use crate::climate::{Conditions, Temperature};
use crate::item::Inventory;
use crate::item::Item;
use crate::journal::Entry;
use crate::map::Map;
use crate::neighbour::Neighbour;
use crate::object::Sex;
use crate::people::{Bond, PersonId, Role, Trait, WORKING_AGE};
use crate::perception::Surroundings;
use crate::politics::{Era, Law, Office, Views};
use crate::settlement::{Advance, Age, Chronicled, Ledger, Running};
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
    /// The coins the player character has, which are fewer than none when they owe.
    pub purse: i64,
    /// The other settlements of the wider world.
    pub neighbours: Vec<Neighbour>,
    /// The shares the player character holds in workplaces of each kind, and what they
    /// paid the day before.
    pub holdings: Vec<(StructureKind, u32)>,
    pub income: i64,
    /// The kin of the player character, dead or alive.
    pub kin: Arc<[KinView]>,
}

/// What the player is shown of one of their kin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KinView {
    pub name: String,
    /// What they are to the player character.
    pub relation: &'static str,
    /// How old they are, or were when they died.
    pub age: u64,
    pub alive: bool,
    /// Whether they live in the settlement.
    pub here: bool,
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
    pub traits: Vec<Trait>,
    /// How good they are at their trade, out of a hundred.
    pub skill: u8,
    /// Whether they are a child learning the trade they are set to.
    pub apprenticed: bool,
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
    /// The age the settlement has reached by what it knows.
    pub age: Age,
    /// Whether coin is known, and whether goods are bought and sold for it at a market.
    pub coined: bool,
    pub monetized: bool,
    pub treasury: i64,
    /// The share of what people earn that is raised in taxes, in hundredths.
    pub tax: u8,
    /// What came into the treasury and went out of it the day before.
    pub ledger: Ledger,
    /// What the merchants have been asked to buy from outside.
    pub orders: Vec<(Item, u32)>,
    /// How many in a hundred of the grown-ups can read.
    pub literacy: u8,
    /// How content people are with their lot, out of a hundred.
    pub contentment: u8,
    pub running: Running,
    /// How many hundredths of what it yields without, work yields with the tools in the
    /// store.
    pub tools: u32,
    /// How many work at each trade, and how many places there are to work at it for
    /// trades that take a building.
    pub jobs: Vec<(Role, u32, Option<u32>)>,
    /// How many raiders the settlement can stand against.
    pub defence: u32,
    /// How much thieving and brawling there is, out of a hundred.
    pub crime: u8,
    /// How foul the air and the water are, out of a hundred.
    pub pollution: u32,
    /// What has happened that is worth remembering, oldest first.
    pub chronicle: Vec<Chronicled>,
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
    pub ayes: Vec<u32>,
    /// How many of them a law takes to be made.
    pub needed: u32,
    /// How many of the grown-ups lean to the one side of each issue, to neither, and to
    /// the other, in the order of [`Issue::ALL`](crate::politics::Issue::ALL).
    pub leanings: [[u32; 3]; 4],
    /// Whether an election is a month or less away.
    pub campaigning: bool,
    /// Who holds each office, in the order of [`Office::ALL`], and whether they are of
    /// the kind that holds it best.
    pub offices: Vec<(Office, Option<(String, bool)>)>,
    /// Those who could be given an office: the grown-ups held in the highest esteem,
    /// with their traits in words.
    pub trusted: Vec<(PersonId, String)>,
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
    pub appeal_rest: Vec<u64>,
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
