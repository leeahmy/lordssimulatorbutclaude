mod art;

use std::iter;
use std::ops::RangeInclusive;

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke};
use lords_sim::climate::Light;
use lords_sim::map::{Map, Mark, RANGE, Water};
use lords_sim::object::Living;
use lords_sim::people::Person;
use lords_sim::time::gregorian::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use lords_sim::world::{Kind, StructureKind};
use lords_sim::{Command, Length, Location, Subject, Timestamp, View};

use crate::scenes::count;

use self::art::{Art, Dress, Sprite, TILE_METERS};
use super::Pointed;
use super::surroundings::{distance, name};

/// How many meters there are from the player to the edges of the map at first.
pub const DEFAULT_REACH: f32 = 30.0;
const REACH_LIMITS: RangeInclusive<f32> = 10.0..=RANGE.as_millimeters() as f32 / 1_000.0;
const ZOOM_PER_SCROLLED_POINT: f32 = 0.004;

const PANEL_WIDTH: f32 = 320.0;
const PANEL_MIN_WIDTH: f32 = 160.0;

/// The width left to the rest of the overview, without which the map is not shown.
const ROOM_FOR_THE_LIST: f32 = 380.0;

/// How many times wider than its trunk a tree is drawn.
const CANOPY_SPREAD: f32 = 8.0;
const SMALLEST_DOT: f32 = 2.0;

const FRESHEST: f32 = 0.6;
const FAINTEST: f32 = 0.15;

const FADING_HALF_LIFE: f32 = SECONDS_PER_HOUR as f32;

/// How far outside a dot or a shore the pointer still points at it.
const POINTING_SLACK: f32 = 4.0;

/// The least width that the pictures of things, of people, and of the player are drawn at.
const SMALLEST_SPRITE: f32 = 8.0;
const FIGURE_WIDTH: f32 = 8.0;
const PLAYER_WIDTH: f32 = 10.0;

/// How many meters around it a fire is drawn to light up at night.
const FIRE_GLOW: f32 = 6.0;

/// What is written and drawn over the map, and what is put under it to set it off.
const INK: Color32 = Color32::from_rgb(250, 248, 238);
const SHADOW: Color32 = Color32::from_rgba_premultiplied(14, 12, 18, 170);
const SHORE: Color32 = Color32::from_rgb(226, 208, 152);

/// The whole of a picture.
const WHOLE: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));

const HIGHLIGHT_GAP: f32 = 3.0;
const MARGIN: f32 = 10.0;
const LABEL_SIZE: f32 = 12.0;

/// The lengths a scale bar may have, in meters.
const SCALE_BARS: [f32; 6] = [2.0, 5.0, 10.0, 20.0, 50.0, 100.0];

pub fn show(
    ui: &mut egui::Ui,
    view: &View,
    reach: &mut f32,
    highlighted: Option<Subject>,
) -> Pointed {
    let room = ui.available_width() - ROOM_FOR_THE_LIST;
    if room < PANEL_MIN_WIDTH {
        return Pointed::default();
    }

    egui::Panel::right("map_panel")
        .default_size(PANEL_WIDTH.min(room))
        .size_range(PANEL_MIN_WIDTH..=room)
        .show(ui, |ui| chart(ui, view, reach, highlighted))
        .inner
}

