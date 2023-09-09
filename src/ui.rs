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
    fn new_open_button(&self) -> gtk::Button;
    fn new_autoview_button(&self) -> gtk::Button;
    fn new_save_button(&self) -> gtk::Button;
    fn new_export_button(&self) -> gtk::Button;
    fn error_str(&self, msg: &str);
    fn error(&self, e: eyre::Error);
    fn set_context(&self, context: WindowContext);
    fn get_context<'a>(&'a self) -> &'a mut WindowContext;
}

/// Extend DataViewer Notebook (tabs) with some utils functions
pub trait DrawingAreaDVExt {
    fn from_dataviewer(dataviewer: dataviewer::DataViewer) -> Self;
    fn set_context(&self, context: DrawingAreaContext);
    fn get_context<'a>(&'a self) -> &'a mut DrawingAreaContext;
}

pub struct WindowContext {}

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
            .build()
            .upcast::<gtk::Window>();

        window.set_context(WindowContext {});

        // Create the title bar
        let titlebar = gtk::HeaderBar::new();
        // Create the notebook (tabs manager)
        let notebook = gtk::Notebook::new();
        window.set_child(Some(&notebook));

        titlebar.pack_start(
            &window.new_open_button());

        titlebar.pack_end(
            &window.new_save_button());

        titlebar.pack_end(
            &window.new_export_button());

        titlebar.pack_end(
            &window.new_autoview_button());

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

    fn new_open_button(&self) -> gtk::Button {
        // Create the Open File button and Dialog
        let buttons = [("Open", gtk::ResponseType::Ok)];
        let dialog = gtk::FileChooserDialog::new(
            Some("Open file to view"),
            Some(self),
            gtk::FileChooserAction::Open,
            &buttons,
            );

        let window = self.clone();
        dialog.connect_response(move |file, response| {
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
            if let Err(e) = window.new_draw_area_from_file(&filename) {
                window.error(e.wrap_err(format!("Failed to open {:?}", filename)));
            }
        });

        let button = gtk::Button::with_label("Open");
        button.connect_clicked(move |_| {
            dialog.present();
        });
        button
    }

    fn new_autoview_button(&self) -> gtk::Button {
        let window = self.clone();
        let button = gtk::Button::with_label("AutoView");
        button.connect_clicked(move |_| {
            let notebook = window.get_notebook();
            let i = match notebook.current_page() {
                Some(i) => i,
                None => {return;},
            };
            let page = match notebook.pages().item(i) {
                Some(page) => page.downcast::<gtk::NotebookPage>().unwrap(),
                None => {return;},
            };
            let draw_area = page.child().downcast::<gtk::DrawingArea>().unwrap();
            let context = draw_area.get_context();
            context.dataviewer.set_autoview(true);
        });
        button
    }

    fn new_save_button(&self) -> gtk::Button {
        // Create the Open File button and Dialog
        let buttons = [
            ("Save", gtk::ResponseType::Ok),
        ];
        let dialog = gtk::FileChooserDialog::new(
            Some("Save"),
            Some(self),
            gtk::FileChooserAction::Save,
            &buttons,
            );
        dialog.set_current_name("dataviewer.dv.toml");

        let window = self.clone();
        dialog.connect_response(move |file, response| {
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
            let notebook = window.get_notebook();
            let i = match notebook.current_page() {
                Some(i) => i,
                None => {return;},
            };
            let page = match notebook.pages().item(i) {
                Some(page) => page.downcast::<gtk::NotebookPage>().unwrap(),
                None => {return;},
            };
            let draw_area = page.child().downcast::<gtk::DrawingArea>().unwrap();
            let context = draw_area.get_context();
            println!("Saving file under {:?}", filename);
            if let Err(e) = context.dataviewer.save_as(&filename) {
                window.error(e.wrap_err("Failed to save image"));
            }
        });

        let button = gtk::Button::with_label("Save");
        button.connect_clicked(move |_| {
            dialog.present();
        });
        button
    }

    fn new_export_button(&self) -> gtk::Button {
        // Create the Open File button and Dialog
        let buttons = [
            ("Export as PNG", gtk::ResponseType::Ok),
        ];
        let dialog = gtk::FileChooserDialog::new(
            Some("Export as PNG"),
            Some(self),
            gtk::FileChooserAction::Save,
            &buttons,
            );
        dialog.set_current_name("dataviewer.png");

        let window = self.clone();
        dialog.connect_response(move |file, response| {
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
            let notebook = window.get_notebook();
            let i = match notebook.current_page() {
                Some(i) => i,
                None => {return;},
            };
            let page = match notebook.pages().item(i) {
                Some(page) => page.downcast::<gtk::NotebookPage>().unwrap(),
                None => {return;},
            };
            let draw_area = page.child().downcast::<gtk::DrawingArea>().unwrap();
            let context = draw_area.get_context();
            println!("Export image under {:?}", filename);
            if let Err(e) = context.dataviewer.export_as_png(&draw_area, &filename) {
                window.error(e.wrap_err("Failed to export image"));
            }
        });

        let button = gtk::Button::with_label("Export PNG");
        button.connect_clicked(move |_| {
            dialog.present();
        });
        button
    }

    fn error_str(&self, msg: &str) {
        let dialog = gtk::MessageDialog::builder()
            .transient_for(self)
            .title("Data Viewer Error")
            .text(msg)
            .message_type(gtk::MessageType::Error)
            .buttons(gtk::ButtonsType::Ok)
            .deletable(true)
            .modal(true)
            .destroy_with_parent(true)
            .default_width(400)
            .default_height(200)
            .visible(true)
            .build();
        dialog.connect_response(|dialog,_| {
            dialog.destroy();
        });
    }

    fn error(&self, e: eyre::Error) {
        let msg = format!("{:?}", e);
        self.error_str(&msg);
    }

    fn set_context(&self, context: WindowContext) {
        unsafe {
            self.set_data::<WindowContext>(ME, context);
        }
    }

    fn get_context<'a>(&'a self) -> &'a mut WindowContext {
        unsafe {
            self.data::<WindowContext>(ME).unwrap().as_mut()
        }
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

