mod _dark_light;
mod _tabs;
mod dummy;
mod file_loader;
mod files;
mod heatmap;
mod limits;
mod selection;
mod violinplot;
use std::collections::HashMap;

use crate::{
    data_types::{FileKey, FileKeyGenerator, LimitKey},
    Language,
};
use _tabs::TabTrait;
type Filtering = Vec<bool>;
static RESET: crate::LocalizableStr<'static> = crate::LocalizableStr { english: "Reset" };

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(super) struct App {
    language: super::Language,
    tabs: _tabs::Tabs,
    mode: _dark_light::DarkLightMode,
    limits: limits::LimitContainer,
    files: files::FileContainer,
    file_key_generator: crate::data_types::FileKeyGenerator,
    limit_key_generator: crate::data_types::LimitKeyGenerator,
    file_loader: file_loader::FileLoader,
    #[serde(skip)]
    data_events: Vec<DataEvent>,
    #[serde(skip)]
    filterings: HashMap<(LimitKey, FileKey), Filtering>,
    #[serde(skip)]
    total_filterings: HashMap<FileKey, Box<[u32]>>,
}

impl App {
    pub(super) fn init(&mut self, cc: &eframe::CreationContext) {
        let mut kinds = _tabs::TabKind::kinds();
        for kind in self.tabs.tabs.tabs().map(|x| x.kind()) {
            if let Some(index) = kinds.iter().position(|&x| kind == x) {
                kinds.remove(index);
            }
        }
        for kind in kinds {
            self.tabs.tabs.push_to_first_leaf(kind.to_tab());
        }
        cc.egui_ctx.set_visuals(match self.mode {
            _dark_light::DarkLightMode::Dark => egui::Visuals::dark(),
            _dark_light::DarkLightMode::Light => egui::Visuals::light(),
        });
        self.data_events.extend(self.files.init());
    }
    pub(super) fn show(&mut self, ui: &mut egui::Ui) -> Vec<AppEvent> {
        self.data_events.extend(self.file_loader.check_progress());
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(3));
        {
            let events = self.check_dropped_files(ui);
            self.data_events.extend(events);
        }
        let mut app_events = Vec::new();
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                // dark/light mode switch
                {
                    /// Show small toggle-button for light and dark mode.
                    #[must_use]
                    fn light_dark_small_toggle_button(
                        is_dark_mode: bool,
                        ui: &mut egui::Ui,
                    ) -> Option<_dark_light::DarkLightMode> {
                        #![allow(clippy::collapsible_else_if)]
                        if is_dark_mode {
                            if ui
                                .add(egui::Button::new("☀").frame(false))
                                .on_hover_text("Switch to light mode")
                                .clicked()
                            {
                                ui.close_menu();
                                return Some(_dark_light::DarkLightMode::Light);
                            }
                        } else {
                            if ui
                                .add(egui::Button::new("🌙").frame(false))
                                .on_hover_text("Switch to dark mode")
                                .clicked()
                            {
                                ui.close_menu();
                                return Some(_dark_light::DarkLightMode::Dark);
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
                        app_events.push(AppEvent::CloseRequested);
                        ui.close_menu();
                    }
                }
            });
        });
        let state = &mut AppState {
            language: self.language,
            app_events: &mut app_events,
            limits: &mut self.limits,
            files: &mut self.files,
            data_events: &mut self.data_events,
            file_key_generator: &mut self.file_key_generator,
            total_filterings: &mut self.total_filterings,
            filtering: &mut self.filterings,
        };
        self.tabs.progress(state);

        egui_dock::DockArea::new(&mut self.tabs.tabs)
            .show_close_buttons(false)
            .show_inside(ui, state);
        while !self.data_events.is_empty() {
            let current_events = std::mem::take(&mut self.data_events);
            for event in current_events {
                let event = &event;
                self.check_for_file_loading(event);
                self.check_for_limit_event(event);
                self.data_events.extend(self.limits.notify(event));
                self.data_events.extend(self.files.notify(event));
                self.data_events.extend(self.tabs.notify(event));
            }
        }
        app_events
    }

    #[must_use]
    fn check_dropped_files(&mut self, ui: &mut egui::Ui) -> Vec<DataEvent> {
        let mut events = Vec::new();
        fn classify_dropped_file(key: FileKey, dropped: &egui::DroppedFile) -> files::FileEvent {
            match dropped {
                egui::DroppedFile {
                    path: Some(path),
                    name,
                    last_modified: None,
                    bytes: None,
                } if name.is_empty() => files::FileEvent::LoadFromPath {
                    key,
                    path: path.clone(),
                },
                egui::DroppedFile {
                    path: None,
                    name,
                    last_modified: _,
                    bytes: Some(bytes),
                } if name.is_empty() => files::FileEvent::ParseFromBytes {
                    key,
                    label: name.to_string(),
                    bytes: bytes.to_vec(),
                },
                _ => panic!("Unexpected dropped file"),
            }
        }
        ui.ctx().input(|i| {
            for dropped in &i.raw.dropped_files {
                events.push(DataEvent::File(classify_dropped_file(
                    self.file_key_generator.next(),
                    dropped,
                )));
            }
        });
        events
    }

    fn check_for_file_loading(&mut self, event: &DataEvent) {
        if let DataEvent::File(event) = event {
            match event {
                files::FileEvent::LoadFromPath { key, path } => {
                    self.files.insert(key.clone(), path.clone());
                    self.file_loader.load(key.clone(), path.clone());
                }
                files::FileEvent::ParseFromBytes { key, label, bytes } => {
                    self.files.make_parsing(key, label);
                    self.file_loader.parse(key.clone(), bytes.to_vec());
                }
                files::FileEvent::Remove(key) => self.files.remove(key),
                files::FileEvent::MoveUp(key) => self.files.move_up(key),
                files::FileEvent::MoveDown(key) => self.files.move_down(key),
                files::FileEvent::LoadError { key, msg } => {
                    self.files.make_loaderror(key, msg.clone())
                }
                files::FileEvent::Loaded {
                    key,
                    file: filedata,
                    non_conforming_tooltip,
                } => {
                    self.file_loaded(key, filedata, non_conforming_tooltip);
                }
                files::FileEvent::ToShow(_) => {}
                files::FileEvent::Label(_) => {}
            }
        }
    }

    fn file_loaded(
        &mut self,
        key: &FileKey,
        filedata: &files::FileData,
        non_conforming_tooltip: &Option<crate::LocalizableString>,
    ) {
        let Self {
            language: _,
            tabs: _,
            mode: _,
            limits,
            files,
            file_key_generator: _,
            limit_key_generator,
            file_loader: _,
            data_events,
            filterings,
            total_filterings,
        } = self;
        assert!(total_filterings
            .insert(
                key.clone(),
                vec![0; filedata.data_count()].into_boxed_slice(),
            )
            .is_none());

        let mut limit_sorting = HashMap::new();
        for (column, limit) in filedata.limits().enumerate() {
            let (is_new, limit_key) = limits.insert(limit_key_generator, limit);
            if is_new {
                data_events.push(DataEvent::Limit(limits::LimitEvent::New(limit_key.clone())))
            }
            let limit = limits.get(&limit_key).unwrap();
            let filtering = filedata.apply_limit(limit, column);

            for f in filtering
                .iter()
                .zip(total_filterings.get_mut(key).unwrap().iter_mut())
                .filter_map(|(&b, f)| b.then_some(f))
            {
                *f += 1;
            }
            assert!(filterings
                .insert((limit_key.clone(), key.clone()), filtering)
                .is_none());
            assert!(limit_sorting.insert(limit_key, column).is_none());
        }
        files.make_loaded(key, filedata, limit_sorting, non_conforming_tooltip);
    }

    fn check_for_limit_event(&mut self, event: &DataEvent) {
        if let DataEvent::Limit(event) = event {
            match event {
                limits::LimitEvent::ToShow(_) => {}
                limits::LimitEvent::Label(_) => {}
                limits::LimitEvent::Limit(limit_key) | limits::LimitEvent::New(limit_key) => {
                    let Self {
                        language: _,
                        tabs: _,
                        mode: _,
                        limits,
                        files,
                        file_key_generator: _,
                        limit_key_generator: _,
                        file_loader: _,
                        data_events,
                        filterings,
                        total_filterings,
                    } = self;
                    let mut changed = false;
                    if let Some(limit) = limits.get(limit_key) {
                        changed |=
                            compute_filters(files, limit_key, total_filterings, filterings, limit);
                    }
                    if changed {
                        data_events.push(DataEvent::Filtering)
                    }
                }
                limits::LimitEvent::RequestLabel(key, label) => {
                    if let Some(limit) = self.limits.get_mut(key) {
                        if limit.change_label(label) {
                            self.data_events
                                .push(DataEvent::Limit(limits::LimitEvent::Label(key.clone())));
                        }
                    }
                }
            }
        }
    }
}

