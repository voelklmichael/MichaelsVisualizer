#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod violin_data;

// hide console window on Windows in release
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Michael Visualizer",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}
struct MyApp {
    data: Vec<(String, data_format::DataFormat)>,
    status: String,
    new_data_label: String,
    new_data_row_count: String,
    limits: Vec<Limit>,
}
struct Limit {
    label: String,
    lower_string: String,
    lower_value: Option<f32>,
    upper_string: String,
    upper_value: Option<f32>,
    is_selected: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut app = Self {
            data: Default::default(),
            status: "App started".into(),
            new_data_label: Default::default(),
            new_data_row_count: Default::default(),
            limits: Default::default(),
        };
        app.add_dataset(
            "DataSet 1".into(),
            data_format::DataFormat::example_rectangle_simple(10),
        );
        app.add_dataset(
            "DataSet 2".into(),
            data_format::DataFormat::example_rectangle_simple(200),
        );
        app
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::bottom_up(egui::Align::Min).with_cross_justify(true),
                |ui| {
                    ui.label(&self.status);
                    egui_extras::StripBuilder::new(ui)
                        .size(egui_extras::Size::relative(0.3).at_most(200.0))
                        .size(egui_extras::Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::TOP)
                                        .with_cross_justify(true),
                                    |ui| {
                                        ui.vertical(|ui| {
                                            self.add_new_dataset(ui);
                                            ui.separator();
                                            self.show_availale_datasets(ui);
                                            ui.separator();
                                            self.show_limits(ui);
                                        });
                                    },
                                );
                            });
                            strip.cell(|ui| {
                                self.show_plot(ui);
                            });
                        });
                },
            );
        });
    }
}

