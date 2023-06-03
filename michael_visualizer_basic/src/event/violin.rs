pub enum ViolinEvent<FileKey, LimitKey, LimitData> {
    ShowHide(super::ShowHideEvent<FileKey>),
    Value(LimitKey, LimitData),
    Label(LimitKey, crate::data_types::LimitLabel),
}
