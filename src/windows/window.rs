//use crate::utility::*;
use crate::windows::*;

use wchar::wch_c;

use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::winuser::*;

use std::any::{Any, TypeId};
use std::cell::*;
use std::cmp::Ordering;
use std::collections::*;
use std::ffi::c_void;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::*;
use std::ptr::*;
use std::rc::*;



/// A safe wrapper around Win32 `HWND`s
#[derive(Clone)]
pub struct Window {
    hwnd:   HWND,
    unique: Rc<()>,
    assoc:  Weak<Assoc>,
}

/// Construction methods
impl Window {
    pub fn null() -> Window { Window { hwnd: null_mut(), unique: Rc::new(()), assoc: Weak::new() } }

    /// Create a fullscreen window.  Unlike a maximized window, it has no title bar, and obscures the taskbar.
    ///
    /// ### Arguments
    /// * `monitor` - 0 indicates the primary monitor, 1+ indicates the monitor *number* (**NOT INDEX**).  That is, `1` == `\\.\DISPLAY1` == probably the primary monitor?
    /// * `title`   - The title to give the window, typically shown in the Alt+Tab menu.
    pub fn create_fullscreen(monitor: usize, title: &str) -> Result<Self, Error> {
        // XXX: Should I always use an index instead? monitors appear to be sorted when I call enum_display_monitors these days...
        let rc_monitor = match monitor {
            0 => MONITORINFO::get_primary()?.rcMonitor,
            n => {
                let expected_monitor_name = format!(r"\\.\DISPLAY{}", n).encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
                let mut monitor : Result<RECT, Error> = Err(Error::new("Window::create_fullscreen", "", 0, "monitor matched no expected monitor"));
                let mut no = 0;
                enum_display_monitors((), None, |hmonitor, _hdc, _rect| {
                    no += 1;
                    let info = unsafe { MONITORINFOEXW::get(hmonitor) };
                    match info {
                        Ok(info) => {
                            if no == n && monitor.is_err() {
                                monitor = Ok(info.rcMonitor);
                            } else if info.szDevice.starts_with(&expected_monitor_name[..]) {
                                monitor = Ok(info.rcMonitor);
                            }
                            true
                        },
                        Err(err) => {
                            monitor = Err(err);
                            false
                        },
                    }
                });
                monitor?
            },
        };

        let hwnd = unsafe { CreateWindowExW(
            0,
            MAKEINTATOMW(*KAKISTOCRACY_WNDCLASS),
            title.encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
            WS_POPUP | WS_VISIBLE,
            rc_monitor.left, rc_monitor.top, rc_monitor.right - rc_monitor.left, rc_monitor.bottom - rc_monitor.top,
            null_mut(), null_mut(), get_module_handle_exe(), null_mut()
        )};
        assert!(!hwnd.is_null(), "Unable to create fullscreen HWND");
        Ok(Window::find(hwnd))
    }

    pub fn create_at(title: &str, area: impl IntoRect) -> Result<Self, Error> {
        let area = area.into();
        let hwnd = unsafe { CreateWindowExW(
            0,
            MAKEINTATOMW(*KAKISTOCRACY_WNDCLASS),
            title.encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            area.left, area.top, area.right - area.left, area.bottom - area.top,
            null_mut(), null_mut(), get_module_handle_exe(), null_mut()
        )};
        assert!(!hwnd.is_null(), "Unable to create HWND");
        Ok(Window::find(hwnd))
    }
}

/// Public methods
impl Window {
    /// `true` until the window has been [`DestroyWindow()`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)ed
    pub fn is_alive(&self) -> bool { self.assoc.strong_count() > 0 }

    /// `self.is_alive()` && [`IsWindowVisible`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-iswindowvisible)
    ///
    /// **NOTE:** this may be `true` even if the window is obscured / minimized
    pub fn is_visible(&self) -> bool { self.is_alive() && unsafe { IsWindowVisible(self.hwnd) != 0 } }

    /// `self.is_alive()` && [`IsIconic`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-isiconic) (minimized)
    pub fn is_iconic(&self) -> bool { self.is_alive() && unsafe { IsIconic(self.hwnd) != 0 } }

    /// `self.is_alive()` && [`IsWindowVisible`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-iswindowvisible) && not [`IsIconic`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-isiconic) (minimized)
    pub fn should_render(&self) -> bool { self.is_alive() && !self.is_iconic() }

