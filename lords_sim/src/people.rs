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

/// What someone who lives with the player does with their days.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Picks berries in their season, and finds what little else there is to eat.
    #[default]
    Forager,
    /// Brings in boughs, bark, roots, and stones.
    Gatherer,
    Fisher,
    Trapper,
    Woodcutter,
    Farmer,
    Builder,
    Crafter,
    /// Works out better ways of doing things.
    Thinker,
    Idle,
}

impl Role {
    pub const ALL: [Self; 10] = [
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
            Self::Idle => "At leisure",
        }
    }

    /// Returns what someone of the role brings about.
    pub fn work(self) -> &'static str {
        match self {
            Self::Forager => "Picks berries in season, and strips inner bark when there are none",
            Self::Gatherer => "Brings in boughs, birch bark, spruce roots, and stones",
            Self::Fisher => "Catches perch, though fewer through the ice",
            Self::Trapper => "Snares hares for meat and pelts",
            Self::Woodcutter => "Fells trees for logs and sticks",
            Self::Farmer => "Grows rye on a field, harvested in the middle of September",
            Self::Builder => "Builds what has been planned",
            Self::Crafter => "Twists cordage and sews winter clothes out of pelts",
            Self::Thinker => "Works towards the advance being pursued",
            Self::Idle => "Does nothing in particular",
        }
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
            ..Self::new(name, sex, Timestamp::from_seconds(born), location)
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
    fn every_role_has_a_name_and_work() {
        for role in Role::ALL {
            assert!(!role.name().is_empty() && !role.work().is_empty());
        }
        assert_eq!(Role::default(), Role::Forager);
    }
}
