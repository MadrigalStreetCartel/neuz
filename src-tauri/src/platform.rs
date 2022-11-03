pub mod shared;
pub use self::shared::*;

//
// Windows
//

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::IGNORE_AREA_TOP;

//
// macOS
//

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::IGNORE_AREA_TOP;

//
// Linux
//

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::IGNORE_AREA_TOP;
