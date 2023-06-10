use crate::data_types::finite_f32::FiniteF32;
use crate::data_types::{LimitKey, LimitLabel};
use crate::{LocalizableStr, LocalizableString};

use super::{DataEvent, DataEvents};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(super) struct LimitContainer {
    limits: indexmap::IndexMap<LimitKey, Limit>,
    to_show: Option<LimitKey>,
}
impl LimitContainer {
    fn show(&mut self, ui: &mut egui::Ui, language: crate::Language, data_events: &mut DataEvents) {
        let Self { limits, to_show } = self;
        egui_extras::TableBuilder::new(ui)
            .columns(egui_extras::Column::auto().resizable(true), 4)
            .header(14., |mut header| {
                header.col(|ui| {
                    ui.heading(LocalizableStr { english: "Plot?" }.localize(language));
                });
                header.col(|ui| {
                    ui.heading(LocalizableStr { english: "Label" }.localize(language));
                });
                header.col(|ui| {
                    ui.heading(LocalizableStr { english: "Lower" }.localize(language));
                });
                header.col(|ui| {
                    ui.heading(LocalizableStr { english: "Upper" }.localize(language));
                });
            })
            .body(|mut body| {
                for (key, limit) in limits.iter_mut() {
                    body.row(30.0, |mut row| {
                        let mut changed = false;
                        row.col(|ui| {
                            let previous = to_show.clone();
                            ui.radio_value(to_show, Some(key.clone()), "");
                            if previous != *to_show {
                                data_events
                                    .push(DataEvent::Limit(LimitEvent::ToShow(to_show.clone())));
                            }
                        });
                        row.col(|ui| {
                            if limit.show_label(ui, language) {
                                data_events.push(DataEvent::Limit(LimitEvent::Label(key.clone())));
                            }
                        });
                        row.col(|ui| {
                            changed |= limit.show_lower(ui, language);
                        });
                        row.col(|ui| {
                            changed |= limit.show_upper(ui, language);
                        });
                        if changed {
                            data_events.push(DataEvent::Limit(LimitEvent::Limit(key.clone())));
                        }
                    });
                }
            });
    }

    pub(crate) fn insert(
        &mut self,
        limit_key_generator: &mut crate::data_types::LimitKeyGenerator,
        limit: Limit,
    ) -> (bool, LimitKey) {
        if let Some(key) = self
            .limits
            .iter()
            .filter(|(_, l)| l.original_label == limit.original_label)
            .map(|(key, _)| key.clone())
            .next()
        {
            (false, key)
        } else {
            let key = limit_key_generator.next();
            self.limits.insert(key.clone(), limit);
            (true, key)
        }
    }
    #[must_use]
    pub(super) fn to_show(&self) -> Option<&LimitKey> {
        self.to_show.as_ref()
    }

    pub(crate) fn get(&self, limit_key: &LimitKey) -> Option<&Limit> {
        self.limits.get(limit_key)
    }

    #[must_use]
    pub(crate) fn get_mut(&mut self, key: &LimitKey) -> Option<&mut Limit> {
        self.limits.get_mut(key)
    }
}
impl super::DataEventNotifyable for LimitContainer {
    fn notify(&mut self, _event: &super::DataEvent) -> Vec<super::DataEvent> {
        Default::default()
    }

