use crate::{
    app::{files::FileEvent, limits::LimitEvent, DataEvent},
    data_types::finite_f32::FiniteF32,
    dialog::Dialog,
    LocalizableStr, LocalizableString,
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViolinTab {
    resolution: usize,
    #[serde(skip)]
    state: State,
}
impl Default for ViolinTab {
    fn default() -> Self {
        Self {
            resolution: 31,
            state: Default::default(),
        }
    }
}
impl super::DataEventNotifyable for ViolinTab {
    fn notify(&mut self, event: &DataEvent) -> Vec<DataEvent> {
        let (needs_recompute, events) = self.state.notify(event);
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
                            state.data_events.push(super::DataEvent::Limit(
                                super::limits::LimitEvent::RequestLabel(plot.limit_key.clone(), s),
                            ))
                        }
                    }
                }
            }
        }
    }
}
struct ViolinPlot {
    // plot data
    limit_key: crate::data_types::LimitKey,
    limit_label: crate::data_types::LimitLabel,
    min: FiniteF32,
    max: FiniteF32,
    entries: Vec<ViolinEntry>,
    // user input
    context_pos: Option<(egui::Vec2, bool)>,
    limit_label_change_sender: std::sync::mpsc::Sender<LimitLabelChange>,
    limit_label_change_value: Option<String>,
    limit_label_change_receiver: std::sync::mpsc::Receiver<LimitLabelChange>,
}
enum LimitLabelChange {
    Change(String),
    Ok,
}
#[derive(PartialEq)]
enum Normalization {
    //FileByFile,
    SameForAllFiles,
}
impl ViolinPlot {
    fn show(&mut self, ui: &mut egui::Ui, state: &mut super::AppState) -> Vec<DataEvent> {
        let language = state.language;
        let background = egui::Color32::WHITE;
        let fontsize = 16.;
        let axis_color = egui::Color32::BLACK;
        let fontid = egui::FontId::proportional(fontsize);
        let y_steps_count = 5;
        let boundary_color = egui::Color32::BLACK;
        let boundary_thickness = 1.0;
        let normalization = Normalization::SameForAllFiles;

        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            egui::Sense::click_and_drag(),
        );
        let original_size = response.rect.size();

        let mouse = if let Some(mouse) = response.hover_pos() {
            let rect = response.rect;
            let mouse = mouse - rect.left_top();
            if mouse.x >= 0.
                && mouse.y >= 0.
                && mouse.x <= original_size.x
                && mouse.y <= original_size.y
            {
                mouse
            } else {
                egui::vec2(-1., -1.)
            }
        } else {
            egui::vec2(-1., -1.)
        };
        let mut mouse_above_limit_label = false;

