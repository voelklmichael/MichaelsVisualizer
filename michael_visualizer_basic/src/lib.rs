mod data_types;
mod event;
mod file;

mod simple_keys;
pub use simple_keys::{SimpleFileKey, SimpleLimitKey};
use std::hash::Hash;

pub use data_types::FileLabel;
pub use data_types::LimitLabel;
use data_types::OrderedMap;
pub use event::*;
pub use file::FileTrait;

mod key_generator;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DataCenter<FileKey: Eq + Hash, LimitKey: Eq + Hash, File, Limit> {
    files: OrderedMap<FileKey, file::FileWrapper<File, LimitKey>>,
    limits: OrderedMap<LimitKey, Limit>,
    filters: std::collections::HashMap<(FileKey, LimitKey), Vec<bool>>,
    limit_to_plot: Option<LimitKey>,
    file_key_generator: key_generator::KeyGenerator<FileKey>,
    limit_key_generator: key_generator::KeyGenerator<LimitKey>,
}
impl<FileKey: Default + Eq + Hash, LimitKey: Default + Eq + Hash, File, Limit> Default
    for DataCenter<FileKey, LimitKey, File, Limit>
{
    fn default() -> Self {
        Self {
            files: Default::default(),
            limits: Default::default(),
            filters: Default::default(),
            limit_to_plot: Default::default(),
            file_key_generator: Default::default(),
            limit_key_generator: Default::default(),
        }
    }
}
#[derive(Debug, Default)]
pub struct RedrawSelection {
    pub files: bool,
    pub limits: bool,
    pub heatmap: bool,
    pub violin: bool,
}
impl std::ops::BitOrAssign for RedrawSelection {
    fn bitor_assign(&mut self, rhs: Self) {
        self.files |= rhs.files;
        self.limits |= rhs.limits;
        self.heatmap |= rhs.heatmap;
        self.violin |= rhs.violin;
    }
}
impl RedrawSelection {
    fn redraw() -> Self {
        Self {
            files: true,
            limits: true,
            heatmap: true,
            violin: true,
        }
    }
    fn limit() -> Self {
        Self {
            files: false,
            limits: true,
            heatmap: true,
            violin: true,
        }
    }
}

pub trait LimitTrait {
    fn has_same_label(&self, other: &Self) -> bool;
    fn change_label(&mut self, label: LimitLabel) -> bool;
    fn label(&self) -> &LimitLabel;
}

