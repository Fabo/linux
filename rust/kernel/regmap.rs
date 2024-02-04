/// TODO:
///  * Assert if lsb > msb
///  * Assert if msb >= 32

#[cfg(CONFIG_I2C)]
use crate::i2c;
use crate::{
    bindings,
    error::{to_result, Result},
    macros::paste,
    str::CStr,
};
use core::{marker::PhantomData, mem::MaybeUninit};

/// Register Endianness
#[repr(u32)]
pub enum Endian {
    Default = bindings::regmap_endian_REGMAP_ENDIAN_DEFAULT,
    /// Big Endian
    Big = bindings::regmap_endian_REGMAP_ENDIAN_BIG,
    /// Little Endian
    Little = bindings::regmap_endian_REGMAP_ENDIAN_LITTLE,
    /// System's native Endian
    Native = bindings::regmap_endian_REGMAP_ENDIAN_NATIVE,
}

/// Type of caching
#[repr(u32)]
pub enum CacheType {
    /// Don't cache anything
    None = bindings::regcache_type_REGCACHE_NONE,
    /// Use RbTree caching
    RbTree = bindings::regcache_type_REGCACHE_RBTREE,
    /// Use Flat caching
    Flat = bindings::regcache_type_REGCACHE_FLAT,
    /// Use Maple caching
    Maple = bindings::regcache_type_REGCACHE_MAPLE,
}

pub struct Regmap {
    ptr: *mut bindings::regmap,
}
impl Regmap {
    #[cfg(CONFIG_I2C)]
    pub fn init_i2c<T: ConfigOps>(i2c: &i2c::Client, config: &Config<T>) -> Self {
        let regmap = unsafe { bindings::regmap_init_i2c(i2c.raw_client(), &config.raw) };

        Self { ptr: regmap }
    }

    pub fn alloc_fields<const N: usize>(
        &mut self,
        reg_fields: &'static RegFields<N>,
    ) -> Result<Fields<N>> {
        let mut rm_fields = [core::ptr::null_mut(); N];
        to_result(unsafe {
            bindings::regmap_field_bulk_alloc(
                self.ptr,
                &mut rm_fields[0],
                reg_fields.0.as_ptr(),
                reg_fields.0.len() as i32,
            )
        })?;

        Ok(Fields {
            rm_fields,
            _reg_fields: reg_fields,
        })
    }

    pub fn to_raw(&self) -> *mut bindings::regmap {
        self.ptr
    }
}

pub struct RegFields<const N: usize>([bindings::reg_field; N]);

impl<const N: usize> RegFields<N> {
    pub const fn new(fields: [bindings::reg_field; N]) -> Self {
        Self(fields)
    }

    pub const fn count(&self) -> usize {
        N
    }
}

pub struct Fields<const N: usize> {
    rm_fields: [*mut bindings::regmap_field; N],
    _reg_fields: &'static RegFields<N>,
}
impl<const N: usize> Fields<N> {
    pub fn index(&mut self, index: usize) -> *mut bindings::regmap_field {
        self.rm_fields[index]
    }
}

unsafe impl<const N: usize> Send for Fields<N> {}

/// Helper macro for [`Config`] to create methods to set a fields from [`regmap_config`]
///
/// The following code will create a method named `with_max_register`:
/// ```
/// config_with!(max_register: u32);
/// ```
macro_rules! config_with {
    ($name:ident: $type:ty) => {
        config_with!($name: $type, $name);
    };

    ($name:ident: $type:ty, $e:expr) => {
        paste! {
            pub const fn [<with_$name>](mut self, $name: $type) -> Self {
                self.raw.$name = $e;
                self
            }
        }
    };
}

pub trait ConfigOps {
    fn is_readable_reg(reg: u32) -> bool;
    fn is_writeable_reg(reg: u32) -> bool;
    fn is_volatile_reg(reg: u32) -> bool;
    fn is_precious_reg(reg: u32) -> bool;
    fn is_readable_noinc_reg(reg: u32) -> bool;
    fn is_writeable_noinc_reg(reg: u32) -> bool;
}

