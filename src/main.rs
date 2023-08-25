use gtk4 as gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use std::collections::HashMap;
use async_std::os::unix::net::UnixListener;
use async_std::prelude::*;

mod canvas;
mod chart;
mod dataview;
mod dataviewer;

struct AppContext {
    files: Vec<PathBuf>,
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
        let timer = gtk::glib::source::timeout_add_local_once(
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
        gtk::glib::signal::Propagation::Proceed
    });
    draw_area.add_controller(scroll_ctl);



    draw_area
}

fn dataviewer_from_file(notebook: &gtk::Notebook, path: &PathBuf) {
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

fn window_new(app: &gtk::Application, files: &mut Vec<PathBuf>) {
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

    // Check if there are files to open
    for file in files.iter().skip(1) {
        dataviewer_from_file(&notebook, &file);
    }
    files.clear();

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
        dataviewer_from_file(&notebook, &filename);

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
}

fn server_new(_app: &gtk::Application) {
    let ipc_path = "/tmp/dataviewer.ipc";
    println!("Start ipc listening socket on {}", ipc_path);
    async_std::task::spawn(async move {
        // Let's build a webserver, here.
        // So, than user can push its data with a simple 'curl'
        let path = PathBuf::from(ipc_path);
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).await.unwrap();
        loop {
            let (socket, _addr) = listener.accept().await.unwrap();
            async_std::task::spawn(async {
                let reader = async_std::io::BufReader::new(socket);
                let mut split = reader.split(b'\0');

                // Read dataview::File from ipc socket

                while let Some(buff) = split.next().await {
                    let buff = buff.unwrap();
                    let s = std::str::from_utf8(&buff).unwrap();
                    // printf "1.data=[1,2]\n2.data=[4,5]\0" | nc -U /tmp/dataviewer.ipc
                    //
                    let update : HashMap<String, dataview::Data> = toml::from_str(&s).unwrap();
                    println!("update = {:?}", update);
                }
            });
        }
    });
}



fn main() -> gtk::glib::ExitCode {
    let mut flags = gtk::gio::ApplicationFlags::empty();
    flags.insert(gtk::gio::ApplicationFlags::HANDLES_COMMAND_LINE);

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
        window_new(app, context.files());
    });
    app.connect_startup(move |app| {
        server_new(app);
    });

    app.run()
}
