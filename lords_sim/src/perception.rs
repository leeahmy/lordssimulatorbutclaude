//! What a character notices of the world around it.

use std::collections::BTreeMap;

use crate::object::Human;
use crate::space::{Bearing, Length, Location};
use crate::world::index::Grid;
use crate::world::{Kind, Subject, ThingId, World};

/// How far away things still count as nearby, and are told apart one by one.
pub const NEARBY: Length = Length::from_meters(10);

/// The farthest anything is seen from.
const SIGHT_LIMIT: Length = Length::from_meters(300);

/// How many times its own width away a thing can still be made out.
const ACUITY: i64 = 300;

/// What a character perceives around it, each list is nearest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Surroundings {
    /// Close enough to touch and interact.
    pub within_reach: Vec<Sighting>,
    /// No farther than [`NEARBY`].
    pub nearby: Vec<Sighting>,
    /// Everything farther away, with one sighting for all of a kind.
    pub in_sight: Vec<Sighting>,
}

impl Surroundings {
    pub fn is_empty(&self) -> bool {
        self.sightings().next().is_none()
    }

    pub fn sightings(&self) -> impl Iterator<Item = &Sighting> {
        self.within_reach
            .iter()
            .chain(&self.nearby)
            .chain(&self.in_sight)
    }

    pub fn sighting(&self, subject: Subject) -> Option<&Sighting> {
        self.sightings()
            .find(|sighting| sighting.subject == subject)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sighting {
    /// The nearest one, if the sighting is of several.
    pub subject: Subject,
    pub kind: Kind,
    pub count: u32,
    /// To the surface of the subject.
    pub distance: Length,
    pub bearing: Bearing,
}

/// Returns what a perceiver whose eyes are above `eye` sees.
pub(crate) fn look(world: &World, grid: &Grid, eye: Location) -> Surroundings {
    let things = grid.around(eye, SIGHT_LIMIT).filter_map(|id| {
        let thing = world.thing(id)?;
        let middle = thing.location();
        let distance = eye.distance_to(middle);
        let width = thing.footprint().as_millimeters() * 2;
        let made_out = distance <= SIGHT_LIMIT && distance.as_millimeters() <= width * ACUITY;

        (made_out && is_clear(world, grid, eye, middle, Some(id))).then(|| Sighting {
            subject: Subject::Thing(id),
            kind: thing.kind(),
            count: 1,
            distance: (distance - thing.footprint()).max(Length::ZERO),
            bearing: eye.bearing_to(middle),
        })
    });

    let areas = world.areas().filter_map(|(id, area)| {
        let shore = std::iter::once(area.nearest_shore(eye))
            .chain(area.outline().iter().copied())
            .filter(|point| eye.distance_to(*point) <= SIGHT_LIMIT)
            .filter(|point| is_clear(world, grid, eye, *point, None))
            .min_by_key(|point| eye.distance_to(*point))?;
        let subject = Subject::Area(id);

        Some(Sighting {
            subject,
            kind: world.kind(subject)?,
            count: 1,
            distance: eye.distance_to(shore),
            bearing: eye.bearing_to(shore),
        })
    });

    arrange(things.chain(areas).collect())
}

fn arrange(mut sightings: Vec<Sighting>) -> Surroundings {
    sightings.sort_by_key(|sighting| (sighting.distance, sighting.subject));

    let mut surroundings = Surroundings::default();
    let mut groups: BTreeMap<Kind, usize> = BTreeMap::new();

    for sighting in sightings {
        if sighting.distance <= Human::REACH {
            surroundings.within_reach.push(sighting);
        } else if sighting.distance <= NEARBY {
            surroundings.nearby.push(sighting);
        } else if let Some(group) = groups.get(&sighting.kind) {
            surroundings.in_sight[*group].count += 1;
        } else {
            groups.insert(sighting.kind, surroundings.in_sight.len());
            surroundings.in_sight.push(sighting);
        }
    }

    surroundings
}

/// Returns whether nothing that stands above eye height is in the way from `eye` to
/// `target`, leaving out the thing looked at.
fn is_clear(
    world: &World,
    grid: &Grid,
    eye: Location,
    target: Location,
    looked_at: Option<ThingId>,
) -> bool {
    !grid.along(eye, target).any(|id| {
        looked_at != Some(id)
            && world.thing(id).is_some_and(|thing| {
                thing.height() >= Human::EYE_HEIGHT
                    && crosses(eye, target, thing.location(), thing.footprint())
            })
    })
}

/// Returns whether the straight line between `from` and `to` comes within `radius` of
/// `middle`.
fn crosses(from: Location, to: Location, middle: Location, radius: Length) -> bool {
    let offset = |start: Location, end: Location| {
        (
            i128::from(end.x()) - i128::from(start.x()),
            i128::from(end.y()) - i128::from(start.y()),
        )
    };
    let (line_x, line_y) = offset(from, to);
    let (middle_x, middle_y) = offset(from, middle);
    let radius = i128::from(radius.as_millimeters());

    let along = middle_x * line_x + middle_y * line_y;
    let length_squared = line_x * line_x + line_y * line_y;

    if along <= 0 {
        middle_x * middle_x + middle_y * middle_y <= radius * radius
    } else if along >= length_squared {
        let (end_x, end_y) = offset(to, middle);
        end_x * end_x + end_y * end_y <= radius * radius
    } else {
        let aside = line_x * middle_y - line_y * middle_x;
        aside * aside <= radius * radius * length_squared
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Object;
    use crate::world::{Area, AreaKind, Plant, PlantSpecies, Rock, Thing};

    const EYE: Location = Location::new(0, 0);

    fn tree(species: PlantSpecies, x: i64, y: i64) -> Thing {
        Thing::Plant(Plant::new(
            Object::new(Location::new(x, y), Default::default()),
            species,
            Length::from_meters(15),
            Length::from_millimeters(300),
        ))
    }

    fn boulder(x: i64, y: i64, diameter: i64, height: i64) -> Thing {
        Thing::Rock(Rock::new(
            Object::new(Location::new(x, y), Default::default()),
            Length::from_millimeters(diameter),
            Length::from_millimeters(height),
        ))
    }

    fn seen(world: &World) -> Surroundings {
        look(world, &Grid::new(world), EYE)
    }

    #[test]
    fn empty_world_has_nothing_to_see() {
        assert!(seen(&World::default()).is_empty());
    }

    #[test]
    fn thing_behind_a_trunk_is_hidden() {
        let mut world = World::default();
        let front = Subject::Thing(world.add_thing(tree(PlantSpecies::Spruce, 0, 5_000)));
        let behind = Subject::Thing(world.add_thing(tree(PlantSpecies::Pine, 0, 20_000)));
        let beside = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 3_000, 20_000)));

        let surroundings = seen(&world);

        assert!(surroundings.sighting(front).is_some());
        assert!(surroundings.sighting(behind).is_none());
        assert!(surroundings.sighting(beside).is_some());
    }

