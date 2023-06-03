use michael_visualizer_basic::LimitLabel;

use crate::{Language, LocalizableStr, LocalizableString};

static RESET: LocalizableStr<'static> = LocalizableStr { english: "Reset" };
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct LimitTab {
    to_show: Option<usize>,
    limits: Vec<(super::SimpleLimitKey, Limit)>,
}
impl LimitTab {
    #[must_use]
    fn show_label(
        label: &mut String,
        label_previous: &mut String,
        ui: &mut egui::Ui,
        language: Language,
        original_label: &str,
        info: &str,
    ) -> bool {
        ui.text_edit_singleline(label)
            .on_hover_text(format!(
                "{info}\n{original_header}: {original_label}",
                original_header = LocalizableStr {
                    english: "Original label"
                }
                .localize(language)
            ))
            .context_menu(|ui| {
                if ui.button(RESET.localize(language)).clicked() {
                    *label = original_label.to_string().into();
                }
            });
        if label != label_previous {
            *label_previous = label.clone();
            true
        } else {
            false
        }
    }
}
impl super::TabTrait for LimitTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr { english: "Limits" }.localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        egui::Grid::new(self.title(state))
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                for (index, (key, limit)) in self.limits.iter_mut().enumerate() {
                    let previous = self.to_show;
                    ui.radio_value(&mut self.to_show, Some(index), "");
                    if previous != self.to_show {
                        state
                            .data_events
                            .push(michael_visualizer_basic::DataEvent::Limit(
                                michael_visualizer_basic::LimitEvent::ToPlot(*key),
                            ));
                    }
                    if Self::show_label(
                        limit.label.get_mut(),
                        limit.label_previous.get_mut(),
                        ui,
                        state.language,
                        &limit.original_label,
                        &limit.tooltip_original,
                    ) {
                        state
                            .data_events
                            .push(michael_visualizer_basic::DataEvent::Limit(
                                michael_visualizer_basic::LimitEvent::Label(
                                    *key,
                                    limit.label.clone(),
                                ),
                            ))
                    }
                    let mut changed = limit
                        .lower
                        .show(ui, &limit.tooltip_original, state.language);
                    changed |= limit
                        .upper
                        .show(ui, &limit.tooltip_original, state.language);
                    if changed {
                        state
                            .data_events
                            .push(michael_visualizer_basic::DataEvent::Limit(
                                michael_visualizer_basic::LimitEvent::Value(*key, limit.clone()),
                            ))
                    }
                    ui.end_row();
                }
            });
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Limit {
    original_label: String,
    label: LimitLabel,
    label_previous: LimitLabel,
    tooltip_original: String,
    lower: LimitValue,
    upper: LimitValue,
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct LimitValue {
    value: Option<f32>,
    value_original: Option<f32>,
    parse_issue: bool,
    parsed: String,
    current: String,
    tooltip: String,
}
impl LimitValue {
    fn show(&mut self, ui: &mut egui::Ui, tooltip_original: &str, language: Language) -> bool {
        let mut reset_requested = false;
        let LimitValue {
            value,
            value_original,
            parse_issue,
            parsed,
            current,
            tooltip,
        } = self;
        ui.scope(|ui| {
            let text = egui::TextEdit::singleline(current);
            if *parse_issue {
                let text = text.text_color(egui::Color32::WHITE);
                ui.visuals_mut().extreme_bg_color = egui::Color32::RED;
                text.show(ui)
            } else {
                text.show(ui)
            }
            .response
            .on_hover_text(tooltip.as_str())
            .context_menu(|ui| {
                if ui.button(RESET.localize(language)).clicked() {
                    reset_requested = true;
                    ui.close_menu();
                }
            });
        });
        if reset_requested {
            self.reset(language, tooltip_original);
            return true;
        }
        if current == parsed {
            false
        } else {
            *parsed = current.to_string();
            let parsed = parsed.trim();
            let parsed = if parsed.is_empty() {
                Ok(None)
            } else if let Ok(f) = parsed.parse() {
                Ok(Some(f))
            } else {
                Err(())
            };
            *parse_issue = parsed.is_err();
            if let Ok(parsed) = parsed {
                *value = parsed;
            }
            *tooltip = Self::compute_tooltip(tooltip_original, language, *value, *value_original);

            parsed.is_ok()
        }
    }
    fn compute_tooltip(
        info: &str,
        language: Language,
        value: Option<f32>,
        original_value: Option<f32>,
    ) -> String {
        format!(
            "{info}\n{original_header}: {original}\n{current_header}: {current}",
            original_header = LocalizableStr {
                english: "Original value:"
            }
            .localize(language),
            original = original_value.map(|x| x.to_string()).unwrap_or(
                LocalizableString {
                    english: "Limit was not in use".into(),
                }
                .localize(language)
            ),
            current_header = LocalizableStr {
                english: "Current value"
            }
            .localize(language),
            current = value.map(|x| x.to_string()).unwrap_or(
                LocalizableString {
                    english: "Limit is not in use (text is empty".into(),
                }
                .localize(language)
            ),
        )
    }
    fn new(value: Option<f32>, language: Language, info: &str) -> Self {
        Self {
            parsed: value.map(|f| f.to_string()).unwrap_or_default(),
            current: value.map(|f| f.to_string()).unwrap_or_default(),
            tooltip: Self::compute_tooltip(info, language, value, value),
            value_original: value,
            value,
            parse_issue: false,
        }
    }

    fn reset(&mut self, language: Language, info: &str) {
        *self = Self::new(self.value_original, language, info)
    }
}

pub(super) struct LimitData {
    pub label: String,
    pub lower: Option<f32>,
    pub upper: Option<f32>,
    pub info: String,
}
impl Limit {
    pub(super) fn new(data: LimitData, language: Language) -> Self {
        let LimitData {
            label,
            lower,
            upper,
            info,
        } = data;
        Self {
            original_label: label.clone(),
            label_previous: label.clone().into(),
            label: label.into(),
            lower: LimitValue::new(lower, language, &info),
            upper: LimitValue::new(upper, language, &info),
            tooltip_original: info,
        }
    }
}
impl michael_visualizer_basic::LimitTrait for Limit {
    fn has_same_label(&self, other: &Self) -> bool {
        self.original_label == other.original_label
    }

    fn change_label(&mut self, label: LimitLabel) -> bool {
        let r = self.label == label;
        if !r {
            self.label = label;
        }
        r
    }

    fn label(&self) -> &michael_visualizer_basic::LimitLabel {
        &self.label
    }
}
