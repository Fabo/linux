use crate::{bindings, error::{to_result, Result}};
#[cfg(CONFIG_I2C)]
use crate::i2c;

pub struct Regmap {
    ptr: *mut bindings::regmap,
}
impl Regmap {
    #[cfg(CONFIG_I2C)]
    pub fn init_i2c(i2c: &i2c::Client, config: *const bindings::regmap_config) -> Self {
        let regmap = unsafe { bindings::regmap_init_i2c(i2c.raw_client(), config) };

        Self {
            ptr: regmap,
        }
    }

    pub fn alloc_fields<const N: usize>(&mut self, reg_fields: &'static RegFields<N>) -> Result<RegmapFields<N>> {
        let mut rm_fields = core::ptr::null_mut();
        to_result(unsafe {
            bindings::regmap_field_bulk_alloc(self.ptr, &mut rm_fields, reg_fields.0.as_ptr(),
                                              reg_fields.0.len() as i32)
        })?;

        Ok(RegmapFields {
            rm_fields,
            _reg_fields: reg_fields,
        })
    }
}

pub struct RegFields<const N: usize>([bindings::reg_field; N]);

impl<const N: usize> RegFields<N> {
    pub const fn new(fields: [bindings::reg_field; N]) -> Self {
        Self(fields)
    }
}

pub struct RegmapFields<const N: usize> {
    rm_fields: *mut bindings::regmap_field,
    _reg_fields: &'static RegFields<N>,
}
impl<const N: usize> RegmapFields<N> {
    pub unsafe fn index(&mut self, index: usize) -> *mut bindings::regmap_field {
        unsafe {
            self.rm_fields.add(index)
        }
    }
}

#[macro_export]
macro_rules! field_bit {
    ($reg_name:ident, $field_name:ident, $reg:literal, $pos:literal) => {
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
            pub fn set<const N: usize>(fields: &mut regmap::RegmapFields<N>) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_write(field, 1) })
            }

            #[allow(dead_code)]
            pub fn force_set<const N: usize>(fields: &mut regmap::RegmapFields<N>) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_force_write(field, 1) })
            }

            #[allow(dead_code)]
            pub fn clear<const N: usize>(fields: &mut regmap::RegmapFields<N>) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_write(field, 0) })
            }

            #[allow(dead_code)]
            pub fn force_clear<const N: usize>(fields: &mut regmap::RegmapFields<N>) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_force_write(field, 0) })
            }

            #[allow(dead_code)]
            pub fn is_set<const N: usize>(fields: &mut regmap::RegmapFields<N>) -> Result<bool> {
                let field = unsafe { fields.index(Self::id() as usize) };
                let mut val: core::ffi::c_uint = 0;
                kernel::error::to_result(unsafe { bindings::regmap_field_read(field, &mut val) })?;
                Ok(val == 1)
            }
        }
    };
}

