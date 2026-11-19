use serde::{Deserialize, Serialize};

use crate::body::Cause;
use crate::climate::Climate;
use crate::item::Item;
use crate::journal::{Journal, Tone};
use crate::memory::Memory;
use crate::neighbour::Neighbour;
use crate::object::{Human, Living, Name, Object, Sex};
use crate::people::{Person, Role, family_name, given_name};
use crate::politics::random_views;
use crate::rng::Rng;
use crate::settlement::{Advance, Settlement};
use crate::space::Location;
use crate::time::Timestamp;
use crate::time::gregorian::{Date, SECONDS_PER_DAY, SECONDS_PER_HOUR};
use crate::world::{Structure, StructureKind, Thing, World};

/// Version of [`State`] as it is written to disk.
///
/// Is raised with any change to [`State`], or to anything inside it, that would make a
/// state saved before the change read back wrongly or not at all. Adding a field marked
/// `#[serde(default)]` does not need it.
pub const STATE_VERSION: u32 = 2;

/// The first morning of a generated life: eight in the morning on 1 May of year 81, late
/// enough for there to be old people who were born after [`Timestamp::EPOCH`].
pub const FIRST_MORNING: Timestamp =
    Timestamp::from_seconds((80 * 365 + 20 + 120) * SECONDS_PER_DAY + 8 * SECONDS_PER_HOUR);

/// How old the player character is when a generated life starts, in years, the life
/// starting on their birthday.
const STARTING_AGE: u64 = 20;

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

    /// The place the player and those who live with them have made their own, once they
    /// have founded one.
    #[serde(default)]
    pub settlement: Option<Settlement>,

    /// Whether the player character sees to eating, drinking, and sleeping by themselves.
    #[serde(default)]
    pub routine: bool,

    /// How many lives the player has lived before this one.
    #[serde(default)]
    pub past_lives: u32,

    /// The coins the player character has, which are fewer than none when they owe.
    #[serde(default)]
    pub purse: i64,

    /// The other settlements of the wider world.
    #[serde(default)]
    pub neighbours: Vec<Neighbour>,

    /// The shares the player character holds in workplaces of each kind.
    #[serde(default)]
    pub holdings: Vec<(StructureKind, u32)>,

    /// What those shares paid the day before.
    #[serde(default)]
    pub income: i64,
}

/// How a generated life starts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Start {
    /// Alone in the forest, with everything still to be built.
    #[default]
    Alone,
    /// As a newcomer to a hamlet that someone else leads.
    Hamlet,
}

impl Start {
    pub const ALL: [Self; 2] = [Self::Alone, Self::Hamlet];

