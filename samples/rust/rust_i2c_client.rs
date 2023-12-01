// SPDX-License-Identifier: GPL-2.0

//! Rust regulator consumer driver sample.

use kernel::{bindings, i2c, of, prelude::*, regmap};

kernel::module_i2c_driver! {
    type: Driver,
    name: "rust_i2c_client",
    license: "GPL",
}

regmap::registers!(REGS, {
    {
        int_ack, 0x00, {
            ack_pg    => bit(0),
            ack_idcdc => bit(1),
            ack_uvlo  => bit(2),
            ack_tprew => bit(5),
            ack_twarn => bit(6),
            ack_tsd   => bit(7),
        }
    }, {
        pid, 0x03, {
            value => val(0, 7, rw),
        }
    }, {
        rid, 0x04, {
            value => enum(0, 7, {
                FirstSilicon      = 0x00,
                VersionOptimized  = 0x01,
                Production        = 0x10,
            }),
        }
    }, {
        fid, 0x05, {
            value => val(0, 7, ro),
        }
    }, {
        pgood, 0x12, {
            pgdcdc => bit(0),
            pgdvs  => bit(1),
            tor    => enum(2, 3, {
                Tor0ms    = 0x0,
                Tor8p8ms  = 0x1,
                Tor35p2ms = 0x2,
                Tor70p4ms = 0x3,
            }),
            dischg => bit(4),
        }
    }, {
        limconf, 0x16, {
            rearm     => bit(0),
            rststatus => bit(1),
            tpwth     => enum(4, 5, {
                Temp83C  = 0x1,
                Temp94C  = 0x2,
                Temp105C = 0x3,
                Temp116C = 0x4,
            }),
            ipeak     => enum(6, 7, {
                Peak5p2A = 0x1,
                Peak5p8A = 0x2,
                Peak6p4A = 0x3,
                Peak6p8A = 0x4,
            }),
        }
    }
});

struct Driver;
impl i2c::Driver for Driver {
    kernel::driver_i2c_id_table!(I2C_CLIENT_I2C_ID_TABLE);
    kernel::driver_of_id_table!(I2C_CLIENT_OF_ID_TABLE);

    fn probe(client: &mut i2c::Client, _id_info: Option<&Self::IdInfo>) -> Result {
        pr_err!("XXX: probing I2C");
        let config = bindings::regmap_config {
            reg_bits: 8,
            val_bits: 8,
            max_register: 0x16,
            ..Default::default()
        };
        let mut regmap = regmap::Regmap::init_i2c(&client, &config);
        let mut fields = unsafe { regmap.alloc_fields(&REGS).unwrap_unchecked() };
        use register as reg;

        pr_err!("XXX: FID = {}", reg::fid::value::read(&mut fields)?);

        reg::limconf::ipeak::write(&mut fields, reg::limconf::ipeak_enum::Peak5p2A)?;

        Ok(())
    }
}

kernel::module_i2c_id_table!(I2C_MOD_TABLE, I2C_CLIENT_I2C_ID_TABLE);
kernel::module_of_id_table!(OF_MOD_TABLE, I2C_CLIENT_OF_ID_TABLE);

kernel::define_i2c_id_table! {I2C_CLIENT_I2C_ID_TABLE, (), [
    (i2c::DeviceId(b"rust-sample-i2c"), None),
]}
kernel::define_of_id_table! {I2C_CLIENT_OF_ID_TABLE, (), [
    (of::DeviceId::Compatible(b"rust,rust-sample-i2c"), None),
]}
