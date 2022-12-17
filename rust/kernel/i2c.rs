// SPDX-License-Identifier: GPL-2.0

//! I2C devices and drivers.
//!
//! See [`Driver`] trait for an example on how to use this API.
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
    types::{ARef, AlwaysRefCounted, ForeignOwnable, Opaque},
    ThisModule,
};
use core::ptr::NonNull;

/// An I2C device id.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DeviceId(bindings::i2c_device_id);

// SAFETY:
// * `DeviceId` is a `#[repr(transparent)` wrapper of `struct i2c_device_id` and does not add
//   additional invariants, so it's safe to transmute to `RawType`.
// * `DRIVER_DATA_OFFSET` is the offset to the `data` field.
unsafe impl RawDeviceId for DeviceId {
    type RawType = bindings::i2c_device_id;

    const DRIVER_DATA_OFFSET: usize = core::mem::offset_of!(bindings::i2c_device_id, driver_data);

    fn index(&self) -> usize {
        self.0.driver_data as _
    }
}

impl DeviceId {
    /// Create a new device id from an I2C name.
    pub const fn new(name: &CStr) -> Self {
        let src = name.as_bytes_with_nul();
        // Replace with `bindings::i2c_device_id::default()` once stabilized for `const`.
        // SAFETY: FFI type is valid to be zero-initialized.
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

/// An adapter for the registration of I2C drivers.
#[doc(hidden)]
pub struct Adapter<T: Driver + 'static>(T);

impl<T: Driver + 'static> driver::RegistrationOps for Adapter<T> {
    type RegType = bindings::i2c_driver;

    fn register(
        i2cdrv: &Opaque<Self::RegType>,
        name: &'static CStr,
        module: &'static ThisModule,
    ) -> Result {
        unsafe {
            (*i2cdrv.get()).driver.name = name.as_char_ptr();
            (*i2cdrv.get()).probe = Some(Self::probe_callback);
            (*i2cdrv.get()).remove = Some(Self::remove_callback);
            if let Some(t) = T::I2C_ID_TABLE {
                (*i2cdrv.get()).id_table = t.as_ptr();
            }
            if let Some(t) = T::OF_ID_TABLE {
                (*i2cdrv.get()).driver.of_match_table = t.as_ptr();
            }
        }

        // SAFETY: `i2cdrv` is guaranteed to be a valid `RegType`.
        to_result(unsafe { bindings::i2c_register_driver(module.0, i2cdrv.get()) })
    }

    fn unregister(i2cdrv: &Opaque<Self::RegType>) {
        // SAFETY: `i2cdrv` is guaranteed to be a valid `RegType`.
        unsafe { bindings::i2c_del_driver(i2cdrv.get()) };
    }
}

impl<T: Driver> Adapter<T> {
    /// Get the `Self::IdInfo` that matched during probe.
    fn id_info(client: &ARef<Client>) -> Option<&'static T::IdInfo> {
        let id = <Self as driver::Adapter>::id_info(client.as_ref());
        if id.is_some() {
            return id;
        }

        let id = unsafe { bindings::i2c_client_get_device_id(client.as_raw()) };
        if !id.is_null() {
            let id = unsafe { &*id.cast::<DeviceId>() };
            return Some(T::I2C_ID_TABLE?.info(id.index()));
        }

        None
    }