    /// `self.is_alive()` && [`GetClientRect`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclientrect)
    pub fn get_client_rect(&self) -> Result<RECT, Error> {
        let mut rect = unsafe { std::mem::zeroed() };
        if !self.is_alive() {
            Err(Error::new("Window::get_client_rect", "", 0, "window already destroyed"))
        } else if unsafe { GetClientRect(self.hwnd, &mut rect) } != FALSE {
            Ok(rect)
        } else {
            Error::last("GetClientRect", "")
        }
    }

    /// `Some(hwnd)` until the window has been [`DestroyWindow()`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)ed
    pub fn hwnd(&self) -> Option<HWND> { self.is_alive().then(|| self.hwnd) }

    /// [`DestroyWindow`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)
    pub fn destroy(self) -> Result<(), Error> { self.destroy_ref() }

    /// [`DestroyWindow`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)
    pub fn destroy_ref(&self) -> Result<(), Error> {
        if !self.is_alive() {
            Err(Error::new("Window::destroy", "", 0, "window already destroyed"))
        } else if unsafe { DestroyWindow(self.hwnd) } != FALSE {
            Ok(())
        } else {
            Err(Error::new_gle("DestroyWindow", get_last_error(), ""))
        }
    }

    pub fn has<T: Any>(&self) -> bool {
        self.assoc.upgrade().map_or(false, |a| a.borrow().contains_key(&TypeId::of::<T>()))
    }

    pub fn set<T: Any>(&self, value: T) {
        if let Some(assoc) = self.assoc.upgrade() {
            assoc.borrow_mut().insert(TypeId::of::<T>(), Rc::new(value));
        }
    }

    pub fn get<T: Any>(&self) -> Option<impl Deref<Target = T>> {
        self.assoc.upgrade()?.borrow().get(&TypeId::of::<T>()).map(|rc_any| rc_any.clone().downcast().unwrap())
    }

    pub fn get_or_default<T: Default + Any>(&self) -> Option<impl Deref<Target = T>> {
        let assoc = self.assoc.upgrade()?;
        let mut assoc = assoc.borrow_mut();
        let assoc = &mut assoc;
        let rc_any = match assoc.entry(TypeId::of::<T>()) {
            hash_map::Entry::Occupied(e) => e.get().clone(),
            hash_map::Entry::Vacant(e) => e.insert(Rc::new(T::default())).clone(),
        };
        Some(rc_any.downcast().unwrap())
    }
}

/// Private methods
impl Window {
    pub(crate) fn find(hwnd: HWND) -> Self {
        WINDOWS.with(|w| {
            let w = w.borrow();
            let pw = w.get(&hwnd).expect("Window::find: HWND not registered, does the wndproc properly call on_create/on_destroy in WM_CREATE/WM_DESTROY?");
            Self { hwnd, unique: pw.unique.clone(), assoc: Rc::downgrade(&pw.assoc) }
        })
    }

    pub(crate) fn on_create(hwnd: HWND) -> Self {
        let unique = Rc::new(());
        let assoc = Rc::new(Assoc::default());
        let assocw = Rc::downgrade(&assoc);

        let pw = PerWindow { unique: unique.clone(), assoc };
        let prev = WINDOWS.with(|w| w.borrow_mut().insert(hwnd, pw));
        assert!(prev.is_none(), "Window::on_create: HWND was previously registered");
        Self { hwnd, unique, assoc: assocw }
    }

    pub(crate) fn on_destroy(hwnd: HWND) {
        let e = WINDOWS.with(|w| w.borrow_mut().remove(&hwnd));
        let _ = e.expect("Window::on_destroy: HWND was not registered");
    }

    //pub(crate) fn list() -> Vec<Self> {
    //    WINDOWS.with(|w| w.borrow().iter().map(|(hwnd, pw)| Self { hwnd: *hwnd, unique: pw.unique.clone(), assoc: Rc::downgrade(&pw.assoc) }).collect())
    //}

    fn cmp_contents(&self) -> (HWND, *const c_void) {
        (self.hwnd, Rc::as_ptr(&self.unique).cast())
    }
}

impl Debug for Window {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Window")
            .field("hwnd", &self.hwnd)
            .field("alive", &self.is_alive())
            .finish()
    }
}

