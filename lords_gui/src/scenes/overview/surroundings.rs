use egui::RichText;
use lords_sim::perception::Sighting;
use lords_sim::world::Kind;
use lords_sim::{Bearing, Command, Length, Subject, Verb, View};

use super::Pointed;

const ROW_SPACING: egui::Vec2 = egui::vec2(12.0, 6.0);
const SECTION_SPACING: f32 = 12.0;

const COLUMN_WIDTH: f32 = 56.0;

/// How many rows of a section show at most without scrolling.
const ROWS_SHOWN: f32 = 5.0;

/// Lists what the player perceives around them, with what there is to do to each.
pub fn show(ui: &mut egui::Ui, view: &View, highlighted: Option<Subject>) -> Pointed {
    let mut pointed = Pointed::default();
    let highlighted = highlighted.and_then(|subject| row_of(view, subject));

    let surroundings = &view.surroundings;
    for category in Category::ALL {
        let reachable = surroundings.within_reach.iter();
        let farther = surroundings.nearby.iter().chain(&surroundings.in_sight);
        let rows: Vec<Row<'_>> = reachable
            .map(|sighting| (sighting, true))
            .chain(farther.map(|sighting| (sighting, false)))
            .filter(|(sighting, _)| Category::of(sighting.kind) == category)
            .map(|(sighting, within_reach)| Row {
                sighting,
                within_reach,
                highlighted: highlighted == Some(sighting.subject),
            })
            .collect();
        if rows.is_empty() && !category.is_always_listed() {
            continue;
        }

        let section = section(ui, category, &rows);
        pointed.command = pointed.command.or(section.chosen);
        pointed.hovered = pointed.hovered.or(section.hovered);
    }

    pointed
}

fn row_of(view: &View, subject: Subject) -> Option<Subject> {
    let surroundings = &view.surroundings;
    if surroundings.sighting(subject).is_some() {
        return Some(subject);
    }

    let mark = view
        .map
        .things
        .iter()
        .find(|mark| mark.subject == subject)?;
    surroundings
        .in_sight
        .iter()
        .find(|sighting| mark.in_sight() && sighting.kind == mark.kind)
        .map(|sighting| sighting.subject)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Category {
    Camp,
    Ground,
    Trees,
    Berries,
    Rocks,
    Water,
}

impl Category {
    const ALL: [Self; 6] = [
        Self::Camp,
        Self::Ground,
        Self::Trees,
        Self::Berries,
        Self::Rocks,
        Self::Water,
    ];

    fn of(kind: Kind) -> Self {
        match kind {
            Kind::Plant(species) if species.is_tree() => Self::Trees,
            Kind::Plant(_) => Self::Berries,
            Kind::Rock => Self::Rocks,
            Kind::Lake => Self::Water,
            Kind::Pile(_) => Self::Ground,
            Kind::Structure(_) => Self::Camp,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Camp => "Camp",
            Self::Ground => "On the ground",
            Self::Trees => "Trees",
            Self::Berries => "Berries",
            Self::Rocks => "Rocks",
            Self::Water => "Water",
        }
    }

    /// Returns whether the category is listed even when nothing of it is in sight.
    fn is_always_listed(self) -> bool {
        matches!(self, Self::Trees | Self::Rocks | Self::Water)
    }
}

struct Row<'a> {
    sighting: &'a Sighting,
    within_reach: bool,
    highlighted: bool,
}

#[derive(Default)]
struct Section {
    chosen: Option<Command>,
    hovered: Option<Subject>,
}

fn section(ui: &mut egui::Ui, category: Category, rows: &[Row<'_>]) -> Section {
    let mut section = Section::default();
    let seen: u32 = rows.iter().map(|row| row.sighting.count).sum();
    let row_height = ui.spacing().interact_size.y + ROW_SPACING.y;

    ui.horizontal(|ui| {
        ui.label(RichText::new(category.title()).strong());
        ui.label(RichText::new(seen.to_string()).weak());
    });
    egui::ScrollArea::vertical()
        .id_salt(category.title())
        .max_height(ROWS_SHOWN * row_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            if rows.is_empty() {
                ui.label(RichText::new("None in sight").weak());
                return;
            }

            egui::Grid::new(category.title())
                .num_columns(4)
                .min_col_width(COLUMN_WIDTH)
                .min_row_height(ui.spacing().interact_size.y)
                .spacing(ROW_SPACING)
                .show(ui, |ui| {
                    for row in rows {
                        let sighting = row.sighting;
                        let label = RichText::new(label(sighting));
                        let mut shown = ui.label(if row.highlighted {
                            label.strong()
                        } else {
                            label
                        });
                        shown |= ui.label(RichText::new(compass_point(sighting.bearing)).weak());
                        shown |= if row.within_reach {
                            ui.label(RichText::new("within reach").weak())
                        } else {
                            ui.label(RichText::new(distance_to(sighting)).weak())
                        };
                        let buttons = ui.horizontal(|ui| actions(ui, row));
                        shown |= buttons.response;
                        section.chosen = section.chosen.take().or(buttons.inner);

                        if ui.rect_contains_pointer(shown.rect) {
                            section.hovered = Some(sighting.subject);
                        }
                        ui.end_row();
                    }
                });
        });
    ui.add_space(SECTION_SPACING);

    section
}

