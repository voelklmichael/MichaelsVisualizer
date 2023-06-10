#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// hide console window on Windows in release
mod app;
mod confirm_exit;
mod data_types;
mod dialog;
mod localization;
pub use localization::{Language, LocalizableStr, LocalizableString};

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Michael Visualizer",
        options,
        Box::new(|cc| Box::new(Visualizer::new(cc))),
    )
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct Visualizer {
    #[serde(skip)]
    dialogs: dialog::DialogWidget,
    #[serde(skip)]
    confirm_exit: confirm_exit::ConfirmExit,
    body: app::App,
}

impl Visualizer {
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut vis = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };
        vis.body.init(cc);
        vis
    }
}

impl eframe::App for Visualizer {
    fn on_close_event(&mut self) -> bool {
        if let Some(dialog) = self.confirm_exit.close_event() {
            self.dialogs.push(dialog);
        }
        self.confirm_exit.shall_be_closed()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.confirm_exit.shall_be_closed() {
            frame.close()
        }
        let dialogs_are_done = self.dialogs.progress(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(dialogs_are_done);
            let events = self.body.show(ui);
            for event in events {
                match event {
                    app::AppEvent::CloseRequested => {
                        if let Some(dialog) = self.confirm_exit.close_event() {
                            self.dialogs.push(dialog);
                        }
                    }
                    app::AppEvent::Dialog(dialog) => self.dialogs.push(dialog),
                    app::AppEvent::Reset => self.body = Default::default(),
                }
            }
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
