use egui::RichText;
use lords_sim::action::Lack;
use lords_sim::craft::Trade;
use lords_sim::skill::Skill;
use lords_sim::world::Kind;
use lords_sim::{Command, Item, Recipe, View};

const ROW_SPACING: f32 = 2.0;
const SECTION_SPACING: f32 = 12.0;
const NAME_WIDTH: f32 = 170.0;

/// Lists everything that can be made, with what each takes and whether it is at hand.
/// Returns what was asked to be made.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let mut command = None;

    // What takes knowing something that nobody here knows yet is left out.
    let within_grasp = |recipe: Recipe| {
        let knowledge = recipe.knowledge();
        knowledge.is_none_or(|advance| view.at_hand.advances.contains(&advance))
    };
    for trade in Trade::ALL {
        ui.label(RichText::new(trade.name()).strong());
        for recipe in Recipe::ALL {
            if recipe.trade() == trade && within_grasp(recipe) && row(ui, view, recipe) {
                command = Some(Command::Craft(recipe));
            }
        }
        ui.add_space(SECTION_SPACING);
    }
    let beyond = Recipe::ALL
        .into_iter()
        .filter(|recipe| !within_grasp(*recipe))
        .count();
    if beyond > 0 {
        ui.label(
            RichText::new(format!(
                "{beyond} more things can be made once your people have worked out how."
            ))
            .weak(),
        );
        ui.add_space(SECTION_SPACING);
    }

    command
}

/// Shows the row of `recipe`, and returns whether it was asked to be made.
fn row(ui: &mut egui::Ui, view: &View, recipe: Recipe) -> bool {
    let carried = view.player.carried();
    let lacks = view.at_hand.lacks(carried, recipe);
    let lacked = |item: Item| {
        lacks.iter().any(|lack| match lack {
            Lack::Tool(lacked) | Lack::Material(lacked, _) => *lacked == item,
            Lack::Fire | Lack::Water | Lack::Knowledge(_) | Lack::Workplace(_) => false,
        })
    };

    let mut needs: Vec<(String, bool)> = Vec::new();
    for (item, count) in recipe.materials() {
        let had = view.at_hand.count(carried, *item);
        let name = item.name().to_lowercase();
        needs.push((format!("{had}/{count} {name}"), lacked(*item)));
    }
    for tool in recipe.tools() {
        needs.push((tool.name().to_lowercase(), lacked(*tool)));
    }
    if recipe.needs_fire() {
        needs.push(("a fire".to_owned(), lacks.contains(&Lack::Fire)));
    }
    if recipe.needs_water() {
        needs.push(("the lake".to_owned(), lacks.contains(&Lack::Water)));
    }
    if let Some(workplace) = recipe.workplace() {
        needs.push((
            format!("a {}", Kind::Structure(workplace).name().to_lowercase()),
            lacks.contains(&Lack::Workplace(workplace)),
        ));
    }

    let mut made = false;
    ui.horizontal_wrapped(|ui| {
        let button = egui::Button::new(recipe.name()).min_size(egui::vec2(NAME_WIDTH, 0.0));
        made = ui
            .add_enabled(lacks.is_empty(), button)
            .on_disabled_hover_text(missing(&lacks))
            .clicked();

        let level = view.player.skills().level(recipe.skill());
        ui.label(RichText::new(duration(recipe.seconds())).weak())
            .on_hover_text(format!(
                "How long it takes someone unskilled. Your {}.",
                skill_label(recipe.skill(), level)
            ));

        let last = needs.len().saturating_sub(1);
        for (index, (need, lacking)) in needs.into_iter().enumerate() {
            let need = if index < last {
                format!("{need},")
            } else {
                need
            };
            if lacking {
                ui.label(RichText::new(need).color(ui.visuals().warn_fg_color));
            } else {
                ui.label(RichText::new(need).weak());
            }
        }
    });
    ui.add_space(ROW_SPACING);

    made
}

fn missing(lacks: &[Lack]) -> String {
    let lacks: Vec<String> = lacks.iter().map(|lack| lack.describe()).collect();
    format!("Missing: {}", lacks.join(", "))
}

fn skill_label(skill: Skill, level: u32) -> String {
    format!("{} is at level {level}", skill.name().to_lowercase())
}

/// Returns the words for something taking `seconds` in-game seconds.
fn duration(seconds: u32) -> String {
    let minutes = seconds / 60;
    if minutes < 60 {
        format!("{minutes} min")
    } else if minutes.is_multiple_of(60) {
        format!("{} h", minutes / 60)
    } else {
        format!("{} h {} min", minutes / 60, minutes % 60)
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::Harness;

    #[test]
    fn duration_is_told_in_minutes_and_hours() {
        assert_eq!(duration(120), "2 min");
        assert_eq!(duration(2_700), "45 min");
        assert_eq!(duration(7_200), "2 h");
        assert_eq!(duration(5_400), "1 h 30 min");
    }

    #[test]
    fn what_is_missing_is_listed_for_the_hover_text() {
        let lacks = [
            Lack::Tool(Item::Knife),
            Lack::Material(Item::Stick, 2),
            Lack::Fire,
        ];

        assert_eq!(
            missing(&lacks),
            "Missing: knife, 2 × stick, a burning fire within reach"
        );
    }

    #[test]
    fn every_recipe_is_in_a_listed_trade() {
        for recipe in Recipe::ALL {
            assert!(Trade::ALL.contains(&recipe.trade()), "{recipe:?}");
        }
    }

    #[test]
    fn list_draws_with_nothing_and_with_a_starting_outfit() {
        let harness = Harness::new();
        let empty = Simulation::new(Name::default(), Sex::Female).view();
        let outfitted = Simulation::generate(Name::default(), Sex::Female, 1).view();

        harness.show(|ui| assert_eq!(show(ui, &empty), None));
        harness.show(|ui| assert_eq!(show(ui, &outfitted), None));
    }
}
