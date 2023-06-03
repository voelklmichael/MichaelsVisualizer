use crate::data_types::FileLabel;

pub enum FileEvent<Key, File> {
    Loaded(Key, FileLabel, File),
    ToLoad1(FileLabel, File),
    ToLoad2 { path: std::path::PathBuf },
    Removed(Key),
    Title(Key, FileLabel),
    // Order of files has changed
    // Data is the new order
    OrderSwitched(Key, Key),
    ShowHide(super::ShowHideEvent<Key>),
}
