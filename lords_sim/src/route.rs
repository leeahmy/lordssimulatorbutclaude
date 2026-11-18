use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::space::{Length, Location};
use crate::world::Area;

/// Maximum distance to water on the route.
const CLEARANCE: Length = Length::from_meters(1);

/// How far past a corner its sides are followed to tell which way is out of it.
const FAR: Length = Length::from_meters(1_000);

/// Returns the places to turn at on the shortest way from `from` to `to` that crosses none
/// of `shapes`, which are none if the straight way is clear or there is no way around.
///
/// A shape that holds `from` or `to` is left out.
pub(crate) fn plan(from: Location, to: Location, shapes: &[Area]) -> Vec<Location> {
    let shapes: Vec<&Area> = shapes
        .iter()
        .filter(|shape| !shape.contains(from) && !shape.contains(to))
        .collect();
    let clear = |a: Location, b: Location| shapes.iter().all(|shape| !shape.crosses(a, b));
    if clear(from, to) {
        return Vec::new();
    }

    let on_dry_ground = |point: &Location| !shapes.iter().any(|shape| shape.contains(*point));
    let mut points = vec![from, to];
    points.extend(
        shapes
            .iter()
            .flat_map(|shape| corners(shape))
            .filter(on_dry_ground),
    );

    let mut cost: Vec<Option<Length>> = vec![None; points.len()];
    let mut came_from = vec![0; points.len()];
    let mut done = vec![false; points.len()];
    let mut open = BinaryHeap::from([Reverse((from.distance_to(to), 0))]);
    cost[0] = Some(Length::ZERO);

    while let Some(Reverse((_, at))) = open.pop() {
        if done[at] {
            continue;
        }
        done[at] = true;
        if at == 1 {
            let mut turns = Vec::new();
            let mut step = came_from[1];
            while step != 0 {
                turns.push(points[step]);
                step = came_from[step];
            }
            turns.reverse();
            return turns;
        }

        let here = points[at];
        let so_far = cost[at].unwrap_or_default();
        for (next, point) in points.iter().copied().enumerate() {
            let through = so_far + here.distance_to(point);
            let shorter = cost[next].is_none_or(|known| through < known);
            if !done[next] && shorter && clear(here, point) {
                cost[next] = Some(through);
                came_from[next] = at;
                open.push(Reverse((through + point.distance_to(to), next)));
            }
        }
    }

    Vec::new()
}

/// Returns the places [`CLEARANCE`] out from the corners of `shape`, leaving out corners
/// that are all but straight.
fn corners(shape: &Area) -> impl Iterator<Item = Location> + '_ {
    let outline = shape.outline();
    let count = outline.len();

    (0..count).filter_map(move |at| {
        let corner = outline[at];
        let before = corner.away_from(outline[(at + count - 1) % count], FAR);
        let after = corner.away_from(outline[(at + 1) % count], FAR);
        let out = Location::new(
            before.x() + after.x() - corner.x(),
            before.y() + after.y() - corner.y(),
        );

        (corner.distance_to(out) > CLEARANCE).then(|| corner.toward(out, CLEARANCE))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::AreaKind;

    fn shape(corners: &[(i64, i64)]) -> Area {
        Area::new(
            AreaKind::Lake,
            corners.iter().map(|(x, y)| Location::new(*x, *y)).collect(),
        )
    }

    fn square() -> Area {
        shape(&[
            (-20_000, 30_000),
            (20_000, 30_000),
            (20_000, 50_000),
            (-20_000, 50_000),
        ])
    }

    /// A lake 20 m wide and 10 m tall, with a notch 4 m wide cut 6 m into its top.
    fn notched() -> Area {
        shape(&[
            (0, 0),
            (20_000, 0),
            (20_000, 10_000),
            (12_000, 10_000),
            (12_000, 4_000),
            (8_000, 4_000),
            (8_000, 10_000),
            (0, 10_000),
        ])
    }

    #[test]
    fn clear_way_has_no_turns() {
        let to = Location::new(30_000, 20_000);

        assert!(plan(Location::default(), to, &[square()]).is_empty());
        assert!(plan(Location::default(), to, &[]).is_empty());
    }

    #[test]
    fn way_around_turns_past_the_corners_on_the_shorter_side() {
        let turns = plan(
            Location::new(5_000, 0),
            Location::new(5_000, 60_000),
            &[square()],
        );

        assert_eq!(
            turns,
            [Location::new(20_707, 29_293), Location::new(20_707, 50_707)]
        );
    }

    #[test]
    fn way_out_of_a_notch_leaves_it_before_going_around() {
        let turns = plan(
            Location::new(10_000, 7_000),
            Location::new(15_000, -5_000),
            &[notched()],
        );

        assert_eq!(
            turns,
            [
                Location::new(11_293, 10_707),
                Location::new(20_707, 10_707),
                Location::new(20_707, -707),
            ]
        );
    }

    #[test]
    fn shape_holding_the_start_or_the_end_is_left_out() {
        let inside = Location::new(0, 40_000);
        let beyond = Location::new(0, 60_000);

        assert!(plan(inside, beyond, &[square()]).is_empty());
        assert!(plan(beyond, inside, &[square()]).is_empty());
    }

    #[test]
    fn single_stretch_of_shore_is_passed_by_its_nearer_end() {
        let shore = shape(&[(-20_000, 30_000), (20_000, 30_000)]);

        let turns = plan(
            Location::new(5_000, 0),
            Location::new(5_000, 60_000),
            &[shore],
        );

        assert_eq!(turns, [Location::new(21_000, 30_000)]);
    }
}
