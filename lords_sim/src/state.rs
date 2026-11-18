use serde::{Deserialize, Serialize};

use crate::body::Cause;
use crate::climate::Climate;
use crate::item::Item;
use crate::journal::{Journal, Tone};
use crate::memory::Memory;
use crate::object::{Human, Living, Name, Sex};
use crate::rng::Rng;
use crate::space::{Bearing, Location};
use crate::time::Timestamp;
use crate::time::gregorian::{SECONDS_PER_DAY, SECONDS_PER_HOUR};
use crate::world::World;

/// Version of [`State`] as it is written to disk.
///
/// Is raised with any change to [`State`], or to anything inside it, that would make a
/// state saved before the change read back wrongly or not at all. Adding a field marked
/// `#[serde(default)]` does not need it.
pub const STATE_VERSION: u32 = 2;

/// The first morning of a generated life: eight in the morning on 1 May of year 21, on
/// which someone born at [`Timestamp::EPOCH`] is twenty years old.
pub const FIRST_MORNING: Timestamp =
    Timestamp::from_seconds((20 * 365 + 5 + 120) * SECONDS_PER_DAY + 8 * SECONDS_PER_HOUR);

/// What a generated life starts with in hand.
const OUTFIT: [(Item, u32); 3] = [(Item::Knife, 1), (Item::FireSteel, 1), (Item::RyeBread, 4)];

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

    #[serde(default)]
    pub climate: Climate,

    /// When the life being lived began.
    #[serde(default)]
    pub started: Timestamp,

    #[serde(default)]
    pub journal: Journal,

    /// How the player died, once they have.
    #[serde(default)]
    pub death: Option<Death>,

    /// How far the player has walked, in millimeters.
    #[serde(default)]
    pub walked: u64,

    #[serde(default)]
    pub rng: Rng,
}

/// The end of a life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Death {
    pub time: Timestamp,
    pub cause: Cause,
}

impl State {
    /// Returns the state of a player in an empty world.
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            player: Human::new(Location::default(), Timestamp::EPOCH, name, sex),
            time: Timestamp::EPOCH,
            world: World::default(),
            memory: Memory::default(),
            climate: Climate::Still,
            started: Timestamp::EPOCH,
            journal: Journal::default(),
            death: None,
            walked: 0,
            rng: Rng::default(),
        }
    }

    /// Returns the state of a player setting out on [`FIRST_MORNING`] in the world that
    /// [`World::generate`] makes of `seed`.
    pub fn generate(name: Name, sex: Sex, seed: u64) -> Self {
        let mut state = Self {
            world: World::generate(seed),
            ..Self::new(name, sex)
        };
        state.set_out(seed);
        state
    }

    /// Returns this state, which is of a player who could do nothing but walk, as that of
    /// one setting out on a life in the same world, no earlier than [`FIRST_MORNING`].
    ///
    /// States were like that up to version 1.
    pub fn brought_to_life(mut self) -> Self {
        let seed = self.world.things().count() as u64 ^ self.time.as_seconds();
        self.set_out(seed);
        self
    }

    fn set_out(&mut self, seed: u64) {
        self.time = self.time.max(FIRST_MORNING);
        self.started = self.time;
        self.climate = Climate::Boreal { seed };
        self.rng = Rng::new(seed);
        for (item, count) in OUTFIT {
            self.player.carried_mut().add(item, count);
        }
        let mut words = "You are alone in the forest with a knife, a fire steel, and a little \
                         bread. Winter is half a year away."
            .to_owned();
        let at = self.player.location();
        let shores = self.world.areas().map(|(_, area)| area.nearest_shore(at));
        if let Some(shore) = shores.min_by_key(|shore| at.distance_to(*shore)) {
            let way = match at.bearing_to(shore) {
                Bearing::North => "north",
                Bearing::Northeast => "northeast",
                Bearing::East => "east",
                Bearing::Southeast => "southeast",
                Bearing::South => "south",
                Bearing::Southwest => "southwest",
                Bearing::West => "west",
                Bearing::Northwest => "northwest",
            };
            words.push_str(&format!(" You know of a lake somewhere to the {way}."));
        }
        self.journal.write(self.time, Tone::Plain, words);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::Season;

    use crate::time::gregorian::{Date, TimeOfDay};

    #[test]
    fn generated_life_starts_on_a_spring_morning_at_the_age_of_twenty() {
        let state = State::generate(Name::default(), Sex::Female, 3);
        let today = Date::from_timestamp(state.time);

        assert_eq!(state.time, FIRST_MORNING);
        assert_eq!(state.started, FIRST_MORNING);
        assert_eq!(today.to_string(), "0021-05-01");
        assert_eq!(
            TimeOfDay::from_timestamp(state.time).to_string(),
            "08:00:00"
        );
        assert_eq!(
            today.years_since(Date::from_timestamp(state.player.birth())),
            20
        );
        assert_eq!(state.climate.conditions(state.time).season, Season::Spring);
        assert!(state.player.carried().has(Item::Knife));
        assert_eq!(state.journal.entries().len(), 1);
        assert!(
            state.journal.entries()[0]
                .text
                .contains("a lake somewhere to the")
        );
    }

    #[test]
    fn old_state_is_brought_to_life_where_it_was() {
        let mut old = State::new(Name::default(), Sex::Male);
        old.time = Timestamp::from_seconds(500);

        let state = old.clone().brought_to_life();

        assert_eq!(state.time, FIRST_MORNING);
        assert_eq!(state.player.location(), old.player.location());
        assert!(matches!(state.climate, Climate::Boreal { .. }));
        assert!(state.player.carried().has(Item::FireSteel));

        let mut late = old;
        late.time = Timestamp::from_seconds(FIRST_MORNING.as_seconds() + 500);
        assert_eq!(late.clone().brought_to_life().time, late.time);
    }

    #[test]
    fn state_written_before_there_was_more_to_life_than_walking_still_reads() {
        let old = "(player:(animal:(object:(location:(x:0,y:0),creation:0),\
                   name:(first:\"Ada\",middle:[],last:\"Lord\"),sex:Female)),time:7)";

        let read: State = ron::from_str(old).expect("the old state decodes");

        let mut expected = State::new(
            Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned()),
            Sex::Female,
        );
        expected.time = Timestamp::from_seconds(7);
        assert_eq!(read, expected);
    }
}
