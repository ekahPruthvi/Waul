use gtk4::{
    glib, prelude::*, Application, ApplicationWindow, CssProvider, Picture,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use gtk4::gdk::Display;
use std::{fs, time::Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

const PROBE_PATH: &str = "/var/lib/cynager/info.probe";
const POLL_MS: u64 = 500;

fn parse_walls(probe_path: &str) -> HashMap<String, String> {
    let mut walls: HashMap<String, String> = HashMap::new();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());

    let content = match fs::read_to_string(probe_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[waul] Failed to read {}: {}", probe_path, e);
            return walls;
        }
    };

    let mut in_set = false;
    let mut in_walls = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == ":set"           { in_set = true;  continue; }
        if in_set && trimmed == ":end" { in_set = false; in_walls = false; continue; }
        if in_set && trimmed.starts_with("walls") { in_walls = true; continue; }

        if in_walls {
            if trimmed == "}" { in_walls = false; continue; }

            if let Some(colon_pos) = trimmed.rfind(':') {
                let rel_path = trimmed[..colon_pos].trim();
                let output   = trimmed[colon_pos + 1..].trim().to_string();

                let abs_path = if rel_path.starts_with('/') {
                    rel_path.to_string()
                } else {
                    format!("{}/{}", home, rel_path)
                };

                if !abs_path.is_empty() && !output.is_empty() {
                    walls.insert(output, abs_path);
                }
            }
        }
    }

    walls
}

struct WallState {
    pictures: HashMap<String, Picture>,
}

impl WallState {
    fn apply(&self, walls: &HashMap<String, String>) {
        for (connector, picture) in &self.pictures {
            if let Some(path) = walls.get(connector) {
                let file = gtk4::gio::File::for_path(path);
                picture.set_file(Some(&file));
                println!("[waul] updated {} => {}", connector, path);
            }
        }
    }
}

fn build_wallpaper_window(
    app: &Application,
    monitor: &gtk4::gdk::Monitor,
    image_path: &str,
) -> Picture {
    let window = ApplicationWindow::new(app);

    window.init_layer_shell();
    window.set_layer(Layer::Background);
    window.set_monitor(Some(monitor));
    window.set_decorated(false);
    window.set_namespace(Some("cynideWallpaperService"));
    window.set_anchor(Edge::Top,    true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left,   true);
    window.set_anchor(Edge::Right,  true);
    window.set_exclusive_zone(-1);

    let picture = Picture::for_filename(image_path);
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    window.set_child(Some(&picture));
    window.present();

    picture
}

fn activate(app: &Application) {
    let css = CssProvider::new();
    css.load_from_data(
        "
            window  { 
                background-color: #000000;
            }
            picture { 
                background-color: #000000;
                background-image: radial-gradient(rgba(255, 255, 255, 0.06) 2px, transparent 0);
                background-size: 30px 30px;
                background-position: -5px -5px;
            }
        ",
    );
    gtk4::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let walls = parse_walls(PROBE_PATH);

    let display = Display::default().expect("Could not get default display");
    let monitors = display.monitors();

    let mut pictures: HashMap<String, Picture> = HashMap::new();

    for i in 0..monitors.n_items() {
        let monitor = monitors
            .item(i)
            .and_downcast::<gtk4::gdk::Monitor>()
            .expect("Expected Monitor object");

        let connector = monitor.connector().unwrap_or_default();
        let connector_str = connector.as_str();

        let path = walls.get(connector_str).map(|s| s.as_str()).unwrap_or("");
        if path.is_empty() {
            eprintln!("[waul] No wall configured for output '{}'", connector_str);
        }

        let picture = build_wallpaper_window(app, &monitor, path);
        pictures.insert(connector_str.to_string(), picture);
    }

    let state = Arc::new(Mutex::new(WallState { pictures }));

    let last_modified: Arc<Mutex<Option<SystemTime>>> = Arc::new(Mutex::new(
        fs::metadata(PROBE_PATH).ok().and_then(|m| m.modified().ok()),
    ));

    glib::timeout_add_local(Duration::from_millis(POLL_MS), move || {
        let current_mtime = fs::metadata(PROBE_PATH)
            .ok()
            .and_then(|m| m.modified().ok());

        let mut last = last_modified.lock().unwrap();

        if current_mtime != *last {
            *last = current_mtime;
            println!("[waul] info.probe changed, reloading walls...");
            let new_walls = parse_walls(PROBE_PATH);
            state.lock().unwrap().apply(&new_walls);
        }

        glib::ControlFlow::Continue
    });
}

fn main() {
    let app = Application::new(Some("ekah.scu.waul"), Default::default());
    app.connect_activate(activate);
    app.run();
}