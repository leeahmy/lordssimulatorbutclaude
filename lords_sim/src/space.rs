//! Places and distances on the plane the simulation is laid out on.

use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

/// A distance, with millimeter accuracy.
#[derive(
    Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Length(i64);

impl Length {
    pub const ZERO: Self = Self(0);

    pub const fn from_millimeters(millimeters: i64) -> Self {
        Self(millimeters)
    }

    pub const fn from_meters(meters: i64) -> Self {
        Self(meters * 1_000)
    }

    pub const fn as_millimeters(self) -> i64 {
        self.0
    }
}

impl Add for Length {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for Length {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

/// Represents a location on a 2-dimensional plane, with millimeter accuracy.
///
/// A location is represented by the x and y coordinates relative to the center of the plane,
/// x growing eastward and y northward.
/// The coordinates are stored as integers, and floating point errors are not a consideration.
/// This location implementation should only be used for simplistic representations of 2-dimensional flat worlds
/// and never in more complex, planetary contexts.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Location {
    x: i64,
    y: i64,
}

impl Location {
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub const fn x(self) -> i64 {
        self.x
    }

    pub const fn y(self) -> i64 {
        self.y
    }

    pub fn distance_to(self, other: Self) -> Length {
        let (dx, dy) = self.offset_to(other);
        Length((dx * dx + dy * dy).isqrt() as i64)
    }

    /// Returns the location `step` closer to `target`, or `target` itself if it is no
    /// farther away than that.
    pub fn toward(self, target: Self, step: Length) -> Self {
        let distance = i128::from(self.distance_to(target).0);
        let step = i128::from(step.0);
        if distance <= step {
            return target;
        }

        let (dx, dy) = self.offset_to(target);
        Self {
            x: self.x + (dx * step / distance) as i64,
            y: self.y + (dy * step / distance) as i64,
        }
    }

    /// Returns the compass point nearest to the direction of `other`, which is north if
    /// the two locations are the same.
    pub fn bearing_to(self, other: Self) -> Bearing {
        let (dx, dy) = self.offset_to(other);
        let (rise, run) = OCTANT_SLOPE;

        if dy.abs() * run < dx.abs() * rise {
            if dx > 0 { Bearing::East } else { Bearing::West }
        } else if dx.abs() * run <= dy.abs() * rise {
            if dy >= 0 {
                Bearing::North
            } else {
                Bearing::South
            }
        } else {
            match (dx > 0, dy > 0) {
                (true, true) => Bearing::Northeast,
                (true, false) => Bearing::Southeast,
                (false, true) => Bearing::Northwest,
                (false, false) => Bearing::Southwest,
            }
        }
    }

    fn offset_to(self, other: Self) -> (i128, i128) {
        (
            i128::from(other.x) - i128::from(self.x),
            i128::from(other.y) - i128::from(self.y),
        )
    }
}

/// The tangent of 22.5°, half the angle between two neighboring compass points, as a ratio.
const OCTANT_SLOPE: (i128, i128) = (70, 169);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bearing {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

#[cfg(test)]
mod tests {
    use super::*;

    const HERE: Location = Location::new(1_000, -2_000);

    fn offset(x: i64, y: i64) -> Location {
        Location::new(HERE.x + x, HERE.y + y)
    }

    #[test]
    fn distance_is_measured_in_a_straight_line() {
        assert_eq!(HERE.distance_to(HERE), Length::ZERO);
        assert_eq!(
            HERE.distance_to(offset(3_000, -4_000)),
            Length::from_meters(5)
        );
        assert_eq!(
            offset(3_000, -4_000).distance_to(HERE),
            Length::from_meters(5)
        );
    }

    #[test]
    fn steps_toward_a_target_end_on_it() {
        let target = offset(0, 2_500);
        let step = Length::from_meters(1);

        let first = HERE.toward(target, step);
        let second = first.toward(target, step);
        let third = second.toward(target, step);

        assert_eq!(first, offset(0, 1_000));
        assert_eq!(second, offset(0, 2_000));
        assert_eq!(third, target);
        assert_eq!(third.toward(target, step), target);
    }

    #[test]
    fn step_keeps_to_the_line_toward_the_target() {
        let target = offset(30_000, 40_000);

        assert_eq!(
            HERE.toward(target, Length::from_meters(5)),
            offset(3_000, 4_000)
        );
    }

    #[test]
    fn bearing_is_the_nearest_compass_point() {
        let bearings = [
            ((0, 10), Bearing::North),
            ((10, 10), Bearing::Northeast),
            ((10, 0), Bearing::East),
            ((10, -10), Bearing::Southeast),
            ((0, -10), Bearing::South),
            ((-10, -10), Bearing::Southwest),
            ((-10, 0), Bearing::West),
            ((-10, 10), Bearing::Northwest),
            ((4, 10), Bearing::North),
            ((5, 10), Bearing::Northeast),
            ((10, 4), Bearing::East),
            ((-10, -5), Bearing::Southwest),
        ];

        for ((x, y), bearing) in bearings {
            assert_eq!(HERE.bearing_to(offset(x, y)), bearing, "toward {x}, {y}");
        }
    }

    #[test]
    fn bearing_to_the_same_place_is_north() {
        assert_eq!(HERE.bearing_to(HERE), Bearing::North);
    }
}