/// Shows the buttons for what there is to do to what `row` is of, and returns what was
/// chosen to be done.
fn actions(ui: &mut egui::Ui, row: &Row<'_>) -> Option<Command> {
    let Sighting { subject, kind, .. } = *row.sighting;
    let mut chosen = None;

    if !row.within_reach && ui.button("Walk to").clicked() {
        chosen = Some(Command::WalkTo(subject));
    }
    match Verb::for_kind(kind) {
        [] => {}
        [verb] => {
            if ui.button(verb.name(kind)).clicked() {
                chosen = Some(Command::Act {
                    verb: *verb,
                    subject,
                });
            }
        }
        verbs => {
            ui.menu_button("Do…", |ui| {
                for verb in verbs {
                    if ui.button(verb.name(kind)).clicked() {
                        chosen = Some(Command::Act {
                            verb: *verb,
                            subject,
                        });
                        ui.close();
                    }
                }
            });
        }
    }

    chosen
}

fn label(sighting: &Sighting) -> String {
    let name = name(sighting.kind);
    if sighting.count > 1 {
        format!("{name} ×{}", sighting.count)
    } else {
        name.to_owned()
    }
}

pub(super) fn name(kind: Kind) -> &'static str {
    kind.name()
}

fn distance_to(sighting: &Sighting) -> String {
    let distance = distance(sighting.distance);
    if sighting.count > 1 {
        format!("from {distance}")
    } else {
        distance
    }
}

pub(super) fn distance(length: Length) -> String {
    let millimeters = length.as_millimeters();
    if millimeters < 1_000 {
        return "under 1 m".to_owned();
    }

    let precision = if millimeters < 10_000 {
        1_000
    } else if millimeters < 100_000 {
        5_000
    } else {
        10_000
    };
    let rounded = (millimeters + precision / 2) / precision * precision;
    format!("{} m", rounded / 1_000)
}

fn compass_point(bearing: Bearing) -> &'static str {
    match bearing {
        Bearing::North => "N",
        Bearing::Northeast => "NE",
        Bearing::East => "E",
        Bearing::Southeast => "SE",
        Bearing::South => "S",
        Bearing::Southwest => "SW",
        Bearing::West => "W",
        Bearing::Northwest => "NW",
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::world::{Area, AreaKind, PlantSpecies};
    use lords_sim::{Name, Sex, Simulation, World};

    use super::*;
    use crate::scenes::harness::Harness;

    fn sighting(kind: Kind, count: u32, millimeters: i64) -> Sighting {
        let anything = World::default().add_area(Area::new(AreaKind::Lake, Vec::new()));

        Sighting {
            subject: Subject::Area(anything),
            kind,
            count,
            distance: Length::from_millimeters(millimeters),
            bearing: Bearing::North,
        }
    }

    #[test]
    fn distance_is_rounded_more_roughly_the_farther_it_is() {
        let worded = |millimeters| distance(Length::from_millimeters(millimeters));

        assert_eq!(worded(0), "under 1 m");
        assert_eq!(worded(999), "under 1 m");
        assert_eq!(worded(1_000), "1 m");
        assert_eq!(worded(3_600), "4 m");
        assert_eq!(worded(9_400), "9 m");
        assert_eq!(worded(12_000), "10 m");
        assert_eq!(worded(13_000), "15 m");
        assert_eq!(worded(97_000), "95 m");
        assert_eq!(worded(104_000), "100 m");
        assert_eq!(worded(138_000), "140 m");
    }

    #[test]
    fn sighting_of_several_says_how_many_and_how_near() {
        let one = sighting(Kind::Rock, 1, 4_000);
        let many = sighting(Kind::Plant(PlantSpecies::Spruce), 41, 23_000);

        assert_eq!(label(&one), "Boulder");
        assert_eq!(distance_to(&one), "4 m");
        assert_eq!(label(&many), "Spruce ×41");
        assert_eq!(distance_to(&many), "from 25 m");
    }

    #[test]
    fn list_draws_for_an_empty_world_a_forest_and_a_walk() {
        let harness = Harness::new();
        let empty = Simulation::new(Name::default(), Sex::Female).view();
        let mut forest = Simulation::generate(Name::default(), Sex::Female, 1);
        let target = forest
            .view()
            .surroundings
            .in_sight
            .first()
            .expect("there is something in sight")
            .subject;

        harness.show(|ui| assert_eq!(show(ui, &empty, None), Pointed::default()));
        harness.show(|ui| {
            assert_eq!(show(ui, &forest.view(), Some(target)), Pointed::default());
        });

        forest.apply(Command::WalkTo(target));
        forest.tick();
        let walking = forest.view();
        assert!(walking.walk.is_some());
        harness.show(|ui| assert_eq!(show(ui, &walking, None), Pointed::default()));
    }

    #[test]
    fn row_of_something_far_away_is_that_of_the_nearest_of_its_kind() {
        let view = Simulation::generate(Name::default(), Sex::Female, 1).view();
        let group = view
            .surroundings
            .in_sight
            .iter()
            .find(|sighting| sighting.count > 1)
            .expect("there are several of some kind in sight");
        let another = view
            .map
            .things
            .iter()
            .filter(|mark| mark.kind == group.kind)
            .find(|mark| view.surroundings.sighting(mark.subject).is_none())
            .expect("the group has another in it");
        let nearby = view.surroundings.sightings().next().expect("the first");

        assert_eq!(row_of(&view, group.subject), Some(group.subject));
        assert_eq!(row_of(&view, another.subject), Some(group.subject));
        assert_eq!(row_of(&view, nearby.subject), Some(nearby.subject));
    }
}
