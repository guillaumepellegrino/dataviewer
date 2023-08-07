use gtk4 as gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;

mod canvas;
mod chart;
mod dataview;
mod dataviewer;

/*
extern "C" {
fn gtk_widget_add_events(widget: *mut gtk::ffi::GtkWidget, events: isize);
}

fn add_events(widget: &impl IsA<gtk::Widget>, events: isize) {
    //let ptr : gtk::ffi::GtkWidget = widget.as_ptr();
    //
    let ptr : *mut gtk::ffi::GtkWidget = widget.as_ref().to_glib_none().0;
    //unsafe {gtk_widget_add_events(ptr, events)};
}
*/

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
        let mut dataviewer = dataviewer.borrow_mut();
        if dataviewer.mouse_is_pressed() {
            dataviewer.mouse_moved(x, y);
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


fn main() -> gtk::glib::ExitCode {
    let mut flags = gtk::gio::ApplicationFlags::empty();
    flags.insert(gtk::gio::ApplicationFlags::HANDLES_COMMAND_LINE);

    let app = gtk::Application::builder()
        .application_id("org.gtk.dataviewer")
        .flags(flags)
        .build();

    // Files opened by the application in command line.
    let g_files = Rc::new(RefCell::new(Vec::<PathBuf>::new()));

    let files = g_files.clone();
    app.connect_command_line(move |app, cmdline| {
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
            //let _ = dataviewer.borrow_mut().open(&path);
            files.borrow_mut().push(path);
        }
        app.activate();
        0
    });

    let files = g_files.clone();
    app.connect_activate(move |app| {
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
        let mut files = files.borrow_mut();
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
    });
    app.run()
}
