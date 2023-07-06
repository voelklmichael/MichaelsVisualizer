use crate::{
    app::{files::FileEvent, limits::LimitEvent, DataEvent},
    data_types::finite_f32::FiniteF32,
    dialog::Dialog,
    LocalizableStr, LocalizableString,
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DistributionTab {
    to_show: super::LockableLimitKey,
    to_color: Option<super::LockableLimitKey>,
    #[serde(skip)]
    state: State,
    resolution: usize,
}
impl Default for DistributionTab {
    fn default() -> Self {
        Self {
            to_show: Default::default(),
            to_color: Default::default(),
            state: Default::default(),
            resolution: 31,
        }
    }
}

impl super::DataEventNotifyable for DistributionTab {
    fn notify(&mut self, event: &DataEvent) -> Vec<DataEvent> {
        let (needs_recompute, events) = self.state.notify(event, &self.to_show);
        if needs_recompute {
            self.state = State::NeedsRecompute;
        }
        events
    }

    fn progress(&mut self, state: &mut super::AppState) {
        if let State::Plot(plot) = &mut self.state {
            while let Ok(msg) = { plot.limit_label_change_receiver.try_recv() } {
                match msg {
                    LimitLabelChange::Change(s) => plot.limit_label_change_value = Some(s),
                    LimitLabelChange::Ok => {
                        if let Some(s) = plot.limit_label_change_value.take() {
                            state.data_events.push(super::DataEvent::LimitRequest(
                                super::limits::LimitRequest::RequestLabel(
                                    plot.limit_key.clone(),
                                    s,
                                ),
                            ))
                        }
                    }
                }
            }
        }
    }
}
struct DistributionPlot {
    // plot data
    limit_key: crate::data_types::LimitKey,
    limit_label: crate::data_types::LimitLabel,
    min: FiniteF32,
    max: FiniteF32,
    entries: Vec<ColoredDistributionEntry>,
    colors: Vec<(i32, egui::Color32)>,
    // user input
    context_pos: Option<(egui::Vec2, bool)>,
    limit_label_change_sender: std::sync::mpsc::Sender<LimitLabelChange>,
    limit_label_change_value: Option<String>,
    limit_label_change_receiver: std::sync::mpsc::Receiver<LimitLabelChange>,
    legend_left_top: Option<egui::Pos2>,
}
enum LimitLabelChange {
    Change(String),
    Ok,
}
impl DistributionPlot {
    fn show(&mut self, ui: &mut egui::Ui, state: &mut super::AppState) -> Vec<DataEvent> {
        ui.label("ToDo");
        Default::default()
    }
}

#[derive(Default)]
enum State {
    #[default]
    NeedsRecompute,
    NoLimitSelected,
    Plot(DistributionPlot),
    Error(LocalizableString),
}
impl State {
    fn notify(
        &mut self,
        event: &DataEvent,
        to_show: &super::LockableLimitKey,
    ) -> (bool, Vec<DataEvent>) {
        fn condition(condition: bool) -> (bool, Vec<DataEvent>) {
            if condition {
                (true, Default::default())
            } else {
                (false, Default::default())
            }
        }
        let unaffected: (bool, Vec<DataEvent>) = condition(false);
        let affected: (bool, Vec<DataEvent>) = condition(true);
        match self {
            State::NeedsRecompute => unaffected,
            State::NoLimitSelected => match event {
                DataEvent::Limit(event) => match event {
                    LimitEvent::LockableLimit(index) => {
                        if to_show.is_locked(Some(index)) {
                            affected
                        } else {
                            unaffected
                        }
                    }
                    LimitEvent::Label(_) => unaffected,
                    LimitEvent::Limit(_) => unaffected,
                    LimitEvent::New(_) => affected,
                },
                _ => unaffected,
            },
            State::Plot(DistributionPlot {
                limit_key,
                limit_label: _,
                min: _,
                max: _,
                entries,
                context_pos: _,
                limit_label_change_sender: _,
                limit_label_change_receiver: _,
                limit_label_change_value: _,
                colors: _,
                legend_left_top: _,
            }) => match event {
                DataEvent::Limit(event) => match event {
                    LimitEvent::LockableLimit(index) => {
                        if to_show.is_locked(Some(index)) {
                            affected
                        } else {
                            unaffected
                        }
                    }
                    LimitEvent::Label(key) => condition(key == limit_key),
                    LimitEvent::Limit(key) => condition(key == limit_key),
                    LimitEvent::New(_) => unaffected,
                },
                DataEvent::File(event) => match event {
                    FileEvent::LoadFromPath { .. } => unaffected,
                    FileEvent::ParseFromBytes { .. } => unaffected,
                    FileEvent::ToShow(_) => affected,
                    FileEvent::Remove(key) => condition(entries.iter().any(|x| &x.key == key)),
                    FileEvent::MoveUp(_) => affected,
                    FileEvent::MoveDown(_) => affected,
                    FileEvent::Label(key) => condition(entries.iter().any(|x| &x.key == key)),
                    FileEvent::LoadError { .. } => unaffected,
                    FileEvent::Loaded { .. } => affected,
                },
                DataEvent::Filtering => affected,
                DataEvent::LimitRequest(_) => unaffected,
                DataEvent::FileRequest(_) => unaffected,
                DataEvent::SelectionRequest(_) => unaffected,
                DataEvent::SelectionEvent(_) => unaffected,
            },
            State::Error(_) => affected,
        }
    }
}

impl DistributionTab {
    fn recompute(&mut self, state: &super::AppState) -> State {
        if let Some(limit_key) = self.to_show.get(state.locked_limits).1 {
            if let Some(limit) = state.limits.get(limit_key) {
                let super::limits::LimitData {
                    label: limit_label,
                    lower,
                    upper,
                    info: _,
                    data_kind: _,
                } = limit.data();
                let min = lower.unwrap_or(FiniteF32::new(f32::MIN));
                let max = upper.unwrap_or(FiniteF32::new(f32::MAX));
                let mut entries = Vec::new();
                let to_color_key = self.to_color.as_ref().and_then(|k| {
                    k.get(state.locked_limits)
                        .1
                        .filter(|&to_color_key| to_color_key != limit_key)
                });
                for file_key in state.get_files_for_limit(limit_key) {
                    let filtering = state.total_filterings.get(file_key);
                    let file = state.files.get(file_key).and_then(|x| x.get_loaded());
                    if let (Some((label, file, sorting)), Some(filtering)) = (file, filtering) {
                        if let Some(column) = sorting.get(limit_key) {
                            let data = file.get_column(*column);
                            assert_eq!(data.len(), filtering.len());
                            let data = data.filter(filtering, min, max);
                            let to_color = to_color_key
                                .and_then(|k| sorting.get(k))
                                .and_then(|column| file.get_column(*column).as_int());
                            if data.is_empty() {
                                continue;
                            }
                            let data = if let Some(to_color) = to_color {
                                let min = if let Some(min) = to_color.iter().min() {
                                    min
                                } else {
                                    continue;
                                };
                                let max = if let Some(max) = to_color.iter().max() {
                                    max
                                } else {
                                    continue;
                                };
                                let mut colors = Vec::new();
                                for i in *min..(*max + 1) {
                                    let d = data
                                        .iter()
                                        .zip(to_color.iter())
                                        .filter(|(_, &c)| c == i)
                                        .map(|(d, _)| *d)
                                        .collect();
                                    colors.push((Some(i), d));
                                }
                                colors
                            } else {
                                vec![(None, data)]
                            };
                            entries.push((file_key.clone(), label.clone(), data));
                        }
                    }
                }
                if entries.is_empty() {
                    return State::Error(LocalizableString {
                        english: "No data after filtering".into(),
                    });
                }
                let min: FiniteF32 = lower.unwrap_or_else(|| {
                    entries
                        .iter()
                        .flat_map(|(_, _, e)| e.iter().flat_map(|(_, x)| x.iter().min()).min())
                        .min()
                        .cloned()
                        .unwrap_or(min)
                });
                let max: FiniteF32 = upper.unwrap_or_else(|| {
                    entries
                        .iter()
                        .flat_map(|(_, _, e)| e.iter().flat_map(|(_, x)| x.iter().max()).max())
                        .max()
                        .cloned()
                        .unwrap_or(max)
                });
                let entries = entries
                    .into_iter()
                    .filter_map(|(key, label, data)| {
                        ColoredDistributionEntry::new(key, label, data, self.resolution, min, max)
                    })
                    .collect::<Vec<_>>();
                let mut colors = entries
                    .iter()
                    .flat_map(|e| e.entries.iter().flat_map(|x| x.0))
                    .collect::<Vec<_>>();
                colors.sort();
                let colors = colors
                    .into_iter()
                    .enumerate()
                    .map(|(i, c)| (c, state.get_color(i)))
                    .collect();
                let (s, r) = std::sync::mpsc::channel();
                State::Plot(DistributionPlot {
                    limit_key: limit_key.clone(),
                    limit_label,
                    min,
                    max,
                    entries,
                    context_pos: Default::default(),
                    limit_label_change_sender: s,
                    limit_label_change_receiver: r,
                    limit_label_change_value: Default::default(),
                    colors,
                    legend_left_top: Default::default(),
                })
            } else {
                State::Error(LocalizableString {
                    english: "No limits available".into(),
                })
            }
        } else {
            State::NoLimitSelected
        }
    }
}

struct ColoredDistributionEntry {
    key: crate::data_types::FileKey,
    label: crate::data_types::FileLabel,
    entries: Vec<(Option<i32>, ViolinEntry)>,
}
struct ViolinEntry {
    bins: Vec<u64>,
    max_bin: u64,
    mean_height: f32,
}

impl super::TabTrait for DistributionTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr {
            english: "Distribution",
        }
        .localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if state.ui_selectable_limit(ui, &mut self.to_show) {
                self.state = State::NeedsRecompute;
            }
            if state.ui_coloring_limit(ui, &mut self.to_color) {
                self.state = State::NeedsRecompute;
            }
        });

        if let &State::NeedsRecompute = &self.state {
            self.state = self.recompute(state);
        }
        match &mut self.state {
            State::NeedsRecompute => {
                ui.heading(
                    LocalizableStr {
                        english: "Recomputing …",
                    }
                    .localize(state.language),
                );
            }
            State::NoLimitSelected => {
                ui.heading(
                    LocalizableStr {
                        english: "Please select limit to plot",
                    }
                    .localize(state.language),
                );
            }
            State::Plot(plot) => {
                let events = plot.show(ui, state);
                state.data_events.extend(events);
            }
            State::Error(msg) => {
                ui.heading(
                    LocalizableStr {
                        english: "Failed to plot data due to:",
                    }
                    .localize(state.language),
                );
                ui.label(msg.as_str().localize(state.language));
            }
        }
    }
}
impl ColoredDistributionEntry {
    #[must_use]
    fn new(
        key: crate::data_types::FileKey,
        label: crate::data_types::FileLabel,
        data: Vec<(Option<i32>, Vec<FiniteF32>)>,
        resolution: usize,
        min: FiniteF32,
        max: FiniteF32,
    ) -> Option<Self> {
        if resolution == 0 {
            return None;
        }
        let entries = data
            .into_iter()
            .filter_map(|(color, data)| {
                ViolinEntry::new(data, resolution, min, max).map(|d| (color, d))
            })
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return None;
        }
        Some(Self {
            key,
            label,
            entries,
        })
    }

    fn max_bin(&self) -> u64 {
        self.entries.iter().map(|(_, x)| x.max_bin).max().unwrap()
    }

    fn to_shapes(
        &self,
        state: &super::AppState,
        to_inner_screen: egui::emath::RectTransform,
        coloring_index: usize,
        entries_count: usize,
        normalization: Option<u64>,
        colors: &[(i32, egui::Color32)],
    ) -> Vec<egui::Shape> {
        let entries = &self.entries;
        if entries.is_empty() {
            return Default::default();
        }
        if colors.is_empty() {
            let color = state.get_color(coloring_index);
            entries.first().unwrap().1.to_shapes(
                color,
                to_inner_screen,
                coloring_index,
                entries_count,
                normalization,
            )
        } else {
            let mut shapes = Vec::new();
            for (color, entry) in &self.entries {
                let color = color.expect("Colors were defined, but some entry had no color");
                let (color_index, (_, color)) = colors
                    .iter()
                    .enumerate()
                    .find(|(_, (x, _))| *x == color)
                    .unwrap();
                shapes.extend(entry.to_shapes(
                    *color,
                    to_inner_screen,
                    coloring_index * colors.len() + color_index,
                    entries_count * colors.len(),
                    normalization,
                ))
            }
            shapes
        }
    }
}
impl ViolinEntry {
    #[must_use]
    fn new(
        data: Vec<FiniteF32>,
        resolution: usize,
        min: FiniteF32,
        max: FiniteF32,
    ) -> Option<Self> {
        if resolution == 0 {
            return None;
        }
        let count = data.len();
        let mean = data.iter().map(|x| x.inner()).sum::<f32>() / (count as f32);
        let delta = max.inner() - min.inner();
        let mut bins = vec![0u64; resolution];
        let resolution_float = resolution as f32;
        let factor = resolution_float / delta;
        if !delta.is_finite() || delta <= 0. {
            return None;
        }
        for d in data {
            let ratio = (d.inner() - min.inner()) * factor; // between 0. and (resolution)
            let ratio = ratio.clamp(0., resolution_float); // numerical precision - might be unnecessary???
            let bin = (ratio as usize).clamp(0, resolution - 1);
            bins[bin] += 1;
        }
        let max_bin = bins.iter().max().cloned().unwrap_or(0);
        let mean_height = (mean - min.inner()) / delta;

        Some(Self {
            //count,
            //mean,
            bins,
            max_bin,
            mean_height,
        })
    }

