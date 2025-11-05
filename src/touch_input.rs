use anyhow::{Context, Result};
use nix::poll::{poll, PollFd, PollFlags};
use std::fs::File;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::time::Duration;

/// Touch event types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchType {
    Press,
    Release,
    Move,
}

/// Touch event data
#[derive(Debug, Clone)]
pub struct TouchEvent {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    pub touch_type: TouchType,
}

/// Trait for touch input sources
pub trait TouchSource: Send {
    /// Read available touch events
    fn read_events(&mut self) -> Result<Vec<TouchEvent>>;
    
    /// Wait for events with timeout
    fn wait_for_events(&mut self, timeout: Duration) -> Result<Vec<TouchEvent>>;
}

/// Evdev-based touch input (without tslib)
pub struct EvdevTouch {
    device: File,
}

impl EvdevTouch {
    /// Create a new EvdevTouch from device path
    pub fn new(device_path: &str) -> Result<Self> {
        let device = File::open(device_path)
            .context(format!("Failed to open touch device: {}", device_path))?;
        
        log::info!("Opened touch device: {}", device_path);
        Ok(Self { device })
    }
}

impl TouchSource for EvdevTouch {
    fn read_events(&mut self) -> Result<Vec<TouchEvent>> {
        let mut events = Vec::new();
        
        // Non-blocking read of available events
        while let Some(event) = self.read_single_event()? {
            events.push(event);
        }
        
        Ok(events)
    }

    fn wait_for_events(&mut self, timeout: Duration) -> Result<Vec<TouchEvent>> {
        let fd = PollFd::new(self.device.as_raw_fd(), PollFlags::POLLIN);
        let timeout_ms = timeout.as_millis() as i32;
        
        match poll(&mut [fd], timeout_ms)? {
            0 => Ok(Vec::new()), // Timeout
            _ => self.read_events(),
        }
    }
}

impl EvdevTouch {
    fn read_single_event(&mut self) -> Result<Option<TouchEvent>> {
        let mut event: InputEvent = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<InputEvent>();
        
        let bytes_read = unsafe {
            let buffer = std::slice::from_raw_parts_mut(
                &mut event as *mut _ as *mut u8,
                size,
            );
            self.device.read(buffer)
        };
        
        match bytes_read {
            Ok(read) if read == size => {
                // Parse the input event
                self.parse_input_event(&event)
            }
            Ok(_) => Ok(None), // Incomplete read
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    fn parse_input_event(&self, event: &InputEvent) -> Result<Option<TouchEvent>> {
        // Simplified evdev parsing - in real implementation you'd track state
        // across multiple events (ABS_X, ABS_Y, BTN_TOUCH, etc.)
        
        // This is a placeholder - real implementation would be more complex
        // and maintain state between events
        
        log::debug!("Input event: type={}, code={}, value={}", event.type_, event.code, event.value);
        
        Ok(None)
    }
}

/// Linux input event structure
#[repr(C)]
#[derive(Debug)]
struct InputEvent {
    time: libc::timeval,
    type_: u16,
    code: u16,
    value: i32,
}

/// Simple mock touch source for testing
#[cfg(feature = "test")]
pub struct MockTouch {
    events: Vec<TouchEvent>,
}

#[cfg(feature = "test")] 
impl MockTouch {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
    
    pub fn add_event(&mut self, event: TouchEvent) {
        self.events.push(event);
    }
}

#[cfg(feature = "test")]
impl TouchSource for MockTouch {
    fn read_events(&mut self) -> Result<Vec<TouchEvent>> {
        Ok(self.events.drain(..).collect())
    }
    
    fn wait_for_events(&mut self, _timeout: Duration) -> Result<Vec<TouchEvent>> {
        self.read_events()
    }
}
