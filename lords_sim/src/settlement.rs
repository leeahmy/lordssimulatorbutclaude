//! The place the player and those who live with them have made their own, what they have
//! built there, and what they have learned.

use serde::{Deserialize, Serialize};

pub use crate::advance::{Advance, Age};
pub use crate::building::{BUILDINGS, Building, FARMERS_PER_FIELD, building, houses, places};
use crate::item::Item;
use crate::people::PersonId;
use crate::politics::{Law, Office};
use crate::time::Timestamp;
use crate::world::ThingId;

/// How many hours of work one builder puts in in a day.
pub const WORKDAY: u32 = 8;

/// The share of what people earn that is raised in taxes unless something else is set,
/// and the most that can be, in hundredths.
pub const USUAL_TAX: u8 = 10;
pub const HIGHEST_TAX: u8 = 50;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settlement {
    pub name: String,
    /// The pile of stones the settlement was founded with, which holds its common store.
    pub cairn: ThingId,
    pub founded: Timestamp,
    /// What has been learned, in the order it was.
    #[serde(default)]
    pub advances: Vec<Advance>,
    /// What is being worked towards.
    #[serde(default)]
    pub pursuit: Option<Advance>,
    /// How far the pursuit has got, in tenths of a day of one thinker's work.
    #[serde(default)]
    pub insight: u32,
    /// The grain growing in the fields, to be had at harvest.
    #[serde(default)]
    pub growing: u32,
    /// Who leads, which is nobody only until the first moot after a leader is lost.
    #[serde(default)]
    pub leader: Option<PersonId>,
    #[serde(default)]
    pub laws: Vec<Law>,
    /// Whether the leader leaves the running of things to custom: who does what, what is
    /// built, and what is worked out. A leader who is not the player character always does.
    #[serde(default)]
    pub steward: bool,
    /// The year in which a leader was last chosen.
    #[serde(default)]
    pub moot_year: u64,
    /// The days, counted from the epoch, on which the player character last made each
    /// appeal, in the order of [`Appeal::ALL`](crate::politics::Appeal::ALL).
    #[serde(default)]
    pub appealed_on: Vec<Option<u64>>,
    /// What the player character has brought to the common store over what they have
    /// taken out of it, in kilocalories and their worth.
    #[serde(default)]
    pub credit: i64,
    /// How much of `credit` people have taken note of already.
    #[serde(default)]
    pub noted_credit: i64,
    /// The coins in the treasury, once there is coin.
    #[serde(default)]
    pub treasury: i64,
    /// The share of what people earn that is raised in taxes, in hundredths.
    #[serde(default = "usual_tax")]
    pub tax: u8,
    /// What came into the treasury and went out of it the day before.
    #[serde(default)]
    pub ledger: Ledger,
    /// What whoever leads has asked the merchants to buy from outside.
    #[serde(default)]
    pub orders: Vec<(Item, u32)>,
    /// How many in a hundred of the grown-ups can read.
    #[serde(default)]
    pub literacy: u8,
    /// How content people are with their lot, out of a hundred.
    #[serde(default = "middling")]
    pub contentment: u8,
    /// How much use the tools in the store have seen since one last wore out.
    #[serde(default)]
    pub wear: u32,
    /// How many came to settle, were born, and died since the last time it was told.
    #[serde(default)]
    pub tally: [u32; 3],
    /// Who holds which office.
    #[serde(default)]
    pub offices: Vec<(Office, PersonId)>,
    /// How much thieving and brawling there is, out of a hundred.
    #[serde(default)]
    pub crime: u8,
    /// The day, counted from the epoch, until which a hard winter lasts.
    #[serde(default)]
    pub hard_winter_until: u64,
    /// What has happened that is worth remembering, oldest first.
    #[serde(default)]
    pub chronicle: Vec<Chronicled>,
    /// The era the settlement had reached when that was last looked at, counted from a
    /// band.
    #[serde(default)]
    pub era_reached: u8,
    /// What part of the work that takes it was done with power, with tools, and with
    /// tractors the day before, which the next day's work goes by.
    #[serde(default)]
    pub running: Running,
}

const fn usual_tax() -> u8 {
    USUAL_TAX
}

const fn middling() -> u8 {
    50
}

/// Something that happened to a settlement, and when.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chronicled {
    pub time: Timestamp,
    pub text: String,
}

/// What came into a treasury and went out of it in a day, in coins.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    /// What was raised in taxes, on what people earned and on what they sold outside.
    pub taxes: i64,
    /// What was sold outside, which is the gain of those who made it and not of the
    /// treasury.
    pub sales: i64,
    pub interest: i64,
    pub wages: i64,
    /// What keeping up what stands built cost.
    #[serde(default)]
    pub upkeep: i64,
    pub purchases: i64,
    /// Whether there was not enough to pay everyone who is paid out of the treasury.
    pub unpaid: bool,
}

impl Ledger {
    pub const fn balance(&self) -> i64 {
        self.taxes + self.interest - self.wages - self.upkeep - self.purchases
    }
}

