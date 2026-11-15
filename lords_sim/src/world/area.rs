use serde::{Deserialize, Serialize};

use crate::space::Location;

/// A stretch of the world's surface, bounded by an outline that closes on itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Area {
    kind: AreaKind,
    outline: Vec<Location>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AreaKind {
    Lake,
}

impl Area {
    pub fn new(kind: AreaKind, outline: Vec<Location>) -> Self {
        Self { kind, outline }
    }

    pub fn kind(&self) -> AreaKind {
        self.kind
    }

    pub fn outline(&self) -> &[Location] {
        &self.outline
    }

    pub fn contains(&self, point: Location) -> bool {
        let (x, y) = coordinates(point);
        let mut inside = false;

        for (from, to) in self.edges() {
            let (from_x, from_y) = coordinates(from);
            let (to_x, to_y) = coordinates(to);
            if (from_y > y) == (to_y > y) {
                continue;
            }

            let rise = to_y - from_y;
            let left_of_edge = (x - from_x) * rise;
            let along_edge = (y - from_y) * (to_x - from_x);
            if (rise > 0) == (left_of_edge < along_edge) && left_of_edge != along_edge {
                inside = !inside;
            }
        }

        inside
    }

    /// Returns the point of the outline nearest to `from`, which is `from` itself if the
    /// area has no outline.
    pub fn nearest_shore(&self, from: Location) -> Location {
        self.edges()
            .map(|(start, end)| nearest_on_edge(start, end, from))
            .min_by_key(|point| from.distance_to(*point))
            .unwrap_or(from)
    }

    fn edges(&self) -> impl Iterator<Item = (Location, Location)> {
        let following = self.outline.iter().cycle().skip(1);
        self.outline.iter().copied().zip(following.copied())
    }
}

fn nearest_on_edge(start: Location, end: Location, from: Location) -> Location {
    let (start_x, start_y) = coordinates(start);
    let (edge_x, edge_y) = (i128::from(end.x()) - start_x, i128::from(end.y()) - start_y);
    let (from_x, from_y) = (
        i128::from(from.x()) - start_x,
        i128::from(from.y()) - start_y,
    );

    let along = from_x * edge_x + from_y * edge_y;
    let length_squared = edge_x * edge_x + edge_y * edge_y;
    if along <= 0 {
        start
    } else if along >= length_squared {
        end
    } else {
        Location::new(
            (start_x + edge_x * along / length_squared) as i64,
            (start_y + edge_y * along / length_squared) as i64,
        )
    }
}

fn coordinates(location: Location) -> (i128, i128) {
    (i128::from(location.x()), i128::from(location.y()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A lake 20 m wide and 10 m tall, with a notch 4 m wide cut 6 m into its top.
    fn lake() -> Area {
        let outline = [
            (0, 0),
            (20_000, 0),
            (20_000, 10_000),
            (12_000, 10_000),
            (12_000, 4_000),
            (8_000, 4_000),
            (8_000, 10_000),
            (0, 10_000),
        ];

        Area::new(
            AreaKind::Lake,
            outline
                .into_iter()
                .map(|(x, y)| Location::new(x, y))
                .collect(),
        )
    }

    #[test]
    fn area_contains_only_what_its_outline_encloses() {
        let lake = lake();

        assert!(lake.contains(Location::new(2_000, 2_000)));
        assert!(lake.contains(Location::new(18_000, 9_000)));
        assert!(lake.contains(Location::new(10_000, 3_000)));

        assert!(!lake.contains(Location::new(10_000, 7_000)));
        assert!(!lake.contains(Location::new(-1, 5_000)));
        assert!(!lake.contains(Location::new(20_001, 5_000)));
        assert!(!lake.contains(Location::new(5_000, -1)));
        assert!(!lake.contains(Location::new(5_000, 10_001)));
    }

    #[test]
    fn nearest_shore_is_the_closest_point_of_the_outline() {
        let lake = lake();

        assert_eq!(
            lake.nearest_shore(Location::new(5_000, -3_000)),
            Location::new(5_000, 0)
        );
        assert_eq!(
            lake.nearest_shore(Location::new(25_000, 15_000)),
            Location::new(20_000, 10_000)
        );
        assert_eq!(
            lake.nearest_shore(Location::new(9_000, 7_000)),
            Location::new(8_000, 7_000)
        );
        assert_eq!(
            lake.nearest_shore(Location::new(3_000, 5_000)),
            Location::new(0, 5_000)
        );
    }

    #[test]
    fn area_without_an_outline_holds_nothing() {
        let nothing = Area::new(AreaKind::Lake, Vec::new());
        let somewhere = Location::new(1, 2);

        assert!(!nothing.contains(somewhere));
        assert_eq!(nothing.nearest_shore(somewhere), somewhere);
    }
}
