pub mod shared;
pub use self::shared::*;

//
// Windows
//

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::{send_keystroke, send_slot, send_message, IGNORE_AREA_TOP};

//
// macOS
//

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{send_keystroke, send_slot, send_message, IGNORE_AREA_TOP};

//
// Linux
//

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::{send_keystroke, send_slot, send_message, IGNORE_AREA_TOP};