pub struct Config<T: ConfigOps> {
    raw: bindings::regmap_config,
    _phantom: PhantomData<T>,
}
impl<T: ConfigOps> Config<T> {
    pub const fn new(reg_bits: i32, val_bits: i32) -> Self {
        let cfg = MaybeUninit::<bindings::regmap_config>::zeroed();
        let mut cfg = unsafe { cfg.assume_init() };

        cfg.reg_bits = reg_bits;
        cfg.val_bits = val_bits;
        cfg.writeable_reg = Some(Self::writeable_reg_callback);
        cfg.readable_reg = Some(Self::readable_reg_callback);
        cfg.volatile_reg = Some(Self::volatile_reg_callback);
        cfg.precious_reg = Some(Self::precious_reg_callback);
        cfg.writeable_noinc_reg = Some(Self::writeable_noinc_reg_callback);
        cfg.readable_noinc_reg = Some(Self::readable_noinc_reg_callback);

        Self {
            raw: cfg,
            _phantom: PhantomData,
        }
    }

    config_with!(can_sleep: bool);

    config_with!(name: &'static CStr, name.as_char_ptr());

    config_with!(max_register: u32);
    config_with!(pad_bits: i32);
    config_with!(reg_base: u32);
    config_with!(reg_stride: i32);

    config_with!(can_multi_write: bool);
    config_with!(use_single_read: bool);
    config_with!(use_single_write: bool);
    config_with!(use_relaxed_mmio: bool);

    config_with!(fast_io: bool);
    config_with!(io_port: bool);

    config_with!(read_flag_mask: core::ffi::c_ulong);
    config_with!(write_flag_mask: core::ffi::c_ulong);
    config_with!(zero_flag_mask: bool);

    config_with!(max_raw_read: usize);
    config_with!(max_raw_write: usize);

    config_with!(reg_format_endian: Endian, reg_format_endian as _);
    config_with!(val_format_endian: Endian, val_format_endian as _);

    config_with!(cache_type: CacheType, cache_type as _);

    /*
    const struct regmap_range_cfg *ranges;
    unsigned int num_ranges;

    const struct reg_default *reg_defaults;
    unsigned int num_reg_defaults;
    */

    unsafe extern "C" fn writeable_reg_callback(_dev: *mut bindings::device, reg: u32) -> bool {
        T::is_writeable_reg(reg)
    }

    unsafe extern "C" fn readable_reg_callback(_dev: *mut bindings::device, reg: u32) -> bool {
        T::is_readable_reg(reg)
    }

    unsafe extern "C" fn volatile_reg_callback(_dev: *mut bindings::device, reg: u32) -> bool {
        T::is_volatile_reg(reg)
    }

    unsafe extern "C" fn precious_reg_callback(_dev: *mut bindings::device, reg: u32) -> bool {
        T::is_precious_reg(reg)
    }

    unsafe extern "C" fn writeable_noinc_reg_callback(
        _dev: *mut bindings::device,
        reg: u32,
    ) -> bool {
        T::is_writeable_noinc_reg(reg)
    }

    unsafe extern "C" fn readable_noinc_reg_callback(_dev: *mut bindings::device, reg: u32) -> bool {
        T::is_readable_noinc_reg(reg)
    }
}

pub mod access {
    pub const READ: u32 = 0b000001;
    pub const WRITE: u32 = 0b000010;
    pub const PRECIOUS: u32 = 0b000100;
    pub const VOLATILE: u32 = 0b001000;
    pub const WRITE_NO_INC: u32 = 0b010000;
    pub const READ_NO_INC: u32 = 0b100000;

