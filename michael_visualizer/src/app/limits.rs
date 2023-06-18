use super::{DataEvent, DataEvents};
use crate::data_types::finite_f32::FiniteF32;
use crate::data_types::{LimitKey, LimitLabel};
use crate::{LocalizableStr, LocalizableString};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(super) struct LimitContainer {
    limits: indexmap::IndexMap<LimitKey, Limit>,
}

impl LimitContainer {
    fn show(&mut self, ui: &mut egui::Ui, language: crate::Language, data_events: &mut DataEvents) {
        let Self { limits } = self;
        egui_extras::TableBuilder::new(ui)
            .columns(egui_extras::Column::auto().resizable(true), 4)
            .header(14., |mut header| {
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
                    if limit.is_trivial() {
                        continue;
                    }
                    body.row(30.0, |mut row| {
                        let mut changed = false;
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
        if let Some((key, current)) = self
            .limits
            .iter_mut()
            .find(|(_, l)| l.original_label == limit.original_label)
        {
            current.update_kind(limit.data_kind());
            (false, key.clone())
        } else {
            let key = limit_key_generator.next();
            self.limits.insert(key.clone(), limit);
            (true, key)
        }
    }

    pub(crate) fn get(&self, limit_key: &LimitKey) -> Option<&Limit> {
        self.limits.get(limit_key)
    }

    #[must_use]
    pub(crate) fn get_mut(&mut self, key: &LimitKey) -> Option<&mut Limit> {
        self.limits.get_mut(key)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&LimitKey, &Limit)> {
        self.limits.iter()
    }

    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.limits.is_empty()
    }
}
impl super::DataEventNotifyable for LimitContainer {
    fn notify(&mut self, _event: &super::DataEvent) -> Vec<super::DataEvent> {
        Default::default()
    }

    fn progress(&mut self, _state: &mut super::AppState) {}
}

pub enum LimitEvent {
    LockableLimit(usize),
    Label(LimitKey),
    Limit(LimitKey),
    New(LimitKey),
}
pub enum LimitRequest {
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
    data_kind: LimitDataKind,
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct LimitValue {
    value: Option<FiniteF32>,
    value_original: Option<FiniteF32>,
    parse_issue: bool,
    parsed: String,
    current: String,
    tooltip: LocalizableString,
    warn: Option<LocalizableString>,
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(super) enum LimitDataKind {
    Float,
    Int {
        uniques: UniqueInt,
        min: i32,
        max: i32,
    },
}
impl LimitDataKind {
    pub(crate) fn is_int(&self) -> bool {
        match self {
            LimitDataKind::Float => false,
            LimitDataKind::Int { .. } => true,
        }
    }

    fn check(&self, value: FiniteF32) -> Option<LocalizableString> {
        if let &LimitDataKind::Int { .. } = self {
            if value.round() != value.inner() {
                Some(LocalizableString {
                    english: format!("Value is not an integer: {value:?}"),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn update(&mut self, data_kind: &LimitDataKind) {
        *self = match (std::mem::replace(self, LimitDataKind::Float), data_kind) {
            (LimitDataKind::Float, _) | (_, LimitDataKind::Float) => LimitDataKind::Float,
            (
                LimitDataKind::Int {
                    uniques: previous,
                    min: min1,
                    max: max1,
                },
                LimitDataKind::Int {
                    uniques: new,
                    min: min2,
                    max: max2,
                },
            ) => LimitDataKind::Int {
                uniques: match (previous, new) {
                    (UniqueInt::Uniques(mut previous), UniqueInt::Uniques(new)) => {
                        previous.extend(new);
                        if previous.len() > 100 {
                            UniqueInt::MoreThanHundredDifferent
                        } else {
                            UniqueInt::Uniques(previous)
                        }
                    }
                    (_, UniqueInt::MoreThanHundredDifferent)
                    | (UniqueInt::MoreThanHundredDifferent, _) => {
                        UniqueInt::MoreThanHundredDifferent
                    }
                },
                min: std::cmp::min(min1, *min2),
                max: std::cmp::max(max1, *max2),
            },
        }
    }

    pub(crate) fn new(data: &super::files::DataColumn) -> Self {
        match data {
            super::files::DataColumn::Float(_) => Self::Float,
            super::files::DataColumn::Int(d) => {
                let set: std::collections::HashSet<i32> = d.iter().cloned().collect();
                let min = set.iter().cloned().min().unwrap_or_default();
                let max = set.iter().cloned().max().unwrap_or_default();
                if set.len() > 100 {
                    Self::Int {
                        min,
                        max,
                        uniques: UniqueInt::MoreThanHundredDifferent,
                    }
                } else {
                    Self::Int {
                        min,
                        max,
                        uniques: UniqueInt::Uniques(set),
                    }
                }
            }
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(super) enum UniqueInt {
    Uniques(std::collections::HashSet<i32>),
    MoreThanHundredDifferent,
}

impl LimitValue {
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        tooltip_original: LocalizableStr,
        data_kind: &LimitDataKind,
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
            warn,
        } = self;
        let mut has_focus = false;
        ui.scope(|ui| {
            let text = egui::TextEdit::singleline(current);
            let text = if *parse_issue {
                let text = text.text_color(egui::Color32::WHITE);
                ui.visuals_mut().extreme_bg_color = egui::Color32::RED;
                text.show(ui)
            } else if warn.is_some() {
                let text = text.text_color(egui::Color32::BLACK);
                ui.visuals_mut().extreme_bg_color = egui::Color32::YELLOW;
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
            self.reset(tooltip_original, data_kind);
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
                match FiniteF32::new_checked(f) {
                    Some(f) => Ok(Some(f)),
                    None => Err(()),
                }
            } else {
                Err(())
            };
            *parse_issue = parsed.is_err();
            *warn = match parsed {
                Ok(Some(parsed)) => {
                    *value = Some(parsed);
                    data_kind.check(parsed)
                }
                Ok(None) => {
                    *value = None;
                    None
                }
                Err(()) => None,
            };

            *tooltip = Self::compute_tooltip(
                tooltip_original,
                *value,
                *value_original,
                warn.as_ref().map(|x| x.as_str()),
                data_kind,
            );

            parsed.is_ok()
        }
    }
    fn compute_tooltip(
        LocalizableStr { english: info_eng }: LocalizableStr,
        value: Option<FiniteF32>,
        original_value: Option<FiniteF32>,
        warn: Option<LocalizableStr>,
        data_kind: &LimitDataKind,
    ) -> LocalizableString {
        LocalizableString {
            english: format!(
                            "{info_eng}\n{data_kind_header}: {data_kind}\nOriginal value: {original}\nCurrent value: {current}{warn}",
                            data_kind_header = "Type",
                            data_kind = match data_kind{
                                                LimitDataKind::Float => "Float",
                                                                                                                LimitDataKind::Int{..} => "Integer",
                                                                    },
                            original = original_value
                                .map(|x| x.to_string())
                                .unwrap_or("Limit was not in use".into()),
                            current = value
                                .map(|x| x.to_string())
                                .unwrap_or("Limit is not in use (text is empty)".into()),
                            warn = warn
                                .map(|msg| format!("\n{msg}", msg = msg.english))
                                .unwrap_or_default()
                        ),
        }
    }
    fn new(value: Option<FiniteF32>, info: LocalizableStr, data_kind: &LimitDataKind) -> Self {
        let warn = value.and_then(|value| data_kind.check(value));
        Self {
            parsed: value.map(|f| f.to_string()).unwrap_or_default(),
            current: value.map(|f| f.to_string()).unwrap_or_default(),
            tooltip: Self::compute_tooltip(
                info,
                value,
                value,
                warn.as_ref().map(|x| x.as_str()),
                data_kind,
            ),
            value_original: value,
            value,
            parse_issue: false,
            warn,
        }
    }

    fn reset(&mut self, info: LocalizableStr, data_kind: &LimitDataKind) {
        *self = Self::new(self.value_original, info, data_kind)
    }

    fn update(&mut self, info: LocalizableStr, data_kind: &LimitDataKind) {
        if let Some(value) = self.value {
            self.warn = data_kind.check(value);
            self.tooltip = Self::compute_tooltip(
                info,
                self.value,
                self.value_original,
                self.warn.as_ref().map(|x| x.as_str()),
                data_kind,
            );
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(super) struct LimitData {
    pub label: LimitLabel,
    pub lower: Option<FiniteF32>,
    pub upper: Option<FiniteF32>,
    pub info: LocalizableString,
    pub data_kind: LimitDataKind,
}
impl Limit {
    pub(super) fn new(data: LimitData) -> Self {
        let LimitData {
            label,
            lower,
            upper,
            info,
            data_kind,
        } = data;
        Self {
            original_label: label.clone(),
            label_previous: label.clone(),
            label,
            lower: LimitValue::new(lower, info.as_str(), &data_kind),
            upper: LimitValue::new(upper, info.as_str(), &data_kind),
            tooltip_original: info,
            data_kind,
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
            data_kind: _,
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
                    ui.close_menu();
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
        self.lower.show(
            ui,
            self.tooltip_original.as_str(),
            &self.data_kind,
            language,
        )
    }
    pub(super) fn show_upper(&mut self, ui: &mut egui::Ui, language: crate::Language) -> bool {
        self.upper.show(
            ui,
            self.tooltip_original.as_str(),
            &self.data_kind,
            language,
        )
    }

    pub(super) fn is_outside(&self, x: f32) -> bool {
        if let Some(x) = FiniteF32::new_checked(x) {
            if let Some(l) = self.lower.value {
                if x < l {
                    return true;
                }
            }
            if let Some(u) = self.upper.value {
                if x > u {
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
            data_kind: self.data_kind.clone(),
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

    pub(super) fn data_kind(&self) -> &LimitDataKind {
        &self.data_kind
    }

    fn update_kind(&mut self, data_kind: &LimitDataKind) {
        self.data_kind.update(data_kind);
        self.lower
            .update(self.tooltip_original.as_str(), &self.data_kind);
        self.upper
            .update(self.tooltip_original.as_str(), &self.data_kind);
    }

    pub(crate) fn is_trivial(&self) -> bool {
        match &self.data_kind {
            LimitDataKind::Float => false,
            LimitDataKind::Int {
                uniques: UniqueInt::MoreThanHundredDifferent,
                min: _,
                max: _,
            } => false,
            LimitDataKind::Int {
                uniques: UniqueInt::Uniques(uniques),
                min: _,
                max: _,
            } => uniques.len() < 2,
        }
    }

    pub(crate) fn is_int(&self) -> bool {
        self.data_kind.is_int()
    }

    pub(crate) fn get_label(&self) -> &LimitLabel {
        &self.label
    }

    pub(crate) fn get_limits(&self) -> (Option<FiniteF32>, Option<FiniteF32>) {
        (self.lower.value, self.upper.value)
    }
}
