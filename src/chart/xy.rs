use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use eyre::{eyre, Result};
use crate::chart::chart::{Range, Chart};
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

    fn range(&self, file: &dataview::File) -> Range {
        let mut range = Range::new();

        for (_, data) in &file.data {
            let iter = data.pair_iter();
            for (x, y) in iter {
                if x < range.x_min {
                    range.x_min = x;
                }
                if x > range.x_max {
                    range.x_max = x;
                }
                if y < range.y_min {
                    range.y_min = y;
                }
                if y > range.y_max {
                    range.y_max = y;
                }
            }
        }

        range
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
