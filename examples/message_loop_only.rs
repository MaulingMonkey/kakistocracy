#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))] fn main() {}

#[cfg(windows)] fn main() {
    use kakistocracy::windows::*;

    message::post_quit(-42);
    let exit = message::loop_until_wm_quit();
    assert_eq!(exit, -42);
}
