use std::sync::Arc;

use crate::perception::Surroundings;
use crate::space::Length;
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
    /// `None` unless the player character is walking somewhere.
    pub walk: Option<Walk>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Walk {
    pub toward: Kind,
    pub remaining: Length,
}
