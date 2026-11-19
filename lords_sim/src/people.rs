//! The other people of the world: wanderers met in the wild, and those who live with the
//! player.
//!
//! Other people are not lived minute by minute the way the player character is. What becomes
//! of them is worked out a day at a time.

use serde::{Deserialize, Serialize};

use crate::object::{Name, Sex};
use crate::politics::{Views, random_views};
use crate::rng::Rng;
use crate::space::Location;
use crate::time::Timestamp;
use crate::time::gregorian::Date;

/// The regard from which someone is a friend, and from which they would share their life.
pub const FRIENDLY: u8 = 30;
pub const DEVOTED: u8 = 60;

/// The age from which someone works, and from which they are grown up.
pub const WORKING_AGE: u64 = 12;
pub const ADULT_AGE: u64 = 16;

/// The age from which a child can be apprenticed to a trade.
pub const APPRENTICE_AGE: u64 = 8;

/// The age after which a woman no longer bears children.
pub const FERTILE_UNTIL: u64 = 45;

const FEMALE_NAMES: [&str; 24] = [
    "Aino", "Elli", "Helka", "Ilta", "Kaisa", "Lempi", "Maija", "Noora", "Oili", "Pirkko",
    "Riitta", "Saara", "Tuuli", "Ulla", "Venla", "Aila", "Eeva", "Hilja", "Inka", "Kerttu",
    "Liisa", "Marja", "Sanni", "Taimi",
];

const MALE_NAMES: [&str; 24] = [
    "Aatos", "Eero", "Heikki", "Ilmari", "Jouko", "Kalle", "Lauri", "Mikko", "Niilo", "Olavi",
    "Paavo", "Reino", "Sakari", "Tapio", "Urho", "Veikko", "Antti", "Eino", "Jussi", "Kauko",
    "Matti", "Otso", "Pekka", "Toivo",
];

const FAMILY_NAMES: [&str; 20] = [
    "Aalto", "Harju", "Järvi", "Koski", "Lahti", "Mäki", "Niemi", "Oja", "Pelto", "Ranta", "Saari",
    "Salo", "Suo", "Vaara", "Virta", "Kallio", "Lehto", "Metsä", "Kangas", "Korpi",
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonId(pub(crate) u32);

/// What someone is to the player character.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bond {
    Stranger,
    Friend,
    /// Someone who lives with the player character.
    Companion,
    Partner,
    Child,
}

impl Bond {
    pub fn name(self) -> &'static str {
        match self {
            Self::Stranger => "Stranger",
            Self::Friend => "Friend",
            Self::Companion => "Companion",
            Self::Partner => "Partner",
            Self::Child => "Child",
        }
    }
}

/// Something about the way someone is, which they are born with or grow into.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trait {
    /// Gets good at a trade twice as fast.
    Diligent,
    /// Gets good at a trade half as fast, and is well liked for it.
    Easygoing,
    /// Gets twice as far as others when set to thinking, studying, or teaching.
    Clever,
    /// Is held in higher esteem.
    Kind,
    /// Is held in lower esteem, and is worth two as a clerk.
    Stern,
    /// Sickens half as easily.
    Hardy,
    /// Sickens twice as easily.
    Frail,
    /// Is worth two as a guard.
    Bold,
}

impl Trait {
    pub const ALL: [Self; 8] = [
        Self::Diligent,
        Self::Easygoing,
        Self::Clever,
        Self::Kind,
        Self::Stern,
        Self::Hardy,
        Self::Frail,
        Self::Bold,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Diligent => "Diligent",
            Self::Easygoing => "Easygoing",
            Self::Clever => "Clever",
            Self::Kind => "Kind",
            Self::Stern => "Stern",
            Self::Hardy => "Hardy",
            Self::Frail => "Frail",
            Self::Bold => "Bold",
        }
    }

    /// Returns what comes of being so.
    pub fn telling(self) -> &'static str {
        match self {
            Self::Diligent => "gets good at a trade twice as fast",
            Self::Easygoing => "gets good at a trade half as fast, and is well liked",
            Self::Clever => "gets twice as far when thinking, studying, or teaching",
            Self::Kind => "is held in higher esteem",
            Self::Stern => "is held in lower esteem, and is worth two as a clerk",
            Self::Hardy => "sickens half as easily",
            Self::Frail => "sickens twice as easily",
            Self::Bold => "is worth two as a guard",
        }
    }

    /// Returns the trait that nobody has along with this one.
    pub const fn opposite(self) -> Option<Self> {
        match self {
            Self::Diligent => Some(Self::Easygoing),
            Self::Easygoing => Some(Self::Diligent),
            Self::Kind => Some(Self::Stern),
            Self::Stern => Some(Self::Kind),
            Self::Hardy => Some(Self::Frail),
            Self::Frail => Some(Self::Hardy),
            Self::Clever | Self::Bold => None,
        }
    }
}

