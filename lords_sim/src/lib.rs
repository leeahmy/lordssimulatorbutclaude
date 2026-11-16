pub mod map;
pub mod memory;
pub mod object;
pub mod perception;
pub mod space;
pub mod time;
pub mod view;
pub mod world;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::map::{Map, Surface};
use crate::memory::Memory;
use crate::object::{Activity, Living};
use crate::perception::Surroundings;
use crate::time::TICKS_PER_SECOND;
use crate::world::index::Grid;

pub use crate::object::{Human, Name, Sex};
pub use crate::space::{Bearing, Length, Location};
pub use crate::time::Timestamp;
pub use crate::view::View;
use crate::world::ThingId;
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
    /// The triangles covering each area of the world, in the order of the areas.
    surfaces: Vec<Surface>,
    surroundings: Arc<Surroundings>,
    map: Arc<Map>,
    /// The things the player saw when they last looked.
    in_sight: Vec<ThingId>,
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
        let surfaces = state.world.areas().map(|_| Surface::default()).collect();

        let mut simulation = Self {
            state,
            grid,
            surfaces,
            surroundings: Arc::default(),
            map: Arc::default(),
            in_sight: Vec::new(),
        };
        simulation.look();
        simulation
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
                destination,
                remaining: player.location().distance_to(destination),
            }),
        });

        View {
            time: self.state.time,
            player: player.clone(),
            surroundings: Arc::clone(&self.surroundings),
            map: Arc::clone(&self.map),
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
        if !self.state.memory.knows(subject) {
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

    /// Takes in what the player sees from where they are, and commits it to memory.
    fn look(&mut self) {
        let State {
            player,
            world,
            memory,
            time,
        } = &mut self.state;
        let eye = player.location();
        let seen = perception::look(world, &self.grid, eye);

        for thing in self.in_sight.iter().chain(&seen.things) {
            memory.note_thing(*thing, *time);
        }
        for (area, edge) in &seen.shores {
            memory.note_edge(*area, *edge);
        }
        for ((id, area), surface) in world.areas().zip(&mut self.surfaces) {
            surface.update(id, area, memory);
        }

        self.map = Arc::new(map::chart(
            world,
            &self.grid,
            &self.surfaces,
            memory,
            &seen,
            eye,
        ));
        self.surroundings = Arc::new(seen.surroundings);
        self.in_sight = seen.things;
    }
}

/// A change to the simulation requested from outside it, such as by the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Rewinds the simulation clock to [`Timestamp::EPOCH`].
    Reset,
    /// Sends the player walking to something they know of until it is within reach.
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

    #[serde(default)]
    pub memory: Memory,
}

impl State {
    /// Returns the state of a player in an empty world.
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            player: Human::new(Default::default(), Default::default(), name, sex),
            time: Timestamp::EPOCH,
            world: World::default(),
            memory: Memory::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Mark;
    use crate::object::Object;
    use crate::world::{Area, AreaKind, Kind, Plant, PlantSpecies, Rock, Thing};

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

    fn tall_boulder(x: i64, y: i64) -> Thing {
        Thing::Rock(Rock::new(
            Object::new(Location::new(x, y), Default::default()),
            Length::from_millimeters(2_500),
            Length::from_meters(2),
        ))
    }

    fn mark(sim: &Simulation, subject: Subject) -> Option<Mark> {
        let map = sim.view().map;
        map.things
            .iter()
            .copied()
            .find(|mark| mark.subject == subject)
    }

    /// A simulation of a player who has walked south out of sight of a tree to the north,
    /// along with that tree, one that was behind it all the way, and the one walked to.
    fn walk_away() -> (Simulation, [Subject; 3]) {
        let mut world = World::default();
        let left_behind = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let hidden = Subject::Thing(world.add_thing(birch(0, 40_000)));
        let walked_to = Subject::Thing(world.add_thing(birch(0, -85_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(walked_to));
        walk_until_done(&mut sim);

        (sim, [left_behind, hidden, walked_to])
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
        assert_eq!(*view.map, Map::default());
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
                destination: Location::new(0, 19_450),
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
    fn map_marks_what_is_in_sight_where_it_stands() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(3_000, 20_000)));
        let sim = simulation(world);

        assert_eq!(
            mark(&sim, tree),
            Some(Mark {
                subject: tree,
                kind: Kind::Plant(PlantSpecies::Birch),
                location: Location::new(3_000, 20_000),
                footprint: Length::from_millimeters(150),
                last_seen: None,
            })
        );
    }

    #[test]
    fn map_keeps_what_was_seen_and_never_has_what_was_not() {
        let (sim, [left_behind, hidden, walked_to]) = walk_away();

        assert!(sim.view().surroundings.sighting(left_behind).is_none());
        assert_eq!(
            mark(&sim, left_behind).map(|mark| mark.in_sight()),
            Some(false)
        );
        assert_eq!(
            mark(&sim, walked_to).map(|mark| mark.in_sight()),
            Some(true)
        );
        assert_eq!(mark(&sim, hidden), None);
    }

