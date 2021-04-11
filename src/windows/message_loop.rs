use crate::windows::*;

use futures::Future;
use futures::executor::*;
use futures::task::*;

use winapi::ctypes::c_int;
use winapi::shared::minwindef::*;
use winapi::um::winuser::*;

use std::cell::RefCell;
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
        // TODO: rendering & per-frame tasks
        LOCAL_POOL.with(|lp| {
            if let Ok(mut pool) = lp.try_borrow_mut() {
                pool.run_until_stalled();
            }
            // else we're recursively running a message loop within a task? probably an incredibly bad idea, but don't crash
        });
    }
}

/// [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage), but safe.
pub fn post_quit_message(exit_code: c_int) {
    unsafe { PostQuitMessage(exit_code) }
}

/// Run a future/task in the main message loop
pub fn spawn_local<F: Future<Output = ()> + 'static>(f: F) -> Result<(), SpawnError> {
    LOCAL_SPAWNER.with(|ls| ls.spawn_local(f))
}



#[allow(non_snake_case)] pub(crate) fn MAKEINTATOMW(atom: ATOM) -> *const u16 {
    atom as usize as *const _
}

thread_local! {
    static LOCAL_POOL : RefCell<LocalPool> = Default::default();
    static LOCAL_SPAWNER : LocalSpawner = LOCAL_POOL.with(|lp| lp.borrow().spawner());
}



#[test] fn message_loop_test_1() {
    const CODE : i32 = -9001;
    assert_eq!(CODE, std::thread::spawn(|| {
        post_quit_message(CODE);
        message_loop_until_wm_quit()
    }).join().unwrap());
}

#[test] fn message_loop_test_2() {
    const CODE : i32 = 42;
    assert_eq!(CODE, std::thread::spawn(|| {
        post_quit_message(CODE);
        message_loop_until_wm_quit()
    }).join().unwrap());
}
