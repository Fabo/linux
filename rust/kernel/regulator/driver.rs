use crate::{
    device::{Device, RawDevice},
    error::{code::*, from_err_ptr, from_result, to_result, Error, Result},
    macros::vtable,
    regmap::Regmap,
    regulator::Mode,
    str::CStr,
    types::ForeignOwnable,
    ThisModule,
};
use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

pub type LinearRange = bindings::linear_range;
///
/// [`Regulator`] status
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Status {
    /// Regulator is OFF
    Off = bindings::regulator_status_REGULATOR_STATUS_OFF,
    /// Regulator is ON
    On = bindings::regulator_status_REGULATOR_STATUS_ON,
    /// Regulator is in an error state
    Error = bindings::regulator_status_REGULATOR_STATUS_ERROR,
    /// Regulator is ON and in Fast mode
    Fast = bindings::regulator_status_REGULATOR_STATUS_FAST,
    /// Regulator is ON and in Normal mode
    Normal = bindings::regulator_status_REGULATOR_STATUS_NORMAL,
    /// Regulator is ON and in Idle mode
    Idle = bindings::regulator_status_REGULATOR_STATUS_IDLE,
    /// Regulator is ON and in Standby mode
    Standby = bindings::regulator_status_REGULATOR_STATUS_STANDBY,
    /// Regulator is enabled but not regulating
    Bypass = bindings::regulator_status_REGULATOR_STATUS_BYPASS,
    /// Regulator is any other status
    Undefined = bindings::regulator_status_REGULATOR_STATUS_UNDEFINED,
}

impl Status {
    fn from_bindings(status: core::ffi::c_uint) -> Result<Self> {
        match status {
            bindings::regulator_status_REGULATOR_STATUS_OFF => Ok(Self::Off),
            bindings::regulator_status_REGULATOR_STATUS_ON => Ok(Self::On),
            bindings::regulator_status_REGULATOR_STATUS_ERROR => Ok(Self::Error),
            bindings::regulator_status_REGULATOR_STATUS_FAST => Ok(Self::Fast),
            bindings::regulator_status_REGULATOR_STATUS_NORMAL => Ok(Self::Normal),
            bindings::regulator_status_REGULATOR_STATUS_IDLE => Ok(Self::Idle),
            bindings::regulator_status_REGULATOR_STATUS_STANDBY => Ok(Self::Standby),
            bindings::regulator_status_REGULATOR_STATUS_BYPASS => Ok(Self::Bypass),
            bindings::regulator_status_REGULATOR_STATUS_UNDEFINED => Ok(Self::Undefined),
            _ => Err(EINVAL),
        }
    }
}

impl From<Mode> for Status {
    fn from(mode: Mode) -> Self {
        let status = unsafe { bindings::regulator_mode_to_status(mode as _) };
        if status < 0 {
            return Self::Undefined;
        }

        Self::from_bindings(status as _).unwrap_or(Self::Undefined)
    }
}

#[vtable]
pub trait Operations {
    type Data: ForeignOwnable + Send + Sync = ();

    fn list_voltage(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _selector: u32,
    ) -> Result<i32> {
        Err(ENOTSUPP)
    }

    fn set_voltage(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _min_uv: i32,
        _max_uv: i32,
    ) -> Result<u32> {
        Err(ENOTSUPP)
    }

    fn map_voltage(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _min_uv: i32,
        _max_uv: i32,
    ) -> Result<i32> {
        Err(ENOTSUPP)
    }

