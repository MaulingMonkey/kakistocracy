#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))] fn main() {}

#[cfg(windows)] fn main() {
    use kakistocracy::windows::*;

    let stub    = OwnedWindow::create_stub("");
    let d3d     = d3d9::create_d3d(cfg!(debug_features)).unwrap();
    let _device = unsafe { d3d9::create_device_windowed(&d3d, &stub) }.unwrap();
}
