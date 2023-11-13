// SPDX-License-Identifier: GPL-2.0

//! SoC Regulator consumer abstractions.
//!
//! C header: [`include/linux/regulator/consumer.h`](srctree/include/linux/regulator/consumer.h)
//!
//! Reference: <https://docs.kernel.org/driver-api/regulator.html>

use crate::{
    bindings,
    device::Device,
    error::{code::*, from_err_ptr, to_result, Error, Result},
    regulator::Mode,
    str::CStr,
};
use core::{
    cmp::min,
    ffi::{c_int, c_uint},
    time::Duration,
};

/// [`Regulator`] in its default state (disabled)
///
/// # Invariants
///   - [`self.0`] is valid and non-null
pub struct Regulator(*mut bindings::regulator);

impl Regulator {
    /// Lookup and obtain an instance of a regulator
    ///
    /// If the supply does not exists a dummy one will be
    /// created
    pub fn get<D: AsRef<Device>>(dev: D, id: &'static CStr) -> Result<Self> {
        // SAFETY: `dev.as_ref().as_raw() is valid and non-null by the type invariant and
        // id has a static lifetime so it lives indefinitely
        let reg = from_err_ptr(unsafe {
            bindings::regulator_get(dev.as_ref().as_raw(), id.as_char_ptr())
        })?;

        // This should not happen: in case of error `regulator_get` returns an
        // error encoded into the pointer. And in case the device does not
        // exists, a dummy regulator is returned
        if reg.is_null() {
            return Err(ENODEV);
        }

        Ok(Self(reg))
    }

    /// Same as `get`, but if the regulator does not exists
    /// an error will be returned instead of a dummy regulator
    pub fn get_optional<D: AsRef<Device>>(dev: D, id: &'static CStr) -> Result<Self> {
        // SAFETY: `dev.as_ref().as_raw() is valid and non-null by the type invariant and
        // id has a static lifetime so it lives indefinitely
        let reg = from_err_ptr(unsafe {
            bindings::regulator_get_optional(dev.as_ref().as_raw(), id.as_char_ptr())
        })?;

        // does not exists `regulator_get_optional` returns an
        // error encoded into the pointer.
        if reg.is_null() {
            return Err(ENODEV);
        }

        Ok(Self(reg))
    }

    /// Same as `get` but ensure that we have exclusive access to the regulator
    pub fn get_exclusive<D: AsRef<Device>>(dev: D, id: &'static CStr) -> Result<Self> {
        // SAFETY: `dev.as_ref().as_raw() is valid and non-null by the type invariant and
        // id has a static lifetime so it lives indefinitely
        let reg = from_err_ptr(unsafe {
            bindings::regulator_get_exclusive(dev.as_ref().as_raw(), id.as_char_ptr())
        })?;

        // This should not happen: in case of error `regulator_get` returns an
        // error encoded into the pointer. And in case the device does not
        // exists, a dummy regulator is returned
        if reg.is_null() {
            return Err(ENODEV);
        }

        Ok(Self(reg))
    }

    /// Check if the regulator is enabled
    pub fn is_enabled(&mut self) -> Result<bool> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let val = unsafe { bindings::regulator_enable(self.0) };
        if val < 0 {
            return Err(Error::from_errno(val));
        }