impl<FileKey, LimitKey, File, Limit> DataCenter<FileKey, LimitKey, File, Limit>
where
    FileKey: Hash + Eq + Clone + key_generator::Incrementable,
    LimitKey: Hash + Eq + Clone + key_generator::Incrementable,
    File: FileTrait<Limit = Limit>,
    Limit: LimitTrait + Clone,
{
    #[must_use]
    pub fn progress(
        &mut self,
        events: impl Iterator<Item = event::DataEvent<FileKey, LimitKey, File, Limit>>,
    ) -> RedrawSelection {
        use event::*;
        let mut action = RedrawSelection::default();
        for event in events {
            let a = match event {
                DataEvent::File(event) => match event {
                    FileEvent::Loaded { key, label, file } => {
                        self.add_limits(file.limits());
                        let mut file = file::FileWrapper::new(label, file, &self.limits);
                        for (limit_key, limit) in self.limits.iter() {
                            file.apply_limit(limit_key, limit);
                        }
                        let x = self.files.insert(key, file);
                        assert!(x.is_none());
                        Some(RedrawSelection::redraw())
                    }
                    FileEvent::Removed(key) => {
                        if self.files.remove(&key).is_some() {
                            self.remove_unnecessary_limits();
                            Some(RedrawSelection::redraw())
                        } else {
                            None
                        }
                    }
                    FileEvent::Title(key, label) => {
                        if let Some(file) = self.files.get_mut(&key) {
                            file.change_label(label);
                            if file.is_shown() {
                                Some(RedrawSelection {
                                    files: true,
                                    limits: false,
                                    heatmap: true,
                                    violin: true,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    FileEvent::ShowHide(show_hide_event) => self.show_hide_event(show_hide_event),
                    FileEvent::OrderSwitched(k1, k2) => {
                        self.files.swap(&k1, &k2);
                        let v1 = self.files.get(&k1).map(|f| f.is_shown()).unwrap_or(false);
                        let v2 = self.files.get(&k2).map(|f| f.is_shown()).unwrap_or(false);
                        if v1 || v2 {
                            Some(RedrawSelection {
                                files: true,
                                limits: false,
                                heatmap: true,
                                violin: true,
                            })
                        } else {
                            None
                        }
                    }
                    FileEvent::LoadFromContent { label, content } => todo!(),
                    FileEvent::LoadFromPath { path } => todo!(),
                },
                DataEvent::Limit(event) => match event {
                    LimitEvent::Value(limit_key, new) => self.limit_value(limit_key, new),
                    LimitEvent::Label(key, label) => self.limit_label(key, label),
                    LimitEvent::ToPlot(key) => {
                        if self.limit_to_plot.as_ref() != Some(&key) {
                            self.limit_to_plot = Some(key);
                            Some(RedrawSelection::limit())
                        }else{None}
                    }
                    //event::LimitEvent::FormulaAdded(_) => todo!(),
                    //event::LimitEvent::FormulaRemoved(_) => todo!(),
                },
                DataEvent::Heatmap(event) => match event {
                   HeatmapEvent::ShowHide(show_hide_event) => {
                        self.show_hide_event(show_hide_event)
                    }
                    //event::HeatmapEvent::Selection => todo!(),
                    //event::HeatmapEvent::Area => todo!(),
                },
                DataEvent::Violin(event) => match event {
                    ViolinEvent::ShowHide(show_hide_event) => self.show_hide_event(show_hide_event),
                    ViolinEvent::Value(key, limit) => self.limit_value(key, limit),
                    ViolinEvent::Label(key, label) => self.limit_label(key, label),
                },
            };
            if let Some(a) = a {
                action |= a;
            }
        }
        action
    }

    #[must_use]
    fn limit_value(&mut self, limit_key: LimitKey, new: Limit) -> Option<RedrawSelection> {
        let Self {
            files,
            limits,
            filters,
            limit_to_plot: _,
            file_key_generator: _,
            limit_key_generator: _,
        } = self;
        if let Some(previous) = limits.get_mut(&limit_key) {
            let mut filters_changed = false;
            for (file_key, file) in files.iter_mut() {
                if let Some(new_filters) = file.apply_limit(&limit_key, &new) {
                    if let Some(old_filters) =
                        filters.get_mut(&(file_key.clone(), limit_key.clone()))
                    {
                        filters_changed |= new_filters
                            .iter()
                            .zip(old_filters.iter())
                            .any(|(a, b)| a != b);
                        file.filters_adjusted(&new_filters, old_filters);
                    } else {
                        filters_changed |= new_filters.iter().any(|x| *x);
                        file.filters_new(&new_filters);
                        filters.insert((file_key.clone(), limit_key.clone()), new_filters);
                    }
                }
            }
            *previous = new;
            filters_changed.then_some(RedrawSelection::limit())
        } else {
            None
        }
    }
    #[must_use]
    fn limit_label(&mut self, key: LimitKey, label: LimitLabel) -> Option<RedrawSelection> {
        if let Some(limit) = self.limits.get_mut(&key) {
            if limit.change_label(label) && self.limit_to_plot.as_ref() == Some(&key) {
                return Some(RedrawSelection::limit());
            }
        }
        None
    }
    #[must_use]
    fn show_hide_event(
        &mut self,
        show_hide_event: ShowHideEvent<FileKey>,
    ) -> Option<RedrawSelection> {
        let ShowHideEvent {
            hidden_or_shown,
            single_or_all,
        } = show_hide_event;
        if let Some(key) = single_or_all {
            if let Some(file) = self.files.get_mut(&key) {
                file.change_shown(hidden_or_shown);
            }
        } else {
            for (_, file) in self.files.iter_mut() {
                file.change_shown(hidden_or_shown);
            }
        }
        Some(RedrawSelection {
            files: true,
            limits: false,
            heatmap: true,
            violin: true,
        })
    }
    fn remove_unnecessary_limits(&mut self) {
        let mut to_remove = Vec::new();
        'outer: for (key, limit) in self.limits.iter() {
            for file_limits in self.files.iter().flat_map(|(_, f)| f.limits()) {
                if file_limits.has_same_label(limit) {
                    continue 'outer;
                }
            }
            to_remove.push(key.clone());
        }
        for key in to_remove {
            self.limits.remove(&key);
        }
    }
    fn add_limits(&mut self, limits: &[Limit]) {
        for limit in limits {
            let has_limit = self.limits.iter().any(|(_, l)| l.has_same_label(limit));
            if !has_limit {
                let key = self.new_limit_key();
                // for each existing file, find data filtered by this limit
                for (file_key, file) in self.files.iter_mut() {
                    if let Some(filters) = file.apply_limit(&key, limit) {
                        file.filters_new(&filters);
                        let x = self
                            .filters
                            .insert((file_key.clone(), key.clone()), filters);
                        assert!(x.is_none());
                    }
                }
                let x = self.limits.insert(key, limit.clone());
                assert!(x.is_none());
            }
        }
    }
    #[must_use]
    fn new_limit_key(&mut self) -> LimitKey {
        self.limit_key_generator.generate()
    }
    #[must_use]
    pub fn new_file_key(&mut self) -> FileKey {
        self.file_key_generator.generate()
    }
}
