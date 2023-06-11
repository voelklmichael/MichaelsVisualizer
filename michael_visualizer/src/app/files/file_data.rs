use super::super::file_loader::FileParseError;
use super::super::limits::{Limit, LimitData};
use crate::data_types::finite_f32::FiniteF32;
use crate::{LocalizableStr, LocalizableString};

#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub(crate) enum DataKind {
    Float,
    Int,
}

#[derive(Clone)]
pub(crate) enum DataColumn {
    Float(Box<[f32]>),
    Int(Box<[i32]>),
}
impl DataColumn {
    fn kind(&self) -> DataKind {
        match self {
            DataColumn::Float(_) => DataKind::Float,
            DataColumn::Int(_) => DataKind::Int,
        }
    }
    pub(crate) fn len(&self) -> usize {
        match self {
            DataColumn::Float(d) => d.len(),
            DataColumn::Int(d) => d.len(),
        }
    }

    pub(crate) fn filter(
        &self,
        filtering: &[u32],
        min: FiniteF32,
        max: FiniteF32,
    ) -> Vec<FiniteF32> {
        match self {
            DataColumn::Float(data) => filtering
                .iter()
                .zip(data.iter())
                .flat_map(|(&n, f)| {
                    (n == 0 && f.is_finite() && f >= &min && f <= &max).then(|| FiniteF32::new(*f))
                })
                .collect(),
            DataColumn::Int(data) => filtering
                .iter()
                .zip(data.iter())
                .filter_map(|(&n, &i)| FiniteF32::new_checked(i as f32).map(|f| (n, f)))
                .flat_map(|(n, f)| {
                    (n == 0 && f.is_finite() && f >= min && f <= max).then(|| FiniteF32::new(*f))
                })
                .collect(),
        }
    }

    fn apply_limit(&self, limit: &Limit) -> Vec<bool> {
        if limit.data_kind() == DataKind::Int && self.kind() == DataKind::Float {
            unreachable!("This case should never happen")
        }
        match self {
            DataColumn::Float(d) => d.iter().map(|x| limit.is_outside(*x)).collect(),
            DataColumn::Int(d) => d.iter().map(|x| limit.is_outside(*x as f32)).collect(),
        }
    }

    fn get_as_string(&self, i: usize) -> String {
        match self {
            DataColumn::Float(d) => d[i].to_string(),
            DataColumn::Int(d) => d[i].to_string(),
        }
    }
}
impl From<Vec<f32>> for DataColumn {
    fn from(data: Vec<f32>) -> Self {
        if data.iter().any(|f| f.round() != *f) {
            Self::Float(data.into_boxed_slice())
        } else {
            Self::Int(data.into_iter().map(|f| f as i32).collect())
        }
    }
}

#[derive(Clone)]
pub(crate) struct FileData {
    header: LocalizableString,
    content: Vec<(LimitData, DataColumn)>,
}

impl FileData {
    pub(super) fn tooltip(&self) -> LocalizableStr {
        self.header.as_str()
    }

