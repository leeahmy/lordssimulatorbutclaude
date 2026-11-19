use std::f64::consts::TAU;

use crate::item::{Item, Stack};
use crate::object::Object;
use crate::rng::Rng;
use crate::space::{Length, Location};
use crate::time::Timestamp;
use crate::world::{Area, AreaKind, Pile, Plant, PlantSpecies, Rock, Thing, World};

/// Half the side of the square that things are placed in, around the center of the plane.
const EXTENT: i64 = 200_000;

/// The radius around the center of the plane that is left empty to stand in.
const CLEARING: i64 = 2_000;

const LAKE_VERTICES: u32 = 64;
const LAKE_RADIUS: f64 = 52_000.0;
const LAKE_DISTANCE: (i64, i64) = (100_000, 125_000);

/// How much the lake's radius swells and shrinks around its outline, as pairs of the share
/// of the radius and the number of times it does so on the way around.
const LAKE_WAVES: [(f64, f64); 3] = [(0.12, 2.0), (0.08, 3.0), (0.05, 5.0)];

const BOULDERS: u32 = 320;
const BOULDER_DIAMETER: (i64, i64) = (300, 2_500);
const BOULDER_HEIGHT_PERCENT: (i64, i64) = (40, 80);

/// The side of the squares of which each has one place for a tree.
const TREE_SPACING: i64 = 3_900;
const TREE_GAP_PERCENT: u64 = 10;
const TRUNK_DIAMETER: (i64, i64) = (80, 450);
const TREE_SLENDERNESS: (i64, i64) = (55, 75);

/// The side of the squares in which either spruce or pine is the common tree.
const PATCH: i64 = 40_000;
const COMMON_TREE_PERCENT: u64 = 70;
const BIRCH_PERCENT: u64 = 15;

/// How far from the water birch is the common tree.
const SHORE: i64 = 15_000;
const SHORE_BIRCH_PERCENT: u64 = 60;

/// How many places are tried for loose stones, and how many stones lie in one.
const STONE_PLACES: u32 = 260;
const STONES_IN_A_PLACE: (i64, i64) = (1, 3);

/// How many places are tried for patches of each berry.
const BERRY_PLACES: u32 = 420;
const PATCH_WIDTH: (i64, i64) = (1_000, 3_000);
const BILBERRY_HEIGHT: i64 = 300;
const LINGONBERRY_HEIGHT: i64 = 150;

/// The chance of a berry patch growing under the trees it does not favor: bilberries
/// favor spruce, and lingonberries pine.
const UNFAVORED_PERCENT: u64 = 30;

impl World {
    /// Returns a forest with a lake in it, the same one every time for the same `seed`.
    ///
    /// The center of the plane is left clear and without water.
    pub fn generate(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let lake = lake(&mut rng);
        let boulders = boulders(&mut rng, &lake);
        let (trees, patches) = trees(&mut rng, &lake, &boulders);
        let stones = stones(&mut rng, &lake);
        let berries = berries(&mut rng, &lake, &patches);

        let mut world = Self::default();
        world.add_area(lake);
        for thing in boulders
            .into_iter()
            .chain(trees)
            .chain(stones)
            .chain(berries)
        {
            world.add_thing(thing);
        }
        world
    }
}

fn lake(rng: &mut Rng) -> Area {
    let direction = rng.unit() * TAU;
    let distance = rng.between(LAKE_DISTANCE) as f64;
    let (center_x, center_y) = (distance * direction.cos(), distance * direction.sin());
    let waves = LAKE_WAVES.map(|(share, times)| (share, times, rng.unit() * TAU));

    let outline = (0..LAKE_VERTICES)
        .map(|vertex| {
            let angle = TAU * f64::from(vertex) / f64::from(LAKE_VERTICES);
            let swell: f64 = waves
                .iter()
                .map(|(share, times, shift)| share * (times * angle + shift).cos())
                .sum();
            let radius = LAKE_RADIUS * (1.0 + swell);

            Location::new(
                (center_x + radius * angle.cos()).round() as i64,
                (center_y + radius * angle.sin()).round() as i64,
            )
        })
        .collect();

    Area::new(AreaKind::Lake, outline)
}

