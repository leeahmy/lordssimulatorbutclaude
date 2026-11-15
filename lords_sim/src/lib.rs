pub mod object;
pub mod perception;
pub mod space;
pub mod time;
pub mod view;
pub mod world;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::object::{Activity, Living};
use crate::perception::Surroundings;
use crate::time::TICKS_PER_SECOND;
use crate::world::index::Grid;

pub use crate::object::{Human, Name, Sex};
pub use crate::space::{Bearing, Length, Location};
pub use crate::time::Timestamp;
pub use crate::view::View;
pub use crate::world::{Subject, World};

/// Version of [`State`] as it is written to disk.
///
/// Is raised with any change to [`State`], or to anything inside it, that would make a
/// state saved before the change read back wrongly or not at all: removing or renaming a
/// field, changing its type, or changing what its value means. Adding a field marked
/// `#[serde(default)]` does not need it.
pub const STATE_VERSION: u32 = 1;

pub struct Simulation {
    state: State,
    grid: Grid,
    surroundings: Arc<Surroundings>,
}

impl Simulation {
    /// Returns a simulation of a player in an empty world.
    pub fn new(name: Name, sex: Sex) -> Self {
        Self::from_state(State::new(name, sex))
    }

    /// Returns a simulation of a player in the world that [`World::generate`] makes of `seed`.
    pub fn generate(name: Name, sex: Sex, seed: u64) -> Self {
        Self::from_state(State {
            world: World::generate(seed),
            ..State::new(name, sex)
        })
    }

    pub fn from_state(state: State) -> Self {
        let grid = Grid::new(&state.world);
        let surroundings = perception::look(&state.world, &grid, state.player.location());

        Self {
            state,
            grid,
            surroundings: Arc::new(surroundings),
        }
    }

    /// Returns the state of the simulation.
    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn view(&self) -> View {
        let player = &self.state.player;
        let walk = player.activity().and_then(|activity| match activity {
            Activity::Walking {
                toward,
                destination,
            } => Some(view::Walk {
                toward: self.state.world.kind(toward)?,
                remaining: player.location().distance_to(destination),
            }),
        });

        View {
            time: self.state.time,
            player: player.clone(),
            surroundings: Arc::clone(&self.surroundings),
            walk,
        }
    }

    /// Advances the simulation by one tick.
    pub fn tick(&mut self) {
        self.state.time.advance();
        self.walk();
    }

    /// Applies a change requested from outside the simulation.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Reset => self.state.time = Timestamp::EPOCH,
            Command::WalkTo(subject) => self.walk_to(subject),
            Command::Stop => self.state.player.animal_mut().set_activity(None),
        }
    }

    fn walk_to(&mut self, subject: Subject) {
        if self.surroundings.sighting(subject).is_none() {
            return;
        }

        let from = self.state.player.location();
        let destination = self.state.world.approach(subject, from, Human::REACH);
        let activity = destination
            .filter(|destination| *destination != from)
            .map(|destination| Activity::Walking {
                toward: subject,
                destination,
            });

        self.state.player.animal_mut().set_activity(activity);
    }

    /// Moves a walking player a tick's worth closer to where they are going.
    fn walk(&mut self) {
        let Some(Activity::Walking { destination, .. }) = self.state.player.activity() else {
            return;
        };
        let step = Length::from_millimeters(
            Human::WALKING_SPEED.as_millimeters() / TICKS_PER_SECOND as i64,
        );
        let next = self.state.player.location().toward(destination, step);
        let blocked = self.state.world.is_water(next);

        let player = self.state.player.animal_mut();
        if blocked || next == destination {
            player.set_activity(None);
        }
        if !blocked {
            player.move_to(next);
            self.look();
        }
    }

    fn look(&mut self) {
        let eye = self.state.player.location();
        self.surroundings = Arc::new(perception::look(&self.state.world, &self.grid, eye));
    }
}

/// A change to the simulation requested from outside it, such as by the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Rewinds the simulation clock to [`Timestamp::EPOCH`].
    Reset,
    /// Sends the player walking to something until it is within reach.
    WalkTo(Subject),
    /// Stops the player's current action.
    Stop,
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

    #[serde(default)]
    pub world: World,
}