    pub(crate) fn limits(&self) -> impl Iterator<Item = Limit> + '_ {
        self.content.iter().map(|(d, _)| Limit::new(d.clone()))
    }

    pub(crate) fn apply_limit(&self, limit: &Limit, column: usize) -> super::super::Filtering {
        let (_, column) = self
            .content
            .get(column)
            .unwrap_or_else(|| panic!("File has no data in column {column}"));
        column.apply_limit(limit)
    }

    pub(crate) fn parse(bytes: Vec<u8>) -> Result<Self, FileParseError> {
        let s = std::string::String::from_utf8(bytes)
            .map_err(|e| FileParseError::DummyError(format!("{e:?}")))?;
        let mut limits = Vec::new();
        let mut rows = Vec::new();
        let mut lines = s.lines();
        let header = lines.next().unwrap_or_default();
        for (row_index, line) in lines.enumerate() {
            let c = line.split(';').map(|x| x.trim());
            if row_index < 4 {
                let h = c.collect::<Vec<_>>();
                if let Some(cc) = limits.first().map(|x: &Vec<&str>| x.len()) {
                    if h.len() != cc {
                        return Err(FileParseError::DummyError(format!(
                            "Not enough columns in row #{row_index}"
                        )));
                    }
                }
                limits.push(h);
            } else {
                let mut row = Vec::new();
                for (column_index, c) in c.enumerate() {
                    match c.parse::<f32>() {
                        Ok(f) => row.push(f),
                        Err(e) => {
                            return Err(FileParseError::DummyError(format!(
                                "Failed to parse row #{row_index}, column #{column_index} due to error: {e}"
                            )))
                        }
                    }
                }
                if limits.iter().map(|x| x.len()).next().unwrap() != row.len() {
                    return Err(FileParseError::DummyError(format!(
                        "Not enough columns in row #{row_index}"
                    )));
                }
                rows.push(row);
            }
        }
        if rows.is_empty() {
            return Err(FileParseError::DummyError(
                "File contains no data, at most test header description".into(),
            ));
        }
        let mut columns = Vec::new();
        while !limits.first().unwrap().is_empty() {
            let label = limits.get_mut(0).unwrap().pop().unwrap();
            let lower = limits.get_mut(1).unwrap().pop().unwrap();
            let upper = limits.get_mut(2).unwrap().pop().unwrap();
            let info = limits.get_mut(3).unwrap().pop().unwrap();
            fn parse(s: &str) -> Result<Option<f32>, FileParseError> {
                if s.is_empty() || s == "-" {
                    Ok(None)
                } else {
                    match s.parse::<f32>() {
                        Ok(f) => Ok(Some(f)),
                        Err(e) => Err(FileParseError::DummyError(format!("Failed to parse: {e}"))),
                    }
                }
            }
            let lower = parse(lower)?;
            let upper = parse(upper)?;
            let data: DataColumn = rows
                .iter_mut()
                .map(|r| r.pop().unwrap())
                .collect::<Vec<_>>()
                .into();
            let limit = LimitData {
                label: label.to_string().into(),
                lower: lower.and_then(FiniteF32::new_checked),
                upper: upper.and_then(FiniteF32::new_checked),
                info: LocalizableString {
                    english: info.to_string(),
                },
                data_kind: data.kind(),
            };
            columns.push((limit, data));
        }
        columns.reverse();
        Ok(FileData {
            header: LocalizableString {
                english: header.into(),
            },
            content: columns,
        })
    }
    pub fn to_csv(&self) -> Vec<String> {
        let Self { header, content } = self;
        let rows = content.first().unwrap().1.len();
        let mut csv = Vec::with_capacity(rows + 5);
        csv.push(header.as_str().english.to_string());
        csv.push(
            content
                .iter()
                .map(|(l, _)| l.label.as_str().to_string())
                .collect::<Vec<String>>()
                .join(";"),
        );
        csv.push(
            content
                .iter()
                .map(|(l, _)| l.lower.map(|x| x.to_string()).unwrap_or("-".to_string()))
                .collect::<Vec<String>>()
                .join(";"),
        );
        csv.push(
            content
                .iter()
                .map(|(l, _)| l.upper.map(|x| x.to_string()).unwrap_or("-".to_string()))
                .collect::<Vec<String>>()
                .join(";"),
        );
        csv.push(
            content
                .iter()
                .map(|(l, _)| l.info.as_str().english.to_string())
                .collect::<Vec<String>>()
                .join(";"),
        );
        for i in 0..rows {
            csv.push(
                content
                    .iter()
                    .map(|(_, d)| d.get_as_string(i))
                    .collect::<Vec<String>>()
                    .join(";"),
            );
        }
        csv
    }

    #[must_use]
    pub(crate) fn get_column(&self, column: usize) -> &DataColumn {
        &self.content[column].1
    }

    #[must_use]
    pub(crate) fn data_count(&self) -> usize {
        self.get_column(0).len()
    }
}

