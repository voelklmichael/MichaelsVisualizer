pub(super) struct ConfirmExit {
    close_is_allowed: bool,
    open_dialog: Option<std::sync::mpsc::Receiver<bool>>,
    title: String,
    question: String,
    label_ok: String,
    label_cancel: String,
}
impl Default for ConfirmExit {
    fn default() -> Self {
        Self::english()
    }
}
impl ConfirmExit {
    pub(super) fn new<A, B, C, D>(title: A, question: B, label_ok: C, label_cancel: D) -> Self
    where
        A: Into<String>,
        B: Into<String>,
        C: Into<String>,
        D: Into<String>,
    {
        Self {
            close_is_allowed: false,
            open_dialog: None,
            title: title.into(),
            question: question.into(),
            label_ok: label_ok.into(),
            label_cancel: label_cancel.into(),
        }
    }
    pub(super) fn english() -> Self {
        Self::new("Close?", "Shall the app be closed?", "Ok", "Cancel")
    }
    pub(super) fn close_event(&mut self) -> Option<super::dialog::Dialog> {
        use crate::dialog::*;
        if self.close_is_allowed || self.open_dialog.is_some() {
            None
        } else {
            let (sender1, receiver) = std::sync::mpsc::channel();
            let question = self.question.clone();
            let sender2 = sender1.clone();
            let dialog = Dialog::new(
                self.title.clone(),
                Box::new(move |ui| {
                    ui.label(&question);
                    false
                }),
                DialogKind::Button {
                    buttons: vec![
                        Button {
                            label: self.label_cancel.clone(),
                            action: Box::new(move || {
                                let _ = sender1.send(false);
                                true
                            }),
                        },
                        Button {
                            label: self.label_ok.clone(),
                            action: Box::new(move || {
                                let _ = sender2.send(true);
                                true
                            }),
                        },
                    ],
                    has_exit: Some(0),
                },
            );
            self.open_dialog = Some(receiver);
            Some(dialog)
        }
    }

    pub(super) fn shall_be_closed(&mut self) -> bool {
        use std::sync::mpsc::*;
        if let Some(dialog) = self.open_dialog.take() {
            match dialog.try_recv() {
                Ok(close_is_allowed) => {
                    self.close_is_allowed |= close_is_allowed;
                }
                Err(TryRecvError::Disconnected) => {}
                Err(TryRecvError::Empty) => {
                    self.open_dialog = Some(dialog);
                }
            }
        }
        self.close_is_allowed
    }
}
