mod file_data;
use crate::{
    data_types::{FileKey, FileLabel},
    Language, LocalizableStr, LocalizableString,
};
pub(super) use file_data::{DataColumn, FileData};

pub(super) enum FileEvent {
    LoadFromPath {
        key: FileKey,
        path: std::path::PathBuf,
    },
    ParseFromBytes {
        key: FileKey,
        label: String,
        bytes: Vec<u8>,
    },
    ToShow(FileKey),
    Remove(FileKey),
    MoveUp(FileKey),
    MoveDown(FileKey),
    Label(FileKey),
    LoadError {
        key: FileKey,
        msg: LocalizableString,
    },
    Loaded {
        key: FileKey,
        file: file_data::FileData,
        non_conforming_tooltip: Option<LocalizableString>,
    },
}

pub enum FileRequest {
    Hide(crate::data_types::FileKey),
    ShowAll,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(super) struct FileContainer {
    files: indexmap::IndexMap<FileKey, File>,
}
impl FileContainer {
    pub(super) fn insert(&mut self, key: FileKey, path: std::path::PathBuf) {
        let _ = self.files.insert(key, File::from_path(path));
    }

    pub(super) fn make_parsing(&mut self, key: &FileKey, label: &str) {
        if let Some(entry) = self.files.get_mut(key) {
            if entry.state.is_loading() {
                entry.state = FileState::Parsing;
            }
        } else {
            self.files.insert(key.clone(), File::from_label(label));
        }
    }

    fn move_up_or_down(&mut self, key: &FileKey, steps: isize) {
        if let Some(from) = self.files.get_index_of(key) {
            let to = from as isize + steps;
            if to >= 0 && to < self.files.len() as isize {
                self.files.move_index(from, to as usize);
            }
        }
    }
    pub(super) fn move_down(&mut self, key: &FileKey) {
        self.move_up_or_down(key, 1);
    }

    pub(super) fn move_up(&mut self, key: &FileKey) {
        self.move_up_or_down(key, -1);
    }

    pub(super) fn remove(&mut self, key: &FileKey) {
        self.files.shift_remove(key);
    }
    pub(super) fn make_loaded(
        &mut self,
        key: &FileKey,
        filedata: &FileData,
        limit_sorting: std::collections::HashMap<crate::data_types::LimitKey, usize>,
        non_conforming_tooltip: &Option<LocalizableString>,
    ) {
        if let Some(file) = self.files.get_mut(key) {
            file.state = FileState::Loaded {
                file: filedata.clone(),
                limit_sorting,
                non_conforming_tooltip: non_conforming_tooltip.clone(),
            };
        }
    }

    pub(super) fn make_loaderror(&mut self, key: &FileKey, msg: LocalizableString) {
        if let Some(file) = self.files.get_mut(key) {
            file.state = FileState::Error(msg);
        }
    }

    pub(super) fn init(&mut self) -> Vec<super::DataEvent> {
        let mut events = Vec::new();
        for (key, file) in std::mem::take(&mut self.files).into_iter() {
            if let Some(path) = file.original_path {
                events.push(super::DataEvent::File(
                    super::files::FileEvent::LoadFromPath { key, path },
                ))
            }
        }
        events
    }

