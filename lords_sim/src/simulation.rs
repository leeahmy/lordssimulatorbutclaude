mod life;
mod politics;
mod society;

use std::sync::Arc;

use crate::action::{AtHand, Job, Verb};
use crate::climate::Conditions;
use crate::craft::Recipe;
use crate::item::{HEAVY_LOAD, Item};
use crate::journal::{Entry, Tone};
use crate::map::{self, Map, Surface};
use crate::object::{Activity, Human, Living, Name, Sex};
use crate::people::{PersonId, Role};
use crate::perception::{self, Surroundings};
use crate::politics::{Appeal, Issue, Law};
use crate::route;
use crate::settlement::Advance;
use crate::space::{Length, Location};
use crate::state::State;
use crate::time::{TICKS_PER_SECOND, Timestamp};
use crate::view::{PersonView, SettlementView, View, Walk, Work};
use crate::world::index::Grid;
use crate::world::{Area, AreaKind, StructureKind, Subject, Thing, ThingId, World};

/// How far a fire lights up the night.
const FIRELIGHT: Length = Length::from_meters(30);

/// How far from a fire its light still falls on someone.
const FIRESIDE: Length = Length::from_meters(5);

/// How fast someone walks through snow and under a heavy load, in hundredths of how fast
/// they walk otherwise.
const SNOW_PACE: i64 = 80;
const LOADED_PACE: i64 = 75;

pub struct Simulation {
    state: State,
    grid: Grid,
    /// The triangles covering each area of the world, in the order of the areas.
    surfaces: Vec<Surface>,
    surroundings: Arc<Surroundings>,
    map: Arc<Map>,
    in_sight: Vec<ThingId>,
    route: Vec<Location>,
    /// What the sky and the air were like when last looked at, which is every minute.
    conditions: Conditions,
    at_hand: Arc<AtHand>,
    /// The journal of the state as it is shown.
    journal: Arc<[Entry]>,
    /// The piles and the structures of the world, which change with time.
    kept: Vec<ThingId>,
    /// Whether what the player perceives may have changed since they last looked.
    stale: bool,
    /// Whether something has happened that the player should not miss.
    alert: bool,
    /// What the player is shown of the other people, nearest to them at heart first.
    census: Arc<[PersonView]>,
    town: Option<Arc<SettlementView>>,
    /// Whether what is shown of the others and of the settlement may have changed.
    society_stale: bool,
}

impl Simulation {
    /// Returns a simulation of a player in an empty world.
    pub fn new(name: Name, sex: Sex) -> Self {
        Self::from_state(State::new(name, sex))
    }

    /// Returns a simulation of a player setting out on a life in the world that
    /// [`World::generate`] makes of `seed`.
    pub fn generate(name: Name, sex: Sex, seed: u64) -> Self {
        Self::from_state(State::generate(name, sex, seed))
    }

    pub fn from_state(state: State) -> Self {
        let grid = Grid::new(&state.world);
        let surfaces = state.world.areas().map(|_| Surface::default()).collect();
        let kept = state
            .world
            .things()
            .filter(|(_, thing)| matches!(thing, Thing::Pile(_) | Thing::Structure(_)))
            .map(|(id, _)| id)
            .collect();

        let mut simulation = Self {
            conditions: state.climate.conditions(state.time),
            journal: state.journal.entries().into(),
            state,
            grid,
            surfaces,
            surroundings: Arc::default(),
            map: Arc::default(),
            in_sight: Vec::new(),
            route: Vec::new(),
            at_hand: Arc::default(),
            kept,
            stale: false,
            alert: false,
            census: Arc::from(Vec::new()),
            town: None,
            society_stale: false,
        };
        simulation.ensure_protagonist();
        simulation.refresh_society();
        simulation.look();
        simulation
    }

    /// Returns the state of the simulation.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Returns whether something has happened since this was last asked that the player
    /// should not miss, such as their life being in danger.
    pub fn take_alert(&mut self) -> bool {
        std::mem::take(&mut self.alert)
    }

