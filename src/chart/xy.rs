use crate::chart::chart::{View, Chart};
use crate::canvas::Canvas;
use crate::dataview;

// Plot an XY Chart
#[derive(Default)]
pub struct XY;

impl Chart for XY {
    /*
    fn load(&mut self, reader: &mut BufReader<File>) -> Result<()> {
        self.points.clear();

        for line in reader.lines() {
            let line = line?;
            let mut parts = line.split_whitespace();
            let x = parts.next()
                .ok_or(eyre!("parse X error"))?
                .parse::<f64>()?;
            let y = parts.next()
                .ok_or(eyre!("parse X error"))?
                .parse::<f64>()?;
            self.points.push((x, y));
        }

        Ok(())
    }
    */

    fn view(&self, file: &dataview::File) -> View {
        let mut view = View::new();

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

    fn draw(&self, canvas: &Canvas, file: &dataview::File) {
        canvas.draw_axis();

        for (_, data) in &file.data {
            let mut iter = data.pair_iter();
            let (x0, y0) = iter.next().unwrap();
            canvas.move_to(x0, y0);

            for (x, y) in iter {
                canvas.line_to(x, y);
            }
        }
        /*
        let first = &self.points[0];
        canvas.move_to(first.0, first.1);
        for point in &self.points {
            //canvas.rectangle(point.0, point.1);
            canvas.line_to(point.0, point.1);
        }
        */
    }
}
