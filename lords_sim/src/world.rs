mod area;
mod generate;
pub(crate) mod index;
mod store;
mod thing;

use serde::{Deserialize, Serialize};

pub use self::area::{Area, AreaKind};
pub use self::store::{Bits, Store};
pub use self::thing::{Pile, Plant, PlantSpecies, Rock, Structure, StructureKind, Thing};
use crate::item::Item;
use crate::people::{Bond, FRIENDLY, Person, PersonId};
use crate::space::{Length, Location};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ThingId(u32);

impl ThingId {
    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AreaId(u32);

impl AreaId {
    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

/// Anything in the world that can be pointed out.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Subject {
    Thing(ThingId),
    Area(AreaId),
    Person(PersonId),
}

/// What a subject is, as far as telling it apart from others goes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Plant(PlantSpecies),
    Rock,
    Lake,
    /// A pile of mostly the item.
    Pile(Item),
    Structure(StructureKind),
    Person(Bond),
}

impl Kind {
    /// Returns what one of the kind is called.
    pub fn name(self) -> &'static str {
        match self {
            Self::Plant(PlantSpecies::Spruce) => "Spruce",
            Self::Plant(PlantSpecies::Pine) => "Pine",
            Self::Plant(PlantSpecies::Birch) => "Birch",
            Self::Plant(PlantSpecies::Bilberry) => "Bilberry patch",
            Self::Plant(PlantSpecies::Lingonberry) => "Lingonberry patch",
            Self::Rock => "Boulder",
            Self::Lake => "Lake",
            Self::Pile(item) => item.name(),
            Self::Structure(StructureKind::Campfire) => "Campfire",
            Self::Structure(StructureKind::Fireplace) => "Cold fireplace",
            Self::Structure(StructureKind::LeanTo) => "Lean-to",
            Self::Structure(StructureKind::Bed) => "Bough bed",
            Self::Structure(StructureKind::Cabin) => "Log cabin",
            Self::Structure(StructureKind::Snare) => "Snare",
            Self::Structure(StructureKind::SprungSnare) => "Sprung snare",
            Self::Structure(StructureKind::FishTrap) => "Fish trap",
            Self::Structure(StructureKind::DryingRack) => "Drying rack",
            Self::Structure(StructureKind::Cairn) => "Settlement cairn",
            Self::Structure(StructureKind::Site) => "Building site",
            Self::Structure(StructureKind::Hut) => "Hut",
            Self::Structure(StructureKind::Longhouse) => "Longhouse",
            Self::Structure(StructureKind::Storehouse) => "Storehouse",
            Self::Structure(StructureKind::Smokehouse) => "Smokehouse",
            Self::Structure(StructureKind::Workshop) => "Workshop",
            Self::Structure(StructureKind::Field) => "Field",
            Self::Structure(StructureKind::Well) => "Well",
            Self::Structure(StructureKind::Hall) => "Meeting hall",
            Self::Structure(StructureKind::Kiln) => "Kiln",
            Self::Structure(StructureKind::Dock) => "Dock",
            Self::Structure(StructureKind::Smithy) => "Smithy",
            Self::Person(bond) => bond.name(),
        }
    }

    pub const fn is_tree(self) -> bool {
        matches!(self, Self::Plant(species) if species.is_tree())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    things: Store<Thing>,
    areas: Vec<Area>,
    /// The things that are no longer there.
    #[serde(default)]
    gone: Bits,
    /// Everyone there is and has been, apart from what is kept of the player character
    /// as a [`Human`](crate::Human).
    #[serde(default)]
    people: Store<Person>,
    /// Which of the people the player character is.
    #[serde(default)]
    protagonist: Option<PersonId>,
}

impl World {
    pub fn add_thing(&mut self, thing: Thing) -> ThingId {
        ThingId(self.things.push(thing))
    }

    pub fn add_area(&mut self, area: Area) -> AreaId {
        self.areas.push(area);
        AreaId(self.areas.len() as u32 - 1)
    }

    /// Returns the thing, unless there is no such thing or it is no longer there.
    pub fn thing(&self, id: ThingId) -> Option<&Thing> {
        if self.gone.has(id.0) {
            return None;
        }
        self.things.get(id.0)
    }

    pub fn things(&self) -> impl Iterator<Item = (ThingId, &Thing)> {
        (0..)
            .map(ThingId)
            .zip(self.things.iter())
            .filter(|(id, _)| !self.gone.has(id.0))
    }

    pub fn plant(&self, id: ThingId) -> Option<&Plant> {
        match self.thing(id)? {
            Thing::Plant(plant) => Some(plant),
            _ => None,
        }
    }