    extern "C" fn probe_callback(i2c: *mut bindings::i2c_client) -> core::ffi::c_int {
        from_result(|| {
            // SAFETY: `i2c` is a valid pointer to a `struct i2c_client`.
            let client = unsafe { Client::from_raw(i2c) };
            let info = Self::id_info(&client);
            let data = T::probe(&client, info)?;

            // SAFETY: `i2c` is a valid pointer to a `struct i2c_client`.
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

impl<T: Driver + 'static> driver::Adapter for Adapter<T> {
    type IdInfo = T::IdInfo;

    fn of_id_table() -> Option<of::IdTable<Self::IdInfo>> {
        T::OF_ID_TABLE
    }
}

/// The I2C driver trait.
///
/// Drivers must implement this trait in order to get a platform driver registered.
///
/// # Example
///
///```
/// # use kernel::{bindings, c_str, i2c, of};
///
/// struct MyDriver;
///
/// kernel::of_device_table!(
///     OF_ID_TABLE,
///     MODULE_OF_ID_TABLE,
///     <MyDriver as platform::Driver>::IdInfo,
///     [(of::DeviceId::new(c_str!("onnn,ncv6336")), ()),]
/// );
///
/// kernel::i2c_device_table!(
///     I2C_ID_TABLE,
///     MODULE_I2C_ID_TABLE,
///     <Ncv6336 as i2c::Driver>::IdInfo,
///     [(i2c::DeviceId::new(c_str!("ncv6336")), ()),]
/// );
///
/// impl i2c::Driver for MyDriver {
///     type IdInfo = ();
///     const OF_ID_TABLE: of::IdTable<Self::IdInfo> = &OF_ID_TABLE;
///     const I2C_ID_TABLE: i2c::IdTable<Self::IdInfo> = &I2C_ID_TABLE;
///
///     fn probe(_client: &ARef<i2c::Client>, id_info: Option<&Self::IdInfo>) -> Result {
///         Ok(())
///     }
/// }
///```
pub trait Driver {
    /// The type holding information about each device id supported by the driver.
    // TODO: Use associated_type_defaults once stabilized:
    // type IdInfo: 'static = ();
    type IdInfo: 'static;

    /// An optional table of I2C device ids supported by the driver.
    const I2C_ID_TABLE: Option<IdTable<Self::IdInfo>>;

    /// An optional table of OF device ids supported by the driver.
    const OF_ID_TABLE: Option<of::IdTable<Self::IdInfo>>;

    /// I2C driver probe.
    ///
    /// Called when a new I2C client is added or discovered.
    fn probe(client: &ARef<Client>, id_info: Option<&Self::IdInfo>) -> Result<Pin<KBox<Self>>>;
}

/// An I2C Client.
///
/// # Invariants
///
/// `Device` holds a valid reference of `ARef<device::Device>` whose underlying `struct device` is a
/// member of a `struct i2c_client`.
#[repr(transparent)]
pub struct Client(Opaque<bindings::i2c_client>);

impl Client {
    /// Convert a raw client into a `Client`.
    ///
    /// # Safety
    ///
    /// `i2c` must be non-null and valid. It must remain valid for the lifetime of the returned
    /// instance.
    unsafe fn from_raw(i2c: *mut bindings::i2c_client) -> ARef<Self> {
        // SAFETY: By the safety requirements, ptr is valid.
        // Initially increase the reference count by one to compensate for the final decrement once
        // this newly created `ARef<Device>` instance is dropped.
        unsafe { bindings::get_device(&mut (*i2c).dev) };

        // CAST: `Self` is a `repr(transparent)` wrapper around `bindings::device`.
        let i2c = i2c.cast::<Self>();

        // SAFETY: `ptr` is valid by the safety requirements of this function. By the above call to
        // `bindings::get_device` we also own a reference to the underlying `struct device`.
        unsafe { ARef::from_raw(NonNull::new_unchecked(i2c)) }
    }

    /// Returns the raw I2C client structure.
    pub fn as_raw(&self) -> *mut bindings::i2c_client {
        unsafe { &mut (*self.0.get()) }
    }
}

impl AsRef<Device> for Client {
    fn as_ref(&self) -> &Device {
        unsafe { Device::as_ref(&mut (*self.as_raw()).dev) }
    }
}

unsafe impl AlwaysRefCounted for Client {
    fn inc_ref(&self) {
        unsafe { bindings::get_device(&mut (*self.as_raw()).dev) };
    }

    unsafe fn dec_ref(obj: NonNull<Self>) {
        unsafe { bindings::put_device(&mut (*obj.as_ref().as_raw()).dev) };
    }
}

/// Declares a kernel module that exposes a single I2C driver.
///
/// # Examples
///
/// ```ignore
/// kernel::module_i2c_driver! {
///     type: MyDriver,
///     name: "Module name",
///     author: "Author name",
///     description: "Description",
///     license: "GPL v2",
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
