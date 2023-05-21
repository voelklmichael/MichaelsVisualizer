#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::TOP).with_cross_justify(true),
                        |ui| {
                            ui.vertical(|ui| {
                                self.add_new_dataset(ui);
                                ui.separator();
                                self.show_availale_datasets(ui);
                                ui.separator();
                                self.show_limits(ui);
                            });
                            self.show_plot(ui);
                        },
                    );
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
        if self.data.len() >= index {
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
        use egui::plot::{BoxElem, BoxPlot, BoxSpread, Legend, Plot};
        use egui::{Color32, Stroke};
        let yellow = Color32::from_rgb(248, 252, 168);
        let mut box1 = BoxPlot::new(vec![
            BoxElem::new(0.5, BoxSpread::new(1.5, 2.2, 2.5, 2.6, 3.1)).name("Day 1"),
            BoxElem::new(2.5, BoxSpread::new(0.4, 1.0, 1.1, 1.4, 2.1)).name("Day 2"),
            BoxElem::new(4.5, BoxSpread::new(1.7, 2.0, 2.2, 2.5, 2.9)).name("Day 3"),
        ])
        .name("Experiment A");

        let mut box2 = BoxPlot::new(vec![
            BoxElem::new(1.0, BoxSpread::new(0.2, 0.5, 1.0, 2.0, 2.7)).name("Day 1"),
            BoxElem::new(3.0, BoxSpread::new(1.5, 1.7, 2.1, 2.9, 3.3))
                .name("Day 2: interesting")
                .stroke(Stroke::new(1.5, yellow))
                .fill(yellow.linear_multiply(0.2)),
            BoxElem::new(5.0, BoxSpread::new(1.3, 2.0, 2.3, 2.9, 4.0)).name("Day 3"),
        ])
        .name("Experiment B");

        let mut box3 = BoxPlot::new(vec![
            BoxElem::new(1.5, BoxSpread::new(2.1, 2.2, 2.6, 2.8, 3.0)).name("Day 1"),
            BoxElem::new(3.5, BoxSpread::new(1.3, 1.5, 1.9, 2.2, 2.4)).name("Day 2"),
            BoxElem::new(5.5, BoxSpread::new(0.2, 0.4, 1.0, 1.3, 1.5)).name("Day 3"),
        ])
        .name("Experiment C");

        /*if !self.vertical {
            box1 = box1.horizontal();
            box2 = box2.horizontal();
            box3 = box3.horizontal();
        }*/

        Plot::new("Box Plot Demo")
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.box_plot(box1);
                plot_ui.box_plot(box2);
                plot_ui.box_plot(box3);
            });
    }
}
