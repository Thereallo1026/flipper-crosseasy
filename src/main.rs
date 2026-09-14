#![no_main]
#![no_std]

extern crate flipperzero_rt;

mod crosseasy;
mod dolphin;
mod ui;

use core::ffi::{CStr, c_void};
use core::sync::atomic::Ordering;

use flipperzero_rt::{entry, manifest};
use flipperzero_sys as sys;
use flipperzero_sys::furi::UnsafeRecord;

use crosseasy::CrossEasyBeacon;
use ui::{
    FULLSCREEN, MENU_LEN, Model, SCREEN_BROADCAST, SCREEN_MENU, draw_callback, input_callback,
};

manifest!(
    name = "CrossEasy Beacon",
    app_version = 1,
    has_icon = true,
    icon = "crosseasy-10x10.icon",
);
entry!(main);

static MODEL: Model = Model::new();

const POLL_MS: u32 = 40;

fn led(value: u8) {
    unsafe { sys::furi_hal_light_set(sys::LightBlue, value) };
}

fn main(_args: Option<&CStr>) -> i32 {
    let mut beacon = CrossEasyBeacon::acquire();
    if beacon.is_none() {
        MODEL.error.store(true, Ordering::Relaxed);
    }

    unsafe {
        let queue =
            sys::furi_message_queue_alloc(8, core::mem::size_of::<sys::InputEvent>() as u32);

        let view_port = sys::view_port_alloc();
        sys::view_port_draw_callback_set(
            view_port,
            Some(draw_callback),
            &MODEL as *const Model as *mut c_void,
        );
        sys::view_port_input_callback_set(view_port, Some(input_callback), queue as *mut c_void);

        let gui = UnsafeRecord::open(c"gui");
        sys::gui_add_view_port(gui.as_ptr(), view_port, FULLSCREEN);

        loop {
            let mut event: sys::InputEvent = core::mem::zeroed();
            let got = sys::furi_message_queue_get(
                queue,
                &mut event as *mut sys::InputEvent as *mut c_void,
                POLL_MS,
            );

            if got == sys::FuriStatusOk && event.type_ == sys::InputTypeShort {
                let screen = MODEL.screen.load(Ordering::Relaxed);
                if event.key == sys::InputKeyBack {
                    if screen == SCREEN_BROADCAST {
                        if let Some(b) = beacon.as_mut() {
                            b.stop();
                        }
                        led(0);
                        MODEL.screen.store(SCREEN_MENU, Ordering::Relaxed);
                    } else {
                        break;
                    }
                } else if screen == SCREEN_MENU {
                    let idx = MODEL.menu_index.load(Ordering::Relaxed);
                    if event.key == sys::InputKeyUp {
                        MODEL
                            .menu_index
                            .store((idx + MENU_LEN - 1) % MENU_LEN, Ordering::Relaxed);
                    } else if event.key == sys::InputKeyDown {
                        MODEL.menu_index.store((idx + 1) % MENU_LEN, Ordering::Relaxed);
                    } else if event.key == sys::InputKeyOk {
                        if let Some(b) = beacon.as_mut() {
                            if b.start() {
                                MODEL.frame.store(0, Ordering::Relaxed);
                                MODEL.screen.store(SCREEN_BROADCAST, Ordering::Relaxed);
                            } else {
                                MODEL.error.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }

            // pulse the led while broadcasting
            if MODEL.screen.load(Ordering::Relaxed) == SCREEN_BROADCAST {
                let frame = MODEL.frame.fetch_add(1, Ordering::Relaxed);
                led(if frame % 2 == 0 { 0xFF } else { 0x20 });
            }

            sys::view_port_update(view_port);
        }

        if let Some(b) = beacon.as_mut() {
            b.stop();
        }
        led(0);
        sys::view_port_enabled_set(view_port, false);
        sys::gui_remove_view_port(gui.as_ptr(), view_port);
        sys::view_port_free(view_port);
        sys::furi_message_queue_free(queue);
    }

    0
}
