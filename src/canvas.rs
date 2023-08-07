use gtk4 as gtk;
use gtk::cairo;
use crate::chart::chart::Range;

pub struct Canvas<'a> {
    cairo: &'a cairo::Context,
    width: f64,
    height: f64,
    range: Range,
}

impl<'a> Canvas<'a> {
    pub fn new(_area: &'a gtk::DrawingArea, cairo: &'a cairo::Context, width: i32, height: i32, range: &Range) -> Self {
        Self {
            cairo,
            width: width as f64,
            height: height as f64,
            range: range.clone(),
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
        let x_range = self.range.x_max - self.range.x_min;
        let x_norm = (x - self.range.x_min) / x_range;
        x_norm * self.width
    }

    fn y_pixel(&self, y: f64) -> f64 {
        let y_range = self.range.y_max - self.range.y_min;
        let y_norm = (y - self.range.y_min) / y_range;
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

    pub fn draw_axis(&self) -> &Self {
        self.cairo.move_to(0.0, self.y_pixel(0.0));
        self.cairo.line_to(self.width, self.y_pixel(0.0));

        self.cairo.move_to(self.x_pixel(0.0), 0.0);
        self.cairo.line_to(self.x_pixel(0.0), self.height);

        /*
        self.cairo.move_to(64.0, 64.0);
        self.cairo.show_text("hello World");
        for i in 0..10 {
        }
        */
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
