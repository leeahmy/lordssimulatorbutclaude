use egui::RichText;
use lords_sim::object::Living;
use lords_sim::time::gregorian::{Date, SECONDS_PER_DAY};
use lords_sim::{Human, Timestamp};

use crate::datetime::DateTimeFormat;
use crate::icons;
use crate::scenes::count;

const PANEL_WIDTH: f32 = 200.0;
const PANEL_PADDING: f32 = 4.0;
const FACT_SPACING: egui::Vec2 = egui::vec2(12.0, 6.0);

pub fn show(ui: &mut egui::Ui, player: &Human, time: Timestamp, format: &DateTimeFormat) {
    let sex = player.sex();
    let birth = player.birth();

    egui::Panel::left("character_panel")
        .resizable(false)
        .exact_size(PANEL_WIDTH)
        .show(ui, |ui| {
            ui.add_space(PANEL_PADDING);
            ui.add(
                egui::Label::new(RichText::new(player.name().to_string()).heading().strong())
                    .wrap(),
            );
            ui.separator();

            egui::Grid::new("character_facts")
                .num_columns(2)
                .spacing(FACT_SPACING)
                .show(ui, |ui| {
                    fact(ui, "Sex", format!("{} {sex}", icons::sex(sex)));
                    fact(ui, "Age", format_age(time, birth));
                    fact(ui, "Born", format.date(birth));
                });
        });
}

fn fact(ui: &mut egui::Ui, key: &str, value: String) {
    ui.label(RichText::new(key).weak());
    ui.label(value);
    ui.end_row();
}

fn format_age(now: Timestamp, birth: Timestamp) -> String {
    let today = Date::from(now);
    let birthday = Date::from(birth);

    let years = today.years_since(birthday);
    let months = today.months_since(birthday);
    if years > 0 {
        count(years, "year")
    } else if months > 0 {
        count(months, "month")
    } else {
        count(now.seconds_since(birth) / SECONDS_PER_DAY, "day")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn days(days: u64) -> Timestamp {
        Timestamp::from_seconds(days * SECONDS_PER_DAY)
    }

    #[test]
    fn age_is_given_in_the_largest_completed_unit() {
        let birth = Timestamp::EPOCH;

        assert_eq!(format_age(birth, birth), "0 days");
        assert_eq!(format_age(Timestamp::from_seconds(86_399), birth), "0 days");
        assert_eq!(format_age(days(1), birth), "1 day");
        assert_eq!(format_age(days(30), birth), "30 days");
        // 1 February of the first year.
        assert_eq!(format_age(days(31), birth), "1 month");
        assert_eq!(format_age(days(364), birth), "11 months");
        assert_eq!(format_age(days(365), birth), "1 year");
        assert_eq!(format_age(days(2 * 365), birth), "2 years");
    }

    #[test]
    fn age_never_counts_backwards() {
        assert_eq!(format_age(Timestamp::EPOCH, days(400)), "0 days");
    }
}