/// How good someone is at their trade at the most, and the skill from which they are a
/// master of it.
pub const FULL_SKILL: u8 = 100;
pub const MASTERY: u8 = 70;

/// What someone who lives with the player does with their days.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Picks berries in their season, and finds what little else there is to eat.
    #[default]
    Forager,
    Gatherer,
    Fisher,
    Trapper,
    Woodcutter,
    Farmer,
    Builder,
    Crafter,
    Thinker,
    Idle,
    Herder,
    Brewer,
    Innkeeper,
    Quarrier,
    Miner,
    Burner,
    Smith,
    Weaver,
    Merchant,
    Clerk,
    Healer,
    Sailor,
    Teacher,
    Miller,
    Sawyer,
    Brickmaker,
    Glassmaker,
    Scholar,
    Printer,
    Steelworker,
    FactoryHand,
    MillHand,
    Carrier,
    Canner,
    Engineer,
    Operator,
    Mechanic,
    Guard,
    Hunter,
    Priest,
    Performer,
}

impl Role {
    pub const ALL: [Self; 41] = [
        Self::Forager,
        Self::Gatherer,
        Self::Fisher,
        Self::Trapper,
        Self::Woodcutter,
        Self::Farmer,
        Self::Builder,
        Self::Crafter,
        Self::Thinker,
        Self::Idle,
        Self::Herder,
        Self::Brewer,
        Self::Innkeeper,
        Self::Quarrier,
        Self::Miner,
        Self::Burner,
        Self::Smith,
        Self::Weaver,
        Self::Merchant,
        Self::Clerk,
        Self::Healer,
        Self::Sailor,
        Self::Teacher,
        Self::Miller,
        Self::Sawyer,
        Self::Brickmaker,
        Self::Glassmaker,
        Self::Scholar,
        Self::Printer,
        Self::Steelworker,
        Self::FactoryHand,
        Self::MillHand,
        Self::Carrier,
        Self::Canner,
        Self::Engineer,
        Self::Operator,
        Self::Mechanic,
        Self::Guard,
        Self::Hunter,
        Self::Priest,
        Self::Performer,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Forager => "Forager",
            Self::Gatherer => "Gatherer",
            Self::Fisher => "Fisher",
            Self::Trapper => "Trapper",
            Self::Woodcutter => "Woodcutter",
            Self::Farmer => "Farmer",
            Self::Builder => "Builder",
            Self::Crafter => "Crafter",
            Self::Thinker => "Thinker",
            Self::Idle => "At home",
            Self::Herder => "Herder",
            Self::Brewer => "Brewer",
            Self::Innkeeper => "Innkeeper",
            Self::Quarrier => "Quarrier",
            Self::Miner => "Miner",
            Self::Burner => "Charcoal burner",
            Self::Smith => "Smith",
            Self::Weaver => "Weaver",
            Self::Merchant => "Merchant",
            Self::Clerk => "Clerk",
            Self::Healer => "Healer",
            Self::Sailor => "Sailor",
            Self::Teacher => "Teacher",
            Self::Miller => "Miller",
            Self::Sawyer => "Sawyer",
            Self::Brickmaker => "Brickmaker",
            Self::Glassmaker => "Glassmaker",
            Self::Scholar => "Scholar",
            Self::Printer => "Printer",
            Self::Steelworker => "Steelworker",
            Self::FactoryHand => "Factory hand",
            Self::MillHand => "Mill hand",
            Self::Carrier => "Transport worker",
            Self::Canner => "Canner",
            Self::Engineer => "Engineer",
            Self::Operator => "Operator",
            Self::Mechanic => "Mechanic",
            Self::Guard => "Guard",
            Self::Hunter => "Hunter",
            Self::Priest => "Priest",
            Self::Performer => "Performer",
        }
    }

    /// Returns what someone of the role brings about.
    pub fn work(self) -> &'static str {
        match self {
            Self::Forager => "Picks berries in season, and strips inner bark when there are none",
            Self::Gatherer => {
                "Brings in boughs, birch bark, spruce roots, and stones, and clay, sand, and bog ore once they are known to be of use"
            }
            Self::Fisher => {
                "Catches perch, though fewer through the ice. The lake feeds only so many"
            }
            Self::Trapper => "Snares hares for meat and pelts. The forest feeds only so many",
            Self::Woodcutter => "Fells trees for logs and sticks",
            Self::Farmer => "Grows rye on a field, harvested in the middle of September",
            Self::Builder => "Builds what has been planned",
            Self::Crafter => "Twists cordage and sews winter clothes out of pelts",
            Self::Thinker => "Works towards the advance being pursued",
            Self::Idle => "Keeps house, minds the children, and has no trade",
            Self::Herder => "Keeps animals on a pasture for meat, cheese, wool, and hides",
            Self::Brewer => "Brews ale of rye at a brewery",
            Self::Innkeeper => "Pours ale at a tavern, which keeps people content",
            Self::Quarrier => "Breaks stone at a quarry",
            Self::Miner => {
                "Brings up copper and tin ore from a mine, and iron ore and coal once they are known to be of use"
            }
            Self::Burner => "Chars logs at a charcoal kiln",
            Self::Smith => "Smelts ore with charcoal at a smithy, and forges tools of the metal",
            Self::Weaver => "Weaves cloth of wool at a weaving shed",
            Self::Merchant => "Sells what the store can spare and buys what it lacks at a market",
            Self::Clerk => "Keeps the peace, the tax rolls, and the accounts",
            Self::Healer => "Tends the sick at an infirmary or a hospital, and makes medicine",
            Self::Sailor => "Mans the trading ships of a harbour",
            Self::Teacher => "Teaches children to read at a school",
            Self::Miller => "Grinds the harvest at a mill, so that more comes of it",
            Self::Sawyer => {
                "Cuts logs into planks at a sawmill, and makes paper once that is known"
            }
            Self::Brickmaker => "Fires bricks at a brickworks, and burns cement once that is known",
            Self::Glassmaker => "Melts sand into panes at a glassworks",
            Self::Scholar => {
                "Works towards the advance being pursued, three times as fast as a thinker"
            }
            Self::Printer => "Prints books at a printing house",
            Self::Steelworker => "Makes steel of iron and coal at a steelworks",
            Self::FactoryHand => "Makes tools, machine parts, and wire at a factory",
            Self::MillHand => "Weaves cloth at a textile mill",
            Self::Carrier => "Runs the trains of a railway station and the flights of an airport",
            Self::Canner => "Puts up fish in cans at a cannery",
            Self::Engineer => "Keeps a power station running, on coal unless there is clean power",
            Self::Operator => "Works the telegraph and the broadcasts",
            Self::Mechanic => "Keeps the tractors of a machine depot running on oil",
            Self::Guard => "Keeps watch from a barracks, and stands against raiders",
            Self::Hunter => "Hunts deer for meat and hides. The forest feeds only so many",
            Self::Priest => "Keeps the rites at a shrine or a temple",
            Self::Performer => "Plays, sings, and competes at a theatre, a cinema, or a stadium",
        }
    }

    /// Returns what a day of the work is paid, in coins, where work is paid.
    pub const fn wage(self) -> u32 {
        match self {
            Self::Forager => 3,
            Self::Gatherer => 3,
            Self::Fisher => 3,
            Self::Trapper => 3,
            Self::Woodcutter => 3,
            Self::Farmer => 3,
            Self::Builder => 4,
            Self::Crafter => 4,
            Self::Thinker => 5,
            Self::Idle => 0,
            Self::Herder => 3,
            Self::Brewer => 4,
            Self::Innkeeper => 4,
            Self::Quarrier => 4,
            Self::Miner => 5,
            Self::Burner => 4,
            Self::Smith => 6,
            Self::Weaver => 4,
            Self::Merchant => 6,
            Self::Clerk => 6,
            Self::Healer => 6,
            Self::Sailor => 5,
            Self::Teacher => 5,
            Self::Miller => 4,
            Self::Sawyer => 4,
            Self::Brickmaker => 4,
            Self::Glassmaker => 6,
            Self::Scholar => 9,
            Self::Printer => 6,
            Self::Steelworker => 6,
            Self::FactoryHand => 5,
            Self::MillHand => 5,
            Self::Carrier => 6,
            Self::Canner => 5,
            Self::Engineer => 8,
            Self::Operator => 6,
            Self::Mechanic => 6,
            Self::Guard => 3,
            Self::Hunter => 3,
            Self::Priest => 4,
            Self::Performer => 5,
        }
    }

    /// Returns whether those of the role are paid out of the treasury, once there is one.
    pub const fn is_public(self) -> bool {
        matches!(
            self,
            Self::Builder
                | Self::Thinker
                | Self::Clerk
                | Self::Guard
                | Self::Healer
                | Self::Teacher
                | Self::Scholar
        )
    }

    /// Returns whether the role is a craft, which a guild makes better at what it does.
    pub const fn is_craft(self) -> bool {
        matches!(
            self,
            Self::Crafter
                | Self::Brewer
                | Self::Burner
                | Self::Smith
                | Self::Weaver
                | Self::Miller
                | Self::Sawyer
                | Self::Brickmaker
                | Self::Glassmaker
                | Self::Printer
        )
    }

    /// Returns whether the role takes a building to work at.
    pub fn takes_a_workplace(self) -> bool {
        crate::building::places(self, &[]).is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    pub name: Name,
    pub sex: Sex,
    pub born: Timestamp,
    pub location: Location,
    #[serde(default)]
    pub died: Option<Timestamp>,
    /// Whether the person has left the forest for good.
    #[serde(default)]
    pub gone: bool,
    /// How well the person thinks of the player character, out of a hundred.
    #[serde(default)]
    pub regard: u8,
    /// Whether the person lives with the player character.
    #[serde(default)]
    pub member: bool,
    #[serde(default)]
    pub role: Role,
    #[serde(default)]
    pub partner: Option<PersonId>,
    #[serde(default)]
    pub parents: [Option<PersonId>; 2],
    /// When the child the person carries is due.
    #[serde(default)]
    pub expecting: Option<Timestamp>,
    #[serde(default)]
    pub last_birth: Option<Timestamp>,
    /// The day, counted from the epoch, on which the player last talked with the person.
    #[serde(default)]
    pub talked_on: Option<u64>,
    /// How many days in a row the person has gone without enough to eat.
    #[serde(default)]
    pub hungry_days: u16,
    /// When a wanderer moves on.
    #[serde(default)]
    pub leaves: Option<Timestamp>,
    /// Whether the person has winter clothes.
    #[serde(default)]
    pub clothed: bool,
    /// What the person holds on how to live together.
    #[serde(default)]
    pub views: Views,
    /// How well the others of a settlement think of the person, out of a hundred.
    #[serde(default)]
    pub esteem: u8,
    /// Until when the player character has hired the person to think for them.
    #[serde(default)]
    pub hired_until: Option<Timestamp>,
    /// The way the person is: up to two traits.
    #[serde(default)]
    pub traits: Vec<Trait>,
    /// How good the person is at their trade, out of [`FULL_SKILL`].
    #[serde(default)]
    pub skill: u8,
    /// Whether the person is a child learning the trade they are set to before they
    /// are old enough to work at it.
    #[serde(default)]
    pub apprenticed: bool,
}

