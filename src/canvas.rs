use gtk4 as gtk;
use gtk::cairo;
use crate::chart::View;
use crate::dataview;

pub struct Canvas<'a> {
    cairo: &'a cairo::Context,
    width: f64,
    height: f64,
    mouse_x: f64,
    mouse_y: f64,
    view: View,
}

pub struct Tooltip {
    pub key: String,
    pub xpixel: f64,
    pub ypixel: f64,
    pub x: f64,
    pub y: f64,
}

pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

pub struct Palette {
    colors: Vec<u32>,
    current: usize,
}

pub static BLACK: Color = Color {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
};

impl Color {
    pub fn rgb(value: u32) -> Self {
        Self {
            red: ((value & 0xFF0000) >> 16) as f64 / 255.0,
            green: ((value & 0x00FF00) >> 8) as f64 / 255.0,
            blue: (value & 0x0000FF) as f64 / 255.0,
        }
    }
}

impl Palette {
    pub fn new(colors: Vec<u32>) -> Self {
        Self {
            colors,
            current: 0,
        }
    }

    pub fn palette1() -> Self {
        Self::new(vec!(
            0x7D092F, // 2
            0xFB8B24, // 5
            0x5F0F40, // 1
            0x795838, // 9
            0xCB4721, // 4
            0xAE5E26, // 8
            0xE36414, // 7
            0x9A031E, // 3
            0xEF781C, // 6
        ))
    }

    pub fn next(&mut self) -> Color {
        if self.current >= self.colors.len() {
            self.current = 0;
        }
        let color = Color::rgb(
            self.colors[self.current]);
        self.current += 1;
        color
    }
}


