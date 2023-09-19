#![deny(warnings)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

// Don't ask. Remnants of earlier experimentation
use stoppable_thread;

use simple_websockets::{Event, Responder};

use egui_backend::egui::{ecolor, epaint, ColorImage};
use egui_overlay::EguiOverlay;
use egui_render_three_d::ThreeDBackend;
use egui_extras::image::RetainedImage;

fn main()
{
    run_ws_server()
}

fn spawn_egui(message: simple_websockets::Message) {
    stoppable_thread::spawn(move |_stopped| {
        let overlay_image_data = match message {
            simple_websockets::Message::Text(string) => string,
            simple_websockets::Message::Binary(_) => "Expected String - received Binary".to_string(),
        };
        let image: Vec<u8> = image_base64::from_base64(overlay_image_data);
        egui_overlay::start(JobGauge {
            text: "Necromancy Job Gauge".to_string(),
            overlay: image,
        });
    });
}

fn run_ws_server() {

    // Setup WebSocket Server
    // listen for WebSockets on port 8080:
    let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
    // map between client ids and the client's `Responder`:
    let mut clients: HashMap<u64, Responder> = HashMap::new();


    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}", client_id);
                // add their Responder to our `clients` map:
                clients.insert(client_id, responder);
            },
            Event::Disconnect(client_id) => {
                println!("Client #{} disconnected.", client_id);
                // remove the disconnected client from the clients map:
                clients.remove(&client_id);
            },
            Event::Message(client_id, message) => {
                // Uncomment below line for debugging
                // println!("Received a message from client #{}: {:?}", client_id, message);
                // retrieve this client's `Responder`:
                let responder = clients.get(&client_id).unwrap();
                // echo the message back - this prompts the webapp to see if it needs to send a new frame
                responder.send(message.clone());

                // I know not to create this every message like I am doing here.
                // However I don't know how to have it load the image after creation
                // I want to call this in Event::Connect and have it load images received via Event::Message.
                // But I don't know how
                spawn_egui(message);
            },
        }
    }
}

pub struct JobGauge {
    pub text: String,
    pub overlay: Vec<u8>,
}

impl EguiOverlay for JobGauge {
    fn gui_run(
        &mut self,
        egui_context: &egui_backend::egui::Context,
        _three_d_backend: &mut ThreeDBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        glfw_backend.window.maximize();

        // I want this to be set after init of egui. :(
        let image = RetainedImage::from_image_bytes("overlay".to_string(), &self.overlay).expect("Failed to load image");

        egui_backend::egui::Window::new(&self.text).default_pos((200.0, 200.0)).fixed_size((177.0, 112.0)).title_bar(false).collapsible(false).show(egui_context, |ui| {
            glfw_backend.window.set_decorated(false);
            glfw_backend.window.set_opacity(1.0);
            let mut style = (*egui_context.style()).clone();
            style.visuals.window_fill = ecolor::Color32::TRANSPARENT;
            style.visuals.window_shadow = epaint::Shadow::NONE;
            style.visuals.panel_fill = ecolor::Color32::TRANSPARENT;
            style.visuals.window_stroke = epaint::Stroke::NONE;
            egui_context.set_style(style);
            ui.image(image.texture_id(egui_context), image.size_vec2());
            if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
                glfw_backend.window.set_mouse_passthrough(false);
            } else {
                glfw_backend.window.set_mouse_passthrough(true);
            }
        });
    }
}
