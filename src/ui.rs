use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};

use flipperzero_sys as sys;

use crate::crosseasy::Mode;
use crate::dolphin;

pub const FULLSCREEN: sys::GuiLayer = sys::GuiLayerFullscreen;

pub const SCREEN_MENU: u8 = 0;
pub const SCREEN_BROADCAST: u8 = 1;

pub const MENU_LEN: u8 = 2;

pub struct Model {
    pub screen: AtomicU8,
    pub menu_index: AtomicU8,
    pub frame: AtomicU32,
    pub error: AtomicBool,
}

impl Model {
    pub const fn new() -> Self {
        Self {
            screen: AtomicU8::new(SCREEN_MENU),
            menu_index: AtomicU8::new(0),
            frame: AtomicU32::new(0),
            error: AtomicBool::new(false),
        }
    }
}

pub fn mode_for_index(index: u8) -> Mode {
    match index {
        1 => Mode::Trigger,
        _ => Mode::SoundBoost,
    }
}

// safety: ctx is a *const Model that outlives the view port
pub unsafe extern "C" fn draw_callback(canvas: *mut sys::Canvas, ctx: *mut c_void) {
    let model = unsafe { &*(ctx as *const Model) };
    unsafe {
        if model.error.load(Ordering::Relaxed) {
            draw_error(canvas);
        } else if model.screen.load(Ordering::Relaxed) == SCREEN_BROADCAST {
            draw_broadcast(canvas, model);
        } else {
            draw_menu(canvas, model);
        }
    }
}

unsafe fn draw_error(canvas: *mut sys::Canvas) {
    unsafe {
        sys::canvas_set_font(canvas, sys::FontPrimary);
        sys::canvas_draw_str(canvas, 2, 12, c"CrossEasy Beacon".as_ptr());
        sys::canvas_draw_line(canvas, 0, 16, 127, 16);
        sys::canvas_set_font(canvas, sys::FontSecondary);
        sys::canvas_draw_str(canvas, 2, 34, c"Bluetooth unavailable.".as_ptr());
        sys::canvas_draw_str(canvas, 2, 46, c"Enable BT, then reopen.".as_ptr());
    }
}

unsafe fn draw_menu(canvas: *mut sys::Canvas, model: &Model) {
    let selected = model.menu_index.load(Ordering::Relaxed);
    unsafe {
        sys::canvas_set_font(canvas, sys::FontPrimary);
        sys::canvas_draw_str(canvas, 2, 12, c"CrossEasy Beacon".as_ptr());
        sys::canvas_draw_line(canvas, 0, 16, 127, 16);

        sys::canvas_set_font(canvas, sys::FontSecondary);
        for i in 0..MENU_LEN {
            let top = 21 + i as i32 * 15;
            if i == selected {
                sys::canvas_draw_box(canvas, 1, top, 126, 13);
                sys::canvas_set_color(canvas, sys::ColorWhite);
            }
            sys::canvas_draw_str(canvas, 6, top + 10, mode_for_index(i).label().as_ptr());
            sys::canvas_set_color(canvas, sys::ColorBlack);
        }
    }
}

unsafe fn draw_broadcast(canvas: *mut sys::Canvas, model: &Model) {
    let mode = mode_for_index(model.menu_index.load(Ordering::Relaxed));
    unsafe {
        // layout mirrors the firmware nfc emulate screen (nfc_protocol_support.c):
        // dolphin at 0,3 with a bold title centred at 90,26 and detail below
        sys::canvas_draw_xbm(
            canvas,
            0,
            3,
            dolphin::WIDTH as usize,
            dolphin::HEIGHT as usize,
            dolphin::DATA.as_ptr(),
        );

        sys::canvas_set_font(canvas, sys::FontPrimary);
        sys::canvas_draw_str_aligned(
            canvas,
            90,
            26,
            sys::AlignCenter,
            sys::AlignCenter,
            c"Broadcasting".as_ptr(),
        );

        sys::canvas_set_font(canvas, sys::FontSecondary);
        sys::canvas_draw_str_aligned(
            canvas,
            90,
            42,
            sys::AlignCenter,
            sys::AlignCenter,
            mode.short_label().as_ptr(),
        );
    }
}

// safety: ctx is a *mut FuriMessageQueue sized for InputEvent
pub unsafe extern "C" fn input_callback(event: *mut sys::InputEvent, ctx: *mut c_void) {
    let queue = ctx as *mut sys::FuriMessageQueue;
    unsafe {
        sys::furi_message_queue_put(queue, event as *const c_void, 0);
    }
}