    pub fn pile(&self, id: ThingId) -> Option<&Pile> {
        match self.thing(id)? {
            Thing::Pile(pile) => Some(pile),
            _ => None,
        }
    }

    pub fn structure(&self, id: ThingId) -> Option<&Structure> {
        match self.thing(id)? {
            Thing::Structure(structure) => Some(structure),
            _ => None,
        }
    }

    pub(crate) fn plant_mut(&mut self, id: ThingId) -> Option<&mut Plant> {
        match self.thing_mut(id)? {
            Thing::Plant(plant) => Some(plant),
            _ => None,
        }
    }

    pub(crate) fn pile_mut(&mut self, id: ThingId) -> Option<&mut Pile> {
        match self.thing_mut(id)? {
            Thing::Pile(pile) => Some(pile),
            _ => None,
        }
    }

    pub(crate) fn structure_mut(&mut self, id: ThingId) -> Option<&mut Structure> {
        match self.thing_mut(id)? {
            Thing::Structure(structure) => Some(structure),
            _ => None,
        }
    }

    fn thing_mut(&mut self, id: ThingId) -> Option<&mut Thing> {
        if self.gone.has(id.0) {
            return None;
        }
        self.things.get_mut(id.0)
    }

    /// Takes the thing out of the world. Its id is never given to another.
    pub(crate) fn remove_thing(&mut self, id: ThingId) {
        if self.things.get(id.0).is_some() {
            self.gone.mark(id.0);
        }
    }

    pub fn area(&self, id: AreaId) -> Option<&Area> {
        self.areas.get(id.0 as usize)
    }

    pub fn areas(&self) -> impl Iterator<Item = (AreaId, &Area)> {
        (0..).map(AreaId).zip(&self.areas)
    }

    pub fn add_person(&mut self, person: Person) -> PersonId {
        PersonId(self.people.push(person))
    }

    /// Returns the person, dead or alive.
    pub fn person(&self, id: PersonId) -> Option<&Person> {
        self.people.get(id.0)
    }

    pub(crate) fn person_mut(&mut self, id: PersonId) -> Option<&mut Person> {
        self.people.get_mut(id.0)
    }

    /// Returns everyone there is and has been.
    pub fn everyone(&self) -> impl Iterator<Item = (PersonId, &Person)> {
        (0..).map(PersonId).zip(self.people.iter())
    }

    /// Returns everyone alive and in the forest apart from the player character.
    pub fn people(&self) -> impl Iterator<Item = (PersonId, &Person)> {
        self.everyone()
            .filter(|(id, person)| person.is_present() && Some(*id) != self.protagonist)
    }

    /// Returns which of the people the player character is.
    pub fn protagonist(&self) -> Option<PersonId> {
        self.protagonist
    }

    pub(crate) fn set_protagonist(&mut self, id: PersonId) {
        self.protagonist = Some(id);
    }

    /// Returns what the person is to the player character.
    pub fn bond(&self, id: PersonId) -> Option<Bond> {
        let person = self.person(id)?;
        let me = self.protagonist;

        Some(if me.is_some() && person.partner == me {
            Bond::Partner
        } else if me.is_some() && person.parents.contains(&me) {
            Bond::Child
        } else if person.member {
            Bond::Companion
        } else if person.regard >= FRIENDLY {
            Bond::Friend
        } else {
            Bond::Stranger
        })
    }

    pub fn kind(&self, subject: Subject) -> Option<Kind> {
        match subject {
            Subject::Person(id) => {
                let present = self.person(id).is_some_and(Person::is_present);
                self.bond(id).filter(|_| present).map(Kind::Person)
            }
            Subject::Thing(id) => self.thing(id).map(Thing::kind),
            Subject::Area(id) => self.area(id).map(|area| match area.kind() {
                AreaKind::Lake => Kind::Lake,
            }),
        }
    }

    pub fn is_water(&self, at: Location) -> bool {
        self.lake_at(at).is_some()
    }

    pub fn lake_at(&self, at: Location) -> Option<(AreaId, &Area)> {
        self.areas()
            .find(|(_, area)| area.kind() == AreaKind::Lake && area.contains(at))
    }

