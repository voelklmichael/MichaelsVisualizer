use crate::{data_types::FileKey, LocalizableString};

#[derive(Debug)]
pub(crate) enum FileParseError {
    DummyError(String),
}

struct LoadThread {
    key: FileKey,
    path: std::path::PathBuf,
    thread: std::thread::JoinHandle<std::io::Result<Vec<u8>>>,
}

struct Type {
    key: FileKey,
    thread: std::thread::JoinHandle<Result<super::files::FileData, FileParseError>>,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(super) struct FileLoader {
    #[serde(skip)]
    load_thread: Vec<LoadThread>,
    #[serde(skip)]
    parse_thread: Vec<Type>,
}
impl FileLoader {
    pub(super) fn load(&mut self, key: FileKey, path: std::path::PathBuf) {
        self.load_thread.push(LoadThread {
            key,
            path: path.clone(),
            thread: std::thread::spawn(move || std::fs::read(path)),
        })
    }
    pub(super) fn parse(&mut self, key: FileKey, bytes: Vec<u8>) {
        self.parse_thread.push(Type {
            key,
            thread: std::thread::spawn(move || super::files::FileData::parse(bytes)),
        })
    }
    #[must_use]
    pub(super) fn check_progress(&mut self) -> Vec<super::DataEvent> {
        let mut events = Vec::new();
        loop {
            if let Some(index) = self.load_thread.iter().position(|t| t.thread.is_finished()) {
                let LoadThread { key, path, thread } = self.load_thread.remove(index);
                let label = path
                    .as_path()
                    .file_name()
                    .unwrap_or(path.as_os_str())
                    .to_string_lossy()
                    .to_string();
                let event = match thread.join() {
                    Ok(Ok(bytes)) => super::files::FileEvent::ParseFromBytes { key, label, bytes },
                    Ok(Err(err)) => super::files::FileEvent::LoadError {
                        key,
                        msg: LocalizableString {
                            english: format!("Failed to load: {err:?}"),
                        },
                    },
                    Err(err) => super::files::FileEvent::LoadError {
                        key,
                        msg: LocalizableString {
                            english: format!("Failed to join thread: {err:?}"),
                        },
                    },
                };
                events.push(super::DataEvent::File(event));
            } else if let Some(index) = self
                .parse_thread
                .iter()
                .position(|t| t.thread.is_finished())
            {
                let Type { key, thread } = self.parse_thread.remove(index);
                let event = match thread.join() {
                    Ok(Ok(file)) => super::files::FileEvent::Loaded {
                        key,
                        file,
                        non_conforming_tooltip: None,
                    },
                    Ok(Err(err)) => super::files::FileEvent::LoadError {
                        key,
                        msg: LocalizableString {
                            english: format!("Failed to parse: {err:?}"),
                        },
                    },
                    Err(err) => super::files::FileEvent::LoadError {
                        key,
                        msg: LocalizableString {
                            english: format!("Failed to join thread: {err:?}"),
                        },
                    },
                };
                events.push(super::DataEvent::File(event));
            } else {
                break events;
            }
        }
    }
}
