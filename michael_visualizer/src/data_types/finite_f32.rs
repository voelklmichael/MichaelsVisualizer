use std::ops::Deref;

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Debug)]
pub(crate) struct FiniteF32(f32);
impl FiniteF32 {
    pub(crate) fn new_checked(f: f32) -> Option<Self> {
        f.is_finite().then_some(Self::new(f))
    }
    pub(crate) fn new(f: f32) -> Self {
        debug_assert!(f.is_finite());
        Self(f)
    }

    pub(crate) fn inner(&self) -> f32 {
        self.0
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
        let l = self.0;
        let r = other.0;
        Some(if r < l {
            std::cmp::Ordering::Less
        } else if r == l {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        })
    }
}
impl Ord for FiniteF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let l = self.0;
        let r = other.0;
        if l < r {
            std::cmp::Ordering::Less
        } else if r == l {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    }
}
