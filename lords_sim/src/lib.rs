pub mod object;
pub mod time;
pub mod view;

use serde::{Deserialize, Serialize};

pub use crate::object::{Human, Name, Sex};
pub use crate::time::Timestamp;
pub use crate::view::View;

/// Version of [`State`] as it is written to disk.
///
/// Is raised with any change to [`State`], or to anything inside it, that would make a
/// state saved before the change read back wrongly or not at all: removing or renaming a
/// field, changing its type, or changing what its value means. Adding a field marked
/// `#[serde(default)]` does not need it.
pub const STATE_VERSION: u32 = 1;

pub struct Simulation {
    state: State,
}

impl Simulation {
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            state: State::new(name, sex),
        }
    }

    pub fn from_state(state: State) -> Self {
        Self { state }
    }

    /// Returns the state of the simulation.
    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn view(&self) -> View {
        View {
            time: self.state.time,
            player: self.state.player.clone(),
        }
    }

    /// Advances the simulation by one tick.
    pub fn tick(&mut self) {
        self.state.time.advance();
    }

    /// Applies a change requested from outside the simulation.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Reset => self.state.time = Timestamp::EPOCH,
        }
    }
}

/// A change to the simulation requested from outside it, such as by the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Rewinds the simulation clock to [`Timestamp::EPOCH`].
    Reset,
}

/// Everything there is to the simulation.
///
/// A state is cloned whenever the game is saved, while the simulation keeps running. The
/// clone must therefore stay cheap however large the world grows: anything that grows with
/// the world belongs in storage that clones share until one of them changes it, such as
/// chunks behind an [`Arc`](std::sync::Arc) changed through
/// [`Arc::make_mut`](std::sync::Arc::make_mut).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub player: Human,

    pub time: Timestamp,
}

impl State {
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            player: Human::new(Default::default(), Default::default(), name, sex),
            time: Timestamp::EPOCH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_carries_on_from_a_state() {
        let mut original = Simulation::new(Name::default(), Sex::Male);
        original.tick();
        original.tick();

        let mut resumed = Simulation::from_state(original.state().clone());
        assert_eq!(resumed.state(), original.state());

        resumed.tick();
        assert_eq!(resumed.state().time, Timestamp::from_seconds(3));
    }

    #[test]
    fn view_shows_the_player_at_the_current_time() {
        let name = Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned());
        let mut sim = Simulation::new(name, Sex::Female);
        sim.tick();

        let view = sim.view();

        assert_eq!(view.time, Timestamp::from_seconds(1));
        assert_eq!(view.player, sim.state().player);
    }
}