fn chart(ui: &mut egui::Ui, view: &View, reach: &mut f32, highlighted: Option<Subject>) -> Pointed {
    let side = ui.available_width().min(ui.available_height());
    ui.take_available_width();
    let (response, painter) = ui.allocate_painter(egui::Vec2::splat(side), Sense::click());
    let rect = response.rect;
    if response.hovered() {
        zoom(ui, reach);
    }

    let ctx = ui.ctx().clone();
    let art = Art::of(&ctx);
    let dress = Dress::of(&view.conditions);
    let painter = painter.with_clip_rect(rect);
    let map = &*view.map;
    let projection = Projection {
        center: rect.center(),
        origin: view.player.location(),
        points_per_meter: side / 2.0 / *reach,
    };

    // People are pointed at and drawn like things that are always in sight.
    let figures: Vec<Mark> = map
        .people
        .iter()
        .map(|figure| Mark {
            subject: figure.subject,
            kind: Kind::Person(figure.bond),
            location: figure.location,
            footprint: Person::GIRTH,
            last_seen: None,
        })
        .collect();
    let mut dots: Vec<Dot<'_>> = map
        .things
        .iter()
        .chain(&figures)
        .map(|mark| Dot::of(mark, projection))
        .filter(|dot| {
            rect.expand(dot.radius + SMALLEST_SPRITE)
                .contains(dot.center)
        })
        .collect();
    // What stands farther south is drawn over what stands behind it.
    dots.sort_by(|a, b| a.center.y.total_cmp(&b.center.y));

    let pointed_at = response
        .hover_pos()
        .and_then(|pointer| pointed_at(pointer, &dots, &map.waters, projection));
    let lit = |subject| pointed_at == Some(subject) || highlighted == Some(subject);

    let ground = art.texture(&ctx, Sprite::Ground, dress, false);
    painter.image(
        ground.id(),
        rect,
        Rect::from_min_max(projection.tile_at(rect.min), projection.tile_at(rect.max)),
        Color32::WHITE,
    );

    let waves = art.texture(&ctx, Sprite::Water, dress, false);
    for water in &map.waters {
        let width = if lit(water.subject) { 4.0 } else { 2.0 };
        painter.add(surface(water, projection, waves.id()));
        for ends in &water.shore {
            let ends = ends.map(|end| projection.position(end));
            painter.line_segment(ends, Stroke::new(width, SHORE));
        }
    }

    // A fire flickers: its picture is turned around every other second.
    let flicker = view.time.as_seconds() % 2 == 1;
    let mut rings = Vec::new();
    let mut fires = Vec::new();
    for dot in &dots {
        let kind = dot.mark.kind;
        let burning = kind == Kind::Structure(StructureKind::Campfire);
        let texture = art.texture(&ctx, Sprite::Thing(kind), dress, burning && flicker);
        let size = sprite_size(dot, &texture);
        let tint = match dot.mark.last_seen {
            None => Color32::WHITE,
            Some(last_seen) => {
                let age = view.time.seconds_since(last_seen);
                Color32::WHITE.gamma_multiply(opacity(age))
            }
        };

        painter.image(
            texture.id(),
            Rect::from_center_size(dot.center, size),
            WHOLE,
            tint,
        );
        if lit(dot.mark.subject) {
            rings.push((dot.center, size.x.max(size.y) / 2.0 + HIGHLIGHT_GAP));
        }
        if burning && dot.mark.in_sight() {
            fires.push(dot.center);
        }
    }

    if let Some(walk) = &view.walk {
        let turns = walk.via.iter().chain([&walk.destination]);
        let route: Vec<Pos2> = iter::once(rect.center())
            .chain(turns.map(|turn| projection.position(*turn)))
            .collect();
        painter.add(Shape::line(route.clone(), Stroke::new(3.0, SHADOW)));
        painter.add(Shape::line(route, Stroke::new(1.5, INK)));
    }

    let player = art.texture(&ctx, Sprite::Player, dress, false);
    let width = (projection.points_per_meter * 1.5).max(PLAYER_WIDTH);
    let [texels_wide, texels_high] = player.size();
    let height = width * texels_high as f32 / texels_wide as f32;
    painter.image(
        player.id(),
        Rect::from_center_size(rect.center(), egui::vec2(width, height)),
        WHOLE,
        Color32::WHITE,
    );

    let darkness = match view.conditions.light {
        Light::Day => 0,
        Light::Twilight => 70,
        Light::Dusk => 120,
        Light::Dark => 165,
    };
    if darkness > 0 {
        let night = Color32::from_rgba_unmultiplied(10, 14, 44, darkness);
        painter.rect_filled(rect, 0.0, night);
        for fire in fires {
            let glow = projection.points_per_meter * FIRE_GLOW;
            painter.circle_filled(
                fire,
                glow,
                Color32::from_rgba_unmultiplied(255, 176, 72, 36),
            );
            painter.circle_filled(
                fire,
                glow / 2.0,
                Color32::from_rgba_unmultiplied(255, 200, 96, 48),
            );
        }
    }

    for (center, radius) in rings {
        painter.circle_stroke(center, radius, Stroke::new(3.0, SHADOW));
        painter.circle_stroke(center, radius, Stroke::new(1.5, INK));
    }
    legend(&painter, rect, projection);

    let clicked = response.clicked();
    if let Some(subject) = pointed_at {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        let description = match view.person(subject) {
            Some(person) if person.met => Some(format!(
                "{}, {}",
                person.name,
                person.bond.name().to_lowercase()
            )),
            Some(_) => Some("Someone you have not met".to_owned()),
            None => describe(subject, map, projection, view.time),
        };
        if let Some(description) = description {
            response.on_hover_text_at_pointer(description);
        }
    }

    Pointed {
        command: pointed_at.filter(|_| clicked).map(Command::WalkTo),
        hovered: pointed_at,
    }
}

