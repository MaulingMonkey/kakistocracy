//! Monitor selection and enumeration utilities

use crate::windows::{Error, RectExt};

use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::winnt::LONG;
use winapi::um::winuser::*;

use std::ptr::*;



// Dev notes:
//
// It's not clear at all when HMONITORs are invalidated.
// Modern windows has probably made these types into validateable handles, instead of real pointers.



/// [`EnumDisplayMonitors`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors)
///
/// ### Returns
/// * `true` if "successful"
/// * `false` if `proc` returned `false` for any monitor
pub(crate) fn enum_display_monitors<F: FnMut(HMONITOR, HDC, &RECT) -> bool>(hdc: (), clip: impl Into<Option<RECT>>, mut proc: F) -> bool {
    unsafe extern "system" fn imp<F: FnMut(HMONITOR, HDC, &RECT) -> bool>(monitor: HMONITOR, hdc: HDC, clip_monitor_intersect: LPRECT, proc: LPARAM) -> BOOL {
        let proc = proc as *mut F;
        let proc = &mut *proc;

        let continue_ = proc(monitor, hdc, &*clip_monitor_intersect);
        BOOL::from(continue_)
    }

    let _ = hdc; let hdc = null_mut();
    let clip = clip.into(); let clip = clip.as_ref().map_or_else(|| null(), |c| c as *const _);
    let success = unsafe { EnumDisplayMonitors(hdc, clip, Some(imp::<F>), &mut proc as *mut F as isize) } != FALSE;
    success
}

/// [`EnumDisplayMonitors`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors) filtered into a [`Vec`]
#[allow(dead_code)]
pub(crate) fn collect_display_monitors<R, F: FnMut(HMONITOR, HDC, &RECT) -> R>(hdc: (), clip: impl Into<Option<RECT>>, mut proc: F) -> Vec<R> {
    let mut v = Vec::new();
    let success = enum_display_monitors(hdc, clip, |hmonitor, hdc, rect|{ v.push(proc(hmonitor, hdc, rect)); true });
    debug_assert!(success, "enum_display_monitors failed unexpectedly");
    v
}

/// Get the primary monitor of the system.  A good monitor to default to for fullscreen game windows.
#[allow(dead_code)]
pub(crate) fn get_primary_monitor() -> HMONITOR {
    // As recommended by https://devblogs.microsoft.com/oldnewthing/20070809-00/?p=25643
    unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) }
}



#[test] fn test_collect_monitors() {
    let monitors = collect_display_monitors((), None, |_monitor, _dc, rect| *rect);
    assert!(monitors.len() > 0);
    for monitor in monitors.iter().copied() {
        let w = monitor.right - monitor.left;
        let h = monitor.bottom - monitor.top;
        assert!(w * h > 100 * 100);
    }
}

#[test] fn test_get_monitor() {
    let primary_monitor_area = MONITORINFOEXW::get_primary().unwrap();
    assert_eq!(0, primary_monitor_area.rcMonitor.left);
    assert_eq!(0, primary_monitor_area.rcMonitor.top);
    assert_ne!(0, primary_monitor_area.rcMonitor.right);
    assert_ne!(0, primary_monitor_area.rcMonitor.bottom);
}



/// Marker trait for `MONITORINFO`/`MONITORINFOEXA`/`MONITORINFOEXW`
pub(crate) trait MonitorInfo : Sized {
    /// [`GetMonitorInfo`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow) for the primary monitor.
    /// Useful for creating e.g. borderless fullscreen game windows or centering game launchers.
    fn get_primary() -> Result<Self, Error> { unsafe { Self::get(get_primary_monitor()) } }

    /// [`GetMonitorInfo`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow)
    unsafe fn get(monitor: HMONITOR) -> Result<Self, Error>;
}

impl MonitorInfo for MONITORINFOEXA {
    unsafe fn get(monitor: HMONITOR) -> Result<Self, Error> {
        let mut info = Self { cbSize: std::mem::size_of::<Self>() as _, .. std::mem::zeroed() };
        let success = GetMonitorInfoA(monitor, &mut info as *mut Self as *mut _) != FALSE;
        if !success { Error::last("GetMonitorInfoA", "")?; }
        Ok(info)
    }
}

