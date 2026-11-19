use egui::RichText;
use lords_sim::politics::{Appeal, Issue, Law};
use lords_sim::view::{PoliticsView, SettlementView};
use lords_sim::{Command, View};

use crate::scenes::count;

const SECTION_SPACING: f32 = 12.0;
const BAR_WIDTH: f32 = 160.0;
const LEAN_WIDTH: f32 = 120.0;
const SIDE_WIDTH: f32 = 180.0;
const OFFICE_WIDTH: f32 = 260.0;

/// Shows how the settlement is led, where the player character stands in it, what they
/// hold, and the laws, with the means to change each. Returns what was asked for.
pub fn show(ui: &mut egui::Ui, view: &View) -> Option<Command> {
    let Some(town) = &view.settlement else {
        ui.label(RichText::new("No settlement").strong());
        ui.label(
            RichText::new(
                "There is nobody to lead and nothing to decide until you live among others.",
            )
            .weak(),
        );
        return None;
    };
    let politics = &town.politics;
    let mut command = None;

    leadership(ui, town);
    if politics.campaigning {
        ui.label(
            RichText::new("An election is near. Now is the time to be seen and heard.")
                .color(ui.visuals().warn_fg_color),
        );
    }
    ui.add_space(SECTION_SPACING);
    standing(ui, politics);
    ui.add_space(SECTION_SPACING);
    command = command.or(appeals(ui, politics));
    ui.add_space(SECTION_SPACING);
    command = command.or(views(ui, politics));
    ui.add_space(SECTION_SPACING);
    command = command.or(laws(ui, town));
    ui.add_space(SECTION_SPACING);

    command = command.or(offices(ui, politics));
    ui.add_space(SECTION_SPACING);

    if politics.leads {
        let mut steward = politics.steward;
        let toggled = ui
            .checkbox(&mut steward, "Leave the running of things to custom")
            .on_hover_text(
                "Who does what, what is built, and what is worked out are then seen to \
                 without you, the way they are under any other leader.",
            )
            .changed();
        if toggled {
            command = Some(Command::SetSteward(steward));
        }
    }

    command
}

/// Shows who holds which office, with the means to give each to someone else.
fn offices(ui: &mut egui::Ui, politics: &PoliticsView) -> Option<Command> {
    let mut command = None;

    ui.label(RichText::new("Offices").strong()).on_hover_text(
        "Whoever leads gives each office to one of the twenty grown-ups people think the \
         most of. Being given one raises the esteem someone is held in, and what they \
         think of you.",
    );
    egui::Grid::new("offices")
        .num_columns(2)
        .min_row_height(ui.spacing().interact_size.y)
        .spacing(egui::vec2(16.0, 6.0))
        .show(ui, |ui| {
            for (office, holder) in &politics.offices {
                ui.label(office.name()).on_hover_text(office.telling());
                let held = match holder {
                    Some((name, true)) => format!("{name}, who suits it"),
                    Some((name, false)) => name.clone(),
                    None => "Nobody".to_owned(),
                };
                if !politics.leads {
                    ui.label(RichText::new(held).weak());
                    ui.end_row();
                    continue;
                }

                let mut chosen: Option<Option<lords_sim::people::PersonId>> = None;
                egui::ComboBox::from_id_salt(("office", *office))
                    .width(OFFICE_WIDTH)
                    .selected_text(held)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(holder.is_none(), "Nobody").clicked() {
                            chosen = Some(None);
                        }
                        for (id, words) in &politics.trusted {
                            if ui.selectable_label(false, words).clicked() {
                                chosen = Some(Some(*id));
                            }
                        }
                    })
                    .response
                    .on_hover_text(office.telling());
                if let Some(chosen) = chosen {
                    command = Some(Command::Appoint(*office, chosen));
                }
                ui.end_row();
            }
        });

    command
}

fn leadership(ui: &mut egui::Ui, town: &SettlementView) {
    let politics = &town.politics;

    ui.label(
        RichText::new(format!(
            "{}, a {}",
            town.name,
            politics.era.name().to_lowercase()
        ))
        .heading()
        .strong(),
    );
    ui.label(RichText::new(politics.era.ways()).weak());
    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        let title = politics.era.title();
        match (&politics.leader, politics.leads) {
            (_, true) => ui.label(RichText::new(format!("You are {title} here.")).strong()),
            (Some(leader), false) => ui.label(format!("{leader} is {title} here.")),
            (None, false) => ui.label("Nobody leads here for now."),
        };
        ui.label(RichText::new(next_moot(politics)).weak());
    });
    if !politics.leads {
        ui.label(
            RichText::new(
                "Only whoever leads sets people to work, plans buildings, and makes laws.",
            )
            .weak(),
        );
    }
}

