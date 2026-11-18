use std::sync::Arc;

use crate::action::{AtHand, Job};
use crate::climate::{Conditions, Temperature};
use crate::journal::Entry;
use crate::map::Map;
use crate::perception::Surroundings;
use crate::space::{Length, Location};
use crate::state::Death;
use crate::world::Kind;
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
