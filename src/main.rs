
mod drm_backend;
mod touch_input;
mod egui_integration;

use anyhow::Result;
use drm_backend::DrmDevice;
use touch_input::TouchInput;
use egui_integration::BareMetalEgui;
use std::time::Duration;

fn main() -> Result<()> {
    // Initialize DRM
    let drm = DrmDevice::new()?;
    
    // Initialize touch input (using tslib or direct input)
    let mut touch = TouchInput::new("/dev/input/event0")?;
    
    // Initialize egui
    let mut egui_state = BareMetalEgui::new();
    let mut demo_ui = egui_demo_lib::DemoWindows::default();

    // Main loop
    loop {
        // Handle touch input
        if let Some(event) = touch.wait_for_event(Duration::from_millis(16))? {
            egui_state.handle_touch_event(event.x, event.y, matches!(event.touch_type, TouchType::Press | TouchType::Move));
        }

        // Run UI
        let output = egui_state.run_ui(|ctx| {
            demo_ui.ui(ctx);
        });

        // Render frame
        render_frame(&drm, &output)?;
    }
}

fn render_frame(drm: &DrmDevice, output: &egui::FullOutput) -> Result<()> {
    // Create buffer and render egui
    let buffer = drm.create_buffer(1920, 1080)?;
    
    // Use egui_glow to render to the buffer
    // Then flip the buffer to screen
    
    drm.page_flip(buffer.framebuffer())?;
    Ok(())
}
