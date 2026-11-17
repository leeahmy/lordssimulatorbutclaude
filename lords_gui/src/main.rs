#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    env_logger::init();

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/icon-256.png"))
        .expect("the icon built into the game is not a PNG image");
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(lords_gui::APP_ID)
            .with_maximized(true)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Lord's Simulator",
        native_options,
        Box::new(|cc| Ok(Box::new(lords_gui::App::new(cc)))),
    )
}
