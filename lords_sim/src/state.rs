use serde::{Deserialize, Serialize};

use crate::memory::Memory;
use crate::object::{Human, Name, Sex};
use crate::space::Location;
use crate::time::Timestamp;
use crate::world::World;

/// Version of [`State`] as it is written to disk.
///
/// Is raised with any change to [`State`], or to anything inside it, that would make a
/// state saved before the change read back wrongly or not at all. Adding a field marked
/// `#[serde(default)]` does not need it.
pub const STATE_VERSION: u32 = 1;

/// Everything there is to the simulation.
///
/// A state is cloned whenever the game is saved, while the simulation keeps running. The
/// clone must therefore stay cheap however large the world grows. Anything that grows with
/// the world belongs in storage that clones share until one of them changes it, such as
/// chunks behind an [`Arc`](std::sync::Arc) changed through
/// [`Arc::make_mut`](std::sync::Arc::make_mut).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub player: Human,

    pub time: Timestamp,

    #[serde(default)]
    pub world: World,

    #[serde(default)]
    pub memory: Memory,
}

impl State {
    /// Returns the state of a player in an empty world.
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            player: Human::new(Location::default(), Timestamp::EPOCH, name, sex),
            time: Timestamp::EPOCH,
            world: World::default(),
            memory: Memory::default(),
        }
    }
}
