#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(super) struct DummyTab;

impl super::TabTrait for DummyTab {
    fn title(&self, _state: &super::AppState) -> &str {
        "Dummy"
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        use crate::dialog::Dialog;
        if ui.button("Dialog Progress").clicked() {
            state
                .app_events
                .push(super::AppEvent::Dialog(Dialog::example_progress(
                    true,
                    Some(std::time::Duration::from_secs(5)),
                )));
        }
        for i in 1..4 {
            if ui.button("Dialog Buttons").clicked() {
                state
                    .app_events
                    .push(super::AppEvent::Dialog(Dialog::example_button(i)));
            }
        }
    }
}
