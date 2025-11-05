use anyhow::Result;
use egui::{Context, FullOutput, RawInput};
use std::time::Instant;

pub mod drm_backend;
pub mod touch_input;

pub use drm_backend::DrmDevice;
pub use touch_input::{TouchEvent, TouchType, EvdevTouch, TouchSource};

/// Main egui manager for bare-metal DRM/KMS
pub struct EguiManager {
    ctx: Context,
    start_time: Instant,
}

impl EguiManager {
    /// Create a new EguiManager
    pub fn new() -> Self {
        let ctx = Context::default();
        
        // Install image loaders if image feature is enabled
        #[cfg(feature = "image")]
        {
            egui_extras::install_image_loaders(&ctx);
        }
        
        Self {
            ctx,
            start_time: Instant::now(),
        }
    }

    /// Get access to the egui context
    pub fn context(&self) -> &Context {
        &self.ctx
    }

    /// Run the UI and return rendering output
    pub fn run<F>(&self, ui_callback: F, width: u32, height: u32, touch_events: &[TouchEvent]) -> FullOutput 
    where
        F: FnOnce(&Context),
    {
        let raw_input = self.build_raw_input(width, height, touch_events);
        self.ctx.run(raw_input, ui_callback)
    }

    fn build_raw_input(&self, width: u32, height: u32, touch_events: &[TouchEvent]) -> RawInput {
        // Convert touch events to egui pointer states
        let pointers: Vec<egui::PointerState> = touch_events.iter()
            .filter_map(|event| {
                // Only include valid positions
                if event.x >= 0.0 && event.y >= 0.0 {
                    Some(egui::PointerState {
                        pos: egui::Pos2::new(event.x, event.y),
                        pressed: matches!(event.touch_type, TouchType::Press | TouchType::Move),
                        ..Default::default()
                    })
                } else {
                    None
                }
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

    /// Check if egui wants keyboard input
    pub fn wants_keyboard_input(&self) -> bool {
        self.ctx.wants_keyboard_input()
    }

    /// Check if egui wants pointer input  
    pub fn wants_pointer_input(&self) -> bool {
        self.ctx.wants_pointer_input()
    }
}

impl Default for EguiManager {
    fn default() -> Self {
        Self::new()
    }
}