impl MyApp {
    fn add_new_dataset(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui.button("Add Dataset").clicked() {
                match self.new_data_row_count.parse::<usize>() {
                    Ok(rows) => {
                        self.add_dataset(
                            self.new_data_label.clone(),
                            data_format::DataFormat::example_rectangle_simple(rows),
                        );
                        self.new_data_label.clear();
                        self.new_data_row_count.clear();
                    }
                    Err(e) => self.status = format!("{e:?}"),
                }
            }
            egui::Grid::new("Data_Selector_Add_Grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Label");
                    ui.text_edit_singleline(&mut self.new_data_label);
                    ui.end_row();
                    ui.label("Rows");
                    ui.text_edit_singleline(&mut self.new_data_row_count);
                    ui.end_row();
                });
        });
    }

    fn show_availale_datasets(&mut self, ui: &mut egui::Ui) {
        ui.push_id("DataSet_Selector_ScrollArea", |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                egui::Grid::new("DataSet_Selector_Grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        let mut to_remove = None;
                        ui.heading("Remove?");
                        ui.heading("Label");
                        ui.heading("Row count");
                        ui.end_row();
                        for (index, (label, data)) in self.data.iter().enumerate() {
                            if ui.button("Remove").clicked() {
                                to_remove = Some(index);
                            }
                            ui.label(label);
                            ui.label(&data.row_count().to_string());
                            ui.end_row()
                        }
                        if let Some(index) = to_remove {
                            self.remove_dataset(index);
                        }
                    });
            })
        });
    }

    fn show_limits(&mut self, ui: &mut egui::Ui) {
        ui.push_id("DataSet_Limits_ScrollArea", |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                egui::Grid::new("DataSet_Limits_Grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.heading("Label");
                        ui.heading("Plot?");
                        ui.heading("Lower");
                        ui.heading("Upper");
                        ui.end_row();
                        let mut selected = None;
                        let mut limits_changed = false;
                        for (
                            index,
                            Limit {
                                label,
                                lower_string,
                                lower_value,
                                upper_string,
                                upper_value,
                                is_selected,
                            },
                        ) in self.limits.iter_mut().enumerate()
                        {
                            ui.label(label.as_str());
                            if ui.button(if *is_selected { "x" } else { " " }).clicked() {
                                *is_selected = !*is_selected;
                                selected = Some(index);
                            }
                            match lower_string.parse::<f32>() {
                                Ok(lower) => {
                                    if lower_value != &Some(lower) {
                                        *lower_value = Some(lower);
                                        limits_changed = true;
                                    }
                                    ui.text_edit_singleline(lower_string);
                                }
                                Err(_) => {
                                    *lower_value = None;
                                    egui::Widget::ui(
                                        egui::TextEdit::singleline(lower_string)
                                            .text_color(egui::Color32::RED),
                                        ui,
                                    );
                                }
                            }
                            match upper_string.parse::<f32>() {
                                Ok(upper) => {
                                    if upper_value != &Some(upper) {
                                        *upper_value = Some(upper);
                                        limits_changed = true;
                                    }
                                    ui.text_edit_singleline(upper_string);
                                }
                                Err(_) => {
                                    *upper_value = None;
                                    egui::Widget::ui(
                                        egui::TextEdit::singleline(upper_string)
                                            .text_color(egui::Color32::RED),
                                        ui,
                                    );
                                }
                            }
                            ui.end_row()
                        }
                        if let Some(selected) = selected {
                            self.limit_selected(selected);
                        }
                        if limits_changed {
                            self.update_plot();
                        }
                    });
            })
        });
    }

    fn remove_dataset(&mut self, index: usize) {
        if index >= self.data.len() {
            self.status =
                "Failed to remove dataset - internal error - this should never happen".into();
        } else {
            let _ = self.data.remove(index);
            self.update_limits();
        }
    }

    fn update_limits(&mut self) {
        for (_, data) in &self.data {
            for label in data.header() {
                if !self.limits.iter().any(|l| &l.label == label) {
                    self.limits.push(Limit {
                        label: label.clone(),
                        lower_string: Default::default(),
                        lower_value: Default::default(),
                        upper_string: Default::default(),
                        upper_value: Default::default(),
                        is_selected: false,
                    });
                }
            }
        }
    }

    fn add_dataset(&mut self, label: String, data: data_format::DataFormat) {
        self.data.push((label, data));
        self.update_limits();
    }

    fn limit_selected(&mut self, selected: usize) {
        for (index, limit) in self.limits.iter_mut().enumerate() {
            if index != selected {
                limit.is_selected = false;
            }
        }
        self.update_plot();
    }

    fn update_plot(&mut self) {
        self.status = "TODO!".into();
        dbg!("TODO");
    }

    fn show_plot(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            egui::Sense::click_and_drag(),
        );
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
            egui::Color32::WHITE,
        ));

        let original_size = response.rect.size();
        let fontsize = 16.;
        let grid_line_thickness = 1.;
        let x_labels: Vec<&str> = vec!["A", "AEBAEIEBa0000000000000000", "atan Dist", "GAUSS"];
        let y_positions = [0.1, 0.5, 0.9];
        let y_labels = vec!["-3.135", "0", "-3E+34"];
        let axis_color = egui::Color32::BLACK;
        let fontid = egui::FontId::proportional(fontsize);
        let x_labels = x_labels
            .into_iter()
            .map(|t| painter.layout_no_wrap(t.into(), fontid.clone(), axis_color))
            .collect::<Vec<_>>();
        let y_labels = y_labels
            .into_iter()
            .map(|t| painter.layout_no_wrap(t.into(), fontid.clone(), axis_color))
            .collect::<Vec<_>>();
        let (show_y_axis, x_offset) = {
            let x_offset = y_labels
                .iter()
                .map(|x| x.size().x)
                .fold(0., |p, n| if p < n { n } else { p });
            let total_height = y_labels.iter().map(|x| x.size().y).sum::<f32>();
            if x_offset > original_size.x / 3. || total_height * 2. > original_size.y {
                (false, 0.)
            } else {
                (true, x_offset)
            }
        };
        let x_length = original_size.x - x_offset;
        let (show_x_axis, y_offset) = 'compute_angle: {
            let x_size = x_length / x_labels.len() as f32;
            let mut y_offset = 0.;
            let mut angles = Vec::with_capacity(x_labels.len());
            for x_label in &x_labels {
                let egui::Vec2 { x, y } = x_label.size();
                angles.push(if x < x_size {
                    y_offset = if y_offset < y { y } else { y_offset };
                    0.
                } else {
                    let angle = optimize(x, y, x_size);
                    let y = y * angle.cos() + x * angle.sin();
                    if y > original_size.y * 0.8 {
                        break 'compute_angle (None, 0.);
                    }
                    y_offset = if y_offset < y { y } else { y_offset };
                    angle
                });
            }
            (Some(angles), y_offset)
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
                let y_pos = (y_pos - y_label.size().y / y_length / 2.).clamp(0., 1.);
                let text_pos = egui::pos2(x / original_size.x, y_pos * y_length_ratio);
                painter.add(egui::Shape::galley(to_outer_screen * text_pos, y_label));
            }
        }
        // draw x-axis labels
        if let Some(angles) = show_x_axis {
            let y_length_ratio = (original_size.y - y_offset) / original_size.y;
            let count = x_labels.len();
            for (x_pos, (x_label, angle)) in
                x_labels.into_iter().zip(angles.into_iter()).enumerate()
            {
                if angle == 0. {
                    let x_pos = (2 * x_pos + 1) as f32 / (2 * count) as f32;
                    let x_pos = x_offset + x_pos * x_length;
                    let x_pos = x_pos - x_label.size().x / 2.;
                    let x_pos = x_pos.clamp(x_offset, original_size.x);
                    let text_pos = egui::pos2(x_pos / original_size.x, y_length_ratio);
                    painter.add(egui::Shape::galley(to_outer_screen * text_pos, x_label));
                } else {
                    let x_pos = x_pos as f32 / count as f32;
                    let x_pos = x_offset + x_pos * x_length;
                    let text_pos = egui::pos2(x_pos / original_size.x, y_length_ratio);
                    let text_pos = to_outer_screen * text_pos;
                    let s = egui::Shape::Text(egui::epaint::TextShape {
                        pos: text_pos,
                        galley: x_label,
                        underline: egui::Stroke::NONE,
                        override_text_color: None,
                        angle,
                    });
                    painter.add(s);
                }
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
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        ));
        let datasets = vec![
            (
                violin_data::ViolinData::construct(
                    &violin_data::ExampleData::zero_p_five(10000),
                    0.,
                    1.,
                    33,
                ),
                egui::Color32::RED,
            ),
            (
                violin_data::ViolinData::construct(
                    &violin_data::ExampleData::linear(-1., 2., 10000),
                    0.,
                    1.,
                    33,
                ),
                egui::Color32::BLUE,
            ),
            (
                violin_data::ViolinData::construct(
                    &violin_data::ExampleData::atan_distribution(-1., 1., 10000),
                    -1.,
                    1.,
                    33,
                ),
                egui::Color32::DARK_RED,
            ),
            (
                violin_data::ViolinData::construct(
                    &violin_data::ExampleData::gauss(0.5, 0.3, -1.5, 1.5, 10000),
                    0.,
                    1.,
                    33,
                ),
                egui::Color32::DARK_GREEN,
            ),
        ];
        let _m = datasets.iter().fold(0., |p, (d, _)| {
            let m = d.max_bin;
            if m < p {
                p
            } else {
                m
            }
        });
        let n = datasets.len();
        for (index, (d, c)) in datasets.into_iter().enumerate() {
            painter.extend(d.to_shapes(c, to_inner_screen, index, n, None));
        }
    }
}

fn optimize(a: f32, b: f32, c: f32) -> f32 {
    let val = |angle: f32| ((a * angle.cos() + b * angle.sin()) / c - 1.);
    let mut max = std::f32::consts::FRAC_PI_2;
    let mut min = 0.;
    let mut counter = 0;
    loop {
        let angle = (max + min) / 2.;
        let e = val(angle);
        if e.abs() < 1E-4 || counter > 10 {
            break angle;
        } else if e < 0. {
            max = angle;
        } else {
            min = angle;
        }
        counter += 1;
    }
}

#[test]
fn optimize_test() {
    let a = dbg!(optimize(223., 18., 182.));
    let b = 0.70122105;
    assert!((a - b).abs() < 1e-4);
}