/// What a settlement had going for it when it last did a day's work.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Running {
    /// Whether a power station was manned and fuelled.
    pub power: bool,
    /// Whether tractors were kept running.
    pub tractors: bool,
    /// Whether ale was poured at a tavern.
    pub tavern: bool,
    /// How many healers, teachers, clerks, and millers were at work.
    pub healers: u32,
    pub teachers: u32,
    pub clerks: u32,
    pub millers: u32,
    /// How many coins' worth the merchants, sailors, and transport workers can trade in
    /// a day.
    pub trade: u32,
    /// Whether there was nothing to heat the houses with when it last froze.
    #[serde(default)]
    pub unheated: bool,
    /// How many guards were on watch.
    #[serde(default)]
    pub guards: u32,
    /// How foul the air and the water are with what is burned and made, out of a
    /// hundred.
    #[serde(default)]
    pub pollution: u32,
    /// How many priests kept the rites, and how many performers performed.
    #[serde(default)]
    pub priests: u32,
    #[serde(default)]
    pub performers: u32,
}

impl Settlement {
    pub fn new(name: String, cairn: ThingId, founded: Timestamp) -> Self {
        Self {
            name,
            cairn,
            founded,
            advances: Vec::new(),
            pursuit: None,
            insight: 0,
            growing: 0,
            leader: None,
            laws: Vec::new(),
            steward: false,
            moot_year: 0,
            appealed_on: Vec::new(),
            credit: 0,
            noted_credit: 0,
            treasury: 0,
            tax: USUAL_TAX,
            ledger: Ledger::default(),
            orders: Vec::new(),
            literacy: 0,
            contentment: 50,
            wear: 0,
            tally: [0; 3],
            offices: Vec::new(),
            crime: 0,
            hard_winter_until: 0,
            chronicle: Vec::new(),
            era_reached: 0,
            running: Running::default(),
        }
    }

    pub fn lives_by(&self, law: Law) -> bool {
        self.laws.contains(&law)
    }

    pub fn knows(&self, advance: Advance) -> bool {
        self.advances.contains(&advance)
    }

    /// Returns the age the settlement has reached.
    pub fn age(&self) -> Age {
        Age::of(&self.advances)
    }

    /// Returns whether `advance` is neither known nor out of reach for want of another.
    pub fn can_pursue(&self, advance: Advance) -> bool {
        !self.knows(advance) && advance.requires().iter().all(|needed| self.knows(*needed))
    }

    /// Returns how many hundredths of what it does without, work yields with the tools
    /// the settlement knows how to make.
    pub fn tool_percent(&self) -> u32 {
        if self.knows(Advance::Steel) {
            140
        } else if self.knows(Advance::Ironworking) {
            125
        } else if self.knows(Advance::Bronze) {
            115
        } else {
            100
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::StructureKind;

    fn settlement() -> Settlement {
        let mut world = crate::World::default();
        let cairn = world.add_thing(crate::world::Thing::Structure(
            crate::world::Structure::new(Default::default(), StructureKind::Cairn),
        ));

        Settlement::new("Lordstead".to_owned(), cairn, Timestamp::EPOCH)
    }

    #[test]
    fn advances_build_on_one_another() {
        let mut settlement = settlement();
        assert!(settlement.can_pursue(Advance::Pottery));
        assert!(!settlement.can_pursue(Advance::Masonry));
        assert!(!settlement.can_pursue(Advance::Bronze));

        settlement.advances.push(Advance::Pottery);
        assert!(!settlement.can_pursue(Advance::Pottery));
        assert!(settlement.can_pursue(Advance::Masonry));
        assert!(settlement.can_pursue(Advance::Writing));
        assert!(settlement.can_pursue(Advance::Charcoal));
        assert!(!settlement.can_pursue(Advance::Bronze));
    }

    #[test]
    fn better_metal_makes_for_better_tools() {
        let mut settlement = settlement();
        assert_eq!(settlement.tool_percent(), 100);
        assert_eq!(settlement.age(), Age::Stone);

        settlement
            .advances
            .extend([Advance::Bronze, Advance::Mining]);
        assert_eq!(settlement.tool_percent(), 115);
        assert_eq!(settlement.age(), Age::Bronze);

        settlement.advances.push(Advance::Ironworking);
        assert_eq!(settlement.tool_percent(), 125);
        settlement.advances.push(Advance::Steel);
        assert_eq!(settlement.tool_percent(), 140);
    }

    #[test]
    fn ledger_balances_what_came_in_against_what_went_out() {
        let ledger = Ledger {
            taxes: 40,
            sales: 25,
            interest: 5,
            wages: 30,
            upkeep: 0,
            purchases: 50,
            unpaid: false,
        };

        assert_eq!(ledger.balance(), -35);
        assert_eq!(Ledger::default().balance(), 0);
    }

    #[test]
    fn settlement_written_before_there_was_coin_still_reads() {
        let old = "(name:\"Koivula\",cairn:3,founded:0,appealed_on:[Some(4),None,None])";

        let read: Settlement = ron::from_str(old).expect("the old settlement decodes");

        assert_eq!(read.appealed_on, [Some(4), None, None]);
        assert_eq!(read.tax, USUAL_TAX);
        assert_eq!(read.contentment, 50);
        assert_eq!(read.treasury, 0);
    }
}