fn boulders(rng: &mut Rng, lake: &Area) -> Vec<Thing> {
    let mut boulders = Vec::new();

    for _ in 0..BOULDERS {
        let location = Location::new(
            rng.between((-EXTENT, EXTENT)),
            rng.between((-EXTENT, EXTENT)),
        );
        let diameter = rng.mostly_low(BOULDER_DIAMETER);
        let height = diameter * rng.between(BOULDER_HEIGHT_PERCENT) / 100;

        if is_free(location, diameter / 2, lake) {
            boulders.push(Thing::Rock(Rock::new(
                object(location),
                Length::from_millimeters(diameter),
                Length::from_millimeters(height),
            )));
        }
    }

    boulders
}

/// The squares of the forest in which spruce is the common tree. Pine is in the rest.
struct Patches {
    spruce: Vec<bool>,
}

impl Patches {
    const PER_SIDE: i64 = 2 * EXTENT / PATCH + 1;

    fn new(rng: &mut Rng) -> Self {
        Self {
            spruce: (0..Self::PER_SIDE * Self::PER_SIDE)
                .map(|_| rng.chance(50))
                .collect(),
        }
    }

    fn is_spruce(&self, location: Location) -> bool {
        let column = (location.x() + EXTENT) / PATCH;
        let row = (location.y() + EXTENT) / PATCH;
        self.spruce[(row * Self::PER_SIDE + column) as usize]
    }
}

fn trees(rng: &mut Rng, lake: &Area, boulders: &[Thing]) -> (Vec<Thing>, Patches) {
    let patches = Patches::new(rng);
    let is_spruce_patch = |location: Location| patches.is_spruce(location);

    let places_per_side = 2 * EXTENT / TREE_SPACING;
    let jitter = (TREE_SPACING / 10, TREE_SPACING * 9 / 10);
    let mut trees = Vec::new();

    for row in 0..places_per_side {
        for column in 0..places_per_side {
            let location = Location::new(
                -EXTENT + column * TREE_SPACING + rng.between(jitter),
                -EXTENT + row * TREE_SPACING + rng.between(jitter),
            );
            let trunk_diameter = rng.mostly_low(TRUNK_DIAMETER);
            let height = trunk_diameter * rng.between(TREE_SLENDERNESS);
            let by_the_shore = location
                .distance_to(lake.nearest_shore(location))
                .as_millimeters()
                < SHORE;
            let species = species(rng, by_the_shore, is_spruce_patch(location));

            let radius = trunk_diameter / 2;
            let on_a_boulder = boulders.iter().any(|boulder| {
                let room = boulder.footprint() + Length::from_millimeters(radius);
                location.distance_to(boulder.location()) <= room
            });
            if rng.chance(TREE_GAP_PERCENT) || on_a_boulder || !is_free(location, radius, lake) {
                continue;
            }

            trees.push(Thing::Plant(Plant::new(
                object(location),
                species,
                Length::from_millimeters(height),
                Length::from_millimeters(trunk_diameter),
            )));
        }
    }

    (trees, patches)
}

fn stones(rng: &mut Rng, lake: &Area) -> Vec<Thing> {
    let mut stones = Vec::new();

    for _ in 0..STONE_PLACES {
        let location = Location::new(
            rng.between((-EXTENT, EXTENT)),
            rng.between((-EXTENT, EXTENT)),
        );
        let count = rng.between(STONES_IN_A_PLACE) as u32;
        let pile = Pile::new(object(location), [Stack::new(Item::Stone, count)]);
        let pile = Thing::Pile(pile);

        if is_free(location, pile.footprint().as_millimeters(), lake) {
            stones.push(pile);
        }
    }

    stones
}

const CHANTERELLE_HEIGHT: i64 = 80;
const NETTLE_HEIGHT: i64 = 700;