    pub fn view(&self) -> View {
        let player = &self.state.player;
        let world = &self.state.world;
        let activity = player.activity();

        let walk = activity.and_then(|activity| match activity {
            Activity::Walking {
                toward,
                destination,
            } => {
                let mut remaining = Length::ZERO;
                let mut at = player.location();
                for stop in self.route.iter().copied().chain([destination]) {
                    remaining = remaining + at.distance_to(stop);
                    at = stop;
                }

                Some(Walk {
                    toward: world.kind(toward)?,
                    destination,
                    via: self.route.clone(),
                    remaining,
                    intent: player.intent(),
                })
            }
            Activity::Doing { .. } | Activity::Sleeping => None,
        });
        let work = activity.and_then(|activity| match activity {
            Activity::Doing { job, done } => Some(Work {
                job,
                subject: match job {
                    Job::Act { subject, .. } => world.kind(subject),
                    Job::Craft(_) | Job::Consume(_) | Job::EatSnow | Job::Appeal(_) => None,
                },
                done,
                needed: self.needed(job),
            }),
            Activity::Walking { .. } | Activity::Sleeping => None,
        });
        let effort = self.effort();

        View {
            time: self.state.time,
            player: player.clone(),
            surroundings: Arc::clone(&self.surroundings),
            map: Arc::clone(&self.map),
            walk,
            work,
            sleeping: activity == Some(Activity::Sleeping),
            conditions: self.conditions,
            felt: self.felt().warmer_by(effort.warmth()),
            at_hand: Arc::clone(&self.at_hand),
            journal: Arc::clone(&self.journal),
            started: self.state.started,
            walked: Length::from_millimeters(self.state.walked as i64),
            death: self.state.death,
            people: Arc::clone(&self.census),
            settlement: self.town.clone(),
            routine: self.state.routine,
            past_lives: self.state.past_lives,
        }
    }

    /// Advances the simulation by one tick. Nothing moves on once the player has died.
    pub fn tick(&mut self) {
        if self.state.death.is_some() {
            return;
        }

        self.state.time.advance();
        match self.state.player.activity() {
            Some(Activity::Walking { .. }) => self.walk(),
            Some(Activity::Doing { .. }) => self.work(),
            Some(Activity::Sleeping) | None => {}
        }
        if self.state.time.as_seconds().is_multiple_of(60) {
            self.pass_minute();
        }
        if self.stale {
            self.look();
        }
        if self.society_stale {
            self.refresh_society();
        }
    }

    /// Applies a change requested from outside the simulation. Once the player character
    /// has died, the only change there is to make is to live on as someone else.
    pub fn apply(&mut self, command: Command) {
        if self.state.death.is_some() {
            if let Command::LiveOn(heir) = command {
                self.live_on(heir);
                self.look();
                self.refresh_society();
            }
            return;
        }

        match command {
            Command::Reset => self.state.time = Timestamp::EPOCH,
            Command::WalkTo(subject) => {
                self.set_aside();
                self.walk_to(subject, None);
            }
            Command::Stop => self.set_aside(),
            Command::Act { verb, subject } => self.take_up(Job::Act { verb, subject }),
            Command::Craft(recipe) => self.take_up(Job::Craft(recipe)),
            Command::Consume(item) => self.take_up(Job::Consume(item)),
            Command::EatSnow => self.take_up(Job::EatSnow),
            Command::Sleep => self.lie_down(),
            Command::Drop(item, count) => self.drop(item, count),
            Command::Take(item, count) => self.take(item, count),
            Command::Assign(person, role) => self.assign(person, role),
            Command::Plan(kind) => self.plan(kind),
            Command::Pursue(advance) => self.pursue(advance),
            Command::NameSettlement(name) => self.rename_settlement(name),
            Command::SetRoutine(routine) => self.state.routine = routine,
            Command::SetView(issue, lean) => self.set_view(issue, lean),
            Command::Legislate(law, wanted) => self.legislate(law, wanted),
            Command::SetSteward(steward) => self.set_steward(steward),
            Command::Appeal(appeal) => self.take_up(Job::Appeal(appeal)),
            Command::LiveOn(_) => {}
        }
        if self.stale {
            self.look();
        }
        if self.society_stale {
            self.refresh_society();
        }
    }