impl Person {
    /// The room a person takes up around where they stand.
    pub const GIRTH: crate::space::Length = crate::space::Length::from_millimeters(300);

    pub fn new(name: Name, sex: Sex, born: Timestamp, location: Location) -> Self {
        Self {
            name,
            sex,
            born,
            location,
            died: None,
            gone: false,
            regard: 0,
            member: false,
            role: Role::default(),
            partner: None,
            parents: [None, None],
            expecting: None,
            last_birth: None,
            talked_on: None,
            hungry_days: 0,
            leaves: None,
            clothed: false,
            views: [0; 4],
            esteem: 0,
            hired_until: None,
            traits: Vec::new(),
            skill: 0,
            apprenticed: false,
        }
    }

    /// Returns a wanderer of a random name, sex, and grown-up age at `location`.
    pub(crate) fn wanderer(rng: &mut Rng, now: Timestamp, location: Location) -> Self {
        let sex = if rng.chance(50) {
            Sex::Female
        } else {
            Sex::Male
        };
        let family = FAMILY_NAMES[rng.below(FAMILY_NAMES.len() as u64) as usize];
        let name = Name::new(
            given_name(rng, sex).to_owned(),
            Vec::new(),
            family.to_owned(),
        );
        let age_in_days = rng.between((17 * 365, 38 * 365)) as u64;
        let born = now
            .as_seconds()
            .saturating_sub(age_in_days * crate::time::gregorian::SECONDS_PER_DAY);

        Self {
            views: random_views(rng),
            traits: random_traits(rng, &[]),
            // Someone grown has learned to do something.
            skill: rng.between((0, 40)) as u8,
            ..Self::new(name, sex, Timestamp::from_seconds(born), location)
        }
    }

