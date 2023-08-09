use crate::chart::chart::{View, Chart};
use crate::canvas::{Canvas, Tooltip};
use crate::dataview;

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
            .show_axis()
            .margin()
    }

    fn draw(&self, canvas: &Canvas, file: &dataview::File) {
        let mut tooltip = None;
        let mut tooltip_distance = 200.0;
        canvas.draw_axis();

        for (key, data) in &file.data {
            let mut iter = data.pair_iter();
            let (x0, y0) = iter.next().unwrap();
            canvas.move_to(x0, y0);

            for (x, y) in iter {
                let (xpixel, ypixel) = canvas.line_to(x, y);
                let distance = squaredistance(xpixel, ypixel, canvas.mouse_x(), canvas.mouse_y());
                if distance < tooltip_distance {
                    tooltip = Some(Tooltip {
                        key: key.clone(),
                        x, y, xpixel, ypixel,
                    });
                    tooltip_distance = distance;
                }
            }
        }

        if let Some(tooltip) = &tooltip {
            canvas.draw_tooltip(file, tooltip);
        }
    }
}
