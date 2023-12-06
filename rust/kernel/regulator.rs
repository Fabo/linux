// SPDX-License-Identifier: GPL-2.0

//! SoC Regulators

pub mod consumer;
pub mod driver;

use crate::{
    bindings,
    error::{code::*, Result},
};

/// [`Regulator`] operating modes
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Mode {
    /// Invalid mode
    Invalid = bindings::REGULATOR_MODE_INVALID,
    /// Regulator can handle fast changes in it's load
    Fast = bindings::REGULATOR_MODE_FAST,
    /// Normal regulator power supply mode
    Normal = bindings::REGULATOR_MODE_NORMAL,
    /// Regulator runs in a more efficient mode for light loads
    Idle = bindings::REGULATOR_MODE_IDLE,
    /// Regulator runs in the most efficient mode for very light loads
    Standby = bindings::REGULATOR_MODE_STANDBY,
}

impl Mode {
    fn from_bindings(mode: core::ffi::c_uint) -> Result<Self> {
        match mode {
            bindings::REGULATOR_MODE_FAST => Ok(Self::Fast),
            bindings::REGULATOR_MODE_NORMAL => Ok(Self::Normal),
            bindings::REGULATOR_MODE_IDLE => Ok(Self::Idle),
            bindings::REGULATOR_MODE_STANDBY => Ok(Self::Standby),
            bindings::REGULATOR_MODE_INVALID => Ok(Self::Invalid),
            _ => Err(EINVAL),
        }
    }
}