    fn progress(&mut self, state: &mut super::AppState) {}
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum LimitEvent {
    ToShow(Option<LimitKey>),
    Label(LimitKey),
    Limit(LimitKey),
    New(LimitKey),
    RequestLabel(crate::data_types::LimitKey, String),
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct LimitTab {}
impl super::TabTrait for LimitTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr { english: "Limits" }.localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        let super::AppState {
            language,
            limits,
            data_events,
            ..
        } = state;
        limits.show(ui, *language, data_events);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Limit {
    original_label: LimitLabel,
    label: LimitLabel,
    label_previous: LimitLabel,
    tooltip_original: LocalizableString,
    lower: LimitValue,
    upper: LimitValue,
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct LimitValue {
    value: Option<FiniteF32>,
    value_original: Option<FiniteF32>,
    parse_issue: bool,
    parsed: String,
    current: String,
    tooltip: LocalizableString,
}
impl LimitValue {
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        tooltip_original: LocalizableStr,
        language: crate::Language,
    ) -> bool {
        let mut reset_requested = false;
        let LimitValue {
            value,
            value_original,
            parse_issue,
            parsed,
            current,
            tooltip,
        } = self;
        let mut has_focus = false;
        ui.scope(|ui| {
            let text = egui::TextEdit::singleline(current);
            let text = if *parse_issue {
                let text = text.text_color(egui::Color32::WHITE);
                ui.visuals_mut().extreme_bg_color = egui::Color32::RED;
                text.show(ui)
            } else {
                text.show(ui)
            }
            .response;
            has_focus = text.has_focus();
            text.on_hover_text(tooltip.as_str().localize(language))
                .context_menu(|ui| {
                    if ui.button(super::RESET.localize(language)).clicked() {
                        reset_requested = true;
                        ui.close_menu();
                    }
                });
        });
        if reset_requested {
            self.reset(tooltip_original);
            return true;
        }
        if has_focus || current == parsed {
            false
        } else {
            *parsed = current.to_string();
            let parsed = parsed.trim();
            let parsed = if parsed.is_empty() {
                Ok(None)
            } else if let Ok(f) = parsed.parse::<f32>() {
                Ok(FiniteF32::new_checked(f))
            } else {
                Err(())
            };
            *parse_issue = parsed.is_err();
            if let Ok(parsed) = parsed {
                *value = parsed;
            }
            *tooltip = Self::compute_tooltip(tooltip_original, *value, *value_original);

            parsed.is_ok()
        }
    }
    fn compute_tooltip(
        LocalizableStr { english: info_eng }: LocalizableStr,
        value: Option<FiniteF32>,
        original_value: Option<FiniteF32>,
    ) -> LocalizableString {
        LocalizableString {
            english: format!(
                "{info_eng}\nOriginal value: {original}\nCurrent value: {current}",
                original = original_value
                    .map(|x| x.to_string())
                    .unwrap_or("Limit was not in use".into()),
                current = value
                    .map(|x| x.to_string())
                    .unwrap_or("Limit is not in use (text is empty".into())
            ),
        }
    }
    fn new(value: Option<FiniteF32>, info: LocalizableStr) -> Self {
        Self {
            parsed: value.map(|f| f.to_string()).unwrap_or_default(),
            current: value.map(|f| f.to_string()).unwrap_or_default(),
            tooltip: Self::compute_tooltip(info, value, value),
            value_original: value,
            value,
            parse_issue: false,
        }
    }

    fn reset(&mut self, info: LocalizableStr) {
        *self = Self::new(self.value_original, info)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(super) struct LimitData {
    pub label: LimitLabel,
    pub lower: Option<FiniteF32>,
    pub upper: Option<FiniteF32>,
    pub info: LocalizableString,
}
impl Limit {
    pub(super) fn new(data: LimitData) -> Self {
        let LimitData {
            label,
            lower,
            upper,
            info,
        } = data;
        Self {
            original_label: label.clone(),
            label_previous: label.clone(),
            label,
            lower: LimitValue::new(lower, info.as_str()),
            upper: LimitValue::new(upper, info.as_str()),
            tooltip_original: info,
        }
    }

    pub(super) fn show_label(&mut self, ui: &mut egui::Ui, language: crate::Language) -> bool {
        let Self {
            original_label,
            label,
            label_previous,
            tooltip_original: info,
            lower: _,
            upper: _,
        } = self;
        let has_focus = ui
            .text_edit_singleline(label.get_mut())
            .on_hover_text(format!(
                "{info}\n{original_header}: {original_label:?}",
                info = info.as_str().localize(language),
                original_header = LocalizableStr {
                    english: "Original label"
                }
                .localize(language)
            ))
            .context_menu(|ui| {
                if ui.button(super::RESET.localize(language)).clicked() {
                    *label = original_label.clone();
                }
            })
            .has_focus();
        if !has_focus && label != label_previous {
            *label_previous = label.clone();
            true
        } else {
            false
        }
    }

    pub(super) fn show_lower(&mut self, ui: &mut egui::Ui, language: crate::Language) -> bool {
        self.lower
            .show(ui, self.tooltip_original.as_str(), language)
    }
    pub(super) fn show_upper(&mut self, ui: &mut egui::Ui, language: crate::Language) -> bool {
        self.upper
            .show(ui, self.tooltip_original.as_str(), language)
    }

    pub(super) fn is_outside(&self, x: f32) -> bool {
        if let Some(x) = FiniteF32::new_checked(x) {
            if let Some(l) = self.lower.value {
                if l < x {
                    return true;
                }
            }
            if let Some(u) = self.upper.value {
                if u > x {
                    return true;
                }
            }
            false
        } else {
            true
        }
    }

    pub(super) fn data(&self) -> LimitData {
        LimitData {
            label: self.label.clone(),
            lower: self.lower.value,
            upper: self.upper.value,
            info: self.tooltip_original.clone(),
        }
    }

    pub(super) fn change_label(&mut self, label: &str) -> bool {
        if self.label.as_str() != label {
            self.label = label.to_string().into();
            self.label_previous = label.to_string().into();
            true
        } else {
            false
        }
    }
}
