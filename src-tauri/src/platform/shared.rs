use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::time::Duration;
use tauri::Window;

use crate::data::Point;

#[derive(Debug)]
pub enum KeyMode {
    Press,
    Hold,
    Release,
}

// For visual recognition: Avoids mouse clicks outside the window by ignoring monster names that are too close to the bottom of the GUI
pub const IGNORE_AREA_BOTTOM: u32 = 110;

/// Get the native window id.
pub fn get_window_id(window: &Window) -> Option<u64> {
    #[allow(unused_variables)]
    match window.raw_window_handle() {
        RawWindowHandle::Xlib(handle) => Some(handle.window as u64),
        RawWindowHandle::Win32(handle) => Some(handle.hwnd as u64),
        RawWindowHandle::AppKit(handle) => {
            #[cfg(target_os = "macos")]
            unsafe {
                use std::ffi::c_void;
                let ns_window_ptr = handle.ns_window as *const c_void;
                libscreenshot::platform::macos::macos_helper::ns_window_to_window_id(ns_window_ptr)
                    .map(|id| id as u64)
            }
            #[cfg(not(target_os = "macos"))]
            unreachable!()
        }
        _ => Some(0_u64),
    }
}

pub fn eval_send_key(window: &Window, key: &str, mode: KeyMode) {
    match mode {
        KeyMode::Press => {
            drop(window.eval(format!("
                document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keydown', {{'key': '{0}'}}))
                document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keyup', {{'key': '{0}'}}))"
            , key).as_str()))
        },
        KeyMode::Hold => {
            drop(window.eval(format!("
                document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keydown', {{'key': '{0}'}}))"
            , key).as_str()))
        },
        KeyMode::Release => {
            drop(window.eval(format!("
                document.querySelector('canvas').dispatchEvent(new KeyboardEvent('keyup', {{'key': '{0}'}}))"
            , key).as_str()))
        },
    }
}

pub fn send_slot_eval(window: &Window, slot_bar_index: usize, k: usize) {
    eval_send_key(
        window,
        format!("F{}", slot_bar_index + 1).to_string().as_str(),
        KeyMode::Press,
    );
    eval_send_key(window, k.to_string().as_str(), KeyMode::Press);
    std::thread::sleep(Duration::from_millis(100));
}

pub fn eval_mouse_click_at_point(window: &Window, pos: Point) {
    drop(
        window.eval(
            format!(
                "
        document.querySelector('canvas').dispatchEvent(new MouseEvent('mousedown', {{
            clientX: {0},
            clientY: {1}
        }}))

        document.querySelector('canvas').dispatchEvent(new MouseEvent('mouseup', {{
            clientX: {0},
            clientY: {1}
        }}))",
                pos.x, pos.y
            )
            .as_str(),
        ),
    );
}

pub fn eval_mouse_move(window: &Window, pos: Point) {
    drop(
        window.eval(
            format!(
                "
        document.querySelector('canvas').dispatchEvent(new MouseEvent('mousemove', {{
            clientX: {0},
            clientY: {1}
        }}))",
                pos.x, pos.y
            )
            .as_str(),
        ),
    );
}

pub fn eval_mob_click(window: &Window, pos: Point) {
    eval_mouse_move(window, pos);
    std::thread::sleep(Duration::from_millis(30));
    drop(
        window.eval(
            format!(
                "
                    if (document.body.style.cursor.indexOf('curattack') > 0) {{
                        document.querySelector('canvas').dispatchEvent(new MouseEvent('mousedown', {{
                            clientX: {0},
                            clientY: {1}
                        }}))

                        document.querySelector('canvas').dispatchEvent(new MouseEvent('mouseup', {{
                            clientX: {0},
                            clientY: {1}
                        }}))
                    }}
                    global.gc();;",
                pos.x, pos.y
            )
            .as_str(),
        ),
    );
}

pub fn eval_send_message(window: &Window, text: &str) {
    drop(
        window.eval(
            format!(
                "
    document.querySelector('input').value = '{0}';
    document.querySelector('input').select();",
                text
            )
            .as_str(),
        ),
    );
}
