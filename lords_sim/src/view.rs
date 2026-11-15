use crate::{Human, Timestamp};

/// What the player is shown of the simulation.
///
/// A view holds only what the player character knows or perceives, so its size follows the
/// character's surroundings and never the size of the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub time: Timestamp,
    pub player: Human,
}