/// Returns how large the picture of what `dot` is of is drawn: as wide as the thing is,
/// but never too small to make out, and as high as the picture is to its width.
fn sprite_size(dot: &Dot<'_>, texture: &egui::TextureHandle) -> egui::Vec2 {
    let smallest = match dot.mark.kind {
        Kind::Person(_) => FIGURE_WIDTH,
        _ => SMALLEST_SPRITE,
    };
    let width = (dot.radius * 2.0).max(smallest);
    let [texels_wide, texels_high] = texture.size();

    egui::vec2(
        width,
        width * texels_high as f32 / texels_wide.max(1) as f32,
    )
}

fn zoom(ui: &egui::Ui, reach: &mut f32) {
    let (scrolled, pinched) = ui.input(|input| (input.smooth_scroll_delta.y, input.zoom_delta()));
    let closer = pinched * (scrolled * ZOOM_PER_SCROLLED_POINT).exp();

    *reach = (*reach / closer).clamp(*REACH_LIMITS.start(), *REACH_LIMITS.end());
}

/// Draws which way north is and how long a round distance is on the map.
fn legend(painter: &egui::Painter, rect: Rect, projection: Projection) {
    let font = FontId::proportional(LABEL_SIZE);
    let inked = |at: Pos2, anchor: Align2, text: String| {
        painter.text(
            at + egui::vec2(1.0, 1.0),
            anchor,
            &text,
            font.clone(),
            SHADOW,
        );
        painter.text(at, anchor, &text, font.clone(), INK);
    };

    let north = rect.center_top() + egui::vec2(0.0, MARGIN / 2.0);
    inked(north, Align2::CENTER_TOP, "N".to_owned());

    let longest = rect.width() / 3.0 / projection.points_per_meter;
    let meters = SCALE_BARS
        .into_iter()
        .rev()
        .find(|meters| *meters <= longest)
        .unwrap_or(SCALE_BARS[0]);
    let start = rect.left_bottom() + egui::vec2(MARGIN, -MARGIN);
    let end = start + egui::vec2(meters * projection.points_per_meter, 0.0);
    painter.line_segment([start, end], Stroke::new(3.5, SHADOW));
    painter.line_segment([start, end], Stroke::new(1.5, INK));
    inked(
        start + egui::vec2(0.0, -MARGIN / 2.0),
        Align2::LEFT_BOTTOM,
        format!("{meters} m"),
    );
}

/// Returns the known surface of `water`, laid with the tiles of `texture`.
fn surface(water: &Water, projection: Projection, texture: egui::TextureId) -> Shape {
    let mut mesh = egui::Mesh::with_texture(texture);
    for triangle in &water.surface {
        let first = mesh.vertices.len() as u32;
        for corner in triangle {
            let pos = projection.position(*corner);
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv: projection.tile_at(pos),
                color: Color32::WHITE,
            });
        }
        mesh.add_triangle(first, first + 1, first + 2);
    }

    Shape::mesh(mesh)
}

