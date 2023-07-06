mod _dark_light;
mod _helper;
mod _tabs;
mod dummy;
mod file_loader;
mod files;
mod heatmap;
mod limits;
mod plot;
mod selection;
mod violinplot;
mod distribution;

use std::collections::HashMap;

use crate::{
    data_types::{FileKey, FileKeyGenerator, LimitKey},
    Language, LocalizableStr,
};

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
enum LockableLimitKey {
    Locked(usize),
    Single(LimitKey),
}

impl LockableLimitKey {
    fn get<'a>(&'a self, locked_limits: &'a [LimitKey]) -> (Option<usize>, Option<&'a LimitKey>) {
        match self {
            LockableLimitKey::Locked(index) => (Some(*index), locked_limits.get(*index)),
            LockableLimitKey::Single(key) => (None, Some(key)),
        }
    }

    /// Check if instance is locked
    /// If index is given, it is also checked if the index matches, otherwise only variant is checked
    fn is_locked(&self, index: Option<&usize>) -> bool {
        match self {
            LockableLimitKey::Locked(i) => {
                if let Some(index) = index {
                    index == i
                } else {
                    true
                }
            }
            LockableLimitKey::Single(_) => false,
        }
    }

    fn update(&mut self, value: LimitKey, locked_limits: &mut Vec<LimitKey>) -> Option<usize> {
        match self {
            LockableLimitKey::Locked(index) => {
                while locked_limits.len() <= *index {
                    locked_limits.push(value.clone());
                }
                let key = locked_limits.get_mut(*index).unwrap();
                *key = value;
                Some(*index)
            }
            LockableLimitKey::Single(key) => {
                *key = value;
                None
            }
        }
    }
}
impl Default for LockableLimitKey {
    fn default() -> Self {
        Self::Locked(0)
    }
}

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
    selected: Option<selection::Selection>,
    file_key_generator: crate::data_types::FileKeyGenerator,
    limit_key_generator: crate::data_types::LimitKeyGenerator,
    file_loader: file_loader::FileLoader,
    #[serde(skip)]
    data_events: Vec<DataEvent>,
    #[serde(skip)]
    filterings: HashMap<(LimitKey, FileKey), Filtering>,
    #[serde(skip)]
    total_filterings: HashMap<FileKey, Box<[u32]>>,
    locked_limits: Vec<LimitKey>,
    #[serde(skip)]
    requested_screenshot: Option<egui::Rect>,
}

impl App {
    pub(super) fn init(&mut self, cc: &eframe::CreationContext) {
        let mut kinds = _tabs::TabKind::kinds();
        for kind in self.tabs.tabs.tabs().map(|(_, x)| x.kind()) {
            if let Some(index) = kinds.iter().position(|&x| kind == x) {
                kinds.remove(index);
            }
        }
        for kind in kinds {
            self.tabs.push(kind);
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
            ui.menu_button(
                LocalizableStr { english: "File" }.localize(self.language),
                |ui| {
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
                        let new_visuals =
                            light_dark_small_toggle_button(style.visuals.dark_mode, ui);
                        if let Some(mode) = new_visuals {
                            self.mode = mode;
                            ui.ctx().set_visuals(mode.visuals());
                        }
                    }
                    // quit button
                    {
                        if ui
                            .button(LocalizableStr { english: "Quit" }.localize(self.language))
                            .clicked()
                        {
                            app_events.push(AppEvent::CloseRequested);
                            ui.close_menu();
                        }
                    }
                },
            );
            ui.menu_button(
                LocalizableStr { english: "Tabs" }.localize(self.language),
                |ui| {
                    for (label, tab) in [
                        (LocalizableStr { english: "Files" }, _tabs::TabKind::Files),
                        (LocalizableStr { english: "Limits" }, _tabs::TabKind::Limit),
                        (
                            LocalizableStr {
                                english: "Distribution",
                            },
                            _tabs::TabKind::Violinplot,
                        ),
                        (
                            LocalizableStr { english: "Heatmap" },
                            _tabs::TabKind::Heatmap,
                        ),
                        (
                            LocalizableStr {
                                english: "Selection",
                            },
                            _tabs::TabKind::Selection,
                        ),
                        (LocalizableStr { english: "Plot" }, _tabs::TabKind::Plot),
                    ] {
                        if ui.button(label.localize(self.language)).clicked() {
                            self.tabs.push(tab);
                            ui.close_menu();
                        }
                    }
                },
            );
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
            locked_limits: &mut self.locked_limits,
            selected: &mut self.selected,
            requested_screenshot: &mut self.requested_screenshot,
        };
        self.tabs.progress(state);