fn berries(rng: &mut Rng, lake: &Area, patches: &Patches) -> Vec<Thing> {
    let kinds = [
        (PlantSpecies::Bilberry, BILBERRY_HEIGHT, true),
        (PlantSpecies::Lingonberry, LINGONBERRY_HEIGHT, false),
        (PlantSpecies::Chanterelle, CHANTERELLE_HEIGHT, true),
        (PlantSpecies::Nettle, NETTLE_HEIGHT, false),
    ];
    let mut berries = Vec::new();

    for (species, height, favors_spruce) in kinds {
        for _ in 0..BERRY_PLACES {
            let location = Location::new(
                rng.between((-EXTENT, EXTENT)),
                rng.between((-EXTENT, EXTENT)),
            );
            let width = rng.between(PATCH_WIDTH);
            let favored = patches.is_spruce(location) == favors_spruce;
            let grows = favored || rng.chance(UNFAVORED_PERCENT);

            if grows && is_free(location, width / 2, lake) {
                berries.push(Thing::Plant(Plant::new(
                    object(location),
                    species,
                    Length::from_millimeters(height),
                    Length::from_millimeters(width),
                )));
            }
        }
    }

    berries
}

fn species(rng: &mut Rng, by_the_shore: bool, spruce_patch: bool) -> PlantSpecies {
    let common = if spruce_patch {
        PlantSpecies::Spruce
    } else {
        PlantSpecies::Pine
    };
    let rare = if spruce_patch {
        PlantSpecies::Pine
    } else {
        PlantSpecies::Spruce
    };

    let roll = rng.below(100);
    if by_the_shore {
        if roll < SHORE_BIRCH_PERCENT {
            PlantSpecies::Birch
        } else {
            common
        }
    } else if roll < COMMON_TREE_PERCENT {
        common
    } else if roll < COMMON_TREE_PERCENT + BIRCH_PERCENT {
        PlantSpecies::Birch
    } else {
        rare
    }
}

/// Returns whether something of `radius` can stand at `location`, which it can on dry
/// ground outside the clearing.
fn is_free(location: Location, radius: i64, lake: &Area) -> bool {
    let from_center = Location::default().distance_to(location).as_millimeters();

    from_center > CLEARING + radius && !lake.contains(location)
}

fn object(location: Location) -> Object {
    Object::new(location, Timestamp::EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Kind;

    #[test]
    fn same_seed_gives_the_same_world() {
        assert_eq!(World::generate(7), World::generate(7));
        assert_ne!(World::generate(7), World::generate(8));
    }

    #[test]
    fn forest_has_a_lake_and_all_kinds_of_things() {
        let world = World::generate(1);

        let count = |kind| {
            world
                .things()
                .filter(|(_, thing)| thing.kind() == kind)
                .count()
        };
        let trees = count(Kind::Plant(PlantSpecies::Spruce))
            + count(Kind::Plant(PlantSpecies::Pine))
            + count(Kind::Plant(PlantSpecies::Birch));

        assert_eq!(world.areas().count(), 1);
        assert!((7_000..10_000).contains(&trees), "{trees} trees");
        assert!(count(Kind::Plant(PlantSpecies::Spruce)) > 1_000);
        assert!(count(Kind::Plant(PlantSpecies::Pine)) > 1_000);
        assert!(count(Kind::Plant(PlantSpecies::Birch)) > 500);
        assert!((250..=320).contains(&count(Kind::Rock)));
        assert!((200..=260).contains(&count(Kind::Pile(Item::Stone))));
        assert!(count(Kind::Plant(PlantSpecies::Bilberry)) > 150);
        assert!(count(Kind::Plant(PlantSpecies::Lingonberry)) > 150);
    }

    #[test]
    fn nothing_stands_in_the_water_or_at_the_start() {
        for seed in 0..4 {
            let world = World::generate(seed);
            let start = Location::default();

            assert!(!world.is_water(start));
            for (_, thing) in world.things() {
                let location = thing.location();
                let room = start.distance_to(location) - thing.footprint();

                assert!(!world.is_water(location), "{thing:?} of seed {seed}");
                assert!(room.as_millimeters() > CLEARING, "{thing:?} of seed {seed}");
            }
        }
    }

    #[test]
    fn everything_stands_inside_the_square() {
        let world = World::generate(3);

        for (_, thing) in world.things() {
            let location = thing.location();
            assert!(location.x().abs() <= EXTENT && location.y().abs() <= EXTENT);
        }
        for (_, area) in world.areas() {
            for point in area.outline() {
                assert!(point.x().abs() <= EXTENT && point.y().abs() <= EXTENT);
            }
        }
    }
}
