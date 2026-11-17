use std::fmt::{self, Display, Formatter};
use std::iter;

use serde::{Deserialize, Serialize};

use crate::space::{Length, Location};
use crate::time::Timestamp;
use crate::world::Subject;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Object {
    location: Location,
    creation: Timestamp,
}

impl Object {
    pub fn new(location: Location, creation: Timestamp) -> Self {
        Self { location, creation }
    }

    pub fn location(&self) -> Location {
        self.location
    }

    /// Returns the moment this object came into existence.
    pub fn creation(&self) -> Timestamp {
        self.creation
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Animal {
    object: Object,
    name: Name,
    sex: Sex,
    #[serde(default)]
    activity: Option<Activity>,
}

impl Animal {
    pub fn new(location: Location, time: Timestamp, name: Name, sex: Sex) -> Self {
        Self {
            object: Object::new(location, time),
            name,
            sex,
            activity: None,
        }
    }

    pub(crate) fn move_to(&mut self, location: Location) {
        self.object.location = location;
    }

    pub(crate) fn set_activity(&mut self, activity: Option<Activity>) {
        self.activity = activity;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    /// Walking to `destination`, which is where `toward` is within reach.
    Walking {
        toward: Subject,
        destination: Location,
    },
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name {
    first: String,
    middle: Vec<String>,
    last: String,
}

impl Name {
    pub fn new(first: String, middle: Vec<String>, last: String) -> Self {
        Self {
            first,
            middle,
            last,
        }
    }

    pub fn first(&self) -> &str {
        &self.first
    }

    pub fn middle(&self) -> &[String] {
        &self.middle
    }

    pub fn last(&self) -> &str {
        &self.last
    }

    pub fn full(&self) -> String {
        self.to_string()
    }

    pub fn parts(&self) -> impl Iterator<Item = &str> {
        iter::once(self.first.as_str())
            .chain(self.middle.iter().map(String::as_str))
            .chain(iter::once(self.last.as_str()))
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut parts = self.parts().filter(|part| !part.is_empty());

        if let Some(first) = parts.next() {
            f.write_str(first)?;
        }
        for part in parts {
            write!(f, " {part}")?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}

impl Display for Sex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Male => "Male",
            Self::Female => "Female",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Human {
    animal: Animal,
}

impl Human {
    /// How far from the body a hand gets.
    pub const REACH: Length = Length::from_millimeters(800);

    /// How far off the ground the eyes are.
    pub const EYE_HEIGHT: Length = Length::from_millimeters(1_600);

    /// The ground covered in an in-game second of walking.
    pub const WALKING_SPEED: Length = Length::from_millimeters(1_400);

    pub fn new(location: Location, time: Timestamp, name: Name, sex: Sex) -> Self {
        Self {
            animal: Animal::new(location, time, name, sex),
        }
    }

    pub(crate) fn animal_mut(&mut self) -> &mut Animal {
        &mut self.animal
    }
}

pub trait Living {
    fn animal(&self) -> &Animal;

    fn location(&self) -> Location {
        self.animal().object.location()
    }

    fn activity(&self) -> Option<Activity> {
        self.animal().activity
    }

    fn name(&self) -> &Name {
        &self.animal().name
    }

    fn sex(&self) -> Sex {
        self.animal().sex
    }

    fn birth(&self) -> Timestamp {
        self.animal().object.creation()
    }
}

impl Living for Human {
    fn animal(&self) -> &Animal {
        &self.animal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_written_with_one_space_between_the_parts_it_has() {
        let name = |first: &str, middle: &[&str], last: &str| {
            let middle = middle.iter().map(|part| (*part).to_owned()).collect();
            Name::new(first.to_owned(), middle, last.to_owned()).to_string()
        };

        assert_eq!(
            name("Ada", &["Maria", "Mäki"], "Lord"),
            "Ada Maria Mäki Lord"
        );
        assert_eq!(name("Ada", &[], "Lord"), "Ada Lord");
        assert_eq!(name("", &[], "Lord"), "Lord");
        assert_eq!(name("Ada", &[""], ""), "Ada");
        assert_eq!(Name::default().to_string(), "");
    }
}
