use michael_visualizer_basic::SimpleFileKey;

use crate::LocalizableStr;

#[derive(serde::Deserialize, serde::Serialize)]
pub(super) struct File;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct FileTab {
    files: Vec<(SimpleFileKey, File)>,
}

impl super::TabTrait for FileTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr { english: "Files" }.localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        if ui.button("Add Demo File").clicked() {
            state
                .data_events
                .push(michael_visualizer_basic::DataEvent::File(
                    michael_visualizer_basic::FileEvent::ToLoad2("ai".to_string().into(), File),
                ));
        }
        egui::Grid::new(self.title(state)).show(ui, |ui| {});
    }
}

impl michael_visualizer_basic::FileTrait for File {
    type Limit = super::limits::Limit;

    fn limits(&self) -> &[Self::Limit] {
        todo!()
    }

    fn row_count(&self) -> usize {
        todo!()
    }

    fn apply_limit(&mut self, limit_index: usize, limit: &Self::Limit) -> Vec<bool> {
        todo!()
    }
}
