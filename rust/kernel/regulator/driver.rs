use core::{
    marker::PhantomData,
    mem::MaybeUninit,
};
use crate::{
    device::{Device, RawDevice},
    error::{code::*, from_err_ptr, from_result, Result},
    macros::vtable, str::CStr,
    regulator::Mode,
    //regmap::Regmap,
};

pub type LinearRange = bindings::linear_range;

#[vtable]
pub trait Operations {
    fn list_voltage(selector: u32) -> Result<i32> {
        Err(ENOTSUPP)
    }

    fn set_voltage(min_uv: i32, max_uv: i32) -> Result<i32> {
        Err(ENOTSUPP)
    }

    fn map_voltage(min_uv: i32, max_uv: i32) -> Result<i32> {
        Err(ENOTSUPP)
    }

    fn set_voltage_sel(selector: u32) -> Result {
        Err(ENOTSUPP)
    }

    fn get_voltage() -> Result<i32> {
		Err(ENOTSUPP)
    }

    fn get_voltage_sel() -> Result<i32> {
		Err(ENOTSUPP)
    }

	fn set_current_limit(min_ua: i32, max_ua: i32) -> Result {
		Err(ENOTSUPP)
	}

	fn get_current_limit() -> Result<i32> {
		Err(ENOTSUPP)
	}

	fn set_input_current_limit(lim_ua: i32) -> Result {
		Err(ENOTSUPP)
	}

	fn set_over_current_protection(lim_ua: i32, severity: i32, enable: bool) -> Result {
		Err(ENOTSUPP)
	}

	fn set_over_voltage_protection(lim_uv: i32, severity: i32, enable: bool) -> Result {
		Err(ENOTSUPP)
	}

	fn set_under_voltage_protection(lim_uv: i32, severity: i32, enable: bool) -> Result {
		Err(ENOTSUPP)
	}

	fn set_thermal_protection(lim: i32, severity: i32, enable: bool) -> Result {
		Err(ENOTSUPP)
	}

	fn set_active_discharge(enable: bool) -> Result {
		Err(ENOTSUPP)
	}

	fn enable() -> Result {
		Err(ENOTSUPP)
	}

	fn disable() -> Result {
		Err(ENOTSUPP)
	}

	fn is_enabled() -> Result<bool> {
		Err(ENOTSUPP)
	}

	fn set_mode(mode: Mode) -> Result {
		Err(ENOTSUPP)
	}

	fn get_mode() -> Mode {
		Mode::Invalid
    }

	fn get_error_flags() -> Result<u32> {
		Err(ENOTSUPP)
    }

	fn enable_time() -> Result {
		Err(ENOTSUPP)
	}

	fn set_ramp_delay(ramp_delay: i32) -> Result {
		Err(ENOTSUPP)
	}

	fn set_voltage_time(old_uv: i32, new_uv: i32) -> Result {
		Err(ENOTSUPP)
	}

	fn set_voltage_time_sel(old_selector: u32, new_selector: u32) -> Result {
		Err(ENOTSUPP)
	}

	fn set_soft_start() -> Result {
		Err(ENOTSUPP)
	}

	fn get_status() -> Result {
		Err(ENOTSUPP)
	}

	fn get_optimum_mode(input_uv: i32, output_uv: i32, load_ua: i32) -> Result<u32> {
		Err(ENOTSUPP)
    }

	fn set_load(load_ua: i32) -> Result {
		Err(ENOTSUPP)
	}

	fn set_bypass(enable: bool) -> Result {
		Err(ENOTSUPP)
	}

	fn get_bypass() -> Result<bool> {
		Err(ENOTSUPP)
    }

	fn set_suspend_voltage(uv: i32) -> Result {
		Err(ENOTSUPP)
	}

	fn set_suspend_enable() -> Result {
		Err(ENOTSUPP)
	}

	fn set_suspend_disable() -> Result {
		Err(ENOTSUPP)
	}

	fn set_suspend_mode(mode: u32) -> Result {
		Err(ENOTSUPP)
	}

	fn resume() -> Result {
		Err(ENOTSUPP)
	}