    /* Helper */
    pub const RW: u32 = READ | WRITE;
}

#[macro_export]
macro_rules! check_access {
    ($type:ident, $access:expr, $reg:ident, $addr:literal) => {
        if kernel::regmap::access::$type & $access > 0 && $reg == $addr {
            return true;
        }
    };
}
pub use check_access;

#[macro_export]
macro_rules! field_bit {
    ($field_name:ident, $reg:literal, $pos:literal, rw) => {
        $crate::field_bit!($field_name, $reg, $pos, reserved);
        $crate::field_bit!($field_name, _ro);
        $crate::field_bit!($field_name, _wo);
    };

    ($field_name:ident, $reg:literal, $pos:literal, ro) => {
        $crate::field_bit!($field_name, $reg, $pos, reserved);
        $crate::field_bit!($field_name, _ro);
    };

    ($field_name:ident, $reg:literal, $pos:literal, wo) => {
        $crate::field_bit!($field_name, $reg, $pos, reserved);
        $crate::field_bit!($field_name, _wo);
    };

    ($field_name:ident, $reg:literal, $pos:literal, reserved) => {
        kernel::macros::paste! {
            struct [<_Bit $pos >];
        }

        impl $field_name {
            pub const fn reg_field() -> bindings::reg_field {
                bindings::reg_field {
                    reg: $reg,
                    lsb: $pos,
                    msb: $pos + 1,
                    id_offset: 0,
                    id_size: 0,
                }
            }

            #[allow(dead_code)]
            pub const fn mask() -> u32 {
                kernel::bits::genmask($pos, $pos)
            }
        }
    };

    ($field_name:ident, _ro) => {
        impl $field_name {
            #[allow(dead_code)]
            pub fn is_set<const N: usize>(fields: &mut regmap::Fields<N>) -> Result<bool> {
                let field = unsafe { fields.index(Self::id() as usize) };
                let mut val: core::ffi::c_uint = 0;
                kernel::error::to_result(unsafe { bindings::regmap_field_read(field, &mut val) })?;
                Ok(val == 1)
            }
        }
    };

    ($field_name:ident, _wo) => {
        impl $field_name {
            #[allow(dead_code)]
            pub fn set<const N: usize>(fields: &mut regmap::Fields<N>) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_write(field, 1) })
            }

            #[allow(dead_code)]
            pub fn force_set<const N: usize>(fields: &mut regmap::Fields<N>) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_force_write(field, 1) })
            }

            #[allow(dead_code)]
            pub fn clear<const N: usize>(fields: &mut regmap::Fields<N>) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_write(field, 0) })
            }

            #[allow(dead_code)]
            pub fn force_clear<const N: usize>(fields: &mut regmap::Fields<N>) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_force_write(field, 0) })
            }
        }
    };
}

#[macro_export]
macro_rules! field_enum {
    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], ro, {
        $($k:ident = $v:literal,)+ }) => {
        $crate::field_enum!($field_name, $reg, [$msb:$lsb], reserved, { $($k = $v,)+ });
        $crate::field_enum!($field_name, _ro);
    };

    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], rw, {
        $($k:ident = $v:literal,)+ }) => {
        $crate::field_enum!($field_name, $reg, [$msb:$lsb], reserved, { $($k = $v,)+ });
        $crate::field_enum!($field_name, _ro);
        $crate::field_enum!($field_name, _wo);
    };

    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], wo, {
        $($k:ident = $v:literal,)+ }) => {
        $crate::field_enum!($field_name, $reg, [$msb:$lsb], reserved, { $($k = $v,)+ });
        $crate::field_enum!($field_name, _wo);
    };

    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], reserved, {
        $($k:ident = $v:literal,)+ }) => {
        kernel::macros::seq!(i in $lsb..=$msb {
            kernel::macros::paste! {
                struct [<_Bit $i>];
            }
        });

        kernel::macros::paste! {
            #[repr(u32)]
            #[allow(non_camel_case_types)]
            pub enum [<$field_name _enum>] {
                $($k = $v,)+
            }

            impl TryFrom<core::ffi::c_uint> for [<$field_name _enum>] {
                type Error = kernel::error::Error;

                fn try_from(raw_value: core::ffi::c_uint) -> Result<Self> {
                    match raw_value {
                        $($v => Ok(Self::$k),)+
                        _ => Err(kernel::error::code::EINVAL),
                    }
                }
            }

            impl $field_name {
                pub const fn reg_field() -> bindings::reg_field {
                    bindings::reg_field {
                        reg: $reg,
                        lsb: $lsb,
                        msb: $msb,
                        id_offset: 0,
                        id_size: 0,
                    }
                }

                #[allow(dead_code)]
                pub const fn mask() -> u32 {
                    kernel::bits::genmask($msb, $lsb)
                }
            }
        }
    };

    ($field_name:ident, _ro) => {
        kernel::macros::paste! {
            impl $field_name {
                #[allow(dead_code)]
                pub fn read<const N: usize>(fields: &mut regmap::Fields<N>) -> Result<[<$field_name _enum>]> {
                    let field = unsafe { fields.index(Self::id() as usize) };
                    let mut val = 0;
                    let res = unsafe { bindings::regmap_field_read(field, &mut val) };
                    if res < 0 {
                        return Err(kernel::error::Error::from_errno(res));
                    }

                    [<$field_name _enum>]::try_from(val)
                }
            }
        }
    };

    ($field_name:ident, _wo) => {
        kernel::macros::paste! {
            impl $field_name {
                #[allow(dead_code)]
                pub fn write<const N: usize>(fields: &mut regmap::Fields<N>, val: [<$field_name _enum>]) -> Result {
                    let field = unsafe { fields.index(Self::id() as usize) };
                    kernel::error::to_result(unsafe { bindings::regmap_field_write(field, val as _) })
                }

                #[allow(dead_code)]
                pub fn force_write<const N: usize>(fields: &mut regmap::Fields<N>, val: [<$field_name _enum>]) -> Result {
                    let field = unsafe { fields.index(Self::id() as usize) };
                    kernel::error::to_result(unsafe { bindings::regmap_field_force_write(field, val as _) })
                }
            }
        }
    };
}