/// Returns the words for when a leader is next chosen.
fn next_moot(politics: &PoliticsView) -> String {
    match politics.next_moot {
        None => "Whoever leads does so for life.".to_owned(),
        Some(0) => "A leader is chosen today.".to_owned(),
        Some(days) => format!("A leader is chosen in {}.", count(days, "day")),
    }
}

fn standing(ui: &mut egui::Ui, politics: &PoliticsView) {
    ui.label(RichText::new("If a leader were chosen today").strong());
    if politics.candidates.is_empty() {
        ui.label(RichText::new("There is nobody to choose, and nobody to choose them.").weak());
    }
    for candidate in &politics.candidates {
        ui.horizontal(|ui| {
            let share = candidate.voices as f32 / politics.voices.max(1) as f32;
            ui.add(
                egui::ProgressBar::new(share)
                    .desired_width(BAR_WIDTH)
                    .text(format!("{} of {}", candidate.voices, politics.voices)),
            );
            if candidate.you {
                ui.label(RichText::new("You").strong());
            } else {
                ui.label(&candidate.name);
            }
        });
    }
    ui.label(RichText::new(account(politics.credit)).weak())
        .on_hover_text(
            "People take note of every 5,000 kcal of food, or as much as is worth that, \
             that you bring to the common store or take out of it, and of every hour you \
             work on a building.",
        );
}

/// Returns the words for how the player character stands with the common store.
fn account(credit: i64) -> String {
    match credit {
        0 => "You have neither brought to the common store nor taken out of it.".to_owned(),
        1.. => {
            format!("You have brought {credit} kcal more to the common store than you have taken.")
        }
        _ => format!(
            "You have taken {} kcal more out of the common store than you have brought.",
            -credit
        ),
    }
}

fn appeals(ui: &mut egui::Ui, politics: &PoliticsView) -> Option<Command> {
    let mut command = None;

    ui.label(RichText::new("Winning people over").strong());
    ui.horizontal_wrapped(|ui| {
        for appeal in Appeal::ALL {
            let rest = politics
                .appeal_rest
                .get(appeal as usize)
                .copied()
                .unwrap_or(0);
            let label = if rest > 0 {
                format!("{} (in {})", appeal.name(), count(rest, "day"))
            } else {
                appeal.name().to_owned()
            };
            let button = ui
                .add_enabled(rest == 0, egui::Button::new(label))
                .on_hover_text(appeal.effect())
                .on_disabled_hover_text(appeal.effect());
            if button.clicked() {
                command = Some(Command::Appeal(appeal));
            }
        }
    });
    ui.label(
        RichText::new(
            "Talking with people, giving them food, bringing goods to the store, and \
             working on buildings count too.",
        )
        .weak(),
    );

    command
}

fn views(ui: &mut egui::Ui, politics: &PoliticsView) -> Option<Command> {
    let mut command = None;

    ui.label(RichText::new("What you hold").strong())
        .on_hover_text(
            "Those who hold what you hold think better of you as a leader, and back the \
             laws you would make. Those who hold the opposite do neither.",
        );
    egui::Grid::new("views")
        .num_columns(3)
        .min_row_height(ui.spacing().interact_size.y)
        .show(ui, |ui| {
            for issue in Issue::ALL {
                let (one, other) = issue.sides();
                let mut lean = politics.views[issue as usize];

                ui.allocate_ui_with_layout(
                    egui::vec2(SIDE_WIDTH, ui.spacing().interact_size.y),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| ui.label(RichText::new(one).weak()),
                );
                ui.spacing_mut().slider_width = LEAN_WIDTH;
                let slider = egui::Slider::new(&mut lean, -2..=2).show_value(false);
                let [ones, neither, others] = politics.leanings[issue as usize];
                let leanings = format!(
                    "{}. Of the grown-ups, {ones} hold the one, {others} the other, and \
                     {neither} neither.",
                    issue.name()
                );
                if ui.add(slider).on_hover_text(leanings).changed() {
                    command = Some(Command::SetView(issue, lean));
                }
                ui.label(RichText::new(other).weak());
                ui.end_row();
            }
        });

    command
}

