//! Utility methods for interacting with Win32 messages, message queues, and loops thereof.
//!
//! ### See Also
//! * [About Messages and Message Queues](https://docs.microsoft.com/en-us/windows/win32/winmsg/about-messages-and-message-queues)
//! * [Using Messages and Message Queues](https://docs.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues)

use crate::windows::*;

use futures::Future;
use futures::executor::*;
use futures::task::*;

use winapi::ctypes::c_int;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::winuser::*;

use std::cell::RefCell;
use std::ptr::null_mut;



/// Run a message loop on this thread until [`WM_QUIT`](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-quit) is encountered.
///
/// Returns the `nExitCode` that was passed to [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage).
pub fn loop_until_wm_quit() -> c_int {
    loop {
        if let Some(exit) = loop_one_frame() {
            return exit;
        }
    }
}

/// Run a message loop on this thread once.
///
/// If [`WM_QUIT`](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-quit) is encountered, returns `Some(nExitCode)` based on what was passed to [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage).
pub fn loop_one_frame() -> Option<c_int> {
    let mut msg = unsafe { std::mem::zeroed() };
    while unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) } != 0 {
        match msg.message {
            WM_QUIT => return Some(msg.wParam as c_int),
            _other  => {},
        }
        unsafe { TranslateMessage(&msg) }; // generate WM_CHAR, WM_DEADCHAR, WM_UNICHAR, etc.
        unsafe { DispatchMessageW(&msg) }; // invoke WndProcs
    }

    TL.with(|tl|{
        // TODO: rendering & per-frame tasks
        if let Ok(mut pool) = tl.local_pool.try_borrow_mut() {
            pool.run_until_stalled();
        }
        // else we're recursively running a message loop within a task? probably an incredibly bad idea, but don't crash

        if let Ok(mut each_frame) = tl.each_frame.try_borrow_mut() {
            each_frame.append(&mut *tl.each_frame_pending.borrow_mut());
            let efa = EachFrameArgs {};
            retain_mut(&mut each_frame, |f| f(&efa));
        }
        // else we're recursively running a message loop within an each_frame callback? probably an incredibly bad idea, but don't crash
    });
    None
}

/// [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage), but safe.
pub fn post_quit(exit_code: c_int) {
    unsafe { PostQuitMessage(exit_code) }
}

/// Run a future/task in the current thread's message handling loop
pub fn spawn_local<F: Future<Output = ()> + 'static>(f: F) -> Result<(), SpawnError> {
    TL.with(|tl| tl.local_spawner.spawn_local(f))
}

/// Run logic in the current thread's message handling loop "each frame".
///
/// Read "each frame" as: roughly in sync with the refresh rate of one of your monitors, possibly skipping some if falling behind.
///
/// If the callback ever returns `false`, it will be unregistered and not called again.
pub fn each_frame(f: impl 'static + FnMut(&EachFrameArgs) -> bool) {
    TL.with(|tl| tl.each_frame_pending.borrow_mut().push(Box::new(f)));
}

/// A win32 message handler (provides a [`wndproc`](Self::wndproc))
pub trait Handler {
    /// A [`WNDPROC`](https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms633573(v=vs.85)) analog
    ///
    /// ### Safety
    ///
    /// * `hwnd` must be a valid window handle (older versions of windows may treat this as a raw, dereferencable pointer)
    /// * `wparam` / `lparam` may need to be valid pointers, depending on which `msg` is passed in.
    unsafe fn wndproc(&self, hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

/// Arguments to [`each_frame`]'s callbacks.
#[non_exhaustive]
pub struct EachFrameArgs {
    // TODO: time deltas?
}



struct ThreadLocal {
    local_pool:         RefCell<LocalPool>,
    local_spawner:      LocalSpawner,
    each_frame:         RefCell<Vec<Box<dyn FnMut(&EachFrameArgs) -> bool>>>,
    each_frame_pending: RefCell<Vec<Box<dyn FnMut(&EachFrameArgs) -> bool>>>,
}

impl Default for ThreadLocal {
    fn default() -> Self {
        let local_pool          = LocalPool::new();
        let local_spawner       = local_pool.spawner();
        let local_pool          = RefCell::new(local_pool);
        let each_frame          = Default::default();
        let each_frame_pending  = Default::default();
        Self { local_pool, local_spawner, each_frame, each_frame_pending }
    }
}

thread_local! { static TL : ThreadLocal = ThreadLocal::default(); }

fn retain_mut<T, F: FnMut(&mut T) -> bool>(vec: &mut Vec<T>, mut f: F) {
    let len = vec.len();
    let mut del = 0;
    let v = &mut **vec;
    for i in 0..len {
        if !f(&mut v[i]) {
            del += 1;
        } else if del > 0 {
            v.swap(i - del, i);
        }
    }
    vec.truncate(len - del);
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
