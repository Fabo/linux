// SPDX-License-Identifier: GPL-2.0

//! Driver for the Onsemi Buck Converter NCV6336
//!
//! Datasheet: https://www.onsemi.com/pdf/datasheet/ncv6336bm-d.pdf

use kernel::{
    c_str, i2c, of,
    prelude::*,
    regmap,
    regulator::{
        driver::{Config, Desc, Operations, RegmapHelpers, RegulatorDev, Status, Type},
        Mode,
    },
    sync::{Arc, ArcBorrow},
};
use register::*;

kernel::module_i2c_id_table!(I2C_MOD_TABLE, NCV6336_I2C_ID_TABLE);
kernel::define_i2c_id_table! {NCV6336_I2C_ID_TABLE, (), [
    (i2c::DeviceId(b"ncv6336"), None),
]}

kernel::module_of_id_table!(OF_MOD_TABLE, NCV6336_OF_ID_TABLE);
kernel::define_of_id_table! {NCV6336_OF_ID_TABLE, (), [
    (of::DeviceId::Compatible(b"onnn,ncv6336"), None),
]}

kernel::module_i2c_driver! {
    type: Driver,
    name: "ncv6336",
    license: "GPL",
}

regmap::registers!(REGS, {
    {
        pid, 0x3, kernel::regmap::access::READ, {
            value => raw([7:0], ro),
        }
    }, {
        rid, 0x4, kernel::regmap::access::READ, {
            value => raw([7:0], ro),
        }
    }, {
        fid, 0x5, kernel::regmap::access::READ, {
            value => raw([7:0], ro),
        }
    }, {
        progvsel1, 0x10, kernel::regmap::access::RW, {
            voutvsel1 => raw([6:0], rw),
            envsel1   => bit(7, rw),
        }
    }, {
        progvsel0, 0x11, kernel::regmap::access::RW, {
            voutvsel0 => raw([6:0], rw),
            envsel0   => bit(7, rw),
        }
    }, {
        pgood, 0x12, kernel::regmap::access::RW, {
            dischg => bit(4, rw),
        }
    }, {
        command, 0x14, kernel::regmap::access::RW, {
            vselgt   => bit(0, rw),
            pwmvsel1 => bit(6, rw),
            pwmvsel0 => bit(7, rw),
        }
    }, {
        limconf, 0x16, kernel::regmap::access::RW, {
            rearm     => bit(0, rw),
            rststatus => bit(1, rw),
            tpwth     => enum([5:4], rw, {
                Temp83C  = 0x0,
                Temp94C  = 0x1,
                Temp105C = 0x2,
                Temp116C = 0x3,
            }),
            ipeak     => enum([7:6], rw, {
                Peak3p5A = 0x0,
                Peak4p0A = 0x1,
                Peak4p5A = 0x2,
                Peak5p0A = 0x3,
            }),
        }
    }
});

static NCV6336_DESC: Desc = Desc::new::<Driver>(c_str!("ncv6336"), Type::Voltage)
    .with_owner(&THIS_MODULE)
    .with_active_discharge(
        pgood::addr(),
        pgood::dischg::mask(),
        pgood::dischg::mask(),
        0,
    )
    .with_csel(
        limconf::addr(),
        limconf::ipeak::mask(),
        &[3500000, 4000000, 4500000, 5000000],
    )
    .with_enable(
        progvsel0::addr(),
        progvsel0::envsel0::mask(),
        progvsel0::envsel0::mask(),
        0,
    )
    .with_linear_mapping(
        progvsel0::addr(),
        progvsel0::voutvsel0::mask(),
        600000,
        6250,
        128,
        0,
    );

type DeviceData = kernel::device::Data<Registrations, (), ()>;

struct Registrations {
    fields: regmap::Fields<{ REGS.count() }>,
    rdev: Option<RegulatorDev>,
}

struct Driver;
impl i2c::Driver for Driver {
    type Data = Arc<DeviceData>;

    kernel::driver_i2c_id_table!(NCV6336_I2C_ID_TABLE);
    kernel::driver_of_id_table!(NCV6336_OF_ID_TABLE);

    fn probe(client: &mut i2c::Client, _id_info: Option<&Self::IdInfo>) -> Result<Self::Data> {
        let dev = client.device();
        let config = regmap::Config::<register::AccessOps>::new(8, 8)
            .with_max_register(0x16)
            .with_cache_type(regmap::CacheType::RbTree);
        let mut regmap = regmap::Regmap::init_i2c(client, &config);

        let mut registrations = Registrations {
            fields: regmap.alloc_fields(&REGS)?,
            rdev: None,
        };
        let pid = pid::value::read(&mut registrations.fields)?;
        let rid = rid::value::read(&mut registrations.fields)?;
        let fid = fid::value::read(&mut registrations.fields)?;

        let data = kernel::new_device_data!(registrations, (), (), "Ncv6336::Registrations")?;
        let data: Arc<DeviceData> = data.into();

        let config = Config::<Self::Data>::new(&dev)
            .with_drvdata(data.clone())
            .with_regmap(&regmap);

        let rdev = RegulatorDev::register(&dev, &NCV6336_DESC, config)?;
        data.registrations().ok_or(EINVAL)?.rdev = Some(rdev);

        dev_info!(dev, "PID: {pid:#x}, RID: {rid:#x}, FID: {fid:#x}");

        Ok(data)
    }
}

