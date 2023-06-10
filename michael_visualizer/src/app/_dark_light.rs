#[derive(PartialEq, Default, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub(crate) enum DarkLightMode {
    Dark,
    #[default]
    Light,
}

impl DarkLightMode {
    pub(crate) fn visuals(&self) -> egui::Visuals {
        match self {
            DarkLightMode::Dark => egui::Visuals::dark(),
            DarkLightMode::Light => egui::Visuals::light(),
        }
    }
}
