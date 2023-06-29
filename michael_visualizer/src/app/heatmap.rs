use egui_heatmap::CoordinatePoint;

use crate::{data_types::LimitKey, LocalizableStr, LocalizableString};

use super::{limits::LimitDataKind, DataEvent};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct HeatmapTab {
    to_show: super::LockableLimitKey,
    #[serde(skip)]
    state: HeatmapState,
    x_key: Option<LimitKey>,
    y_key: Option<LimitKey>,
}
#[derive(Default)]
enum HeatmapState {
    #[default]
    Recompute,
    Heatmap(
        Box<(
            egui_heatmap::MultiBitmapWidget<crate::data_types::FileKey>,
            egui_heatmap::ShowState<crate::data_types::FileKey>,
        )>,
    ),
    Error(LocalizableString),
}
impl HeatmapState {
    fn needs_recompute(&mut self) {
        *self = HeatmapState::Recompute;
    }
}

impl super::DataEventNotifyable for HeatmapTab {
    fn notify(&mut self, event: &DataEvent) -> Vec<DataEvent> {
        match event {
            DataEvent::Limit(limit) => match limit {
                super::limits::LimitEvent::LockableLimit(index) => {
                    if self.to_show.is_locked(Some(index)) {
                        self.needs_recompute()
                    }
                }
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
        }
        Default::default()
    }

    fn progress(&mut self, state: &mut super::AppState) {
        if let HeatmapState::Heatmap(heatmap_with_state) = &mut self.state {
            let heatmap_state = &mut heatmap_with_state.as_mut().1;
            for event in heatmap_state.events() {
                let event = match event {
                    egui_heatmap::Event::Hide(key) => {
                        DataEvent::FileRequest(super::files::FileRequest::Hide(key))
                    }
                    egui_heatmap::Event::ShowAll => {
                        DataEvent::FileRequest(super::files::FileRequest::ShowAll)
                    }
                    egui_heatmap::Event::UnselectAll => {
                        DataEvent::SelectionRequest(super::selection::SelectionRequest::UnselectAll)
                    }
                    egui_heatmap::Event::ShowRectangle => {
                        //TODO: limits adjustment buttona
                        if true {
                            continue;
                        }
                        if let (Some(x_key), Some(y_key)) = (&self.x_key, &self.y_key) {
                            DataEvent::LimitRequest(super::limits::LimitRequest::ShowRectangle {
                                x_key: x_key.clone(),
                                y_key: y_key.clone(),
                                rectangle: heatmap_state.currently_showing(),
                            })
                        } else {
                            continue;
                        }
                    }
                    egui_heatmap::Event::Selection => {
                        DataEvent::SelectionRequest(super::selection::SelectionRequest::Selected(
                            heatmap_state.selected().clone(),
                        ))
                    }
                };
                state.data_events.push(event);
            }
        }
    }
}
impl super::TabTrait for HeatmapTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr { english: "Heatmap" }.localize(state.language)
    }
    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        if let &HeatmapState::Recompute = &self.state {
            self.state = self.recompute(state);
        }

        ui.vertical(|ui| {
            if state.ui_selectable_limit(ui, &mut self.to_show) {
                self.state = HeatmapState::Recompute;
            }
            ui.with_layout(
                egui::Layout::bottom_up(egui::Align::Min).with_cross_justify(true),
                |ui| {
                    let before = (self.x_key.clone(), self.y_key.clone());
                    ui.horizontal(|ui| {
                        let int_limits = state
                            .limits
                            .iter()
                            .filter(|(_, l)| l.is_int() && !l.is_trivial())
                            .collect::<Vec<_>>();
                        let mut needs_recompute = Self::axis_selection(
                            &mut self.x_key,
                            LocalizableStr {
                                english: "Select X-Axis",
                            },
                            state,
                            ui,
                            &int_limits,
                            self.y_key.as_ref(),
                        );
                        needs_recompute |= Self::axis_selection(
                            &mut self.y_key,
                            LocalizableStr {
                                english: "Select Y-Axis",
                            },
                            state,
                            ui,
                            &int_limits,
                            self.x_key.as_ref(),
                        );
                        if needs_recompute {
                            self.state.needs_recompute();
                        }
                    });
                    if (self.x_key.clone(), self.y_key.clone()) != before {
                        self.state = HeatmapState::Recompute;
                    }
                    match &mut self.state {
                        HeatmapState::Recompute => {
                            ui.label(
                            LocalizableString {
                                english:
                                    "Please select limits for both visualization, x-axis and y-axis"
                                        .into(),
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
                        HeatmapState::Heatmap(heatmap_with_state) => {
                            let heatmap_with_state: &mut (_, _) = &mut *heatmap_with_state;
                            let (heatmap, heatmap_state) = heatmap_with_state;
                            if let Some(problem) = heatmap_state.render_problem() {
                                ui.label(
                                    egui::RichText::new(format!("Rendering issue: {problem:?}"))
                                        .color(egui::Color32::WHITE)
                                        .background_color(egui::Color32::RED),
                                );
                            }
                            let label = match heatmap_state.hover() {
                                egui_heatmap::MultiMapPosition::NotHovering => LocalizableString {
                                    english: "Mouse not above heatmap".into(),
                                },
                                egui_heatmap::MultiMapPosition::NoData(
                                    file_key,
                                    CoordinatePoint { x, y },
                                ) => {
                                    let file = state
                                        .files
                                        .get(file_key)
                                        .and_then(|x| x.get_loaded())
                                        .map(|l| l.0.as_str())
                                        .unwrap_or(
                                            LocalizableStr {
                                                english: "File does not exist",
                                            }
                                            .localize(state.language),
                                        );
                                    LocalizableString {
                                        english: format!("{file}: {x}/{y} - no data"),
                                    }
                                }
                                egui_heatmap::MultiMapPosition::Pixel(
                                    file_key,
                                    CoordinatePoint { x, y },
                                ) => {
                                    let file = state
                                        .files
                                        .get(file_key)
                                        .and_then(|x| x.get_loaded())
                                        .map(|l| l.0.as_str())
                                        .unwrap_or(
                                            LocalizableStr {
                                                english: "File does not exist",
                                            }
                                            .localize(state.language),
                                        );
                                    LocalizableString {
                                        english: format!("{file}: {x}/{y}"),
                                    }
                                }
                                egui_heatmap::MultiMapPosition::Colorbar(f) => LocalizableString {
                                    english: format!("Colorbar: {f}"),
                                },
                            };
                            ui.label(label.localize(state.language));
                            heatmap.ui(ui, heatmap_state)
                        }
                        HeatmapState::Error(msg) => {
                            ui.label(msg.as_str().localize(state.language));
                            ui.heading(
                                LocalizableStr { english: "ERROR" }.localize(state.language),
                            );
                        }
                    }
                },
            );
        });
    }
}

impl HeatmapTab {
    fn recompute(&mut self, state: &mut super::AppState) -> HeatmapState {
        let x = check_key(&mut self.x_key, state);
        let y = check_key(&mut self.y_key, state);

        if let (
            Some((x_key, min_x, max_x)),
            Some((y_key, min_y, max_y)),
            Some((limit_key, limit)),
        ) = (
            x,
            y,
            self.to_show
                .get(state.locked_limits)
                .1
                .and_then(|k| state.limits.get(k).map(|l| (k, l))),
        ) {
            let (mut min_vis, mut max_vis) = limit.get_limits();
            let mut data = Vec::new();
            // find files which need to be drawn, and compute limits (if non are given, min/max will be used)
            for (file_key, (file_label, file, sorting)) in state.files.iter_loaded() {
                let filtering = state.total_filterings.get(file_key);
                let vis_data = sorting
                    .get(limit_key)
                    .map(|column| file.get_column(*column));
                let x_data = sorting
                    .get(&x_key)
                    .and_then(|column| file.get_column(*column).as_int());
                let y_data = sorting
                    .get(&y_key)
                    .and_then(|column| file.get_column(*column).as_int());
                if let (Some(filtering), Some(vis_data), Some(x_data), Some(y_data)) =
                    (filtering, vis_data, x_data, y_data)
                {
                    let filtered = vis_data.simple_filter(filtering);
                    if filtered.is_empty() {
                        continue;
                    }
                    {
                        let min_f = *filtered.iter().min().expect("Empty-case already covered");
                        let min_vis = min_vis.get_or_insert(min_f);
                        *min_vis = std::cmp::min(*min_vis, min_f);
                    }
                    {
                        let max_f = *filtered.iter().max().expect("Empty-case already covered");
                        let max_vis = max_vis.get_or_insert(max_f);
                        *max_vis = std::cmp::max(*max_vis, max_f);
                    }
                    data.push((
                        file_key.clone(),
                        filtering,
                        vis_data,
                        x_data,
                        y_data,
                        file_label,
                    ));
                }
            }
            if min_vis.is_none() || max_vis.is_none() {
                return HeatmapState::Error(LocalizableString {
                    english: "No data after filtering - check limits".into(),
                });
            }
            let min_vis = min_vis.unwrap().inner();
            let max_vis = max_vis.unwrap().inner();
            let delta_vis = max_vis - min_vis;
            // compute data
            let width = (max_x - min_x + 1) as usize;
            let height = (max_y - min_y + 1) as usize;
            let gradient = egui_heatmap::colors::Gradient::with_options(
                &egui_heatmap::colors::ColorGradientOptions::StartCenterEnd {
                    start: egui::Color32::BLUE,
                    center: egui::Color32::GREEN,
                    end: egui::Color32::RED,
                    steps: 63,
                },
            );
            let filtered_color = egui::Color32::GRAY;
            let background_color = egui::Color32::BLACK;
            let first_point_coordinate = egui_heatmap::CoordinatePoint { x: min_x, y: min_y };
            let data = data
                .into_iter()
                .map(|(key, filtering, vis_data, x_data, y_data, label)| {
                    let mut data = vec![background_color; width * height];
                    for (((&x, &y), vis), filter) in x_data
                        .iter()
                        .zip(y_data.iter())
                        .zip(vis_data.iter_float())
                        .zip(filtering.iter().map(|&f| f == 0))
                    {
                        let i = {
                            let x = (x - min_x) as usize;
                            let y = (y - min_y) as usize;
                            x + y * width
                        };
                        data[i] = if filter {
                            let vis = (vis - min_vis) / delta_vis;
                            gradient.lookup_color(vis)
                        } else {
                            filtered_color
                        };
                    }
                    (
                        key,
                        egui_heatmap::Data {
                            width,
                            height,
                            data,
                            first_point_coordinate: first_point_coordinate.clone(),
                            overlay: egui_heatmap::Overlay::new(
                                egui_heatmap::FontOptions {
                                    font: egui_heatmap::Font::EguiMonospace,
                                    background_is_transparent: true,
                                    font_height: 18.,
                                },
                                true,
                                Default::default(),
                                label.as_str(),
                            )
                            .unwrap(),
                        },
                    )
                })
                .collect::<Vec<_>>();

            let settings = egui_heatmap::MultiBitmapWidgetSettings {
                start_size: None,
                id: "HeatmapID".into(),
                boundary_between_data: egui_heatmap::ColorWithThickness {
                    color: egui::Color32::DARK_GRAY,
                    thickness: 5,
                },
                colorbar: Some((gradient, 100, (min_vis, max_vis))),
                background: background_color,
                boundary_unselected: egui_heatmap::ColorWithThickness {
                    color: egui::Color32::BROWN,
                    thickness: 3,
                },
                boundary_selected: egui::Color32::WHITE,
                boundary_factor_min: 7,
            };

            let heatmap = egui_heatmap::MultiBitmapWidget::with_settings(data, settings);
            let state = heatmap.default_state_english();
            HeatmapState::Heatmap((heatmap, state).into())
        } else {
            HeatmapState::Error(LocalizableString {
                english: "Please check limits both plot, x-axis and y-axis".into(),
            })
        }
    }
    #[must_use]
    fn axis_selection(
        value: &mut Option<LimitKey>,
        axis_selection_text: LocalizableStr,
        state: &super::AppState,
        ui: &mut egui::Ui,
        int_limits: &[(&LimitKey, &super::limits::Limit)],
        other: Option<&LimitKey>,
    ) -> bool {
        let mut needs_recompute = false;
        let axis_selection_text = axis_selection_text.localize(state.language);
        ui.label(axis_selection_text);
        if value.is_none() {
            *value = int_limits
                .iter()
                .filter(|(k, _)| Some(k) != other.as_ref())
                .map(|(key, _)| key)
                .next()
                .cloned()
                .cloned();
        }
        let selected_label = if let Some(key) = value.as_ref() {
            if let Some(limit) = state.limits.get(key) {
                limit.get_label().as_str()
            } else {
                axis_selection_text
            }
        } else {
            axis_selection_text
        };
        if value
            .as_ref()
            .map(|x| int_limits.iter().any(|(k, _)| k == &x))
            != Some(true)
        {
            *value = None;
        }
        if int_limits.is_empty() {
            ui.label(
                LocalizableStr {
                    english: "No integer limits available",
                }
                .localize(state.language),
            );
        } else {
            egui::ComboBox::from_id_source(axis_selection_text)
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for (key, limit) in int_limits {
                        let key: &LimitKey = key;
                        let previous = value.clone();
                        ui.selectable_value(value, Some(key.clone()), limit.get_label().as_str());
                        if &previous != value {
                            needs_recompute = true;
                        }
                    }
                });
        }
        needs_recompute
    }

    fn needs_recompute(&mut self) {
        self.state.needs_recompute()
    }
}

#[must_use]
fn check_key(key: &mut Option<LimitKey>, state: &super::AppState) -> Option<(LimitKey, i32, i32)> {
    if let Some(limit_key) = key.as_ref() {
        if let Some(limit) = state.limits.get(limit_key) {
            if let LimitDataKind::Int {
                uniques: _,
                min,
                max,
            } = &limit.data_kind()
            {
                Some((limit_key.clone(), *min, *max))
            } else {
                *key = None;
                None
            }
        } else {
            *key = None;
            None
        }
    } else {
        None
    }
}
