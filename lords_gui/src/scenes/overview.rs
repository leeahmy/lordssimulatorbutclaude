mod activity;
mod character;
mod crafting;
mod inventory;
mod journal;
mod map;
mod skills;
mod surroundings;
mod time_bar;

use std::time::Instant;

use egui::RichText;
use lords_runtime::Snapshot;
use lords_save::SaveKind;
use lords_sim::time::gregorian::{Date, SECONDS_PER_DAY};
use lords_sim::{Command, Death, Subject, View};

use crate::datetime::DateTimeFormat;
use crate::playthrough::Playthrough;
use crate::scenes::count;

use super::{MainMenu, MenuAction, SAVE, Scene, SceneResponse, Shared, top_panel};

const SAVE_DIALOG_WIDTH: f32 = 300.0;
const SAVE_LABEL_LIMIT: usize = 40;
const DEATH_DIALOG_WIDTH: f32 = 340.0;

pub struct Overview {
    game: Playthrough,
    save_dialog: Option<String>,
    focus_save_dialog: bool,
    map_reach: f32,
    hovered: Option<Subject>,
    tab: Tab,
    /// Whether the player has chosen to look around after their character died.
    death_acknowledged: bool,
}

/// What the middle of the overview shows.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Tab {
    #[default]
    Surroundings,
    Inventory,
    Crafting,
    Skills,
}

impl Tab {
    const ALL: [Self; 4] = [
        Self::Surroundings,
        Self::Inventory,
        Self::Crafting,
        Self::Skills,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Surroundings => "Surroundings",
            Self::Inventory => "Inventory",
            Self::Crafting => "Crafting",
            Self::Skills => "Skills",
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Pointed {
    command: Option<Command>,
    hovered: Option<Subject>,
}

impl Overview {
    pub fn new(game: Playthrough) -> Self {
        Self {
            game,
            save_dialog: None,
            focus_save_dialog: false,
            map_reach: map::DEFAULT_REACH,
            hovered: None,
            tab: Tab::default(),
            death_acknowledged: false,
        }
    }
}

impl Scene for Overview {
    fn update(&mut self, ui: &mut egui::Ui, shared: &mut Shared<'_>) -> SceneResponse {
        let runtime = self.game.runtime();
        let snapshot = runtime.snapshot();
        let view = &snapshot.view;
        let keyboard_free = !shared.dialog_open && self.save_dialog.is_none();

        let mut action = top_panel(ui, true);
        if keyboard_free && ui.input_mut(|input| input.consume_shortcut(&SAVE)) {
            action = Some(MenuAction::Save);
        }

        let format = &shared.settings.datetime;
        time_bar::show(ui, runtime, &snapshot, format, keyboard_free);
        let mut leave = !runtime.is_running() && stopped_banner(ui);
        character::show(ui, view, format);
        journal::show(ui, view, format);
        let on_map = map::show(ui, view, &mut self.map_reach, self.hovered);
        let in_list = central(ui, view, &mut self.tab, self.hovered);
        self.hovered = on_map.hovered.or(in_list.hovered);
        if let Some(command) = on_map.command.or(in_list.command) {
            runtime.send(command);
        }
        if let Some(death) = view.death.filter(|_| !self.death_acknowledged) {
            match death_dialog(ui, view, death, format) {
                Some(AfterDeath::Leave) => leave = true,
                Some(AfterDeath::Stay) => self.death_acknowledged = true,
                None => {}
            }
        }

        let interval = shared.settings.autosave_interval();
        self.game.autosave(&snapshot, interval, Instant::now());
        self.show_save_dialog(ui);

        let mut app_action = None;
        match action {
            Some(MenuAction::Save) => match self.game.save_label() {
                Some(label) => self.game.save(SaveKind::Manual, label.to_owned()),
                None => self.open_save_dialog(shared, &snapshot),
            },
            Some(MenuAction::SaveAs) => self.open_save_dialog(shared, &snapshot),
            Some(MenuAction::MainMenu) => leave = true,
            Some(MenuAction::App(action)) => app_action = Some(action),
            None => {}
        }

        for request in self.game.take_captured() {
            shared.saves.save(request);
        }

        if let Some(action) = app_action {
            SceneResponse::App(action)
        } else if leave {
            SceneResponse::Transition(Box::new(MainMenu::default()))
        } else {
            SceneResponse::None
        }
    }

    fn on_exit(&mut self, shared: &mut Shared<'_>) {
        for request in self.game.finish() {
            shared.saves.save(request);
        }
    }
}

impl Overview {
    fn open_save_dialog(&mut self, shared: &Shared<'_>, snapshot: &Snapshot) {
        let suggestion = self.game.save_label().map_or_else(
            || shared.settings.datetime.date(snapshot.view.time),
            str::to_owned,
        );

        self.save_dialog = Some(suggestion);
        self.focus_save_dialog = true;
    }

    fn show_save_dialog(&mut self, ui: &mut egui::Ui) {
        let Some(mut label) = self.save_dialog.take() else {
            return;
        };
        let mut save = false;
        let mut cancel = false;

        let modal = egui::Modal::new(egui::Id::unique("save_dialog")).show(ui.ctx(), |ui| {
            ui.set_width(SAVE_DIALOG_WIDTH);
            ui.heading("Save game");
            ui.add_space(4.0);

            let field = ui.add(
                egui::TextEdit::singleline(&mut label)
                    .hint_text("Name of the save")
                    .char_limit(SAVE_LABEL_LIMIT)
                    .desired_width(f32::INFINITY),
            );
            if self.focus_save_dialog {
                field.request_focus();
                self.focus_save_dialog = false;
            }
            let entered =
                field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

            ui.label(
                RichText::new("A save with the same name is replaced.")
                    .weak()
                    .small(),
            );
            ui.add_space(8.0);

            let named = !label.trim().is_empty();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                save = ui.add_enabled(named, egui::Button::new("Save")).clicked()
                    || (entered && named);
                cancel = ui.button("Cancel").clicked();
            });
        });

        if save {
            self.game.save(SaveKind::Manual, label.trim().to_owned());
        } else if !(cancel || modal.should_close()) {
            self.save_dialog = Some(label);
        }
    }
}

