use egui::RichText;
use lords_sim::View;
use lords_sim::skill::{Skill, TOP_LEVEL};

const ROW_SPACING: egui::Vec2 = egui::vec2(16.0, 6.0);
const BAR_WIDTH: f32 = 160.0;

/// Lists how good the player character is at each skill, and how far along to getting
/// better.
pub fn show(ui: &mut egui::Ui, view: &View) {
    let skills = view.player.skills();

    ui.label(RichText::new("Skills").strong());
    egui::Grid::new("skills")
        .num_columns(3)
        .min_row_height(ui.spacing().interact_size.y)
        .spacing(ROW_SPACING)
        .show(ui, |ui| {
            for skill in Skill::ALL {
                let level = skills.level(skill);

                ui.label(skill.name()).on_hover_text(use_of(skill));
                ui.label(RichText::new(format!("Level {level}")).weak());
                if level < TOP_LEVEL {
                    let share = skills.progress_percent(skill) as f32 / 100.0;
                    ui.add(egui::ProgressBar::new(share).desired_width(BAR_WIDTH));
                } else {
                    ui.label(RichText::new("Mastered").weak());
                }
                ui.end_row();
            }
        });
    ui.add_space(12.0);
    ui.label(
        RichText::new("Skills grow by use. Each level makes the work a little faster.").weak(),
    );
}

fn use_of(skill: Skill) -> &'static str {
    match skill {
        Skill::Foraging => "Gathering sticks, boughs, bark, roots, and berries",
        Skill::Woodcraft => "Felling trees",
        Skill::Crafting => "Making tools and clothes",
        Skill::Firemaking => "Lighting fires, which also catch more surely",
        Skill::Fishing => "Angling and fish traps, which also catch more",
        Skill::Trapping => "Snares, which also catch more, and butchering",
        Skill::Cooking => "Cooking, boiling water, and drying food",
        Skill::Building => "Building shelters and camp furniture",
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::Harness;

    #[test]
    fn skills_draw_and_each_says_what_it_is_for() {
        let harness = Harness::new();
        let view = Simulation::generate(Name::default(), Sex::Female, 1).view();

        harness.show(|ui| show(ui, &view));
        assert!(Skill::ALL.iter().all(|skill| !use_of(*skill).is_empty()));
    }
}
