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
        /*
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Verify this is a dataviewer file
        let mut program = String::new();
        reader.read_line(&mut program)?;
        if !program.contains("dataviewer") {
            return Err(eyre!("Not a dataviewer file"));
        }

        // Create a chart of the specified type
        let mut chart = String::new();
        reader.read_line(&mut chart)?;
        let mut chart = match chart.as_str() {
            "xy\n" => Box::new(xy::XY::default()),
            _ => {return Err(eyre!("Unknown chart format '{}'", chart));},
        };

        // Load data into the chart
        chart.load(&mut reader)?;
        self.chart = Some(chart);
        */

        Ok(())
    }

    // TODO: implement zoomin(), zoomout(), move()
    pub fn draw(&self, area: &gtk::DrawingArea, cairo: &cairo::Context, width: i32, height: i32) {
        let chart = match &self.chart {
            Some(chart) => chart,
            None => {return;},
        };
        let canvas = Canvas::new(area, cairo, width, height, &self.range);
        chart.draw(&canvas, &self.file);
        canvas.draw();
    }

    fn move_canvas(&mut self, dx: f64, dy: f64) {
        println!("move {}x{}", dx, dy);

        self.range.x_min += dx / 10.0;
        self.range.x_max += dx / 10.0;
        self.range.y_min += dy / 10.0;
        self.range.y_max += dy / 10.0;
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

