// SPDX-License-Identifier: GPL-2.0

//! Driver for the Truly R63350 DSI panel

use core::ops::DerefMut;
use kernel::{
    c_str,
    drm::{
        self, connector, mipi_dsi,
        panel::{self, Panel, PanelRegistration},
    },
    error::Result,
    gpio::consumer as gpio,
    of,
    prelude::*,
    regulator::consumer::Regulator,
    sync::{new_mutex, Arc, Mutex},
    time::{delay, Delta},
    types::{ForeignOwnable, ScopeGuard},
};

type DeviceData = Mutex<Registrations>;

struct Registrations {
    dsi: mipi_dsi::Device,
    regulators: [Regulator; 3],
    reset_gpio: Option<gpio::Desc>,
    prepared: bool,
    pdata: &'static PlatformData,
    _panel: PanelRegistration,
}

struct Driver;

impl mipi_dsi::Driver for Driver {
    type Data = Arc<DeviceData>;
    type IdInfo = &'static PlatformData;

    const OF_DEVICE_ID_TABLE: Option<of::IdTable<Self::IdInfo>> = Some(&OF_TABLE);

    fn probe(mut dsi: mipi_dsi::Device, id_info: Option<&Self::IdInfo>) -> Result<Self::Data> {
        let id_info = id_info.ok_or(EINVAL)?;

        let mut panel = Panel::new::<Driver>(&dsi, connector::Type::DSI)?;
        panel.init_of_backlight()?;
        panel.prepare_prev_first(true);
        let panel = panel.register();

        let reset_gpio = gpio::Desc::get_opt(&dsi, c_str!("reset"), gpio::flags::OUT_LOW)?;

        let regulators = [
            Regulator::get(&dsi, c_str!("avdd"))?,
            Regulator::get(&dsi, c_str!("avee"))?,
            Regulator::get(&dsi, c_str!("iovcc"))?,
        ];

        dsi.set_lanes(4);
        dsi.set_format(mipi_dsi::PixelFormat::Rgb888);
        *dsi.mode_flags_mut() = mipi_dsi::mode::VIDEO | mipi_dsi::mode::VIDEO_BURST;

        let registrations = Arc::pin_init(
            new_mutex!(Registrations {
                dsi,
                regulators,
                reset_gpio,
                prepared: false,
                pdata: id_info,
                _panel: panel,
            }),
            GFP_KERNEL,
        )?;
        Ok(registrations)
    }
}

#[vtable]
impl panel::Operations for Driver {
    type Data = Arc<DeviceData>;

    fn prepare(data: <Self::Data as ForeignOwnable>::Borrowed<'_>) -> Result {
        let mut data = data.lock();
        let registrations: &mut Registrations = data.deref_mut();

        let reset_gpio = &mut registrations.reset_gpio;
        if let Some(ref mut reset) = reset_gpio {
            reset.set_value(1);
        }

        delay::fsleep(Delta::from_millis(30));

        let mut regulators = ScopeGuard::new_with_data(
            KVec::with_capacity(registrations.regulators.len(), GFP_KERNEL)?,
            |v| {
                v.into_iter().for_each(|r: &mut Regulator| {
                    let _ = r.disable();
                })
            },
        );

        for regulator in &mut registrations.regulators {
            regulator.enable()?;
            regulators.push(regulator, GFP_KERNEL)?;
        }

        delay::fsleep(Delta::from_millis(30));

        let reset = if let Some(ref mut reset) = reset_gpio {
            reset.set_value(0);
            Some(ScopeGuard::new_with_data((), |_| {
                if let Some(ref mut reset) = reset_gpio {
                    reset.set_value(1)
                }
            }))
        } else {
            None
        };

        delay::fsleep(Delta::from_millis(30));

        let dsi = &mut registrations.dsi;

        *dsi.mode_flags_mut() |= mipi_dsi::mode::LPM;

        dsi.dcs_soft_reset()?;
        delay::fsleep(Delta::from_millis(20));

        for cmd in registrations.pdata.panel_on_cmds {
            dsi.dcs_write_buffer(cmd.0)?;

            let duration = if cmd.1 == 0 {
                Delta::from_nanos(100)
            } else {
                Delta::from_millis(cmd.1)
            };
            delay::fsleep(duration);
        }

        dsi.dcs_exit_sleep_mode()?;
        delay::fsleep(Delta::from_millis(120));
        dsi.dcs_set_display_on()?;
        delay::fsleep(Delta::from_millis(120));

        regulators.dismiss();
        if let Some(reset) = reset {
            reset.dismiss();
        }
        registrations.prepared = true;

        Ok(())
    }