/// Shows the middle of the overview: what the player character is doing, and under it
/// the tab chosen.
fn central(ui: &mut egui::Ui, view: &View, tab: &mut Tab, hovered: Option<Subject>) -> Pointed {
    let mut pointed = Pointed::default();

    egui::CentralPanel::default().show(ui, |ui| {
        let asked = activity::show(ui, view);

        ui.horizontal(|ui| {
            for choice in Tab::ALL {
                ui.selectable_value(tab, choice, choice.title());
            }
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| match tab {
                Tab::Surroundings => pointed = surroundings::show(ui, view, hovered),
                Tab::Inventory => pointed.command = inventory::show(ui, view),
                Tab::Crafting => pointed.command = crafting::show(ui, view),
                Tab::Skills => skills::show(ui, view),
            });
        pointed.command = asked.or(pointed.command.take());
    });

    pointed
}

/// What the player chose to do once their character had died.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AfterDeath {
    Leave,
    Stay,
}

fn death_dialog(
    ui: &mut egui::Ui,
    view: &View,
    death: Death,
    format: &DateTimeFormat,
) -> Option<AfterDeath> {
    let mut choice = None;

    let modal = egui::Modal::new(egui::Id::unique("death_dialog")).show(ui.ctx(), |ui| {
        ui.set_width(DEATH_DIALOG_WIDTH);
        ui.heading("Your life has ended");
        ui.add_space(4.0);
        ui.label(epitaph(view, death, format));
        ui.add_space(8.0);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Main Menu").clicked() {
                choice = Some(AfterDeath::Leave);
            }
            if ui.button("Look around").clicked() {
                choice = Some(AfterDeath::Stay);
            }
        });
    });

    choice.or(modal.should_close().then_some(AfterDeath::Stay))
}

/// Returns the words for how a life went and how it ended.
fn epitaph(view: &View, death: Death, format: &DateTimeFormat) -> String {
    use lords_sim::object::Living;

    let player = &view.player;
    let age = Date::from(death.time).years_since(Date::from(player.birth()));
    let days = death.time.seconds_since(view.started) / SECONDS_PER_DAY;
    let walked = view.walked.as_millimeters() / 1_000_000;

    format!(
        "{} died of {} on {}, at the age of {age}, after {} in the forest and {} on foot.",
        player.name(),
        death.cause.name(),
        format.date(death.time),
        count(days, "day"),
        count(walked as u64, "kilometer"),
    )
}

