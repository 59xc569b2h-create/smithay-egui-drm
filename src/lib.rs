use anyhow::Result;
use egui::{Context, FullOutput, RawInput};
use std::time::Instant;

pub struct BareMetalEgui {
    ctx: Context,
    start_time: Instant,
    touch_x: f32,
    touch_y: f32,
    touch_pressed: bool,
}

impl BareMetalEgui {
    pub fn new() -> Self {
        let ctx = Context::default();
        Self {
            ctx,
            start_time: Instant::now(),
            touch_x: 0.0,
            touch_y: 0.0,
            touch_pressed: false,
        }
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    pub fn handle_touch_event(&mut self, x: f32, y: f32, pressed: bool) {
        self.touch_x = x;
        self.touch_y = y;
        self.touch_pressed = pressed;
    }

    pub fn run_ui<F>(&mut self, ui_callback: F) -> FullOutput 
    where
        F: FnOnce(&Context),
    {
        let raw_input = RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1920.0, 1080.0), // Your screen size
            )),
            time: Some(self.start_time.elapsed().as_secs_f64()),
            pointers: if self.touch_pressed {
                vec![egui::PointerState {
                    pos: egui::Pos2::new(self.touch_x, self.touch_y),
                    pressed: self.touch_pressed,
                    ..Default::default()
                }]
            } else {
                vec![]
            },
            ..Default::default()
        };

        self.ctx.run(raw_input, ui_callback)
    }
}
