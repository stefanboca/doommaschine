mod image_utils;
mod keys;

use maschine::{get_device, Color, Device, Event, EventContext};
use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

const DOOMGENERIC_RESX: u32 = 640;
const DOOMGENERIC_RESY: u32 = 400;

static mut START_TIME: Option<std::time::Instant> = None;
static mut DEVICE: Option<Arc<Mutex<Box<dyn Device>>>> = None;
static mut EVENT_CONTEXT: Option<Arc<Mutex<EventContext>>> = None;
static mut PAD_STATES: [bool; 16] = [false; 16];

extern "C" {
    pub static mut DG_ScreenBuffer: *mut u32;
}

#[no_mangle]
pub extern "C" fn DG_Init() {
    color_eyre::install().unwrap();

    let start_time = Some(std::time::Instant::now());
    let device = Some(Arc::new(Mutex::new(get_device().unwrap())));
    let event_context = Some(Arc::new(Mutex::new(EventContext::new())));
    unsafe {
        START_TIME = start_time;
        DEVICE = device;
        EVENT_CONTEXT = event_context;
    }

    std::thread::spawn(|| loop {
        if let Some(device) = unsafe { &DEVICE } {
            if let Some(event_context) = unsafe { &EVENT_CONTEXT } {
                let mut device = device.lock().unwrap();
                let mut event_context = event_context.lock().unwrap();
                device
                    .borrow_mut()
                    .tick(event_context.borrow_mut())
                    .unwrap();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    });
}

#[no_mangle]
pub extern "C" fn DG_DrawFrame() {
    let raw_img = unsafe {
        std::slice::from_raw_parts(
            DG_ScreenBuffer,
            (DOOMGENERIC_RESX * DOOMGENERIC_RESY) as usize,
        )
    };

    let mut img = image::imageops::resize(
        &image::RgbaImage::from_raw(
            DOOMGENERIC_RESX,
            DOOMGENERIC_RESY,
            raw_img
                .iter()
                .flat_map(|v| {
                    [
                        ((v >> 16) & 0xFF) as u8,
                        ((v >> 8) & 0xFF) as u8,
                        (v & 0xFF) as u8,
                        ((v >> 24) & 0xFF) as u8,
                    ]
                })
                .collect(),
        )
        .unwrap(),
        256,
        128,
        image::imageops::FilterType::CatmullRom,
    );
    image::imageops::colorops::dither(&mut img, &image_utils::BiLevelRgba);

    let displays_data = img.as_raw().split_at(4 * 256 * 64);

    if let Some(device) = unsafe { &DEVICE } {
        let mut device = device.lock().unwrap();

        device
            .get_display(0)
            .unwrap()
            .get_data_u8_mut()
            .copy_from_slice(displays_data.0);
        device
            .get_display(1)
            .unwrap()
            .get_data_u8_mut()
            .copy_from_slice(displays_data.1);
    }
}

#[no_mangle]
pub extern "C" fn DG_SleepMs(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

#[no_mangle]
pub extern "C" fn DG_GetTicksMs() -> u32 {
    let start_time = unsafe { START_TIME.unwrap() };
    std::time::Instant::now()
        .duration_since(start_time)
        .as_millis() as u32
}

#[no_mangle]
pub unsafe extern "C" fn DG_GetKey(
    pressed_out: *mut std::os::raw::c_int,
    key_out: *mut std::os::raw::c_uchar,
) -> std::os::raw::c_int {
    if let Some(event_context) = unsafe { &EVENT_CONTEXT } {
        let mut event_context = event_context.lock().unwrap();

        while !event_context.events.is_empty() {
            let event = event_context.events.pop_front().unwrap();
            if let Event::Pad(pad, velocity, _shift) = event {
                let pressed = velocity > 0;
                if let Some(device) = unsafe { &DEVICE } {
                    let mut device = device.lock().unwrap();
                    if pressed {
                        device.set_pad_led(pad, Color::new(0xFF, 0xFF, 0x00, 0x00));
                    } else {
                        device.set_pad_led(pad, Color::new(0xFF, 0x00, 0x00, 0x00));
                    }
                }

                if pressed != unsafe { PAD_STATES }[pad as usize] {
                    unsafe {
                        PAD_STATES[pad as usize] = pressed;
                    }
                    let key = match pad {
                        0 => Some(keys::USE),
                        1 => Some(keys::UPARROW),
                        3 => Some(keys::ENTER),
                        4 => Some(keys::LEFTARROW),
                        5 => Some(keys::FIRE),
                        6 => Some(keys::RIGHTARROW),
                        8 => Some(keys::STRAFE_L),
                        9 => Some(keys::DOWNARROW),
                        10 => Some(keys::STRAFE_R),
                        _ => None,
                    };
                    if let Some(key) = key {
                        unsafe {
                            *key_out = key;
                            *pressed_out = pressed as std::os::raw::c_int;
                            return 1;
                        }
                    }
                }
            }
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn DG_SetWindowTitle(title: *const ::std::os::raw::c_char) {
    let title = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            title as *const u8,
            libc::strlen(title) + 1,
        ))
    };
    println!("DG_SetWindowTitle: {title}");
}