/// Returns what the map shows nearest to under `pointer`, things before water.
fn pointed_at(
    pointer: Pos2,
    dots: &[Dot<'_>],
    waters: &[Water],
    projection: Projection,
) -> Option<Subject> {
    let dot = dots
        .iter()
        .filter(|dot| dot.center.distance(pointer) <= dot.radius + POINTING_SLACK)
        .min_by(|a, b| {
            let (a, b) = (a.center.distance(pointer), b.center.distance(pointer));
            a.total_cmp(&b)
        });
    let water = || {
        waters.iter().find(|water| {
            let on_the_surface = water
                .surface
                .iter()
                .any(|triangle| is_inside(pointer, triangle.map(|at| projection.position(at))));
            on_the_surface || distance_to_shore(pointer, water, projection) <= POINTING_SLACK
        })
    };

    dot.map(|dot| dot.mark.subject)
        .or_else(|| water().map(|water| water.subject))
}

fn opacity(age: u64) -> f32 {
    let left_to_fade = 0.5_f32.powf(age as f32 / FADING_HALF_LIFE);

    FAINTEST + (FRESHEST - FAINTEST) * left_to_fade
}

/// Returns the words for how long ago `age` in-game seconds were.
fn ago(age: u64) -> String {
    let minutes = age / SECONDS_PER_MINUTE;
    let hours = age / SECONDS_PER_HOUR;
    let days = age / SECONDS_PER_DAY;

    if minutes == 0 {
        "moments ago".to_owned()
    } else if hours == 0 {
        format!("{} ago", count(minutes, "minute"))
    } else if days == 0 {
        format!("{} ago", count(hours, "hour"))
    } else {
        format!("{} ago", count(days, "day"))
    }
}

/// Returns the words for what `subject` is, how far from the player, and when a thing no
/// longer in sight was last seen.
fn describe(subject: Subject, map: &Map, projection: Projection, now: Timestamp) -> Option<String> {
    let mark = map.things.iter().find(|mark| mark.subject == subject);
    let water = map.waters.iter().find(|water| water.subject == subject);

    if let Some(mark) = mark {
        let away = projection.origin.distance_to(mark.location) - mark.footprint;
        let seen = mark
            .last_seen
            .map(|last_seen| format!(", seen {}", ago(now.seconds_since(last_seen))))
            .unwrap_or_default();

        Some(format!(
            "{}, {}{seen}",
            name(mark.kind),
            distance(away.max(Length::ZERO))
        ))
    } else {
        let water = water?;
        let points = distance_to_shore(projection.center, water, projection);
        let away =
            Length::from_millimeters((points / projection.points_per_meter * 1_000.0) as i64);

        Some(format!("{}, {}", name(water.kind), distance(away)))
    }
}

/// Returns how many points there are on the map from `point` to the known shore of `water`.
fn distance_to_shore(point: Pos2, water: &Water, projection: Projection) -> f32 {
    water
        .shore
        .iter()
        .map(|ends| distance_to_segment(point, ends.map(|end| projection.position(end))))
        .fold(f32::INFINITY, f32::min)
}

fn distance_to_segment(point: Pos2, [start, end]: [Pos2; 2]) -> f32 {
    let along = end - start;
    let length_squared = along.length_sq();
    if length_squared == 0.0 {
        return start.distance(point);
    }

    let share = ((point - start).dot(along) / length_squared).clamp(0.0, 1.0);
    (start + along * share).distance(point)
}

fn is_inside(point: Pos2, [a, b, c]: [Pos2; 3]) -> bool {
    let turn = |from: Pos2, to: Pos2| {
        (to.x - from.x) * (point.y - from.y) - (to.y - from.y) * (point.x - from.x)
    };
    let turns = [turn(a, b), turn(b, c), turn(c, a)];

    turns.iter().all(|turn| *turn >= 0.0) || turns.iter().all(|turn| *turn <= 0.0)
}

#[derive(Copy, Clone, Debug)]
struct Projection {
    center: Pos2,
    origin: Location,
    points_per_meter: f32,
}

impl Projection {
    fn position(self, location: Location) -> Pos2 {
        let east = (location.x() - self.origin.x()) as f32 / 1_000.0;
        let north = (location.y() - self.origin.y()) as f32 / 1_000.0;

        self.center + egui::vec2(east, -north) * self.points_per_meter
    }

    /// Returns where on the tiles that the ground is laid with `position` on the map is,
    /// counted in tiles, so that the ground stays put under the things on it.
    fn tile_at(self, position: Pos2) -> Pos2 {
        let east = self.origin.x() as f32 / 1_000.0;
        let north = self.origin.y() as f32 / 1_000.0;
        let across = (position - self.center) / self.points_per_meter;

        Pos2::new(
            (east + across.x) / TILE_METERS,
            (-north + across.y) / TILE_METERS,
        )
    }

    fn length(self, length: Length) -> f32 {
        length.as_millimeters() as f32 / 1_000.0 * self.points_per_meter
    }
}

#[derive(Copy, Clone, Debug)]
struct Dot<'a> {
    mark: &'a Mark,
    center: Pos2,
    radius: f32,
}