        let to_outer_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1., 1.)),
            response.rect,
        );
        painter.add(egui::Shape::rect_filled(
            egui::Rect::from_min_max(
                to_outer_screen * egui::pos2(0., 0.),
                to_outer_screen * egui::pos2(1., 1.),
            ),
            egui::Rounding::none(),
            background,
        ));

        let grid_line_thickness = 1.;
        let x_labels = {
            self.entries
                .iter()
                .map(|t| &t.label)
                .map(|t| painter.layout_no_wrap(t.as_str().into(), fontid.clone(), axis_color))
                .collect::<Vec<_>>()
        };
        let y_positions = {
            (0..y_steps_count)
                .map(|x| 1. - 1. / (y_steps_count as f32 - 1.) * (x as f32))
                .collect::<Vec<_>>()
        };
        let y_labels = {
            (0..y_steps_count)
                .map(|x| {
                    self.min.inner()
                        + (self.max.inner() - self.min.inner()) / (y_steps_count as f32 - 1.)
                            * (x as f32)
                })
                .map(|t| painter.layout_no_wrap(t.to_string(), fontid.clone(), axis_color))
                .collect::<Vec<_>>()
        };
        let limit_label =
            painter.layout_no_wrap(self.limit_label.as_str().into(), fontid, axis_color);
        let (show_y_axis, x_offset, show_limit_label) = {
            let x_offset = y_labels
                .iter()
                .map(|x| x.size().x)
                .fold(0., |p, n| if p < n { n } else { p });
            let (x_offset, show_limit_label) = if limit_label.size().x < original_size.y {
                (x_offset + limit_label.size().y, true)
            } else {
                (x_offset, false)
            };
            let total_height = y_labels.iter().map(|x| x.size().y).sum::<f32>();
            if x_offset > original_size.x / 3. || total_height * 2. > original_size.y {
                (false, 0., false)
            } else {
                (true, x_offset, show_limit_label)
            }
        };
        let x_length = original_size.x - x_offset;
        let x_labels_placement = place_x_labels(&x_labels, x_length, x_offset);
        let y_offset = {
            x_labels_placement
                .as_ref()
                .and_then(|x| x.iter().map(|p| FiniteF32::new(p.y_size + p.y_top)).max())
                .map(|x| x.inner())
                .unwrap_or(0.)
        };
        // draw y-axis labels and grid
        if show_y_axis {
            let y_length = original_size.y - y_offset;
            let y_length_ratio = y_length / original_size.y;
            for (y_label, y_pos) in y_labels.into_iter().zip(y_positions.into_iter()) {
                let x = x_offset - y_label.size().x;
                painter.add(egui::Shape::line(
                    vec![
                        to_outer_screen
                            * egui::pos2(x_offset / original_size.x, y_pos * y_length_ratio),
                        to_outer_screen * egui::pos2(1., y_pos * y_length_ratio),
                    ],
                    egui::Stroke::new(grid_line_thickness, axis_color),
                ));
                if y_label.size().y / y_length >= 1. {
                    continue;
                }
                let y_pos = (y_pos - y_label.size().y / y_length / 2.)
                    .clamp(0., 1. - y_label.size().y / y_length);
                let text_pos = egui::pos2(x / original_size.x, y_pos * y_length_ratio);
                painter.add(egui::Shape::galley(to_outer_screen * text_pos, y_label));
            }
            // limit label
            if show_limit_label {
                let egui::Vec2 { x: lx, y: ly } = limit_label.size();
                let y = if y_length > lx {
                    if mouse.x.inside(0., ly)
                        && mouse
                            .y
                            .inside(y_length / 2. - lx / 2., y_length / 2. + lx / 2.)
                    {
                        mouse_above_limit_label = true;
                    }
                    y_length / 2. + lx / 2.
                } else {
                    if mouse.x.inside(0., ly) && mouse.y.inside(0., ly) {
                        mouse_above_limit_label = true;
                    }
                    lx
                };
                let y = y / original_size.y;
                let text_pos = to_outer_screen * egui::Pos2 { x: 0., y };
                let s = egui::Shape::Text(egui::epaint::TextShape {
                    pos: text_pos,
                    galley: limit_label,
                    underline: egui::Stroke::NONE,
                    override_text_color: None,
                    angle: -std::f32::consts::FRAC_PI_2,
                });
                painter.add(s);
            }
        }
        // draw x-axis labels
        if let Some(placements) = x_labels_placement {
            assert_eq!(placements.len(), x_labels.len());
            for (p, x_label) in placements.into_iter().zip(x_labels.into_iter()) {
                let text_pos = egui::Pos2 {
                    x: (p.x_left + x_offset) / original_size.x,
                    y: (original_size.y - (y_offset - p.y_top)) / original_size.y,
                };
                painter.add(egui::Shape::galley(to_outer_screen * text_pos, x_label));
            }
        }

        let to_inner_screen = {
            let mut changed = response.rect;
            *changed.left_mut() += x_offset;
            *changed.bottom_mut() -= y_offset;
            egui::emath::RectTransform::from_to(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1., 1.)),
                changed,
            )
        };
        painter.add(egui::Shape::rect_stroke(
            egui::Rect::from_min_max(
                to_inner_screen * egui::pos2(0., 0.),
                to_inner_screen * egui::pos2(1., 1.),
            ),
            egui::Rounding::none(),
            egui::Stroke::new(boundary_thickness, boundary_color),
        ));

        let normalization = if normalization == Normalization::SameForAllFiles {
            self.entries.iter().map(|e| e.max_bin).max()
        } else {
            None
        };
        let n = self.entries.len();
        fn get_color(i: usize) -> egui::Color32 {
            let colors = [
                egui::Color32::RED,
                egui::Color32::BLUE,
                egui::Color32::GREEN,
                egui::Color32::GOLD,
            ];
            colors[i % colors.len()]
        }

        for (index, d) in self.entries.iter().enumerate() {
            painter.extend(d.to_shapes(get_color(index), to_inner_screen, index, n, normalization));
        }
        let mut new = None;
        let previous = self.context_pos;
        response.context_menu(|ui| {
            new = Some(mouse);
            let id = egui::Id::new("LimitLabelChangeDialogViolinPlot");
            let mouse_above_limit_label = if previous.is_none() {
                ui.data_mut(|x| {
                    x.remove::<String>(id);
                });
                mouse_above_limit_label
            } else {
                previous.unwrap().1
            };
            if mouse_above_limit_label
                && ui
                    .button(
                        LocalizableStr {
                            english: "Change label",
                        }
                        .localize(language),
                    )
                    .clicked()
            {
                let label = self.limit_label.as_str().to_string();
                let label1 = label.clone();
                let s1 = self.limit_label_change_sender.clone();
                let s2 = self.limit_label_change_sender.clone();
                state
                    .app_events
                    .push(crate::app::AppEvent::Dialog(Dialog::new(
                        LocalizableString {
                            english: "Limit label".into(),
                        }
                        .localize(language),
                        Box::new(move |ui| {
                            ui.heading(
                                LocalizableStr {
                                    english: "Change limit label",
                                }
                                .localize(language),
                            );
                            ui.vertical(|ui| {
                                let label = label.clone();
                                let label1 = label1.clone();
                                ui.horizontal(|ui| {
                                    ui.label(
                                        LocalizableStr {
                                            english: "Current: ",
                                        }
                                        .localize(language),
                                    );
                                    ui.label(&label);
                                });
                                ui.horizontal(|ui| {
                                    ui.label(
                                        LocalizableStr { english: "New: " }.localize(language),
                                    );
                                    let label_before = ui.data_mut(|x| {
                                        x.get_temp_mut_or_insert_with::<String>(id, move || label1)
                                            .clone()
                                    });
                                    let mut label = label_before.clone();
                                    ui.text_edit_singleline(&mut label);
                                    if label != label_before {
                                        ui.data_mut(|x| {
                                            let t = x
                                                .get_temp_mut_or_insert_with::<String>(id, || {
                                                    label.clone()
                                                });
                                            *t = label.clone();
                                        });
                                        let _ = s2.send(LimitLabelChange::Change(label));
                                    }
                                });
                            });
                            false
                        }),
                        crate::dialog::DialogKind::Button {
                            buttons: vec![
                                crate::dialog::Button {
                                    label: LocalizableString {
                                        english: "Cancel".into(),
                                    }
                                    .localize(language),
                                    action: Box::new(|| true),
                                },
                                crate::dialog::Button {
                                    label: LocalizableString {
                                        english: "Ok".into(),
                                    }
                                    .localize(language),
                                    action: Box::new(move || {
                                        let _ = s1.send(LimitLabelChange::Ok);
                                        true
                                    }),
                                },
                            ],
                            has_exit: Some(0),
                        },
                    )));
                ui.close_menu();
            }
        });
        if new.is_none() {
            self.context_pos = None;
        } else if self.context_pos.is_none() {
            self.context_pos = new.map(|n| (n, mouse_above_limit_label));
        }

        Default::default()
    }
}

