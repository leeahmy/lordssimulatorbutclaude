use egui::RichText;
use lords_sim::View;
use lords_sim::settlement::Chronicled;
use lords_sim::time::gregorian::Date;

const SECTION_SPACING: f32 = 12.0;
const YEAR_WIDTH: f32 = 70.0;

/// Shows what the settlement remembers of itself, the latest first. Unlike the journal,
/// which is of one life and forgets, the chronicle is kept for good.
pub fn show(ui: &mut egui::Ui, view: &View) {
    let Some(town) = &view.settlement else {
        ui.label(RichText::new("No settlement").strong());
        ui.label(
            RichText::new("A chronicle is kept of a place, and there is no place to keep one of.")
                .weak(),
        );
        return;
    };

    ui.label(
        RichText::new(format!("The chronicle of {}", town.name))
            .heading()
            .strong(),
    );
    ui.label(RichText::new(span(view, &town.chronicle)).weak());
    ui.add_space(SECTION_SPACING);

    if town.chronicle.is_empty() {
        ui.label(RichText::new("Nothing has been written down yet.").weak());
    }
    for entry in town.chronicle.iter().rev() {
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(YEAR_WIDTH, ui.spacing().interact_size.y),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| ui.label(RichText::new(year(entry)).weak()),
            );
            ui.add(egui::Label::new(&entry.text).wrap());
        });
    }
    ui.add_space(SECTION_SPACING);
}

/// Returns the words for the year something happened in.
fn year(entry: &Chronicled) -> String {
    format!("Year {}", Date::from_timestamp(entry.time).year())
}

/// Returns the words for how far back the chronicle goes.
fn span(view: &View, chronicle: &[Chronicled]) -> String {
    let now = Date::from_timestamp(view.time).year();
    match chronicle.first() {
        None => "Kept for good, whoever comes and goes.".to_owned(),
        Some(first) => {
            let years = now.saturating_sub(Date::from_timestamp(first.time).year());
            match years {
                0 => "It was begun this year.".to_owned(),
                1 => "It goes back a year.".to_owned(),
                years => format!("It goes back {years} years."),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::time::gregorian::SECONDS_PER_DAY;
    use lords_sim::{Name, Sex, Simulation, Timestamp};

    use super::*;
    use crate::scenes::harness::Harness;
    use crate::scenes::overview::town::tests::settlement;

    #[test]
    fn chronicle_is_told_by_the_year_and_by_how_far_back_it_goes() {
        let mut view = Simulation::generate(Name::default(), Sex::Female, 1).view();
        let entry = |years: u64| Chronicled {
            time: Timestamp::from_seconds(years * 366 * SECONDS_PER_DAY),
            text: "The first well was built.".to_owned(),
        };
        view.time = Timestamp::from_seconds(90 * 366 * SECONDS_PER_DAY);
        let now = Date::from_timestamp(view.time).year();

        assert_eq!(year(&entry(90)), format!("Year {now}"));
        assert_eq!(span(&view, &[]), "Kept for good, whoever comes and goes.");
        assert_eq!(span(&view, &[entry(90)]), "It was begun this year.");
        assert!(span(&view, &[entry(40), entry(90)]).starts_with("It goes back 5"));
    }

    #[test]
    fn tab_draws_alone_and_with_a_chronicle() {
        let harness = Harness::new();
        let alone = Simulation::new(Name::default(), Sex::Female).view();
        harness.show(|ui| show(ui, &alone));

        let mut town = Simulation::generate(Name::default(), Sex::Female, 1).view();
        town.settlement = Some(std::sync::Arc::new(settlement(true)));
        harness.show(|ui| show(ui, &town));
    }
}
