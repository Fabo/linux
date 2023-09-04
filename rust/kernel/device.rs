// SPDX-License-Identifier: GPL-2.0

//! Generic devices that are part of the kernel's driver model.
//!
//! C header: [`include/linux/device.h`](../../../../include/linux/device.h)

use macros::pin_data;

use crate::{
    alloc::flags::*,
    bindings,
    error::Result,
    init::InPlaceInit,
    init::PinInit,
    pin_init,
    revocable::{Revocable, RevocableGuard},
    str::CStr,
    sync::{LockClassKey, RevocableMutex, RevocableMutexGuard, UniqueArc},
    types::{ARef, Opaque},
};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
};

/// A ref-counted device.
///
/// # Invariants
///
/// `ptr` is valid, non-null, and has a non-zero reference count. One of the references is owned by
/// `self`, and will be decremented when `self` is dropped.
#[repr(transparent)]
pub struct Device(Opaque<bindings::device>);

// SAFETY: `Device` only holds a pointer to a C device, which is safe to be used from any thread.
unsafe impl Send for Device {}

// SAFETY: `Device` only holds a pointer to a C device, references to which are safe to be used
// from any thread.
unsafe impl Sync for Device {}

impl Device {
    /// Creates a new device instance.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` is valid, non-null, and has a non-zero reference count.
    pub unsafe fn new(ptr: *mut bindings::device) -> ARef<Self> {
        // CAST: `Self` is a `repr(transparent)` wrapper around `bindings::device`.
        let ptr = ptr.cast::<Self>();

        // SAFETY: By the safety requirements, ptr is valid and its refcounted will be incremented.
        unsafe { ARef::from_raw(ptr::NonNull::new_unchecked(ptr)) }.clone()
    }

    /// Obtain the raw `struct device *` from a `&Device`.
    pub fn as_raw(&self) -> *mut bindings::device {
        self.0.get()
    }

    /// Convert a raw `struct device` pointer to a `&Device`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` is valid, non-null, and has a non-zero reference count for
    /// the entire duration when the returned reference exists.
    pub unsafe fn from_raw<'a>(ptr: *mut bindings::device) -> &'a Self {
        // SAFETY: Guaranteed by the safety requirements of the function.
        unsafe { &*ptr.cast() }
    }
}

// SAFETY: Instances of `Device` are always refcounted.
unsafe impl crate::types::AlwaysRefCounted for Device {
    fn inc_ref(&self) {
        // SAFETY: The existence of a shared reference means that the refcount is nonzero.
        unsafe { bindings::get_device(self.0.get()) };
    }

    unsafe fn dec_ref(obj: ptr::NonNull<Self>) {
        // SAFETY: The safety requirements guarantee that the refcount is nonzero.
        unsafe { bindings::put_device(obj.cast().as_ptr()) }
    }
}

/// Device data.
///
/// When a device is removed (for whatever reason, for example, because the device was unplugged or
/// because the user decided to unbind the driver), the driver is given a chance to clean its state
/// up, and all io resources should ideally not be used anymore.
///
/// However, the device data is reference-counted because other subsystems hold pointers to it. So
/// some device state must be freed and not used anymore, while others must remain accessible.
///
/// This struct separates the device data into three categories:
///   1. Registrations: are destroyed when the device is removed, but before the io resources
///      become inaccessible.
///   2. Io resources: are available until the device is removed.
///   3. General data: remain available as long as the ref count is nonzero.
///
/// This struct implements the `DeviceRemoval` trait so that it can clean resources up even if not
/// explicitly called by the device drivers.
#[pin_data]
pub struct Data<T, U, V> {
    #[pin]
    registrations: RevocableMutex<T>,
    #[pin]
    resources: Revocable<U>,
    #[pin]
    general: V,
}

/// Safely creates an new reference-counted instance of [`Data`].
#[doc(hidden)]
#[macro_export]
macro_rules! new_device_data {
    ($reg:expr, $res:expr, $gen:expr, $name:literal) => {{
        static CLASS1: $crate::sync::LockClassKey = $crate::sync::LockClassKey::new();
        let regs = $reg;
        let res = $res;
        let gen = $gen;
        let name = $crate::c_str!($name);
        $crate::device::Data::try_new(regs, res, gen, name, &CLASS1)
    }};
}

impl<T, U, V> Data<T, U, V> {
    /// Creates a new instance of `Data`.
    ///
    /// It is recommended that the [`new_device_data`] macro be used as it automatically creates
    /// the lock classes.
    pub fn try_new(
        registrations: T,
        resources: impl PinInit<U>,
        general: impl PinInit<V>,
        name: &'static CStr,
        key1: &'static LockClassKey,
    ) -> Result<Pin<UniqueArc<Self>>> {
        let ret = UniqueArc::pin_init(
            pin_init!(Self {
                registrations <- RevocableMutex::new(
                    registrations,
                    name,
                    key1,
                ),
                resources <- Revocable::new(resources),
                general <- general,
            }),
            GFP_KERNEL,
        )?;

        Ok(ret)
    }

    /// Returns the resources if they're still available.
    pub fn resources(&self) -> Option<RevocableGuard<'_, U>> {
        self.resources.try_access()
    }

    /// Returns the locked registrations if they're still available.
    pub fn registrations(&self) -> Option<RevocableMutexGuard<'_, T>> {
        self.registrations.try_write()
    }
}

impl<T, U, V> crate::driver::DeviceRemoval for Data<T, U, V> {
    fn device_remove(&self) {
        // We revoke the registrations first so that resources are still available to them during
        // unregistration.
        self.registrations.revoke();

        // Release resources now. General data remains available.
        self.resources.revoke();
    }
}

impl<T, U, V> Deref for Data<T, U, V> {
    type Target = V;

    fn deref(&self) -> &V {
        &self.general
    }
}

impl<T, U, V> DerefMut for Data<T, U, V> {
    fn deref_mut(&mut self) -> &mut V {
        &mut self.general
    }
}
