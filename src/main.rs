use gtk4 as gtk;
use gtk::{glib, gio};
use gtk::prelude::*;
use ui::*;
use std::path::PathBuf;

mod canvas;
mod chart;
mod dataview;
mod dataviewer;
mod server;
mod stream;
mod ui;
mod utils;

fn main() -> glib::ExitCode {
    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::HANDLES_COMMAND_LINE);

    let app = gtk::Application::builder()
        .application_id("org.gtk.dataviewer")
        .flags(flags)
        .build();

    app.connect_command_line(move |app, cmdline| {
        let window = app.new_window();
        let cwd = match cmdline.cwd() {
            Some(cwd) => cwd,
            None => {return 1;},
        };
        for arg in cmdline.arguments().iter().skip(1) {
            let path = PathBuf::from(arg);
            let path = match path.is_absolute() {
                true => path,
                false => cwd.join(path),
            };
            let _ = window.new_draw_area_from_file(&path);
        }
        0
    });
    app.connect_startup(move |app| {
        server::run(app);
    });

    app.run()
}
