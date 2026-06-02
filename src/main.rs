use gtk4::{
    glib, prelude::*, Application, ApplicationWindow, Box as GtkBox, Box, Button, CssProvider, Image, Label, Orientation
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use gtk4::gdk::Display;
use rand::prelude::IndexedRandom;
use glib::timeout_add_local;
use rand::rngs::ThreadRng;
use std::{fs, time::Duration, process};
use std::rc::Rc;
use std::cell::Cell;
use std::fs::File;



fn activate(app: &Application) {
    
    let css = CssProvider::new();
    css.load_from_data(
        "
        #mainshadow {
            background-color:rgba(0, 0, 0, 0);
        }

        #main {
            background-color:rgba(0, 0, 0, 0);

            animation: pulse 1.5s infinite ease-in-out;
        }

        .error_box {
            border-radius: 20px;
            padding: 5px;
            background-color: rgba(34, 34, 34, 0.559);
            border: 2px solid transparent;
            background-image: linear-gradient(rgb(29, 29, 29), rgb(29, 29, 29)),
                                linear-gradient(0deg, rgb(9, 9, 9), rgba(94, 94, 94, 0.686));
            background-origin: border-box;
            background-clip: padding-box, border-box;
            box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
        }

        #icon_circle {
            background-color: rgba(40, 40, 40, 0);
            border-radius: 50%;
            padding: 5px;
            transition: transform 0.2s;

            animation: shake 0.5s infinite ease-in-out;
        }

        @keyframes pulse {
            0% {
                box-shadow: inset 0 0 0px rgba(255, 0, 102, 0.6);
            }
            50% {
                box-shadow: inset 0 -30px 30px -20px rgba(255, 25, 25, 0.8);
            }
            100% {
                box-shadow: inset 0 0 0px rgba(255, 0, 102, 0.6);
            }
        }

        @keyframes shake {
            0% {
                transform: rotate(-5deg);
            }
            100% {
                transform: rotate(5deg);
            }    
        }


        #title_label {
            font-weight:400;
            font-size: 12px;
            color: rgba(255, 255, 255, 0.71);
        }

        #subtitle_label {
            color: #cccccc72;
            font-size: 12px;
        }

        .ok_button {
            all: unset;
            min-height: 20px;
            min-width: 20px;
            background-color: rgba(251, 251, 251, 0.08);
            color: rgba(198, 198, 198, 0);
            font-size: 1px;
            border-radius: 50px;
            padding: 0px;
            transition: all 300ms ease;
        }
        

        .ok_button:hover {
            background-color:rgba(255, 230, 0, 0.92);
            color: rgb(198, 198, 198);
            font-style: italic;
            font-size: 12px;
            font-weight: 300;
        }

        .ok_button_bye {
            all: unset;
            min-height: 20px;
            min-width: 20px;
            background-color: rgba(251, 251, 251, 0.08);
            color: rgba(198, 198, 198, 0);
            font-size: 1px;
            border-radius: 50px;
            padding: 0px;
            transition: all 300ms ease;
        }
        
        .ok_button_bye:hover {
            background-color:rgba(255, 85, 116, 0.55);
            color: rgb(198, 198, 198);
            font-style: italic;
            font-size: 12px;
            font-weight: 300;
        }

        #shadow {
            color: rgba(255, 0, 0, 0);
            box-shadow: rgba(0, 0, 0, 0.25) 0px 54px 55px, rgba(0, 0, 0, 0.12) 0px -12px 30px, rgba(0, 0, 0, 0.12) 0px 4px 6px, rgba(0, 0, 0, 0.17) 0px 12px 13px, rgba(0, 0, 0, 0.09) 0px -3px 5px;
            border-radius: 25px;
        }

        .osd-hide {
            animation: osd-disappear 0.3s ease-in forwards;
        }

        @keyframes osd-disappear {
            from {
                opacity: 1;
                transform: translateY(0) scale(1);
            }
            to {
                opacity: 0;
                transform: translateY(10px) scale(0.95);
            }
        }
    ",
    );

    gtk4::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::new(app);
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.fullscreen();
    window.set_decorated(false);
    window.set_namespace(Some("cynideWallpaperService"));
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Top, true);


    // window.set_child(Some(&batt));

    window.show();
}

fn main() {
    // Start reading from here dumbass

    let app = Application::new(Some("ekah.scu.waul"), Default::default());
    app.connect_activate(activate);

    app.run();

}
