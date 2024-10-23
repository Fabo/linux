// SPDX-License-Identifier: GPL-2.0

//! I2C devices and drivers.
//!
//! C header: [`include/linux/i2c.h`](srctree/include/linux/i2c.h)

use crate::{
    bindings,
    device::Device,
    device_id::{self, RawDeviceId},
    driver,
    error::{from_result, to_result, Result},
    of,
    prelude::*,
    str::CStr,
    types::ForeignOwnable,
    ThisModule,
};

/// An I2C device id.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DeviceId(bindings::i2c_device_id);

// SAFETY: `ZERO` is all zeroed-out and `to_rawid` stores `offset` in `i2c_device_id::driver_data`.
unsafe impl RawDeviceId for DeviceId {
    type RawType = bindings::i2c_device_id;

    const DRIVER_DATA_OFFSET: usize = core::mem::offset_of!(bindings::i2c_device_id, driver_data);

    fn index(&self) -> usize {
        self.0.driver_data as _
    }
}

impl DeviceId {
    /// Create a new I2C DeviceId
    pub const fn new(name: &CStr) -> Self {
        let src = name.as_bytes_with_nul();
        let mut i2c: bindings::i2c_device_id = unsafe { core::mem::zeroed() };

        let mut i = 0;
        while i < src.len() {
            i2c.name[i] = src[i] as _;
            i += 1;
        }

        Self(i2c)
    }
}

/// Alias for `device_id::IdTable` containing I2C's `DeviceId`
pub type IdTable<T> = &'static dyn device_id::IdTable<DeviceId, T>;

/// An adapter for the registration of i2c drivers.
pub struct Adapter<T: Driver + 'static>(T);

impl<T: Driver + 'static> driver::RegistrationOps for Adapter<T> {
    type RegType = bindings::i2c_driver;

    fn register(
        i2cdrv: &mut Self::RegType,
        name: &'static CStr,
        module: &'static ThisModule,
    ) -> Result {
        i2cdrv.driver.name = name.as_char_ptr();
        i2cdrv.probe = Some(Self::probe_callback);
        i2cdrv.remove = Some(Self::remove_callback);
        if let Some(t) = T::I2C_TABLE {
            i2cdrv.id_table = t.as_ptr();
        }
        if let Some(t) = T::OF_TABLE {
            i2cdrv.driver.of_match_table = t.as_ptr();
        }

        // SAFETY:
        //   - `pdrv` lives at least until the call to `platform_driver_unregister()` returns.
        //   - `name` pointer has static lifetime.
        //   - `module.0` lives at least as long as the module.
        //   - `probe()` and `remove()` are static functions.
        //   - `of_match_table` is either a raw pointer with static lifetime,
        //      as guaranteed by the [`device_id::IdTable`] type, or null.
        to_result(unsafe { bindings::i2c_register_driver(module.0, i2cdrv) })
    }

    fn unregister(i2cdrv: &mut Self::RegType) {
        // SAFETY: By the safety requirements of this function (defined in the trait definition),
        // `reg` was passed (and updated) by a previous successful call to
        // `i2c_register_driver`.
        unsafe { bindings::i2c_del_driver(i2cdrv) };
    }
}

impl<T: Driver> Adapter<T> {
    extern "C" fn probe_callback(i2c: *mut bindings::i2c_client) -> core::ffi::c_int {
        from_result(|| {
            let mut client = unsafe { Client::from_ptr(i2c) };
            let data = T::probe(&mut client)?;

            // SAFETY: `i2c` is guaranteed to be a valid, non-null pointer.
            unsafe { bindings::i2c_set_clientdata(i2c, data.into_foreign() as _) };
            Ok(0)
        })
    }

    extern "C" fn remove_callback(i2c: *mut bindings::i2c_client) {
        // SAFETY: `i2c` is a valid pointer to a `struct i2c_client`.
        let ptr = unsafe { bindings::i2c_get_clientdata(i2c) };

        // SAFETY: `remove_callback` is only ever called after a successful call to
        // `probe_callback`, hence it's guaranteed that `ptr` points to a valid and initialized
        // `KBox<T>` pointer created through `KBox::into_foreign`.
        let _ = unsafe { KBox::<T>::from_foreign(ptr) };
    }
}

/// A I2C driver.
pub trait Driver {
    /// The type holding information about each device id supported by the driver.
    type IdInfo: 'static;

    /// The table of i2c device ids supported by the driver.
    const I2C_TABLE: Option<IdTable<Self::IdInfo>>;

    /// The table of OF device ids supported by the driver.
    const OF_TABLE: Option<of::IdTable<Self::IdInfo>>;

    /// I2C driver probe.
    ///
    /// Called when a new i2c client is added or discovered.
    /// Implementers should attempt to initialize the client here.
    fn probe(client: &mut Client) -> Result<Pin<KBox<Self>>>;
}

/// A I2C Client device.
///
/// # Invariants
///
/// The field `ptr` is non-null and valid for the lifetime of the object.
pub struct Client {
    ptr: *mut bindings::i2c_client,
}

impl Client {
    /// Creates a new client from the given pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and valid. It must remain valid for the lifetime of the returned
    /// instance.
    unsafe fn from_ptr(ptr: *mut bindings::i2c_client) -> Self {
        // INVARIANT: The safety requirements of the function ensure the lifetime invariant.
        Self { ptr }
    }

    /// Returns the raw I2C client structure.
    pub fn raw_client(&self) -> *mut bindings::i2c_client {
        self.ptr
    }
}

impl AsRef<Device> for Client {
    fn as_ref(&self) -> &Device {
        // SAFETY: By the type invariants, we know that `self.ptr` is non-null and valid.
        unsafe { Device::as_ref(&mut (*self.ptr).dev) }
    }
}

/// Declares a kernel module that exposes a single i2c driver.
///
/// # Examples
///
/// ```ignore
/// # use kernel::{i2c, define_i2c_id_table, module_i2c_driver};
/// kernel::module_i2c_id_table!(MOD_TABLE, I2C_CLIENT_I2C_ID_TABLE);
/// kernel::define_i2c_id_table! {I2C_CLIENT_I2C_ID_TABLE, (), [
///     (i2c::DeviceId(b"fpga"), None),
/// ]}
/// struct MyDriver;
/// impl i2c::Driver for MyDriver {
///     kernel::driver_i2c_id_table!(I2C_CLIENT_I2C_ID_TABLE);
///     // [...]
/// #   fn probe(_client: &mut i2c::Client) -> Result {
/// #       Ok(())
/// #   }
/// }
///
/// module_i2c_driver! {
///     type: MyDriver,
///     name: "module_name",
///     author: "Author name",
///     license: "GPL",
/// }
/// ```
#[macro_export]
macro_rules! module_i2c_driver {
    ($($f:tt)*) => {
        $crate::module_driver!(<T>, $crate::i2c::Adapter<T>, { $($f)* });
    };
}

/// Create an I2C `IdTable` with an "alias" for modpost.
#[macro_export]
macro_rules! i2c_device_table {
    ($table_name:ident, $module_table_name:ident, $id_info_type: ty, $table_data: expr) => {
        const $table_name: $crate::device_id::IdArray<
            $crate::i2c::DeviceId,
            $id_info_type,
            { $table_data.len() },
        > = $crate::device_id::IdArray::new($table_data);

        $crate::module_device_table!("i2c", $module_table_name, $table_name);
    };
}