#[macro_export]
macro_rules! field_enum {
    ($reg_name:ident, $field_name:ident, $reg:literal, $lsb:literal, $msb:literal, {
        $($k:ident = $v:literal,)+ }) => {
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
                pub fn write<const N: usize>(fields: &mut regmap::RegmapFields<N>, val: [<$field_name _enum>]) -> Result {
                    let field = unsafe { fields.index(Self::id() as usize) };
                    kernel::error::to_result(unsafe { bindings::regmap_field_write(field, val as _) })
                }

                #[allow(dead_code)]
                pub fn force_write<const N: usize>(fields: &mut regmap::RegmapFields<N>, val: [<$field_name _enum>]) -> Result {
                    let field = unsafe { fields.index(Self::id() as usize) };
                    kernel::error::to_result(unsafe { bindings::regmap_field_force_write(field, val as _) })
                }

                #[allow(dead_code)]
                pub fn read<const N: usize>(fields: &mut regmap::RegmapFields<N>) -> Result<[<$field_name _enum>]> {
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
}

#[macro_export]
macro_rules! field_val {
    ($reg_name:ident, $field_name:ident, $reg:literal, $lsb:literal, $msb:literal, rw) => {
        $crate::field_val!($reg_name, $field_name, $reg, $lsb, $msb, reserved);
        $crate::field_val!($reg_name, $field_name, $reg, $lsb, $msb, _ro);
        $crate::field_val!($reg_name, $field_name, $reg, $lsb, $msb, _wo);
    };
    ($reg_name:ident, $field_name:ident, $reg:literal, $lsb:literal, $msb:literal, ro) => {
        $crate::field_val!($reg_name, $field_name, $reg, $lsb, $msb, reserved);
        $crate::field_val!($reg_name, $field_name, $reg, $lsb, $msb, _ro);
    };
    ($reg_name:ident, $field_name:ident, $reg:literal, $lsb:literal, $msb:literal, wo) => {
        $crate::field_val!($reg_name, $field_name, $reg, $lsb, $msb, reserved);
        $crate::field_val!($reg_name, $field_name, $reg, $lsb, $msb, _wo);
    };

    ($reg_name:ident, $field_name:ident, $reg:literal, $lsb:literal, $msb:literal, reserved) => {
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
        }
    };

    ($reg_name:ident, $field_name:ident, $reg:literal, $lsb:literal, $msb:literal, _ro) => {
        impl $field_name {
            #[allow(dead_code)]
            pub fn read<const N: usize>(fields: &mut regmap::RegmapFields<N>) -> Result<core::ffi::c_uint> {
                let field = unsafe { fields.index(Self::id() as usize) };
                let mut val = 0;
                let res = unsafe { bindings::regmap_field_read(field, &mut val) };
                if res < 0 {
                    return Err(kernel::error::Error::from_errno(res));
                }

                Ok(val)
            }
        }
    };

    ($reg_name:ident, $field_name:ident, $reg:literal, $lsb:literal, $msb:literal, _wo) => {
        impl $field_name {
            #[allow(dead_code)]
            pub fn write<const N: usize>(fields: &mut regmap::RegmapFields<N>, val: core::ffi::c_uint) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_write(field, val as _) })
            }

            #[allow(dead_code)]
            pub fn force_write<const N: usize>(fields: &mut regmap::RegmapFields<N>, val: core::ffi::c_uint) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_force_write(field, val as _) })
            }

            #[allow(dead_code)]
            pub fn update_bits<const N: usize>(fields: &mut regmap::RegmapFields<N>, mask: core::ffi::c_uint, val: core::ffi::c_uint) -> Result{
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_update_bits(field, mask, val) })
            }

            #[allow(dead_code)]
            pub fn force_update_bits<const N: usize>(fields: &mut regmap::RegmapFields<N>, mask: core::ffi::c_uint, val: core::ffi::c_uint) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_force_update_bits(field, mask, val) })
            }

            #[allow(dead_code)]
            pub fn set_bits<const N: usize>(fields: &mut regmap::RegmapFields<N>, bits: core::ffi::c_uint) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_set_bits(field, bits) })
            }

            #[allow(dead_code)]
            pub fn clear_bits<const N: usize>(fields: &mut regmap::RegmapFields<N>, bits: core::ffi::c_uint) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_clear_bits(field, bits) })
            }

            #[allow(dead_code)]
            pub fn test_bits<const N: usize>(fields: &mut regmap::RegmapFields<N>, bits: core::ffi::c_uint) -> Result {
                let field = unsafe { fields.index(Self::id() as usize) };
                kernel::error::to_result(unsafe { bindings::regmap_field_test_bits(field, bits) })
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

            $crate::[<field_ $type>]!($reg, $name, $($t)*);
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
            $reg_name:ident, $reg_addr:literal, {
                $($field_name:ident => $type:ident($($x:tt),*)),*,
            }
        }),+
    }) => {
        mod register {
            kernel::macros::paste! {
                $(
                    pub mod $reg_name {
                        use kernel::{bindings, error::{Result}, regmap};
                        $( $crate::fields!($type, $reg_name, $field_name, $reg_addr, $($x),*); )*
                    }
                )+

                #[repr(u32)]
                #[allow(non_camel_case_types)]
                pub enum Fields {
                    $($(
                        [<$reg_name _ $field_name>],
                    )*)+
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
