//! What the XY chart parser builds. Ported from Pele `src/diagrams/xychart/db.ts`.

use super::{LinearAxis, Orientation, Plot, PlotType, XAxis, XyChartModel};
use crate::util::js;

/// A value of a series with the label written after it (`""` when none).
pub(crate) struct DataPoint {
    pub value: f64,
    pub label: String,
}

pub(crate) struct XyChartDb {
    pub model: XyChartModel,
    has_set_x_axis: bool,
}

impl XyChartDb {
    pub fn new(title: Option<&str>) -> Self {
        XyChartDb {
            model: XyChartModel {
                title: title.map(str::to_string),
                acc_title: None,
                acc_descr: None,
                orientation: None,
                x_axis: XAxis::Band { title: String::new(), categories: Vec::new() },
                y_axis: LinearAxis { title: String::new(), min: f64::INFINITY, max: f64::NEG_INFINITY },
                has_set_y_axis: false,
                plots: Vec::new(),
            },
            has_set_x_axis: false,
        }
    }

    pub fn set_orientation(&mut self, orientation: &str) {
        self.model.orientation = Some(if orientation == "horizontal" { Orientation::Horizontal } else { Orientation::Vertical });
    }

    pub fn set_x_axis_title(&mut self, title: &str) {
        let title = js::trim(title).to_string();
        match &mut self.model.x_axis {
            XAxis::Band { title: t, .. } | XAxis::Linear { title: t, .. } => *t = title,
        }
    }

    fn x_title(&self) -> String {
        match &self.model.x_axis {
            XAxis::Band { title, .. } | XAxis::Linear { title, .. } => title.clone(),
        }
    }

    pub fn set_x_axis_range_data(&mut self, min: f64, max: f64) {
        self.model.x_axis = XAxis::Linear { title: self.x_title(), min, max };
        self.has_set_x_axis = true;
    }

    pub fn set_x_axis_band(&mut self, categories: Vec<String>) {
        let categories = categories.iter().map(|c| js::trim(c).to_string()).collect();
        self.model.x_axis = XAxis::Band { title: self.x_title(), categories };
        self.has_set_x_axis = true;
    }

    pub fn set_y_axis_title(&mut self, title: &str) {
        self.model.y_axis.title = js::trim(title).to_string();
    }

    pub fn set_y_axis_range_data(&mut self, min: f64, max: f64) {
        self.model.y_axis = LinearAxis { title: self.model.y_axis.title.clone(), min, max };
        self.model.has_set_y_axis = true;
    }

    fn plot_data(&mut self, values: Vec<f64>) -> Vec<(String, Option<f64>)> {
        let mut values = values;
        if values.is_empty() {
            return Vec::new();
        }
        // The first series fixes the x range when no x-axis statement did.
        if !self.has_set_x_axis {
            self.set_x_axis_range_data(1.0, values.len() as f64);
        }
        if let XAxis::Band { categories, .. } = &self.model.x_axis {
            if values.len() > categories.len() {
                values.truncate(categories.len());
            }
        }
        if !self.model.has_set_y_axis {
            let LinearAxis { mut min, mut max, .. } = self.model.y_axis;
            for &value in &values {
                if value < min {
                    min = value;
                }
                if value > max {
                    max = value;
                }
            }
            self.model.y_axis = LinearAxis { title: self.model.y_axis.title.clone(), min, max };
        }
        match &self.model.x_axis {
            XAxis::Band { categories, .. } => {
                categories.iter().enumerate().map(|(i, category)| (category.clone(), values.get(i).copied())).collect()
            }
            XAxis::Linear { min, max, .. } => {
                if values.len() == 1 {
                    return vec![(js::to_string(*min), Some(values[0]))];
                }
                let step = (max - min) / (values.len() - 1) as f64;
                values.iter().enumerate().map(|(i, &value)| (js::to_string(min + i as f64 * step), Some(value))).collect()
            }
        }
    }

    fn add_plot(&mut self, kind: PlotType, title: &str, data: Vec<DataPoint>) {
        let values = data.iter().map(|d| d.value).collect();
        let plot_data = self.plot_data(values);
        // Mermaid accepts labels on the points of a bar series and ignores them.
        let point_labels = if kind == PlotType::Line && data.iter().any(|d| !js::trim(&d.label).is_empty()) {
            Some(data.iter().map(|d| js::trim(&d.label).to_string()).collect())
        } else {
            None
        };
        self.model.plots.push(Plot { kind, title: js::trim(title).to_string(), data: plot_data, point_labels });
    }

    pub fn set_line_data(&mut self, title: &str, data: Vec<DataPoint>) {
        self.add_plot(PlotType::Line, title, data);
    }

    pub fn set_bar_data(&mut self, title: &str, data: Vec<DataPoint>) {
        self.add_plot(PlotType::Bar, title, data);
    }

    pub fn set_diagram_title(&mut self, text: String) {
        self.model.title = Some(text);
    }

    pub fn set_acc_title(&mut self, text: &str) {
        self.model.acc_title = Some(js::trim_start(text).to_string());
    }

    pub fn set_acc_description(&mut self, text: &str) {
        self.model.acc_descr = Some(crate::diagrams::squeeze_line_starts(text));
    }
}