    fn set_voltage_sel(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _selector: u32,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn get_voltage(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result<i32> {
        Err(ENOTSUPP)
    }

    fn get_voltage_sel(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result<i32> {
        Err(ENOTSUPP)
    }

    fn set_current_limit(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _min_ua: i32,
        _max_ua: i32,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn get_current_limit(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result<i32> {
        Err(ENOTSUPP)
    }

    fn set_input_current_limit(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _lim_ua: i32,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_over_current_protection(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _lim_ua: i32,
        _severity: i32,
        _enable: bool,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_over_voltage_protection(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _lim_uv: i32,
        _severity: i32,
        _enable: bool,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_under_voltage_protection(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _lim_uv: i32,
        _severity: i32,
        _enable: bool,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_thermal_protection(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _lim: i32,
        _severity: i32,
        _enable: bool,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_active_discharge(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _enable: bool,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn enable(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>, _rdev: &RegulatorDev) -> Result {
        Err(ENOTSUPP)
    }

    fn disable(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn is_enabled(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result<bool> {
        Err(ENOTSUPP)
    }

    fn set_mode(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _mode: Mode,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn get_mode(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>, _rdev: &RegulatorDev) -> Mode {
        Mode::Invalid
    }

    fn get_error_flags(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result<u32> {
        Err(ENOTSUPP)
    }

    fn enable_time(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_ramp_delay(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _ramp_delay: i32,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_voltage_time(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _old_uv: i32,
        _new_uv: i32,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_voltage_time_sel(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _old_selector: u32,
        _new_selector: u32,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_soft_start(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn get_status(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result<Status> {
        Err(ENOTSUPP)
    }

    fn get_optimum_mode(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _input_uv: i32,
        _output_uv: i32,
        _load_ua: i32,
    ) -> Mode {
        Mode::Invalid
    }

    fn set_load(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _load_ua: i32,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_bypass(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _enable: bool,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn get_bypass(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result<bool> {
        Err(ENOTSUPP)
    }

    fn set_suspend_voltage(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _uv: i32,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_suspend_enable(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_suspend_disable(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn set_suspend_mode(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
        _mode: Mode,
    ) -> Result {
        Err(ENOTSUPP)
    }

    fn resume(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>, _rdev: &RegulatorDev) -> Result {
        Err(ENOTSUPP)
    }

    fn set_pull_down(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _rdev: &RegulatorDev,
    ) -> Result {
        Err(ENOTSUPP)
    }
}

pub struct Desc(bindings::regulator_desc);
impl Desc {
    pub const fn new<T: Operations>(name: &'static CStr, reg_type: Type) -> Self {
        let desc = MaybeUninit::<bindings::regulator_desc>::zeroed();
        let mut desc = unsafe { desc.assume_init() };
        desc.name = name.as_char_ptr();
        desc.type_ = reg_type as _;
        desc.ops = unsafe { OperationsVtable::<T>::build() };
        Self(desc)
    }

    pub const fn with_name(mut self, name: &'static CStr) -> Self {
        self.0.name = name.as_char_ptr();
        self
    }

    pub const fn with_enable(mut self, reg: u32, mask: u32, en_val: u32, dis_val: u32) -> Self {
        self.0.enable_reg = reg;
        self.0.enable_mask = mask;
        self.0.enable_val = en_val;
        self.0.disable_val = dis_val;
        self
    }

    pub const fn with_inverted_enable(
        mut self,
        reg: u32,
        mask: u32,
        en_val: u32,
        dis_val: u32,
    ) -> Self {
        self.0.enable_is_inverted = true;
        self.with_enable(reg, mask, en_val, dis_val)
    }

    pub const fn with_active_discharge(mut self, reg: u32, mask: u32, on: u32, off: u32) -> Self {
        self.0.active_discharge_on = on;
        self.0.active_discharge_off = off;
        self.0.active_discharge_reg = reg;
        self.0.active_discharge_mask = mask;
        self
    }

    pub const fn with_csel(mut self, reg: u32, mask: u32, table: &'static [u32]) -> Self {
        self.0.csel_reg = reg;
        self.0.csel_mask = mask;
        self.0.curr_table = table.as_ptr();
        self
    }

    pub const fn with_linear_mapping(
        mut self,
        reg: u32,
        mask: u32,
        min_uv: u32,
        uv_step: u32,
        n_voltages: u32,
        linear_min_sel: u32,
    ) -> Self {
        self.0.vsel_reg = reg;
        self.0.vsel_mask = mask;
        self.0.n_voltages = n_voltages;
        self.0.min_uV = min_uv;
        self.0.uV_step = uv_step;
        self.0.linear_min_sel = linear_min_sel;
        self
    }

    pub const fn with_linear_ranges(
        mut self,
        reg: u32,
        mask: u32,
        ranges: &'static [LinearRange],
    ) -> Self {
        self.0.vsel_reg = reg;
        self.0.vsel_mask = mask;
        self.0.linear_ranges = ranges.as_ptr();
        self.0.n_linear_ranges = ranges.len() as _;
        self
    }

    pub const fn with_owner(mut self, owner: &'static ThisModule) -> Self {
        self.0.owner = owner.to_ptr();
        self
    }
}
unsafe impl Sync for Desc {}

pub struct Config<T: ForeignOwnable + Send + Sync = ()> {
    cfg: bindings::regulator_config,
    data: Option<T>,
}

impl<T: ForeignOwnable + Send + Sync> Config<T> {
    pub fn new(dev: &Device) -> Self {
        Self {
            cfg: bindings::regulator_config {
                dev: dev.raw_device(),
                ..Default::default()
            },
            data: None,
        }
    }

    pub fn with_regmap(mut self, regmap: &Regmap) -> Self {
        self.cfg.regmap = regmap.to_raw();
        self
    }

    pub fn with_drvdata(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    fn into_raw(mut self) -> bindings::regulator_config {
        let data = self.data.take();
        if let Some(data) = data {
            self.cfg.driver_data = data.into_foreign() as _;
        }
        self.cfg
    }
}

pub struct RegulatorDev(*mut bindings::regulator_dev);
impl RegulatorDev {
    pub fn register<T: ForeignOwnable + Send + Sync>(
        dev: &Device,
        desc: &'static Desc,
        cfg: Config<T>,
    ) -> Result<Self> {
        let rdev = from_err_ptr(unsafe {
            bindings::regulator_register(dev.raw_device(), &desc.0, &cfg.into_raw())
        })?;
        Ok(Self(rdev))
    }

    pub fn get_voltage_sel_pickable_regmap(&self) -> Result {
        to_result(unsafe { bindings::regulator_get_voltage_sel_pickable_regmap(self.0) })
    }

    pub fn set_voltage_sel_pickable_regmap(&self, sel: u32) -> Result {
        to_result(unsafe { bindings::regulator_set_voltage_sel_pickable_regmap(self.0, sel) })
    }

    pub fn get_voltage_sel_regmap(&self) -> Result<i32> {
        let ret = unsafe { bindings::regulator_get_voltage_sel_regmap(self.0) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }
        Ok(ret)
    }

    pub fn set_voltage_sel_regmap(&self, sel: u32) -> Result {
        to_result(unsafe { bindings::regulator_set_voltage_sel_regmap(self.0, sel) })
    }

    pub fn is_enabled_regmap(&self) -> Result<bool> {
        let ret = unsafe { bindings::regulator_is_enabled_regmap(self.0) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }
        Ok(ret > 0)
    }

    pub fn enable_regmap(&self) -> Result {
        to_result(unsafe { bindings::regulator_enable_regmap(self.0) })
    }

    pub fn disable_regmap(&self) -> Result {
        to_result(unsafe { bindings::regulator_disable_regmap(self.0) })
    }

    pub fn set_bypass_regmap(&self, enable: bool) -> Result {
        to_result(unsafe { bindings::regulator_set_bypass_regmap(self.0, enable) })
    }

    pub fn get_bypass_regmap(&self) -> Result<bool> {
        let mut enable: bool = false;
        let ret = to_result(unsafe { bindings::regulator_get_bypass_regmap(self.0, &mut enable) });
        ret.map(|_| enable)
    }

    pub fn set_soft_start_regmap(&self) -> Result {
        to_result(unsafe { bindings::regulator_set_soft_start_regmap(self.0) })
    }

    pub fn set_pull_down_regmap(&self) -> Result {
        to_result(unsafe { bindings::regulator_set_pull_down_regmap(self.0) })
    }

    pub fn set_active_discharge_regmap(&self, enable: bool) -> Result {
        to_result(unsafe { bindings::regulator_set_active_discharge_regmap(self.0, enable) })
    }

    pub fn set_current_limit_regmap(&self, min_ua: i32, max_ua: i32) -> Result {
        to_result(unsafe { bindings::regulator_set_current_limit_regmap(self.0, min_ua, max_ua) })
    }

    pub fn get_current_limit_regmap(&self) -> Result<i32> {
        let ret = unsafe { bindings::regulator_get_current_limit_regmap(self.0) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }
        Ok(ret)
    }

    pub fn set_ramp_delay_regmap(&self, ramp_delay: i32) -> Result {
        to_result(unsafe { bindings::regulator_set_ramp_delay_regmap(self.0, ramp_delay) })
    }

    pub fn list_voltage_linear_range(&self, selector: u32) -> Result<i32> {
        let ret = unsafe { bindings::regulator_list_voltage_linear_range(self.0, selector) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }
        Ok(ret)
    }

    pub fn list_voltage_linear(&self, selector: u32) -> Result<i32> {
        let ret = unsafe { bindings::regulator_list_voltage_linear(self.0, selector) };
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }
        Ok(ret)
    }

    pub fn get_name(&self) -> &'static CStr {
        unsafe { CStr::from_char_ptr(bindings::rdev_get_name(self.0)) }
    }

    pub fn get_id(&self) -> i32 {
        unsafe { bindings::rdev_get_id(self.0) }
    }
}

impl Drop for RegulatorDev {
    fn drop(&mut self) {
        unsafe { bindings::regulator_unregister(self.0) }
    }
}

unsafe impl Send for RegulatorDev {}

/// Type of regulator
#[repr(u32)]
pub enum Type {
    /// Voltage regulator
    Voltage = bindings::regulator_type_REGULATOR_VOLTAGE,
    /// Current regulator
    Current = bindings::regulator_type_REGULATOR_CURRENT,
}

pub(crate) struct OperationsVtable<T>(PhantomData<T>);

impl<T: Operations> OperationsVtable<T> {
    unsafe extern "C" fn list_voltage_callback(
        rdev: *mut bindings::regulator_dev,
        selector: core::ffi::c_uint,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| T::list_voltage(data, &rdev, selector))
    }

    unsafe extern "C" fn set_voltage_callback(
        rdev: *mut bindings::regulator_dev,
        min_uv: core::ffi::c_int,
        max_uv: core::ffi::c_int,
        selector: *mut core::ffi::c_uint,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        match T::set_voltage(data, &rdev, min_uv, max_uv) {
            Ok(v) => {
                unsafe { *selector = v };
                0
            }
            Err(e) => e.to_errno(),
        }
    }

    unsafe extern "C" fn map_voltage_callback(
        rdev: *mut bindings::regulator_dev,
        min_uv: core::ffi::c_int,
        max_uv: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| T::map_voltage(data, &rdev, min_uv, max_uv))
    }

    unsafe extern "C" fn set_voltage_sel_callback(
        rdev: *mut bindings::regulator_dev,
        selector: core::ffi::c_uint,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_voltage_sel(data, &rdev, selector)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_voltage_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| T::get_voltage(data, &rdev))
    }

    unsafe extern "C" fn get_voltage_sel_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| T::get_voltage_sel(data, &rdev))
    }

    unsafe extern "C" fn set_current_limit_callback(
        rdev: *mut bindings::regulator_dev,
        min_ua: core::ffi::c_int,
        max_ua: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_current_limit(data, &rdev, min_ua, max_ua)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_current_limit_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| T::get_current_limit(data, &rdev))
    }

    unsafe extern "C" fn set_input_current_limit_callback(
        rdev: *mut bindings::regulator_dev,
        lim_ua: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_input_current_limit(data, &rdev, lim_ua)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_over_current_protection_callback(
        rdev: *mut bindings::regulator_dev,
        lim_ua: core::ffi::c_int,
        severity: core::ffi::c_int,
        enable: bool,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_over_current_protection(data, &rdev, lim_ua, severity, enable)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_over_voltage_protection_callback(
        rdev: *mut bindings::regulator_dev,
        lim_uv: core::ffi::c_int,
        severity: core::ffi::c_int,
        enable: bool,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_over_voltage_protection(data, &rdev, lim_uv, severity, enable)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_under_voltage_protection_callback(
        rdev: *mut bindings::regulator_dev,
        lim_uv: core::ffi::c_int,
        severity: core::ffi::c_int,
        enable: bool,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_under_voltage_protection(data, &rdev, lim_uv, severity, enable)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_thermal_protection_callback(
        rdev: *mut bindings::regulator_dev,
        lim: core::ffi::c_int,
        severity: core::ffi::c_int,
        enable: bool,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_thermal_protection(data, &rdev, lim, severity, enable)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_active_discharge_callback(
        rdev: *mut bindings::regulator_dev,
        enable: bool,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_active_discharge(data, &rdev, enable)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn enable_callback(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::enable(data, &rdev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn disable_callback(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::disable(data, &rdev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn is_enabled_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::is_enabled(data, &rdev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_mode_callback(
        rdev: *mut bindings::regulator_dev,
        mode: core::ffi::c_uint,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            let mode = Mode::from_bindings(mode).unwrap_or(Mode::Invalid);
            T::set_mode(data, &rdev, mode)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_mode_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_uint {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        T::get_mode(data, &rdev) as _
    }

    unsafe extern "C" fn get_error_flags_callback(
        rdev: *mut bindings::regulator_dev,
        flags: *mut core::ffi::c_uint,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        match T::get_error_flags(data, &rdev) {
            Ok(v) => {
                unsafe { *flags = v };
                0
            }
            Err(e) => e.to_errno(),
        }
    }

    unsafe extern "C" fn enable_time_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::enable_time(data, &rdev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_ramp_delay_callback(
        rdev: *mut bindings::regulator_dev,
        ramp_delay: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_ramp_delay(data, &rdev, ramp_delay)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_voltage_time_callback(
        rdev: *mut bindings::regulator_dev,
        old_uv: core::ffi::c_int,
        new_uv: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_voltage_time(data, &rdev, old_uv, new_uv)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_voltage_time_sel_callback(
        rdev: *mut bindings::regulator_dev,
        old_selector: core::ffi::c_uint,
        new_selector: core::ffi::c_uint,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_voltage_time_sel(data, &rdev, old_selector, new_selector)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_soft_start_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_soft_start(data, &rdev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_status_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| Ok(T::get_status(data, &rdev)? as _))
    }

    unsafe extern "C" fn get_optimum_mode_callback(
        rdev: *mut bindings::regulator_dev,
        input_uv: core::ffi::c_int,
        output_uv: core::ffi::c_int,
        load_ua: core::ffi::c_int,
    ) -> core::ffi::c_uint {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        T::get_optimum_mode(data, &rdev, input_uv, output_uv, load_ua) as _
    }

    unsafe extern "C" fn set_load_callback(
        rdev: *mut bindings::regulator_dev,
        load_ua: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_load(data, &rdev, load_ua)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_bypass_callback(
        rdev: *mut bindings::regulator_dev,
        enable: bool,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_bypass(data, &rdev, enable)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_bypass_callback(
        rdev: *mut bindings::regulator_dev,
        enable: *mut bool,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        match T::get_bypass(data, &rdev) {
            Ok(v) => {
                unsafe { *enable = v };
                0
            }
            Err(e) => e.to_errno(),
        }
    }

    unsafe extern "C" fn set_suspend_voltage_callback(
        rdev: *mut bindings::regulator_dev,
        uv: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_suspend_voltage(data, &rdev, uv)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_suspend_enable_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_suspend_enable(data, &rdev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_suspend_disable_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_suspend_disable(data, &rdev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_suspend_mode_callback(
        rdev: *mut bindings::regulator_dev,
        mode: core::ffi::c_uint,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            let mode = Mode::from_bindings(mode).unwrap_or(Mode::Invalid);
            T::set_suspend_mode(data, &rdev, mode)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn resume_callback(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::resume(data, &rdev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_pull_down_callback(
        rdev: *mut bindings::regulator_dev,
    ) -> core::ffi::c_int {
        let rdev = ManuallyDrop::new(RegulatorDev(rdev));
        let data = unsafe { T::Data::borrow(bindings::rdev_get_drvdata(rdev.0)) };
        from_result(|| {
            T::set_pull_down(data, &rdev)?;
            Ok(0)
        })
    }

    const VTABLE: bindings::regulator_ops = bindings::regulator_ops {
        list_voltage: if T::HAS_LIST_VOLTAGE {
            Some(Self::list_voltage_callback)
        } else {
            None
        },
        set_voltage: if T::HAS_SET_VOLTAGE {
            Some(Self::set_voltage_callback)
        } else {
            None
        },
        map_voltage: if T::HAS_MAP_VOLTAGE {
            Some(Self::map_voltage_callback)
        } else {
            None
        },
        set_voltage_sel: if T::HAS_SET_VOLTAGE_SEL {
            Some(Self::set_voltage_sel_callback)
        } else {
            None
        },
        get_voltage: if T::HAS_GET_VOLTAGE {
            Some(Self::get_voltage_callback)
        } else {
            None
        },
        get_voltage_sel: if T::HAS_GET_VOLTAGE_SEL {
            Some(Self::get_voltage_sel_callback)
        } else {
            None
        },
        set_current_limit: if T::HAS_SET_CURRENT_LIMIT {
            Some(Self::set_current_limit_callback)
        } else {
            None
        },
        get_current_limit: if T::HAS_GET_CURRENT_LIMIT {
            Some(Self::get_current_limit_callback)
        } else {
            None
        },
        set_input_current_limit: if T::HAS_SET_INPUT_CURRENT_LIMIT {
            Some(Self::set_input_current_limit_callback)
        } else {
            None
        },
        set_over_current_protection: if T::HAS_SET_OVER_CURRENT_PROTECTION {
            Some(Self::set_over_current_protection_callback)
        } else {
            None
        },
        set_over_voltage_protection: if T::HAS_SET_OVER_VOLTAGE_PROTECTION {
            Some(Self::set_over_voltage_protection_callback)
        } else {
            None
        },
        set_under_voltage_protection: if T::HAS_SET_UNDER_VOLTAGE_PROTECTION {
            Some(Self::set_under_voltage_protection_callback)
        } else {
            None
        },
        set_thermal_protection: if T::HAS_SET_THERMAL_PROTECTION {
            Some(Self::set_thermal_protection_callback)
        } else {
            None
        },
        set_active_discharge: if T::HAS_SET_ACTIVE_DISCHARGE {
            Some(Self::set_active_discharge_callback)
        } else {
            None
        },
        enable: if T::HAS_ENABLE {
            Some(Self::enable_callback)
        } else {
            None
        },
        disable: if T::HAS_DISABLE {
            Some(Self::disable_callback)
        } else {
            None
        },
        is_enabled: if T::HAS_IS_ENABLED {
            Some(Self::is_enabled_callback)
        } else {
            None
        },
        set_mode: if T::HAS_SET_MODE {
            Some(Self::set_mode_callback)
        } else {
            None
        },
        get_mode: if T::HAS_GET_MODE {
            Some(Self::get_mode_callback)
        } else {
            None
        },
        get_error_flags: if T::HAS_GET_ERROR_FLAGS {
            Some(Self::get_error_flags_callback)
        } else {
            None
        },
        enable_time: if T::HAS_ENABLE_TIME {
            Some(Self::enable_time_callback)
        } else {
            None
        },
        set_ramp_delay: if T::HAS_SET_RAMP_DELAY {
            Some(Self::set_ramp_delay_callback)
        } else {
            None
        },
        set_voltage_time: if T::HAS_SET_VOLTAGE_TIME {
            Some(Self::set_voltage_time_callback)
        } else {
            None
        },
        set_voltage_time_sel: if T::HAS_SET_VOLTAGE_TIME_SEL {
            Some(Self::set_voltage_time_sel_callback)
        } else {
            None
        },
        set_soft_start: if T::HAS_SET_SOFT_START {
            Some(Self::set_soft_start_callback)
        } else {
            None
        },
        get_status: if T::HAS_GET_STATUS {
            Some(Self::get_status_callback)
        } else {
            None
        },
        get_optimum_mode: if T::HAS_GET_OPTIMUM_MODE {
            Some(Self::get_optimum_mode_callback)
        } else {
            None
        },
        set_load: if T::HAS_SET_LOAD {
            Some(Self::set_load_callback)
        } else {
            None
        },
        set_bypass: if T::HAS_SET_BYPASS {
            Some(Self::set_bypass_callback)
        } else {
            None
        },
        get_bypass: if T::HAS_GET_BYPASS {
            Some(Self::get_bypass_callback)
        } else {
            None
        },
        set_suspend_voltage: if T::HAS_SET_SUSPEND_VOLTAGE {
            Some(Self::set_suspend_voltage_callback)
        } else {
            None
        },
        set_suspend_enable: if T::HAS_SET_SUSPEND_ENABLE {
            Some(Self::set_suspend_enable_callback)
        } else {
            None
        },
        set_suspend_disable: if T::HAS_SET_SUSPEND_DISABLE {
            Some(Self::set_suspend_disable_callback)
        } else {
            None
        },
        set_suspend_mode: if T::HAS_SET_SUSPEND_MODE {
            Some(Self::set_suspend_mode_callback)
        } else {
            None
        },
        resume: if T::HAS_RESUME {
            Some(Self::resume_callback)
        } else {
            None
        },
        set_pull_down: if T::HAS_SET_PULL_DOWN {
            Some(Self::set_pull_down_callback)
        } else {
            None
        },
    };

    pub(crate) const unsafe fn build() -> &'static bindings::regulator_ops {
        &Self::VTABLE
    }
}