struct Placement {
    x_left: f32,
    y_top: f32,
    //x_size: f32,
    y_size: f32,
}
fn place_x_labels(
    x_labels: &[std::sync::Arc<egui::Galley>],
    x_length: f32,
    x_offset: f32,
) -> Option<Vec<Placement>> {
    let mut y_tops = Vec::new();
    let mut placements = Vec::new();
    let mut left_most_xs = Vec::new();
    'labels: for (index, label) in x_labels.iter().enumerate() {
        let egui::Vec2 { x: sx, y: sy } = label.size();
        let x_left = x_length / (2. * x_labels.len() as f32) * (2. * index as f32 + 1.) - sx / 2.;
        // try each row, one by one
        for row_index in 0..y_tops.len() + 1 {
            if left_most_xs.get(row_index).is_none() {
                left_most_xs.push(-x_offset);
            }
            let left_most_x = left_most_xs[row_index];
            if x_length - left_most_x < sx {
                // now we know: there is not enought space
                if y_tops.get(row_index).is_none() {
                    // this row is empty, so failed
                    return None;
                } else {
                    // try another row
                    continue;
                }
            }
            // there is enough space, check if we can place the label at the center
            let x_left = if x_left + sx < x_length {
                x_left
            } else {
                x_length - sx
            };
            let x_left = if x_left > left_most_x {
                x_left
            } else {
                left_most_x
            };
            // place
            let x_right = x_left + sx;
            if let Some(previous) = left_most_xs.get_mut(row_index) {
                if *previous < x_right {
                    *previous = x_right;
                }
            } else {
                left_most_xs.push(x_right);
            }
            if let Some(previous) = y_tops.get_mut(row_index) {
                if *previous < sy {
                    *previous = sy;
                }
            } else {
                y_tops.push(sy);
            }
            placements.push(Placement {
                x_left,
                y_top: (0..row_index).map(|i| y_tops[i]).sum(),
                //x_size: sx,
                y_size: sy,
            });
            continue 'labels;
        }
        unreachable!()
    }
    Some(placements)
}

