mod dummy;
mod file;
mod limits;
use egui_dock::Tree;
use michael_visualizer_basic::{SimpleFileKey, SimpleLimitKey};

use crate::Language;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(super) struct App {
    language: super::Language,
    tabs: Tabs,
    mode: DarkLightMode,
    #[serde(skip)]
    data_events: Vec<
        michael_visualizer_basic::DataEvent<
            SimpleFileKey,
            SimpleLimitKey,
            file::File,
            limits::Limit,
        >,
    >,
    data_center: michael_visualizer_basic::DataCenter<
        SimpleFileKey,
        SimpleLimitKey,
        file::File,
        limits::Limit,
    >,
}
impl App {
    pub(super) fn init(&self, cc: &eframe::CreationContext) {
        cc.egui_ctx.set_visuals(match self.mode {
            DarkLightMode::Dark => egui::Visuals::dark(),
            DarkLightMode::Light => egui::Visuals::light(),
        })
    }
    pub(super) fn show(&mut self, ui: &mut egui::Ui) -> Vec<AppEvent> {
        let mut events = Vec::new();
        let redraw_event = self
            .data_center
            .progress(std::mem::take(&mut self.data_events).into_iter());
        self.redraw_event(redraw_event);
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                // dark/light mode switch
                {
                    /// Show small toggle-button for light and dark mode.
                    #[must_use]
                    fn light_dark_small_toggle_button(
                        is_dark_mode: bool,
                        ui: &mut egui::Ui,
                    ) -> Option<DarkLightMode> {
                        #![allow(clippy::collapsible_else_if)]
                        if is_dark_mode {
                            if ui
                                .add(egui::Button::new("☀").frame(false))
                                .on_hover_text("Switch to light mode")
                                .clicked()
                            {
                                ui.close_menu();
                                return Some(DarkLightMode::Light);
                            }
                        } else {
                            if ui
                                .add(egui::Button::new("🌙").frame(false))
                                .on_hover_text("Switch to dark mode")
                                .clicked()
                            {
                                ui.close_menu();
                                return Some(DarkLightMode::Dark);
                            }
                        }
                        None
                    }
                    let style: egui::Style = (*ui.ctx().style()).clone();
                    let new_visuals = light_dark_small_toggle_button(style.visuals.dark_mode, ui);
                    if let Some(mode) = new_visuals {
                        self.mode = mode;
                        ui.ctx().set_visuals(mode.visuals());
                    }
                }
                // quit button
                {
                    if ui.button("Quit").clicked() {
                        events.push(AppEvent::CloseRequested);
                        ui.close_menu();
                    }
                }
            });
        });
        egui_dock::DockArea::new(&mut self.tabs.tabs).show_inside(
            ui,
            &mut AppState {
                language: self.language,
                app_events: &mut events,
                data_events: &mut self.data_events,
            },
        );
        events
    }

    fn redraw_event(&self, redraw_event: michael_visualizer_basic::RedrawSelection) {}
}

#[derive(serde::Deserialize, serde::Serialize)]
enum Tab {
    Dummy(dummy::DummyTab),
    Limit(limits::LimitTab),
    Files(file::FileTab),
}
trait TabTrait {
    fn title(&self, state: &AppState) -> &str;
    fn show(&mut self, state: &mut AppState, ui: &mut egui::Ui);
}
impl Tab {
    fn title(&self, viewer: &AppState) -> &str {
        match self {
            Tab::Dummy(d) => d.title(viewer),
            Tab::Limit(d) => d.title(viewer),
            Tab::Files(d) => d.title(viewer),
        }
    }
    fn show(&mut self, viewer: &mut AppState, ui: &mut egui::Ui) {
        match self {
            Tab::Dummy(d) => d.show(viewer, ui),
            Tab::Limit(d) => d.show(viewer, ui),
            Tab::Files(d) => d.show(viewer, ui),
        }
    }
}
struct AppState<'a> {
    language: Language,
    app_events: &'a mut Vec<AppEvent>,
    data_events: &'a mut Vec<
        michael_visualizer_basic::DataEvent<
            SimpleFileKey,
            SimpleLimitKey,
            file::File,
            limits::Limit,
        >,
    >,
}

impl<'a> egui_dock::TabViewer for AppState<'a> {
    type Tab = Tab;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, tab: &mut Self::Tab) {
        tab.show(self, ui)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        tab.title(self).into()
    }
}
pub(super) enum AppEvent {
    CloseRequested,
    Dialog(crate::dialog::Dialog),
}
#[derive(PartialEq, Default, serde::Deserialize, serde::Serialize, Clone, Copy)]
enum DarkLightMode {
    Dark,
    #[default]
    Light,
}
impl DarkLightMode {
    fn visuals(&self) -> egui::Visuals {
        match self {
            DarkLightMode::Dark => egui::Visuals::dark(),
            DarkLightMode::Light => egui::Visuals::light(),
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
struct Tabs {
    tabs: Tree<Tab>,
}
impl Default for Tabs {
    fn default() -> Self {
        let tabs: Tree<Tab> = Tree::new(vec![
            Tab::Files(Default::default()),
            Tab::Limit(Default::default()),
            Tab::Dummy(Default::default()),
        ]);
        Self { tabs }
    }
}
