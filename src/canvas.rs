use gtk4 as gtk;
use gtk::cairo;
use crate::chart::chart::View;

pub struct Canvas<'a> {
    cairo: &'a cairo::Context,
    width: f64,
    height: f64,
    view: View,
}

impl<'a> Canvas<'a> {
    pub fn new(_area: &'a gtk::DrawingArea, cairo: &'a cairo::Context, width: i32, height: i32, view: &View) -> Self {
        Self {
            cairo,
            width: width as f64,
            height: height as f64,
            view: view.clone(),
        }
    }

    fn x_pixel(&self, x: f64) -> f64 {
        // <-----------width------------>
        //
        // x_min        x           x_max
        //
        // 1. normalize x => [0, 1]
        // 2. pixel = x_norm * width
        //
        // x_norm = (x - x_min) / (x_max - x_min)
        // x_pixel = x_norm * width
        let x_range = self.view.x_max - self.view.x_min;
        let x_norm = (x - self.view.x_min) / x_range;
        x_norm * self.width
    }

    fn y_pixel(&self, y: f64) -> f64 {
        let y_range = self.view.y_max - self.view.y_min;
        let y_norm = (y - self.view.y_min) / y_range;
        self.height - y_norm * self.height
    }

    /*
    pub fn rectangle(&self, x: f64, y: f64) -> &Self {
        self.cairo.rectangle(self.x_pixel(x), self.y_pixel(y), 5.0, 5.0);
        self
    }
    */

    pub fn move_to(&self, x: f64, y: f64) -> &Self {
        self.cairo.move_to(self.x_pixel(x), self.y_pixel(y));
        self
    }

    pub fn line_to(&self, x: f64, y: f64) -> &Self {
        self.cairo.line_to(self.x_pixel(x), self.y_pixel(y));
        self
    }

    /// Try to format float to a nice viewable string for user.
    /// We try to take in account the user view:
    /// - if user zoom in, we add more decimal
    /// - if user zoom out, we remove unecessart decimal
    fn fmtfloat(val: f64, view_range: f64) -> String {
        let abs = view_range;
        if abs > 10000.0 {
            format!("{:.0}", val)
        }
        else if abs > 1000.0 {
            format!("{:.1}", val)
        }
        else if abs > 100.0 {
            format!("{:.2}", val)
        }
        else if abs > 10.0 {
            format!("{:.3}", val)
        }
        else if abs > 0.01 {
            format!("{:.4}", val)
        }
        else if abs > 0.001 {
            format!("{:.5}", val)
        }
        else if abs > 0.0001 {
            format!("{:.6}", val)
        }
        else if abs > 0.00001 {
            format!("{:.7}", val)
        }
        else {
            format!("{:.8}", val)
        }
    }

    pub fn draw_axis(&self) -> &Self {
        let x0 = self.x_pixel(0.0);
        let y0 = self.y_pixel(0.0);
        let x_range = self.view.x_max - self.view.x_min;
        let y_range = self.view.y_max - self.view.y_min;

        // Draw X Axis
        self.cairo.move_to(0.0, y0);
        self.cairo.line_to(self.width, y0);
        for i in 0..11 {
            let i = i as f64;
            let step = x_range / 10.0;
            let start = (self.view.x_min / step).floor() * step;
            let x = start + (i * step);
            self.cairo.move_to(self.x_pixel(x), y0+10.0);
            let _ = self.cairo.show_text(&Self::fmtfloat(x, x_range));
        }

        // Draw Y Axis
        self.cairo.move_to(x0, 0.0);
        self.cairo.line_to(x0, self.height);
        for i in 0..10 {
            let i = i as f64;
            let step = y_range / 10.0;
            let start = (self.view.y_min / step).floor() * step;
            let y = start + (i * step);
            self.cairo.move_to(x0-40.0, self.y_pixel(y));
            let _ = self.cairo.show_text(&Self::fmtfloat(y, y_range));
        }

        self.cairo.set_source_rgb(0.0, 0.0, 0.0);
        self.cairo.stroke()
            .expect("Cairo stroke failed");

        self
    }

    pub fn draw(&self) -> &Self {
        self.cairo.set_source_rgb(0.5, 0.8, 0.8);
        self.cairo.stroke()
            .expect("Cairo stroke failed");
        self
    }
}
