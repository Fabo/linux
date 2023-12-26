use crate::{
    bindings,
    device::Device,
    drm::connector::{Connector, Type},
    error::{code::*, from_result, to_result, Result},
    macros::vtable,
    prelude::*,
    types::ForeignOwnable,
};
use core::{marker::PhantomData, mem::MaybeUninit, ops::DerefMut};

pub struct Panel(KBox<bindings::drm_panel>);

impl Panel {
    pub fn new<T: Operations>(parent: impl AsRef<Device>, connector_type: Type) -> Result<Self> {
        let this = MaybeUninit::<bindings::drm_panel>::zeroed();
        let this = unsafe { this.assume_init() };
        let ops = unsafe { OperationsVtable::<T>::build() };
        let mut this = Self(KBox::new(this, GFP_KERNEL)?);

        unsafe {
            bindings::drm_panel_init(
                (&mut this.0).deref_mut(),
                parent.as_ref().as_raw(),
                ops,
                connector_type as _,
            )
        };

        Ok(this)
    }

    // FIXME: check that the correct backlight class is enabled, otherwise drm_panel_of_backlight
    // becomes a C static inline function
    pub fn init_of_backlight(&mut self) -> Result {
        to_result(unsafe { bindings::drm_panel_of_backlight((&mut self.0).deref_mut()) })
    }

    pub fn prepare_prev_first(&mut self, val: bool) {
        self.0.prepare_prev_first = val;
    }

    pub fn register(mut self) -> PanelRegistration {
        unsafe { bindings::drm_panel_add((&mut self.0).deref_mut()) };
        PanelRegistration(self)
    }
}

pub struct PanelRegistration(Panel);

impl Drop for PanelRegistration {
    fn drop(&mut self) {
        unsafe { bindings::drm_panel_remove((&mut self.0 .0).deref_mut()) };
    }
}

unsafe impl Send for PanelRegistration {}

#[vtable]
pub trait Operations {
    type Data: ForeignOwnable + Send + Sync = ();

    fn prepare(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>) -> Result {
        Err(ENOTSUPP)
    }

    fn unprepare(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>) -> Result {
        Err(ENOTSUPP)
    }

    fn get_modes(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _connector: &mut Connector,
    ) -> Result<u16> {
        Err(ENOTSUPP)
    }
}

pub(crate) struct OperationsVtable<T>(PhantomData<T>);

impl<T: Operations> OperationsVtable<T> {
    const VTABLE: bindings::drm_panel_funcs = bindings::drm_panel_funcs {
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
        get_modes: if T::HAS_GET_MODES {
            Some(Self::get_modes_callback)
        } else {
            None
        },
    };

    pub(crate) const unsafe fn build() -> &'static bindings::drm_panel_funcs {
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
            let data = unsafe {
                let data = Self::get_drvdata(panel)?;
                T::Data::borrow(data)
            };
            T::prepare(data)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn unprepare_callback(panel: *mut bindings::drm_panel) -> core::ffi::c_int {
        from_result(|| {
            let data = unsafe {
                let data = Self::get_drvdata(panel)?;
                T::Data::borrow(data)
            };
            T::unprepare(data)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_modes_callback(
        panel: *mut bindings::drm_panel,
        connector: *mut bindings::drm_connector,
    ) -> core::ffi::c_int {
        from_result(|| {
            let data = unsafe {
                let data = Self::get_drvdata(panel)?;
                T::Data::borrow(data)
            };
            T::get_modes(data, &mut Connector::from_raw(connector)).map(|v| v as i32)
        })
    }
}
