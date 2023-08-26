use crate::canvas::Canvas;
use crate::dataview;

#[derive(Clone, Debug)]
pub struct View {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl View {
    pub fn new() -> Self {
        Self {
            x_min: f64::MAX,
            x_max: f64::MIN,
            y_min: f64::MAX,
            y_max: f64::MIN,
        }
    }

    pub fn sanity_check(&self, file: &dataview::File) -> Self {
        let mut empty = true;
        for (_, data) in &file.data {
            if data.data.len() > 2 {
                empty = false;
            }
        }

        match empty {
            true => Self {
                x_min: -1.0,
                x_max: 1.0,
                y_min: -1.0,
                y_max: 1.0,
            },
            false => self.clone(),
        }
    }

    pub fn xy_minmax(file: &dataview::File) -> Self {
        let mut view = Self::new();
        for (_, data) in &file.data {
            let iter = data.pair_iter();
            for (x, y) in iter {
                if x < view.x_min {
                    view.x_min = x;
                }
                if x > view.x_max {
                    view.x_max = x;
                }
                if y < view.y_min {
                    view.y_min = y;
                }
                if y > view.y_max {
                    view.y_max = y;
                }
            }
        }

        view
    }

    pub fn show_axis(&self) -> Self {
        let mut new = self.clone();
        if new.x_min > 0.0 {
            new.x_min = 0.0;
        }
        if new.x_max < 0.0 {
            new.x_max = 0.0;
        }
        if new.y_min > 0.0 {
            new.y_min = 0.0;
        }
        if new.y_max < 0.0 {
            new.y_max = 0.0;
        }
        new
    }

    pub fn margin(&self) -> Self {
        let x_range = self.x_max - self.x_min;
        let y_range = self.y_max - self.y_min;
        Self {
            x_min: self.x_min - 0.07 * x_range,
            x_max: self.x_max + 0.07 * x_range,
            y_min: self.y_min - 0.07 * y_range,
            y_max: self.y_max + 0.07 * y_range,
        }
    }
}

pub trait Chart {
    fn view(&self, _file: &dataview::File) -> View;

    // Draw the Chart
    fn draw(&self, canvas: &Canvas, file: &dataview::File);
}

