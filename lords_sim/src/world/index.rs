use std::collections::HashMap;

use crate::space::{Length, Location};
use crate::world::{ThingId, World};

const CELL: i64 = 10_000;

/// Finds the things of a [`World`] by where they are.
///
/// It is worked out from the world, and has to be told of every thing added since. Things
/// do not move, and those taken out of the world are still found here.
pub(crate) struct Grid {
    cells: HashMap<(i64, i64), Vec<ThingId>>,
    widest_footprint: i64,
}

impl Grid {
    pub(crate) fn new(world: &World) -> Self {
        let mut cells: HashMap<_, Vec<ThingId>> = HashMap::new();
        let mut widest_footprint = 0;

        for (id, thing) in world.things() {
            cells.entry(cell_of(thing.location())).or_default().push(id);
            widest_footprint = widest_footprint.max(thing.footprint().as_millimeters());
        }

        Self {
            cells,
            widest_footprint,
        }
    }

    /// Takes note of a thing added to the world since the grid was made.
    pub(crate) fn insert(&mut self, id: ThingId, location: Location, footprint: Length) {
        self.cells.entry(cell_of(location)).or_default().push(id);
        self.widest_footprint = self.widest_footprint.max(footprint.as_millimeters());
    }

    pub(crate) fn around(
        &self,
        center: Location,
        distance: Length,
    ) -> impl Iterator<Item = ThingId> + '_ {
        self.within(Cells::around(center, distance.as_millimeters()))
    }

    /// Returns every thing whose footprint touches the straight line between `from` and
    /// `to`, among others near it, some of them more than once.
    pub(crate) fn along(&self, from: Location, to: Location) -> impl Iterator<Item = ThingId> + '_ {
        let (dx, dy) = (to.x() - from.x(), to.y() - from.y());
        let steps = dx.abs().max(dy.abs()) / (CELL / 2) + 1;
        let margin = CELL / 4 + self.widest_footprint + 1;

        let mut last = None;
        (0..=steps)
            .filter_map(move |step| {
                let point =
                    Location::new(from.x() + dx * step / steps, from.y() + dy * step / steps);
                let cells = Cells::around(point, margin);
                (last.replace(cells) != Some(cells)).then_some(cells)
            })
            .flat_map(move |cells| self.within(cells))
    }

    fn within(&self, cells: Cells) -> impl Iterator<Item = ThingId> + '_ {
        (cells.west..=cells.east)
            .flat_map(move |column| (cells.south..=cells.north).map(move |row| (column, row)))
            .filter_map(move |cell| self.cells.get(&cell))
            .flatten()
            .copied()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Cells {
    west: i64,
    east: i64,
    south: i64,
    north: i64,
}

impl Cells {
    fn around(center: Location, margin: i64) -> Self {
        let (west, south) = cell_of(Location::new(center.x() - margin, center.y() - margin));
        let (east, north) = cell_of(Location::new(center.x() + margin, center.y() + margin));

        Self {
            west,
            east,
            south,
            north,
        }
    }
}

fn cell_of(location: Location) -> (i64, i64) {
    (location.x().div_euclid(CELL), location.y().div_euclid(CELL))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::object::Object;
    use crate::world::{Rock, Thing};

    fn boulder(x: i64, y: i64, diameter: i64) -> Thing {
        Thing::Rock(Rock::new(
            Object::new(Location::new(x, y), Default::default()),
            Length::from_millimeters(diameter),
            Length::from_meters(1),
        ))
    }

    #[test]
    fn things_around_a_place_are_found_and_distant_ones_left_out() {
        let mut world = World::default();
        let near = world.add_thing(boulder(-3_000, 4_000, 500));
        let across_a_cell_border = world.add_thing(boulder(12_000, -1_000, 500));
        let far = world.add_thing(boulder(90_000, 90_000, 500));
        let grid = Grid::new(&world);

        let found: HashSet<ThingId> = grid
            .around(Location::default(), Length::from_meters(15))
            .collect();

        assert!(found.contains(&near));
        assert!(found.contains(&across_a_cell_border));
        assert!(!found.contains(&far));
    }

    #[test]
    fn things_touching_a_line_are_found_wherever_their_middle_is() {
        let mut world = World::default();
        let on_the_line = world.add_thing(boulder(40_000, 40_000, 500));
        let beside_it_in_the_next_cell = world.add_thing(boulder(49_000, 51_000, 4_000));
        let off_to_the_side = world.add_thing(boulder(80_000, 10_000, 500));
        let beyond_the_end = world.add_thing(boulder(150_000, 150_000, 500));
        let grid = Grid::new(&world);

        let found: HashSet<ThingId> = grid
            .along(Location::default(), Location::new(100_000, 100_000))
            .collect();

        assert!(found.contains(&on_the_line));
        assert!(found.contains(&beside_it_in_the_next_cell));
        assert!(!found.contains(&off_to_the_side));
        assert!(!found.contains(&beyond_the_end));
    }
}