    #[test]
    fn thing_left_behind_is_remembered_from_when_it_went_out_of_sight() {
        let (mut sim, [left_behind, ..]) = walk_away();
        let lost = Some(Timestamp::from_seconds(51));
        assert_eq!(
            mark(&sim, left_behind).map(|mark| mark.last_seen),
            Some(lost)
        );

        for _ in 0..100 {
            sim.tick();
        }

        assert_eq!(
            mark(&sim, left_behind).map(|mark| mark.last_seen),
            Some(lost)
        );
    }

    #[test]
    fn thing_watched_while_standing_still_is_remembered_from_when_the_player_left() {
        let mut world = World::default();
        let barely_seen = Subject::Thing(world.add_thing(birch(0, 90_000)));
        let walked_to = Subject::Thing(world.add_thing(birch(0, -50_000)));
        let mut sim = simulation(world);
        for _ in 0..100 {
            sim.tick();
        }
        assert_eq!(
            mark(&sim, barely_seen).map(|mark| mark.last_seen),
            Some(None)
        );

        sim.apply(Command::WalkTo(walked_to));
        sim.tick();

        assert_eq!(
            mark(&sim, barely_seen).map(|mark| mark.last_seen),
            Some(Some(Timestamp::from_seconds(101)))
        );
    }

    #[test]
    fn memory_is_part_of_the_state_a_simulation_carries_on_from() {
        let (sim, [left_behind, ..]) = walk_away();

        let resumed = Simulation::from_state(sim.state().clone());

        assert_eq!(resumed.state(), sim.state());
        assert_eq!(resumed.view().map, sim.view().map);
        assert!(mark(&resumed, left_behind).is_some());
    }

    #[test]
    fn walk_can_head_for_what_is_only_remembered() {
        let (mut sim, [left_behind, hidden, _]) = walk_away();

        sim.apply(Command::WalkTo(hidden));
        assert_eq!(sim.state().player.activity(), None);

        sim.apply(Command::WalkTo(left_behind));
        walk_until_done(&mut sim);
        assert_eq!(sim.state().player.location(), Location::new(0, 19_450));
    }

    #[test]
    fn shore_is_mapped_where_it_has_been_seen() {
        let mut world = World::default();
        let water = Subject::Area(world.add_area(lake(30_000, 50_000)));
        world.add_thing(tall_boulder(4_000, 10_000));
        world.add_thing(tall_boulder(-4_000, 10_000));
        let mut sim = simulation(world);
        let corner = |x, y| Location::new(x, y);

        let map = sim.view().map;
        assert_eq!(map.waters.len(), 1);
        assert_eq!(map.waters[0].subject, water);
        assert_eq!(map.waters[0].kind, Kind::Lake);
        assert_eq!(
            map.waters[0].shore,
            [
                [corner(-20_000, 30_000), corner(20_000, 30_000)],
                [corner(20_000, 30_000), corner(20_000, 50_000)],
                [corner(-20_000, 50_000), corner(-20_000, 30_000)],
            ]
        );
        assert_eq!(map.waters[0].surface.len(), 2);

        sim.apply(Command::WalkTo(water));
        walk_until_done(&mut sim);

        assert_eq!(sim.view().map.waters[0].shore.len(), 4);
    }

    #[test]
    fn water_is_cut_straight_across_where_its_shore_has_not_been_seen() {
        let mut world = World::default();
        let water = Subject::Area(world.add_area(lake(30_000, 50_000)));
        world.add_thing(tall_boulder(-4_000, 6_000));
        world.add_thing(tall_boulder(-4_000, 10_000));
        world.add_thing(tall_boulder(4_000, 10_000));
        let mut sim = simulation(world);
        let southwest = Location::new(-20_000, 30_000);
        let southeast = Location::new(20_000, 30_000);
        let northeast = Location::new(20_000, 50_000);

        let map = sim.view().map;
        assert_eq!(
            map.waters[0].shore,
            [[southwest, southeast], [southeast, northeast]]
        );
        assert_eq!(map.waters[0].surface.len(), 1);
        for corner in [southwest, southeast, northeast] {
            assert!(map.waters[0].surface[0].contains(&corner));
        }

        sim.apply(Command::WalkTo(water));
        walk_until_done(&mut sim);

        assert_eq!(sim.view().map.waters[0].surface.len(), 2);
    }

    #[test]
    fn single_stretch_of_shore_has_no_water_drawn_inside_it() {
        let mut world = World::default();
        world.add_area(lake(30_000, 50_000));
        for x in [-4_000, 4_000] {
            world.add_thing(tall_boulder(x, 6_000));
            world.add_thing(tall_boulder(x, 10_000));
        }

        let map = simulation(world).view().map;

        assert_eq!(map.waters[0].shore.len(), 1);
        assert!(map.waters[0].surface.is_empty());
    }

    #[test]
    fn water_never_seen_is_not_on_the_map() {
        let mut world = World::default();
        world.add_area(lake(400_000, 420_000));

        assert!(simulation(world).view().map.waters.is_empty());
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