#[derive(Default)]
enum State {
    #[default]
    NeedsRecompute,
    NoLimitSelected,
    Plot(ViolinPlot),
    Error(LocalizableString),
}
impl State {
    fn notify(&mut self, event: &DataEvent) -> (bool, Vec<DataEvent>) {
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
                    LimitEvent::ToShow(_) => affected,
                    LimitEvent::Label(_) => unaffected,
                    LimitEvent::Limit(_) => unaffected,
                    LimitEvent::New(_) => affected,
                    LimitEvent::RequestLabel(_, _) => unaffected,
                },
                _ => unaffected,
            },
            State::Plot(ViolinPlot {
                limit_key,
                limit_label: _,
                min: _,
                max: _,
                entries,
                context_pos: _,
                limit_label_change_sender: _,
                limit_label_change_receiver: _,
                limit_label_change_value: _,
            }) => match event {
                DataEvent::Limit(event) => match event {
                    LimitEvent::ToShow(key) => condition(key.as_ref() != Some(limit_key)),
                    LimitEvent::Label(key) => condition(key == limit_key),
                    LimitEvent::Limit(key) => condition(key == limit_key),
                    LimitEvent::New(_) => unaffected,
                    LimitEvent::RequestLabel(_, _) => unaffected,
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
                    FileEvent::Loaded(_, _) => affected,
                },
                DataEvent::Filtering => affected,
            },
            State::Error(_) => affected,
        }
    }
}

impl ViolinTab {
    fn recompute(&mut self, state: &super::AppState) -> State {
        if let Some(limit_key) = state.limits.to_show() {
            if let Some(limit) = state.limits.get(limit_key) {
                let super::limits::LimitData {
                    label: limit_label,
                    lower,
                    upper,
                    info: _,
                } = limit.data();
                let min = lower.unwrap_or(FiniteF32::new(f32::MIN));
                let max = upper.unwrap_or(FiniteF32::new(f32::MAX));
                let mut entries = Vec::new();
                for file_key in state.get_files_for_limit(limit_key) {
                    let filtering = state.total_filterings.get(file_key);
                    let file = state.files.get(file_key).and_then(|x| x.get_loaded());
                    if let (Some((label, file, sorting)), Some(filtering)) = (file, filtering) {
                        if let Some(column) = sorting.get(limit_key) {
                            let data = file.get_column(*column).data();
                            assert_eq!(data.len(), filtering.len());
                            let data = filtering
                                .iter()
                                .zip(data.iter())
                                .flat_map(|(&n, f)| {
                                    (n == 0 && f.is_finite() && f >= &min && f <= &max)
                                        .then_some(FiniteF32::new(*f))
                                })
                                .collect::<Vec<_>>();
                            entries.push((file_key.clone(), label.clone(), data));
                        }
                    }
                }
                let min: FiniteF32 = lower.unwrap_or_else(|| {
                    entries
                        .iter()
                        .flat_map(|(_, _, e)| e.first())
                        .min()
                        .cloned()
                        .unwrap_or(min)
                });
                let max: FiniteF32 = upper.unwrap_or_else(|| {
                    entries
                        .iter()
                        .flat_map(|(_, _, e)| e.last())
                        .max()
                        .cloned()
                        .unwrap_or(max)
                });
                let entries = entries
                    .into_iter()
                    .filter_map(|(key, label, data)| {
                        ViolinEntry::new(key, label, data, self.resolution, min, max)
                    })
                    .collect();
                let (s, r) = std::sync::mpsc::channel();
                State::Plot(ViolinPlot {
                    limit_key: limit_key.clone(),
                    limit_label,
                    min,
                    max,
                    entries,
                    context_pos: Default::default(),
                    limit_label_change_sender: s,
                    limit_label_change_receiver: r,
                    limit_label_change_value: Default::default(),
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

struct ViolinEntry {
    key: crate::data_types::FileKey,
    label: crate::data_types::FileLabel,
    //count: usize,
    //mean: f32,
    bins: Vec<u64>,
    max_bin: u64,
    mean_height: f32,
}

impl super::TabTrait for ViolinTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr { english: "Violin" }.localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
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
impl ViolinEntry {
    #[must_use]
    fn new(
        key: crate::data_types::FileKey,
        label: crate::data_types::FileLabel,
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
            key,
            label,
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
                let y = (2 * index + 1) as f32 / bin_count_twice;
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
                transform * egui::pos2(center, mean),
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