#[vtable]
impl Operations for Driver {
    type Data = Arc<DeviceData>;

    fn list_voltage(
        _data: ArcBorrow<'_, DeviceData>,
        rdev: &RegulatorDev,
        selector: u32,
    ) -> Result<i32> {
        rdev.list_voltage_linear(selector)
    }

    fn enable(_data: ArcBorrow<'_, DeviceData>, rdev: &RegulatorDev) -> Result {
        rdev.enable_regmap()
    }

    fn disable(_data: ArcBorrow<'_, DeviceData>, rdev: &RegulatorDev) -> Result {
        rdev.disable_regmap()
    }

    fn is_enabled(_data: ArcBorrow<'_, DeviceData>, rdev: &RegulatorDev) -> Result<bool> {
        rdev.is_enabled_regmap()
    }

    fn set_active_discharge(
        _data: ArcBorrow<'_, DeviceData>,
        rdev: &RegulatorDev,
        enable: bool,
    ) -> Result {
        rdev.set_active_discharge_regmap(enable)
    }

    fn set_current_limit(
        _data: ArcBorrow<'_, DeviceData>,
        rdev: &RegulatorDev,
        min_ua: i32,
        max_ua: i32,
    ) -> Result {
        rdev.set_current_limit_regmap(min_ua, max_ua)
    }

    fn get_current_limit(_data: ArcBorrow<'_, DeviceData>, rdev: &RegulatorDev) -> Result<i32> {
        rdev.get_current_limit_regmap()
    }

    fn set_voltage_sel(
        _data: ArcBorrow<'_, DeviceData>,
        rdev: &RegulatorDev,
        selector: u32,
    ) -> Result {
        rdev.set_voltage_sel_regmap(selector)
    }

    fn get_voltage_sel(_data: ArcBorrow<'_, DeviceData>, rdev: &RegulatorDev) -> Result<i32> {
        rdev.get_voltage_sel_regmap()
    }

    fn set_mode(data: ArcBorrow<'_, DeviceData>, _rdev: &RegulatorDev, mode: Mode) -> Result {
        let mut registrations = data.registrations().ok_or(EINVAL)?;
        match mode {
            Mode::Normal => register::command::pwmvsel0::clear(&mut registrations.fields),
            Mode::Fast => register::command::pwmvsel0::set(&mut registrations.fields),
            _ => Err(ENOTSUPP),
        }
    }

    fn get_mode(data: ArcBorrow<'_, DeviceData>, _rdev: &RegulatorDev) -> Mode {
        if let Some(mut registrations) = data.registrations() {
            let val = register::command::pwmvsel0::is_set(&mut registrations.fields);
            match val {
                Ok(true) => Mode::Fast,
                Ok(false) => Mode::Normal,
                Err(_) => Mode::Invalid,
            }
        } else {
            Mode::Invalid
        }
    }

    fn get_status(data: ArcBorrow<'_, DeviceData>, rdev: &RegulatorDev) -> Result<Status> {
        if !Self::is_enabled(data, rdev)? {
            return Ok(Status::Off);
        }

        Ok(Self::get_mode(data, rdev).into())
    }

    fn set_suspend_voltage(
        data: ArcBorrow<'_, DeviceData>,
        _rdev: &RegulatorDev,
        uv: i32,
    ) -> Result {
        let quot = (uv - 600000) / 6250;
        let rem = (uv - 600000) % 6250;

        let selector = if rem > 0 {
            quot + 1
        } else {
            quot
        };

        let mut registrations = data.registrations().ok_or(EINVAL)?;
        register::progvsel1::voutvsel1::write(&mut registrations.fields, selector as _)
    }

    fn set_suspend_enable(data: ArcBorrow<'_, DeviceData>, _rdev: &RegulatorDev) -> Result {
        let mut registrations = data.registrations().ok_or(EINVAL)?;
        register::progvsel1::envsel1::set(&mut registrations.fields)?;
        register::command::vselgt::clear(&mut registrations.fields)
    }

    fn set_suspend_disable(data: ArcBorrow<'_, DeviceData>, _rdev: &RegulatorDev) -> Result {
        let mut registrations = data.registrations().ok_or(EINVAL)?;
        register::progvsel1::envsel1::clear(&mut registrations.fields)?;
        register::command::vselgt::set(&mut registrations.fields)
    }

    fn set_suspend_mode(
        data: ArcBorrow<'_, DeviceData>,
        _rdev: &RegulatorDev,
        mode: Mode,
    ) -> Result {
        let mut registrations = data.registrations().ok_or(EINVAL)?;
        match mode {
            Mode::Normal => register::command::pwmvsel1::clear(&mut registrations.fields),
            Mode::Fast => register::command::pwmvsel1::set(&mut registrations.fields),
            _ => Err(ENOTSUPP),
        }
    }
}