#[must_use]
fn compute_filters(
    files: &mut files::FileContainer,
    limit_key: &LimitKey,
    total_filterings: &mut HashMap<FileKey, Box<[u32]>>,
    filterings: &mut HashMap<(LimitKey, FileKey), Vec<bool>>,
    limit: &limits::Limit,
) -> bool {
    let mut changed = false;
    for (file_key, (_, data, sorting)) in files.get_files_with_limit(limit_key) {
        if let Some(total_filtering) = total_filterings.get_mut(file_key) {
            if let Some(old_filtering) = filterings.get_mut(&(limit_key.clone(), file_key.clone()))
            {
                if let Some(column) = sorting.get(limit_key) {
                    let new_filtering = data.apply_limit(limit, *column);
                    debug_assert_eq!(old_filtering.len(), new_filtering.len());
                    debug_assert_eq!(old_filtering.len(), total_filtering.len());
                    total_filtering
                        .iter_mut()
                        .zip(new_filtering.iter())
                        .zip(old_filtering.iter())
                        .for_each(|((t, new), old)| match (new, old) {
                            (true, true) => {}
                            (true, false) => {
                                changed = true;
                                *t += 1;
                            }
                            (false, true) => {
                                changed = true;
                                *t -= 1;
                            }
                            (false, false) => {}
                        });
                    assert!(filterings
                        .insert((limit_key.clone(), file_key.clone()), new_filtering)
                        .is_some());
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
    changed
}

struct AppState<'a> {
    language: Language,
    app_events: &'a mut Vec<AppEvent>,
    limits: &'a mut limits::LimitContainer,
    files: &'a mut files::FileContainer,
    data_events: &'a mut Vec<DataEvent>,
    file_key_generator: &'a mut FileKeyGenerator,
    filtering: &'a mut HashMap<(LimitKey, FileKey), Filtering>,
    total_filterings: &'a mut HashMap<FileKey, Box<[u32]>>,
}

enum DataEvent {
    Limit(limits::LimitEvent),
    File(files::FileEvent),
    Filtering,
}
type DataEvents = Vec<DataEvent>;
trait DataEventNotifyable {
    fn progress(&mut self, state: &mut AppState);
    fn notify(&mut self, event: &DataEvent) -> Vec<DataEvent>;
}

impl<'a> AppState<'a> {
    fn next_file_key(&mut self) -> FileKey {
        self.file_key_generator.next()
    }

    fn get_files_for_limit<'b>(
        &'b self,
        limit_key: &'b LimitKey,
    ) -> impl Iterator<Item = &'b FileKey> + 'b {
        self.files.keys().filter(move |k: &&FileKey| {
            let k: &FileKey = k;
            self.filtering
                .keys()
                .any(|(l, ref f)| l == limit_key && f == k)
        })
    }
}

impl<'a> egui_dock::TabViewer for AppState<'a> {
    type Tab = _tabs::Tab;

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
    Reset,
}