        Ok(val > 0)
    }

    /// Enable the regulator
    pub fn enable(&mut self) -> Result {
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_enable(self.0) })
    }

    /// Disable the regulator
    pub fn disable(&mut self) -> Result {
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_disable(self.0) })
    }

    /// Disable the regulator with a specified delay
    ///
    /// Every non-zero delay < 1ms will be rounded up to 1ms, and any delay
    /// longer than [`core::ffi::c_int`] will become [`core::ffi::c_int::MAX`]
    pub fn disable_deferred(&mut self, duration: Duration) -> Result {
        let ms = min(duration.as_millis(), c_int::MAX as u128) as c_int;
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_disable_deferred(self.0, ms) })
    }

    /// Force disable the regulator. Even if other consumer
    /// have enabled it, the regulator will be forcibly disabled.
    pub fn force_disable(&mut self) -> Result {
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_force_disable(self.0) })
    }

    /// Check if the voltage range can be supported
    pub fn is_supported_voltage(&self, min_uv: c_int, max_uv: c_int) -> Result<bool> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let ret = unsafe { bindings::regulator_is_supported_voltage(self.0, min_uv, max_uv) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }

        Ok(ret > 0)
    }

    /// Returns the number of selectors supported by the regulator
    pub fn count_voltages(&self) -> Result<usize> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let ret = unsafe { bindings::regulator_count_voltages(self.0) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }

        Ok(ret as _)
    }

    /// Returns the voltage corresponding to the `selector`
    pub fn list_voltage(&self, selector: c_uint) -> Result<Option<c_int>> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let ret = unsafe { bindings::regulator_list_voltage(self.0, selector) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }

        Ok(if ret == 0 { None } else { Some(ret) })
    }

    /// Returns the voltage step size between VSEL values
    pub fn get_linear_step(&self) -> Option<c_uint> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let ret = unsafe { bindings::regulator_get_linear_step(self.0) };
        if ret == 0 {
            None
        } else {
            Some(ret)
        }
    }

    /// Returns the regulator output voltage
    pub fn get_voltage(&self) -> Result<c_int> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let ret = unsafe { bindings::regulator_get_voltage(self.0) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }

        Ok(ret)
    }

    /// Set the regulator output voltage
    pub fn set_voltage(&mut self, min_uv: c_int, max_uv: c_int) -> Result {
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_set_voltage(self.0, min_uv, max_uv) })
    }

    /// Get the raise/fall time required for switching voltage
    pub fn set_voltage_time(&mut self, old_uv: c_int, new_uv: c_int) -> Result<c_int> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let ret = unsafe { bindings::regulator_set_voltage_time(self.0, old_uv, new_uv) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }

        Ok(ret)
    }

    /// Re-apply last regulator output voltage
    pub fn sync_voltage(&mut self) -> Result {
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_sync_voltage(self.0) })
    }

    /// Get regulator output current
    pub fn get_current_limit(&self) -> Result<c_int> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let ret = unsafe { bindings::regulator_get_current_limit(self.0) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }

        Ok(ret)
    }

    /// Set regulator output current limit
    pub fn set_current_limit(&mut self, min_ua: c_int, max_ua: c_int) -> Result<c_int> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        let ret = unsafe { bindings::regulator_set_current_limit(self.0, min_ua, max_ua) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }

        Ok(ret)
    }

    /// Set regulator load
    pub fn set_load(&mut self, load_ua: c_int) -> Result {
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_set_load(self.0, load_ua) })
    }

    /// Allow the regulator to go into bypass mode
    pub fn allow_bypass(&mut self, allow: bool) -> Result {
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_allow_bypass(self.0, allow) })
    }

    /// Set the mode of the regulator
    pub fn set_mode(&mut self, mode: Mode) -> Result {
        // SAFETY: The pointer is valid and non-null by the type invariant
        to_result(unsafe { bindings::regulator_set_mode(self.0, mode as _) })
    }

    /// Get the current mode of the regulator
    pub fn get_mode(&mut self) -> Result<Mode> {
        // SAFETY: The pointer is valid and non-null by the type invariant
        Mode::try_from(unsafe { bindings::regulator_get_mode(self.0) })
    }
}

impl Drop for Regulator {
    fn drop(&mut self) {
        // SAFETY: The pointer is valid and non-null by the type invariant
        unsafe { bindings::regulator_put(self.0) }
    }
}

// SAFETY: `Regulator` is not restricted to a single thread so it is safe
// to move it between threads
unsafe impl Send for Regulator {}

impl PartialEq<Regulator> for Regulator {
    fn eq(&self, other: &Regulator) -> bool {
        // SAFETY: The pointers are valid and non-null by the type invariant
        unsafe { bindings::regulator_is_equal(self.0, other.0) }
    }
}
