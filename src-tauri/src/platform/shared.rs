use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use tauri::Window;

mod key;
mod platform_accessor;

pub use key::{Key, KeyMode};
pub use platform_accessor::PlatformAccessor;

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

/// Determine whether the window is currently focused.
pub fn get_window_focused(window: &Window) -> bool {
    #[cfg(target_os = "windows")]
    {
        let focused_hwnd = unsafe { winapi::um::winuser::GetForegroundWindow() };
        if let Ok(hwnd) = window.hwnd().map(|hwnd| hwnd.0) {
            focused_hwnd as isize == hwnd
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}
