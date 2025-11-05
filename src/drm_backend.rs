use anyhow::{Context, Result};
use drm::control::{connector, crtc, dumbbuffer::DumbBuffer, framebuffer, Device, Mode};
use drm_fourcc::{DrmFormat, DrmFourcc};
use gbm::{BufferObject, Device as GbmDevice, Surface as GbmSurface};
use std::os::fd::AsRawFd;

pub struct DrmDevice {
    pub device: drm::Device,
    pub gbm_device: GbmDevice<drm::Device>,
    pub connector: connector::Handle,
    pub crtc: crtc::Handle,
    pub mode: Mode,
    pub surface: GbmSurface<drm::Device>,
}

impl DrmDevice {
    pub fn new() -> Result<Self> {
        // Open first DRM device
        let device = drm::Device::first_card()
            .context("No DRM device found")?
            .context("Failed to open DRM device")?;
        
        let gbm_device = GbmDevice::new(device.clone())
            .context("Failed to create GBM device")?;

        // Find first connected connector
        let resources = device.resource_handles().context("Failed to get DRM resources")?;
        
        let connector = resources.connectors().iter()
            .find_map(|&handle| {
                let connector = device.get_connector(handle).ok()?;
                if connector.state() == connector::State::Connected {
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

        Ok(Self {
            device,
            gbm_device,
            connector,
            crtc,
            mode,
            surface,
        })
    }

    pub fn create_buffer(&self, width: u32, height: u32) -> Result<BufferObject<drm::Device>> {
        self.gbm_device.create_buffer_object(
            width,
            height,
            DrmFourcc::Xrgb8888 as u32,
            gbm::BO_USE_SCANOUT | gbm::BO_USE_RENDERING,
        ).context("Failed to create buffer object")
    }

    pub fn page_flip(&self, fb: framebuffer::Handle) -> Result<()> {
        self.device.page_flip(self.crtc, fb, None)
            .context("Page flip failed")
    }
}