    pub fn has(&self, wanted: Trait) -> bool {
        self.traits.contains(&wanted)
    }

    /// Sets the person to `role`. Half of what they were good at carries over to a new
    /// trade.
    pub(crate) fn take_up(&mut self, role: Role) {
        if self.role != role {
            self.role = role;
            self.skill /= 2;
        }
    }

    /// Has the person get a month better at their trade.
    pub(crate) fn practise(&mut self) {
        let gain = if self.has(Trait::Diligent) {
            4
        } else if self.has(Trait::Easygoing) {
            1
        } else {
            2
        };
        if self.role != Role::Idle {
            self.skill = self.skill.saturating_add(gain).min(FULL_SKILL);
        }
    }

    /// Returns whether the person is alive and in the forest.
    pub fn is_present(&self) -> bool {
        self.died.is_none() && !self.gone
    }

    /// Returns how many full years old the person is at `now`.
    pub fn age(&self, now: Timestamp) -> u64 {
        Date::from_timestamp(now).years_since(Date::from_timestamp(self.born))
    }

    pub fn is_adult(&self, now: Timestamp) -> bool {
        self.age(now) >= ADULT_AGE
    }

    pub fn works(&self, now: Timestamp) -> bool {
        self.age(now) >= WORKING_AGE
    }