    fn unprepare(data: <Self::Data as ForeignOwnable>::Borrowed<'_>) -> Result {
        let mut data = data.lock();
        let registrations: &mut Registrations = data.deref_mut();

        if !registrations.prepared {
            return Ok(());
        }

        let dsi = &mut registrations.dsi;
        *dsi.mode_flags_mut() &= !mipi_dsi::mode::LPM;

        let _ = dsi.dcs_set_display_off();
        delay::fsleep(Delta::from_millis(120));
        let _ = dsi.dcs_enter_sleep_mode();

        for cmd in registrations.pdata.panel_off_cmds {
            let _ = dsi.dcs_write_buffer(cmd.0);

            let duration = if cmd.1 == 0 {
                Delta::from_nanos(100)
            } else {
                Delta::from_millis(cmd.1)
            };
            delay::fsleep(duration);
        }

        if let Some(ref mut reset) = &mut registrations.reset_gpio {
            reset.set_value(1);
        }

        for regulator in &mut registrations.regulators {
            let _ = regulator.disable();
        }

        registrations.prepared = false;

        Ok(())
    }

    fn get_modes(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        connector: &mut drm::connector::Connector,
    ) -> Result<u16> {
        let display_mode = drm::mode::DisplayModeData {
            clock: 144981,
            hdisplay: 1080,
            hsync_start: 1080 + 92,
            hsync_end: 1080 + 92 + 20,
            htotal: 1080 + 92 + 20 + 60,
            vdisplay: 1920,
            vsync_start: 1920 + 4,
            vsync_end: 1920 + 4 + 1,
            vtotal: 1920 + 4 + 1 + 5,
            width_mm: 68,
            height_mm: 121,
            ..Default::default()
        };

        let mut mode =
            drm::mode::DisplayMode::duplicate(unsafe { connector.drm_device() }, &display_mode)?;
        mode.set_name();

        // FIXME
        unsafe {
            mode.probed_add(connector);

            connector.display_info_mut().width_mm = display_mode.width_mm as _;
            connector.display_info_mut().height_mm = display_mode.height_mm as _;
        };

        Ok(1)
    }
}

struct PlatformData {
    panel_on_cmds: &'static [(&'static [u8], i64)],
    panel_off_cmds: &'static [(&'static [u8], i64)],
}

const AUO_PDATA: PlatformData = PlatformData {
    panel_on_cmds: &[
        (&[0xb0, 0x04], 0),
        (&[0xd6, 0x01], 0),
        (&[0xb3, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00], 0),
        (&[0xb4, 0x0c, 0x00], 0),
        (&[0xb6, 0x4b, 0xdb, 0x00], 0),
        (&[0xc0, 0x66], 0),
        (
            &[
                0xc1, 0x04, 0x60, 0x00, 0x20, 0x29, 0x41, 0x22, 0xfb, 0xf0, 0xff, 0xff, 0x9b, 0x7b,
                0xcf, 0xb5, 0xff, 0xff, 0x87, 0x8c, 0xc5, 0x11, 0x54, 0x02, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x22, 0x11, 0x02, 0x21, 0x00, 0xff, 0x11,
            ],
            0,
        ),
        (&[0xc2, 0x31, 0xf7, 0x80, 0x06, 0x04, 0x00, 0x00, 0x08], 0),
        (
            &[
                0xc4, 0x70, 0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x00, 0x02,
            ],
            0,
        ),
        (
            &[
                0xc6, 0xc8, 0x3c, 0x3c, 0x07, 0x01, 0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x0e, 0x1a, 0x07, 0xc8,
            ],
            0,
        ),
        (
            &[
                0xc7, 0x0a, 0x18, 0x20, 0x29, 0x37, 0x43, 0x4d, 0x5b, 0x3f, 0x46, 0x52, 0x5f, 0x67,
                0x70, 0x7c, 0x0a, 0x18, 0x20, 0x29, 0x37, 0x43, 0x4d, 0x5b, 0x3f, 0x46, 0x52, 0x5f,
                0x67, 0x70, 0x7c,
            ],
            0,
        ),
        (
            &[
                0xcb, 0x7f, 0xe1, 0x87, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0,
                0x00, 0x00,
            ],
            0,
        ),
        (&[0xcc, 0x32], 0),
        (
            &[
                0xd0, 0x11, 0x00, 0x00, 0x56, 0xd7, 0x40, 0x19, 0x19, 0x09, 0x00,
            ],
            0,
        ),
        (&[0xd1, 0x00, 0x48, 0x16, 0x0f], 0),
        (
            &[
                0xd3, 0x1b, 0x33, 0xbb, 0xbb, 0xb3, 0x33, 0x33, 0x33, 0x33, 0x00, 0x01, 0x00, 0x00,
                0xd8, 0xa0, 0x0c, 0x37, 0x37, 0x33, 0x33, 0x72, 0x12, 0x8a, 0x57, 0x3d, 0xbc,
            ],
            0,
        ),
        (&[0xd5, 0x06, 0x00, 0x00, 0x01, 0x35, 0x01, 0x35], 0),
        (&[0x29], 100),
        (&[0x11], 120),
    ],
    panel_off_cmds: &[(&[0x28], 10), (&[0xb0, 0x04], 120)],
};