    pub fn get_boundaries(&self) -> Vec<Vec<(usize, u64)>> {
        let mut parts = Vec::new();
        let mut ongoing = None;
        for (index, &width) in self.bins.iter().enumerate() {
            match (ongoing.take(), width) {
                (None, 0) => { /* nothing to do */ }
                (None, width) => ongoing = Some(vec![(index, width)]),
                (Some(ongoing), 0) => parts.push(ongoing),
                (Some(mut current), width) => {
                    current.push((index, width));
                    ongoing = Some(current);
                }
            }
        }
        if let Some(ongoing) = ongoing {
            parts.push(ongoing);
        }
        parts
    }
    fn to_shapes(
        &self,
        color: egui::Color32,
        transform: egui::emath::RectTransform,
        i: usize,
        n: usize,
        normalization: Option<u64>,
    ) -> Vec<egui::Shape> {
        let mut shapes = Vec::new();
        let normalization = normalization.unwrap_or(self.max_bin);

        let bin_count_twice = (2 * self.bins.len()) as f32;
        let center = (2 * i + 1) as f32 / (2 * n) as f32;
        let height = 1. / bin_count_twice;
        for segments in self.get_boundaries() {
            if segments.is_empty() {
                continue;
            }
            let mut points_right = Vec::new();
            let mut points_left = Vec::new();
            for (index, width) in segments {
                let ratio = width as f32 / normalization as f32;
                let y = 1.0 - (2 * index + 1) as f32 / bin_count_twice;
                let width = ratio / (n as f32) / 2. * 0.95;
                points_left.push(transform * egui::pos2(center - width, y - height));
                points_left.push(transform * egui::pos2(center - width, y + height));
                points_right.push(transform * egui::pos2(center + width, y - height));
                points_right.push(transform * egui::pos2(center + width, y + height));
            }
            points_left.extend(points_right.into_iter().rev());
            shapes.push(egui::Shape::closed_line(
                points_left,
                egui::Stroke::new(1.0, color),
            ));
        }
        let mean = self.mean_height;
        if mean.is_finite() && mean > 0. && mean < 1. {
            shapes.push(egui::Shape::circle_filled(
                transform * egui::pos2(center, 1. - mean),
                5.,
                color,
            ))
        }
        shapes
    }
}

pub trait IsInside {
    fn inside(self, lower: Self, upper: Self) -> bool;
}
impl IsInside for f32 {
    fn inside(self, lower: Self, upper: Self) -> bool {
        self >= lower && self <= upper
    }
}
