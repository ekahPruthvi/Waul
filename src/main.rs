use gtk4::{
    glib, prelude::*, Application, ApplicationWindow, CssProvider, GestureClick, Picture,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell, KeyboardMode};
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

struct AppState {
    pictures: HashMap<String, Picture>,
    windows: HashMap<String, ApplicationWindow>,
}

impl AppState {
    fn new() -> Self {
        Self {
            pictures: HashMap::new(),
            windows: HashMap::new(),
        }
    }

    fn apply_walls(&self, walls: &HashMap<String, String>) {
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
) -> (ApplicationWindow, Picture) {
    let window = ApplicationWindow::new(app);

    window.init_layer_shell();
    window.set_layer(Layer::Bottom);
    window.set_monitor(Some(monitor));
    window.set_decorated(false);
    window.set_namespace(Some("cynideWallpaperService"));
    window.set_anchor(Edge::Top,    true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left,   true);
    window.set_anchor(Edge::Right,  true);
    window.set_exclusive_zone(-1);
    window.set_keyboard_mode(KeyboardMode::None);

    let picture = Picture::for_filename(image_path);
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    window.set_child(Some(&picture));

    let gesture = GestureClick::new();
    gesture.set_button(1); 
    gesture.connect_released(|_, _, _, _| {
        if let Err(e) = std::process::Command::new("pkill")
            .args(["-USR1", "capsule"])
            .spawn()
        {
            eprintln!("[waul] Failed to run pkill -USR1 capsule: {}", e);
        }
    });
    window.add_controller(gesture);

    window.present();

    (window, picture)
}

fn snapshot_monitors(display: &Display) -> HashMap<String, gtk4::gdk::Monitor> {
    let mut map = HashMap::new();
    let monitors = display.monitors();
    println!("[waul] snapshot: {} item(s) in monitor list", monitors.n_items());
    for i in 0..monitors.n_items() {
        if let Some(monitor) = monitors.item(i).and_downcast::<gtk4::gdk::Monitor>() {
            let connector = monitor.connector().unwrap_or_default().to_string();
            let valid     = monitor.is_valid();
            println!("[waul]   [{i}] connector={connector:?} valid={valid}");
            if valid && !connector.is_empty() {
                map.insert(connector, monitor);
            }
        }
    }
    map
}

fn reconcile_monitors(
    app: &Application,
    display: &Display,
    state: &mut AppState,
    walls: &HashMap<String, String>,
) {
    let current = snapshot_monitors(display);

    let gone: Vec<String> = state
        .pictures
        .keys()
        .filter(|k| !current.contains_key(*k))
        .cloned()
        .collect();

    for connector in &gone {
        println!("[waul] monitor disconnected: {}", connector);
        if let Some(win) = state.windows.remove(connector) {
            win.close();
        }
        state.pictures.remove(connector);
    }

    for (connector, monitor) in &current {
        if state.pictures.contains_key(connector) {
            continue;
        }

        println!("[waul] monitor connected: {}", connector);

        let path = walls.get(connector).map(|s| s.as_str()).unwrap_or("");
        if path.is_empty() {
            eprintln!("[waul] No wall configured for output '{}'", connector);
        }

        let (window, picture) = build_wallpaper_window(app, monitor, path);
        state.windows.insert(connector.clone(), window);
        state.pictures.insert(connector.clone(), picture);
    }
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

    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut s = state.lock().unwrap();
        reconcile_monitors(app, &display, &mut s, &walls);
    }

    {
        let app_clone     = app.clone();
        let display_clone = display.clone();
        let state_clone   = Arc::clone(&state);
        let monitor_list  = display.monitors();

        monitor_list.connect_items_changed(move |_, pos, removed, added| {
            println!("[waul] monitor list changed (pos={pos} removed={removed} added={added})");

            let app_inner     = app_clone.clone();
            let display_inner = display_clone.clone();
            let state_inner   = Arc::clone(&state_clone);

            let delay_ms = if added > 0 { 200 } else { 0 };

            if delay_ms == 0 {
                glib::idle_add_local_once(move || {
                    let walls_now = parse_walls(PROBE_PATH);
                    let mut s = state_inner.lock().unwrap();
                    reconcile_monitors(&app_inner, &display_inner, &mut s, &walls_now);
                });
            } else {
                glib::timeout_add_local_once(Duration::from_millis(delay_ms), move || {
                    let walls_now = parse_walls(PROBE_PATH);
                    let mut s = state_inner.lock().unwrap();
                    reconcile_monitors(&app_inner, &display_inner, &mut s, &walls_now);
                });
            }
        });

        std::mem::forget(monitor_list);
    }

    let last_modified: Arc<Mutex<Option<SystemTime>>> = Arc::new(Mutex::new(
        fs::metadata(PROBE_PATH).ok().and_then(|m| m.modified().ok()),
    ));

    {
        let state_clone = Arc::clone(&state);

        glib::timeout_add_local(Duration::from_millis(POLL_MS), move || {
            let current_mtime = fs::metadata(PROBE_PATH)
                .ok()
                .and_then(|m| m.modified().ok());

            let mut last = last_modified.lock().unwrap();

            if current_mtime != *last {
                *last = current_mtime;
                println!("[waul] info.probe changed, reloading walls...");
                let new_walls = parse_walls(PROBE_PATH);
                state_clone.lock().unwrap().apply_walls(&new_walls);
            }

            glib::ControlFlow::Continue
        });
    }
}

fn main() {
    let app = Application::new(Some("ekah.scu.waul"), Default::default());
    app.connect_activate(activate);
    app.run();
}