use crate::{
    bindings, dev_err, device, driver,
    error::{from_result, to_result, Error, Result},
    of,
    str::CStr,
    types::ForeignOwnable,
    ThisModule,
};

#[repr(u32)]
pub enum PixelFormat {
    Rgb888 = bindings::mipi_dsi_pixel_format_MIPI_DSI_FMT_RGB888,
    Rgb666 = bindings::mipi_dsi_pixel_format_MIPI_DSI_FMT_RGB666,
    Rgb666Packed = bindings::mipi_dsi_pixel_format_MIPI_DSI_FMT_RGB666_PACKED,
    Rgb565 = bindings::mipi_dsi_pixel_format_MIPI_DSI_FMT_RGB565,
}

pub mod mode {
    pub const VIDEO: u64 = 0b1;
    pub const VIDEO_BURST: u64 = 0b10;
    pub const VIDEO_SYNC_PULSE: u64 = 0b100;
    pub const VIDEO_AUTO_VERT: u64 = 0b1000;
    pub const VIDEO_HSE: u64 = 0b10000;
    pub const VIDEO_NO_HFP: u64 = 0b100000;
    pub const VIDEO_NO_HBP: u64 = 0b1000000;
    pub const VIDEO_NO_HSA: u64 = 0b10000000;
    pub const VSYNC_FLUSH: u64 = 0b100000000;
    pub const NO_EOT_PACKET: u64 = 0b1000000000;
    pub const CLOCK_NON_CONTINUOUS: u64 = 0b10000000000;
    pub const LPM: u64 = 0b100000000000;
    pub const HS_PKT_END_ALIGNED: u64 = 0b1000000000000;
}

pub struct Adapter<T: Driver>(T);

impl<T: Driver> driver::RegistrationOps for Adapter<T> {
    type RegType = bindings::mipi_dsi_driver;

    fn register(
        pdrv: &mut bindings::mipi_dsi_driver,
        name: &'static CStr,
        module: &'static ThisModule,
    ) -> Result {
        pdrv.driver.name = name.as_char_ptr();
        pdrv.probe = Some(Self::probe_callback);
        pdrv.remove = Some(Self::remove_callback);
        pdrv.shutdown = None;
        if let Some(t) = T::OF_DEVICE_ID_TABLE {
            pdrv.driver.of_match_table = t.as_ptr();
        }
        // SAFETY:
        //   - `pdrv` lives at least until the call to `mipi_dsi_driver_unregister()` returns.
        //   - `name` pointer has static lifetime.
        //   - `module.0` lives at least as long as the module.
        //   - `probe()` and `remove()` are static functions.
        //   - `of_match_table` is either a raw pointer with static lifetime,
        //      as guaranteed by the [`device_id::IdTable`] type, or null.
        to_result(unsafe { bindings::mipi_dsi_driver_register_full(pdrv, module.0) })
    }

    fn unregister(pdrv: &mut bindings::mipi_dsi_driver) {
        // SAFETY: By the safety requirements of this function (defined in the trait definition),
        // `pdrv` was passed (and updated) by a previous successful call to
        // `mipi_dsi_driver_register`.
        unsafe { bindings::mipi_dsi_driver_unregister(pdrv) };
    }
}

impl<T: Driver> Adapter<T> {
    fn get_id_info(dev: &Device) -> Option<&'static T::IdInfo> {
        let table = T::OF_DEVICE_ID_TABLE?;

        // SAFETY: `table` has static lifetime, so it is valid for read. `dev` is guaranteed to be
        // valid while it's alive, so is the raw device returned by it.
        let id = unsafe { bindings::of_match_device(table.as_ptr(), dev.as_ref().as_raw()) };
        if id.is_null() {
            return None;
        }

        // SAFETY: `id` is a pointer within the static table, so it's always valid.
        let offset = unsafe { (*id).data };
        if offset.is_null() {
            return None;
        }

        // SAFETY: The offset comes from a previous call to `offset_from` in `IdArray::new`, which
        // guarantees that the resulting pointer is within the table.
        let ptr = unsafe {
            id.cast::<u8>()
                .offset(offset as _)
                .cast::<Option<T::IdInfo>>()
        };

