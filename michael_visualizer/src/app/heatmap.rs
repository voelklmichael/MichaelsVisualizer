use crate::LocalizableStr;

use super::DataEvent;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct HeatmapTab {}
impl super::DataEventNotifyable for HeatmapTab {
    fn notify(&mut self, event: &DataEvent) -> Vec<DataEvent> {
        Default::default()
    }

    fn progress(&mut self, state: &mut super::AppState) {}
}
impl super::TabTrait for HeatmapTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr { english: "Heatmap" }.localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {}
}