    pub(super) fn get_files_with_limit<'a>(
        &'a self,
        limit_key: &'a crate::data_types::LimitKey,
    ) -> impl Iterator<
        Item = (
            &'a FileKey,
            (
                &'a FileLabel,
                &'a FileData,
                &'a std::collections::HashMap<crate::data_types::LimitKey, usize>,
            ),
        ),
    > + 'a {
        self.files
            .iter()
            .flat_map(|(key, f)| f.get_loaded().map(|f| (key, f)))
            .filter(move |(_, (_, _, sorting))| sorting.contains_key(limit_key))
    }
    pub(crate) fn get(&self, key: &FileKey) -> Option<&File> {
        self.files.get(key)
    }

    pub(super) fn keys(&self) -> impl Iterator<Item = &FileKey> {
        self.files.keys()
    }

    pub(crate) fn iter_loaded(
        &self,
    ) -> impl Iterator<
        Item = (
            &FileKey,
            (
                &FileLabel,
                &FileData,
                &std::collections::HashMap<crate::data_types::LimitKey, usize>,
            ),
        ),
    > {
        self.files
            .iter()
            .flat_map(|(key, f)| f.get_loaded().map(|f| (key, f)))
    }
}
impl super::DataEventNotifyable for FileContainer {
    fn notify(&mut self, event: &super::DataEvent) -> Vec<super::DataEvent> {
        let mut events = Vec::default();
        match event {
            super::DataEvent::LimitRequest(_) => {}
            super::DataEvent::Limit(_) => {}
            super::DataEvent::File(_) => {}
            super::DataEvent::Filtering => {}
            super::DataEvent::FileRequest(event) => match event {
                FileRequest::Hide(key) => {
                    if let Some(file) = self.files.get_mut(key) {
                        if file.to_show {
                            file.to_show = false;
                            events.push(super::DataEvent::File(FileEvent::ToShow(key.clone())));
                        }
                    }
                }
                FileRequest::ShowAll => {
                    for (key, file) in self.files.iter_mut() {
                        if !file.to_show {
                            file.to_show = true;
                            events.push(super::DataEvent::File(FileEvent::ToShow(key.clone())));
                        }
                    }
                }
            },
            super::DataEvent::SelectionRequest(_) => {}
            super::DataEvent::SelectionEvent(_) => {}
        }
        events
    }

    fn progress(&mut self, _state: &mut super::AppState) {}
}

#[derive(Default)]
pub(super) enum FileState {
    #[default]
    Loading,
    Parsing,
    Error(LocalizableString),
    Loaded {
        file: file_data::FileData,
        limit_sorting: std::collections::HashMap<crate::data_types::LimitKey, usize>,
        non_conforming_tooltip: Option<LocalizableString>,
    },
}
impl FileState {
    fn is_loading(&self) -> bool {
        matches!(self, FileState::Loading)
    }

    fn tooltip(&self) -> LocalizableStr {
        match self {
            FileState::Loading => LocalizableStr { english: "Loading" },
            FileState::Parsing => LocalizableStr { english: "Parsing" },
            FileState::Error(msg) => msg.as_str(),
            FileState::Loaded {
                file,
                limit_sorting: _,
                non_conforming_tooltip,
            } => non_conforming_tooltip
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or(file.tooltip()),
        }
    }

