use chrono::{Local, Timelike};
use gtk::prelude::*;
use gtk::{Box as GtkBox, Image, Orientation, Window, WindowType};
use std::cell::Cell;
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("nixie-clock-popup");
    window.set_decorated(false);

    if let Some(screen) = gtk::prelude::WidgetExt::screen(&window) {
        if let Some(visual) = screen.rgba_visual() {
            window.set_visual(Some(&visual));
        }
    }
    window.set_app_paintable(true);

    let provider = gtk::CssProvider::new();
    provider.load_from_data(b"window { background-color: transparent; }").unwrap();
    gtk::StyleContext::add_provider_for_screen(
        &gtk::gdk::Screen::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let hbox = GtkBox::new(Orientation::Horizontal, 0);
    let asset_dir = format!("{}/.config/waybar/nixie-assets", env::var("HOME").expect("HOME missing"));

    let mut pixbufs = HashMap::new();
    for key in ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "colon"] {
        let file_key = if key == "colon" { "dot" } else { key }; 
        let path = format!("{}/{}.png", asset_dir, file_key);
        let pb = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(&path, -1, 300, true)
            .unwrap_or_else(|_| panic!("Failed to load {}", path));
        pixbufs.insert(key.to_string(), pb);
    }

    let mut images = Vec::new();
    for i in 0..8 {
        let initial = if i == 1 { "colon" } else { "0" };
        let img = Image::from_pixbuf(Some(&pixbufs[initial]));
        hbox.pack_start(&img, false, false, 0);
        images.push(img);
    }
    window.add(&hbox);

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    let r1 = nanos % 3;
    let r2 = (nanos / 10) % 10;
    let r3 = (nanos / 100) % 10;

    let now = Local::now();
    let h1 = now.hour() / 10;
    let h2 = now.hour() % 10;
    let m1 = now.minute() / 10;
    let m2 = now.minute() % 10;

    let final_layout = vec![
        r1.to_string(),
        "colon".to_string(),
        r2.to_string(),
        r3.to_string(),
        h1.to_string(),
        h2.to_string(),
        m1.to_string(),
        m2.to_string(),
    ];

    let tick = Rc::new(Cell::new(0));
    let images_clone = images.clone();
    let pixbufs_clone = pixbufs.clone();
    
    gtk::glib::timeout_add_local(Duration::from_millis(40), move || {
        let current_tick = tick.get();
        
        if current_tick >= 25 {
            for (i, img) in images_clone.iter().enumerate() {
                let key = &final_layout[i];
                img.set_pixbuf(Some(&pixbufs_clone[key]));
            }
            return gtk::glib::ControlFlow::Break;
        }

        let mut seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
        for (i, img) in images_clone.iter().enumerate() {
            if i != 1 {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let d = (seed % 10).to_string();
                img.set_pixbuf(Some(&pixbufs_clone[&d]));
            }
        }

        tick.set(current_tick + 1);
        gtk::glib::ControlFlow::Continue
    });

    window.connect_focus_out_event(|win, _| {
        win.close();
        gtk::glib::Propagation::Proceed
    });

    window.connect_destroy(|_| {
        gtk::main_quit();
    });


    let sound_path = format!("{}/SE.wav", asset_dir);
    let _ = Command::new("paplay").arg(&sound_path).spawn();

    window.show_all();
    gtk::main();
}