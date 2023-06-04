use crate::data_types::FileLabel;

pub enum FileEvent<Key, File> {
    Loaded {
        key: Key,
        label: FileLabel,
        file: File,
    },
    LoadFromContent {
        label: String,
        content: Vec<u8>,
    },
    LoadFromPath {
        path: std::path::PathBuf,
    },
    Removed(Key),
    Title(Key, FileLabel),
    // Order of files has changed
    // Data is the new order
    OrderSwitched(Key, Key),
    ShowHide(super::ShowHideEvent<Key>),
}
