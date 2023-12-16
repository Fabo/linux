//use crate::static_assert;

pub const fn genmask(h: u32, l: u32) -> u32 {
    //static_assert!(h >= l);
    //static_assert!(h < 32);
    ((!0u32) - (1 << l) + 1) & ((!0u32) >> (32 - 1 - h))
}
