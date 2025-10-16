use std::num::NonZero;

use ab_glyph::{Font, FontArc, Glyph, PxScale, point};
use fontdb::Database;
use softbuffer::{Context, Surface};
use tiny_skia::{Color, Paint, Pixmap, Transform};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::keymap_c_parser::Layer;

mod keymap_c_parser;

const KEY_SPACING: f32 = 50.0;
const KEY_WIDTH: f32 = 47.0;

fn main() {
    let folder = "/qmk_firmware/keyboards/macro";
    let home = std::env::var("HOME").unwrap();
    let path = format!("{}/{}", home, folder);
    let keyboard_json = format!("{}/keyboard.json", path);
    let keymap_c = format!("{}/keymaps/macro/keymap.c", path);

    let json_string =
        std::fs::read_to_string(&keyboard_json).expect("Failed to read keyboard.json");
    let mut keyboard = json::parse(&json_string).expect("Failed to parse JSON");

    let mut key_positions = keyboard
        .remove("layouts")
        .remove("LAYOUT_40_macro")
        .remove("layout")
        .members()
        .map(|k| {
            let x = k["x"].as_f32().unwrap();
            let y = k["y"].as_f32().unwrap();
            KeyPosition { x, y }
        })
        .collect::<Vec<_>>();

    //get bounding box
    let min_x = key_positions
        .iter()
        .map(|k| k.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = key_positions
        .iter()
        .map(|k| k.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = key_positions
        .iter()
        .map(|k| k.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = key_positions
        .iter()
        .map(|k| k.y)
        .fold(f32::NEG_INFINITY, f32::max);

    println!(
        "min_x: {}, min_y: {}, max_x: {}, max_y: {}",
        min_x, min_y, max_x, max_y
    );

    for key in &mut key_positions {
        key.x -= min_x;
        key.y -= min_y;
    }

    let width = (max_x - min_x + 1.0) * KEY_SPACING;
    let height = (max_y - min_y + 1.0) * KEY_SPACING;

    let keymap = keymap_c_parser::parse_c_source(&keymap_c);

    render_main(key_positions, keymap, (width as usize, height as usize));
}

fn render_main(key_positions: Vec<KeyPosition>, layers: Vec<Layer>, size: (usize, usize)) {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let font = load_font();

    let mut app = App {
        key_positions,
        size,
        layers,
        font,
        window: None,
        current_layer: 0,
    };
    let _ = event_loop.run_app(&mut app);
}

struct App {
    size: (usize, usize),
    window: Option<Window>,
    key_positions: Vec<KeyPosition>,
    layers: Vec<Layer>,
    current_layer: usize,
    font: FontArc,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("size: {:?}", self.size);
        let mut attrs = Window::default_attributes();
        attrs = attrs.with_resizable(false);
        attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(
            self.size.0 as u32,
            self.size.1 as u32,
        ));
        self.window = Some(event_loop.create_window(attrs).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();

                let size = window.inner_size();
                let width = size.width;
                let height = size.height;

                let context = Context::new(window).unwrap();
                let mut surface = Surface::new(&context, window).unwrap();
                surface
                    .resize(NonZero::new(width).unwrap(), NonZero::new(height).unwrap())
                    .unwrap();

                let mut pixmap = Pixmap::new(width, height).unwrap();
                let mut paint = Paint::default();

                paint.set_color_rgba8(100, 100, 100, 255);
                pixmap.fill(Color::from_rgba8(30, 30, 30, 255));
                let key_scale = PxScale::from(30.0);

                for key in self.key_positions.iter().zip(self.layers[self.current_layer].keys.iter()) {
                    let rect = tiny_skia::Rect::from_xywh(
                        key.0.x * KEY_SPACING,
                        key.0.y * KEY_SPACING,
                        KEY_WIDTH,
                        KEY_WIDTH,
                    )
                    .unwrap();
                    pixmap.fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
                    key.1.render(key.0.x * KEY_SPACING, key.0.y * KEY_SPACING, &mut pixmap, &self.font.clone(), key_scale, Color::from_rgba8(255, 255, 255, 255));
                }

                let layer_scale = PxScale::from(20.0);

                let text = &self.layers[self.current_layer].name;
                draw_text(
                    &mut pixmap,
                    text,
                    &self.font,
                    layer_scale,
                    320.0,
                    80.0,
                    Color::from_rgba8(255, 255, 255, 255),
                );

                // Copy pixmap data to window surface
                let mut buffer = surface.buffer_mut().unwrap();
                let buf = buffer.as_mut();
                let data = pixmap
                    .data()
                    .chunks_exact(4)
                    .map(|px| {
                        ((px[3] as u32) << 24)
                            | ((px[2] as u32) << 16)
                            | ((px[1] as u32) << 8)
                            | (px[0] as u32)
                    })
                    .collect::<Vec<_>>();
                for (dst, src) in buf.chunks_exact_mut(4).zip(data.chunks_exact(4)) {
                    dst.copy_from_slice(src);
                }
                buffer.present().unwrap();

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

pub fn draw_text(
    pixmap: &mut Pixmap,
    text: &str,
    font: &FontArc,
    scale: PxScale,
    x: f32,
    y: f32,
    color: Color,
) {
    let mut transform = Transform::identity();
    transform = transform.pre_translate(x, y);
    let mut paint = Paint::default();
    paint.set_color(color);

    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        let glyph = Glyph {
            id: glyph_id,
            scale,
            position: point(0.0, 0.0),
        };
        let advance = font.h_advance_unscaled(glyph_id);
        let advance_scaled = advance * scale.x / font.units_per_em().unwrap_or(1.0);

        let glyph = font.outline_glyph(glyph);
        let Some(glyph) = glyph else {
            transform = transform.pre_translate(advance_scaled, 0.0);
            continue;
        };

        glyph.draw(|px, py, c| {
            let alpha = (c * 255.0) as u8;
            let color = Color::from_rgba8(255, 255, 255, alpha);
            paint.set_color(color);
            let x = px as f32
                + font.h_side_bearing_unscaled(glyph_id) * scale.x
                    / font.units_per_em().unwrap_or(1.0);
            let y = py as f32;
            let rect = tiny_skia::Rect::from_xywh(x, y, 1.0, 1.0).unwrap();
            pixmap.fill_rect(rect, &paint, transform, None);
        });
        let advance = font.h_advance_unscaled(glyph_id);
        let advance_scaled = advance * scale.x / font.units_per_em().unwrap_or(1.0);
        transform = transform.pre_translate(advance_scaled, 0.0);
    }
}

fn load_font() -> FontArc {
    // Create a font database and load system fonts
    let mut db = Database::new();
    db.load_system_fonts();

    // Pick the first sans-serif font, or fallback
    if let Some(id) = db.query(&fontdb::Query {
        families: &[fontdb::Family::SansSerif],
        ..Default::default()
    }) {
        let (font, _id) = db.face_source(id).unwrap();
        match font {
            fontdb::Source::Binary(data) => {
                FontArc::try_from_vec(data.as_ref().as_ref().to_vec()).unwrap()
            }
            fontdb::Source::File(path) => {
                FontArc::try_from_vec(std::fs::read(path).unwrap()).unwrap()
            }
            _ => panic!("Unsupported font source"),
        }
    } else {
        panic!("No system fonts found!");
    }
}

#[derive(Debug)]
struct KeyPosition {
    x: f32,
    y: f32,
}
