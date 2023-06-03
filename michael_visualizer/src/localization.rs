pub struct LocalizableString {
    pub english: String,
}
pub struct LocalizableStr<'a> {
    pub english: &'a str,
}
#[derive(Clone, Copy, serde::Deserialize, serde::Serialize, Default)]
pub enum Language {
    #[default]
    English,
}
impl<'a> LocalizableStr<'a> {
    pub fn localize(&self, language: Language) -> &'a str {
        match language {
            Language::English => self.english,
        }
    }
}
impl LocalizableString {
    pub fn localize(self, language: Language) -> String {
        match language {
            Language::English => self.english,
        }
    }
}