fn stopped_banner(ui: &mut egui::Ui) -> bool {
    let mut leave = false;

    egui::Panel::top("stopped_banner").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(
                ui.visuals().error_fg_color,
                "The simulation has stopped unexpectedly.",
            );
            leave = ui.button("Main Menu").clicked();
        });
    });

    leave
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lords_runtime::{Runtime, Speed};
    use lords_save::SimulationId;
    use lords_sim::{Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::{Harness, wait_until};

    fn overview_of(sim: Simulation, paused: bool) -> Overview {
        let runtime = Runtime::spawn_with(sim, Speed::new(10), paused, || {});

        Overview::new(Playthrough::new(runtime, SimulationId::generate(), None))
    }

    fn overview(paused: bool) -> Overview {
        let name = Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned());
        overview_of(Simulation::new(name, Sex::Female), paused)
    }

    #[test]
    fn save_shortcut_asks_for_a_name_once_and_then_replaces_that_save() {
        let mut harness = Harness::new();
        let mut overview = overview(true);
        harness.frames(&mut overview, 2);

        harness.press(&mut overview, egui::Modifiers::COMMAND, egui::Key::S);
        assert_eq!(overview.save_dialog.as_deref(), Some("1.1.1"));
        assert!(harness.saves.list().saves.is_empty());

        harness.frames(&mut overview, 3);
        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Enter);
        assert_eq!(overview.save_dialog, None);
        assert_eq!(overview.game.save_label(), Some("1.1.1"));
        harness.wait_for_save_from(&mut overview);

        let saves = harness.saves.list().saves;
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].header.kind, SaveKind::Manual);
        assert_eq!(saves[0].header.label, "1.1.1");
        assert_eq!(saves[0].header.character, "Ada Lord");
        assert_eq!(saves[0].header.speed, 10);

        harness.press(&mut overview, egui::Modifiers::COMMAND, egui::Key::S);
        assert_eq!(overview.save_dialog, None);
        harness.wait_for_save_from(&mut overview);
        assert_eq!(harness.saves.list().saves.len(), 1);
        assert!(harness.saves.error().is_none());
        assert!(harness.saves.saved_notice().is_some());
    }

    #[test]
    fn escape_closes_the_save_dialog_without_saving() {
        let mut harness = Harness::new();
        let mut overview = overview(true);

        harness.press(&mut overview, egui::Modifiers::COMMAND, egui::Key::S);
        harness.frames(&mut overview, 3);
        assert!(overview.save_dialog.is_some());

        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Escape);
        // Once to let go of the name field, once more to close the dialog.
        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Escape);

        assert_eq!(overview.save_dialog, None);
        assert_eq!(overview.game.save_label(), None);
        harness.saves.finish();
        assert!(harness.saves.list().saves.is_empty());
    }

    #[test]
    fn leaving_writes_an_exit_save_that_resumes_the_game() {
        let mut harness = Harness::new();
        let mut overview = overview(true);
        harness.frames(&mut overview, 2);

        harness.exit(&mut overview);
        harness.saves.finish();

        let saves = harness.saves.list().saves;
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].header.kind, SaveKind::Exit);

        let loaded = harness
            .saves
            .load(&saves[0].path)
            .expect("the exit save loads");
        let state = overview
            .game
            .runtime()
            .capture()
            .recv()
            .expect("the simulation is still running");
        assert_eq!(loaded.state, *state);
        assert_eq!(loaded.header.speed, 10);
    }

    #[test]
    fn space_pauses_and_resumes() {
        let mut harness = Harness::new();
        let mut overview = overview(false);
        harness.frames(&mut overview, 2);

        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Space);
        wait_until(|| overview.game.runtime().snapshot().paused);

        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Space);
        wait_until(|| !overview.game.runtime().snapshot().paused);
    }

    #[test]
    fn shortcuts_are_left_alone_while_a_dialog_is_open() {
        let mut harness = Harness::new();
        harness.dialog_open = true;
        let mut overview = overview(false);
        harness.frames(&mut overview, 2);

        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Space);
        harness.press(&mut overview, egui::Modifiers::COMMAND, egui::Key::S);
        std::thread::sleep(Duration::from_millis(50));

        assert!(!overview.game.runtime().snapshot().paused);
        assert_eq!(overview.save_dialog, None);
    }

    #[test]
    fn clicking_a_thing_on_the_map_sends_the_player_walking_to_it() {
        let mut harness = Harness::new();
        let forest = Simulation::generate(Name::default(), Sex::Female, 1);
        let mut overview = overview_of(forest, true);
        harness.frames(&mut overview, 2);

        let mut around_the_player = (0..8).flat_map(|row| {
            (0..8).map(move |column| {
                egui::pos2(770.0 + 8.0 * column as f32, 270.0 + 8.0 * row as f32)
            })
        });
        let spot = around_the_player
            .find(|spot| {
                harness.frame(&mut overview, vec![egui::Event::PointerMoved(*spot)]);
                harness.frame(&mut overview, Vec::new());
                overview.hovered.is_some()
            })
            .expect("the pointer is on something on the map");
        let pointed_at = overview.hovered.expect("it was just found");
        let known = overview.game.runtime().snapshot().view.map.things.clone();
        assert!(known.iter().any(|mark| mark.subject == pointed_at));
        assert_eq!(overview.game.runtime().snapshot().view.walk, None);

        let button = |pressed| egui::Event::PointerButton {
            pos: spot,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        harness.frame(&mut overview, vec![button(true)]);
        harness.frame(&mut overview, vec![button(false)]);

        wait_until(|| overview.game.runtime().snapshot().view.walk.is_some());
    }

    #[test]
    fn overview_draws_in_a_small_window_and_stays() {
        let mut harness = Harness::new().sized(480.0, 300.0);
        let mut overview = overview(true);

        // Nothing should ask to leave the game by itself, however cramped it is.
        for _ in 0..3 {
            assert!(matches!(
                harness.frame(&mut overview, Vec::new()),
                SceneResponse::None
            ));
        }
        assert!(overview.game.runtime().is_running());
    }
}
