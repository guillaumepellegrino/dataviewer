use gtk::prelude::*;
use gtk::{gio, glib};
use gtk4 as gtk;
use std::collections::VecDeque;

pub struct Stream {
    buffer: std::collections::VecDeque<u8>,
    input: gio::InputStream,
}

impl Stream {
    pub fn new(iostream: &gio::SocketConnection) -> Self {
        Self {
            buffer: VecDeque::new(),
            input: iostream.input_stream(),
        }
    }

    pub async fn read_utf8_upto(&mut self, upto: u8) -> String {
        let mut string = String::new();
        loop {
            while let Some(c) = self.buffer.pop_front() {
                if c == upto {
                    return string;
                }
                if c.is_ascii() {
                    string.push(c as char);
                }
            }

            let buffer = vec![0; 4096];
            let (mut buffer, size) = self
                .input
                .read_future(buffer, glib::source::Priority::DEFAULT)
                .await
                .unwrap();

            if size == 0 {
                return string;
            }
            buffer.truncate(size);
            self.buffer.extend(buffer);
        }
    }
}
