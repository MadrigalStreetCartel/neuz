use tauri::Window;

pub struct PlatformAccessor<'a> {
    pub window: &'a Window,
    pub mouse: mouse_rs::Mouse,
}
