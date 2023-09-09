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
    // Open ipc listening socket if it's not already running
    let ipc = match server::ipc_running() {
        false => Some(server::ipc_listen_socket()),
        true => None,
    };

    // deamonize the application if streaming is required
    for arg in std::env::args() {
        if arg == "--stream" {
            let daemon = daemonize::Daemonize::new();
            if let Err(e) = daemon.start() {
                println!("Failed to daemonize: {:?}", e);
            }
        }
    }

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
            if arg.to_str() == Some("--stream") {
                continue;
            }
            let path = PathBuf::from(arg);
            let path = match path.is_absolute() {
                true => path,
                false => cwd.join(path),
            };
            if let Err(e) = window.new_draw_area_from_file(&path) {
                println!("Failed to open {:?}: {:?}", path, e);
            }
        }
        0
    });
    app.connect_startup(move |app| {
        if let Some(ipc) = ipc.clone() {
            server::run(app, ipc);
        }
    });

    app.run()
}
