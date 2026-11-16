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
        self.nearest_on_shore(from).map_or(from, |(_, point)| point)
    }

    /// Returns the point of the outline nearest to `from`, and the edge it is on.
    ///
    /// An edge is numbered by the point of the outline it starts from.
    pub fn nearest_on_shore(&self, from: Location) -> Option<(u32, Location)> {
        (0..)
            .zip(self.edges())
            .map(|(edge, (start, end))| (edge, nearest_on_edge(start, end, from)))
            .min_by_key(|(_, point)| from.distance_to(*point))
    }

    /// Returns the edge that ends where the edge `edge` starts.
    pub fn edge_before(&self, edge: u32) -> u32 {
        let edges = (self.outline.len() as u32).max(1);
        (edge % edges + edges - 1) % edges
    }

    /// Returns triangles that together cover the shape that has the points `corners` of
    /// the outline at its corners, in that order, as the points at the corners of each.
    pub fn triangles(&self, corners: &[u32]) -> Vec<[u32; 3]> {
        let turn_at = |[a, b, c]: [u32; 3]| {
            turn(
                self.outline[a as usize],
                self.outline[b as usize],
                self.outline[c as usize],
            )
        };

        let mut corners = corners.to_vec();
        let doubled_area: i128 = corners
            .iter()
            .zip(corners.iter().cycle().skip(1))
            .map(|(from, to)| {
                let (from_x, from_y) = coordinates(self.outline[*from as usize]);
                let (to_x, to_y) = coordinates(self.outline[*to as usize]);
                from_x * to_y - to_x * from_y
            })
            .sum();
        if doubled_area < 0 {
            corners.reverse();
        }

        let mut triangles = Vec::new();
        while corners.len() >= 3 {
            let count = corners.len();
            let around = |at: usize| {
                [
                    corners[(at + count - 1) % count],
                    corners[at],
                    corners[(at + 1) % count],
                ]
            };
            let is_ear = |at: usize| {
                let ear = around(at);
                turn_at(ear) > 0
                    && !corners.iter().any(|corner| {
                        let point = self.outline[*corner as usize];
                        let is_corner = ear.iter().any(|own| self.outline[*own as usize] == point);
                        let [a, b, c] = ear;
                        !is_corner
                            && turn_at([a, b, *corner]) >= 0
                            && turn_at([b, c, *corner]) >= 0
                            && turn_at([c, a, *corner]) >= 0
                    })
            };

            if let Some(at) = (0..count).find(|at| is_ear(*at)) {
                triangles.push(around(at));
                corners.remove(at);
            } else if let Some(at) = (0..count).find(|at| turn_at(around(*at)) == 0) {
                corners.remove(at);
            } else {
                break;
            }
        }

        triangles
    }

    fn edges(&self) -> impl Iterator<Item = (Location, Location)> {
        let following = self.outline.iter().cycle().skip(1);
        self.outline.iter().copied().zip(following.copied())
    }
}

/// Returns a positive number if the way from `a` through `b` to `c` turns left, a negative
/// one if it turns right, and zero if it goes straight.
fn turn(a: Location, b: Location, c: Location) -> i128 {
    let ((a_x, a_y), (b_x, b_y), (c_x, c_y)) = (coordinates(a), coordinates(b), coordinates(c));

    (b_x - a_x) * (c_y - a_y) - (b_y - a_y) * (c_x - a_x)
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
    fn nearest_shore_is_on_the_edge_numbered_by_where_it_starts() {
        let lake = lake();

        assert_eq!(
            lake.nearest_on_shore(Location::new(5_000, -3_000)),
            Some((0, Location::new(5_000, 0)))
        );
        assert_eq!(
            lake.nearest_on_shore(Location::new(9_000, 7_000)),
            Some((5, Location::new(8_000, 7_000)))
        );
        assert_eq!(lake.edge_before(5), 4);
        assert_eq!(lake.edge_before(0), 7);
    }

    #[test]
    fn triangles_cover_the_area_and_nothing_else() {
        let doubled_area = |[a, b, c]: [Location; 3]| turn(a, b, c).abs();

        for outline in [lake().outline, lake().outline.into_iter().rev().collect()] {
            let lake = Area::new(AreaKind::Lake, outline);
            let triangles: Vec<[Location; 3]> = lake
                .triangles(&[0, 1, 2, 3, 4, 5, 6, 7])
                .into_iter()
                .map(|corners| corners.map(|corner| lake.outline[corner as usize]))
                .collect();

            assert_eq!(triangles.len(), 6);
            assert_eq!(
                triangles.iter().copied().map(doubled_area).sum::<i128>(),
                2 * (20_000 * 10_000 - 4_000 * 6_000)
            );
            for [a, b, c] in triangles {
                let middle =
                    Location::new((a.x() + b.x() + c.x()) / 3, (a.y() + b.y() + c.y()) / 3);
                assert!(lake.contains(middle), "{a:?}, {b:?}, {c:?}");
            }
        }
    }

    #[test]
    fn triangles_cut_straight_across_between_the_corners_given() {
        let lake = lake();
        let point = |corner: u32| lake.outline[corner as usize];
        let doubled_area = |corners: &[u32]| -> i128 {
            lake.triangles(corners)
                .into_iter()
                .map(|[a, b, c]| turn(point(a), point(b), point(c)).abs())
                .sum()
        };

        assert_eq!(doubled_area(&[0, 1, 2, 7]), 2 * 20_000 * 10_000);
        assert_eq!(doubled_area(&[0, 1, 2]), 20_000 * 10_000);
        assert_eq!(doubled_area(&[7, 2, 1, 0]), 2 * 20_000 * 10_000);
        assert!(lake.triangles(&[0, 1]).is_empty());
        assert!(lake.triangles(&[]).is_empty());
    }

    #[test]
    fn area_without_an_outline_holds_nothing() {
        let nothing = Area::new(AreaKind::Lake, Vec::new());
        let somewhere = Location::new(1, 2);

        assert!(!nothing.contains(somewhere));
        assert_eq!(nothing.nearest_shore(somewhere), somewhere);
        assert_eq!(nothing.nearest_on_shore(somewhere), None);
        assert!(nothing.triangles(&[]).is_empty());
    }
}
