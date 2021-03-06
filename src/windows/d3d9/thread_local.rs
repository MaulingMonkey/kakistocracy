use crate::windows::*;
use crate::windows::d3d9::*;

use wchar::wch_c;

use winapi::shared::d3d9::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::winuser::*;

use std::cell::RefCell;
use std::ptr::null_mut;



pub fn create_fullscreen_window(monitor: impl monitor::Selector, title: &str, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
    ThreadLocal::with(|tl| tl.create_fullscreen_window(monitor, title, context))
}

pub fn create_window_at(title: &str, area: impl IntoRect, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
    ThreadLocal::with(|tl| tl.create_window_at(title, area, context))
}



/// Shares [`IDirect3DDevice9`]s between multiple windows.
///
/// The device may be dropped and recreated in device lost scenarios.
/// "Additional" swap chains are created for each window, and recreated when the windows are resized.
/// The "primary" window is a hidden 1x1 message-only stub window.
///
/// [`IDirect3DDevice9`]:   https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3ddevice9
struct ThreadLocal {
    // NOTE: drop order might be important here!
    dac:            RefCell<Option<DeviceAndAssoc>>, // might be None for headless servers, some device lost scenarios, etc.
    d3d:            mcom::Rc<IDirect3D9>,
    stub_window:    OwnedWindow,
    windows:        RefCell<Vec<HWND>>,
}

struct DeviceAndAssoc {
    // NOTE: drop order might be important here!
    device:         mcom::Rc<IDirect3DDevice9>,
}

struct WindowAssoc {
    // NOTE: drop order might be important here!
    context:        Box<dyn Context>,
    swap_chain:     RefCell<Option<mcom::Rc<IDirect3DSwapChain9>>>,
}



trait Context : Render + message::Handler + 'static {}
impl<T: Render + message::Handler + 'static> Context for T {}

struct RenderLock {
    pub d3d:        mcom::Rc<IDirect3D9>,
    pub device:     mcom::Rc<IDirect3DDevice9>,
    pub windows:    Vec<RenderArgs>,
}

/// Constructors
impl ThreadLocal {
    pub fn with<R>(f: impl FnOnce(&ThreadLocal) -> R) -> R { TL.with(f) }
}

thread_local! { static TL : ThreadLocal = ThreadLocal::new(); }

/// Public Methods
impl ThreadLocal {
    pub fn create_fullscreen_window(&self, monitor: impl monitor::Selector, title: &str, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
        self.create_window_impl(title, monitor.monitor_area(), WS_POPUP | WS_VISIBLE, context)
    }

    pub fn create_window_at(&self, title: &str, area: impl IntoRect, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
        self.create_window_impl(title, area.into(), WS_OVERLAPPEDWINDOW | WS_VISIBLE, context)
    }

    fn create_window_impl(&self, title: &str, area: RECT, style: DWORD, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
        let hwnd = unsafe { CreateWindowExW(
            0,
            MAKEINTATOMW(*D3D9_MWC_WNDCLASS),
            title.encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
            style,
            area.left, area.top, area.right - area.left, area.bottom - area.top,
            null_mut(), null_mut(), get_module_handle_exe(), null_mut()
        )};
        if hwnd.is_null() { return Error::last("CreateWindowExW", "resulting hwnd is null"); }

        hwnd::assoc::set(hwnd, WindowAssoc {
            context:    Box::new(context),
            swap_chain: Default::default(),
        })?;
        self.windows.borrow_mut().push(hwnd);
        Ok(())
    }

    pub fn lock(&self, allow_no_rendered_windows: bool) -> Option<RenderLock> {
        self.cull_destroyed_windows();
        if !self.release_lost_devices() .unwrap() { return None; }
        if !self.try_create_device()    .unwrap() { return None; }
        let dac = self.dac.borrow();
        let dac = dac.as_ref()?;

        let device = dac.device.clone();
        let d3d = self.d3d.clone();
        let windows = self.windows.borrow().iter().filter_map(|&hwnd|{
            if unsafe { IsWindowVisible(hwnd) == FALSE } { return None; }
            if unsafe { IsIconic(hwnd)        != FALSE } { return None; }
            // XXX: Check IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop
            // XXX: Check DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, ...)
            // Ref: https://chromium.googlesource.com/external/webrtc/+/HEAD/modules/desktop_capture/win/window_capture_utils.cc

            let mut rect = unsafe { std::mem::zeroed() };
            unsafe { GetClientRect(hwnd, &mut rect) }; // XXX: Error check?
            let client_size = ((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32);
            let wa = hwnd::assoc::get::<WindowAssoc>(hwnd).ok()?;

            let mut wa_swap_chain = wa.swap_chain.borrow_mut();
            let wa_swap_chain = &mut *wa_swap_chain;
            let swap_chain = match wa_swap_chain.clone() {
                Some(swap_chain) if get_back_buffer_size(&swap_chain) == client_size => swap_chain,
                Some(_) | None => {
                    *wa_swap_chain = None; // release previous swap chain before creating a new one
                    let mut swap_chain = null_mut();
                    let mut pp = unsafe { d3d9::default_windowed_presentation_parameters(hwnd) };
                    let _hr = unsafe { dac.device.CreateAdditionalSwapChain(&mut pp, &mut swap_chain) };
                    let swap_chain = unsafe { mcom::Rc::from_raw_opt(swap_chain)? }; // panic on null?
                    *wa_swap_chain = Some(swap_chain.clone());
                    swap_chain
                },
            };
            Some(RenderArgs {
                d3d: d3d.clone(),
                device: device.clone(),
                window: hwnd,
                swap_chain,
                client_size,
            })
        }).collect::<Vec<RenderArgs>>();
        if windows.is_empty() && !allow_no_rendered_windows { return None; }
        Some(RenderLock { d3d, device, windows })
    }

    pub fn render_visible_windows(&self) {
        if let Some(lock) = self.lock(false) {
            for window in lock.windows.iter() {
                if let Ok(assoc) = hwnd::assoc::get::<WindowAssoc>(window.window) {
                    assoc.context.render(&window);
                }
            }
        }
    }
}

impl ThreadLocal {
    fn new() -> Self {
        message::each_frame(|_|{
            ThreadLocal::with(|tl| tl.render_visible_windows());
            true
        });

        Self {
            dac:            Default::default(),
            d3d:            d3d9::create_d3d(cfg!(debug_assertions)).unwrap(),
            stub_window:    OwnedWindow::create_stub("kakistocracy::windows::d3d9::ThreadLocal::stub_window"),
            windows:        Default::default()
        }
    }