const TRULY_PDATA: PlatformData = PlatformData {
    panel_on_cmds: &[
        (&[0xb0, 0x00], 0),
        (&[0xd6, 0x01], 0),
        (&[0xb3, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00], 0),
        (&[0xb4, 0x0c, 0x00], 0),
        (&[0xb6, 0x4b, 0xdb, 0x16], 0),
        (&[0xbe, 0x00, 0x04], 0),
        (&[0xc0, 0x66], 0),
        (
            &[
                0xc1, 0x04, 0x60, 0x00, 0x20, 0xa9, 0x30, 0x20, 0x63, 0xf0, 0xff, 0xff, 0x9b, 0x7b,
                0xcf, 0xb5, 0xff, 0xff, 0x87, 0x8c, 0x41, 0x22, 0x54, 0x02, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x22, 0x33, 0x03, 0x22, 0x00, 0xff,
            ],
            0,
        ),
        (&[0xc2, 0x31, 0xf7, 0x80, 0x06, 0x04, 0x00, 0x00, 0x08], 0),
        (&[0xc3, 0x00, 0x00, 0x00], 0),
        (
            &[
                0xc4, 0x70, 0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x00, 0x02,
            ],
            0,
        ),
        (&[0xc5, 0x00], 0),
        (
            &[
                0xc6, 0xc8, 0x3c, 0x3c, 0x07, 0x01, 0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x0e, 0x1a, 0x07, 0xc8,
            ],
            0,
        ),
        (
            &[
                0xc7, 0x03, 0x15, 0x1f, 0x2a, 0x39, 0x46, 0x4e, 0x5b, 0x3d, 0x45, 0x52, 0x5f, 0x68,
                0x6d, 0x72, 0x01, 0x15, 0x1f, 0x2a, 0x39, 0x46, 0x4e, 0x5b, 0x3d, 0x45, 0x52, 0x5f,
                0x68, 0x6d, 0x78,
            ],
            0,
        ),
        (
            &[
                0xcb, 0xff, 0xe1, 0x87, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xe1, 0x87, 0xff, 0xe8,
                0x00, 0x00,
            ],
            0,
        ),
        (&[0xcc, 0x34], 0),
        (
            &[
                0xd0, 0x11, 0x00, 0x00, 0x56, 0xd5, 0x40, 0x19, 0x19, 0x09, 0x00,
            ],
            0,
        ),
        (&[0xd1, 0x00, 0x48, 0x16, 0x0f], 0),
        (&[0xd2, 0x5c, 0x00, 0x00], 0),
        (
            &[
                0xd3, 0x1b, 0x33, 0xbb, 0xbb, 0xb3, 0x33, 0x33, 0x33, 0x33, 0x00, 0x01, 0x00, 0x00,
                0xd8, 0xa0, 0x0c, 0x4d, 0x4d, 0x33, 0x33, 0x72, 0x12, 0x8a, 0x57, 0x3d, 0xbc,
            ],
            0,
        ),
        (&[0xd5, 0x06, 0x00, 0x00, 0x01, 0x39, 0x01, 0x39], 0),
        (&[0xd8, 0x00, 0x00, 0x00], 0),
        (&[0xd9, 0x00, 0x00, 0x00], 0),
        (&[0xfd, 0x00, 0x00, 0x00, 0x30], 0),
        (&[0x35, 0x00], 0),
        (&[0x29], 50),
        (&[0x11], 120),
    ],
    panel_off_cmds: &[
        (&[0x28], 20),
        (&[0xb0, 0x04], 0),
        (
            &[
                0xd3, 0x13, 0x33, 0xbb, 0xb3, 0xb3, 0x33, 0x33, 0x33, 0x33, 0x00, 0x01, 0x00, 0x00,
                0xd8, 0xa0, 0x0c, 0x4d, 0x4d, 0x33, 0x33, 0x72, 0x12, 0x8a, 0x57, 0x3d, 0xbc,
            ],
            27,
        ),
        (&[0x10], 120),
        (&[0xb0, 0x00], 0),
        (&[0xb1, 0x01], 0),
    ],
};

kernel::of_device_table!(
    MODULE_OF_TABLE,
    OF_TABLE,
    &'static PlatformData,
    [
        (
            of::DeviceId::with_compatible(c_str!("auo,r63350-fhd")),
            &AUO_PDATA
        ),
        (
            of::DeviceId::with_compatible(c_str!("truly,r63350-fhd")),
            &TRULY_PDATA
        ),
    ]
);

kernel::module_mipi_dsi_driver! {
    type: Driver,
    name: "panel_truly_r63350",
    author: "Fabien Parent <fabien.parent@linaro.org>",
    license: "GPL",
}
