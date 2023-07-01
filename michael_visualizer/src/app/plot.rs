use super::{DataEvent, LockableLimitKey};
use crate::{data_types::finite_f32::FiniteF32, LocalizableStr, LocalizableString};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlotTab {
    #[serde(skip)]
    state: PlotState,
    x_key: LockableLimitKey,
    y_key: LockableLimitKey,
}
impl Default for PlotTab {
    fn default() -> Self {
        Self {
            state: Default::default(),
            x_key: LockableLimitKey::Locked(0),
            y_key: LockableLimitKey::Locked(1),
        }
    }
}
#[derive(Default)]
enum PlotState {
    #[default]
    Recompute,
    Plotting(Plotting),
    Error(LocalizableString),
}
struct Plotting {
    data: Vec<(Box<[f64]>, Box<[f64]>, crate::data_types::FileLabel)>,
    min_x: FiniteF32,
    max_x: FiniteF32,
    min_y: FiniteF32,
    max_y: FiniteF32,
}
impl Plotting {
    fn show(&self, ui: &mut egui::Ui, state: &mut super::AppState) {
        fn get_color(i: usize) -> egui::Color32 {
            let colors = egui_heatmap::colors::DISTINGUISHABLE_COLORS;
            let i = i % colors.len();
            colors[i]
        }
        let Self {
            data,
            min_x,
            max_x,
            min_y,
            max_y,
        } = self;

        egui::plot::Plot::new(ui.id().with("x-y-plot"))
            .include_x(min_x.as_f64())
            .include_x(max_x.as_f64())
            .include_y(min_y.as_f64())
            .include_y(max_y.as_f64())
            .legend(egui::plot::Legend::default())
            .show(ui, |ui| {
                for (index, (xx, yy, label)) in data.iter().enumerate() {
                    let points: egui::plot::PlotPoints =
                        xx.iter().zip(yy.iter()).map(|(&x, &y)| [x, y]).collect();
                    let points = egui::plot::Points::new(points)
                        .color(get_color(index))
                        .name(label.as_str())
                        .shape(egui::plot::MarkerShape::Circle)
                        .filled(true)
                        .radius(5.);
                    ui.points(points);
                }
            });
    }
}

impl PlotState {
    fn needs_recompute(&mut self) {
        *self = PlotState::Recompute;
    }
}

impl super::DataEventNotifyable for PlotTab {
    fn notify(&mut self, event: &DataEvent) -> Vec<DataEvent> {
        match event {
            DataEvent::Limit(limit) => match limit {
                super::limits::LimitEvent::LockableLimit(_) => {}
                super::limits::LimitEvent::Label(_) => self.needs_recompute(),
                super::limits::LimitEvent::Limit(_) => self.needs_recompute(),
                super::limits::LimitEvent::New(_) => {}
            },
            DataEvent::File(event) => match event {
                super::files::FileEvent::LoadFromPath { .. } => {}
                super::files::FileEvent::ParseFromBytes { .. } => {}
                super::files::FileEvent::ToShow(_) => self.needs_recompute(),
                super::files::FileEvent::Remove(_) => self.needs_recompute(),
                super::files::FileEvent::MoveUp(_) => self.needs_recompute(),
                super::files::FileEvent::MoveDown(_) => self.needs_recompute(),
                super::files::FileEvent::Label(_) => self.needs_recompute(),
                super::files::FileEvent::LoadError { .. } => {}
                super::files::FileEvent::Loaded { .. } => self.needs_recompute(),
            },
            DataEvent::Filtering => self.needs_recompute(),
            DataEvent::LimitRequest(_) => {}
            DataEvent::FileRequest(_) => {}
            DataEvent::SelectionRequest(_) => {}
            DataEvent::SelectionEvent(_) => {}
        }
        Default::default()
    }

