use super::{on_hwnd_creating, on_hwnd_destroyed};

use winapi::ctypes::c_int;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::winuser::*;
use winapi::um::processthreadsapi::*;

use std::ptr::*;



pub(crate) fn ensure_registered() {
    thread_local! { static HOOKS : Hooks = Hooks::register(); }
    HOOKS.with(|_| {});
}

struct Hooks {
    call:   HHOOK,
    ret:    HHOOK,
}

impl Hooks {
    fn register() -> Self {
        let tid  = unsafe { GetCurrentThreadId() };

        let en   = unsafe { EnumThreadWindows(tid, Some(Self::register_existing_thread_windows), 0) };
        assert!(en != FALSE, "hwnd::Hooks::register(): EnumThreadWindows(...) failed to register existing windows");

        let call = unsafe { SetWindowsHookExW(WH_CALLWNDPROC,    Some(Self::before_wndproc), null_mut(), tid) };
        assert!(!call.is_null(), "hwnd::Hooks::register(): SetWindowsHookExW(WH_CALLWNDPROC, ...) failed to register hook");

        let ret  = unsafe { SetWindowsHookExW(WH_CALLWNDPROCRET, Some(Self::after_wndproc),  null_mut(), tid) };
        assert!(!ret .is_null(), "hwnd::Hooks::register(): SetWindowsHookExW(WH_CALLWNDPROCRET, ...) failed to register hook");

        Hooks { call, ret }
    }

    unsafe extern "system" fn register_existing_thread_windows(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        eprintln!("register_existing_thread_windows: {:08p}", hwnd);
        on_hwnd_creating(hwnd);
        EnumChildWindows(hwnd, Some(Self::register_existing_thread_windows), 0)
    }

    unsafe extern "system" fn before_wndproc(ncode: c_int, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms644975(v=vs.85)
        if ncode == HC_ACTION {
            let _current_thread = wparam != 0;
            let call = &*(lparam as *const CWPSTRUCT); // "The CallWndProc hook procedure can examine the message, but it cannot modify it."
            match call.message {
                WM_NCCREATE => { // https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-nccreate
                    eprintln!("WM_NCCREATE: {:08p}", call.hwnd);
                    on_hwnd_creating(call.hwnd);
                },
                WM_CREATE => { // https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-create
                    eprintln!("WM_CREATE:   {:08p}", call.hwnd);
                //    on_hwnd_creating(call.hwnd);
                },
                _other => {},
            }
        }
        CallNextHookEx(null_mut(), ncode, wparam, lparam)
    }

    unsafe extern "system" fn after_wndproc(ncode: c_int, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nc-winuser-hookproc
        if ncode == HC_ACTION {
            let _current_thread = wparam != 0;
            let ret = &*(lparam as *const CWPRETSTRUCT);
            match ret.message {
                WM_DESTROY => { // https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-destroy
                    eprintln!("WM_DESTROY:   {:08p}", ret.hwnd);
                //    on_hwnd_destroyed(ret.hwnd);
                },
                WM_NCDESTROY => { // https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-ncdestroy
                    eprintln!("WM_NCDESTROY: {:08p}", ret.hwnd);
                    on_hwnd_destroyed(ret.hwnd);
                },
                _other => {},
            }
        }
        CallNextHookEx(null_mut(), ncode, wparam, lparam)
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        let call = unsafe { UnhookWindowsHookEx(self.call) != 0 };
        let ret  = unsafe { UnhookWindowsHookEx(self.ret ) != 0 };
        assert!(call, "hwnd::Hooks::drop(&mut self): UnhookWindowsHookEx(self.call, ...) failed to unregister hook");
        assert!(ret , "hwnd::Hooks::drop(&mut self): UnhookWindowsHookEx(self.ret, ...) failed to unregister hook");
    }
}