    fn get_loaded(
        &self,
    ) -> Option<(
        &file_data::FileData,
        &std::collections::HashMap<crate::data_types::LimitKey, usize>,
    )> {
        match self {
            FileState::Loading => None,
            FileState::Parsing => None,
            FileState::Error(_) => None,
            FileState::Loaded {
                file: a,
                limit_sorting: b,
                non_conforming_tooltip: _,
            } => Some((a, b)),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(super) struct File {
    original_path: Option<std::path::PathBuf>,
    original_label: FileLabel,
    label: FileLabel,
    label_before: FileLabel,
    to_show: bool,
    #[serde(skip)]
    state: FileState,
}
impl File {
    #[must_use]
    fn show(
        &mut self,
        key: &FileKey,
        ui: &mut egui::Ui,
        language: Language,
    ) -> Vec<super::DataEvent> {
        let File {
            original_path: _,
            to_show,
            label,
            label_before,
            original_label,
            state,
        } = self;
        let mut events = Vec::new();
        ui.scope(|ui| {
            // adjust style
            {
                let visuals = &mut ui.style_mut().visuals;
                use egui::Color32;
                if let Some((text, bg)) = match state {
                    FileState::Loaded {
                        file: _,
                        limit_sorting: _,
                        non_conforming_tooltip,
                    } => non_conforming_tooltip
                        .as_mut()
                        .map(|_| (Color32::WHITE, Color32::KHAKI)),
                    FileState::Loading => Some((Color32::BLACK, Color32::LIGHT_BLUE)),
                    FileState::Parsing => Some((Color32::WHITE, Color32::DARK_BLUE)),
                    FileState::Error(_) => Some((Color32::WHITE, Color32::RED)),
                } {
                    visuals.extreme_bg_color = bg;
                    visuals.faint_bg_color = bg;
                    visuals.override_text_color = Some(text);
                }
            }
            let tooltip = state.tooltip().localize(language);
            // to show
            {
                let before = *to_show;
                ui.checkbox(to_show, "")
                    .on_hover_text(tooltip)
                    .context_menu(|ui| {
                        context_menu_entries(ui, language, &mut events, key);
                    });
                if before != *to_show {
                    events.push(super::DataEvent::File(FileEvent::ToShow(key.clone())));
                }
            }
            // label
            {
                let mut reset_requested = false;
                ui.text_edit_singleline(label.get_mut())
                    .on_hover_text(tooltip)
                    .context_menu(|ui| {
                        context_menu_entries(ui, language, &mut events, key);
                        if ui.button(super::RESET.localize(language)).clicked() {
                            reset_requested = true;
                        }
                    });
                if reset_requested {
                    *label = original_label.clone();
                    *label_before = original_label.clone();
                    events.push(super::DataEvent::File(FileEvent::Label(key.clone())));
                }
                if label_before != label {
                    events.push(super::DataEvent::File(FileEvent::Label(key.clone())));
                    *label_before = label.clone();
                }
            }
        });
        events
    }

    fn from_path(path: std::path::PathBuf) -> File {
        let label: FileLabel = path
            .as_path()
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
            .to_string()
            .into();
        File {
            original_path: Some(path),
            original_label: label.clone(),
            label: label.clone(),
            label_before: label,
            to_show: true,
            state: FileState::Loading,
        }
    }

    fn from_label(label: &str) -> File {
        let label: FileLabel = label.to_string().into();
        File {
            original_path: None,
            original_label: label.clone(),
            label: label.clone(),
            label_before: label,
            to_show: true,
            state: FileState::Loading,
        }
    }
    pub(super) fn get_loaded(
        &self,
    ) -> Option<(
        &FileLabel,
        &file_data::FileData,
        &std::collections::HashMap<crate::data_types::LimitKey, usize>,
    )> {
        let Self {
            original_path: _,
            original_label: _,
            label,
            label_before: _,
            to_show,
            state,
        } = self;
        if !*to_show {
            None
        } else {
            state.get_loaded().map(|(f, s)| (label, f, s))
        }
    }
}

fn context_menu_entries(
    ui: &mut egui::Ui,
    language: Language,
    events: &mut Vec<super::DataEvent>,
    key: &FileKey,
) {
    if ui
        .button(LocalizableStr { english: "Remove" }.localize(language))
        .clicked()
    {
        events.push(super::DataEvent::File(FileEvent::Remove(key.clone())));
        ui.close_menu();
    }
    if ui
        .button(LocalizableStr { english: "Move up" }.localize(language))
        .clicked()
    {
        events.push(super::DataEvent::File(FileEvent::MoveUp(key.clone())));
        ui.close_menu();
    }
    if ui
        .button(
            LocalizableStr {
                english: "Move Down",
            }
            .localize(language),
        )
        .clicked()
    {
        events.push(super::DataEvent::File(FileEvent::MoveDown(key.clone())));
        ui.close_menu();
    }
}
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct FileTab {}

impl super::TabTrait for FileTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr { english: "Files" }.localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        if ui
            .button(
                LocalizableStr {
                    english: "Load File",
                }
                .localize(state.language),
            )
            .clicked()
        {
            //TODO: title, extension/filter, …
            if let Some(files) = rfd::FileDialog::new().pick_files() {
                for path in files {
                    let key = state.next_file_key();
                    state
                        .data_events
                        .push(super::DataEvent::File(FileEvent::LoadFromPath {
                            key,
                            path,
                        }))
                }
            }
        }
        let super::AppState {
            language,
            files: FileContainer { files },
            data_events,
            ..
        } = state;
        for (key, file) in files.iter_mut() {
            ui.horizontal(|ui| {
                data_events.extend(file.show(key, ui, *language));
            });
        }
    }
}
