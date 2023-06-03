pub trait Incrementable {
    fn increment(&mut self) -> Self;
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(crate) struct KeyGenerator<Key> {
    pub(crate) current: Key,
}

impl<Key> KeyGenerator<Key> {
    pub(crate) fn generate(&mut self) -> Key
    where
        Key: Incrementable,
    {
        let next = self.current.increment();
        std::mem::replace(&mut self.current, next)
    }
}
