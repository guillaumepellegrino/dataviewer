use gtk4 as gtk;
use gtk::{glib, gio};
use gtk::prelude::*;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::path::PathBuf;
use std::collections::{HashMap, VecDeque};
use async_std::os::unix::net::UnixListener;
use async_std::prelude::*;
use eyre::{Result};

mod canvas;
mod chart;
mod dataview;
mod dataviewer;

struct AppContext {
    files: Vec<PathBuf>,
}

struct WindowContext {
    _window: gtk::ApplicationWindow,
    notebook: gtk::Notebook,
    dataviewers: Vec<Weak<RefCell<dataviewer::DataViewer>>>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            files: vec!(),
        }
    }

    pub fn files(&mut self) -> &mut Vec<PathBuf> {
        &mut self.files
    }
}

impl WindowContext {
    pub fn new(app: &gtk::Application) -> Self {
        println!("New window !");
        // Create a new window (the user may open multiple windows)
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .default_width(900)
            .default_height(600)
            .title("Data Viewer")
            .build();
        // Create the title bar
        let titlebar = gtk::HeaderBar::new();
        // Create the notebook (tabs manager)
        let notebook = gtk::Notebook::new();
        window.set_child(Some(&notebook));

        let context = Self {
            _window: window.clone(),
            notebook: notebook.clone(),
            dataviewers: vec!(),
        };

        // Create the Open File button and Dialog
        let buttons = [("Open", gtk::ResponseType::Ok)];
        let openfile = gtk::FileChooserDialog::new(
            Some("Open file to view"),
            Some(&window),
            gtk::FileChooserAction::Open,
            &buttons,
            );
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
            WindowContext::dataviewer_from_file(&notebook, &filename);
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
        context
    } 

    // Duplicate functions bellow. need to be fixed
    pub fn new_dataviewer(&mut self, file: dataview::File) -> Result<Rc<RefCell<dataviewer::DataViewer>>> {
        let dataviewer = Rc::new(RefCell::new(dataviewer::DataViewer::new()));
        dataviewer.borrow_mut().load(file)?;
        let draw_area = new_draw_area_from_dataviewer(dataviewer.clone());
        let label = gtk::Label::new(Some("Unknown"));
        self.notebook.append_page(&draw_area, Some(&label));
        draw_area.queue_draw();

        let weak = Rc::<RefCell<dataviewer::DataViewer>>::downgrade(&dataviewer);
        self.dataviewers.push(weak);

        Ok(dataviewer)
    }
    pub fn new_dataviewer_from_file(&self, path: &PathBuf) {
        let dataviewer = Rc::new(RefCell::new(dataviewer::DataViewer::new()));
        if let Err(e) = dataviewer.borrow_mut().open(&path) {
            println!("Failed to open {:?}: {:?}", path, e);
        }
        let draw_area = new_draw_area_from_dataviewer(dataviewer);
        let filename = path.file_name().unwrap().to_string_lossy();
        let label = gtk::Label::new(Some(&filename));
        self.notebook.append_page(&draw_area, Some(&label));
        draw_area.queue_draw();
    }
    pub fn dataviewer_from_file(notebook: &gtk::Notebook, path: &PathBuf) {
        let dataviewer = Rc::new(RefCell::new(dataviewer::DataViewer::new()));
        if let Err(e) = dataviewer.borrow_mut().open(&path) {
            println!("Failed to open {:?}: {:?}", path, e);
        }
        let draw_area = new_draw_area_from_dataviewer(dataviewer);
        let filename = path.file_name().unwrap().to_string_lossy();
        let label = gtk::Label::new(Some(&filename));
        notebook.append_page(&draw_area, Some(&label));
        draw_area.queue_draw();
    }

}

struct Stream {
    buffer: std::collections::VecDeque::<u8>,
    input: gio::InputStream,
}

impl Stream {
    fn new(iostream: &gio::SocketConnection) -> Self {
        Self {
            buffer: VecDeque::new(),
            input: iostream.input_stream(),
        }
    }

    async fn read_utf8_upto(&mut self, upto: u8) -> String {
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

            let mut buffer = Vec::<u8>::with_capacity(4096);
            buffer.resize(4096, 0);
            let (mut buffer, size) = self.input.read_future(buffer, glib::source::Priority::DEFAULT)
                .await.unwrap();

            buffer.truncate(size);
            self.buffer.extend(buffer);

            if size == 0 {
                panic!("end of file");
            }
        }
    }
}

fn new_draw_area_from_dataviewer(g_dataviewer: Rc<RefCell<dataviewer::DataViewer>>) -> gtk::DrawingArea {
    // Create the Draw Area
    let draw_area = gtk::DrawingArea::new();
    draw_area.set_content_width(128);
    draw_area.set_content_height(128);

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

fn server_handle_load_file(window: &mut WindowContext, file: dataview::File) {
    window.new_dataviewer(file).unwrap();
}

fn server_handle_merge_file(window: &WindowContext, file: dataview::File) {
    
}

fn server_handle_update(window: &WindowContext, update: HashMap<String, dataview::Data>) {
    let dataviewer = match window.dataviewers.first() {
        Some(dataviewer) => dataviewer,
        None => {return;},
    };
    let dataviewer = match dataviewer.upgrade() {
        Some(dataviewer) => dataviewer,
        None => {return;},
    };
    let mut dataviewer = dataviewer.borrow_mut();
    dataviewer.update(update);
}
  
fn server_handle_message(window: &mut WindowContext, message: dataview::Message) {
     println!("message = {:?}", message);
     match message {
        dataview::Message::None => {},
        dataview::Message::Load(file) => server_handle_load_file(window, file),
        dataview::Message::Merge(file) => server_handle_merge_file(window, file),
        dataview::Message::Delete(_) => {},
        dataview::Message::Update(update) => server_handle_update(window, update),
    }
}

fn server_new(app: &gtk::Application) {
    let ipc_name = "/tmp/dataviewer.ipc";
    let main_context = glib::MainContext::default();
    let app = app.clone();
    main_context.spawn_local(async move {
        println!("Listening on ipc://{}", ipc_name);
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

            // Read dataview::File from ipc socket
            let mut window = WindowContext::new(&app);

            let main_context = glib::MainContext::default();
            main_context.spawn_local(async move {
                let mut stream = Stream::new(&client);
                loop {
                    let buff = stream.read_utf8_upto(0).await;
                    //let update : HashMap<String, dataview::Data> = toml::from_str(&buff).unwrap();
                    let message : dataview::Message = toml::from_str(&buff).unwrap();
                    server_handle_message(&mut window, message);
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

    let g_context = Rc::new(RefCell::new(AppContext::new()));
    let context = g_context.clone();

    app.connect_command_line(move |app, cmdline| {
        let mut context = context.borrow_mut();
        let cwd = match cmdline.cwd() {
            Some(cwd) => cwd,
            None => {return 1;},
        };
        for arg in cmdline.arguments() {
            let path = PathBuf::from(arg);
            let path = match path.is_absolute() {
                true => path,
                false => cwd.join(path),
            };
            context.files().push(path);
        }
        drop(context);
        app.activate();
        0
    });

    let context = g_context.clone();
    app.connect_activate(move |app| {
        let mut context = context.borrow_mut();
        let mut window = WindowContext::new(app);

        // Check if there are files to open
        let files = context.files();
        for file in files.iter().skip(1) {
            window.new_dataviewer_from_file(&file);
        }
        files.clear();
    });
    app.connect_startup(move |app| {
        server_new(app);
    });

    app.run()
}

