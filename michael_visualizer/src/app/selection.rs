use crate::{
    data_types::{finite_f32::FiniteF32, LimitKey},
    Language, LocalizableStr,
};

use super::DataEvent;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Selection {
    pub x_key: LimitKey,
    pub y_key: LimitKey,
    pub selected: std::collections::HashSet<egui_heatmap::CoordinatePoint>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum SelectionRequest {
    UnselectAll,
    Selection(Selection),
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum SelectionEvent {
    UnselectAll,
    Selection(Selection),
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct SelectionTab {
    transpose: bool,
}
impl SelectionTab {
    fn add_context_menu(&mut self, mut response: egui::Response, language: Language) {
        response.sense = egui::Sense::click();
        response.context_menu(|ui| {
            let before = self.transpose;
            ui.checkbox(
                &mut self.transpose,
                LocalizableStr {
                    english: "Transpose",
                }
                .localize(language),
            );
            if self.transpose != before {
                ui.close_menu();
            }
        });
    }

    fn label(&mut self, ui: &mut egui_extras::TableRow, text: &str, state: &super::AppState) {
        let response = ui
            .col(|ui| {
                let label = egui::Label::new(text);
                let label = label.sense(egui::Sense::click());
                let response = ui.add(label);
                self.add_context_menu(response, state.language);
            })
            .1;
        self.add_context_menu(response, state.language);
    }
}
impl super::DataEventNotifyable for SelectionTab {
    fn notify(&mut self, _event: &DataEvent) -> Vec<DataEvent> {
        Default::default()
    }

    fn progress(&mut self, _state: &mut super::AppState) {}
}
impl super::TabTrait for SelectionTab {
    fn title(&self, state: &super::AppState) -> &str {
        LocalizableStr {
            english: "Selection",
        }
        .localize(state.language)
    }

    fn show(&mut self, state: &mut super::AppState, ui: &mut egui::Ui) {
        if let Some(Selection {
            x_key,
            y_key,
            selected,
        }) = &state.selected
        {
            let mut columns = std::collections::VecDeque::with_capacity(selected.len());
            for (selection_index, egui_heatmap::CoordinatePoint { x: xx, y: yy }) in
                selected.iter().enumerate()
            {
                for (_, (file_label, data, limit_sorting)) in state.files.iter_loaded() {
                    if let (Some(x_column), Some(y_column)) =
                        (limit_sorting.get(x_key), limit_sorting.get(y_key))
                    {
                        let x_data = data.get_column(*x_column).as_int();
                        let y_data = data.get_column(*y_column).as_int();
                        if let (Some(x_data), Some(y_data)) = (x_data, y_data) {
                            if let Some(index) = x_data
                                .iter()
                                .zip(y_data.iter())
                                .position(|(x, y)| x == xx && y == yy)
                            {
                                let mut column = vec![None; state.limits.len()];
                                for (row, (limit_key, _)) in state.limits.iter().enumerate() {
                                    if let Some(column_index) = limit_sorting.get(limit_key) {
                                        let data = data.get_column(*column_index);
                                        column[row] = Some(data.get_as_float(index));
                                    }
                                }
                                columns.push_back((file_label, selection_index, column));
                            }
                        }
                    }
                }
            }
            let header_height: f32 = 18.;
            let row_height: f32 = 16.;
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                if self.transpose {
                    let galleys = {
                        let mut labels = Vec::with_capacity(columns.len() + 1);
                        labels.push(LocalizableStr { english: "Limit" }.localize(state.language));
                        columns.iter().for_each(|(f, _, _)| labels.push(f.as_str()));
                        super::_helper::galleys(labels, ui, header_height)
                    };
                    egui_extras::TableBuilder::new(ui)
                        .columns(
                            egui_extras::Column::auto().resizable(true),
                            columns.len() + 1,
                        )
                        .striped(true)
                        .header(header_height, |mut header| {
                            let max_y = galleys
                                .iter()
                                .filter_map(|x| FiniteF32::try_from(x.size().x).ok())
                                .max()
                                .unwrap_or(FiniteF32::new(1.))
                                .inner();
                            for galley in galleys {
                                header.col(|ui| {
                                    let unrotated = galley.size();
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(unrotated.y, max_y),
                                        egui::Sense::click(),
                                    );

                                    let text_shape = egui::Shape::Text(egui::epaint::TextShape {
                                        pos: rect.left_bottom(),
                                        galley,
                                        underline: egui::Stroke::NONE,
                                        override_text_color: None,
                                        angle: -std::f32::consts::FRAC_PI_2,
                                    });
                                    ui.painter().add(text_shape);
                                    self.add_context_menu(response, state.language)
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(row_height, state.limits.len(), |row, mut ui| {
                                // First column - limit label
                                self.label(
                                    &mut ui,
                                    if let Some(limit) = state.limits.element_at(row) {
                                        limit.get_label().as_str()
                                    } else {
                                        "This should never happen"
                                    },
                                    state,
                                );
                                // following columns - data
                                for (_, _, d) in &columns {
                                    if let Some(d) = d[row] {
                                        self.label(&mut ui, &format!("{d}"), state);
                                    } else {
                                        self.label(
                                            &mut ui,
                                            LocalizableStr { english: "n/a" }
                                                .localize(state.language),
                                            state,
                                        );
                                    }
                                }
                            })
                        });
                } else {
                    let limits = &state.limits;
                    let galleys = {
                        let mut labels = Vec::with_capacity(columns.len() + 1);
                        labels.push(LocalizableStr { english: "File" }.localize(state.language));
                        limits
                            .iter()
                            .for_each(|(_, limit)| labels.push(limit.get_label().as_str()));
                        super::_helper::galleys(labels, ui, header_height)
                    };
                    egui_extras::TableBuilder::new(ui)
                        .columns(egui_extras::Column::auto().resizable(true), galleys.len())
                        .striped(true)
                        .header(header_height, |mut header| {
                            let max_y = galleys
                                .iter()
                                .filter_map(|x| FiniteF32::try_from(x.size().x).ok())
                                .max()
                                .unwrap_or(FiniteF32::new(1.))
                                .inner();
                            for galley in galleys {
                                header.col(|ui| {
                                    let unrotated = galley.size();
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(unrotated.y, max_y),
                                        egui::Sense::click(),
                                    );

                                    let text_shape = egui::Shape::Text(egui::epaint::TextShape {
                                        pos: rect.left_bottom(),
                                        galley,
                                        underline: egui::Stroke::NONE,
                                        override_text_color: None,
                                        angle: -std::f32::consts::FRAC_PI_2,
                                    });
                                    ui.painter().add(text_shape);
                                    self.add_context_menu(response, state.language)
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(row_height, columns.len(), |row, mut ui| {
                                let (file, _, data) = &columns[row];
                                // First column - file label
                                let text = file.as_str();
                                self.label(&mut ui, text, state);
                                // following columns - data
                                for d in data {
                                    if let Some(d) = d {
                                        self.label(&mut ui, &format!("{d}"), state);
                                    } else {
                                        self.label(
                                            &mut ui,
                                            LocalizableStr { english: "n/a" }
                                                .localize(state.language),
                                            state,
                                        );
                                    }
                                }
                            })
                        });
                }
            });
        } else {
            ui.label(
                LocalizableStr {
                    english: "No positions are selected",
                }
                .localize(state.language),
            );
        }
    }
}