#[macro_export]
macro_rules! field_raw {
    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], rw) => {
        $crate::field_raw!($field_name, $reg, [$msb:$lsb], reserved);
        $crate::field_raw!($field_name, $reg, [$msb:$lsb], _ro);
        $crate::field_raw!($field_name, $reg, [$msb:$lsb], _wo);
    };

    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], ro) => {
        $crate::field_raw!($field_name, $reg, [$msb:$lsb], reserved);
        $crate::field_raw!($field_name, $reg, [$msb:$lsb], _ro);
    };

    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], wo) => {
        $crate::field_raw!($field_name, $reg, [$msb:$lsb], reserved);
        $crate::field_raw!($field_name, $reg, [$msb:$lsb], _wo);
    };

    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], reserved) => {
        kernel::macros::seq!(i in $lsb..=$msb {
            kernel::macros::paste! {
                struct [<_Bit $i>];
            }
        });

        impl $field_name {
            pub const fn reg_field() -> bindings::reg_field {
                bindings::reg_field {
                    reg: $reg,
                    lsb: $lsb,
                    msb: $msb,
                    id_offset: 0,
                    id_size: 0,
                }
            }

            #[allow(dead_code)]
            pub const fn mask() -> u32 {
                kernel::bits::genmask($msb, $lsb)
            }
        }
    };

    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], _ro) => {
        impl $field_name {
            #[allow(dead_code)]
            pub fn read<const N: usize>(
                fields: &mut regmap::Fields<N>,
            ) -> Result<core::ffi::c_uint> {
                let field = unsafe { fields.index(Self::id() as usize) };
                let mut val = 0;
                let res = unsafe { bindings::regmap_field_read(field, &mut val) };
                if res < 0 {
                    return Err(kernel::error::Error::from_errno(res));
                }

                Ok(val)
            }

            #[allow(dead_code)]
            pub fn test_bits<const N: usize>(
                fields: &mut regmap::Fields<N>,
                bits: core::ffi::c_uint,
            ) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_test_bits(field, bits) })
            }
        }
    };

    ($field_name:ident, $reg:literal, [$msb:literal:$lsb:literal], _wo) => {
        impl $field_name {
            #[allow(dead_code)]
            pub fn write<const N: usize>(
                fields: &mut regmap::Fields<N>,
                val: core::ffi::c_uint,
            ) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_write(field, val as _) })
            }

            #[allow(dead_code)]
            pub fn force_write<const N: usize>(
                fields: &mut regmap::Fields<N>,
                val: core::ffi::c_uint,
            ) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe {
                    bindings::regmap_field_force_write(field, val as _)
                })
            }

            #[allow(dead_code)]
            pub fn update_bits<const N: usize>(
                fields: &mut regmap::Fields<N>,
                mask: core::ffi::c_uint,
                val: core::ffi::c_uint,
            ) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe {
                    bindings::regmap_field_update_bits(field, mask, val)
                })
            }

            #[allow(dead_code)]
            pub fn force_update_bits<const N: usize>(
                fields: &mut regmap::Fields<N>,
                mask: core::ffi::c_uint,
                val: core::ffi::c_uint,
            ) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe {
                    bindings::regmap_field_force_update_bits(field, mask, val)
                })
            }

            #[allow(dead_code)]
            pub fn set_bits<const N: usize>(
                fields: &mut regmap::Fields<N>,
                bits: core::ffi::c_uint,
            ) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_set_bits(field, bits) })
            }

            #[allow(dead_code)]
            pub fn clear_bits<const N: usize>(
                fields: &mut regmap::Fields<N>,
                bits: core::ffi::c_uint,
            ) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_clear_bits(field, bits) })
            }
        }
    };
}

