use crate::windows::*;

use wchar::wch_c;

use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::winuser::*;

use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr::null_mut;
use std::rc::Rc;



type AliveHandle = Rc<Cell<bool>>;

/// A safe wrapper around Win32 `HWND`s
#[derive(Clone)]
pub struct Window {
    hwnd:   HWND,
    alive:  AliveHandle,
}

/// Construction methods
impl Window {
    /// Create an invisible, un-enumerated, 1x1 [Message-Only](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#message-only-windows) window, that doesn't process any messages.
    pub fn create_stub(title: &str) -> Window {
        let hwnd = unsafe { CreateWindowExW(
            0,
            MAKEINTATOMW(*KAKISTOCRACY_STUB_WNDCLASS),
            title.encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
            WS_POPUP,
            CW_USEDEFAULT, CW_USEDEFAULT, 1, 1,
            HWND_MESSAGE, null_mut(), get_module_handle_exe(), null_mut()
        )};
        assert!(!hwnd.is_null(), "Unable to create message-only HWND");
        Window::find(hwnd)
    }
}

/// Public methods
impl Window {
    /// `true` until the window has been [`DestroyWindow()`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)ed
    pub fn is_alive(&self) -> bool { self.alive.get() }

    /// `Some(hwnd)` until the window has been [`DestroyWindow()`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)ed
    pub fn hwnd(&self) -> Option<HWND> { self.alive.get().then(|| self.hwnd) }

    /// [`DestroyWindow`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)
    pub fn destroy(self) -> Result<(), Error> {
        if !self.alive.get() {
            Err(Error::new("Window::destroy", "", 0, "window already destroyed"))
        } else if unsafe { DestroyWindow(self.hwnd) } != FALSE {
            Ok(())
        } else {
            Err(Error::new_gle("DestroyWindow", get_last_error(), ""))
        }
    }
}

/// Private methods
impl Window {
    pub(crate) fn find(hwnd: HWND) -> Self {
        let alive = WINDOWS.with(|w| w.borrow().get(&hwnd).expect("Window::find: HWND not registered, does the wndproc properly call on_create/on_destroy in WM_CREATE/WM_DESTROY?").clone());
        Self { hwnd, alive }
    }

    pub(crate) fn on_create(hwnd: HWND) -> Self {
        let alive = AliveHandle::new(Cell::new(true));
        let prev = WINDOWS.with(|w| w.borrow_mut().insert(hwnd, alive.clone()));
        assert!(prev.is_none(), "Window::on_create: HWND was previously registered");
        Self { hwnd, alive }
    }

    pub(crate) fn on_destroy(hwnd: HWND) {
        let e = WINDOWS.with(|w| w.borrow_mut().remove(&hwnd));
        let e = e.expect("Window::on_destroy: HWND was not registered");
        e.set(false); // window is no longer alive
    }

    fn cmp_contents(&self) -> (HWND, *const c_void) {
        (self.hwnd, self.alive.as_ptr().cast())
    }
}

impl Debug for Window {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Window")
            .field("hwnd", &self.hwnd)
            .field("alive", &self.alive.get())
            .finish()
    }
}

impl PartialEq  for Window { fn eq(&self, other: &Self) -> bool { self.cmp_contents().eq(&other.cmp_contents()) }}
impl Eq         for Window {}
impl PartialOrd for Window { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.cmp_contents().partial_cmp(&other.cmp_contents()) }}
impl Ord        for Window { fn cmp(&self, other: &Self) -> Ordering { self.cmp_contents().cmp(&other.cmp_contents()) }}
impl Hash       for Window { fn hash<H: Hasher>(&self, state: &mut H) { self.cmp_contents().hash(state) }}

// TODO: comparisons against HWNDs?



thread_local! { static WINDOWS : RefCell<HashMap<HWND, AliveHandle>> = Default::default(); }

lazy_static::lazy_static! {
    static ref KAKISTOCRACY_STUB_WNDCLASS : ATOM = {
        let atom = unsafe { RegisterClassW(&WNDCLASSW {
            style:          0,
            lpfnWndProc:    Some(stub_window_proc),
            cbClsExtra:     0,
            cbWndExtra:     0,
            hInstance:      get_module_handle_exe(),
            hIcon:          null_mut(),
            hCursor:        null_mut(),
            hbrBackground:  null_mut(),
            lpszMenuName:   null_mut(),
            lpszClassName:  wch_c!("kakistocracy-stub-window").as_ptr(),
        })};
        assert!(atom != 0, "Unable to register \"kakistocracy-window-message-only\" window class: 0x{:08x}", get_last_error());
        atom
    };
}

unsafe extern "system" fn stub_window_proc(hwnd: HWND, msg: DWORD, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE   => drop(Window::on_create(hwnd)),
        WM_DESTROY  => drop(Window::on_destroy(hwnd)),
        _other      => {},
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}



#[test] fn stub_window_tests() {
    let w = Window::create_stub("example"); assert!( w.is_alive());
    w.clone().destroy().unwrap();           assert!(!w.is_alive());
    w.clone().destroy().unwrap_err();       assert!(!w.is_alive());
    w.clone().destroy().unwrap_err();       assert!(!w.is_alive());
}