	fn set_pull_down() -> Result {
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

    pub const fn with_inverted_enable(mut self, reg: u32, mask: u32, en_val: u32, dis_val: u32) -> Self {
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

    pub const fn with_linear_ranges(mut self, reg: u32, mask: u32, ranges: &'static [LinearRange]) -> Self {
        self.0.vsel_reg = reg;
        self.0.vsel_mask = mask;
        self.0.linear_ranges = ranges.as_ptr();
        self.0.n_linear_ranges = ranges.len() as _;
        self.0.n_voltages = mask + 1; // FIXME
        self
    }
}

pub struct Config<T = ()> {
    cfg: bindings::regulator_config,
    _phantom: PhantomData<T>,
}

impl<T> Config<T> {
    pub fn new(dev: &Device) -> Self {
        Self {
            cfg: bindings::regulator_config {
                dev: dev.raw_device(),
                ..Default::default()
            },
            _phantom: PhantomData,
        }
    }

    /*
    pub fn with_regmap(mut self, regmap: &Regmap) -> Self {
       self 
    }
    */
}

pub struct RegulatorDev(*mut bindings::regulator_dev);
impl RegulatorDev {
    pub fn register<T>(dev: &Device, desc: &'static Desc, cfg: &Config<T>) -> Result<Self> {
        let rdev = from_err_ptr(unsafe {
            bindings::regulator_register(dev.raw_device(), &desc.0, &cfg.cfg)
        })?;
        Ok(Self(rdev))
    }
}

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
    unsafe extern "C" fn list_voltage(
        rdev: *mut bindings::regulator_dev,
        selector: core::ffi::c_uint,
    ) -> core::ffi::c_int {
        from_result(|| T::list_voltage(selector))
    }

    unsafe extern "C" fn map_voltage(
        rdev: *mut bindings::regulator_dev,
        min_uv: core::ffi::c_int,
        max_uv: core::ffi::c_int,
    ) -> core::ffi::c_int {
        from_result(|| T::map_voltage(min_uv, max_uv))
    }

    unsafe extern "C" fn set_voltage_sel(
        rdev: *mut bindings::regulator_dev,
        selector: core::ffi::c_uint,
    ) -> core::ffi::c_int {
        from_result(|| {
            T::set_voltage_sel(selector)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_voltage_sel(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
        from_result(|| T::get_voltage_sel())
    }

    unsafe extern "C" fn set_current_limit(
        rdev: *mut bindings::regulator_dev,
        min_ua: core::ffi::c_int,
        max_ua: core::ffi::c_int,
    ) -> core::ffi::c_int {
        from_result(|| {
            T::set_current_limit(min_ua, max_ua)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_current_limit(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
        from_result(|| T::get_current_limit())
    }

    unsafe extern "C" fn set_active_discharge(rdev: *mut bindings::regulator_dev, enable: bool) -> core::ffi::c_int {
        from_result(|| {
            T::set_active_discharge(enable)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn enable(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
        from_result(|| {
            T::enable()?;
            Ok(0)
        })
    }

    unsafe extern "C" fn disable(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
        from_result(|| {
            T::disable()?;
            Ok(0)
        })
    }

    unsafe extern "C" fn is_enabled(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
        from_result(|| {
            T::is_enabled()?;
            Ok(0)
        })
    }

    unsafe extern "C" fn set_mode(rdev: *mut bindings::regulator_dev, mode: core::ffi::c_uint) -> core::ffi::c_int {
        from_result(|| { 
            let mode = Mode::from_bindings(mode).unwrap_or(Mode::Invalid);
            T::set_mode(mode)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn get_mode(rdev: *mut bindings::regulator_dev) -> core::ffi::c_uint {
        T::get_mode() as _
    }

    /*
    
    unsafe extern "C" fn set_voltage(
        rdev: *mut bindings::regulator_dev,
        min_uV: core::ffi::c_int,
        max_uV: core::ffi::c_int,
        selector: *mut core::ffi::c_uint,
    ) -> core::ffi::c_int {
    }

    unsafe extern "C" fn get_voltage(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int {
    }
    unsafe extern "C" fn set_input_current_limit(
        rdev: *mut bindings::regulator_dev,
        lim_uA: core::ffi::c_int,
    ) -> core::ffi::c_int {
    }

    unsafe extern "C" fn set_over_current_protection(
        rdev: *mut bindings::regulator_dev,
        lim_uA: core::ffi::c_int,
        severity: core::ffi::c_int,
        enable: bool_,
    ) -> core::ffi::c_int {
    }

    unsafe extern "C" fn set_over_voltage_protection(
        rdev: *mut bindings::regulator_dev,
        lim_uV: core::ffi::c_int,
        severity: core::ffi::c_int,
        enable: bool_,
    ) -> core::ffi::c_int {
    }

    unsafe extern "C" fn set_under_voltage_protection(
        rdev: *mut bindings::regulator_dev,
        lim_uV: core::ffi::c_int,
        severity: core::ffi::c_int,
        enable: bool_,
    ) -> core::ffi::c_int {
    }

    unsafe extern "C" fn set_thermal_protection(
        rdev: *mut bindings::regulator_dev,
        lim: core::ffi::c_int,
        severity: core::ffi::c_int,
        enable: bool_,
    ) -> core::ffi::c_int {
    }
    unsafe extern "C" fn get_error_flags(
        rdev: *mut bindings::regulator_dev,
        flags: *mut core::ffi::c_uint,
    ) -> core::ffi::c_int {
        0
    }
    pub enable_time:
        ::core::option::Option<unsafe extern "C" fn(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int>,
    pub set_ramp_delay: ::core::option::Option<
        unsafe extern "C" fn(
            rdev: *mut bindings::regulator_dev,
            ramp_delay: core::ffi::c_int,
        ) -> core::ffi::c_int,
    >,
    pub set_voltage_time: ::core::option::Option<
        unsafe extern "C" fn(
            rdev: *mut bindings::regulator_dev,
            old_uV: core::ffi::c_int,
            new_uV: core::ffi::c_int,
        ) -> core::ffi::c_int,
    >,
    pub set_voltage_time_sel: ::core::option::Option<
        unsafe extern "C" fn(
            rdev: *mut bindings::regulator_dev,
            old_selector: core::ffi::c_uint,
            new_selector: core::ffi::c_uint,
        ) -> core::ffi::c_int,
    >,
    pub set_soft_start:
        ::core::option::Option<unsafe extern "C" fn(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int>,
    pub get_status:
        ::core::option::Option<unsafe extern "C" fn(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int>,
    pub get_optimum_mode: ::core::option::Option<
        unsafe extern "C" fn(
            rdev: *mut bindings::regulator_dev,
            input_uV: core::ffi::c_int,
            output_uV: core::ffi::c_int,
            load_uA: core::ffi::c_int,
        ) -> core::ffi::c_uint,
    >,
    pub set_load: ::core::option::Option<
        unsafe extern "C" fn(
            rdev: *mut bindings::regulator_dev,
            load_uA: core::ffi::c_int,
        ) -> core::ffi::c_int,
    >,
    pub set_bypass: ::core::option::Option<
        unsafe extern "C" fn(dev: *mut bindings::regulator_dev, enable: bool_) -> core::ffi::c_int,
    >,
    pub get_bypass: ::core::option::Option<
        unsafe extern "C" fn(dev: *mut bindings::regulator_dev, enable: *mut bool_) -> core::ffi::c_int,
    >,
    pub set_suspend_voltage: ::core::option::Option<
        unsafe extern "C" fn(rdev: *mut bindings::regulator_dev, uV: core::ffi::c_int) -> core::ffi::c_int,
    >,
    pub set_suspend_enable:
        ::core::option::Option<unsafe extern "C" fn(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int>,
    pub set_suspend_disable:
        ::core::option::Option<unsafe extern "C" fn(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int>,
    pub set_suspend_mode: ::core::option::Option<
        unsafe extern "C" fn(rdev: *mut bindings::regulator_dev, mode: core::ffi::c_uint) -> core::ffi::c_int,
    >,
    pub resume:
        ::core::option::Option<unsafe extern "C" fn(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int>,
    pub set_pull_down:
        ::core::option::Option<unsafe extern "C" fn(rdev: *mut bindings::regulator_dev) -> core::ffi::c_int>,

	list_voltage: None,

	/* get/set regulator voltage */
	int (*set_voltage) (struct regulator_dev *, int min_uV, int max_uV,
			    unsigned *selector);
	map_voltage: None,
	set_voltage_sel: None,
	get_voltage: None,
	get_voltage_sel: None,

	/* get/set regulator current  */
	int (*set_current_limit) (struct regulator_dev *,
				 int min_uA, int max_uA);
	get_current_limit: None,

	set_input_current_limit: None,
	int (*set_over_current_protection)(struct regulator_dev *, int lim_uA,
					   int severity, bool enable);
	int (*set_over_voltage_protection)(struct regulator_dev *, int lim_uV,
					   int severity, bool enable);
	int (*set_under_voltage_protection)(struct regulator_dev *, int lim_uV,
					    int severity, bool enable);
	int (*set_thermal_protection)(struct regulator_dev *, int lim,
				      int severity, bool enable);
	set_active_discharge: None,

	/* enable/disable regulator */
	enable: None,
	disable: None,
	is_enabled: None,

	/* get/set regulator operating mode (defined in consumer.h) */
	set_mode: None,
	unsigned get_mode: None,

	/* retrieve current error flags on the regulator */
	get_error_flags: None,

	/* Time taken to enable or set voltage on the regulator */
	enable_time: None,
	set_ramp_delay: None,
	int (*set_voltage_time) (struct regulator_dev *, int old_uV,
				 int new_uV);
	int (*set_voltage_time_sel) (struct regulator_dev *,
				     unsigned int old_selector,
				     unsigned int new_selector);

	set_soft_start: None,

	/* report regulator status ... most other accessors report
	 * control inputs, this reports results of combining inputs
	 * from Linux (and other sources) with the actual load.
	 * returns REGULATOR_STATUS_* or negative errno.
	 */
	get_status: None,

	/* get most efficient regulator operating mode for load */
	unsigned int (*get_optimum_mode) (struct regulator_dev *, int input_uV,
					  int output_uV, int load_uA);
	/* set the load on the regulator */
	set_load: None,

	/* control and report on bypass mode */
	set_bypass: None,
	get_bypass: None,

	/* the operations below are for configuration of regulator state when
	 * its parent PMIC enters a global STANDBY/HIBERNATE state */

	/* set regulator suspend voltage */
	set_suspend_voltage: None,

	/* enable/disable regulator in suspend state */
	set_suspend_enable: None,
	set_suspend_disable: None,

	/* set regulator suspend operating mode (defined in consumer.h) */
	set_suspend_mode: None,

	resume: None,

	set_pull_down: None,
    */
    const VTABLE: bindings::regulator_ops = bindings::regulator_ops {
        list_voltage: None,
        set_voltage: None,
        map_voltage: None,
        set_voltage_sel: None,
        get_voltage: None,
        get_voltage_sel: None,
        set_current_limit: None,
        get_current_limit: None,
        set_input_current_limit: None,
        set_over_current_protection: None,
        set_over_voltage_protection: None,
        set_under_voltage_protection: None,
        set_thermal_protection: None,
        set_active_discharge: None,
        enable: None,
        disable: None,
        is_enabled: None,
        set_mode: None,
        get_mode: None,
        get_error_flags: None,
        enable_time: None,
        set_ramp_delay: None,
        set_voltage_time: None,
        set_voltage_time_sel: None,
        set_soft_start: None,
        get_status: None,
        get_optimum_mode: None,
        set_load: None,
        set_bypass: None,
        get_bypass: None,
        set_suspend_voltage: None,
        set_suspend_enable: None,
        set_suspend_disable: None,
        set_suspend_mode: None,
        resume: None,
        set_pull_down: None,
    };

    pub(crate) const unsafe fn build() -> &'static bindings::regulator_ops {
        &Self::VTABLE
    }
}