        // SAFETY: The id table has a static lifetime, so `ptr` is guaranteed to be valid for read.
        #[allow(clippy::needless_borrow)]
        unsafe {
            (&*ptr).as_ref()
        }
    }

    extern "C" fn probe_callback(dsi: *mut bindings::mipi_dsi_device) -> core::ffi::c_int {
        from_result(|| {
            let dev = unsafe { Device::from_ptr(dsi) };
            let info = Self::get_id_info(&dev);
            let data = T::probe(dev, info)?;
            // SAFETY: `dev` is guaranteed to be a valid, non-null pointer.
            unsafe { bindings::dev_set_drvdata(&mut (*dsi).dev, data.into_foreign() as _) };

            let mut dsi = unsafe { Device::from_ptr(dsi) };
            dsi.attach()?;

            Ok(0)
        })
    }

    extern "C" fn remove_callback(dsi: *mut bindings::mipi_dsi_device) {
        let dev = unsafe { &mut (*dsi).dev };
        let ptr = unsafe { bindings::dev_get_drvdata(dev) };
        // SAFETY:
        //   - we allocated this pointer using `T::Data::into_foreign`,
        //     so it is safe to turn back into a `T::Data`.
        //   - the allocation happened in `probe`, no-one freed the memory,
        //     `remove` is the canonical kernel location to free driver data. so OK
        //     to convert the pointer back to a Rust structure here.
        let data = unsafe { T::Data::from_foreign(ptr) };
        let mut dsi = unsafe { Device::from_ptr(dsi) };
        if let Err(e) = dsi.detach() {
            dev_err!(dsi.as_ref(), "Failed to detach DSI device: {e:?}");
        }
        T::remove(dsi, &data);
    }
}

pub trait Driver {
    type Data: ForeignOwnable + Send + Sync = ();

    /// The type holding information about each device id supported by the driver.
    type IdInfo: 'static = ();

    /// The table of device ids supported by the driver.
    const OF_DEVICE_ID_TABLE: Option<of::IdTable<Self::IdInfo>> = None;

    fn probe(dev: Device, id_info: Option<&Self::IdInfo>) -> Result<Self::Data>;

    fn remove(_dev: Device, _data: &Self::Data) {}

    fn shutdown(_dev: Device, _data: &Self::Data) {}
}

pub struct Device {
    ptr: *mut bindings::mipi_dsi_device,
}

impl Device {
    /// Creates a new device from the given pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and valid. It must remain valid for the lifetime of the returned
    /// instance.
    unsafe fn from_ptr(ptr: *mut bindings::mipi_dsi_device) -> Self {
        // INVARIANT: The safety requirements of the function ensure the lifetime invariant.
        Self { ptr }
    }

    pub fn set_lanes(&mut self, lanes: u32) {
        unsafe { (*self.ptr).lanes = lanes };
    }

    pub fn set_format(&mut self, format: PixelFormat) {
        unsafe { (*self.ptr).format = format as _ };
    }

    pub fn mode_flags_mut(&mut self) -> &mut u64 {
        unsafe { &mut (*self.ptr).mode_flags }
    }

    fn attach(&mut self) -> Result {
        to_result(unsafe { bindings::mipi_dsi_attach(self.ptr) })
    }

    fn detach(&mut self) -> Result {
        to_result(unsafe { bindings::mipi_dsi_detach(self.ptr) })
    }

    pub fn dcs_soft_reset(&mut self) -> Result {
        to_result(unsafe { bindings::mipi_dsi_dcs_soft_reset(self.ptr) })
    }

    pub fn dcs_enter_sleep_mode(&mut self) -> Result {
        to_result(unsafe { bindings::mipi_dsi_dcs_enter_sleep_mode(self.ptr) })
    }

    pub fn dcs_exit_sleep_mode(&mut self) -> Result {
        to_result(unsafe { bindings::mipi_dsi_dcs_exit_sleep_mode(self.ptr) })
    }

    pub fn dcs_set_display_on(&mut self) -> Result {
        to_result(unsafe { bindings::mipi_dsi_dcs_set_display_on(self.ptr) })
    }

    pub fn dcs_set_display_off(&mut self) -> Result {
        to_result(unsafe { bindings::mipi_dsi_dcs_set_display_off(self.ptr) })
    }

    pub fn dcs_write_buffer(&mut self, buf: &[u8]) -> Result<usize> {
        let ret =
            unsafe { bindings::mipi_dsi_dcs_write_buffer(self.ptr, buf.as_ptr() as _, buf.len()) };
        if ret < 0 {
            Err(Error::from_errno(ret as _))
        } else {
            Ok(ret as _)
        }
    }
}

unsafe impl Send for Device {}

impl AsRef<device::Device> for Device {
    fn as_ref(&self) -> &device::Device {
        // SAFETY: By the type invariants, we know that `self.ptr` is non-null and valid.
        unsafe { device::Device::as_ref(&mut (*self.ptr).dev) }
    }
}

#[macro_export]
macro_rules! module_mipi_dsi_driver {
    ($($f:tt)*) => {
        $crate::module_driver!(<T>, $crate::drm::mipi_dsi::Adapter<T>, { $($f)* });
    };
}