    /// Sends the player walking to `subject`, to set about `intent` once there.
    fn walk_to(&mut self, subject: Subject, intent: Option<Job>) {
        if !self.knows(subject) {
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
        self.state.player.set_intent(activity.and(intent));
        self.plan_route();
    }

    fn walk(&mut self) {
        let Some(Activity::Walking { destination, .. }) = self.state.player.activity() else {
            return;
        };
        let start = self.state.player.location();
        let mut at = start;
        let mut left = self.stride();
        let mut learned = false;

        while left > Length::ZERO {
            let target = self.route.first().copied().unwrap_or(destination);
            let next = at.toward(target, left);

            if let Some((id, lake)) = self.state.world.lake_at(next) {
                let memory = &mut self.state.memory;
                let found = lake
                    .nearest_on_shore(next)
                    .map(|(edge, _)| edge)
                    .filter(|edge| !memory.knows_edge(id, *edge));
                match found {
                    Some(edge) => {
                        memory.note_edge(id, edge);
                        learned = true;
                    }
                    None => self.state.player.animal_mut().set_activity(None),
                }
                break;
            }

            left = left - at.distance_to(next);
            at = next;
            if next != target {
                break;
            }
            if self.route.is_empty() {
                self.state.player.animal_mut().set_activity(None);
                break;
            }
            self.route.remove(0);
        }

        if at != start || learned {
            self.state.walked += start.distance_to(at).as_millimeters() as u64;
            self.state.player.animal_mut().move_to(at);
            self.look();
        }
        if self.state.player.activity().is_none() {
            self.arrive();
        }
    }

    /// Returns the ground the player covers in a tick of walking.
    fn stride(&self) -> Length {
        let player = &self.state.player;
        let mut stride = Human::WALKING_SPEED / TICKS_PER_SECOND as i64;

        stride = stride * i64::from(player.body().pace_percent()) / 100;
        if self.conditions.snow {
            stride = stride * SNOW_PACE / 100;
        }
        if player.carried().weight() > HEAVY_LOAD {
            stride = stride * LOADED_PACE / 100;
        }
        stride
    }

    fn plan_route(&mut self) {
        let Some(Activity::Walking { destination, .. }) = self.state.player.activity() else {
            self.route.clear();
            return;
        };

        self.route = route::plan(self.state.player.location(), destination, &self.lakes());
    }

    fn lakes(&self) -> Vec<Area> {
        self.state
            .world
            .areas()
            .zip(&self.surfaces)
            .filter(|((_, area), _)| area.kind() == AreaKind::Lake)
            .map(|((_, area), surface)| surface.shape(area))
            .filter(|shape| shape.outline().len() >= 2)
            .collect()
    }

    /// Takes in what the player sees from where they are, and commits it to memory.
    fn look(&mut self) {
        self.stale = false;
        let sight = self.sight();
        let cairn = self
            .state
            .settlement
            .as_ref()
            .map(|settlement| settlement.cairn);
        let advances = self
            .state
            .settlement
            .as_ref()
            .map_or_else(Vec::new, |settlement| settlement.advances.clone());
        let State {
            player,
            world,
            memory,
            time,
            ..
        } = &mut self.state;
        let eye = player.location();
        let mut seen = perception::look(world, &self.grid, eye, sight);
        perception::look_at_people(world, &self.grid, eye, sight, &mut seen.surroundings);

        for thing in self.in_sight.iter().chain(&seen.things) {
            memory.note_thing(*thing, *time);
        }
        for (area, edge) in &seen.shores {
            memory.note_edge(*area, *edge);
        }
        let mut shore_learned = false;
        for ((id, area), surface) in world.areas().zip(&mut self.surfaces) {
            shore_learned |= surface.update(id, area, memory);
        }

        self.map = Arc::new(map::chart(
            world,
            &self.grid,
            &self.surfaces,
            memory,
            &seen,
            eye,
        ));
        self.at_hand = Arc::new(at_hand(world, &seen.surroundings, cairn, advances));
        self.surroundings = Arc::new(seen.surroundings);
        self.in_sight = seen.things;
        if shore_learned {
            self.plan_route();
        }
    }

    /// Returns how far the player sees in the light there is.
    fn sight(&self) -> Length {
        let by_daylight = self.conditions.light.sight();
        if by_daylight >= FIRELIGHT {
            return by_daylight;
        }

        let eye = self.state.player.location();
        let by_a_fire = self.kept.iter().any(|id| {
            self.state.world.structure(*id).is_some_and(|structure| {
                structure.kind() == StructureKind::Campfire
                    && eye.distance_to(structure.location()) <= FIRESIDE
            })
        });

        if by_a_fire { FIRELIGHT } else { by_daylight }
    }

    /// Writes in the journal.
    fn note(&mut self, tone: Tone, text: impl Into<String>) {
        self.state.journal.write(self.state.time, tone, text.into());
        self.journal = self.state.journal.entries().into();
    }
}

/// Returns what is within reach of someone whose surroundings are `surroundings`.
///
/// The common store counts as within reach by the `cairn` of the settlement, and in its
/// storehouses.
fn at_hand(
    world: &World,
    surroundings: &Surroundings,
    cairn: Option<ThingId>,
    advances: Vec<Advance>,
) -> AtHand {
    let mut at_hand = AtHand {
        advances,
        ..AtHand::default()
    };

    for sighting in &surroundings.within_reach {
        let id = match sighting.subject {
            Subject::Thing(id) => id,
            Subject::Area(_) => {
                at_hand.water = true;
                continue;
            }
            Subject::Person(_) => continue,
        };

        match world.thing(id) {
            Some(Thing::Pile(pile)) => {
                for stack in pile.items().stacks() {
                    at_hand.loose.add_stack(*stack);
                }
                at_hand.piles.push(id);
            }
            Some(Thing::Structure(structure)) => match structure.kind() {
                StructureKind::Campfire => {
                    at_hand.fire = true;
                    at_hand.fire_until = at_hand.fire_until.max(Some(structure.until()));
                }
                StructureKind::Bed => at_hand.bed = true,
                StructureKind::Well => at_hand.water = true,
                kind @ (StructureKind::Kiln | StructureKind::Smithy) => {
                    at_hand.workplaces.push(kind);
                }
                StructureKind::Cairn | StructureKind::Storehouse if at_hand.store.is_none() => {
                    let store = cairn.and_then(|cairn| Some((cairn, world.structure(cairn)?)));
                    if let Some((cairn, store)) = store {
                        for stack in store.hold().stacks() {
                            at_hand.loose.add_stack(*stack);
                        }
                        at_hand.store = Some(cairn);
                    }
                }
                kind if kind.is_shelter() => {
                    let better = at_hand.shelter.is_none_or(|shelter| !shelter.is_house());
                    if better {
                        at_hand.shelter = Some(kind);
                    }
                }
                _ => {}
            },
            Some(Thing::Plant(_) | Thing::Rock(_)) | None => {}
        }
    }

    at_hand
}

/// A change to the simulation requested from outside it, such as by the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Rewinds the simulation clock to [`Timestamp::EPOCH`].
    Reset,
    /// Sends the player walking to something they know of until it is within reach.
    WalkTo(Subject),
    /// Stops the player's current action, or wakes them up.
    Stop,
    /// Has the player do something to a subject they know of, walking to it first if it
    /// is out of reach.
    Act {
        verb: Verb,
        subject: Subject,
    },
    /// Has the player make something of what they carry and what lies within reach.
    Craft(Recipe),
    /// Has the player eat or drink one of an item they carry.
    Consume(Item),
    /// Has the player eat snow off the ground for the water in it.
    EatSnow,
    /// Has the player lie down to sleep where they are.
    Sleep,
    /// Has the player put down some of what they carry.
    Drop(Item, u32),
    /// Has the player take some of what lies within their reach, the common store of
    /// their settlement included.
    Take(Item, u32),
    /// Gives someone who lives with the player something to do with their days.
    Assign(PersonId, Role),
    /// Plans a building of the settlement, to be built of what is in its store.
    Plan(StructureKind),
    /// Sets the settlement to working out an advance.
    Pursue(Advance),
    NameSettlement(String),
    /// Has the player character see to eating, drinking, and sleeping by themselves,
    /// or leaves that to the player again.
    SetRoutine(bool),
    /// Has the player live on as someone who lived with their character that has died.
    LiveOn(PersonId),
    /// Sets how far the player character leans to which side of an issue, from -2 to 2.
    SetView(Issue, i8),
    /// Puts a law in force, or takes it out of force, if the player leads and the
    /// grown-ups of the settlement have it.
    Legislate(Law, bool),
    /// Leaves the running of the settlement the player leads to custom, or takes it back.
    SetSteward(bool),
    /// Has the player set about winning over the people of the settlement.
    Appeal(Appeal),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Mark;
    use crate::object::Object;
    use crate::world::{Kind, Plant, PlantSpecies, Rock};

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

