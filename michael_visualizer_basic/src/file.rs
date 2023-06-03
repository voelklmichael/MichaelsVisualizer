use std::hash::Hash;

use crate::LimitTrait;

use super::event::HiddenOrShown;

use super::data_types::FileLabel;

pub trait FileTrait {
    type Limit;
    fn limits(&self) -> &[Self::Limit];
    fn row_count(&self) -> usize;
    fn apply_limit(&mut self, limit_index: usize, limit: &Self::Limit) -> Vec<bool>;
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FileWrapper<File, LimitKey: Eq + Hash> {
    label: FileLabel,
    hidden_or_shown: HiddenOrShown,
    content: File,
    limit_indices: std::collections::HashMap<LimitKey, usize>,
    filters_summed: Vec<u32>,
}

impl<File, LimitKey: Eq + Hash> FileWrapper<File, LimitKey> {
    pub(crate) fn get_label(&self) -> &FileLabel {
        &self.label
    }
    pub(crate) fn get_mut_label(&mut self) -> &mut FileLabel {
        &mut self.label
    }
    pub(crate) fn new(
        label: FileLabel,
        content: File,
        limit_keys: &crate::data_types::OrderedMap<LimitKey, File::Limit>,
    ) -> Self
    where
        LimitKey: Clone + std::hash::Hash + Eq,
        File: FileTrait,
        <File as FileTrait>::Limit: crate::LimitTrait,
    {
        let mut limit_indices = std::collections::HashMap::new();
        let mut used_labels = Vec::new();
        for (index, limit) in content.limits().iter().enumerate() {
            if let Some((key, limit)) = limit_keys
                .iter()
                .filter(|(_, l)| l.has_same_label(limit))
                .next()
            {
                if used_labels.contains(limit.label()) {
                    panic!("Limit label is used multiple times: {:?}", limit.label());
                }
                used_labels.push(limit.label().clone());
                let temp = limit_indices.insert(key.clone(), index);
                assert!(temp.is_none());
            } else {
                unreachable!("Before this is called, its limits are already added");
            }
        }
        Self {
            label,
            hidden_or_shown: HiddenOrShown::Shown,
            filters_summed: vec![0; content.row_count()],
            content,
            limit_indices,
        }
    }

    pub(crate) fn is_shown(&self) -> bool {
        self.hidden_or_shown == HiddenOrShown::Shown
    }
    pub(crate) fn change_shown(&mut self, hidden_or_shown: HiddenOrShown) {
        self.hidden_or_shown = hidden_or_shown;
    }

    pub(crate) fn apply_limit(&mut self, key: &LimitKey, limit: &File::Limit) -> Option<Vec<bool>>
    where
        File: FileTrait,
        LimitKey: std::hash::Hash + Eq,
    {
        if let Some(index) = self.limit_indices.get(key) {
            Some(self.content.apply_limit(*index, limit))
        } else {
            None
        }
    }

    pub(crate) fn limits(&self) -> &[File::Limit]
    where
        File: FileTrait,
    {
        self.content.limits()
    }

    pub(crate) fn filters_new(&mut self, filters: &[bool]) {
        self.filters_summed
            .iter_mut()
            .zip(filters.iter())
            .filter(|(_, f)| **f)
            .for_each(|(x, _)| *x += 1);
    }
    pub(crate) fn filters_adjusted(&mut self, new_filters: &[bool], old_filters: &[bool]) {
        self.filters_summed
            .iter_mut()
            .zip(new_filters.iter())
            .zip(old_filters.iter())
            .for_each(|((x, new), old)| {
                *x = match (new, old) {
                    (true, true) => *x,
                    (true, false) => *x + 1,
                    (false, true) => *x - 1,
                    (false, false) => *x,
                }
            })
    }
    pub(crate) fn get_to_befiltered(&self) -> impl Iterator<Item = bool> + '_ {
        self.filters_summed.iter().map(|x| x > &0)
    }

    pub(crate) fn change_label(&mut self, label: FileLabel) {
        self.label = label;
    }
}
