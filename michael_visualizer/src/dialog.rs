use eframe::egui;

pub(super) struct Dialog {
    title: String,
    content: Box<dyn FnMut(&mut egui::Ui) -> bool>,
    kind: DialogKind,
    has_exit: Option<bool>,
    requested_focus: bool,
}

pub enum DialogKind {
    Progress {
        progress: Progress,
        cancelable: Option<Button>,
    },
    Button {
        buttons: Vec<Button>,
        has_exit: Option<usize>,
    },
}
pub enum Progress {
    Indeterminate(f32),
    Determinate {
        start: std::time::Instant,
        expectation: std::time::Duration,
        text: String,
    },
}
impl Progress {
    pub fn new(expectation: Option<std::time::Duration>) -> Self {
        if let Some(expectation) = expectation {
            Self::new_determinate(expectation)
        } else {
            Self::new_indeterminate()
        }
    }
    fn new_indeterminate() -> Self {
        Self::Indeterminate(0.)
    }
    fn new_determinate(expectation: std::time::Duration) -> Self {
        Self::Determinate {
            start: std::time::Instant::now(),
            expectation,
            text: format!("ETA: {s}s", s = expectation.as_secs()),
        }
    }
    fn current(&mut self) -> f32 {
        match self {
            Progress::Indeterminate(f) => {
                *f += 0.008;
                if *f > 1. {
                    *f = 0.;
                }
                *f
            }
            Progress::Determinate {
                start,
                expectation,
                text: _,
            } => {
                let used = std::time::Instant::now() - *start;
                if used > *expectation {
                    *self = Progress::Indeterminate(0.);
                    0.
                } else {
                    (used.as_secs_f32() / expectation.as_secs_f32()).clamp(0., 1.)
                }
            }
        }
    }
    fn show_percentage(&self) -> bool {
        match self {
            Progress::Indeterminate(_) => false,
            Progress::Determinate { .. } => true,
        }
    }
    fn text(&self) -> Option<&str> {
        match self {
            Progress::Indeterminate(_) => None,
            Progress::Determinate { text, .. } => Some(text),
        }
    }
}
pub struct Button {
    pub label: String,
    pub action: Box<dyn Fn() -> bool>,
}
impl Button {
    #[must_use]
    fn show(&self, ui: &mut egui::Ui, requested_focus: &mut bool) -> bool {
        let button = ui.button(&self.label);
        if !*requested_focus {
            button.request_focus();
            *requested_focus = true;
        }
        if button.clicked() {
            (self.action)()
        } else {
            false
        }
    }
}
impl DialogKind {
    #[must_use]
    fn show(&mut self, ui: &mut egui::Ui, requested_focus: &mut bool) -> bool {
        match self {
            DialogKind::Progress {
                cancelable,
                progress,
            } => {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_secs(2));
                ui.vertical(|ui| {
                    if let Some(text) = progress.text() {
                        ui.label(text);
                    }
                    let bar = egui::ProgressBar::new(progress.current());
                    let bar = if progress.show_percentage() {
                        bar.show_percentage()
                    } else {
                        bar
                    };
                    egui::Widget::ui(bar, ui);
                    if let Some(button) = cancelable {
                        button.show(ui, requested_focus)
                    } else {
                        false
                    }
                })
                .inner
            }
            DialogKind::Button {
                buttons,
                has_exit: _,
            } => {
                assert!(!buttons.is_empty(), "Dialog without buttons is not allowed");
                let response = ui.horizontal(|ui| {
                    for button in buttons {
                        if button.show(ui, requested_focus) {
                            return true;
                        }
                    }
                    false
                });
                response.inner
            }
        }
    }

    fn close(&self) -> bool {
        match self {
            DialogKind::Progress {
                cancelable,
                progress: _,
            } => {
                if let Some(Button { label: _, action }) = cancelable {
                    action()
                } else {
                    unreachable!("Some Progress dialog was misconfigured - close called, by 'cancelable' not set")
                }
            }
            DialogKind::Button { buttons, has_exit } => {
                if let Some(index) = has_exit {
                    if let Some(Button { label: _, action }) = buttons.get(*index) {
                        action()
                    } else {
                        panic!("Some Button dialog was misconfigured - close called, but button #{index} does not exist")
                    }
                } else {
                    unreachable!("Some Button dialog was misconfigured - close called, but 'has_exit' not set")
                }
            }
        }
    }

    fn has_exit(&self) -> bool {
        match self {
            DialogKind::Progress {
                progress: _,
                cancelable,
            } => cancelable.is_some(),
            DialogKind::Button {
                buttons: _,
                has_exit,
            } => has_exit.is_some(),
        }
    }
}

