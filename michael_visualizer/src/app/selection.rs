use crate::LocalizableStr;

use super::DataEvent;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct SelectionTab {}
impl super::DataEventNotifyable for SelectionTab {
    fn notify(&mut self, _event: &DataEvent) -> Vec<DataEvent> {
        Default::default()
    }

    fn progress(&mut self, _state: &mut super::AppState) {}
}
impl super::TabTrait for SelectionTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr {
            english: "Selection",
         }
        .localize(state.language)
    }

    fn show(&mut self, _state: &mut super::AppState, _ui: &mut egui::Ui) {}
}