    /// Returns where to stand, coming from `from`, to have `subject` within `reach`.
    ///
    /// That is `from` itself if the subject is within reach from there already.
    pub fn approach(&self, subject: Subject, from: Location, reach: Length) -> Option<Location> {
        let (nearest, clearance) = match subject {
            Subject::Thing(id) => {
                let thing = self.thing(id)?;
                (thing.location(), thing.footprint())
            }
            Subject::Area(id) => (self.area(id)?.nearest_shore(from), Length::ZERO),
            Subject::Person(id) => {
                let person = self.person(id).filter(|person| person.is_present())?;
                (person.location, Person::GIRTH)
            }
        };
        if from.distance_to(nearest) <= clearance + reach {
            return Some(from);
        }

        Some(nearest.toward(from, clearance + reach / 2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Object;

    const REACH: Length = Length::from_millimeters(800);

    fn birch(x: i64, y: i64) -> Thing {
        Thing::Plant(Plant::new(
            Object::new(Location::new(x, y), Default::default()),
            PlantSpecies::Birch,
            Length::from_meters(15),
            Length::from_millimeters(300),
        ))
    }

    fn lake() -> Area {
        let corners = [(0, 30_000), (20_000, 30_000), (20_000, 50_000), (0, 50_000)];
        Area::new(
            AreaKind::Lake,
            corners
                .into_iter()
                .map(|(x, y)| Location::new(x, y))
                .collect(),
        )
    }

    #[test]
    fn subjects_are_found_by_the_id_they_were_given() {
        let mut world = World::default();
        let tree = world.add_thing(birch(0, 5_000));
        let water = world.add_area(lake());

        assert_eq!(
            world.kind(Subject::Thing(tree)),
            Some(Kind::Plant(PlantSpecies::Birch))
        );
        assert_eq!(world.kind(Subject::Area(water)), Some(Kind::Lake));
        assert_eq!(world.things().map(|(id, _)| id).collect::<Vec<_>>(), [tree]);
        assert_eq!(world.areas().map(|(id, _)| id).collect::<Vec<_>>(), [water]);
        assert_eq!(World::default().kind(Subject::Thing(tree)), None);
    }

    #[test]
    fn thing_taken_out_of_the_world_is_no_longer_found() {
        let mut world = World::default();
        let felled = world.add_thing(birch(0, 5_000));
        let standing = world.add_thing(birch(0, 9_000));

        world.remove_thing(felled);

        assert!(world.thing(felled).is_none());
        assert!(world.plant_mut(felled).is_none());
        assert_eq!(world.kind(Subject::Thing(felled)), None);
        assert_eq!(
            world.approach(Subject::Thing(felled), Location::default(), REACH),
            None
        );
        assert_eq!(
            world.things().map(|(id, _)| id).collect::<Vec<_>>(),
            [standing]
        );
        assert_ne!(world.add_thing(birch(0, 5_000)), felled);
    }

    #[test]
    fn world_written_before_things_could_go_still_reads() {
        let old = "(things:[],areas:[])";

        let read: World = ron::from_str(old).expect("the old world decodes");

        assert_eq!(read, World::default());
    }

    #[test]
    fn only_lakes_are_water() {
        let mut world = World::default();
        world.add_area(lake());

        assert!(world.is_water(Location::new(10_000, 40_000)));
        assert!(!world.is_water(Location::new(10_000, 20_000)));
    }

    #[test]
    fn approach_stops_short_of_a_thing_by_less_than_the_reach() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(0, 5_000)));
        let origin = Location::default();

        let spot = world
            .approach(tree, origin, REACH)
            .expect("the tree exists");

        assert_eq!(spot, Location::new(0, 4_450));
        assert_eq!(world.approach(tree, spot, REACH), Some(spot));
    }

    #[test]
    fn approach_stays_put_anywhere_within_reach() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(0, 5_000)));
        let water = Subject::Area(world.add_area(lake()));
        let at_arms_length = Location::new(0, 4_050);
        let just_too_far = Location::new(0, 4_049);
        let by_the_shore = Location::new(10_000, 29_200);

        assert_eq!(
            world.approach(tree, at_arms_length, REACH),
            Some(at_arms_length)
        );
        assert_eq!(
            world.approach(tree, just_too_far, REACH),
            Some(Location::new(0, 4_450))
        );
        assert_eq!(
            world.approach(water, by_the_shore, REACH),
            Some(by_the_shore)
        );
    }

    #[test]
    fn approach_to_a_lake_ends_on_dry_ground_by_the_shore() {
        let mut world = World::default();
        let water = Subject::Area(world.add_area(lake()));

        let spot = world
            .approach(water, Location::new(10_000, 0), REACH)
            .expect("the lake exists");

        assert_eq!(spot, Location::new(10_000, 29_600));
        assert!(!world.is_water(spot));
    }
}
