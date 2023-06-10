use crate::LocalizableStr;

use super::DataEvent;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct SelectionTab {}
impl super::DataEventNotifyable for SelectionTab {
    fn notify(&mut self, event: &DataEvent) -> Vec<DataEvent> {
        Default::default()
    }

    fn progress(&mut self, state:&mut super::AppState) {
        
    }
}
impl super::TabTrait for SelectionTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr {
            english: "Selection",
        }
        .localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {}
}
