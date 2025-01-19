use crate::{
    bindings,
    device::Device,
    drm::connector::{Connector, Type},
    error::{code::*, from_result, to_result, Result},
    prelude::*,
    types::{ForeignOwnable, Opaque},
};
use core::marker::PhantomData;

/// DRM Panel.
///
/// # Invariants
#[repr(transparent)]
#[pin_data]
pub struct Panel {
    #[pin]
    panel: Opaque<bindings::drm_panel>,
}

impl Panel {
    /// Create a new DRM panel.
    pub fn new<T: Operations>(
        parent: impl AsRef<Device>,
        connector_type: Type,
    ) -> impl PinInit<Self, Error> {
        let ops = OperationsVtable::<T>::build();

        try_pin_init!(Self {
            panel <- Opaque::try_ffi_init(|ptr: *mut bindings::drm_panel| {
                // SAFETY: `try_ffi_init` guarantees that `ptr` is valid for write.
                unsafe { ptr.write(bindings::drm_panel::default()) };

                unsafe {
                    bindings::drm_panel_init(
                        ptr,
                        parent.as_ref().as_raw(),
                        ops,
                        connector_type as _,
                    )
                };

                Ok::<(), Error>(())
            }),
        })
    }

    /// Enable backlight via device node.
    #[cfg(CONFIG_BACKLIGHT_CLASS_DEVICE = "y")]
    pub fn init_of_backlight(&mut self) -> Result {
        to_result(unsafe { bindings::drm_panel_of_backlight(self.panel.get()) })
    }

    /// Prepare previous controller first.
    pub fn prepare_prev_first(&mut self, val: bool) {
        unsafe {
            (*self.panel.get()).prepare_prev_first = val;
        };
    }

    /// Register panel.
    pub fn register(self) -> Registration {
        unsafe { bindings::drm_panel_add(self.panel.get()) };
        Registration(self)
    }

    /// Unregister panel.
    fn unregister(&mut self) {
        unsafe { bindings::drm_panel_remove(self.panel.get()) };
    }
}

/// Panel registration.
pub struct Registration(Panel);

impl Drop for Registration {
    fn drop(&mut self) {
        self.0.unregister();
    }
}

unsafe impl Send for Registration {}

/// [`Panel`]'s operations
#[vtable]
pub trait Operations {
    /// User data that will be accessible to all operations.
    type Data: ForeignOwnable + Send + Sync;

    /// Turn on panel and perform set up.
    fn prepare(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>) -> Result {
        Err(ENOTSUPP)
    }

    /// Turn off panel.
    fn unprepare(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>) -> Result {
        Err(ENOTSUPP)
    }

    /// Add modes to the connector
    fn get_modes(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _connector: &mut Connector,
    ) -> Result<u16>;
}

struct OperationsVtable<T>(PhantomData<T>);

impl<T: Operations> OperationsVtable<T> {
    const VTABLE: bindings::drm_panel_funcs = bindings::drm_panel_funcs {
        get_modes: Some(Self::get_modes_callback),
        enable: None,
        disable: None,
        get_orientation: None,
        get_timings: None,
        debugfs_init: None,
        prepare: if T::HAS_PREPARE {
            Some(Self::prepare_callback)
        } else {
            None
        },
        unprepare: if T::HAS_UNPREPARE {
            Some(Self::unprepare_callback)
        } else {
            None
        },
    };

    const fn build() -> &'static bindings::drm_panel_funcs {
        &Self::VTABLE
    }

    unsafe fn get_drvdata(panel: *mut bindings::drm_panel) -> Result<*mut core::ffi::c_void> {
        let data = unsafe { bindings::dev_get_drvdata((*panel).dev) };
        if data.is_null() {
            Err(EINVAL)
        } else {
            Ok(data)
        }
    }

    unsafe extern "C" fn prepare_callback(panel: *mut bindings::drm_panel) -> core::ffi::c_int {
        from_result(|| {
            let data = unsafe { T::Data::borrow(Self::get_drvdata(panel)?) };
            T::prepare(data)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn unprepare_callback(panel: *mut bindings::drm_panel) -> core::ffi::c_int {
        from_result(|| {
            let data = unsafe { T::Data::borrow(Self::get_drvdata(panel)?) };
            T::unprepare(data)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_modes_callback(
        panel: *mut bindings::drm_panel,
        connector: *mut bindings::drm_connector,
    ) -> core::ffi::c_int {
        from_result(|| {
            let data = unsafe { T::Data::borrow(Self::get_drvdata(panel)?) };
            let connector = unsafe { Connector::as_mut(connector) };
            T::get_modes(data, connector).map(|v| v as i32)
        })
    }
}
