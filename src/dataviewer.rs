use crate::canvas::Canvas;
use crate::chart::*;
use crate::dataview;
use eyre::{eyre, Result};
use gtk::cairo;
use gtk::glib::source;
use gtk::prelude::*;
use gtk4 as gtk;

pub struct DataViewer {
    file: dataview::File,
    chart: Option<Box<dyn Chart>>,
    view: View,
    width: f64,
    height: f64,
    mouse_is_pressed: bool,
    mouse_xref: f64,
    mouse_yref: f64,
    redraw_timer: Option<source::SourceId>,
    draw_area: Option<gtk::DrawingArea>,
    autoview: bool,
}

impl DataViewer {
    pub fn new() -> Self {
        Self {
            file: dataview::File::default(),
            chart: None,
            view: View::new(),
            width: 1.0,
            height: 1.0,
            mouse_is_pressed: false,
            mouse_xref: 0.0,
            mouse_yref: 0.0,
            redraw_timer: None,
            draw_area: None,
            autoview: true,
        }
    }

    pub fn load(&mut self, file: dataview::File) -> Result<()> {
        self.file = file;

        println!("load: {:?}", self.file);

        for key in self.file.chart.keys() {
            if self.file.data.get(key).is_none() {
                self.file.data.insert(key.clone(), vec![]);
            }
        }

        //println!("file: {:?}", file);
        let chart = match self.file.dataview.r#type {
            dataview::Type::XY => Box::new(xy::XY),
            r#type => {
                return Err(eyre!("Unimplemented format '{:?}'", r#type));
            }
        };
        self.view = chart.view(&self.file);
        self.chart = Some(chart);
        Ok(())
    }

    pub fn save_as(&self, path: &std::path::Path) -> Result<()> {
        let toml = toml::to_string(&self.file)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    pub fn export_as_png(&mut self, area: &gtk::DrawingArea, path: &std::path::Path) -> Result<()> {
        let width = self.width as i32;
        let height = self.height as i32;
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)?;
        let cairo = cairo::Context::new(&surface)?;
        cairo.set_source_rgb(1.0, 1.0, 1.0);
        cairo.fill()?;
        cairo.paint()?;
        self.draw(area, &cairo, width, height);
        let mut file = std::fs::File::create(path)?;
        surface.write_to_png(&mut file)?;
        Ok(())
    }

    pub fn update(&mut self, update: dataview::File) {
        for (key, value) in update.data {
            let data = self.file.data.get_mut(&key);
            let data = match data {
                Some(data) => data,
                None => {
                    continue;
                }
            };
            data.extend(value);
        }
        if self.autoview {
            self.view = self.chart.as_ref().unwrap().view(&self.file);
        }
        self.queue_redraw();
    }

    pub fn draw(
        &mut self,
        area: &gtk::DrawingArea,
        cairo: &cairo::Context,
        width: i32,
        height: i32,
    ) {
        self.draw_area = Some(area.clone());
        self.width = width.into();
        self.height = height.into();
        let chart = match &self.chart {
            Some(chart) => chart,
            None => {
                return;
            }
        };
        let canvas = Canvas::new(
            area,
            cairo,
            width,
            height,
            self.mouse_xref,
            self.mouse_yref,
            &self.view,
        );
        cairo.set_source_rgb(0.0, 0.0, 0.0);
        chart.draw(&canvas, &self.file);
        canvas.draw(&self.file);
    }

    pub fn queue_redraw(&self) {
        if let Some(draw_area) = &self.draw_area {
            draw_area.queue_draw();
        }
    }

    fn move_canvas(&mut self, dx: f64, dy: f64) {
        let view_x = self.view.x_max - self.view.x_min;
        let dx = (dx * view_x) / self.width;
        self.view.x_min -= dx;
        self.view.x_max -= dx;

        let view_y = self.view.y_max - self.view.y_min;
        let dy = (dy * view_y) / self.height;
        self.view.y_min += dy;
        self.view.y_max += dy;

        self.autoview = false;
    }

    pub fn mouse_clicked(&mut self, x: f64, y: f64) {
        self.mouse_is_pressed = true;
        self.mouse_xref = x;
        self.mouse_yref = y;
    }

    pub fn set_redraw_timer(&mut self, timer: Option<source::SourceId>) {
        // Reset the current redraw timer and set the new one.
        if let Some(timer) = self.redraw_timer.take() {
            timer.remove();
        }
        self.redraw_timer = timer;
    }

    pub fn mouse_moved(&mut self, x: f64, y: f64) {
        if self.mouse_is_pressed {
            let dx = x - self.mouse_xref;
            let dy = y - self.mouse_yref;
            self.move_canvas(dx, dy);
        }
        self.mouse_xref = x;
        self.mouse_yref = y;
    }

    pub fn mouse_released(&mut self) {
        self.mouse_is_pressed = false;
    }

    pub fn mouse_is_pressed(&self) -> bool {
        self.mouse_is_pressed
    }

    pub fn mouse_scroll(&mut self, dy: f64) {
        println!("OLD View: {:?}", self.view);
        let view_x = self.view.x_max - self.view.x_min;
        let view_y = self.view.y_max - self.view.y_min;

        let (zoom_x, zoom_y) = match dy > 0.0 {
            true => (view_x * 0.10, view_y * 0.10),
            false => (-view_x * 0.10, -view_y * 0.10),
        };

        self.view.x_min -= zoom_x;
        self.view.x_max += zoom_x;

        self.view.y_min -= zoom_y;
        self.view.y_max += zoom_y;

        self.autoview = false;
        println!("NEW View: {:?}", self.view);
    }

    pub fn set_autoview(&mut self, autoview: bool) {
        self.autoview = autoview;

        if self.autoview {
            if let Some(chart) = self.chart.as_ref() {
                self.view = chart.view(&self.file);
                self.queue_redraw();
            }
        }
    }
}
