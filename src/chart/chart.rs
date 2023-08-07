use crate::canvas::Canvas;
use crate::dataview;

#[derive(Clone, Debug)]
pub struct Range {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl Range {
    pub fn new() -> Self {
        Self {
            x_min: f64::MAX,
            x_max: f64::MIN,
            y_min: f64::MAX,
            y_max: f64::MIN,
        }
    }

    pub fn margin(&self) -> Self {
        let x_range = self.x_max - self.x_min;
        let y_range = self.y_max - self.y_min;
        Self {
            x_min: self.x_min - 0.05 * x_range,
            x_max: self.x_max + 0.05 * x_range,
            y_min: self.y_min - 0.05 * y_range,
            y_max: self.y_max + 0.05 * y_range,
        }
    }
}

pub trait Chart {
    fn range(&self, _file: &dataview::File) -> Range {
        Range {
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
        }
    }

    // Draw the Chart
    fn draw(&self, canvas: &Canvas, file: &dataview::File);
}

