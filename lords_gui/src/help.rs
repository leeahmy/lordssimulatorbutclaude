//! The guide to the game, shown from the Help menu.

use egui::RichText;

const DIALOG_WIDTH: f32 = 520.0;

/// How much of the window's height the guide takes at most.
const MAX_HEIGHT_SHARE: f32 = 0.7;

const CHAPTERS: [(&str, &[&str]); 7] = [
    (
        "The life",
        &[
            "You are alone in a northern forest by a lake, with a knife, a fire steel, and a \
             little bread. It is the first of May. Nothing is asked of you but to live, and \
             winter is the hardest part of that.",
            "The bars on the left tell how your body is doing. Hunger kills in weeks, thirst \
             in days, and the cold in hours.",
        ],
    ),
    (
        "Getting around",
        &[
            "Click something on the map, or its Walk to button in the list, to walk to it. \
             You can only walk to what you have seen. Scroll over the map to zoom.",
            "Choosing something to do to a thing that is out of reach walks you to it first.",
            "The slider at the top sets how fast time passes, and Space pauses it. The game \
             pauses by itself when your life is in danger.",
        ],
    ),
    (
        "Water and food",
        &[
            "Drink from the lake. Now and then lake water makes you sick. Water boiled in a \
             birch-bark pot by a fire never does.",
            "Fish with a rod, sink fish traps by the shore, and set snares for hares. Pick \
             bilberries from the middle of July and lingonberries from late August. The inner \
             bark of pine fills a stomach when nothing else does.",
            "Raw meat and fish may make you sick, and go off in a day or two unless it is \
             freezing. Cook them over a fire, or hang them on a drying rack for two days to \
             make them keep for good.",
        ],
    ),
    (
        "Fire and warmth",
        &[
            "A campfire takes five sticks, a piece of birch bark for tinder, and a fire steel. \
             Rain and wind make it harder to light. A stick burns for a quarter of an hour \
             and a log for two hours.",
            "Stay within reach of the fire to be warmed by it. A lean-to keeps the rain and \
             the wind off, a bough bed makes for better sleep, and a log cabin is warm \
             even in the dead of winter.",
            "Hare pelts sewn into a hat and mittens, boots, and a coat keep you far warmer \
             than the clothes you came in.",
        ],
    ),
    (
        "Making things",
        &[
            "The Crafting tab lists everything you can make. What you carry and what lies on \
             the ground within your reach both count, so heavy things like logs can be \
             piled up where you build.",
            "Cordage is twisted from spruce roots, and most things take some. A stone axe, \
             of two stones, a stick, and cordage, fells trees for logs.",
            "Work set aside is taken up from where you left it.",
        ],
    ),
    (
        "Sleep",
        &[
            "Sleep when you are tired, or you will collapse sooner or later. Nobody falls \
             asleep cold, and the cold wakes a sleeper, so see to your fire first.",
        ],
    ),
    (
        "Saving",
        &[
            "The game is saved when you leave it, and from time to time as you play. Save \
             under a name of your own from the File menu.",
        ],
    ),
];

/// Shows the guide, and returns whether it is to stay open.
pub fn show(ctx: &egui::Context) -> bool {
    let mut close = false;
    let max_height = ctx.content_rect().height() * MAX_HEIGHT_SHARE;

    let modal = egui::Modal::new(egui::Id::unique("help")).show(ctx, |ui| {
        ui.set_width(
            DIALOG_WIDTH
                .min(ctx.content_rect().width() - 48.0)
                .max(120.0),
        );
        ui.heading("How to Play");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(max_height)
            .show(ui, |ui| {
                for (title, paragraphs) in CHAPTERS {
                    ui.add_space(6.0);
                    ui.label(RichText::new(title).strong());
                    for paragraph in paragraphs {
                        ui.label(*paragraph);
                        ui.add_space(2.0);
                    }
                }
            });

        ui.add_space(8.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            close = ui.button("Close").clicked();
        });
    });

    !(close || modal.should_close())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guide_draws_in_any_window_and_stays_open_until_closed() {
        for (width, height) in [(960.0, 540.0), (300.0, 200.0)] {
            let ctx = egui::Context::default();
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, height),
                )),
                ..Default::default()
            };

            let mut open = false;
            ctx.run_ui(input, |ui| open = show(ui.ctx()))
                .drop_without_applying_deltas();
            assert!(open);
        }
    }

    #[test]
    fn every_chapter_has_something_to_say() {
        for (title, paragraphs) in CHAPTERS {
            assert!(!title.is_empty());
            assert!(!paragraphs.is_empty());
        }
    }
}
