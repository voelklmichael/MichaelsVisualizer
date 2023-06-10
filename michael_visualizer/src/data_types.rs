pub mod finite_f32;

pub use std::hash::Hash;
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct FileLabel(String);
impl FileLabel {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn get_mut(&mut self) -> &mut String {
        &mut self.0
    }
}
impl From<String> for FileLabel {
    fn from(value: String) -> Self {
        Self(value)
    }
}
#[derive(PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct LimitLabel(String);
impl From<String> for LimitLabel {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl LimitLabel {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn get_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
pub struct LimitKey(u64);
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
pub struct FileKey(u64);

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct FileKeyGenerator(FileKey);
impl Default for FileKeyGenerator {
    fn default() -> Self {
        Self(FileKey(0))
    }
}
impl FileKeyGenerator {
    pub(crate) fn next(&mut self) -> FileKey {
        let t = self.0.clone();
        self.0 = FileKey(t.0 + 1);
        t
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct LimitKeyGenerator(LimitKey);
impl Default for LimitKeyGenerator {
    fn default() -> Self {
        Self(LimitKey(0))
    }
}
impl LimitKeyGenerator {
    pub(crate) fn next(&mut self) -> LimitKey {
        let t: LimitKey = self.0.clone();
        self.0 = LimitKey(t.0 + 1);
        t
    }
}