    /// Ticks until the player stops walking, and returns where they were after each tick.
    fn walk_until_done(sim: &mut Simulation) -> Vec<Location> {
        let mut path = Vec::new();
        while sim.state().player.activity().is_some() {
            assert!(path.len() < 1_000, "the walk never ended");
            sim.tick();

            let at = sim.state().player.location();
            assert!(
                !sim.state().world.is_water(at),
                "walked into water at {at:?}"
            );
            path.push(at);
        }
        path
    }

    fn within_reach(sim: &Simulation, subject: Subject) -> bool {
        let view = sim.view();
        view.surroundings
            .within_reach
            .iter()
            .any(|sighting| sighting.subject == subject)
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
            Some(Walk {
                toward: Kind::Plant(PlantSpecies::Birch),
                destination: Location::new(0, 19_450),
                via: Vec::new(),
                remaining: Length::from_millimeters(19_450),
                intent: None,
            })
        );

        sim.tick();
        assert_eq!(sim.state().player.location(), Location::new(0, 1_400));

        assert_eq!(walk_until_done(&mut sim).len(), 13);
        assert_eq!(sim.state().player.location(), Location::new(0, 19_450));
        assert_eq!(sim.view().walk, None);
        assert_eq!(sim.view().surroundings.within_reach.len(), 1);
    }

    /// A simulation of a player who has set off for a tree behind a lake, a little east of
    /// straight across it, along with that tree.
    fn walk_across_water() -> (Simulation, Subject) {
        let mut world = World::default();
        world.add_area(lake(30_000, 50_000));
        let tree = Subject::Thing(world.add_thing(birch(5_000, 60_000)));
        let mut sim = simulation(world);
        sim.apply(Command::WalkTo(tree));

        (sim, tree)
    }

    #[test]
    fn walk_goes_around_water_by_the_shorter_side() {
        let (mut sim, tree) = walk_across_water();
        assert_eq!(sim.view().walk.map(|walk| walk.via.len()), Some(2));

        let path = walk_until_done(&mut sim);

        assert!(path.iter().any(|at| at.x() > 20_000));
        assert!(path.iter().all(|at| at.x() > -20_000));
        assert!(within_reach(&sim, tree));
    }

    #[test]
    fn walk_turns_aside_once_the_water_in_the_way_is_seen() {
        let mut world = World::default();
        world.add_area(lake(310_000, 330_000));
        let tree = world.add_thing(birch(5_000, 340_000));
        let mut state = State {
            world,
            ..State::new(Name::default(), Sex::Female)
        };
        state.memory.note_thing(tree, Timestamp::EPOCH);
        let mut sim = Simulation::from_state(state);

        sim.apply(Command::WalkTo(Subject::Thing(tree)));
        assert_eq!(sim.view().walk.map(|walk| walk.via), Some(Vec::new()));

        let path = walk_until_done(&mut sim);

        assert!(path.iter().any(|at| at.x() > 20_000));
        assert!(within_reach(&sim, Subject::Thing(tree)));
    }

    #[test]
    fn walk_around_water_carries_on_from_a_state() {
        let (mut sim, _) = walk_across_water();
        for _ in 0..30 {
            sim.tick();
        }

        let mut resumed = Simulation::from_state(sim.state().clone());

        assert_eq!(resumed.view().walk, sim.view().walk);
        assert_eq!(walk_until_done(&mut resumed), walk_until_done(&mut sim));
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
    fn map_reaches_as_far_diagonally_as_it_does_straight() {
        let mut world = World::default();
        let walked_to = Subject::Thing(world.add_thing(tall_boulder(250_000, 0)));
        let off_to_the_corner = Subject::Thing(world.add_thing(tall_boulder(0, 250_000)));
        let left_behind = Subject::Thing(world.add_thing(tall_boulder(-60_000, 0)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(walked_to));
        walk_until_done(&mut sim);

        assert!(mark(&sim, off_to_the_corner).is_some());
        assert_eq!(mark(&sim, left_behind), None);
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
