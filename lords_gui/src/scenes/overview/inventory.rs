use egui::RichText;
use lords_sim::item::{CARRY_LIMIT, HEAVY_LOAD, Stack};
use lords_sim::time::gregorian::{SECONDS_PER_DAY, SECONDS_PER_HOUR};
use lords_sim::{Command, View};

use crate::scenes::count;

const ROW_SPACING: egui::Vec2 = egui::vec2(16.0, 6.0);
const SECTION_SPACING: f32 = 12.0;
const COLUMN_WIDTH: f32 = 90.0;

/// Lists what the player character carries and what lies within their reach, with the
/// means to eat, drink, and put things down. Returns what was asked for.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let mut command = None;
    let carried = view.player.carried();

    ui.horizontal(|ui| {
        ui.label(RichText::new("Carried").strong());
        let load = RichText::new(format!(
            "{} of {}",
            weight(carried.weight()),
            weight(CARRY_LIMIT)
        ));
        if carried.weight() > HEAVY_LOAD {
            ui.label(load.color(ui.visuals().warn_fg_color))
                .on_hover_text("A load this heavy slows you down");
        } else {
            ui.label(load.weak());
        }
    });

    if carried.is_empty() {
        ui.label(RichText::new("Nothing").weak());
    } else {
        egui::Grid::new("carried")
            .num_columns(4)
            .min_col_width(COLUMN_WIDTH)
            .min_row_height(ui.spacing().interact_size.y)
            .spacing(ROW_SPACING)
            .show(ui, |ui| {
                for stack in carried.stacks() {
                    stack_cells(ui, stack);
                    ui.horizontal(|ui| {
                        command = command.take().or(stack_buttons(ui, stack));
                    });
                    ui.end_row();
                }
            });
    }
    ui.add_space(SECTION_SPACING);

    let loose = &view.at_hand.loose;
    if !loose.is_empty() {
        ui.label(RichText::new("On the ground within reach").strong());
        egui::Grid::new("loose")
            .num_columns(3)
            .min_col_width(COLUMN_WIDTH)
            .min_row_height(ui.spacing().interact_size.y)
            .spacing(ROW_SPACING)
            .show(ui, |ui| {
                for stack in loose.stacks() {
                    stack_cells(ui, stack);
                    ui.end_row();
                }
            });
        ui.add_space(SECTION_SPACING);
    }

    command
}

fn stack_cells(ui: &mut egui::Ui, stack: &Stack) {
    let name = stack.item.name();
    let label = if stack.count > 1 {
        format!("{name} ×{}", stack.count)
    } else {
        name.to_owned()
    };

    ui.label(label).on_hover_text(description(stack));
    ui.label(RichText::new(weight(stack.weight())).weak());
    ui.label(RichText::new(keeping(stack)).weak());
}

fn stack_buttons(ui: &mut egui::Ui, stack: &Stack) -> Option<Command> {
    let item = stack.item;
    let mut command = None;

    if item.food().is_some() {
        let verb = if item.is_drink() { "Drink" } else { "Eat" };
        if ui.button(verb).clicked() {
            command = Some(Command::Consume(item));
        }
    }
    if ui.button("Drop").on_hover_text("Put one down").clicked() {
        command = Some(Command::Drop(item, 1));
    }
    if stack.count > 1 && ui.button("Drop all").clicked() {
        command = Some(Command::Drop(item, stack.count));
    }

    command
}

/// Returns the words for what one of the stack's item is good for.
fn description(stack: &Stack) -> String {
    let item = stack.item;
    let mut facts = Vec::new();

    if let Some(food) = item.food() {
        if food.energy > 0 {
            facts.push(format!("{} kcal", food.energy));
        }
        if food.water > 0 {
            facts.push(format!("{} ml of water", food.water));
        }
        if food.risk > 0 {
            facts.push("may make you sick".to_owned());
        }
    }
    if item.warmth() > 0 {
        facts.push(format!("worn, {} °C warmer", item.warmth() / 10));
    }
    if item.burn_time() > 0 {
        let minutes = u64::from(item.burn_time()) / 60;
        facts.push(format!("burns for {}", count(minutes, "minute")));
    }
    facts.push(format!("{} each", weight(item.weight())));

    facts.join(", ")
}

/// Returns the words for how long the stack keeps.
fn keeping(stack: &Stack) -> String {
    if stack.item.shelf_life().is_none() {
        return String::new();
    }

    let seconds = u64::from(stack.fresh);
    if seconds >= SECONDS_PER_DAY {
        format!("keeps {}", count(seconds / SECONDS_PER_DAY, "day"))
    } else {
        format!(
            "keeps {}",
            count((seconds / SECONDS_PER_HOUR).max(1), "hour")
        )
    }
}

/// Returns the words for a weight of `grams`.
pub(super) fn weight(grams: u32) -> String {
    if grams < 1_000 {
        format!("{grams} g")
    } else {
        format!("{}.{} kg", grams / 1_000, grams % 1_000 / 100)
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::{Item, Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::Harness;

    #[test]
    fn weight_is_told_in_grams_until_it_comes_to_kilograms() {
        assert_eq!(weight(0), "0 g");
        assert_eq!(weight(950), "950 g");
        assert_eq!(weight(1_000), "1.0 kg");
        assert_eq!(weight(15_480), "15.4 kg");
    }

    #[test]
    fn only_food_is_said_to_keep_for_a_time() {
        let mut meat = Stack::new(Item::RawMeat, 2);
        assert_eq!(keeping(&meat), "keeps 2 days");

        meat.fresh = 5 * 3_600 + 12;
        assert_eq!(keeping(&meat), "keeps 5 hours");
        meat.fresh = 40;
        assert_eq!(keeping(&meat), "keeps 1 hour");

        assert_eq!(keeping(&Stack::new(Item::Stone, 1)), "");
        assert_eq!(keeping(&Stack::new(Item::DriedMeat, 1)), "");
    }

    #[test]
    fn item_is_described_by_what_it_is_good_for() {
        let described = |item| description(&Stack::new(item, 1));

        assert_eq!(
            described(Item::RawMeat),
            "450 kcal, 100 ml of water, may make you sick, 400 g each"
        );
        assert_eq!(described(Item::FurCoat), "worn, 9 °C warmer, 2.0 kg each");
        assert_eq!(described(Item::Stick), "burns for 15 minutes, 400 g each");
    }

    #[test]
    fn inventory_draws_empty_and_with_things_carried_and_put_down() {
        let harness = Harness::new();
        let empty = Simulation::new(Name::default(), Sex::Female).view();
        harness.show(|ui| assert_eq!(show(ui, &empty), None));

        let mut sim = Simulation::generate(Name::default(), Sex::Female, 1);
        sim.apply(Command::Drop(Item::RyeBread, 2));
        let view = sim.view();
        assert!(!view.at_hand.loose.is_empty());
        harness.show(|ui| assert_eq!(show(ui, &view), None));
    }
}
