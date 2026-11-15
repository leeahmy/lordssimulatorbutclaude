use egui::RichText;
use lords_sim::perception::Sighting;
use lords_sim::view::Walk;
use lords_sim::world::{Kind, PlantSpecies};
use lords_sim::{Bearing, Command, Length, View};

const ROW_SPACING: egui::Vec2 = egui::vec2(16.0, 6.0);
const SECTION_SPACING: f32 = 12.0;

const COLUMN_WIDTH: f32 = 90.0;

/// How many rows of a section show at most without scrolling.
const ROWS_SHOWN: f32 = 5.0;

pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let mut command = None;

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if activity(ui, view.walk) {
                    command = Some(Command::Stop);
                }

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
                        })
                        .collect();

                    if let Some(sighting) = section(ui, category, &rows) {
                        command = Some(Command::WalkTo(sighting.subject));
                    }
                }
            });
    });

    command
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Category {
    Trees,
    Rocks,
    Water,
}

impl Category {
    const ALL: [Self; 3] = [Self::Trees, Self::Rocks, Self::Water];

    fn of(kind: Kind) -> Self {
        match kind {
            Kind::Plant(_) => Self::Trees,
            Kind::Rock => Self::Rocks,
            Kind::Lake => Self::Water,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Trees => "Trees",
            Self::Rocks => "Rocks",
            Self::Water => "Water",
        }
    }
}

struct Row<'a> {
    sighting: &'a Sighting,
    within_reach: bool,
}

fn activity(ui: &mut egui::Ui, walk: Option<Walk>) -> bool {
    let mut stop = false;

    ui.horizontal(|ui| {
        ui.set_min_height(ui.spacing().interact_size.y);
        match walk {
            Some(walk) => {
                ui.label(format!(
                    "Walking to the {}, {} to go",
                    name(walk.toward).to_lowercase(),
                    distance(walk.remaining)
                ));
                stop = ui.button("Stop").clicked();
            }
            None => {
                ui.label(RichText::new("Standing still").weak());
            }
        }
    });
    ui.add_space(SECTION_SPACING);

    stop
}

fn section<'a>(ui: &mut egui::Ui, category: Category, rows: &[Row<'a>]) -> Option<&'a Sighting> {
    let mut chosen = None;
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
                        ui.label(label(sighting));
                        ui.label(RichText::new(compass_point(sighting.bearing)).weak());
                        if row.within_reach {
                            ui.label(RichText::new("within reach").weak());
                        } else {
                            ui.label(RichText::new(distance_to(sighting)).weak());
                            if ui.button("Walk to").clicked() {
                                chosen = Some(sighting);
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    ui.add_space(SECTION_SPACING);

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

fn name(kind: Kind) -> &'static str {
    match kind {
        Kind::Plant(PlantSpecies::Spruce) => "Spruce",
        Kind::Plant(PlantSpecies::Pine) => "Pine",
        Kind::Plant(PlantSpecies::Birch) => "Birch",
        Kind::Rock => "Boulder",
        Kind::Lake => "Lake",
    }
}

fn distance_to(sighting: &Sighting) -> String {
    let distance = distance(sighting.distance);
    if sighting.count > 1 {
        format!("from {distance}")
    } else {
        distance
    }
}

fn distance(length: Length) -> String {
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
    use lords_sim::world::{Area, AreaKind};
    use lords_sim::{Name, Sex, Simulation, Subject, World};

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

        harness.show(|ui| assert_eq!(show(ui, &empty), None));
        harness.show(|ui| assert_eq!(show(ui, &forest.view()), None));

        forest.apply(Command::WalkTo(target));
        forest.tick();
        let walking = forest.view();
        assert!(walking.walk.is_some());
        harness.show(|ui| assert_eq!(show(ui, &walking), None));
    }
}
