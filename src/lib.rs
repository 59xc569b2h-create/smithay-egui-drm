pub mod drm_backend;
pub mod touch_input;

pub use drm_backend::DrmDevice;
pub use touch_input::{TouchInput, TouchEvent, TouchType};

use egui::{Context, FullOutput, RawInput};
use std::time::Instant;

/// Основная структура для интеграции egui с bare-metal
pub struct EguiManager {
    ctx: Context,
    start_time: Instant,
}

impl EguiManager {
    pub fn new() -> Self {
        let ctx = Context::default();
        Self {
            ctx,
            start_time: Instant::now(),
        }
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    /// Запуск рендеринга UI
    pub fn run<F>(&self, ui_callback: F, width: u32, height: u32, touch_events: &[TouchEvent]) -> FullOutput 
    where
        F: FnOnce(&Context),
    {
        let raw_input = self.build_raw_input(width, height, touch_events);
        self.ctx.run(raw_input, ui_callback)
    }

    fn build_raw_input(&self, width: u32, height: u32, touch_events: &[TouchEvent]) -> RawInput {
        // Преобразование touch events в egui input
        let pointers = touch_events.iter()
            .map(|event| egui::PointerState {
                pos: egui::Pos2::new(event.x, event.y),
                pressed: matches!(event.touch_type, TouchType::Press | TouchType::Move),
                ..Default::default()
            })
            .collect();

        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(width as f32, height as f32),
            )),
            time: Some(self.start_time.elapsed().as_secs_f64()),
            pointers,
            ..Default::default()
        }
    }
}
