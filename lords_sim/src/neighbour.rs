//! The other settlements of the wider world: too far off to walk to, but near enough to
//! trade with, to learn from, to fall out with, and to be raided by.

use serde::{Deserialize, Serialize};

use crate::advance::{Advance, Age};
use crate::rng::Rng;

/// The regard from which a neighbour will make a pact of trade, from which it will teach
/// what it knows, and at or below which it raids.
pub const TRUSTING: i8 = 30;
pub const GENEROUS: i8 = 40;
pub const HOSTILE: i8 = -30;

/// How many days pass before another envoy is received.
pub const ENVOY_REST: u64 = 30;

const NAMES: [&str; 12] = [
    "Ahola", "Ylämaa", "Kuusamo", "Petäjä", "Vaskila", "Suvanto", "Kaarna", "Lehtola", "Hirvas",
    "Routala", "Sorsala", "Tervala",
];

const WAYS: [&str; 8] = [
    "north",
    "northeast",
    "east",
    "southeast",
    "south",
    "southwest",
    "west",
    "northwest",
];

/// Another settlement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Neighbour {
    pub name: String,
    /// Which way it lies, counted in eighths of a turn from north.
    pub way: u8,
    /// How many days' walk away it lies.
    pub days_away: u8,
    pub people: u32,
    /// How many advances it knows, by which its age is told.
    pub advances: u8,
    /// How well it thinks of the player's settlement, from -100 to 100.
    pub regard: i8,
    /// Whether there is a pact of trade with it.
    #[serde(default)]
    pub pact: bool,
    /// The day, counted from the epoch, on which an envoy was last sent to it.
    #[serde(default)]
    pub envoy_on: Option<u64>,
}

impl Neighbour {
    /// Returns the neighbours a new world has: one wary, one well-disposed, and one that
    /// may go either way, all of them further along than someone starting out.
    pub(crate) fn generate(rng: &mut Rng) -> Vec<Self> {
        let first = rng.below(NAMES.len() as u64) as usize;
        let turn = rng.below(8) as u8;
        let regards = [(-45, -30), (10, 30), (-15, 15)];

        regards
            .into_iter()
            .enumerate()
            .map(|(index, regard)| Self {
                name: NAMES[(first + index * 5) % NAMES.len()].to_owned(),
                way: (turn + index as u8 * 3) % 8,
                days_away: rng.between((3, 12)) as u8,
                people: rng.between((30, 110)) as u32,
                advances: rng.between((5, 13)) as u8,
                regard: rng.between(regard) as i8,
                pact: false,
                envoy_on: None,
            })
            .collect()
    }

    /// Returns which way it lies, in words.
    pub fn way(&self) -> &'static str {
        WAYS[usize::from(self.way) % WAYS.len()]
    }

    /// Returns the age it has reached, as near as can be told from afar.
    pub fn age(&self) -> Age {
        let known = usize::from(self.advances).min(Advance::ALL.len());
        Age::of(&Advance::ALL[..known])
    }

    pub fn is_hostile(&self) -> bool {
        self.regard <= HOSTILE
    }

    /// Returns how it stands towards the player's settlement, in a word.
    pub fn stance(&self) -> &'static str {
        match self.regard {
            ..=HOSTILE => "hostile",
            -29..0 => "wary",
            0..TRUSTING => "civil",
            TRUSTING..GENEROUS => "friendly",
            _ => "close",
        }
    }

    /// Returns how many fighters it can send on a raid.
    pub fn raiders(&self) -> u32 {
        self.people / 20
    }

    pub(crate) fn warm_by(&mut self, by: i32) {
        self.regard = (i32::from(self.regard) + by).clamp(-100, 100) as i8;
    }

    /// Returns how many days are left before it receives another envoy, on day `today`.
    pub fn rest(&self, today: u64) -> u64 {
        self.envoy_on
            .map_or(0, |last| (last + ENVOY_REST).saturating_sub(today))
    }
}

/// What an envoy is sent to a neighbour for.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mission {
    /// Bringing gifts, to be thought better of.
    Gift,
    /// Agreeing to trade freely.
    Pact,
    /// Asking to be taught what they know.
    Learn,
    /// Falling on them with every guard there is.
    Raid,
}

impl Mission {
    pub const ALL: [Self; 4] = [Self::Gift, Self::Pact, Self::Learn, Self::Raid];

    /// What a gift costs: coins where there is coin, and food where there is none.
    pub const GIFT_COINS: i64 = 100;
    pub const GIFT_FOOD: u32 = 20_000;

    pub fn name(self) -> &'static str {
        match self {
            Self::Gift => "Send gifts",
            Self::Pact => "Offer a pact",
            Self::Learn => "Ask to be taught",
            Self::Raid => "Raid",
        }
    }

    pub fn telling(self) -> &'static str {
        match self {
            Self::Gift => {
                "Costs the treasury 100 coins, or the store 20,000 kcal of food where there \
                 is no coin. They think better of you for it."
            }
            Self::Pact => {
                "Takes a market and their being friendly. With a pact, a sixth more is \
                 traded, and they think a little better of you every year."
            }
            Self::Learn => {
                "Takes their being close, and their knowing more than you. Brings what is \
                 being worked out a fifth of the way on. They think a little less of you \
                 for the asking."
            }
            Self::Raid => {
                "Sends your guards against them. If they are beaten you take coin and food. \
                 Either way some may not come back, the pact is broken, and it is not \
                 forgotten."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_world_has_three_neighbours_of_different_minds() {
        let neighbours = Neighbour::generate(&mut Rng::new(4));

        assert_eq!(neighbours.len(), 3);
        assert!(neighbours[0].is_hostile());
        assert!(!neighbours[1].is_hostile() && neighbours[1].regard >= 10);
        for neighbour in &neighbours {
            assert!(!neighbour.name.is_empty() && !neighbour.way().is_empty());
            assert!(neighbour.people >= 30 && neighbour.days_away >= 3);
            assert!(!neighbour.pact);
        }
        assert_ne!(neighbours[0].name, neighbours[1].name);
        assert_eq!(Neighbour::generate(&mut Rng::new(4)), neighbours);
    }

    #[test]
    fn neighbour_is_told_by_its_regard_its_age_and_its_raiders() {
        let mut neighbour = Neighbour::generate(&mut Rng::new(4)).remove(1);
        neighbour.regard = 0;
        assert_eq!(neighbour.stance(), "civil");
        neighbour.warm_by(35);
        assert_eq!(neighbour.stance(), "friendly");
        neighbour.warm_by(500);
        assert_eq!((neighbour.regard, neighbour.stance()), (100, "close"));
        neighbour.warm_by(-140);
        assert_eq!(neighbour.stance(), "hostile");

        neighbour.advances = 2;
        assert_eq!(neighbour.age(), Age::Stone);
        neighbour.advances = 200;
        assert_eq!(neighbour.age(), Age::Modern);
        neighbour.people = 120;
        assert_eq!(neighbour.raiders(), 6);

        neighbour.envoy_on = Some(100);
        assert_eq!(neighbour.rest(110), 20);
        assert_eq!(neighbour.rest(400), 0);
        for mission in Mission::ALL {
            assert!(!mission.name().is_empty() && !mission.telling().is_empty());
        }
    }
}