impl<'a> Canvas<'a> {
    pub fn new(_area: &'a gtk::DrawingArea, cairo: &'a cairo::Context, width: i32, height: i32, mouse_x: f64, mouse_y: f64, view: &View) -> Self {
        Self {
            cairo,
            width: width as f64,
            height: height as f64,
            mouse_x, mouse_y,
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

    pub fn line_to(&self, x: f64, y: f64) -> (f64, f64) {
        let x = self.x_pixel(x);
        let y = self.y_pixel(y);
        self.cairo.line_to(x, y);
        (x, y)
    }

    pub fn circle(&self, x: f64, y: f64, len: f64) -> (f64, f64) {
        let x = self.x_pixel(x);
        let y = self.y_pixel(y);
        self.cairo.arc(x, y, len, 0.0, 2.0 * std::f64::consts::PI);
        (x, y)
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

    fn x_axis_pos(&self) -> f64 {
        let margin = 30.0;
        let mut y0 = self.y_pixel(0.0);
        if y0 > self.height - margin {
            y0 = self.height - margin;
        }
        else if y0 < margin {
            y0 = margin;
        }
        y0
    }

    fn y_axis_pos(&self) -> f64 {
        let margin = 30.0;
        let mut x0 = self.x_pixel(0.0);
        if x0 > self.width - margin {
            x0 = self.width - margin;
        }
        else if x0 < 50.0 {
            x0 = 50.0;
        }
        x0
    }

    pub fn draw_axis(&self) -> &Self {
        self.set_color(&BLACK);
        let x0 = self.y_axis_pos();
        let y0 = self.x_axis_pos();
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

        self.stroke()
    }

    /// Draw main title centered at the top of the canvas
    fn draw_main_title(&self, file: &dataview::File) -> &Self {
        self.set_color(&BLACK);
        let title = match &file.dataview.title {
            Some(title) => title,
            None => {return self;},
        };

        let fontsize = 24.0;
        self.cairo.set_font_size(fontsize);

        let len = (title.len() as f64) * fontsize;
        let x = (self.width - len/2.0) / 2.0;
        self.cairo.move_to(x, fontsize);
        let _ = self.cairo.show_text(title);

        self.stroke()
    }

    /// Draw x title axis at the bottom right of the canvas
    fn draw_x_title(&self, file: &dataview::File) -> &Self {
        self.set_color(&BLACK);
        let mut text = String::new();
        if let Some(title) = &file.dataview.x_title {
            text += title;
        };
        if let Some(unit) = &file.dataview.x_unit {
            text += &format!(" ({})", unit);
        };

        let fontsize = 12.0;
        self.cairo.set_font_size(fontsize);

        let len = (text.len() as f64) * fontsize;
        let x = self.width - len;
        self.cairo.move_to(x, self.x_axis_pos() + 21.0);

        let _ = self.cairo.show_text(&text);
        self.stroke()
    }

    /// Draw y tile axis at the top left of the canvas
    fn draw_y_title(&self, file: &dataview::File) -> &Self {
        self.set_color(&BLACK);
        let mut text = String::new();
        if let Some(title) = &file.dataview.y_title {
            text += title;
        };
        if let Some(unit) = &file.dataview.y_unit {
            text += &format!(" ({})", unit);
        };

        let fontsize = 12.0;
        self.cairo.set_font_size(fontsize);
        self.cairo.move_to(self.y_axis_pos()+2.0, 45.0);

        let _ = self.cairo.show_text(&text);
        self.stroke()
    }

    pub fn draw_multiline_text(&self, text: &str, mut xpixel: f64, mut ypixel: f64, fontsize: f64) -> &Self {
        self.cairo.set_font_size(fontsize);
        let iter = text.split('\n');

        xpixel += 15.0;
        ypixel += 15.0;
        for line in iter {
            self.cairo.move_to(xpixel, ypixel);
            let _ = self.cairo.show_text(line);
            ypixel += fontsize;
        }
        self
    }

    pub fn draw_tooltip(&self, file: &dataview::File, tooltip: &Tooltip) -> &Self {
        println!("Draw tooltip");
        self.set_color(&BLACK);
        let fontsize = 12.0;
        let xpixel = tooltip.xpixel;
        let ypixel = tooltip.ypixel;
        self.cairo.move_to(xpixel, ypixel);
        self.cairo.arc(xpixel, ypixel, 5.0, 0.0, 2.0 * std::f64::consts::PI);


        let mut text = String::new();
        /*
        if let Some(description) = &file.dataview.description {
            text += description;
            text += "\n";
        }
        */
        if let Some(chart) = &file.chart.get(&tooltip.key) {
            if let Some(title) = &chart.title {
                text += "[";
                text += title;
                text += "]\n";
            }
            if let Some(description) = &chart.description {
                text += description;
                text += "\n";
            }
        }

        let x_title = match &file.dataview.x_title {
            Some(x_title) => x_title,
            None => "x",
        };
        let x_unit = match &file.dataview.x_unit {
            Some(x_unit) => x_unit,
            None => "",
        };
        text += &format!("{}: {} {}\n",
            x_title, tooltip.x, x_unit);

        let y_title = match &file.dataview.y_title {
            Some(y_title) => y_title,
            None => "x",
        };
        let y_unit = match &file.dataview.y_unit {
            Some(y_unit) => y_unit,
            None => "",
        };
        text += &format!("{}: {} {}\n",
            y_title, tooltip.y, y_unit);
        self.draw_multiline_text(&text, xpixel, ypixel, fontsize);
        self.stroke()
    }

    pub fn set_color(&self, color: &Color) -> &Self {
        self.cairo.set_source_rgb(color.red, color.green, color.blue);
        self
    }

    pub fn stroke(&self) -> &Self {
        self.cairo.stroke()
            .expect("Cairo stroke failed");
        self
    }

    pub fn draw(&self, file: &dataview::File) -> &Self {
        self.draw_main_title(file)
            .draw_x_title(file)
            .draw_y_title(file);
        self
    }

    pub fn mouse_x(&self) -> f64 {
        self.mouse_x
    }

    pub fn mouse_y(&self) -> f64 {
        self.mouse_y
    }
}
