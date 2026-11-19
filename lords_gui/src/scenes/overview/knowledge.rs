use egui::RichText;
use lords_sim::settlement::{Advance, Age};
use lords_sim::view::SettlementView;
use lords_sim::{Command, View};

const SECTION_SPACING: f32 = 12.0;
const BAR_WIDTH: f32 = 160.0;
const NAME_WIDTH: f32 = 170.0;

/// Shows everything a settlement can work out, age by age: what is known, what is being
/// worked out, what could be, and what is still out of reach. Returns what was asked for.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let Some(town) = &view.settlement else {
        ui.label(RichText::new("No settlement").strong());
        ui.label(
            RichText::new(
                "One person alone works out nothing new. It takes a settlement, and people \
                 in it who are spared from finding food to think.",
            )
            .weak(),
        );
        return None;
    };
    let leads = town.politics.leads;
    let mut command = None;

    ui.label(
        RichText::new(format!("{} is in the {}", town.name, town.age.name()))
            .heading()
            .strong(),
    );
    ui.label(
        RichText::new(
            "Nothing here is worked out by hand. Thinkers and scholars do it, kept by the \
             settlement or hired by you, and they get further with a workshop, with writing, \
             with schools, and with books. A little comes to any band of people by itself.",
        )
        .weak(),
    );
    ui.add_space(4.0);
    match town.pursuit {
        Some(pursuit) => {
            ui.horizontal(|ui| {
                ui.label(format!("Working out {}", pursuit.name().to_lowercase()))
                    .on_hover_text(pursuit.use_of());
                ui.add(
                    egui::ProgressBar::new(town.insight as f32 / pursuit.cost() as f32)
                        .desired_width(BAR_WIDTH)
                        .text(progress(town.insight, pursuit.cost())),
                );
            });
        }
        None => {
            ui.label(
                RichText::new("Nothing is being worked out.").color(ui.visuals().warn_fg_color),
            );
        }
    }
    if !leads {
        ui.label(
            RichText::new(
                "Whoever leads says what is worked out. Anyone may say so while nothing is.",
            )
            .weak(),
        );
    }
    ui.add_space(SECTION_SPACING);

    for age in Age::ALL {
        let of_age: Vec<Advance> = Advance::ALL
            .into_iter()
            .filter(|advance| advance.age() == age)
            .collect();
        let known = of_age
            .iter()
            .filter(|advance| town.advances.contains(advance))
            .count();
        // Ages nothing of which is within reach yet are only named.
        let within_sight = of_age
            .iter()
            .any(|advance| town.advances.contains(advance) || standing(town, *advance).is_none());

        ui.horizontal(|ui| {
            ui.label(RichText::new(age.name()).strong());
            ui.label(RichText::new(format!("{known} of {}", of_age.len())).weak());
        });
        if !within_sight {
            ui.label(RichText::new("Too far off to make out yet.").weak());
            ui.add_space(SECTION_SPACING);
            continue;
        }
        for advance in of_age {
            if row(ui, town, advance, leads) {
                command = Some(Command::Pursue(advance));
            }
        }
        ui.add_space(SECTION_SPACING);
    }

    command
}

/// Shows the row of `advance`, and returns whether it was asked to be worked out.
fn row(ui: &mut egui::Ui, town: &SettlementView, advance: Advance, leads: bool) -> bool {
    let known = town.advances.contains(&advance);
    let pursued = town.pursuit == Some(advance);
    let wanting = standing(town, advance);
    let mut asked = false;

    ui.horizontal(|ui| {
        let name_size = egui::vec2(NAME_WIDTH, ui.spacing().interact_size.y);
        if known {
            ui.allocate_ui_with_layout(
                name_size,
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| ui.label(RichText::new(advance.name()).strong()),
            );
            ui.add(egui::Label::new(RichText::new(advance.use_of()).weak()).wrap());
            return;
        }

        let may = wanting.is_none() && (leads || town.pursuit.is_none());
        let button = egui::Button::new(advance.name())
            .selected(pursued)
            .min_size(egui::vec2(NAME_WIDTH, 0.0));
        let why_not = match &wanting {
            Some(wanting) => wanting.clone(),
            None => "Only whoever leads changes what is being worked out".to_owned(),
        };
        asked = ui
            .add_enabled(may, button)
            .on_hover_text(advance.use_of())
            .on_disabled_hover_text(why_not)
            .clicked();

        let words = match wanting {
            Some(wanting) => format!("{wanting} {}.", advance.use_of()),
            None => format!("{} {}.", effort(advance.cost()), advance.use_of()),
        };
        ui.add(egui::Label::new(RichText::new(words).weak()).wrap());
    });

    asked
}

/// Returns what has to be known before `advance` can be worked out, in words, unless it
/// can be worked out as things are.
fn standing(town: &SettlementView, advance: Advance) -> Option<String> {
    let lacking: Vec<&str> = advance
        .requires()
        .iter()
        .filter(|needed| !town.advances.contains(needed))
        .map(|needed| needed.name())
        .collect();

    (!lacking.is_empty()).then(|| format!("Takes {} first.", lacking.join(" and ").to_lowercase()))
}

/// Returns the words for how far a pursuit has got.
fn progress(insight: u32, cost: u32) -> String {
    format!("{} of {}", thousands(insight), thousands(cost))
}

/// Returns the words for how much thinking an advance takes.
fn effort(cost: u32) -> String {
    let years = cost / 365;
    if years >= 2 {
        format!("{} years of one thinker's work.", thousands(years))
    } else {
        format!("{} days of one thinker's work.", thousands(cost))
    }
}

/// Returns `number` with its thousands set apart.
pub(super) fn thousands(number: u32) -> String {
    let digits = number.to_string();
    let mut words = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            words.push(',');
        }
        words.push(digit);
    }
    words
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::Harness;
    use crate::scenes::overview::town::tests::settlement;

    #[test]
    fn numbers_are_told_with_their_thousands_set_apart() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert_eq!(progress(1_500, 30_000), "1,500 of 30,000");
        assert_eq!(effort(225), "225 days of one thinker's work.");
        assert_eq!(effort(8_000), "21 years of one thinker's work.");
    }

    #[test]
    fn what_stands_in_the_way_of_an_advance_is_named() {
        let town = settlement(true);

        assert_eq!(standing(&town, Advance::Farming), None);
        assert_eq!(standing(&town, Advance::Ironworking), None);
        assert_eq!(
            standing(&town, Advance::Guilds),
            Some("Takes written law first.".to_owned())
        );
        assert_eq!(
            standing(&town, Advance::Navigation),
            Some("Takes sailing and optics first.".to_owned())
        );
    }

    #[test]
    fn tab_draws_alone_and_for_a_settlement_led_or_not() {
        let harness = Harness::new();
        let alone = Simulation::new(Name::default(), Sex::Female).view();
        harness.show(|ui| assert_eq!(show(ui, &alone), None));

        for leads in [false, true] {
            let mut town = Simulation::generate(Name::default(), Sex::Female, 1).view();
            town.settlement = Some(std::sync::Arc::new(settlement(leads)));
            harness.show(|ui| assert_eq!(show(ui, &town), None));
        }
    }
}