impl MonitorInfo for MONITORINFOEXW {
    unsafe fn get(monitor: HMONITOR) -> Result<Self, Error> {
        let mut info = Self { cbSize: std::mem::size_of::<Self>() as _, .. std::mem::zeroed() };
        let success = GetMonitorInfoW(monitor, &mut info as *mut Self as *mut _) != FALSE;
        if !success { Error::last("GetMonitorInfoW", "")?; }
        Ok(info)
    }
}

impl MonitorInfo for MONITORINFO {
    unsafe fn get(monitor: HMONITOR) -> Result<Self, Error> {
        let mut info = Self { cbSize: std::mem::size_of::<Self>() as _, .. std::mem::zeroed() };
        let success = GetMonitorInfoW(monitor, &mut info as *mut Self as *mut _) != FALSE;
        if !success { Error::last("GetMonitorInfoW", "")?; }
        Ok(info)
    }
}



/// Select only the primary monitor
pub struct Primary;

/// Select the monitor relative to the primary monitor by X-order.
///
/// Note that this can be a bit counterintuitive when multiple monitors share the same X coordinate, as in a grid:
///
/// ### Examples
///
/// ```text
/// +---+---+---+
/// |-4 |-1 | 2 |  |   /   /
/// +---+---+---+  |  /|  /|
/// |-3 | 0 | 3 |  | / | / |
/// +---+---+---+  |/  |/  |
/// |-2 | 1 | 4 |  /   /   V
/// +---+---+---+
/// ```
///
/// ```text
/// +---+---+---+          _
/// |-2 | 0 | 2 |  \   ^   /|
/// +-+-+-+-+-+-+   \ / \ /
///   |-1 | 1 |      V   V
///   +---+---+
/// ```
pub struct ByOrderX(pub i16);

/// Select the monitor relative to the primary monitor by Y-order.
///
/// Note that this can be a bit counterintuitive when multiple monitors share the same Y coordinate, as in a grid:
///
/// ### Examples
///
/// ```text
/// +---+---+---+
/// |-4 |-3 |-2 |  --------
/// +---+---+---+    .-`
/// |-1 | 0 | 1 |  --------
/// +---+---+---+    .-`
/// | 2 | 3 | 4 |  ------->
/// +---+---+---+
/// ```
///
/// ```text
/// +---+
/// |-2 +---+
/// +---+-1 |
/// | 0 +---+
/// +---+ 1 |
/// | 2 +---+
/// +---+
/// ```
pub struct ByOrderY(pub i16);

/// [`Primary`]/[`ByOrderX`]/[`ByOrderY`]
pub trait Selector : private::Selector {}
impl<T: private::Selector> Selector for T {}

pub(crate) mod private {
    use super::*;

    pub trait Selector          { fn monitor_area(&self) -> RECT; }
    impl Selector for Primary   { fn monitor_area(&self) -> RECT { MonitorSelectorInfo::by_order(0, |_| ()).area } }
    impl Selector for ByOrderX  { fn monitor_area(&self) -> RECT { MonitorSelectorInfo::by_order(self.0, |m| (m.cx, m.cy)).area } }
    impl Selector for ByOrderY  { fn monitor_area(&self) -> RECT { MonitorSelectorInfo::by_order(self.0, |m| (m.cy, m.cx)).area } }
}

#[derive(Clone, Copy)] struct MonitorSelectorInfo {
    area:   RECT,
    cx:     LONG,
    cy:     LONG,
}

impl MonitorSelectorInfo {
    fn collect() -> Vec<Self> {
        collect_display_monitors((), None, |_monitor, _dc, &area| {
            let (cx, cy) = area.center();
            Self { area, cx, cy }
        })
    }

    fn by_order<K, F>(order: i16, key_fn: F) -> MonitorSelectorInfo
    where
        F: FnMut(&MonitorSelectorInfo) -> K,
        K: Ord,
    {
        let mut monitors = MonitorSelectorInfo::collect();

        monitors.sort_by_key(key_fn);
        let (primary_idx, _primary_msi) = monitors.iter().enumerate().find(|(_idx, msi)| msi.is_primary()).unwrap();

        let idx = if order < 0 {
            primary_idx.saturating_sub(-order as usize).max(0)
        } else { // order > 0
            primary_idx.saturating_add(order as usize).min(monitors.len()-1)
        };

        monitors[idx]
    }

    fn is_primary(&self) -> bool { self.area.left == 0 && self.area.top == 0 }
}
