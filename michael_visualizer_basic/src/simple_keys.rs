#[derive(serde::Deserialize, serde::Serialize, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SimpleFileKey(u64);

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SimpleLimitKey(pub u64); //TODO: remove this pub

impl super::key_generator::Incrementable for SimpleFileKey {
    fn increment(&mut self) -> Self {
        Self(self.0 + 1)
    }
}

impl super::key_generator::Incrementable for SimpleLimitKey {
    fn increment(&mut self) -> Self {
        Self(self.0 + 1)
    }
}
