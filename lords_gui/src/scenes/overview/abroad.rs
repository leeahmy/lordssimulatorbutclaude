use egui::RichText;
use lords_sim::neighbour::{Mission, Neighbour};
use lords_sim::time::gregorian::SECONDS_PER_DAY;
use lords_sim::{Command, View};

use crate::scenes::count;

const SECTION_SPACING: f32 = 12.0;

/// Shows the other settlements of the wider world, how each stands towards the player's
/// own, and the means to send envoys to them. Returns what was asked for.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let mut command = None;
    let today = view.time.as_seconds() / SECONDS_PER_DAY;
    let town = view.settlement.as_ref();
    let leads = town.is_some_and(|town| town.politics.leads);

    ui.label(RichText::new("The wider world").heading().strong());
    ui.label(
        RichText::new(
            "Other settlements lie days away through the forest: too far to walk to, but \
             near enough to trade with, to learn from, and to be raided by. They grow and \
             learn as yours does. Grudges and goodwill both fade with the years, unless \
             there is a pact.",
        )
        .weak(),
    );
    ui.add_space(4.0);
    match town {
        Some(town) => {
            ui.label(defence(town.defence, &view.neighbours))
                .on_hover_text(
                    "Each guard with a place in a barracks counts for one, and the bold for \
                     two. A palisade is worth six and a town wall twenty. Raiders come late \
                     in July, and only to places of twenty-five people or more.",
                );
            if !leads {
                ui.label(RichText::new("Only whoever leads sends envoys.").weak());
            }
        }
        None => {
            ui.label(
                RichText::new("With no settlement of your own, none of them has heard of you.")
                    .weak(),
            );
        }
    }
    ui.add_space(SECTION_SPACING);

    if view.neighbours.is_empty() {
        ui.label(RichText::new("Nobody has brought word of any other place yet.").weak());
    }
    for (index, neighbour) in view.neighbours.iter().enumerate() {
        ui.label(RichText::new(&neighbour.name).strong());
        ui.label(describe(neighbour));
        ui.label(RichText::new(standing(neighbour)).weak());

        let rest = neighbour.rest(today);
        ui.horizontal_wrapped(|ui| {
            for mission in Mission::ALL {
                let label = if rest > 0 {
                    format!("{} (in {})", mission.name(), count(rest, "day"))
                } else {
                    mission.name().to_owned()
                };
                let button = ui
                    .add_enabled(leads && rest == 0, egui::Button::new(label))
                    .on_hover_text(mission.telling())
                    .on_disabled_hover_text(mission.telling());
                if button.clicked() {
                    command = Some(Command::Envoy(index, mission));
                }
            }
        });
        ui.add_space(SECTION_SPACING);
    }

    command
}

/// Returns the words for what a neighbour is and where it lies.
fn describe(neighbour: &Neighbour) -> String {
    format!(
        "{} to the {}, about {} people, in the {}.",
        count(u64::from(neighbour.days_away), "day"),
        neighbour.way(),
        neighbour.people,
        neighbour.age().name()
    )
}

/// Returns the words for how a neighbour stands towards the player's settlement.
fn standing(neighbour: &Neighbour) -> String {
    let mut words = format!(
        "They are {} towards you ({}).",
        neighbour.stance(),
        neighbour.regard
    );
    if neighbour.pact {
        words.push_str(" There is a pact of trade.");
    }
    if neighbour.is_hostile() {
        words.push_str(&format!(" They can send {} raiders.", neighbour.raiders()));
    }
    words
}

/// Returns the words for how the settlement's defence measures up to those who might
/// raid it.
fn defence(defence: u32, neighbours: &[Neighbour]) -> String {
    let threat = neighbours
        .iter()
        .filter(|neighbour| neighbour.is_hostile())
        .map(Neighbour::raiders)
        .max();

    match threat {
        None => format!("Your defence is worth {defence}. Nobody is hostile to you."),
        Some(raiders) if defence >= raiders => format!(
            "Your defence is worth {defence}, which is enough against the {raiders} raiders \
             that the worst of your neighbours can send."
        ),
        Some(raiders) => format!(
            "Your defence is worth {defence}. The worst of your neighbours can send \
             {raiders} raiders."
        ),
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::Harness;

    fn neighbours() -> Vec<Neighbour> {
        Simulation::generate(Name::default(), Sex::Female, 1)
            .view()
            .neighbours
    }

    #[test]
    fn neighbour_is_told_by_where_it_lies_and_how_it_stands() {
        let mut neighbour = neighbours().remove(0);
        neighbour.days_away = 5;
        neighbour.way = 2;
        neighbour.people = 96;
        neighbour.advances = 0;
        neighbour.regard = -40;
        neighbour.pact = false;

        assert_eq!(
            describe(&neighbour),
            "5 days to the east, about 96 people, in the Stone Age."
        );
        assert_eq!(
            standing(&neighbour),
            "They are hostile towards you (-40). They can send 4 raiders."
        );
        neighbour.regard = 35;
        neighbour.pact = true;
        assert_eq!(
            standing(&neighbour),
            "They are friendly towards you (35). There is a pact of trade."
        );
    }

    #[test]
    fn defence_is_measured_against_the_worst_of_the_neighbours() {
        let mut all = neighbours();
        for neighbour in &mut all {
            neighbour.regard = 0;
        }
        assert_eq!(
            defence(4, &all),
            "Your defence is worth 4. Nobody is hostile to you."
        );

        all[0].regard = -50;
        all[0].people = 120;
        assert!(defence(4, &all).ends_with("can send 6 raiders."));
        assert!(defence(12, &all).contains("which is enough against the 6 raiders"));
    }

    #[test]
    fn tab_draws_alone_and_in_a_hamlet() {
        let harness = Harness::new();
        let alone = Simulation::new(Name::default(), Sex::Female).view();
        harness.show(|ui| assert_eq!(show(ui, &alone), None));

        let hamlet = Simulation::generate(Name::default(), Sex::Female, 1).view();
        assert_eq!(hamlet.neighbours.len(), 3);
        harness.show(|ui| assert_eq!(show(ui, &hamlet), None));
    }
}