    /// Returns whether the person could bear a child at `now`.
    pub fn is_fertile(&self, now: Timestamp) -> bool {
        self.sex == Sex::Female
            && (ADULT_AGE..=FERTILE_UNTIL).contains(&self.age(now))
            && self.expecting.is_none()
    }

    /// Returns whether `other` is a parent, a child, or a sibling of the person, who is
    /// `own` and `other` is `theirs`.
    pub fn is_close_kin(&self, own: PersonId, other: &Self, theirs: PersonId) -> bool {
        let shares_a_parent = self
            .parents
            .iter()
            .flatten()
            .any(|parent| other.parents.contains(&Some(*parent)));

        shares_a_parent
            || self.parents.contains(&Some(theirs))
            || other.parents.contains(&Some(own))
    }

    pub(crate) fn warm_to(&mut self, by: u8) {
        self.regard = self.regard.saturating_add(by).min(100);
    }

    pub(crate) fn cool_to(&mut self, by: u8) {
        self.regard = self.regard.saturating_sub(by);
    }
}

/// Returns the traits of someone new: often one, now and then two, taking after those
/// of their parents, which are `inherited`, as often as not.
pub(crate) fn random_traits(rng: &mut Rng, inherited: &[Trait]) -> Vec<Trait> {
    let mut traits: Vec<Trait> = Vec::new();
    let many = match rng.below(10) {
        0..3 => 0,
        3..8 => 1,
        _ => 2,
    };
    for _ in 0..many {
        let from_parent = !inherited.is_empty() && rng.chance(50);
        let picked = if from_parent {
            inherited[rng.below(inherited.len() as u64) as usize]
        } else {
            Trait::ALL[rng.below(Trait::ALL.len() as u64) as usize]
        };
        let clashes = traits
            .iter()
            .any(|had| *had == picked || had.opposite() == Some(picked));
        if !clashes {
            traits.push(picked);
        }
    }
    traits
}

/// Returns a random family name.
pub(crate) fn family_name(rng: &mut Rng) -> &'static str {
    FAMILY_NAMES[rng.below(FAMILY_NAMES.len() as u64) as usize]
}

