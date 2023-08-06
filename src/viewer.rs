use gtk4 as gtk;
use gtk::cairo;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use crate::canvas::Canvas;
use crate::chart::chart::{Range, Chart};
use crate::chart::*;
use crate::dataview;
use eyre::{eyre, Result};

pub struct DataViewer {
    file: dataview::File,
    chart: Option<Box<dyn chart::Chart>>,
    range: Range,
    width: f64,
    height: f64,
    mouse_is_pressed: bool,
    mouse_xref: f64,
    mouse_yref: f64,
}


impl DataViewer {
    pub fn new() -> Self {
        Self {
            file: dataview::File::default(),
            chart: None,
            range: Range::new(),
            width: 1.0,
            height: 1.0,
            mouse_is_pressed: false,
            mouse_xref: 0.0,
            mouse_yref: 0.0,
        }
    }

    pub fn open(&mut self, path: &PathBuf) -> Result<()> {
        // Open the File
        let string = std::fs::read_to_string(path)?;

        self.file = toml::from_str(&string)?;
        //println!("file: {:?}", file);
        let mut chart = match self.file.dataview.r#type {
            dataview::Type::xy => Box::new(xy::XY::default()),
            r#type => {return Err(eyre!("Unimplemented format '{:?}'", r#type));},
        };
        self.range = chart.range(&self.file).margin();
        self.chart = Some(chart);
        Ok(())
    }

    // TODO: implement zoomin(), zoomout(), move()
    pub fn draw(&mut self, area: &gtk::DrawingArea, cairo: &cairo::Context, width: i32, height: i32) {
        self.width = width.into();
        self.height = height.into();
        let chart = match &self.chart {
            Some(chart) => chart,
            None => {return;},
        };
        let canvas = Canvas::new(area, cairo, width, height, &self.range);
        chart.draw(&canvas, &self.file);
        canvas.draw();
    }

    fn move_canvas(&mut self, dx: f64, dy: f64) {
        let range_x = self.range.x_max - self.range.x_min;
        let dx = (dx * range_x) / self.width;
        self.range.x_min -= dx;
        self.range.x_max -= dx;

        let range_y = self.range.y_max - self.range.y_min;
        let dy = (dy * range_y) / self.height;
        self.range.y_min += dy;
        self.range.y_max += dy;
    }

    pub fn mouse_clicked(&mut self, x: f64, y: f64) {
        self.mouse_is_pressed = true;
        self.mouse_xref = x;
        self.mouse_yref = y;
    }

    pub fn mouse_moved(&mut self, x: f64, y: f64) {
        if !self.mouse_is_pressed {
            return;
        }

        let dx = x - self.mouse_xref;
        let dy = y - self.mouse_yref;
        self.move_canvas(dx, dy);

        self.mouse_xref = x;
        self.mouse_yref = y;
    }

    pub fn mouse_released(&mut self) {
        self.mouse_is_pressed = false;
    }

    pub fn mouse_is_pressed(&self) -> bool {
        self.mouse_is_pressed
    }
}