    pub(crate) fn cull_destroyed_windows(&self) {
        self.windows.borrow_mut().retain(|&hwnd| hwnd::assoc::valid_window(hwnd));
    }

    pub(crate) fn release_lost_devices(&self) -> Result<bool, Error> {
        let mut opt_dac = self.dac.borrow_mut();
        let (release, result) = match opt_dac.as_ref() {
            None => (false, Ok(true)),
            Some(dac) => {
                // https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-testcooperativelevel
                let hr = unsafe { dac.device.TestCooperativeLevel() };
                match hr {
                    D3D_OK                      => (false,  Ok(true )), // OK, render away!
                    D3DERR_DEVICELOST           => (false,  Ok(false)), // Device lost, cannot ID3D11Device::Reset (yet), but we can free everything
                    D3DERR_DEVICENOTRESET       => (true,   Ok(true )), // Device lost, can    ID3D11Device::Reset, recreate entirely instead for now (I'm lazy)
                    D3DERR_DRIVERINTERNALERROR  => (true,   Err(Error::new_hr("IDirect3DDevice9::TestCooperativeLevel", hr, "D3DERR_DRIVERINTERNALERROR"))),
                    ok if SUCCEEDED(ok)         => (false,  Ok(true )), // Odd... a non-D3D_OK success?
                    _err                        => (true,   Err(Error::new_hr("IDirect3DDevice9::TestCooperativeLevel", hr, ""))),
                }
            },
        };
        if release {
            *opt_dac = None;
            let windows = self.windows.borrow();
            for assoc in windows.iter().copied().filter_map(|hwnd| hwnd::assoc::get::<WindowAssoc>(hwnd).ok()) {
                *assoc.swap_chain.borrow_mut() = None;
            }
        }
        result
    }

    pub(crate) fn try_create_device(&self) -> Result<bool, Error> {
        let mut opt_dac = self.dac.borrow_mut();
        if opt_dac.is_none() {
            // https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3d9-createdevice
            let device = match unsafe { d3d9::create_device_windowed(&self.d3d, &self.stub_window) } {
                Ok(device) => device,
                Err(err) => match err.hresult() {
                    D3DERR_DEVICELOST               => return Ok(false), // Device lost, cannot create (yet)
                    D3DERR_INVALIDCALL              => panic!("BUG: expected to be able to create device: {}", err),
                    D3DERR_NOTAVAILABLE             => return Err(err), // Semi-fatal error, forward
                    D3DERR_OUTOFVIDEOMEMORY         => return Err(err), // Semi-fatal error, forward
                    _err                            => return Err(err), // Unknown error, forward
                },
            };
            *opt_dac = Some(DeviceAndAssoc { device });
        }
        Ok(true)
    }
}

fn get_back_buffer_size(swap_chain: &mcom::Rc<IDirect3DSwapChain9>) -> (u32, u32) {
    let mut pp = unsafe { std::mem::zeroed() };
    let _hr = unsafe { swap_chain.GetPresentParameters(&mut pp) };
    (pp.BackBufferWidth, pp.BackBufferHeight)
}

lazy_static::lazy_static! {
    static ref D3D9_MWC_WNDCLASS : ATOM = unsafe { register_class_w(&WNDCLASSW {
        style:          0,
        lpfnWndProc:    Some(wndproc),
        cbClsExtra:     0,
        cbWndExtra:     0,
        hInstance:      get_module_handle_exe(),
        hIcon:          null_mut(),
        hCursor:        LoadCursorW(null_mut(), IDC_ARROW),
        hbrBackground:  null_mut(),
        lpszMenuName:   null_mut(),
        lpszClassName:  wch_c!("kakistocracy-d3d9-window").as_ptr(),
    })}.unwrap();
}

/// ### Safety
/// * `hwnd` might be a real pointer in older versions of Windows
/// * `wparam` / `lparam` might be treated as raw pointers depending on `msg`
/// * ...
unsafe extern "system" fn wndproc(hwnd: HWND, msg: DWORD, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match hwnd::assoc::get::<WindowAssoc>(hwnd) {
        Ok(assoc) => assoc.context.wndproc(hwnd, msg, wparam, lparam),
        Err(_err) => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
