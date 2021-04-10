#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))] fn main() {}

#[cfg(windows)] fn main() {
    use kakistocracy::windows::*;

    post_quit_message(-42);
    let exit = message_loop_until_wm_quit();
    assert_eq!(exit, -42);
}
