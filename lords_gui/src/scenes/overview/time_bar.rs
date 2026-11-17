use egui::RichText;
use lords_runtime::{Runtime, Snapshot, Speed};
use lords_sim::Timestamp;

use crate::datetime::DateTimeFormat;
use crate::icons;

const TOGGLE_PAUSE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Space);

const CONTROL_SIZE: f32 = 16.0;
const PAUSE_BUTTON_SIZE: egui::Vec2 = egui::Vec2::splat(24.0);

pub fn show(
    ui: &mut egui::Ui,
    runtime: &Runtime,
    snapshot: &Snapshot,
    format: &DateTimeFormat,
    keyboard_free: bool,
) {
    egui::Panel::top("time_bar").show(ui, |ui| {
        egui::Aligned::new(egui::Align2::CENTER_TOP).show(ui, |ui| {
            ui.horizontal(|ui| {
                pause_button(ui, runtime, snapshot.paused, keyboard_free);
                clock(ui, format, snapshot.view.time, snapshot.paused);
                speed_slider(ui, runtime, snapshot.speed);
            });
        });
    });
}

fn pause_button(ui: &mut egui::Ui, runtime: &Runtime, paused: bool, keyboard_free: bool) {
    let (icon, action) = if paused {
        (icons::PLAY, "Resume")
    } else {
        (icons::PAUSE, "Pause")
    };

    let shortcut_pressed = keyboard_free && toggle_pause_pressed(ui);

    let shortcut = ui.ctx().format_shortcut(&TOGGLE_PAUSE);
    let button =
        egui::Button::new(RichText::new(icon).size(CONTROL_SIZE)).min_size(PAUSE_BUTTON_SIZE);
    let clicked = ui
        .add(button)
        .on_hover_text(format!("{action} ({shortcut})"))
        .clicked();

    if clicked || shortcut_pressed {
        if paused {
            runtime.resume();
        } else {
            runtime.pause();
        }
    }
}

fn clock(ui: &mut egui::Ui, format: &DateTimeFormat, time: Timestamp, paused: bool) {
    let now = RichText::new(format.date_time(time)).size(CONTROL_SIZE);

    ui.label(if paused { now.weak() } else { now.strong() });
}

fn speed_slider(ui: &mut egui::Ui, runtime: &Runtime, speed: Speed) {
    let mut multiplier = speed.multiplier();
    let slider = egui::Slider::new(
        &mut multiplier,
        Speed::BASE.multiplier()..=Speed::MAX.multiplier(),
    )
    .logarithmic(true)
    .suffix("×");

    let changed = ui.add(slider).on_hover_text("Simulation speed").changed();

    if changed {
        runtime.set_speed(Speed::new(multiplier));
    }
}

fn toggle_pause_pressed(ui: &egui::Ui) -> bool {
    if ui.ctx().text_edit_focused() {
        return false;
    }

    let egui::KeyboardShortcut {
        modifiers,
        logical_key,
    } = TOGGLE_PAUSE;

    ui.input_mut(|input| {
        let pressed = input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: held,
                    ..
                } if *key == logical_key && held.matches_logically(modifiers)
            )
        });
        input.consume_key(modifiers, logical_key);
        pressed
    })
}