fn laws(ui: &mut egui::Ui, town: &SettlementView) -> Option<Command> {
    let politics = &town.politics;
    let mut command = None;

    ui.label(RichText::new("Laws").strong());
    ui.label(RichText::new(lawmaking(politics)).weak());
    egui::Grid::new("laws")
        .num_columns(3)
        .min_row_height(ui.spacing().interact_size.y)
        .spacing(egui::vec2(16.0, 6.0))
        .show(ui, |ui| {
            for (index, law) in Law::ALL.into_iter().enumerate() {
                // A law that takes knowing more than is known cannot be thought of yet.
                let thinkable = law
                    .needs()
                    .is_none_or(|needed| town.advances.contains(&needed));
                if !thinkable {
                    continue;
                }
                let in_force = politics.laws.contains(&law);
                let name = RichText::new(law.name());
                ui.label(if in_force { name.strong() } else { name })
                    .on_hover_text(law.effect());

                let ayes = politics.ayes.get(index).copied().unwrap_or(0);
                let mood = format!("{ayes} of {} for it", politics.grown_ups);
                ui.label(
                    RichText::new(if in_force {
                        format!("in force, {mood}")
                    } else {
                        mood
                    })
                    .weak(),
                );

                let verb = if in_force { "Repeal" } else { "Enact" };
                let button = ui
                    .add_enabled(politics.leads, egui::Button::new(verb))
                    .on_hover_text(law.effect())
                    .on_disabled_hover_text("Only whoever leads makes laws");
                if button.clicked() {
                    command = Some(Command::Legislate(law, !in_force));
                }
                ui.end_row();
            }
        });

    command
}

/// Returns the words for how laws come to be.
fn lawmaking(politics: &PoliticsView) -> String {
    if politics.decides_alone {
        "Whoever leads makes the laws without asking anyone. Those a law suits think \
         better of its maker, and the rest worse."
            .to_owned()
    } else {
        format!(
            "A law takes {} of the {} grown-ups for it, and giving one up as many against \
             it. Those a law suits think better of its maker, and the rest worse.",
            politics.needed, politics.grown_ups
        )
    }
}

#[cfg(test)]
pub(super) mod tests {
    use lords_sim::politics::Era;
    use lords_sim::view::Candidate;
    use lords_sim::{Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::Harness;

    pub(in crate::scenes::overview) fn politics(leads: bool) -> PoliticsView {
        PoliticsView {
            era: Era::Village,
            leader: Some("Inka Oja".to_owned()),
            leads,
            candidates: vec![
                Candidate {
                    name: "Inka Oja".to_owned(),
                    voices: 5,
                    you: false,
                },
                Candidate {
                    name: "Ada Lord".to_owned(),
                    voices: 3,
                    you: true,
                },
            ],
            voices: 8,
            laws: vec![Law::OpenGates],
            ayes: vec![1, 2, 3, 4, 5, 6, 7, 0, 4, 3, 2, 1, 0, 7, 6, 5],
            needed: 4,
            leanings: [[2, 3, 2], [0, 7, 0], [5, 1, 1], [1, 2, 4]],
            campaigning: !leads,
            offices: vec![
                (
                    lords_sim::politics::Office::Marshal,
                    Some(("Inka Oja".to_owned(), true)),
                ),
                (lords_sim::politics::Office::Treasurer, None),
            ],
            trusted: Vec::new(),
            grown_ups: 7,
            decides_alone: false,
            next_moot: Some(120),
            steward: false,
            credit: -6_200,
            views: [0, 1, -2, 2],
            appeal_rest: vec![0, 12, 0, 0, 3],
        }
    }

    #[test]
    fn moot_and_account_are_told_in_words() {
        let mut politics = politics(false);
        assert_eq!(next_moot(&politics), "A leader is chosen in 120 days.");
        politics.next_moot = Some(0);
        assert_eq!(next_moot(&politics), "A leader is chosen today.");
        politics.next_moot = None;
        assert_eq!(next_moot(&politics), "Whoever leads does so for life.");

        assert!(account(0).starts_with("You have neither"));
        assert!(account(7_500).starts_with("You have brought 7500 kcal more"));
        assert!(account(-6_200).starts_with("You have taken 6200 kcal more"));
    }

    #[test]
    fn lawmaking_is_told_by_who_has_a_say() {
        let mut politics = politics(true);
        assert!(lawmaking(&politics).starts_with("A law takes 4 of the 7 grown-ups"));
        politics.decides_alone = true;
        assert!(lawmaking(&politics).starts_with("Whoever leads makes the laws"));
    }

    #[test]
    fn tab_draws_alone_in_a_hamlet_and_for_a_leader() {
        let harness = Harness::new();
        let alone = Simulation::new(Name::default(), Sex::Female).view();
        harness.show(|ui| assert_eq!(show(ui, &alone), None));

        let newcomer = Simulation::generate(Name::default(), Sex::Female, 1).view();
        assert!(newcomer.settlement.is_some());
        harness.show(|ui| assert_eq!(show(ui, &newcomer), None));

        let mut leader = newcomer.clone();
        let mut town = (**newcomer.settlement.as_ref().expect("a hamlet")).clone();
        town.politics = politics(true);
        leader.settlement = Some(std::sync::Arc::new(town));
        harness.show(|ui| assert_eq!(show(ui, &leader), None));
    }
}
