fn main() {
    std::fs::create_dir_all("../build").expect("Directory creation failed");
    tauri_build::build()
}
