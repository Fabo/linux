// SPDX-License-Identifier: GPL-2.0

//! Rust regulator driver sample.

use kernel::{
    c_str,
    device,
    module_platform_driver,
    of,
    platform,
    prelude::*,
    regulator::driver::{Config, Desc, LinearRange, Operations, RegulatorDev, Type}};

module_platform_driver! {
    type: Driver,
    name: "rust_regulator_driver",
    license: "GPL",
}

kernel::module_of_id_table!(MOD_TABLE, REGULATOR_DRIVER_ID_TABLE);
kernel::define_of_id_table! {REGULATOR_DRIVER_ID_TABLE, (), [
    (of::DeviceId::Compatible(b"rust,regulator-consumer"), None),
]}

struct Driver;

impl platform::Driver for Driver {
    type Data = ();

    kernel::driver_of_id_table!(REGULATOR_DRIVER_ID_TABLE);

    fn probe(pdev: &mut platform::Device, _id_info: Option<&Self::IdInfo>) -> Result<Self::Data> {
        let dev = device::Device::from_dev(pdev);
        let config = Config::<()>::new(&dev)/*.with_regmap(regmap)*/;
        let _rdev = RegulatorDev::register(&dev, &NCV6336_DESC, &config)?;

        Ok(())
    }
}

#[vtable]
impl Operations for Driver {
}

const NCV6336_DESC: Desc = Desc::new::<Driver>(c_str!("ncv6336"), Type::Voltage)
    //.with_owner(THIS_MODULE)
    .with_active_discharge(0x12, 0x10, 0x10, 0)
    .with_csel(0x16, 0xc0, &[3500000, 4000000, 4500000, 5000000])
    .with_enable(0x11, 0x80, 0x80, 0)
    .with_linear_ranges(0x11, 0x7f, &[LinearRange {
        min: 600000,
        min_sel: 0,
        max_sel: 0x7f,
        step: 6250,
    }]);