/// Returns a random given name for someone of `sex`.
pub(crate) fn given_name(rng: &mut Rng, sex: Sex) -> &'static str {
    let names = match sex {
        Sex::Female => &FEMALE_NAMES,
        Sex::Male => &MALE_NAMES,
    };
    names[rng.below(names.len() as u64) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::gregorian::SECONDS_PER_DAY;

    const NOW: Timestamp = Timestamp::from_seconds(80 * 365 * SECONDS_PER_DAY);

    fn person(sex: Sex, age: u64) -> Person {
        let born = NOW.as_seconds() - (age * 365 + age / 4 + 2) * SECONDS_PER_DAY;
        Person::new(
            Name::default(),
            sex,
            Timestamp::from_seconds(born),
            Location::default(),
        )
    }

    #[test]
    fn wanderers_are_grown_up_and_named() {
        let mut rng = Rng::new(9);

        for _ in 0..200 {
            let wanderer = Person::wanderer(&mut rng, NOW, Location::default());

            assert!(
                (16..=38).contains(&wanderer.age(NOW)),
                "{}",
                wanderer.age(NOW)
            );
            assert!(!wanderer.name.first().is_empty() && !wanderer.name.last().is_empty());
            assert!(wanderer.is_present() && !wanderer.member);
        }
    }

    #[test]
    fn age_decides_who_works_and_who_bears_children() {
        assert!(!person(Sex::Female, 11).works(NOW));
        assert!(person(Sex::Female, 12).works(NOW));
        assert!(!person(Sex::Female, 15).is_adult(NOW));
        assert!(!person(Sex::Female, 15).is_fertile(NOW));
        assert!(person(Sex::Female, 16).is_fertile(NOW));
        assert!(!person(Sex::Female, 46).is_fertile(NOW));
        assert!(!person(Sex::Male, 30).is_fertile(NOW));

        let mut expecting = person(Sex::Female, 30);
        expecting.expecting = Some(NOW);
        assert!(!expecting.is_fertile(NOW));
    }

    #[test]
    fn parents_children_and_siblings_are_close_kin() {
        let (mother, father, stranger) = (PersonId(0), PersonId(1), PersonId(2));
        let (daughter, son) = (PersonId(3), PersonId(4));
        let parent = person(Sex::Female, 40);
        let other = person(Sex::Male, 40);
        let mut child = person(Sex::Female, 18);
        child.parents = [Some(mother), Some(father)];
        let mut sibling = person(Sex::Male, 17);
        sibling.parents = [Some(mother), None];

        assert!(child.is_close_kin(daughter, &parent, mother));
        assert!(parent.is_close_kin(mother, &child, daughter));
        assert!(child.is_close_kin(daughter, &sibling, son));
        assert!(!child.is_close_kin(daughter, &other, stranger));
        assert!(!parent.is_close_kin(mother, &other, father));
    }

    #[test]
    fn regard_stays_between_nothing_and_a_hundred() {
        let mut someone = person(Sex::Male, 20);

        someone.cool_to(5);
        assert_eq!(someone.regard, 0);
        someone.warm_to(250);
        someone.warm_to(250);
        assert_eq!(someone.regard, 100);
    }

    #[test]
    fn traits_never_clash_and_a_trade_is_learned_with_time() {
        let mut rng = Rng::new(5);
        for _ in 0..300 {
            let traits = random_traits(&mut rng, &[Trait::Kind, Trait::Bold]);
            assert!(traits.len() <= 2);
            for one in &traits {
                assert!(one.opposite().is_none_or(|other| !traits.contains(&other)));
                assert!(!one.name().is_empty() && !one.telling().is_empty());
            }
        }

        let mut someone = person(Sex::Female, 20);
        someone.role = Role::Smith;
        for _ in 0..12 {
            someone.practise();
        }
        assert_eq!(someone.skill, 24);
        someone.traits = vec![Trait::Diligent];
        for _ in 0..40 {
            someone.practise();
        }
        assert_eq!(someone.skill, FULL_SKILL);
        someone.take_up(Role::Smith);
        assert_eq!(someone.skill, FULL_SKILL);
        someone.take_up(Role::Miner);
        assert_eq!(someone.skill, 50);
    }

    #[test]
    fn every_role_has_a_name_and_work() {
        for (index, role) in Role::ALL.into_iter().enumerate() {
            assert!(!role.name().is_empty() && !role.work().is_empty());
            assert!(!Role::ALL[..index].contains(&role));
            assert!(role == Role::Idle || role.wage() > 0);
        }
        assert!(Role::Farmer.takes_a_workplace() && !Role::Fisher.takes_a_workplace());
        assert!(Role::Scholar.is_public() && !Role::Smith.is_public());
        assert_eq!(Role::default(), Role::Forager);
    }
}