#[macro_export]
macro_rules! fields {
    ($type:ident, $reg:ident, $name:ident, $($t:tt)*) => {
        kernel::macros::paste! {
            #[allow(non_camel_case_types)]
            pub struct $name;

            impl $name {
                #[allow(dead_code)]
                pub const fn id() -> super::Fields {
                    super::Fields::[<$reg _ $name>]
                }
            }

            $crate::[<field_ $type>]!($name, $($t)*);
        }
    };
}

#[macro_export]
macro_rules! reg_field {
    ($reg_name:ident, $field_name:ident) => {
        register::$reg_name::$field_name::reg_field()
    };
}

#[macro_export]
macro_rules! count_fields {
    () => { 0usize };
    ($type:ident $($rhs:ident)*) => { 1 + $crate::count_fields!($($rhs)*) };
}

#[macro_export]
macro_rules! registers {
    ($name:ident, {
        $( {
            $reg_name:ident, $reg_addr:literal, $access:expr, {
                $($field_name:ident => $type:ident($($x:tt),*)),*,
            }
        }),+
    }) => {
        mod register {
            use kernel::regmap::ConfigOps;

            kernel::macros::paste! {
                $(
                    pub mod $reg_name {
                        use kernel::{bindings, error::{Result}, regmap};
                        $( $crate::fields!($type, $reg_name, $field_name, $reg_addr, $($x),*); )*

                        #[allow(dead_code)]
                        pub const fn addr() -> u32 {
                            $reg_addr
                        }
                    }
                )+

                #[repr(u32)]
                #[allow(non_camel_case_types)]
                pub enum Fields {
                    $($(
                        [<$reg_name _ $field_name>],
                    )*)+
                }

                pub struct AccessOps;
                impl ConfigOps for AccessOps {
                    fn is_readable_reg(reg: u32) -> bool {
                        $(
                            kernel::regmap::check_access!(READ, $access, reg, $reg_addr);
                        )+

                        false
                    }

                    fn is_writeable_reg(reg: u32) -> bool {
                        $(
                            kernel::regmap::check_access!(WRITE, $access, reg, $reg_addr);
                        )+

                        false
                    }

                    fn is_volatile_reg(reg: u32) -> bool {
                        $(
                            kernel::regmap::check_access!(VOLATILE, $access, reg, $reg_addr);
                        )+

                        false
                    }

                    fn is_precious_reg(reg: u32) -> bool {
                        $(
                            kernel::regmap::check_access!(PRECIOUS, $access, reg, $reg_addr);
                        )+

                        false
                    }

                    fn is_writeable_noinc_reg(reg: u32) -> bool {
                        $(
                            kernel::regmap::check_access!(WRITE_NO_INC, $access, reg, $reg_addr);
                        )+

                        false
                    }

                    fn is_readable_noinc_reg(reg: u32) -> bool {
                        $(
                            kernel::regmap::check_access!(READ_NO_INC, $access, reg, $reg_addr);
                        )+

                        false
                    }
                }
            }
        }

        const $name: regmap::RegFields<{$crate::count_fields!($($($type)*)+)}> =
            regmap::RegFields::new([
                $(
                    $(
                        $crate::reg_field!($reg_name, $field_name)
                    ),*
                ),+
            ]);
    };
}
pub use registers;