    pub fn name(self) -> &'static str {
        match self {
            Self::Alone => "Alone in the wild",
            Self::Hamlet => "A newcomer in a hamlet",
        }
    }

    /// Returns what starting so comes to.
    pub fn telling(self) -> &'static str {
        match self {
            Self::Alone => {
                "Nobody for miles. Found a settlement of your own, win over the wanderers \
                 who pass, and build it up from a cairn of stones."
            }
            Self::Hamlet => {
                "Six people in a few huts, led by their eldest. Nobody knows you. Earn \
                 your standing among them, or strike out on your own."
            }
        }
    }
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
            settlement: None,
            routine: false,
            past_lives: 0,
            purse: 0,
            neighbours: Vec::new(),
            holdings: Vec::new(),
            income: 0,
        }
    }

    /// Returns the state of a player setting out on [`FIRST_MORNING`] in the world that
    /// [`World::generate`] makes of `seed`, as a newcomer to a hamlet.
    pub fn generate(name: Name, sex: Sex, seed: u64) -> Self {
        Self::begin(name, sex, seed, Start::Hamlet)
    }

    /// Returns the state of a player setting out on [`FIRST_MORNING`] in the world that
    /// [`World::generate`] makes of `seed`, the way `start` tells.
    pub fn begin(name: Name, sex: Sex, seed: u64, start: Start) -> Self {
        let mut state = Self {
            world: World::generate(seed),
            ..Self::new(name, sex)
        };
        state.set_out(seed, start);
        state
    }

    /// Returns this state, which is of a player who could do nothing but walk, as that of
    /// one setting out on a life in the same world, no earlier than [`FIRST_MORNING`].
    ///
    /// States were like that up to version 1.
    pub fn brought_to_life(mut self) -> Self {
        let seed = self.world.things().count() as u64 ^ self.time.as_seconds();
        self.set_out(seed, Start::Hamlet);
        self
    }

    /// Puts a hamlet by where the player stands: a cairn with a store, a few buildings,
    /// and half a dozen people led by their eldest. Returns its name and that of its
    /// leader, unless there is nowhere dry to put it.
    fn raise_hamlet(&mut self) -> Option<(String, String)> {
        const PLACES: [&str; 8] = [
            "Koivula",
            "Kuusela",
            "Honkala",
            "Rantala",
            "Mäntylä",
            "Niemelä",
            "Kivelä",
            "Harjula",
        ];
        const STORE: [(Item, u32); 9] = [
            (Item::DriedFish, 500),
            (Item::DriedMeat, 150),
            (Item::Log, 40),
            (Item::Stick, 40),
            (Item::SpruceBough, 30),
            (Item::BirchBark, 12),
            (Item::Stone, 12),
            (Item::HarePelt, 6),
            (Item::Cordage, 6),
        ];
        // Whom the hamlet is made up of: their sex, age, and work, and whether they share
        // the family name of the one before them.
        const PEOPLE: [(Sex, u64, Role, bool); 6] = [
            (Sex::Female, 58, Role::Gatherer, false),
            (Sex::Male, 34, Role::Fisher, false),
            (Sex::Female, 31, Role::Trapper, true),
            (Sex::Male, 7, Role::Idle, true),
            (Sex::Male, 19, Role::Forager, false),
            (Sex::Female, 42, Role::Woodcutter, false),
        ];
        const PLOT: i64 = 9_000;

        let now = self.time;
        let at = self.player.location();
        let center = [
            (26_000, 14_000),
            (-26_000, 14_000),
            (14_000, -26_000),
            (-14_000, 28_000),
        ]
        .into_iter()
        .map(|(east, north)| Location::new(at.x() + east, at.y() + north))
        .find(|center| !self.world.is_water(*center))?;
        let plot = |east: i64, north: i64| {
            Location::new(center.x() + east * PLOT, center.y() + north * PLOT)
        };

        let mut cairn = Structure::new(Object::new(center, now), StructureKind::Cairn);
        for (item, count) in STORE {
            cairn.hold_mut().add(item, count);
        }
        let cairn = self.world.add_thing(Thing::Structure(cairn));
        let buildings = [
            (StructureKind::Hut, plot(1, 0)),
            (StructureKind::Hut, plot(-1, 0)),
            (StructureKind::Smokehouse, plot(0, 1)),
        ];
        for (kind, at) in buildings {
            if !self.world.is_water(at) {
                let structure = Structure::new(Object::new(at, now), kind);
                self.world.add_thing(Thing::Structure(structure));
            }
        }

        let mut ids = Vec::new();
        let mut family = "";
        for (index, (sex, age, role, kin)) in PEOPLE.into_iter().enumerate() {
            if !kin {
                family = family_name(&mut self.rng);
            }
            let given = given_name(&mut self.rng, sex).to_owned();
            let days = age * 365 + self.rng.between((0, 300)) as u64;
            let born = now.as_seconds().saturating_sub(days * SECONDS_PER_DAY);
            let home = Location::new(center.x() + 2_000 + 900 * index as i64, center.y() - 2_500);

            let mut person = Person::new(
                Name::new(given, Vec::new(), family.to_owned()),
                sex,
                Timestamp::from_seconds(born),
                home,
            );
            person.member = true;
            person.role = role;
            person.clothed = true;
            person.regard = self.rng.between((8, 18)) as u8;
            person.esteem = 20 + (age / 2) as u8;
            person.views = random_views(&mut self.rng);
            person.traits = crate::people::random_traits(&mut self.rng, &[]);
            person.skill = (age.min(40) as u8).saturating_mul(2).min(60);
            ids.push(self.world.add_person(person));
        }
        // The couple, and their child.
        for (one, other) in [(1, 2), (2, 1)] {
            if let Some(person) = self.world.person_mut(ids[one]) {
                person.partner = Some(ids[other]);
            }
        }
        if let Some(child) = self.world.person_mut(ids[3]) {
            child.parents = [Some(ids[2]), Some(ids[1])];
        }

        let name = PLACES[self.rng.below(PLACES.len() as u64) as usize].to_owned();
        let founded = now.as_seconds().saturating_sub(6 * 365 * SECONDS_PER_DAY);
        let mut settlement = Settlement::new(name.clone(), cairn, Timestamp::from_seconds(founded));
        settlement.advances.push(Advance::Smoking);
        settlement.leader = Some(ids[0]);
        settlement.moot_year = Date::from_timestamp(now).year();
        self.settlement = Some(settlement);

        let leader = self.world.person(ids[0])?.name.first().to_owned();
        Some((name, leader))
    }

    fn set_out(&mut self, seed: u64, start: Start) {
        self.time = self.time.max(FIRST_MORNING);
        self.started = self.time;
        // Twenty years to the day, five of them leap years or near enough.
        let born = self.time.as_seconds() - (STARTING_AGE * 365 + 5) * SECONDS_PER_DAY;
        self.player = Human::new(
            self.player.location(),
            Timestamp::from_seconds(born),
            self.player.name().clone(),
            self.player.sex(),
        );
        self.climate = Climate::Boreal { seed };
        self.rng = Rng::new(seed);
        for (item, count) in OUTFIT {
            self.player.carried_mut().add(item, count);
        }
        let at = self.player.location();
        let shores = self.world.areas().map(|(_, area)| area.nearest_shore(at));
        let lake = shores
            .min_by_key(|shore| at.distance_to(*shore))
            .map(|shore| at.bearing_to(shore).name());
        let hamlet = match start {
            Start::Alone => None,
            Start::Hamlet => self.raise_hamlet(),
        };

        self.neighbours = Neighbour::generate(&mut self.rng);
        let mut words = match &hamlet {
            Some((place, leader)) => format!(
                "You have come to {place}, a few huts in the forest, with a knife, a fire \
                 steel, and a little bread. {leader} leads here, and nobody knows you yet. \
                 Winter is half a year away."
            ),
            None => "You are alone in the forest with a knife, a fire steel, and a little \
                     bread. Nobody lives for miles around, though now and then someone \
                     passes through. Winter is half a year away."
                .to_owned(),
        };
        if let Some(way) = lake {
            words.push_str(&format!(" There is a lake to the {way}."));
        }
        self.journal.write(self.time, Tone::Plain, words);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::Season;

    use crate::time::gregorian::TimeOfDay;

    #[test]
    fn generated_life_starts_on_a_spring_morning_at_the_age_of_twenty() {
        let state = State::generate(Name::default(), Sex::Female, 3);
        let today = Date::from_timestamp(state.time);

        assert_eq!(state.time, FIRST_MORNING);
        assert_eq!(state.started, FIRST_MORNING);
        assert_eq!(today.to_string(), "0081-05-01");
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
                .contains("There is a lake to the")
        );
    }

    #[test]
    fn generated_life_starts_in_a_hamlet_led_by_its_eldest() {
        let state = State::generate(Name::default(), Sex::Female, 3);
        let settlement = state.settlement.as_ref().expect("there is a hamlet");
        let leader = settlement
            .leader
            .and_then(|leader| state.world.person(leader));
        let leader = leader.expect("someone leads it");
        let store = state
            .world
            .structure(settlement.cairn)
            .expect("it has a cairn");

        assert_eq!(state.world.people().count(), 6);
        assert!(state.world.people().all(|(_, person)| person.member));
        assert!(state.world.people().all(|(_, person)| person.regard < 30));
        assert!(leader.age(state.time) >= 58);
        assert!(store.hold().count(Item::DriedFish) > 100);
        assert!(settlement.knows(Advance::Smoking));
        assert!(state.journal.entries()[0].text.contains(&settlement.name));
        assert!(
            state.journal.entries()[0]
                .text
                .contains(leader.name.first())
        );
        assert_eq!(State::generate(Name::default(), Sex::Female, 3), state);
    }

    #[test]
    fn life_can_start_alone_with_nobody_and_nothing_built() {
        let state = State::begin(Name::default(), Sex::Male, 3, Start::Alone);

        assert_eq!(state.settlement, None);
        assert_eq!(state.world.people().count(), 0);
        assert_eq!(state.time, FIRST_MORNING);
        assert!(state.player.carried().has(Item::Knife));
        assert!(state.journal.entries()[0].text.starts_with("You are alone"));
        assert_eq!(Start::default(), Start::Alone);
        for start in Start::ALL {
            assert!(!start.name().is_empty() && !start.telling().is_empty());
        }
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