impl State {
    /// Returns the state of a player in an empty world.
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            player: Human::new(Default::default(), Default::default(), name, sex),
            time: Timestamp::EPOCH,
            world: World::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Object;
    use crate::world::{Area, AreaKind, Kind, Plant, PlantSpecies, Thing};

    fn birch(x: i64, y: i64) -> Thing {
        Thing::Plant(Plant::new(
            Object::new(Location::new(x, y), Default::default()),
            PlantSpecies::Birch,
            Length::from_meters(15),
            Length::from_millimeters(300),
        ))
    }

    fn lake(south: i64, north: i64) -> Area {
        let corners = [
            (-20_000, south),
            (20_000, south),
            (20_000, north),
            (-20_000, north),
        ];
        Area::new(
            AreaKind::Lake,
            corners
                .into_iter()
                .map(|(x, y)| Location::new(x, y))
                .collect(),
        )
    }

    fn simulation(world: World) -> Simulation {
        Simulation::from_state(State {
            world,
            ..State::new(Name::default(), Sex::Female)
        })
    }

    fn walk_until_done(sim: &mut Simulation) -> u32 {
        let mut ticks = 0;
        while sim.state().player.activity().is_some() {
            assert!(ticks < 1_000, "the walk never ended");
            sim.tick();
            ticks += 1;
        }
        ticks
    }

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
        assert!(view.surroundings.is_empty());
        assert_eq!(view.walk, None);
    }

    #[test]
    fn generated_simulation_starts_with_things_in_sight() {
        let sim = Simulation::generate(Name::default(), Sex::Male, 5);

        assert!(!sim.view().surroundings.is_empty());
        assert_eq!(sim.state().player.location(), Location::default());
    }

    #[test]
    fn walk_covers_walking_speed_each_tick_and_ends_within_reach() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(tree));
        assert_eq!(
            sim.view().walk,
            Some(view::Walk {
                toward: Kind::Plant(PlantSpecies::Birch),
                remaining: Length::from_millimeters(19_450),
            })
        );

        sim.tick();
        assert_eq!(sim.state().player.location(), Location::new(0, 1_400));

        assert_eq!(walk_until_done(&mut sim), 13);
        assert_eq!(sim.state().player.location(), Location::new(0, 19_450));
        assert_eq!(sim.view().walk, None);
        assert_eq!(sim.view().surroundings.within_reach.len(), 1);
    }

    #[test]
    fn walk_ends_at_the_edge_of_water() {
        let mut world = World::default();
        world.add_area(lake(30_000, 50_000));
        let tree = Subject::Thing(world.add_thing(birch(0, 60_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(tree));
        walk_until_done(&mut sim);

        let stopped_at = sim.state().player.location();
        assert_eq!(stopped_at, Location::new(0, 29_400));
        assert!(!sim.state().world.is_water(stopped_at));
    }

    #[test]
    fn walk_to_a_lake_ends_with_the_water_within_reach() {
        let mut world = World::default();
        let water = Subject::Area(world.add_area(lake(30_000, 50_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(water));
        walk_until_done(&mut sim);

        let view = sim.view();
        assert_eq!(view.surroundings.within_reach.len(), 1);
        assert_eq!(view.surroundings.within_reach[0].subject, water);
    }

    #[test]
    fn stopping_ends_a_walk_where_it_has_got_to() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let mut sim = simulation(world);
        sim.apply(Command::WalkTo(tree));
        sim.tick();

        sim.apply(Command::Stop);
        sim.tick();

        assert_eq!(sim.state().player.activity(), None);
        assert_eq!(sim.state().player.location(), Location::new(0, 1_400));
    }

    #[test]
    fn nobody_walks_to_what_they_do_not_perceive() {
        let mut world = World::default();
        world.add_thing(birch(0, 5_000));
        let hidden = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(hidden));

        assert_eq!(sim.state().player.activity(), None);
    }

    #[test]
    fn walking_to_what_is_within_reach_goes_nowhere() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(0, 500)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(tree));

        assert_eq!(sim.state().player.activity(), None);
    }
}
