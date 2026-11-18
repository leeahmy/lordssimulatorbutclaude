use egui::RichText;
use lords_sim::view::{Walk, Work};
use lords_sim::{Command, Job, View};

use super::surroundings::{distance, name};

const BAR_WIDTH: f32 = 160.0;
const SPACING: f32 = 12.0;

/// Shows what the player character is doing, with the means to stop it, or to rest when
/// they are doing nothing. Returns what was asked for.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let mut command = None;

    ui.horizontal_wrapped(|ui| {
        ui.set_min_height(ui.spacing().interact_size.y);

        if let Some(walk) = &view.walk {
            ui.label(walking(walk));
            if ui.button("Stop").clicked() {
                command = Some(Command::Stop);
            }
        } else if let Some(work) = &view.work {
            ui.label(working(work));
            let share = work.done as f32 / work.needed.max(1) as f32;
            ui.add(
                egui::ProgressBar::new(share)
                    .desired_width(BAR_WIDTH)
                    .text(time_left(work.needed.saturating_sub(work.done))),
            );
            if ui.button("Stop").clicked() {
                command = Some(Command::Stop);
            }
        } else if view.sleeping {
            ui.label("Sleeping");
            if ui.button("Wake up").clicked() {
                command = Some(Command::Stop);
            }
        } else {
            ui.label(RichText::new("Standing still").weak());
            if ui
                .button("Sleep")
                .on_hover_text("Lie down to sleep where you are")
                .clicked()
            {
                command = Some(Command::Sleep);
            }
            if view.conditions.snow
                && ui
                    .button("Eat snow")
                    .on_hover_text("Quenches thirst a little, and chills")
                    .clicked()
            {
                command = Some(Command::EatSnow);
            }
        }
    });
    ui.add_space(SPACING);

    command
}

fn walking(walk: &Walk) -> String {
    let toward = name(walk.toward).to_lowercase();
    let to_go = distance(walk.remaining);

    match walk.intent {
        Some(job) => format!(
            "Walking to the {toward}, {to_go} to go, for {}",
            doing(job).to_lowercase()
        ),
        None => format!("Walking to the {toward}, {to_go} to go"),
    }
}

fn working(work: &Work) -> String {
    doing(work.job)
}

/// Returns the words for being at `job`.
fn doing(job: Job) -> String {
    match job {
        Job::Act { verb, .. } => verb.doing().to_owned(),
        Job::Craft(recipe) => format!("Working: {}", recipe.name().to_lowercase()),
        Job::Consume(item) if item.is_drink() => {
            format!("Drinking {}", item.name().to_lowercase())
        }
        Job::Consume(item) => format!("Eating {}", item.name().to_lowercase()),
        Job::EatSnow => "Eating snow".to_owned(),
    }
}

/// Returns the words for `seconds` in-game seconds left of something.
fn time_left(seconds: u32) -> String {
    let minutes = seconds.div_ceil(60);
    if seconds < 60 {
        format!("{seconds} s")
    } else if minutes < 60 {
        format!("{minutes} min")
    } else {
        format!("{} h {} min", minutes / 60, minutes % 60)
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::world::{Kind, PlantSpecies};
    use lords_sim::{Item, Length, Location, Name, Recipe, Sex, Simulation, Verb};

    use super::*;
    use crate::scenes::harness::Harness;

    #[test]
    fn time_left_is_told_in_its_largest_units() {
        assert_eq!(time_left(0), "0 s");
        assert_eq!(time_left(59), "59 s");
        assert_eq!(time_left(60), "1 min");
        assert_eq!(time_left(61), "2 min");
        assert_eq!(time_left(3_600), "1 h 0 min");
        assert_eq!(time_left(50_000), "13 h 54 min");
    }

    #[test]
    fn walk_is_worded_with_what_it_is_for() {
        let forest = Simulation::generate(Name::default(), Sex::Female, 1);
        let subject = forest.view().map.things[0].subject;
        let walk = |intent| Walk {
            toward: Kind::Plant(PlantSpecies::Spruce),
            destination: Location::default(),
            via: Vec::new(),
            remaining: Length::from_meters(12),
            intent,
        };
        let fell = Job::Act {
            verb: Verb::Fell,
            subject,
        };

        assert_eq!(walking(&walk(None)), "Walking to the spruce, 10 m to go");
        assert_eq!(
            walking(&walk(Some(fell))),
            "Walking to the spruce, 10 m to go, for felling a tree"
        );
    }

    #[test]
    fn jobs_are_worded_by_what_they_are() {
        assert_eq!(doing(Job::Craft(Recipe::Cordage)), "Working: twist cordage");
        assert_eq!(doing(Job::Consume(Item::RyeBread)), "Eating rye bread");
        assert_eq!(
            doing(Job::Consume(Item::BoiledWater)),
            "Drinking boiled water"
        );
        assert_eq!(doing(Job::EatSnow), "Eating snow");
    }

    #[test]
    fn bar_draws_for_standing_walking_working_and_sleeping() {
        let harness = Harness::new();
        let mut sim = Simulation::generate(Name::default(), Sex::Female, 1);
        let draw = |sim: &Simulation| {
            let view = sim.view();
            harness.show(|ui| assert_eq!(show(ui, &view), None));
        };
        draw(&sim);

        let spruce = sim
            .view()
            .map
            .things
            .iter()
            .find(|mark| mark.kind == Kind::Plant(PlantSpecies::Spruce))
            .map(|mark| mark.subject)
            .expect("a spruce is in sight");
        sim.apply(Command::Act {
            verb: Verb::DigRoots,
            subject: spruce,
        });
        assert!(sim.view().walk.is_some());
        draw(&sim);

        while sim.view().work.is_none() {
            sim.tick();
        }
        draw(&sim);

        sim.apply(Command::Stop);
        for _ in 0..16 * 3_600 {
            sim.tick();
        }
        sim.apply(Command::Sleep);
        assert!(sim.view().sleeping);
        draw(&sim);
    }
}
