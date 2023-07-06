use egui_dock::Tree;

use super::{AppState, TabId};

#[derive(serde::Deserialize, serde::Serialize)]
pub(super) enum Tab {
    Dummy(super::dummy::DummyTab),
    Limit(super::limits::LimitTab),
    Files(super::files::FileTab),
    Heatmap(Box<super::heatmap::HeatmapTab>),
    Violinplot(super::violinplot::ViolinTab),
    Selection(super::selection::SelectionTab),
    Plot(super::plot::PlotTab),
    Distribution(super::distribution::DistributionTab),
}
impl super::DataEventNotifyable for Tab {
    fn notify(&mut self, event: &super::DataEvent) -> Vec<super::DataEvent> {
        match self {
            Tab::Dummy(_) => Default::default(),
            Tab::Files(_) => Default::default(),
            Tab::Limit(_) => Default::default(),
            Tab::Violinplot(d) => d.notify(event),
            Tab::Selection(d) => d.notify(event),
            Tab::Heatmap(d) => d.notify(event),
            Tab::Plot(d) => d.notify(event),
            Tab::Distribution(d) => d.notify(event),
        }
    }

    fn progress(&mut self, state: &mut AppState) {
        match self {
            Tab::Dummy(_) => {}
            Tab::Files(_) => {}
            Tab::Limit(_) => {}
            Tab::Violinplot(d) => d.progress(state),
            Tab::Selection(d) => d.progress(state),
            Tab::Heatmap(d) => d.progress(state),
            Tab::Plot(d) => d.progress(state),
            Tab::Distribution(d) => d.progress(state),
        }
    }
}

impl Tab {
    pub(super) fn kind(&self) -> TabKind {
        match self {
            Tab::Dummy(_) => TabKind::Dummy,
            Tab::Limit(_) => TabKind::Limit,
            Tab::Files(_) => TabKind::Files,
            Tab::Heatmap(_) => TabKind::Heatmap,
            Tab::Violinplot(_) => TabKind::Violinplot,
            Tab::Selection(_) => TabKind::Selection,
            Tab::Plot(_) => TabKind::Plot,
            Tab::Distribution(_) => TabKind::Distribution,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum TabKind {
    Dummy,
    Limit,
    Files,
    Heatmap,
    Violinplot,
    Selection,
    Plot,
    Distribution,
}

impl TabKind {
    pub(super) fn kinds() -> Vec<TabKind> {
        vec![
            TabKind::Dummy,
            TabKind::Limit,
            TabKind::Files,
            TabKind::Heatmap,
            TabKind::Violinplot,
            TabKind::Selection,
            TabKind::Plot,
            TabKind::Distribution,
        ]
    }
    pub(super) fn to_tab(self) -> Tab {
        match self {
            TabKind::Dummy => Tab::Dummy(Default::default()),
            TabKind::Limit => Tab::Limit(Default::default()),
            TabKind::Files => Tab::Files(Default::default()),
            TabKind::Heatmap => Tab::Heatmap(Default::default()),
            TabKind::Violinplot => Tab::Violinplot(Default::default()),
            TabKind::Selection => Tab::Selection(Default::default()),
            TabKind::Plot => Tab::Plot(Default::default()),
            TabKind::Distribution => Tab::Distribution(Default::default()),
        }
    }
}

pub(super) trait TabTrait {
    fn title(&self, state: &AppState) -> &str;
    fn show(&mut self, state: &mut AppState, ui: &mut egui::Ui);
}

impl Tab {
    pub(super) fn title(&self, viewer: &AppState) -> &str {
        match self {
            Tab::Dummy(d) => d.title(viewer),
            Tab::Limit(d) => d.title(viewer),
            Tab::Files(d) => d.title(viewer),
            Tab::Heatmap(d) => d.title(viewer),
            Tab::Violinplot(d) => d.title(viewer),
            Tab::Selection(d) => d.title(viewer),
            Tab::Plot(d) => d.title(viewer),
            Tab::Distribution(d) => d.title(viewer),
        }
    }
    pub(super) fn show(&mut self, viewer: &mut AppState, ui: &mut egui::Ui) {
        match self {
            Tab::Dummy(d) => d.show(viewer, ui),
            Tab::Limit(d) => d.show(viewer, ui),
            Tab::Files(d) => d.show(viewer, ui),
            Tab::Heatmap(d) => d.show(viewer, ui),
            Tab::Violinplot(d) => d.show(viewer, ui),
            Tab::Selection(d) => d.show(viewer, ui),
            Tab::Plot(d) => d.show(viewer, ui),
            Tab::Distribution(d) => d.show(viewer, ui),
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
pub(super) struct Tabs {
    pub(super) tabs: Tree<(TabId, Tab)>,
}
impl Tabs {
    pub(crate) fn push(&mut self, tab: TabKind) {
        for id in 0.. {
            let id = TabId::new(id);
            if self.tabs.tabs().all(|(i, _)| i != &id) {
                self.tabs.push_to_first_leaf((id, tab.to_tab()));
                break;
            } else {
                continue;
            }
        }
    }
}
impl super::DataEventNotifyable for Tabs {
    fn notify(&mut self, event: &super::DataEvent) -> Vec<super::DataEvent> {
        let mut events = Vec::new();
        self.tabs.iter_mut().for_each(|t| {
            if let egui_dock::Node::Leaf {
                rect: _,
                viewport: _,
                tabs,
                active: _,
                scroll: _,
            } = t
            {
                tabs.iter_mut()
                    .for_each(|(_, t)| events.extend(t.notify(event)))
            } else {
                Default::default()
            }
        });
        events
    }

    fn progress(&mut self, state: &mut AppState) {
        self.tabs.iter_mut().for_each(|t| {
            if let egui_dock::Node::Leaf {
                rect: _,
                viewport: _,
                tabs,
                active: _,
                scroll: _,
            } = t
            {
                tabs.iter_mut().for_each(|(_, t)| t.progress(state))
            } else {
            }
        });
    }
}
impl Default for Tabs {
    fn default() -> Self {
        let mut tabs = Self {
            tabs: Default::default(),
        };
        for t in TabKind::kinds() {
            tabs.push(t);
        }
        tabs
    }
}
