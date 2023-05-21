pub struct DataFormat {
    row_count: usize,
    column: Vec<i32>,
    row: Vec<i32>,
    header: Vec<String>,
    data: std::collections::HashMap<String, Vec<f32>>,
}
// Constructors
impl DataFormat {
    pub fn example_rectangle_simple(row_count: usize) -> Self {
        Self::example_rectangle(
            row_count,
            0,
            0,
            vec![("d1".to_string(), 0., 1.), ("d2".to_string(), 1., 10.)],
        )
        .unwrap()
    }
    pub fn example_rectangle(
        row_count: usize,
        top_left_column: i32,
        top_left_row: i32,
        data: Vec<(String, f32, f32)>,
    ) -> Result<Self, String> {
        let (column, row) = (0..(row_count as i32))
            .map(|x| (top_left_column + x, top_left_row + x))
            .unzip();
        let mut header = Vec::with_capacity(data.len());
        let mut data_ = std::collections::HashMap::with_capacity(data.len());
        for (label, min, max) in data {
            if header.contains(&label) {
                return Err(format!("Label '{label}' occurs multiple times"));
            }
            header.push(label.clone());
            let delta = max - min;
            let d = (0..row_count)
                .map(|i| min + (i as f32) * delta / (row_count - 1) as f32)
                .collect();
            data_.insert(label, d);
        }
        Ok(Self {
            row_count,
            column,
            row,
            header,
            data: data_,
        })
    }
}
// Getter
impl DataFormat {
    pub fn row_count(&self) -> usize {
        self.row_count
    }
    pub fn column(&self) -> &[i32] {
        &self.column
    }
    pub fn row(&self) -> &[i32] {
        &self.row
    }
    pub fn header(&self) -> &[String] {
        &self.header
    }
    pub fn data(&self, header: &str) -> Option<&[f32]> {
        self.data.get(header).map(|x| x.as_slice())
    }
}
