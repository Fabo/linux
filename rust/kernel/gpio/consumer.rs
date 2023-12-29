use crate::{
    device::Device,
    error::{code::*, from_err_ptr, Result},
    str::CStr,
};

pub mod flags {
    pub const ASIS: u32 = bindings::gpiod_flags_GPIOD_ASIS as _;
    pub const IN: u32 = bindings::gpiod_flags_GPIOD_IN as _;
    pub const OUT_LOW: u32 = bindings::gpiod_flags_GPIOD_OUT_LOW as _;
    pub const OUT_HIGH: u32 = bindings::gpiod_flags_GPIOD_OUT_HIGH as _;
    pub const OUT_LOW_OPEN_DRAIN: u32 = bindings::gpiod_flags_GPIOD_OUT_LOW_OPEN_DRAIN as _;
    pub const OUT_HIGH_OPEN_DRAIN: u32 = bindings::gpiod_flags_GPIOD_OUT_HIGH_OPEN_DRAIN as _;
}

pub struct Desc(*mut bindings::gpio_desc);

impl Desc {
    pub fn get<D: AsRef<Device>>(dev: D, con_id: &'static CStr, flags: u32) -> Result<Self> {
        let desc = from_err_ptr(unsafe {
            bindings::gpiod_get(dev.as_ref().as_raw(), con_id.as_char_ptr(), flags)
        })?;

        if desc.is_null() {
            return Err(EINVAL);
        }

        Ok(Self(desc))
    }

    pub fn get_opt<D: AsRef<Device>>(
        dev: D,
        con_id: &'static CStr,
        flags: u32,
    ) -> Result<Option<Self>> {
        let desc = from_err_ptr(unsafe {
            bindings::gpiod_get_optional(dev.as_ref().as_raw(), con_id.as_char_ptr(), flags as _)
        })?;

        Ok(if desc.is_null() {
            None
        } else {
            Some(Self(desc))
        })
    }

    pub fn set_value(&mut self, value: i32) {
        unsafe { bindings::gpiod_set_value(self.0, value) }
    }
}

impl Drop for Desc {
    fn drop(&mut self) {
        unsafe { bindings::gpiod_put(self.0) }
    }
}

unsafe impl Send for Desc {}
