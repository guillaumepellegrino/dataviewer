use gtk4 as gtk;
use gtk::{glib};
use gtk::prelude::*;
use std::path::PathBuf;
use eyre::{Result};
use crate::*;

static ME: &str = "dv";

/// Extend DataViewer Window with some utils functions
pub trait ApplicationDVExt {
    fn new_window(&self) -> gtk::Window;
    fn find_empty_window(&self) -> Option<gtk::Window>;
}

/// Extend DataViewer Window with some utils functions
pub trait WindowDVExt {
    fn new_draw_area(&self, file: dataview::File, label: &str) -> Result<gtk::DrawingArea>;
    fn new_draw_area_from_file(&self, path: &PathBuf) -> Result<gtk::DrawingArea>;
    fn get_notebook(&self) -> gtk::Notebook;
}

/// Extend DataViewer Notebook (tabs) with some utils functions
pub trait DrawingAreaDVExt {
    fn from_dataviewer(dataviewer: dataviewer::DataViewer) -> Self;
    fn set_context(&self, context: DrawingAreaContext);
    fn get_context<'a>(&'a self) -> &'a mut DrawingAreaContext;
}

pub struct DrawingAreaContext {
    pub dataviewer: dataviewer::DataViewer,
}

impl ApplicationDVExt for gtk::Application {
    /// Create a new Window for DataViewer application
    fn new_window(&self) -> gtk::Window {
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

    // Find a window without any tab opened
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
    /// Create a new drawing area in a new tab from this Window
    fn new_draw_area(&self, file: dataview::File, label: &str) -> Result<gtk::DrawingArea> {
        let mut dataviewer = dataviewer::DataViewer::new();
        dataviewer.load(file)?;
        let draw_area = gtk::DrawingArea::from_dataviewer(dataviewer);
        let label = gtk::Label::new(Some(label));
        let notebook = self.get_notebook();
        notebook.append_page(&draw_area, Some(&label));
        draw_area.queue_draw();
        Ok(draw_area)
    }

    /// Create a new drawing area from a FILE in a new tab from this Window
    fn new_draw_area_from_file(&self, path: &PathBuf) -> Result<gtk::DrawingArea> {
        let filename = path.file_name().unwrap().to_string_lossy();
        let string = std::fs::read_to_string(path)?;
        let file = toml::from_str(&string)?;
        self.new_draw_area(file, &filename)
    }

    /// Get Notebook element from this Window
    fn get_notebook(&self) -> gtk::Notebook {
        let widget = self.child().unwrap();
        widget.downcast::<gtk::Notebook>().unwrap()
    }
}

// Get or Set our internal context from a Notebook
impl DrawingAreaDVExt for gtk::DrawingArea {
    fn from_dataviewer(dataviewer: dataviewer::DataViewer) -> Self {
        // Create the Draw Area
        let draw_area = gtk::DrawingArea::new();
        draw_area.set_content_width(128);
        draw_area.set_content_height(128);

        // Set the Draw Area Context
        draw_area.set_context(DrawingAreaContext {
            dataviewer,
        });

        // Notify DataViewer when canvas need to be redraw
        draw_area.set_draw_func(move |draw_area, cairo, width, height| {
            let context = draw_area.get_context();
            println!("Draw area {}x{}", width, height);
            context.dataviewer.draw(draw_area, cairo, width, height);
        });

        // Notify DataViewer when mouse is clicked or released
        let key_ctl = gtk::GestureClick::new();
        let draw_area_ref = draw_area.clone();
        key_ctl.connect_pressed(move |_,_,x,y| {
            let context = draw_area_ref.get_context();
            context.dataviewer.mouse_clicked(x, y);
        });

        let draw_area_ref = draw_area.clone();
        key_ctl.connect_released(move |_,_,_,_| {
            let context = draw_area_ref.get_context();
            context.dataviewer.mouse_released();
        });
        draw_area.add_controller(key_ctl);

        // Notify DataViewer when mouse is moved
        let motion_ctl = gtk::EventControllerMotion::new();
        let draw_area_ref = draw_area.clone();
        motion_ctl.connect_motion(move |_,x,y| {
            let draw_area_ref2 = draw_area_ref.clone();
            let context = draw_area_ref.get_context();
            let timer = glib::source::timeout_add_local_once(
                std::time::Duration::from_millis(50), move ||
            {
                println!("redraw!");
                let context = draw_area_ref2.get_context();
                context.dataviewer.set_redraw_timer(None);
                draw_area_ref2.queue_draw();
            });
            context.dataviewer.set_redraw_timer(Some(timer));
            context.dataviewer.mouse_moved(x, y);
            if context.dataviewer.mouse_is_pressed() {
                draw_area_ref.queue_draw();
            }
        });
        draw_area.add_controller(motion_ctl);

        // Notify DataViewer when mouse is scrolling
        let draw_area_ref = draw_area.clone();
        let scroll_ctl = gtk::EventControllerScroll::new(
            gtk::EventControllerScrollFlags::VERTICAL);
        scroll_ctl.connect_scroll(move |ctl,_,dy| {
            let context = draw_area_ref.get_context();
            context.dataviewer.mouse_scroll(dy);
            ctl.widget().queue_draw();
            glib::signal::Propagation::Proceed
        });
        draw_area.add_controller(scroll_ctl);

        draw_area
    }

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