    fn progress(&mut self, _state: &mut super::AppState) {}
}
impl super::TabTrait for PlotTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr {
            english: "X-Y Plot",
        }
        .localize(state.language)
    }
    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        if let &PlotState::Recompute = &self.state {
            self.state = self.recompute(state);
        }

        ui.vertical(|ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                let before = (self.x_key.clone(), self.y_key.clone());
                ui.horizontal(|ui| {
                    let mut needs_recompute = false;
                    ui.push_id("x_key", |ui| {
                        needs_recompute |= state.ui_selectable_limit(ui, &mut self.x_key);
                    });
                    ui.push_id("y_key", |ui| {
                        needs_recompute |= state.ui_selectable_limit(ui, &mut self.y_key);
                    });
                    if needs_recompute {
                        self.state.needs_recompute();
                    }
                });
                if (self.x_key.clone(), self.y_key.clone()) != before {
                    self.state = PlotState::Recompute;
                }
                match &mut self.state {
                    PlotState::Recompute => {
                        ui.label(
                            LocalizableString {
                                english: "Please select limits for x-axis and y-axis".into(),
                            }
                            .localize(state.language),
                        );
                        ui.heading(
                            LocalizableStr {
                                english: "ERROR - Recompute",
                            }
                            .localize(state.language),
                        );
                    }
                    PlotState::Plotting(plotting) => {
                        plotting.show(ui, state);
                    }
                    PlotState::Error(msg) => {
                        ui.label(msg.as_str().localize(state.language));
                        ui.heading(LocalizableStr { english: "ERROR" }.localize(state.language));
                    }
                }
            });
        });
    }
}

impl PlotTab {
    fn recompute(&mut self, state: &mut super::AppState) -> PlotState {
        if let (Some((x_key, x_lim)), Some((y_key, y_lim))) = (
            self.x_key
                .get(state.locked_limits)
                .1
                .and_then(|k| state.limits.get(k).map(|l| (k, l))),
            self.y_key
                .get(state.locked_limits)
                .1
                .and_then(|k| state.limits.get(k).map(|l| (k, l))),
        ) {
            let mut data = Vec::new();

            let mut x_min = FiniteF32::new(f32::MAX);
            let mut x_max = FiniteF32::new(f32::MIN);
            let mut y_min = FiniteF32::new(f32::MAX);
            let mut y_max = FiniteF32::new(f32::MIN);

            // find files which need to be drawn, and compute limits (if non are given, min/max will be used)
            for (file_key, (file_label, file, sorting)) in state.files.iter_loaded() {
                let filtering = state.total_filterings.get(file_key);
                let x_data = sorting.get(x_key).map(|column| file.get_column(*column));
                let y_data = sorting.get(y_key).map(|column| file.get_column(*column));
                if let (Some(filtering), Some(x_data), Some(y_data)) = (filtering, x_data, y_data) {
                    let x_data = x_data.simple_filter(filtering);
                    let y_data = y_data.simple_filter(filtering);
                    if x_data.is_empty() || y_data.is_empty() {
                        continue;
                    }
                    {
                        let min_f = *x_data.iter().min().expect("Empty-case already covered");
                        x_min = std::cmp::min(x_min, min_f);
                    }
                    {
                        let max_f = *x_data.iter().max().expect("Empty-case already covered");
                        x_max = std::cmp::max(x_max, max_f);
                    }
                    {
                        let min_f = *y_data.iter().min().expect("Empty-case already covered");
                        y_min = std::cmp::min(y_min, min_f);
                    }
                    {
                        let max_f = *y_data.iter().max().expect("Empty-case already covered");
                        y_max = std::cmp::max(y_max, max_f);
                    }
                    data.push((
                        x_data.iter().map(|x| x.as_f64()).collect(),
                        y_data.iter().map(|x| x.as_f64()).collect(),
                        file_label.clone(),
                    ));
                }
            }

            let (min_x, max_x) = x_lim.get_limits();
            let min_x = min_x.unwrap_or(x_min);
            let max_x = max_x.unwrap_or(x_max);
            let (min_y, max_y) = y_lim.get_limits();
            let min_y = min_y.unwrap_or(y_min);
            let max_y = max_y.unwrap_or(y_max);

            if data.is_empty() {
                return PlotState::Error(LocalizableString {
                    english: "No data after filtering - check limits".into(),
                });
            }

            PlotState::Plotting(Plotting {
                data,
                min_x,
                max_x,
                min_y,
                max_y,
            })
        } else {
            PlotState::Error(LocalizableString {
                english: "Please check limits both plot, x-axis and y-axis".into(),
            })
        }
    }
    fn needs_recompute(&mut self) {
        self.state.needs_recompute()
    }
}
