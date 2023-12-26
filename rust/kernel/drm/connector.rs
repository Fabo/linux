use crate::types::Opaque;

/// Connector Types
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum Type {
    Unknown = bindings::DRM_MODE_CONNECTOR_Unknown,
    VGA = bindings::DRM_MODE_CONNECTOR_VGA,
    DVII = bindings::DRM_MODE_CONNECTOR_DVII,
    DVID = bindings::DRM_MODE_CONNECTOR_DVID,
    DVIA = bindings::DRM_MODE_CONNECTOR_DVIA,
    Composite = bindings::DRM_MODE_CONNECTOR_Composite,
    SVIDEO = bindings::DRM_MODE_CONNECTOR_SVIDEO,
    LVDS = bindings::DRM_MODE_CONNECTOR_LVDS,
    Component = bindings::DRM_MODE_CONNECTOR_Component,
    NinePinDIN = bindings::DRM_MODE_CONNECTOR_9PinDIN,
    DisplayPort = bindings::DRM_MODE_CONNECTOR_DisplayPort,
    HDMIA = bindings::DRM_MODE_CONNECTOR_HDMIA,
    HDMIB = bindings::DRM_MODE_CONNECTOR_HDMIB,
    TV = bindings::DRM_MODE_CONNECTOR_TV,
    eDP = bindings::DRM_MODE_CONNECTOR_eDP,
    VIRTUAL = bindings::DRM_MODE_CONNECTOR_VIRTUAL,
    DSI = bindings::DRM_MODE_CONNECTOR_DSI,
    DPI = bindings::DRM_MODE_CONNECTOR_DPI,
    WRITEBACK = bindings::DRM_MODE_CONNECTOR_WRITEBACK,
    SPI = bindings::DRM_MODE_CONNECTOR_SPI,
    USB = bindings::DRM_MODE_CONNECTOR_USB,
}

#[repr(transparent)]
pub struct Connector(Opaque<bindings::drm_connector>);

impl Connector {
    pub unsafe fn as_mut<'a>(ptr: *mut bindings::drm_connector) -> &'a mut Self {
        unsafe { &mut *ptr.cast() }
    }

    pub unsafe fn drm_device(&self) -> *mut bindings::drm_device {
        unsafe { (*self.0.get()).dev }
    }

    pub(crate) fn as_raw(&self) -> *mut bindings::drm_connector {
        self.0.get()
    }

    pub unsafe fn display_info_mut(&mut self) -> &mut bindings::drm_display_info {
        unsafe { &mut (*self.0.get()).display_info }
    }
}
