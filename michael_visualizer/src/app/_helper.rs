pub(crate) fn galleys(
    labels: Vec<&str>,
    ui: &mut egui::Ui,
    fontsize: f32,
) -> Vec<std::sync::Arc<egui::Galley>> {
    let font_color = egui::Color32::BLACK;
    let font_id = egui::FontId::proportional(fontsize);
    labels
        .into_iter()
        .map(|label| {
            ui.painter()
                .layout_no_wrap(label.into(), font_id.clone(), font_color)
        })
        .collect()
}
