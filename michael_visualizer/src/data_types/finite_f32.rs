use std::ops::Deref;

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Debug)]
pub(crate) struct FiniteF32(f32);
impl FiniteF32 {
    pub(crate) fn new_checked(f: f32) -> Option<Self> {
        f.is_finite().then(|| Self::new(f))
    }
    pub(crate) fn new(f: f32) -> Self {
        debug_assert!(f.is_finite());
        Self(f)
    }

    pub(crate) fn inner(&self) -> f32 {
        self.0
    }

    fn compare_internal(&self, other: &FiniteF32) -> std::cmp::Ordering {
        if self.0 < other.0 {
            std::cmp::Ordering::Less
        } else if self.0 == other.0 {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    }
}
impl Deref for FiniteF32 {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Eq for FiniteF32 {}
impl PartialOrd<FiniteF32> for FiniteF32 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.compare_internal(other))
    }
}
impl Ord for FiniteF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare_internal(other)
    }
}
impl TryFrom<i32> for FiniteF32 {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let f = value as f32;
        FiniteF32::new_checked(f).ok_or(())
    }
}
impl TryFrom<f32> for FiniteF32 {
    type Error = ();

    fn try_from(f: f32) -> Result<Self, Self::Error> {
        FiniteF32::new_checked(f).ok_or(())
    }
}
