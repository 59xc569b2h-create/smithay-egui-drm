use anyhow::{Context, Result};
use drm::control::{connector, crtc, framebuffer, Device, Mode};
use drm_fourcc::{DrmFormat, DrmFourcc};
use gbm::{BufferObject, Device as GbmDevice, Surface as GbmSurface};
use std::os::fd::AsRawFd;

/// DRM device for bare-metal graphics
pub struct DrmDevice {
    pub device: drm::Device,
    pub gbm_device: GbmDevice<drm::Device>,
    pub connector: connector::Handle,
    pub crtc: crtc::Handle,
    pub mode: Mode,
    pub surface: GbmSurface<drm::Device>,
}

impl DrmDevice {
    /// Create a new DRM device using the first available card
    pub fn new() -> Result<Self> {
        log::info!("Initializing DRM device...");
        
        // Open first DRM device
        let device = drm::Device::first_card()
            .context("No DRM device found")?
            .context("Failed to open DRM device")?;
        
        let gbm_device = GbmDevice::new(device.clone())
            .context("Failed to create GBM device")?;

        // Get DRM resources
        let resources = device.resource_handles().context("Failed to get DRM resources")?;
        
        // Find first connected connector
        let connector = resources.connectors().iter()
            .find_map(|&handle| {
                let connector = device.get_connector(handle).ok()?;
                if connector.state() == connector::State::Connected {
                    log::info!("Found connected connector: {:?}", handle);
                    Some(handle)
                } else {
                    None
                }
            })
            .context("No connected connector found")?;

        // Get connector info and mode
        let connector_info = device.get_connector(connector)
            .context("Failed to get connector info")?;
        let mode = *connector_info.modes().first()
            .context("No mode found for connector")?;

        log::info!("Selected mode: {}x{}", mode.size().0, mode.size().1);

        // Find available CRTC
        let crtc = resources.crtcs().iter()
            .next()
            .copied()
            .context("No CRTC available")?;

        // Create GBM surface
        let surface = gbm_device.create_surface(
            mode.size().0 as u32,
            mode.size().1 as u32,
            DrmFourcc::Xrgb8888 as u32,
            gbm::BO_USE_SCANOUT | gbm::BO_USE_RENDERING,
        ).context("Failed to create GBM surface")?;

        // Set CRTC mode
        device.set_crtc(crtc, None, 0, 0, &[connector], Some(mode))
            .context("Failed to set CRTC mode")?;

        log::info!("DRM device initialized successfully");

        Ok(Self {
            device,
            gbm_device,
            connector,
            crtc,
            mode,
            surface,
        })
    }

    /// Create a new buffer object for rendering
    pub fn create_buffer(&self, width: u32, height: u32) -> Result<BufferObject<drm::Device>> {
        self.gbm_device.create_buffer_object(
            width,
            height,
            DrmFourcc::Xrgb8888 as u32,
            gbm::BO_USE_SCANOUT | gbm::BO_USE_RENDERING,
        ).context("Failed to create buffer object")
    }

    /// Perform a page flip to display the buffer
    pub fn page_flip(&self, fb: framebuffer::Handle) -> Result<()> {
        self.device.page_flip(self.crtc, fb, None)
            .context("Page flip failed")
    }

    /// Get the display size
    pub fn size(&self) -> (u32, u32) {
        (self.mode.size().0 as u32, self.mode.size().1 as u32)
    }
}

impl Drop for DrmDevice {
    fn drop(&mut self) {
        log::debug!("Dropping DRM device");
    }
}
