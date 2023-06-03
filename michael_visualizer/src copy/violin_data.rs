pub struct ExampleData;
impl ExampleData {
    pub fn zero_p_five(n: usize) -> Vec<f32> {
        vec![0.5; n]
    }
    pub fn linear(lower: f32, upper: f32, n: usize) -> Vec<f32> {
        let delta = (upper - lower) / ((n - 1) as f32);
        (0..n).map(|x| (x as f32) * delta + lower).collect()
    }
    pub fn gauss(mean: f32, std: f32, lower: f32, upper: f32, n: usize) -> Vec<f32> {
        Self::linear(lower, upper, n)
            .into_iter()
            .map(|x| (-0.5 * (x - mean).powi(2) / std.powi(2)).exp())
            .map(|x| x / std / (2. * std::f32::consts::PI).sqrt())
            //.map(|x| x * std * (2. * std::f32::consts::PI).sqrt())
            //.map(|x| x.ln())
            //.map(|x| (x*std.powi(2)m))
            //.map(|x| x*-2.)
            //.map(|x| x.atan())
            .collect()
    }
    pub fn atan_distribution(lower: f32, upper: f32, n: usize) -> Vec<f32> {
        Self::linear(lower.tan(), upper.tan(), n)
            .into_iter()
            .map(|y| y.atan())
            //.map(|y: f32| y * 2. / std::f32::consts::PI + 0.5)
            .collect()
    }
}

#[test]
fn atan_test() {
    dbg!(ExampleData::atan_distribution(-1., 1., 11));
}

#[derive(Debug)]
pub struct ViolinData {
    // None indicates: No value is below the minimum
    pub fraction_below: Option<f32>,
    // None indicates: No value is above the maximum
    pub fraction_above: Option<f32>,
    // None indicates: No value is non-finite
    pub fraction_non_finite: Option<f32>,
    // Data in the bins
    pub bins: Box<[Option<f32>]>,
    pub max_bin: f32,
    //pub adjustment: f32,
    pub mean: f32,
}

impl ViolinData {
    pub fn construct(data: &[f32], lower_limit: f32, upper_limit: f32, resolution: usize) -> Self {
        assert_ne!(0, resolution, "At least one bin is needed");
        let mut below_count = 0u32;
        let mut above_count = 0u32;
        let mut non_finite_count = 0u32;
        let delta = upper_limit - lower_limit;
        let mut bins = vec![0u32; resolution];
        let resolution_float = resolution as f32;
        let factor = resolution_float / delta;
        assert!(delta.is_finite() && delta > 0.);
        let mut mean = 0.;
        for &d in data {
            if !d.is_finite() {
                non_finite_count += 1;
            } else if d >= upper_limit {
                above_count += 1;
            } else if d < lower_limit {
                below_count += 1;
            } else {
                let ratio = (d - lower_limit) * factor; // between 0. and (resolution)
                let ratio = ratio.clamp(0., resolution_float); // numerical precision - might be unnecessary???
                let bin = (ratio as usize).clamp(0, resolution - 1);
                bins[bin] += 1;
                mean += d;
            }
        }
        let max_bin = bins.iter().max().cloned().unwrap_or(0);
        let mean = mean / (data.len() as f32);
        let mean = (mean - lower_limit) / delta;
        Self {
            fraction_below: below_count.div_opt(data),
            fraction_above: above_count.div_opt(data),
            fraction_non_finite: non_finite_count.div_opt(data),
            //adjustment: *bins.iter().max().unwrap() as f32,
            bins: bins
                .into_iter()
                .map(|x| if x > 0 { Some(x.div(data)) } else { None })
                .collect(),
            max_bin: max_bin.div(data),
            mean,
        }
    }
    pub fn get_boundaries(&self) -> Vec<Vec<(usize, f32)>> {
        let mut parts = Vec::new();
        let mut ongoing = None;
        for (index, ratio) in self.bins.iter().enumerate() {
            match (ongoing.take(), ratio) {
                (None, None) => { /* nothing to do */ }
                (None, Some(ratio)) => ongoing = Some(vec![(index, *ratio)]),
                (Some(ongoing), None) => parts.push(ongoing),
                (Some(mut current), Some(ratio)) => {
                    current.push((index, *ratio));
                    ongoing = Some(current);
                }
            }
        }
        if let Some(ongoing) = ongoing {
            parts.push(ongoing);
        }
        parts
    }
    pub fn to_shapes(
        &self,
        color: egui::Color32,
        transform: egui::emath::RectTransform,
        i: usize,
        n: usize,
        max_bin: Option<f32>,
    ) -> Vec<egui::Shape> {
        /*let missing = fraction_above.unwrap_or(0.)
            + fraction_below.unwrap_or(0.)
            + fraction_non_finite.unwrap_or(0.);
        let missing_adjustment = 1. / (1. - missing);*/
        let mut parts: Vec<egui::Shape> = Vec::new();
        let bin_count_twice = (2 * self.bins.len()) as f32;
        let center = (2 * i + 1) as f32 / (2 * n) as f32;
        let height = 1. / bin_count_twice;
        let max_bin = max_bin.unwrap_or(self.max_bin);
        for segments in self.get_boundaries() {
            if segments.is_empty() {
                continue;
            }
            let mut points_right = Vec::new();
            let mut points_left = Vec::new();
            for (index, ratio) in segments {
                let y = (2 * index + 1) as f32 / bin_count_twice;
                let width = ratio / max_bin / (n as f32) / 2. * 0.95;
                points_left.push(transform * egui::pos2(center - width, y - height));
                points_left.push(transform * egui::pos2(center - width, y + height));
                points_right.push(transform * egui::pos2(center + width, y - height));
                points_right.push(transform * egui::pos2(center + width, y + height));
            }
            points_left.extend(points_right.into_iter().rev());
            parts.push(egui::Shape::closed_line(
                points_left,
                egui::Stroke::new(1.0, color),
            ));
        }
        let mean = self.mean;
        if mean.is_finite() && mean > 0. && mean < 1. {
            parts.push(egui::Shape::circle_filled(
                transform * egui::pos2(center, mean),
                5.,
                color,
            ))
        }
        parts
    }
}
trait DivF32 {
    fn div(self, data: &[f32]) -> f32;
    fn div_opt(self, data: &[f32]) -> Option<f32>;
}
impl DivF32 for u32 {
    fn div(self, data: &[f32]) -> f32 {
        (self as f32) / (data.len() as f32)
    }

    fn div_opt(self, data: &[f32]) -> Option<f32> {
        if self == 0 {
            None
        } else {
            Some(self.div(data))
        }
    }
}
#[test]
fn example_data_0p5() {
    dbg!(ViolinData::construct(
        &ExampleData::zero_p_five(100),
        0.,
        1.,
        11
    ));
}
#[test]
fn example_data_linear() {
    dbg!(ViolinData::construct(
        &ExampleData::linear(0., 3., 10000),
        1.,
        2.,
        11
    ));
}
