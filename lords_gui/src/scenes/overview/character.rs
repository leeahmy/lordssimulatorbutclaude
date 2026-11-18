use egui::RichText;
use lords_sim::View;
use lords_sim::body::Body;
use lords_sim::climate::Conditions;
use lords_sim::object::Living;
use lords_sim::time::gregorian::{Date, SECONDS_PER_DAY};
use lords_sim::world::StructureKind;
use lords_sim::{Human, Timestamp};

use crate::datetime::DateTimeFormat;
use crate::icons;
use crate::scenes::count;

const PANEL_WIDTH: f32 = 200.0;
const PANEL_PADDING: f32 = 4.0;
const FACT_SPACING: egui::Vec2 = egui::vec2(12.0, 6.0);
const SECTION_SPACING: f32 = 10.0;

/// The level, in hundredths, below which a need is shown as pressing, and as dire.
const LOW: u32 = 35;
const DIRE: u32 = 10;

pub fn show(ui: &mut egui::Ui, view: &View, format: &DateTimeFormat) {
    let player = &view.player;

    egui::Panel::left("character_panel")
        .resizable(false)
        .exact_size(PANEL_WIDTH)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.add_space(PANEL_PADDING);
                    ui.add(
                        egui::Label::new(
                            RichText::new(player.name().to_string()).heading().strong(),
                        )
                        .wrap(),
                    );
                    ui.separator();
                    facts(ui, player, view.time, format);

                    ui.add_space(SECTION_SPACING);
                    ui.separator();
                    needs(ui, player.body());

                    ui.add_space(SECTION_SPACING);
                    ui.separator();
                    weather(ui, view);
                });
        });
}

fn facts(ui: &mut egui::Ui, player: &Human, time: Timestamp, format: &DateTimeFormat) {
    let sex = player.sex();
    let birth = player.birth();

    egui::Grid::new("character_facts")
        .num_columns(2)
        .spacing(FACT_SPACING)
        .show(ui, |ui| {
            fact(ui, "Sex", format!("{} {sex}", icons::sex(sex)));
            fact(ui, "Age", format_age(time, birth));
            fact(ui, "Born", format.date(birth));
        });
}

fn needs(ui: &mut egui::Ui, body: &Body) {
    need(ui, "Health", body.health_percent(), None);
    need(ui, "Food", body.energy_percent(), None);
    need(ui, "Water", body.water_percent(), None);
    need(ui, "Rest", body.rest_percent(), None);
    let temperature = format!("{:.1} °C", f64::from(body.temperature().as_tenths()) / 10.0);
    need(ui, "Warmth", body.warmth_percent(), Some(temperature));

    for ailment in body.ailments() {
        let color = if ailment.is_grave() {
            ui.visuals().error_fg_color
        } else {
            ui.visuals().warn_fg_color
        };
        ui.colored_label(color, ailment.name());
    }
}

/// Shows how well one need of the body is met, as a bar that is `percent` full.
fn need(ui: &mut egui::Ui, name: &str, percent: u32, reading: Option<String>) {
    let reading = reading.unwrap_or_else(|| format!("{percent}%"));
    let mut bar = egui::ProgressBar::new(percent as f32 / 100.0)
        .desired_width(ui.available_width())
        .text(format!("{name}  {reading}"));
    if percent < DIRE {
        bar = bar.fill(ui.visuals().error_fg_color.gamma_multiply(0.6));
    } else if percent < LOW {
        bar = bar.fill(ui.visuals().warn_fg_color.gamma_multiply(0.6));
    }

    ui.add(bar);
}

fn weather(ui: &mut egui::Ui, view: &View) {
    let conditions = &view.conditions;

    egui::Grid::new("weather_facts")
        .num_columns(2)
        .spacing(FACT_SPACING)
        .show(ui, |ui| {
            fact(ui, "Season", conditions.season.name().to_owned());
            fact(ui, "Sky", sky(conditions));
            fact(ui, "Wind", conditions.wind.name().to_owned());
            fact(ui, "Air", conditions.temperature.to_string());
            fact(ui, "Feels", view.felt.to_string());
        });

    let at_hand = &view.at_hand;
    let mut notes = Vec::new();
    match at_hand.shelter {
        Some(StructureKind::Cabin) => notes.push("In the cabin"),
        Some(_) => notes.push("Under the lean-to"),
        None => {}
    }
    let by_the_fire = at_hand.fire_until.map(|until| {
        let minutes = until.seconds_since(view.time) / 60;
        if minutes < 60 {
            format!("By the fire, {minutes} min of wood")
        } else {
            format!("By the fire, {} h of wood", minutes / 60)
        }
    });
    if let Some(by_the_fire) = &by_the_fire {
        notes.push(by_the_fire);
    }
    if conditions.snow {
        notes.push("Snow on the ground");
    }
    if conditions.frozen {
        notes.push("The lake is frozen");
    }
    for note in notes {
        ui.label(RichText::new(note).weak());
    }
}

/// Returns the words for what the sky is like, by day or by night.
fn sky(conditions: &Conditions) -> String {
    format!(
        "{}, {}",
        conditions.sky.name(),
        conditions.light.name().to_lowercase()
    )
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
