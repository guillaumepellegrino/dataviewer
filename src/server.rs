use gtk4 as gtk;
use gtk::{glib, gio};
use gtk::prelude::*;
use std::path::PathBuf;
use crate::*;

fn server_handle_update(window: &gtk::Window, update: dataview::File) {
    let notebook = window.get_notebook();
    let page = match notebook.pages().item(0) {
        Some(page) => page.downcast::<gtk::NotebookPage>().unwrap(),
        None => {return;},
    };
    let draw_area = page.child().downcast::<gtk::DrawingArea>().unwrap();
    let context = draw_area.get_context();
    context.dataviewer.update(update);
}

fn server_handle_message(window: &gtk::Window, file: dataview::File) {
     println!("message = {:?}", file);

     if !file.chart.is_empty() {
         let _ = window.new_draw_area(file, "ipc://tmp/dataviewer.ipc");
     }
     else if !file.data.is_empty() {
         server_handle_update(window, file);
     }
}

pub fn ipc_running() -> bool {
    let ipc_name = "/tmp/dataviewer.ipc";

    match std::os::unix::net::UnixStream::connect(ipc_name) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn ipc_listen_socket() -> gio::Socket {
    let ipc_name = "/tmp/dataviewer.ipc";
    let path = PathBuf::from(ipc_name);
    let _ = std::fs::remove_file(&path);
    let address = gio::UnixSocketAddress::new(
        &PathBuf::from(&path));
    let server = gio::Socket::new(
        gio::SocketFamily::Unix,
        gio::SocketType::Stream,
        gio::SocketProtocol::Default).unwrap();
    server.bind(&address, true).unwrap();
    server.listen().unwrap();
    println!("Listening on ipc://{}", ipc_name);
    server
}

pub fn run(app: &gtk::Application, server: gio::Socket) {
    let main_context = glib::MainContext::default();
    let app = app.clone();
    main_context.spawn_local(async move {

        let listener = gio::SocketListener::new();
        listener.add_socket(&server, None as Option<&glib::Object>).unwrap();

        loop {
            let (client,_) = listener.accept_future().await.unwrap();
            println!("New ipc client connected: Opening a new Window");

            let window = match app.find_empty_window() {
                Some(window) => window,
                None => app.new_window(),
            };
            window.present();

            // Read dataview::File from ipc socket
            let main_context = glib::MainContext::default();
            main_context.spawn_local(async move {
                let mut stream = stream::Stream::new(&client);
                loop {
                    let buff = stream.read_utf8_upto(0).await;
                    let message : dataview::File = toml::from_str(&buff).unwrap();
                    server_handle_message(&window, message);
                }
            });
        }
    });
}