impl PartialEq  for Window { fn eq(&self, other: &Self) -> bool { self.cmp_contents().eq(&other.cmp_contents()) }}
impl Eq         for Window {}
impl PartialOrd for Window { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.cmp_contents().partial_cmp(&other.cmp_contents()) }}
impl Ord        for Window { fn cmp(&self, other: &Self) -> Ordering { self.cmp_contents().cmp(&other.cmp_contents()) }}
impl Hash       for Window { fn hash<H: Hasher>(&self, state: &mut H) { self.cmp_contents().hash(state) }}

// TODO: comparisons against HWNDs?



#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnedWindow(Window);

/// Construction methods
impl OwnedWindow {
    pub fn new(window: Window) -> Self { Self(window) }

    /// Create an invisible, un-enumerated, 1x1 [Message-Only](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#message-only-windows) window, that doesn't process any messages.
    pub fn create_stub(title: &str) -> Self {
        let hwnd = unsafe { CreateWindowExW(
            0,
            MAKEINTATOMW(*KAKISTOCRACY_STUB_WNDCLASS),
            title.encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
            WS_POPUP,
            CW_USEDEFAULT, CW_USEDEFAULT, 1, 1,
            HWND_MESSAGE, null_mut(), get_module_handle_exe(), null_mut()
        )};
        assert!(!hwnd.is_null(), "Unable to create message-only HWND");
        Self(Window::find(hwnd))
    }
}

/// Public methods
impl OwnedWindow {
    /// Get the inner window without destroying it
    pub fn leak(mut self) -> Window { std::mem::replace(&mut self.0, Window::null()) }
}

impl Drop for OwnedWindow {
    fn drop(&mut self) {
        let _ = self.0.destroy_ref();
    }
}

impl Debug for OwnedWindow {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("OwnedWindow")
            .field("hwnd", &self.0.hwnd)
            .field("alive", &self.0.is_alive())
            .finish()
    }
}

impl Deref          for OwnedWindow { fn deref    (&    self) -> &    Self::Target { &    self.0 } type Target = Window; }
impl DerefMut       for OwnedWindow { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 } }
impl AsRef<Window>  for OwnedWindow { fn as_ref   (&    self) -> &          Window { &    self.0 } }
impl AsMut<Window>  for OwnedWindow { fn as_mut   (&mut self) -> &mut       Window { &mut self.0 } }



type Assoc = RefCell<HashMap<TypeId, Rc<dyn Any>>>;

struct PerWindow {
    unique: Rc<()>,
    assoc:  Rc<Assoc>,
}

thread_local! { static WINDOWS : RefCell<HashMap<HWND, PerWindow>> = Default::default(); }

lazy_static::lazy_static! {
    static ref KAKISTOCRACY_STUB_WNDCLASS : ATOM = {
        let atom = unsafe { RegisterClassW(&WNDCLASSW {
            style:          0,
            lpfnWndProc:    Some(stub_window_proc),
            cbClsExtra:     0,
            cbWndExtra:     0,
            hInstance:      get_module_handle_exe(),
            hIcon:          null_mut(),
            hCursor:        LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground:  null_mut(),
            lpszMenuName:   null_mut(),
            lpszClassName:  wch_c!("kakistocracy-stub-window").as_ptr(),
        })};
        assert!(atom != 0, "Unable to register \"kakistocracy-stub-window\" window class: 0x{:08x}", get_last_error());
        atom
    };

    static ref KAKISTOCRACY_WNDCLASS : ATOM = {
        let atom = unsafe { RegisterClassW(&WNDCLASSW {
            style:          0,
            lpfnWndProc:    Some(stub_window_proc),
            cbClsExtra:     0,
            cbWndExtra:     0,
            hInstance:      get_module_handle_exe(),
            hIcon:          null_mut(),
            hCursor:        LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground:  null_mut(),
            lpszMenuName:   null_mut(),
            lpszClassName:  wch_c!("kakistocracy-window").as_ptr(),
        })};
        assert!(atom != 0, "Unable to register \"kakistocracy-window\" window class: 0x{:08x}", get_last_error());
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
    let w = OwnedWindow::create_stub("example").leak(); assert!( w.is_alive());
    w.clone().destroy().unwrap();                       assert!(!w.is_alive());
    w.clone().destroy().unwrap_err();                   assert!(!w.is_alive());
    w.clone().destroy().unwrap_err();                   assert!(!w.is_alive());
}
