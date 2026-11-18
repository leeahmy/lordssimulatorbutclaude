use egui::RichText;
use lords_sim::View;
use lords_sim::journal::{Entry, Tone};

use crate::datetime::DateTimeFormat;

const DEFAULT_HEIGHT: f32 = 84.0;
const MIN_HEIGHT: f32 = 48.0;
const MAX_HEIGHT: f32 = 400.0;

/// The share of the window's height that the journal takes at most.
const MAX_SHARE: f32 = 0.4;

/// Shows the latest of what has happened to the player character along the bottom of the
/// window, the newest last and in view.
pub fn show(ui: &mut egui::Ui, view: &View, format: &DateTimeFormat) {
    let most = (ui.available_height() * MAX_SHARE).clamp(MIN_HEIGHT, MAX_HEIGHT);

    egui::Panel::bottom("journal_panel")
        .default_size(DEFAULT_HEIGHT.min(most))
        .size_range(MIN_HEIGHT..=most)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if view.journal.is_empty() {
                        ui.label(RichText::new("Nothing has happened yet.").weak());
                    }
                    for entry in view.journal.iter() {
                        line(ui, entry, format);
                    }
                });
        });
}

fn line(ui: &mut egui::Ui, entry: &Entry, format: &DateTimeFormat) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(format.date_time(entry.time)).weak());

        let text = RichText::new(&entry.text);
        ui.label(match entry.tone {
            Tone::Plain => text,
            Tone::Good => text.strong(),
            Tone::Bad => text.color(ui.visuals().warn_fg_color),
        });
    });
}

#[cfg(test)]
mod tests {
    use lords_sim::{Command, Item, Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::Harness;

    #[test]
    fn journal_draws_empty_and_with_entries_of_every_tone() {
        let harness = Harness::new();
        let format = DateTimeFormat::default();
        let empty = Simulation::new(Name::default(), Sex::Female).view();
        harness.show(|ui| show(ui, &empty, &format));

        let mut sim = Simulation::generate(Name::default(), Sex::Female, 1);
        sim.apply(Command::Consume(Item::Stone));
        sim.apply(Command::Drop(Item::RyeBread, 1));
        let view = sim.view();
        assert_eq!(view.journal.len(), 3);
        harness.show(|ui| show(ui, &view, &format));
    }
}
