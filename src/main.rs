use gtk4 as gtk;
use gtk::{glib, gio};
use gtk::prelude::*;
use std::rc::{Rc};
use std::cell::RefCell;
use std::path::PathBuf;
use eyre::{Result};

mod canvas;
mod chart;
mod dataview;
mod dataviewer;
mod stream;
mod utils;

static ME: &str = "dv";

/// Extend DataViewer Window with some utils functions
trait ApplicationDVExt {
    fn new_dataviewer_window(&self) -> gtk::Window;
    fn find_empty_window(&self) -> Option<gtk::Window>;
}

/// Extend DataViewer Window with some utils functions
trait WindowDVExt {
    fn new_draw_area(&self, file: dataview::File, label: &str) -> Result<gtk::DrawingArea>;
    fn new_draw_area_from_file(&self, path: &PathBuf) -> Result<gtk::DrawingArea>;
    fn get_notebook(&self) -> gtk::Notebook;
}

/// Extend DataViewer Notebook (tabs) with some utils functions
trait DrawingAreaDVExt {
    fn set_context(&self, context: DrawingAreaContext);
    fn get_context<'a>(&'a self) -> &'a mut DrawingAreaContext;
}

impl ApplicationDVExt for gtk::Application {
    fn new_dataviewer_window(&self) -> gtk::Window {
        println!("New window !");
        // Create a new window (the user may open multiple windows)
        let window = gtk::ApplicationWindow::builder()
            .application(self)
            .default_width(900)
            .default_height(600)
            .title("Data Viewer")
            .build();
        // Create the title bar
        let titlebar = gtk::HeaderBar::new();
        // Create the notebook (tabs manager)
        let notebook = gtk::Notebook::new();
        window.set_child(Some(&notebook));

        // Create the Open File button and Dialog
        let buttons = [("Open", gtk::ResponseType::Ok)];
        let openfile = gtk::FileChooserDialog::new(
            Some("Open file to view"),
            Some(&window),
            gtk::FileChooserAction::Open,
            &buttons,
            );

        let windowref = window.clone().upcast::<gtk::Window>();
        openfile.connect_response(move |file, response| {
            file.hide();
            if response != gtk::ResponseType::Ok {
                return;
            }
            let filename = match file.file() {
                Some(filename) => filename,
                None => {return;},
            };
            let filename = match filename.path() {
                Some(filename) => filename,
                None => {return;},
            };
            println!("Opening {:?}", filename);
            let _ = windowref.new_draw_area_from_file(&filename);
        });
        let openbutton = gtk::Button::with_label("Open");
        openbutton.connect_clicked(move |_| {
            openfile.present();

        });
        titlebar.pack_start(&openbutton);

        // Create the Save File button and Dialog
        let button2 = gtk::Button::with_label("Save");
        button2.connect_clicked(|_| {
            println!("Save");
        });
        titlebar.pack_end(&button2);

        // Create the Export File button and Dialog
        let button3 = gtk::Button::with_label("Export");
        button3.connect_clicked(|_| {
            println!("Export as PNG image");
        });
        titlebar.pack_end(&button3);

        window.set_titlebar(Some(&titlebar));
        window.show();
        window.into()
    }

    fn find_empty_window(&self) -> Option<gtk::Window> {
        for window in self.windows() {
            let notebook = window.get_notebook();
            if notebook.pages().n_items() == 0 {
                return Some(window);
            }
        }
        None
    }
}

impl WindowDVExt for gtk::Window {
    fn new_draw_area(&self, file: dataview::File, label: &str) -> Result<gtk::DrawingArea> {
        let dataviewer = Rc::new(RefCell::new(dataviewer::DataViewer::new()));
        dataviewer.borrow_mut().load(file)?;
        let draw_area = new_draw_area_from_dataviewer(dataviewer.clone());
        let label = gtk::Label::new(Some(label));
        let notebook = self.get_notebook();
        notebook.append_page(&draw_area, Some(&label));
        draw_area.queue_draw();
        Ok(draw_area)
    }

    fn new_draw_area_from_file(&self, path: &PathBuf) -> Result<gtk::DrawingArea> {
        let filename = path.file_name().unwrap().to_string_lossy();
        let string = std::fs::read_to_string(path)?;
        let file = toml::from_str(&string)?;
        self.new_draw_area(file, &filename)
    }

    fn get_notebook(&self) -> gtk::Notebook {
        let widget = self.child().unwrap();
        widget.downcast::<gtk::Notebook>().unwrap()
    }
}

// Get or Set our internal context from a Notebook
impl DrawingAreaDVExt for gtk::DrawingArea {
    fn set_context(&self, context: DrawingAreaContext) {
        unsafe {
            self.set_data::<DrawingAreaContext>(ME, context);
        }
    }

