use std::time::Duration;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use tauri::Window;

use crate::data::{Bounds, Point};

#[derive(Debug)]
pub enum KeyMode {
    Press,
    Hold,
    Release,
}

// For visual recognition: Avoids mouse clicks outside the window by ignoring monster names that are too close to the bottom of the GUI
pub const IGNORE_AREA_BOTTOM: u32 = 110;
//>100 <230 where we get the red announcement for already targetted mob
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
            drop(window.eval(format!("keyboardEvent('press', '{0}');", key).as_str()))
        }

        KeyMode::Hold => drop(window.eval(format!("keyboardEvent('hold', '{0}');", key).as_str())),

        KeyMode::Release => {
            drop(window.eval(format!("keyboardEvent('release', '{0}');", key).as_str()))
        }
    }
}

pub fn send_slot_eval(window: &Window, slot_bar_index: usize, k: usize) {
    drop(window.eval(format!("sendSlot({0}, {1})", slot_bar_index, k).as_str()))
}

pub fn eval_mob_click(window: &Window, pos: Point) {
    drop(
        window.eval(
            format!(
                "mouseEvent('moveClick', {0}, {1}, {{checkMob: true}});",
                pos.x, pos.y
            )
            .as_str(),
        ),
    );
}

pub fn eval_simple_click(window: &Window, pos: Point) {
    drop(window.eval(format!("mouseEvent('moveClick', {0}, {1});", pos.x, pos.y).as_str()));
}

pub fn eval_send_message(window: &Window, text: &str) {
    drop(window.eval(format!("setInputChat({0})", text).as_str()));
}

pub fn eval_draw_bounds(window: &Window, bounds: Bounds) {
    drop(
        window.eval(
            format!(
                "drawBounds({0}, {1}, {2}, {3});",
                bounds.x, bounds.y, bounds.w, bounds.h
            )
            .as_str(),
        ),
    );
}