impl<'a> Dot<'a> {
    fn of(mark: &'a Mark, projection: Projection) -> Self {
        let spread = if mark.kind.is_tree() {
            CANOPY_SPREAD
        } else {
            1.0
        };

        Self {
            mark,
            center: projection.position(mark.location),
            radius: (projection.length(mark.footprint) * spread).max(SMALLEST_DOT),
        }
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::object::Object;
    use lords_sim::world::{PlantSpecies, Rock, Thing};
    use lords_sim::{Name, Sex, Simulation, World};

    use super::*;
    use crate::scenes::harness::Harness;

    const PROJECTION: Projection = Projection {
        center: Pos2::new(200.0, 150.0),
        origin: Location::new(5_000, -3_000),
        points_per_meter: 2.0,
    };

    fn subjects() -> [Subject; 2] {
        let mut world = World::default();
        let pebble = || Thing::Rock(Rock::new(Object::default(), Length::ZERO, Length::ZERO));

        [(); 2].map(|()| Subject::Thing(world.add_thing(pebble())))
    }

    fn mark(subject: Subject, kind: Kind, x: i64, y: i64, footprint: i64) -> Mark {
        Mark {
            subject,
            kind,
            location: Location::new(x, y),
            footprint: Length::from_millimeters(footprint),
            last_seen: None,
        }
    }

    #[test]
    fn player_is_in_the_middle_with_north_up_and_east_right() {
        let at = |x: i64, y: i64| PROJECTION.position(Location::new(5_000 + x, -3_000 + y));

        assert_eq!(at(0, 0), Pos2::new(200.0, 150.0));
        assert_eq!(at(0, 10_000), Pos2::new(200.0, 130.0));
        assert_eq!(at(10_000, 0), Pos2::new(220.0, 150.0));
        assert_eq!(at(-2_500, -5_000), Pos2::new(195.0, 160.0));
        assert_eq!(PROJECTION.length(Length::from_millimeters(1_500)), 3.0);
    }

    #[test]
    fn ground_stays_put_under_what_is_on_it() {
        let moved = Projection {
            origin: Location::new(5_000 + 8_000, -3_000),
            ..PROJECTION
        };
        let tile = PROJECTION.tile_at(Pos2::new(200.0, 150.0));

        // Eight meters east is one tile over, which is 16 points at this scale.
        assert_eq!(moved.tile_at(Pos2::new(200.0, 150.0)).x, tile.x + 1.0);
        assert_eq!(PROJECTION.tile_at(Pos2::new(216.0, 150.0)).x, tile.x + 1.0);
        assert_eq!(PROJECTION.tile_at(Pos2::new(200.0, 166.0)).y, tile.y + 1.0);
    }

    #[test]
    fn trees_are_drawn_wider_than_their_trunks_and_nothing_too_small_to_see() {
        let [subject, _] = subjects();
        let radius =
            |kind, footprint| Dot::of(&mark(subject, kind, 0, 0, footprint), PROJECTION).radius;

        assert_eq!(radius(Kind::Plant(PlantSpecies::Pine), 200), 3.2);
        assert_eq!(radius(Kind::Rock, 1_250), 2.5);
        assert_eq!(radius(Kind::Rock, 150), SMALLEST_DOT);
        assert_eq!(radius(Kind::Plant(PlantSpecies::Pine), 40), SMALLEST_DOT);
    }

    #[test]
    fn pointer_picks_the_nearest_thing_within_its_slack() {
        let [near, far] = subjects();
        let marks = [
            mark(far, Kind::Rock, 8_000, 0, 150),
            mark(near, Kind::Rock, 5_000, 2_000, 150),
        ];
        let dots = marks.each_ref().map(|mark| Dot::of(mark, PROJECTION));
        let picked = |x, y| pointed_at(Pos2::new(x, y), &dots, &[], PROJECTION);

        assert_eq!(picked(200.0, 141.0), Some(near));
        assert_eq!(picked(202.0, 142.0), Some(near));
        assert_eq!(picked(205.0, 143.0), Some(far));
        assert_eq!(picked(200.0, 160.0), None);
    }

    #[test]
    fn pointer_is_on_water_inside_a_triangle_or_by_a_shore() {
        let triangle = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(0.0, 10.0),
        ];
        let reversed = [triangle[2], triangle[1], triangle[0]];

        assert!(is_inside(Pos2::new(2.0, 2.0), triangle));
        assert!(is_inside(Pos2::new(2.0, 2.0), reversed));
        assert!(!is_inside(Pos2::new(8.0, 8.0), triangle));
        assert!(!is_inside(Pos2::new(-1.0, 2.0), reversed));

        let shore = [Pos2::new(0.0, 0.0), Pos2::new(10.0, 0.0)];
        assert_eq!(distance_to_segment(Pos2::new(4.0, 3.0), shore), 3.0);
        assert_eq!(distance_to_segment(Pos2::new(14.0, 3.0), shore), 5.0);
        assert_eq!(distance_to_segment(Pos2::new(-3.0, -4.0), shore), 5.0);
    }

