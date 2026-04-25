use anyhow::{Context, Result};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::randr::{Connection as RandrConnection, ConnectionExt as RandrExt};
use x11rb::protocol::xproto::{ConnectionExt as XprotoExt, KeyButMask};
use x11rb::protocol::xinput::{
    ConnectionExt as XinputExt, Device, EventMask, Fp3232, RawMotionEvent, XIEventMask,
};
use x11rb::rust_connection::RustConnection;
use x11rb::NONE;

use crate::geometry::{Monitor, Point, PointerState, RawMotion};

pub struct X11Backend {
    conn: RustConnection,
    root: u32,
}

impl X11Backend {
    pub fn connect() -> Result<Self> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let root = conn.setup().roots[screen_num].root;

        Ok(Self { conn, root })
    }

    pub fn monitors(&self) -> Result<Vec<Monitor>> {
        self.conn
            .randr_query_version(1, 2)?
            .reply()
            .context("query RandR version")?;

        let resources = self
            .conn
            .randr_get_screen_resources_current(self.root)?
            .reply()
            .context("get RandR screen resources")?;

        let mut monitors = Vec::new();
        for output in resources.outputs {
            let output_info = self
                .conn
                .randr_get_output_info(output, resources.config_timestamp)?
                .reply()
                .with_context(|| format!("get RandR output info for {output}"))?;

            if output_info.connection != RandrConnection::CONNECTED
                || output_info.crtc == NONE
                || output_info.name.is_empty()
            {
                continue;
            }

            let crtc_info = self
                .conn
                .randr_get_crtc_info(output_info.crtc, resources.config_timestamp)?
                .reply()
                .with_context(|| format!("get RandR CRTC info for {output}"))?;

            if crtc_info.width == 0 || crtc_info.height == 0 {
                continue;
            }

            monitors.push(Monitor {
                name: String::from_utf8_lossy(&output_info.name).into_owned(),
                x: i32::from(crtc_info.x),
                y: i32::from(crtc_info.y),
                width: i32::from(crtc_info.width),
                height: i32::from(crtc_info.height),
                mm_width: nonzero_u32_to_i32(output_info.mm_width),
                mm_height: nonzero_u32_to_i32(output_info.mm_height),
            });
        }

        monitors.sort_by_key(|monitor| (monitor.x, monitor.y, monitor.name.clone()));
        Ok(monitors)
    }

    pub fn query_pointer(&self) -> Result<PointerState> {
        let reply = self
            .conn
            .query_pointer(self.root)?
            .reply()
            .context("query pointer")?;

        Ok(PointerState {
            position: Point {
                x: i32::from(reply.root_x),
                y: i32::from(reply.root_y),
            },
            buttons_down: has_button_down(reply.mask),
        })
    }

    pub fn enable_raw_motion(&self) -> Result<()> {
        self.conn
            .xinput_xi_query_version(2, 0)?
            .reply()
            .context("query XInput2 version")?;

        let mask = EventMask {
            deviceid: u16::from(Device::ALL_MASTER),
            mask: vec![XIEventMask::RAW_MOTION],
        };

        self.conn
            .xinput_xi_select_events(self.root, &[mask])?
            .check()
            .context("select XI2 raw motion events")?;
        self.conn.flush().context("flush XInput2 selection")?;
        Ok(())
    }

    pub fn wait_raw_motion(&self) -> Result<RawMotion> {
        loop {
            let event = self.conn.wait_for_event().context("wait for X11 event")?;
            if let Event::XinputRawMotion(event) = event {
                return Ok(raw_motion_delta(&event));
            }
        }
    }

    pub fn warp_pointer(&self, point: Point) -> Result<()> {
        self.conn
            .warp_pointer(
                NONE,
                self.root,
                0,
                0,
                0,
                0,
                i16::try_from(point.x).context("target x is outside X11 i16 range")?,
                i16::try_from(point.y).context("target y is outside X11 i16 range")?,
            )?
            .check()
            .context("warp pointer")?;
        self.conn.flush().context("flush X11 connection")?;
        Ok(())
    }
}

fn raw_motion_delta(event: &RawMotionEvent) -> RawMotion {
    RawMotion {
        dx: valuator_value(event, 0).unwrap_or_default(),
        dy: valuator_value(event, 1).unwrap_or_default(),
    }
}

fn valuator_value(event: &RawMotionEvent, axis: usize) -> Option<f64> {
    let mut value_index = 0usize;

    for (mask_index, mask) in event.valuator_mask.iter().enumerate() {
        for bit in 0..32 {
            let axis_index = mask_index * 32 + bit;
            if mask & (1_u32 << bit) == 0 {
                continue;
            }

            if axis_index == axis {
                return event.axisvalues_raw.get(value_index).map(fp3232_to_f64);
            }
            value_index += 1;
        }
    }

    None
}

fn fp3232_to_f64(value: &Fp3232) -> f64 {
    value.integral as f64 + value.frac as f64 / 4_294_967_296.0
}

fn has_button_down(mask: KeyButMask) -> bool {
    let raw = u16::from(mask);
    let buttons = u16::from(KeyButMask::BUTTON1)
        | u16::from(KeyButMask::BUTTON2)
        | u16::from(KeyButMask::BUTTON3)
        | u16::from(KeyButMask::BUTTON4)
        | u16::from(KeyButMask::BUTTON5);

    raw & buttons != 0
}

fn nonzero_u32_to_i32(value: u32) -> Option<i32> {
    if value == 0 {
        None
    } else {
        i32::try_from(value).ok()
    }
}
