use tauri::Window;

pub struct PlatformAccessor<'a> {
    pub window: &'a Window,
}