impl Dialog {
    pub(super) fn new(
        title: String,
        content: Box<dyn FnMut(&mut egui::Ui) -> bool>,
        kind: DialogKind,
    ) -> Self {
        Self {
            title,
            content,
            has_exit: kind.has_exit().then_some(true),
            kind,
            requested_focus: false,
        }
    }
    pub(super) fn example_progress(
        cancelable: bool,
        expectation: Option<std::time::Duration>,
    ) -> Self {
        Self::new(
            "example".to_string(),
            Box::new(|ui| {
                let id = egui::Id::new("test");
                let t = ui.data_mut(|x| {
                    let t = x.get_temp_mut_or_insert_with::<u8>(id, || 0);
                    *t = t.saturating_add(1);
                    let t = *t;
                    if t == 255 {
                        x.remove::<u8>(id);
                    }
                    t
                });
                ui.label("bla");
                ui.label(&format!("t: {t}"));
                t == 255
            }),
            DialogKind::Progress {
                cancelable: cancelable.then_some(Button {
                    label: "Cancel".to_string(),
                    action: Box::new(|| true),
                }),
                progress: Progress::new(expectation),
            },
        )
    }
    pub(super) fn example_button(n: usize) -> Self {
        Self::new(
            "example".to_string(),
            Box::new(|ui| {
                ui.label("bla");
                false
            }),
            DialogKind::Button {
                buttons: (0..n)
                    .map(|i| Button {
                        label: format!("Ok: {i}"),
                        action: Box::new(|| true),
                    })
                    .collect(),
                has_exit: Some(0),
            },
        )
    }
}

#[derive(Default)]
pub(super) struct DialogWidget {
    dialogs: std::collections::VecDeque<Dialog>,
    current_dialog: Option<Dialog>,
}

impl DialogWidget {
    pub(super) fn push(&mut self, dialog: Dialog) {
        self.dialogs.push_back(dialog)
    }
    pub(super) fn progress(&mut self, ctx: &egui::Context) -> bool {
        if self.current_dialog.is_none() && !self.dialogs.is_empty() {
            self.current_dialog = self.dialogs.pop_front();
        }
        let dialogs_are_done = if let Some(Dialog {
            title,
            content,
            kind,
            has_exit,
            requested_focus,
        }) = self.current_dialog.as_mut()
        {
            let dialog = egui::Window::new(title.as_str())
                .collapsible(false)
                .auto_sized()
                .resizable(false)
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.input(|x| x.screen_rect()).center());
            let dialog = if let Some(is_open) = has_exit {
                dialog.open(is_open)
            } else {
                dialog
            };
            let dialog = dialog.show(ctx, |ui| content(ui) || kind.show(ui, requested_focus));
            if let Some(dialog) = dialog {
                if let Some(dialog_is_completed) = dialog.inner {
                    dialog_is_completed
                } else {
                    unreachable!("'dialog.collapbsile' was set to false")
                }
            } else {
                // this happens if 'dialog.open' is called and clicked
                kind.close()
            }
        } else {
            true
        };
        if dialogs_are_done {
            self.current_dialog = None;
        }
        dialogs_are_done
    }
}