    #[test]
    fn map_draws_for_an_empty_world_a_forest_and_a_walk() {
        let harness = Harness::new();
        let mut reach = DEFAULT_REACH;
        let empty = Simulation::new(Name::default(), Sex::Female).view();
        let mut forest = Simulation::generate(Name::default(), Sex::Female, 1);
        let target = forest.view().map.things[0].subject;

        harness.show(|ui| assert_eq!(show(ui, &empty, &mut reach, None), Pointed::default()));
        for mut reach in [*REACH_LIMITS.start(), DEFAULT_REACH, *REACH_LIMITS.end()] {
            let view = forest.view();
            harness.show(|ui| {
                assert_eq!(
                    show(ui, &view, &mut reach, Some(target)),
                    Pointed::default()
                );
            });
        }

        forest.apply(Command::WalkTo(target));
        forest.tick();
        let walking = forest.view();
        assert!(walking.walk.is_some());
        harness.show(|ui| assert_eq!(show(ui, &walking, &mut reach, None), Pointed::default()));
        assert_eq!(reach, DEFAULT_REACH);
    }

    #[test]
    fn thing_is_described_by_what_it_is_and_how_far() {
        let view = Simulation::generate(Name::default(), Sex::Female, 1).view();
        let nearest = view
            .surroundings
            .sightings()
            .min_by_key(|sighting| sighting.distance)
            .copied()
            .expect("there is something in sight");
        let projection = Projection {
            origin: view.player.location(),
            ..PROJECTION
        };

        let described = format!("{}, {}", name(nearest.kind), distance(nearest.distance));
        let later = Timestamp::from_seconds(view.time.as_seconds() + 2 * 3_600);
        let mut remembered = (*view.map).clone();
        for mark in &mut remembered.things {
            mark.last_seen = Some(view.time);
        }

        assert_eq!(
            describe(nearest.subject, &view.map, projection, later),
            Some(described.clone())
        );
        assert_eq!(
            describe(nearest.subject, &remembered, projection, later),
            Some(format!("{described}, seen 2 hours ago"))
        );
        assert_eq!(
            describe(nearest.subject, &Map::default(), projection, later),
            None
        );
    }

    #[test]
    fn remembered_thing_fades_fast_at_first_and_never_all_the_way() {
        assert_eq!(opacity(0), FRESHEST);
        assert_eq!(opacity(3_600), (FRESHEST + FAINTEST) / 2.0);
        assert!(opacity(60) > opacity(600));
        assert!(opacity(3_600) - opacity(7_200) < opacity(0) - opacity(3_600));
        assert!(opacity(86_400 * 365) >= FAINTEST);
        assert!(opacity(86_400 * 365) < FAINTEST + 0.001);
    }

    #[test]
    fn time_since_a_thing_was_seen_is_told_in_its_largest_whole_unit() {
        assert_eq!(ago(0), "moments ago");
        assert_eq!(ago(59), "moments ago");
        assert_eq!(ago(60), "1 minute ago");
        assert_eq!(ago(3_599), "59 minutes ago");
        assert_eq!(ago(3_600), "1 hour ago");
        assert_eq!(ago(86_399), "23 hours ago");
        assert_eq!(ago(86_400), "1 day ago");
        assert_eq!(ago(86_400 * 40), "40 days ago");
    }
}
