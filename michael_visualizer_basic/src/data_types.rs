pub use std::hash::Hash;
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FileLabel(String);
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OrderedMap<Key: Eq + Hash, Value> {
    keys: Vec<Key>,
    data: std::collections::HashMap<Key, Value>,
}
impl<Key: Eq + Hash, Value> Default for OrderedMap<Key, Value> {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            data: Default::default(),
        }
    }
}
impl<Key: Eq + Hash, Value> OrderedMap<Key, Value> {
    pub fn insert(&mut self, key: Key, value: Value) -> Option<Value>
    where
        Key: Hash + Eq + Clone,
    {
        let temp = self.data.insert(key.clone(), value);
        if temp.is_none() {
            self.keys.push(key);
        }
        temp
    }
    pub fn remove(&mut self, key: &Key) -> Option<Value>
    where
        Key: Hash + Eq,
    {
        let temp = self.data.remove(key);
        if temp.is_some() {
            self.keys.remove(
                self.keys
                    .iter()
                    .position(|k| k == key)
                    .expect("Key to be removed not found"),
            );
        }
        temp
    }
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Value>
    where
        Key: Hash + Eq,
    {
        self.data.get_mut(key)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Key, &mut Value)> {
        self.data.iter_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.data.iter()
    }

    pub fn get(&self, key: &Key) -> Option<&Value>
    where
        Key: Hash + Eq,
    {
        self.data.get(key)
    }

    pub fn swap(&mut self, k1: &Key, k2: &Key)
    where
        Key: PartialEq,
    {
        let i1 = self.keys.iter().position(|k| k == k1);
        let i2 = self.keys.iter().position(|k| k == k2);
        if let (Some(i1), Some(i2)) = (i1, i2) {
            self.keys.swap(i1, i2);
        }
    }
}