        egui_dock::DockArea::new(&mut self.tabs.tabs)
            //.scroll_area_in_tabs(false)
            .show_close_buttons(true)
            .show_inside(ui, state);
        while !self.data_events.is_empty() {
            let current_events = std::mem::take(&mut self.data_events);
            for event in current_events {
                let event = &event;
                self.check_for_file_loading(event);
                self.check_for_limit_event(event);
                self.check_for_selection_event(event);
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
            locked_limits,
            selected: _,
            requested_screenshot: _,
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
                if locked_limits.is_empty() {
                    locked_limits.push(limit_key.clone());
                }
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
        match event {
            DataEvent::Limit(event) => match event {
                limits::LimitEvent::LockableLimit(_) => {}
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
                        locked_limits: _,
                        selected: _,
                        requested_screenshot: _,
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
            },
            DataEvent::LimitRequest(event) => match event {
                limits::LimitRequest::RequestLabel(key, label) => {
                    if let Some(limit) = self.limits.get_mut(key) {
                        if limit.change_label(label) {
                            self.data_events
                                .push(DataEvent::Limit(limits::LimitEvent::Label(key.clone())));
                        }
                    }
                }
                limits::LimitRequest::ShowRectangle {
                    x_key,
                    y_key,
                    rectangle,
                } => {
                    if let Some(limit) = self.limits.get_mut(x_key) {
                        limit.change(rectangle.left_top.x, rectangle.right_bottom.x - 1);
                        self.data_events
                            .push(DataEvent::Limit(limits::LimitEvent::Limit(x_key.clone())));
                    }
                    if let Some(limit) = self.limits.get_mut(y_key) {
                        limit.change(rectangle.left_top.y, rectangle.right_bottom.y - 1);
                        self.data_events
                            .push(DataEvent::Limit(limits::LimitEvent::Limit(y_key.clone())));
                    }
                }
            },
            _ => (),
        }
    }
    fn check_for_selection_event(&mut self, event: &DataEvent) {
        if let DataEvent::SelectionRequest(selection) = event {
            match selection {
                selection::SelectionRequest::UnselectAll => {
                    self.selected = None;
                    self.data_events.push(DataEvent::SelectionEvent(
                        selection::SelectionEvent::UnselectAll,
                    ))
                }
                selection::SelectionRequest::Selection(selection) => {
                    self.selected = Some(selection.clone());
                    self.data_events.push(DataEvent::SelectionEvent(
                        selection::SelectionEvent::Selection(selection.clone()),
                    ))
                }
            }
        }
    }

    pub(crate) fn request_screenshot(&mut self, frame: &mut eframe::Frame) -> Option<egui::Rect> {
        if let Some(rect) = self.requested_screenshot.take() {
            frame.request_screenshot();
            Some(rect)
        } else {
            None
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
    selected: &'a mut Option<selection::Selection>,
    data_events: &'a mut Vec<DataEvent>,
    file_key_generator: &'a mut FileKeyGenerator,
    filtering: &'a mut HashMap<(LimitKey, FileKey), Filtering>,
    total_filterings: &'a mut HashMap<FileKey, Box<[u32]>>,
    locked_limits: &'a mut Vec<LimitKey>,
    requested_screenshot: &'a mut Option<egui::Rect>,
}

enum DataEvent {
    LimitRequest(limits::LimitRequest),
    Limit(limits::LimitEvent),
    File(files::FileEvent),
    Filtering,
    FileRequest(files::FileRequest),
    SelectionRequest(selection::SelectionRequest),
    SelectionEvent(selection::SelectionEvent),
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

    #[must_use]
    fn ui_selectable_limit(&mut self, ui: &mut egui::Ui, to_show: &mut LockableLimitKey) -> bool {
        let mut needs_recompute = false;
        ui.horizontal(|ui| {
            let axis_selection_text = LocalizableStr {
                english: "Select limit",
            }
            .localize(self.language);
            ui.label(axis_selection_text);
            let (is_locked, value) = to_show.get(self.locked_limits);
            let mut value = value.cloned();
            let selected_label = if let Some(key) = value.as_ref() {
                if let Some(limit) = self.limits.get(key) {
                    format!(
                        "{} {}",
                        limit.get_label().as_str(),
                        if let Some(index) = is_locked {
                            format!("\u{1F512}: {index}")
                        } else {
                            "\u{1F513}".to_string()
                        }
                    )
                } else {
                    axis_selection_text.to_string()
                }
            } else {
                axis_selection_text.to_string()
            };

            if self.limits.is_empty() {
                ui.label(
                    LocalizableStr {
                        english: "No limits available",
                    }
                    .localize(self.language),
                );
            } else {
                egui::ComboBox::from_id_source(axis_selection_text)
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        for (key, limit) in self.limits.iter() {
                            let previous = value.clone();
                            ui.selectable_value(
                                &mut value,
                                Some(key.clone()),
                                limit.get_label().as_str(),
                            );
                            if previous != value {
                                needs_recompute = true;
                                if let Some(index) =
                                    to_show.update(value.clone().unwrap(), self.locked_limits)
                                {
                                    self.data_events.push(DataEvent::Limit(
                                        limits::LimitEvent::LockableLimit(index),
                                    ))
                                }
                            }
                        }
                    })
                    .response
                    .context_menu(|ui| {
                        #[must_use]
                        fn context_menu(
                            state: &mut AppState,
                            to_show: &mut LockableLimitKey,
                            ui: &mut egui::Ui,
                        ) -> bool {
                            let mut new = None;
                            let mut needs_recompute = false;
                            match to_show {
                                LockableLimitKey::Locked(index) => {
                                    let key = if let Some(key) = state.locked_limits.get(*index) {
                                        key.clone()
                                    } else {
                                        ui.close_menu();
                                        return needs_recompute;
                                    };
                                    if ui.button("\u{1F513}").clicked() {
                                        if let Some(key) = state.locked_limits.get(*index) {
                                            new = Some(LockableLimitKey::Single(key.clone()));
                                        }
                                        ui.close_menu();
                                    }
                                    ui.label(
                                        LocalizableStr { english: "Locking" }
                                            .localize(state.language),
                                    );
                                    for i in 0..state.locked_limits.len() {
                                        if ui
                                            .add_enabled(
                                                index != &i,
                                                egui::Button::new(&format!("\u{1F512}: {i}")),
                                            )
                                            .clicked()
                                        {
                                            if i == state.locked_limits.len() {
                                                state.locked_limits[i] = key.clone();
                                            }
                                            new = Some(LockableLimitKey::Locked(i));
                                            needs_recompute = true;
                                            ui.close_menu();
                                        }
                                    }
                                    ui.separator();
                                }
                                LockableLimitKey::Single(key) => {
                                    ui.label(
                                        LocalizableStr { english: "Locking" }
                                            .localize(state.language),
                                    );
                                    for i in 0..(state.locked_limits.len() + 1) {
                                        if ui.button(&format!("\u{1F512}: {i}")).clicked() {
                                            if i == state.locked_limits.len() {
                                                state.locked_limits.push(key.clone());
                                            }
                                            new = Some(LockableLimitKey::Locked(i));
                                            needs_recompute = true;
                                            ui.close_menu();
                                        }
                                    }
                                    ui.separator();
                                }
                            }
                            if let Some(new) = new {
                                *to_show = new;
                            }
                            needs_recompute
                        }
                        needs_recompute |= context_menu(self, to_show, ui);
                    });
            }
        });
        needs_recompute
    }

    #[must_use]
    fn ui_coloring_limit(
        &mut self,
        ui: &mut egui::Ui,
        to_color: &mut Option<LockableLimitKey>,
    ) -> bool {
        let mut needs_recompute = false;
        ui.horizontal(|ui| {
            let coloring_selection_text = LocalizableStr {
                english: "Select coloring limit",
            }
            .localize(self.language);
            let (selected_label, mut value) = if let Some(to_color) = to_color {
                ui.label(coloring_selection_text);

                let (is_locked, value) = to_color.get(self.locked_limits);
                let value = value.cloned();
                if let Some(key) = value.as_ref() {
                    let text = if let Some(limit) = self.limits.get(key) {
                        format!(
                            "{} {}",
                            limit.get_label().as_str(),
                            if let Some(index) = is_locked {
                                format!("\u{1F512}: {index}")
                            } else {
                                "\u{1F513}".to_string()
                            }
                        )
                    } else {
                        coloring_selection_text.to_string()
                    };
                    (text, value)
                } else {
                    (coloring_selection_text.to_string(), None)
                }
            } else {
                (coloring_selection_text.to_string(), None)
            };
            if !self.limits.iter().any(|(_, x)| x.is_int()) {
                ui.label(
                    LocalizableStr {
                        english: "No integer limits for coloring available",
                    }
                    .localize(self.language),
                );
            } else {
                egui::ComboBox::from_id_source(coloring_selection_text)
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        {
                            let previous: Option<LimitKey> = value.clone();
                            ui.selectable_value(
                                &mut value,
                                None,
                                LocalizableStr {
                                    english: "no coloring",
                                }
                                .localize(self.language),
                            );
                            if previous != value {
                                needs_recompute = true;
                                *to_color = None;
                            }
                        }
                        for (key, limit) in self.limits.iter().filter(|(_, x)| x.is_int()) {
                            let previous: Option<LimitKey> = value.clone();
                            ui.selectable_value(
                                &mut value,
                                Some(key.clone()),
                                limit.get_label().as_str(),
                            );
                            if previous != value {
                                needs_recompute = true;
                                if let Some(to_color) = to_color {
                                    if let Some(index) =
                                        to_color.update(value.clone().unwrap(), self.locked_limits)
                                    {
                                        self.data_events.push(DataEvent::Limit(
                                            limits::LimitEvent::LockableLimit(index),
                                        ))
                                    }
                                } else {
                                    *to_color=Some(LockableLimitKey::Single(key.clone()));
                                }                                
                            }
                        }
                    })
                    .response
                    .context_menu(|ui| {
                        fn context_menu(
                            state: &mut AppState,
                            to_show: &mut LockableLimitKey,
                            ui: &mut egui::Ui,
                        ) {
                            let mut new = None;
                            match to_show {
                                LockableLimitKey::Locked(index) => {
                                    let key = if let Some(key) = state.locked_limits.get(*index) {
                                        key.clone()
                                    } else {
                                        ui.close_menu();
                                        return;
                                    };
                                    if ui.button("\u{1F513}").clicked() {
                                        if let Some(key) = state.locked_limits.get(*index) {
                                            new = Some(LockableLimitKey::Single(key.clone()));
                                        }
                                        ui.close_menu();
                                    }
                                    ui.label(
                                        LocalizableStr { english: "Locking" }
                                            .localize(state.language),
                                    );
                                    for i in 0..state.locked_limits.len() {
                                        if ui
                                            .add_enabled(
                                                index != &i,
                                                egui::Button::new(&format!("\u{1F512}: {i}")),
                                            )
                                            .clicked()
                                        {
                                            if i == state.locked_limits.len() {
                                                state.locked_limits[i] = key.clone();
                                            }
                                            new = Some(LockableLimitKey::Locked(i));
                                            ui.close_menu();
                                        }
                                    }
                                    ui.separator();
                                }
                                LockableLimitKey::Single(key) => {
                                    ui.label(
                                        LocalizableStr { english: "Locking" }
                                            .localize(state.language),
                                    );
                                    for i in 0..(state.locked_limits.len() + 1) {
                                        if ui.button(&format!("\u{1F512}: {i}")).clicked() {
                                            if i == state.locked_limits.len() {
                                                state.locked_limits.push(key.clone());
                                            }
                                            new = Some(LockableLimitKey::Locked(i));
                                            ui.close_menu();
                                        }
                                    }
                                    ui.separator();
                                }
                            }
                            if let Some(new) = new {
                                *to_show = new;
                            }
                        }
                        if let Some(to_color) = to_color {
                            context_menu(self, to_color, ui);
                        }
                    });
            }
        });
        needs_recompute
    }

    fn request_screenshot(&mut self, rect: egui::Rect) {
        *self.requested_screenshot = Some(rect)
    }

    fn get_color(&self, index: usize) -> egui::Color32 {
        let colors = egui_heatmap::colors::DISTINGUISHABLE_COLORS;
        let i = index % colors.len();
        colors[i]
    }
}

#[derive(Hash, serde::Deserialize, serde::Serialize, PartialEq)]
struct TabId(usize);
impl TabId {
    fn new(id: usize) -> Self {
        Self(id)
    }

    fn to_egui_id(&self) -> egui::Id {
        egui::Id::new(self.0)
    }
}

impl<'a> egui_dock::TabViewer for AppState<'a> {
    type Tab = (TabId, _tabs::Tab);

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, (_, tab): &mut Self::Tab) {
        tab.show(self, ui);
    }

    fn title(&mut self, (_, tab): &mut Self::Tab) -> egui_dock::egui::WidgetText {
        tab.title(self).into()
    }
    fn id(&mut self, (id, _): &mut Self::Tab) -> egui::Id {
        id.to_egui_id()
    }
}
pub(super) enum AppEvent {
    CloseRequested,
    Dialog(crate::dialog::Dialog),
    Reset,
}