    fn get_context<'a>(&'a self) -> &'a mut DrawingAreaContext {
        unsafe {
            self.data::<DrawingAreaContext>(ME).unwrap().as_mut()
        }
    }
}

// Should store the dataviewer context
struct DrawingAreaContext {
    // FIXME: We can probably remove Rc<RefCell<T>>
    dataviewer: Rc<RefCell<dataviewer::DataViewer>>,
}

fn new_draw_area_from_dataviewer(g_dataviewer: Rc<RefCell<dataviewer::DataViewer>>) -> gtk::DrawingArea {
    // Create the Draw Area
    let draw_area = gtk::DrawingArea::new();
    draw_area.set_content_width(128);
    draw_area.set_content_height(128);

    // Set the Draw Area Context
    draw_area.set_context(DrawingAreaContext {
        dataviewer: g_dataviewer.clone(),
    });

    let _= draw_area.get_context();

    // Notify DataViewer when canvas need to be redraw
    let dataviewer = g_dataviewer.clone();
    draw_area.set_draw_func(move |draw_area, cairo, width, height| {
        println!("Draw area {}x{}", width, height);
        let mut dataviewer = dataviewer.borrow_mut();
        dataviewer.draw(draw_area, cairo, width, height);
    });

    // Notify DataViewer when mouse is clicked or released
    let key_ctl = gtk::GestureClick::new();
    let dataviewer = g_dataviewer.clone();
    key_ctl.connect_pressed(move |_,_,x,y| {
        let mut dataviewer = dataviewer.borrow_mut();
        dataviewer.mouse_clicked(x, y);
    });
    let dataviewer = g_dataviewer.clone();
    key_ctl.connect_released(move |_,_,_,_| {
        let mut dataviewer = dataviewer.borrow_mut();
        dataviewer.mouse_released();
    });
    draw_area.add_controller(key_ctl);

    // Notify DataViewer when mouse is moved
    let motion_ctl = gtk::EventControllerMotion::new();
    let dataviewer = g_dataviewer.clone();
    motion_ctl.connect_motion(move |ctl,x,y| {
        let dataviewer2 = dataviewer.clone();
        let ctl2 = ctl.clone();
        let mut dataviewer = dataviewer.borrow_mut();
        let timer = glib::source::timeout_add_local_once(
            std::time::Duration::from_millis(50), move ||
        {
            println!("redraw!");
            let mut dataviewer = dataviewer2.borrow_mut();
            dataviewer.set_redraw_timer(None);
            ctl2.widget().queue_draw();
        });
        dataviewer.set_redraw_timer(Some(timer));
        dataviewer.mouse_moved(x, y);
        if dataviewer.mouse_is_pressed() {
            ctl.widget().queue_draw();
        }
    });
    draw_area.add_controller(motion_ctl);

    // Notify DataViewer when mouse is scrolling
    let scroll_ctl = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL);
    let dataviewer = g_dataviewer.clone();
    scroll_ctl.connect_scroll(move |ctl,_,dy| {
        let mut dataviewer = dataviewer.borrow_mut();
        dataviewer.mouse_scroll(dy);
        ctl.widget().queue_draw();
        glib::signal::Propagation::Proceed
    });
    draw_area.add_controller(scroll_ctl);

    draw_area
}

fn server_handle_update(window: &gtk::Window, update: dataview::File) {
    let notebook = window.get_notebook();
    let page = match notebook.pages().item(0) {
        Some(page) => page.downcast::<gtk::NotebookPage>().unwrap(),
        None => {return;},
    };
    let draw_area = page.child().downcast::<gtk::DrawingArea>().unwrap();
    let context = draw_area.get_context();
    let mut dataviewer = context.dataviewer.borrow_mut();
    dataviewer.update(update);
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

fn server_new(app: &gtk::Application) {
    let ipc_name = "/tmp/dataviewer.ipc";
    let main_context = glib::MainContext::default();
    let app = app.clone();
    println!("Listening on ipc://{}", ipc_name);
    main_context.spawn_local(async move {
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

        let listener = gio::SocketListener::new();
        listener.add_socket(&server, None as Option<&glib::Object>).unwrap();

        loop {
            let (client,_) = listener.accept_future().await.unwrap();
            println!("New ipc client connected: Opening a new Window");

            let window = match app.find_empty_window() {
                Some(window) => window,
                None => app.new_dataviewer_window(),
            };

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

fn main() -> glib::ExitCode {
    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::HANDLES_COMMAND_LINE);

    let app = gtk::Application::builder()
        .application_id("org.gtk.dataviewer")
        .flags(flags)
        .build();

    app.connect_command_line(move |app, cmdline| {
        let window = app.new_dataviewer_window();
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
        server_new(app);
    });

    app.run()
}

