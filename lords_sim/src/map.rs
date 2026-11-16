use crate::memory::Memory;
use crate::perception::{SIGHT_LIMIT, Seen};
use crate::space::{Length, Location};
use crate::time::Timestamp;
use crate::world::index::Grid;
use crate::world::{Kind, Subject, World};

/// How far from the character the map reaches.
pub const RANGE: Length = SIGHT_LIMIT;

/// What a character sees or remembers within [`RANGE`] of where it stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Map {
    pub things: Vec<Mark>,
    pub waters: Vec<Water>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    pub subject: Subject,
    pub kind: Kind,
    pub location: Location,
    /// The radius of the ground the thing stands on.
    pub footprint: Length,
    /// When the thing was last seen, or `None` if it is in sight.
    pub last_seen: Option<Timestamp>,
}

impl Mark {
    pub fn in_sight(&self) -> bool {
        self.last_seen.is_none()
    }
}

/// The known part of a body of water.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Water {
    pub subject: Subject,
    pub kind: Kind,
    pub shore: Vec<[Location; 2]>,
    pub surface: Vec<[Location; 3]>,
}

/// Returns the map of a character at `eye` who remembers `memory` and now sees `seen`.
///
/// `surfaces` are the triangles of each area of `world`, in the order of the areas.
pub(crate) fn chart(
    world: &World,
    grid: &Grid,
    surfaces: &[Vec<[u32; 3]>],
    memory: &Memory,
    seen: &Seen,
    eye: Location,
) -> Map {
    let in_range = |point: &Location| eye.distance_to(*point) <= RANGE;

    let things = grid
        .around(eye, RANGE)
        .filter_map(|id| {
            let thing = world.thing(id)?;
            let last_seen = memory.last_seen(id)?;
            let in_sight = seen.things.binary_search(&id).is_ok();

            in_range(&thing.location()).then(|| Mark {
                subject: Subject::Thing(id),
                kind: thing.kind(),
                location: thing.location(),
                footprint: thing.footprint(),
                last_seen: (!in_sight).then_some(last_seen),
            })
        })
        .collect();

    let waters = world
        .areas()
        .zip(surfaces)
        .filter_map(|((id, area), triangles)| {
            let subject = Subject::Area(id);
            let outline = area.outline();
            let point = |corner: u32| outline[corner as usize];
            let corner_known = |corner: u32| {
                memory.knows_edge(id, corner) || memory.knows_edge(id, area.edge_before(corner))
            };

            let shore: Vec<[Location; 2]> = (0..outline.len() as u32)
                .filter(|edge| memory.knows_edge(id, *edge))
                .map(|edge| [point(edge), point((edge + 1) % outline.len() as u32)])
                .filter(|ends| ends.iter().any(in_range))
                .collect();
            let surface = triangles
                .iter()
                .filter(|corners| corners.iter().all(|corner| corner_known(*corner)))
                .map(|corners| corners.map(point))
                .filter(|corners| corners.iter().any(in_range))
                .collect();

            (!shore.is_empty()).then_some(Water {
                subject,
                kind: world.kind(subject)?,
                shore,
                surface,
            })
        })
        .collect();

    Map { things, waters }
}