    #[test]
    fn thing_below_eye_height_hides_nothing() {
        let mut world = World::default();
        world.add_thing(boulder(0, 5_000, 2_000, 1_000));
        let behind_a_low_boulder =
            Subject::Thing(world.add_thing(tree(PlantSpecies::Pine, 0, 20_000)));
        world.add_thing(boulder(20_000, 0, 2_000, 2_000));
        let behind_a_tall_boulder =
            Subject::Thing(world.add_thing(tree(PlantSpecies::Pine, 40_000, 0)));

        let surroundings = seen(&world);

        assert!(surroundings.sighting(behind_a_low_boulder).is_some());
        assert!(surroundings.sighting(behind_a_tall_boulder).is_none());
    }

    #[test]
    fn thing_too_small_for_its_distance_is_not_made_out() {
        let is_seen = |x, y, diameter| {
            let mut world = World::default();
            world.add_thing(boulder(x, y, diameter, 200));
            !seen(&world).is_empty()
        };

        assert!(is_seen(0, 80_000, 300));
        assert!(!is_seen(80_000, 60_000, 300));
        assert!(is_seen(80_000, 60_000, 2_000));
        assert!(!is_seen(0, 400_000, 2_000));
    }

    #[test]
    fn sightings_are_sorted_into_reach_nearby_and_in_sight() {
        let mut world = World::default();
        let touching = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 0, -900)));
        let close = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, 4_000, 0)));
        let closer = Subject::Thing(world.add_thing(tree(PlantSpecies::Pine, 0, 2_000)));
        let distant = Subject::Thing(world.add_thing(tree(PlantSpecies::Birch, -30_000, 0)));

        let surroundings = seen(&world);
        let subjects = |sightings: &[Sighting]| -> Vec<Subject> {
            sightings.iter().map(|sighting| sighting.subject).collect()
        };

        assert_eq!(subjects(&surroundings.within_reach), [touching]);
        assert_eq!(subjects(&surroundings.nearby), [closer, close]);
        assert_eq!(subjects(&surroundings.in_sight), [distant]);
        assert_eq!(
            surroundings.within_reach[0],
            Sighting {
                subject: touching,
                kind: Kind::Plant(PlantSpecies::Birch),
                count: 1,
                distance: Length::from_millimeters(750),
                bearing: Bearing::South,
            }
        );
    }

    #[test]
    fn distant_things_of_a_kind_are_one_sighting_of_the_nearest() {
        let mut world = World::default();
        world.add_thing(tree(PlantSpecies::Spruce, 20_000, 30_000));
        let nearest_spruce = Subject::Thing(world.add_thing(tree(PlantSpecies::Spruce, 0, 15_000)));
        world.add_thing(tree(PlantSpecies::Spruce, -20_000, 30_000));
        let pine = Subject::Thing(world.add_thing(tree(PlantSpecies::Pine, 12_000, 0)));

        let surroundings = seen(&world);

        assert_eq!(surroundings.in_sight.len(), 2);
        assert_eq!(surroundings.in_sight[0].subject, pine);
        assert_eq!(surroundings.in_sight[0].count, 1);
        assert_eq!(
            surroundings.in_sight[1],
            Sighting {
                subject: nearest_spruce,
                kind: Kind::Plant(PlantSpecies::Spruce),
                count: 3,
                distance: Length::from_millimeters(14_850),
                bearing: Bearing::North,
            }
        );
    }

    #[test]
    fn lake_is_seen_at_the_nearest_shore_in_view() {
        let corners = [
            (30_000, -10_000),
            (50_000, -10_000),
            (50_000, 10_000),
            (30_000, 10_000),
        ];
        let lake = Area::new(
            AreaKind::Lake,
            corners
                .into_iter()
                .map(|(x, y)| Location::new(x, y))
                .collect(),
        );

        let mut world = World::default();
        let water = Subject::Area(world.add_area(lake));
        let open = seen(&world);
        assert_eq!(
            open.sighting(water),
            Some(&Sighting {
                subject: water,
                kind: Kind::Lake,
                count: 1,
                distance: Length::from_meters(30),
                bearing: Bearing::East,
            })
        );

        world.add_thing(boulder(10_000, 0, 2_000, 2_000));
        let blocked = seen(&world);
        let sighting = blocked.sighting(water).expect("the corners are in view");
        assert!(sighting.distance > Length::from_meters(30));
    }

    #[test]
    fn lake_hidden_all_along_its_shore_is_not_seen() {
        let corners = [(30_000, -500), (31_000, -500), (31_000, 500), (30_000, 500)];
        let pond = Area::new(
            AreaKind::Lake,
            corners
                .into_iter()
                .map(|(x, y)| Location::new(x, y))
                .collect(),
        );
        let mut world = World::default();
        let water = Subject::Area(world.add_area(pond));
        world.add_thing(boulder(5_000, 0, 2_500, 2_000));

        assert!(seen(&world).sighting(water).is_none());
    }
}
