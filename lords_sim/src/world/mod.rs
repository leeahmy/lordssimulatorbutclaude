mod area;
mod generate;
pub(crate) mod index;
mod store;
mod thing;

use serde::{Deserialize, Serialize};

pub use self::area::{Area, AreaKind};
pub use self::store::Store;
pub use self::thing::{Plant, PlantSpecies, Rock, Thing};
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
}

/// What a subject is, as far as telling it apart from others goes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Plant(PlantSpecies),
    Rock,
    Lake,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    things: Store<Thing>,
    areas: Vec<Area>,
}

impl World {
    pub fn add_thing(&mut self, thing: Thing) -> ThingId {
        ThingId(self.things.push(thing))
    }

    pub fn add_area(&mut self, area: Area) -> AreaId {
        self.areas.push(area);
        AreaId(self.areas.len() as u32 - 1)
    }

    pub fn thing(&self, id: ThingId) -> Option<&Thing> {
        self.things.get(id.0)
    }

    pub fn things(&self) -> impl Iterator<Item = (ThingId, &Thing)> {
        (0..).map(ThingId).zip(self.things.iter())
    }

    pub fn area(&self, id: AreaId) -> Option<&Area> {
        self.areas.get(id.0 as usize)
    }

    pub fn areas(&self) -> impl Iterator<Item = (AreaId, &Area)> {
        (0..).map(AreaId).zip(&self.areas)
    }

    pub fn kind(&self, subject: Subject) -> Option<Kind> {
        match subject {
            Subject::Thing(id) => self.thing(id).map(Thing::kind),
            Subject::Area(id) => self.area(id).map(|area| match area.kind() {
                AreaKind::Lake => Kind::Lake,
            }),
        }
    }

    pub fn is_water(&self, at: Location) -> bool {
        self.areas
            .iter()
            .any(|area| area.kind() == AreaKind::Lake && area.contains(at))
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
        };
        let margin = Length::from_millimeters(reach.as_millimeters() / 2);

        Some(nearest.toward(from, clearance + margin))
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
