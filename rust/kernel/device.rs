// SPDX-License-Identifier: GPL-2.0

//! Generic devices that are part of the kernel's driver model.
//!
//! C header: [`include/linux/device.h`](../../../../include/linux/device.h)

use crate::{
    bindings,
    types::{ARef, Opaque},
};
use core::ptr;

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
