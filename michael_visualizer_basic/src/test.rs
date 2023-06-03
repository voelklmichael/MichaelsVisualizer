use michael_visualizer_basic::*;



struct SimpleFile {
    limits: Vec<SimpleLimit>,
    data: Vec<Vec<f32>>,
}
impl FileTrait for SimpleFile {
    type Limit = SimpleLimit;

    fn limits(&self) -> &[Self::Limit] {
        &self.limits
    }

    fn row_count(&self) -> usize {
        self.data[0].len()
    }

    fn apply_limit(&mut self, limit_index: usize, limit: &Self::Limit) -> Vec<bool> {
        let data = self
            .data
            .get_mut(limit_index)
            .expect("Wrong limit index given");
        let Self::Limit {
            label: _,
            lower,
            upper,
        } = limit;
        let check = |f| {
            if let Some(lower) = lower {
                if f < lower {
                    return true;
                }
            }
            if let Some(upper) = upper {
                if f > upper {
                    return true;
                }
            }
            false
        };
        data.iter().map(check).collect()
    }
}
#[derive(Clone)]
struct SimpleLimit {
    label: LimitLabel,
    lower: Option<f32>,
    upper: Option<f32>,
}
impl LimitTrait for SimpleLimit {
    fn has_same_label(&self, other: &Self) -> bool {
        &self.label == &other.label
    }

    fn change_label(&mut self, label: LimitLabel) -> bool {
        if self.label == label {
            false
        } else {
            self.label = label;
            true
        }
    }

    fn label(&self) -> &LimitLabel {
        &self.label
    }
}

#[test]
fn simple_test() {
    let mut center =
        DataCenter::<SimpleFileKey, SimpleLimitKey, SimpleFile, SimpleLimit>::default();
    let events = vec![
        Event::File(FileEvent::Loaded(
            center.new_file_key(),
            "FileA".to_string().into(),
            SimpleFile {
                limits: vec![
                    SimpleLimit {
                        label: "LimitNone".to_string().into(),
                        lower: None,
                        upper: None,
                    },
                    SimpleLimit {
                        label: "LimitLower".to_string().into(),
                        lower: Some(1.),
                        upper: None,
                    },
                    SimpleLimit {
                        label: "LimitUpper".to_string().into(),
                        lower: None,
                        upper: Some(3.),
                    },
                    SimpleLimit {
                        label: "LimitAll".to_string().into(),
                        lower: Some(2.),
                        upper: Some(4.),
                    },
                ],
                data: vec![
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                ],
            },
        )),
        Event::File(FileEvent::Loaded(
            center.new_file_key(),
            "FileB".to_string().into(),
            SimpleFile {
                limits: vec![
                    SimpleLimit {
                        label: "LimitNone".to_string().into(),
                        lower: None,
                        upper: None,
                    },
                    SimpleLimit {
                        label: "LimitLower".to_string().into(),
                        lower: Some(1.),
                        upper: None,
                    },
                    SimpleLimit {
                        label: "LimitUpper".to_string().into(),
                        lower: None,
                        upper: Some(3.),
                    },
                    SimpleLimit {
                        label: "LimitAll".to_string().into(),
                        lower: Some(2.),
                        upper: Some(4.),
                    },
                ],
                data: vec![
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                ],
            },
        )),
        Event::File(FileEvent::Loaded(
            center.new_file_key(),
            "FileC".to_string().into(),
            SimpleFile {
                limits: vec![
                    SimpleLimit {
                        label: "LimitNone2".to_string().into(),
                        lower: None,
                        upper: None,
                    },
                    SimpleLimit {
                        label: "LimitLower2".to_string().into(),
                        lower: Some(1.),
                        upper: None,
                    },
                    SimpleLimit {
                        label: "LimitUpper2".to_string().into(),
                        lower: None,
                        upper: Some(3.),
                    },
                    SimpleLimit {
                        label: "LimitAll2".to_string().into(),
                        lower: Some(2.),
                        upper: Some(4.),
                    },
                ],
                data: vec![
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                    vec![-1., 0., 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
                ],
            },
        )),
        Event::Limit(LimitEvent::ToPlot(SimpleLimitKey(1))),
    ];
    dbg!(center.progress(events.into_iter()));
}
