// SPDX-License-Identifier: GPL-2.0

//! Rust regulator consumer driver sample.

use kernel::{c_str, device, module_platform_driver, of, platform, prelude::*,
    regulator::consumer::Regulator};

module_platform_driver! {
    type: Driver,
    name: "rust_regulator_consumer",
    license: "GPL",
}

kernel::module_of_id_table!(MOD_TABLE, REGULATOR_CONSUMER_ID_TABLE);
kernel::define_of_id_table! {REGULATOR_CONSUMER_ID_TABLE, (), [
    (of::DeviceId::Compatible(b"rust,regulator-consumer"), None),
]}

struct Driver;
impl platform::Driver for Driver {
    kernel::driver_of_id_table!(REGULATOR_CONSUMER_ID_TABLE);

    fn probe(pdev: &mut platform::Device, _id_info: Option<&Self::IdInfo>) -> Result {
        let dev = device::Device::from_dev(pdev);
        let vbus = Regulator::get(&dev, c_str!("vbus"))?;
        let _ = vbus.enable()?;

        Ok(())
    }
}
