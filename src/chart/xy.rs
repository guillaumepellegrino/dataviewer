use crate::chart::{View, Chart};
use crate::canvas::{Canvas, Palette, Tooltip};
use crate::dataview;
use crate::utils::PairIterator;

// Plot an XY Chart
#[derive(Default)]
pub struct XY;


fn squaredistance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let x_dt = x2 - x1;
    let y_dt = y2 - y1;
    (y_dt * y_dt) + (x_dt * x_dt)
}

impl Chart for XY {
    fn view(&self, file: &dataview::File) -> View {
        View::xy_minmax(file)
            .sanity_check(file)
            .show_axis()
            .margin()
    }

    // TODO: maybe we should compute first the points
    // in the canvas view.
    // Then we do the drawing.
    fn draw(&self, canvas: &Canvas, file: &dataview::File) {
        let mut tooltip = None;
        let mut tooltip_distance = 200.0;
        canvas.draw_axis();

        let mut palette = Palette::palette1();
        for (key, data) in &file.data {
            let color = palette.next();
            canvas.set_color(&color);

            let mut iter = PairIterator::new(data);
            let (x0, y0) = match iter.next() {
                Some(v) => v,
                None => {return;},
            };
            canvas.move_to(x0, y0);
            canvas.circle(x0, y0, 2.0);

            for (x, y) in iter {
                let (xpixel, ypixel) = canvas.line_to(x, y);
                canvas.circle(x, y, 2.0);
                let distance = squaredistance(xpixel, ypixel, canvas.mouse_x(), canvas.mouse_y());
                if distance < tooltip_distance {
                    tooltip = Some(Tooltip {
                        key: key.clone(),
                        x, y, xpixel, ypixel,
                    });
                    tooltip_distance = distance;
                }
            }
            canvas.stroke();
        }

        if let Some(tooltip) = &tooltip {
            canvas.draw_tooltip(file, tooltip);
        }
    }
}
