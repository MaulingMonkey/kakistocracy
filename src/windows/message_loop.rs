use winapi::ctypes::c_int;
use winapi::um::winuser::*;

use std::ptr::null_mut;



/// Run a message loop on this thread until [`WM_QUIT`](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-quit) is encountered.
///
/// Returns the `nExitCode` that was passed to [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage).
pub fn message_loop_until_wm_quit() -> c_int {
    loop {
        let mut msg = unsafe { std::mem::zeroed() };
        while unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) } != 0 {
            match msg.message {
                WM_QUIT => return msg.wParam as c_int,
                _other  => {},
            }
            unsafe { TranslateMessage(&msg) }; // generate WM_CHAR, WM_DEADCHAR, WM_UNICHAR, etc.
            unsafe { DispatchMessageW(&msg) }; // invoke WndProcs
        }
        // TODO: on thread idle processing
    }
}

/// [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage), but safe.
pub fn post_quit_message(exit_code: c_int) {
    unsafe { PostQuitMessage(exit_code) }
}