#[test]
fn generate_example_a() {
    let data = FileData {
        header: LocalizableString {
            english: "Example A".to_string(),
        },
        content: vec![
            (
                LimitData {
                    label: "X".to_string().into(),
                    lower: None,
                    upper: None,
                    info: LocalizableString {
                        english: "no boundaries".into(),
                    },
                    data_kind: DataKind::Int,
                },
                vec![0., 1., 2., 3., 4.].into(),
            ),
            (
                LimitData {
                    label: "Y".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: None,
                    info: LocalizableString {
                        english: "only lower boundary".into(),
                    },
                    data_kind: DataKind::Int,
                },
                vec![0., 1., 2., 3., 4.].into(),
            ),
            (
                LimitData {
                    label: "Test03".to_string().into(),
                    lower: None,
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "only upper boundary".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 1., 2., 3., 4.].into(),
            ),
            (
                LimitData {
                    label: "Test04".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "both boundaries".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 1., 2., 3., 4.].into(),
            ),
        ],
    };
    let csv = data.to_csv().join("\n");
    std::fs::write("example_a.mv01", csv).unwrap();
}
#[test]
fn parse_example_a() {
    let data = FileData {
        header: LocalizableString {
            english: "Example A".to_string(),
        },
        content: vec![
            (
                LimitData {
                    label: "X".to_string().into(),
                    lower: None,
                    upper: None,
                    info: LocalizableString {
                        english: "no boundaries".into(),
                    },
                    data_kind: DataKind::Int,
                },
                vec![0., 1., 2., 3., 4.].into(),
            ),
            (
                LimitData {
                    label: "Y".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: None,
                    info: LocalizableString {
                        english: "only lower boundary".into(),
                    },
                    data_kind: DataKind::Int,
                },
                vec![0., 1., 2., 3., 4.].into(),
            ),
            (
                LimitData {
                    label: "Test03".to_string().into(),
                    lower: None,
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "only upper boundary".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 1., 2., 3., 4.].into(),
            ),
            (
                LimitData {
                    label: "Test04".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "both boundaries".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 1., 2., 3., 4.].into(),
            ),
        ],
    };
    let csv = data.to_csv().join("\n");
    FileData::parse(csv.into_bytes()).unwrap();
}

#[test]
fn generate_example_b() {
    let data = FileData {
        header: LocalizableString {
            english: "Example B".to_string(),
        },
        content: vec![
            (
                LimitData {
                    label: "X".to_string().into(),
                    lower: None,
                    upper: None,
                    info: LocalizableString {
                        english: "no boundaries".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.].into(),
            ),
            (
                LimitData {
                    label: "Y".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: None,
                    info: LocalizableString {
                        english: "only lower boundary".into(),
                    },
                    data_kind: DataKind::Int,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.].into(),
            ),
            (
                LimitData {
                    label: "Test03".to_string().into(),
                    lower: None,
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "only upper boundary".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.].into(),
            ),
            (
                LimitData {
                    label: "Test04".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "both boundaries".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.].into(),
            ),
            (
                LimitData {
                    label: "Test05".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "both boundaries".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4., 4.5, 5., 1.8].into(),
            ),
        ],
    };
    let csv = data.to_csv().join("\n");
    std::fs::write("example_b.mv01", csv).unwrap();
}
#[test]
fn parse_example_b() {
    let data = FileData {
        header: LocalizableString {
            english: "Example B".to_string(),
        },
        content: vec![
            (
                LimitData {
                    label: "X".to_string().into(),
                    lower: None,
                    upper: None,
                    info: LocalizableString {
                        english: "no boundaries".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.].into(),
            ),
            (
                LimitData {
                    label: "Y".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: None,
                    info: LocalizableString {
                        english: "only lower boundary".into(),
                    },
                    data_kind: DataKind::Int,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.].into(),
            ),
            (
                LimitData {
                    label: "Test03".to_string().into(),
                    lower: None,
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "only upper boundary".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.].into(),
            ),
            (
                LimitData {
                    label: "Test04".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "both boundaries".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.].into(),
            ),
            (
                LimitData {
                    label: "Test05".to_string().into(),
                    lower: Some(FiniteF32::new(1.)),
                    upper: Some(FiniteF32::new(2.)),
                    info: LocalizableString {
                        english: "both boundaries".into(),
                    },
                    data_kind: DataKind::Float,
                },
                vec![0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4., 4.5, 5., 1.8].into(),
            ),
        ],
    };
    let csv = data.to_csv().join("\n");
    FileData::parse(csv.into_bytes()).unwrap();
}
