mod file;
mod heatmap;
mod limit;
mod violin;

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq)]
pub enum HiddenOrShown {
    Hidden,
    Shown,
}

pub struct ShowHideEvent<Key> {
    pub(crate) hidden_or_shown: HiddenOrShown,
    pub(crate) single_or_all: Option<Key>,
}

pub enum DataEvent<FileKey, LimitKey, File, Limit> {
    File(FileEvent<FileKey, File>),
    Limit(LimitEvent<LimitKey, Limit>),
    Heatmap(HeatmapEvent<FileKey>),
    Violin(ViolinEvent<FileKey, LimitKey, Limit>),
}
pub use file::FileEvent;
pub use heatmap::HeatmapEvent;
pub use limit::LimitEvent;
pub use violin::ViolinEvent;
