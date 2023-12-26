// SPDX-License-Identifier: GPL-2.0

//! DRM Mode definitions
//!
//! C header: [`include/drm/drm_modes.h`](srctree/include/drm/drm_modes.h)
//! C header: [`include/uapi/drm/drm_mode.h`](srctree/include/uapi/drm/drm_mode.h)

use crate::{
    bindings,
    drm::connector,
    error::{code::*, Result},
};

pub type DisplayModeData = bindings::drm_display_mode;

/// Display Mode
pub struct DisplayMode(*mut bindings::drm_display_mode);

impl DisplayMode {
    pub fn duplicate(dev: *mut bindings::drm_device, mode: &DisplayModeData) -> Result<Self> {
        let new_mode = unsafe { bindings::drm_mode_duplicate(dev, mode) };
        if new_mode.is_null() {
            return Err(ENOMEM);
        }

        Ok(Self(new_mode))
    }

    pub fn set_name(&mut self) {
        unsafe { bindings::drm_mode_set_name(self.0) }
    }

    // FIXME
    // Marked as unsafe because drm_mode_probed_add is attaching the mode to its
    // `probed_modes` linked list, and according to the C doc, this list should
    // be protected using drm_mode_config.mutex.
    pub unsafe fn probed_add(self, connector: &mut connector::Connector) {
        // FIXME: this function cannot be called on an instance
        unsafe { bindings::drm_mode_probed_add(connector.raw_mut(), self.0) }

        // Mode ownership has now been transfered to the connector. When the connector
        // is being destroyed the mode memory will be released.
        core::mem::forget(self);
    }
}

impl Drop for DisplayMode {
    fn drop(&mut self) {
        // FIXME: should use drm_mode_destroy() instead
        unsafe { bindings::kfree(self.0 as _) };
    }
}
