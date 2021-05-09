use crate::windows::*;

use mcom::AsIUnknown;

use wchar::wch_c;

use winapi::shared::dxgi::*;
use winapi::shared::dxgiformat::{DXGI_FORMAT_B8G8R8A8_UNORM};
use winapi::shared::dxgitype::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::winerror::*;
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;
use winapi::um::winuser::*;

use std::cell::RefCell;
use std::ptr::null_mut;



pub trait Render {
    fn render(&self, args: &RenderArgs);
}

trait Context : Render + message::Handler + 'static {}
impl<T: Render + message::Handler + 'static> Context for T {}



/// Shares [`ID3D11Device`]s between multiple windows.
///
/// The device may be dropped and recreated in device lost scenarios.
/// "Additional" swap chains are created for each window, and recreated when the windows are resized.
/// The "primary" window is a hidden 1x1 message-only stub window.
///
/// [`ID3D11Device`]:   https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nn-d3d11-id3d11device
struct ThreadLocal {
    // NOTE: drop order might be important here!
    dac:            RefCell<Option<DeviceAndAssoc>>, // might be None for headless servers, some device lost scenarios, etc.
    windows:        RefCell<Vec<HWND>>,
}

struct RenderLock {
    pub device:             mcom::Rc<ID3D11Device>,
    pub immediate_context:  mcom::Rc<ID3D11DeviceContext>,
    pub windows:            Vec<RenderArgs>,
}

pub struct RenderArgs {
    pub device:             mcom::Rc<ID3D11Device>,
    pub immediate_context:  mcom::Rc<ID3D11DeviceContext>,
    pub window:             HWND,
    pub rtv:                mcom::Rc<ID3D11RenderTargetView>,
    pub swap_chain:         mcom::Rc<IDXGISwapChain>,
    client_size:            (u32, u32),
}

struct DeviceAndAssoc {
    // NOTE: drop order might be important here!
    #[allow(dead_code)] // TODO: expose somehow?
    feature_level:      D3D_FEATURE_LEVEL,
    immediate_context:  mcom::Rc<ID3D11DeviceContext>,
    device:             mcom::Rc<ID3D11Device>,
}

struct WindowAssoc {
    // NOTE: drop order might be important here!
    context:        Box<dyn Context>,
    swap_chain_rtv: RefCell<Option<(
        mcom::Rc<IDXGISwapChain>,
        mcom::Rc<ID3D11RenderTargetView>,
    )>>,
}


/// Constructors
impl ThreadLocal {
    pub fn with<R>(f: impl FnOnce(&ThreadLocal) -> R) -> R { TL.with(f) }
}

thread_local! { static TL : ThreadLocal = ThreadLocal::new(); }

pub fn create_fullscreen_window(monitor: impl monitor::Selector, title: &str, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
    ThreadLocal::with(|tl| tl.create_fullscreen_window(monitor, title, context))
}

pub fn create_window_at(title: &str, area: impl IntoRect, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
    ThreadLocal::with(|tl| tl.create_window_at(title, area, context))
}

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
            MAKEINTATOMW(*D3D11_MWC_WNDCLASS),
            title.encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
            style,
            area.left, area.top, area.right - area.left, area.bottom - area.top,
            null_mut(), null_mut(), get_module_handle_exe(), null_mut()
        )};
        if hwnd.is_null() { return Error::last("CreateWindowExW", "resulting hwnd is null"); }

        hwnd::assoc::set(hwnd, WindowAssoc {
            context:        Box::new(context),
            swap_chain_rtv: Default::default(),
        })?;
        self.windows.borrow_mut().push(hwnd);
        Ok(())
    }

    pub fn lock(&self, allow_no_rendered_windows: bool) -> Option<RenderLock> {
        self.cull_destroyed_windows();
        let _ = self.try_create_device();
        let dac = self.dac.borrow();
        let dac = dac.as_ref()?;

        let device = dac.device.clone();
        let immediate_context = dac.immediate_context.clone();
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

            let mut wa_swap_chain_rtv = wa.swap_chain_rtv.borrow_mut();
            let wa_swap_chain_rtv = &mut *wa_swap_chain_rtv;
            let (swap_chain, rtv) = match wa_swap_chain_rtv.clone() {
                Some(swap_chain_rtv) if get_back_buffer_size(&swap_chain_rtv.0) == client_size => swap_chain_rtv,
                Some(_) | None => {
                    *wa_swap_chain_rtv = None; // release previous swap chain before creating a new one

                    let dxgi_device     = dac.device.to_dxgi_device();
                    let dxgi_adapter    = dxgi_device.get_parent_dxgi_adapter().unwrap();
                    let dxgi_factory    = dxgi_adapter.get_parent_dxgi_factory().unwrap();
                    let bb_format       = DXGI_FORMAT_B8G8R8A8_UNORM;

                    let swap_chain = {
                        let mut swap_chain = null_mut();
                        let mut desc = DXGI_SWAP_CHAIN_DESC { // https://docs.microsoft.com/en-us/windows/win32/api/dxgi/ns-dxgi-dxgi_swap_chain_desc
                            BufferDesc:     DXGI_MODE_DESC {
                                Width:              0,
                                Height:             0,
                                RefreshRate:        DXGI_RATIONAL { Numerator: 0, Denominator: 0 },
                                Format:             bb_format,
                                ScanlineOrdering:   DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                                Scaling:            DXGI_MODE_SCALING_UNSPECIFIED,
                            },
                            SampleDesc:     DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                            BufferUsage:    DXGI_USAGE_RENDER_TARGET_OUTPUT,
                            BufferCount:    1, // inc. front buffer, for fullscreen
                            OutputWindow:   hwnd,
                            Windowed:       TRUE,
                            SwapEffect:     DXGI_SWAP_EFFECT_DISCARD,
                            Flags:          DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH,
                        };
                        let _hr = unsafe { dxgi_factory.CreateSwapChain(dac.device.as_iunknown_ptr(), &mut desc, &mut swap_chain) };
                        match _hr {
                            DXGI_ERROR_DEVICE_REMOVED   => return None,
                            DXGI_ERROR_DEVICE_RESET     => return None,
                            _other                      => assert!(SUCCEEDED(_hr), "IDXGIFactory::CreateSwapChain failed with HRESULT == 0x{:08x}", _hr as u32),
                        }
                        unsafe { mcom::Rc::from_raw_opt(swap_chain)? } // panic on null?
                    };

                    let bb = swap_chain.get_buffer::<ID3D11Resource>(0).unwrap();
                    let rtv = dac.device.create_render_target_view_from_resource(&bb).unwrap();

                    *wa_swap_chain_rtv = Some((swap_chain.clone(), rtv.clone()));
                    (swap_chain, rtv)
                },
            };
            Some(RenderArgs {
                device: device.clone(),
                immediate_context: immediate_context.clone(),
                window: hwnd,
                swap_chain,
                rtv,
                client_size,
            })
        }).collect::<Vec<RenderArgs>>();
        if windows.is_empty() && !allow_no_rendered_windows { return None; }
        Some(RenderLock { device, immediate_context, windows })
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

impl RenderArgs {
    pub fn bind_immediate_context(&self) -> Result<(), Error> {
        unsafe { self.bind(&self.immediate_context) }
    }

    /// Binds the next back buffer of the window's swap chain as the render target, and sets the viewport to the entire window.
    ///
    /// ### Safety
    /// * `ctx` must belong to <code>self.[device](Self::device)</code>
    pub unsafe fn bind(&self, ctx: &mcom::Rc<ID3D11DeviceContext>) -> Result<(), Error> {
        let rtvs = [self.rtv.as_ptr()];
        ctx.OMSetRenderTargets(rtvs.len() as _, rtvs.as_ptr(), null_mut());

        let viewports = [D3D11_VIEWPORT{
            TopLeftX:   0.0,
            TopLeftY:   0.0,
            Width:      f64::from(self.client_width() ) as f32,
            Height:     f64::from(self.client_height()) as f32,
            MinDepth:   0.0,
            MaxDepth:   1.0,
        }];
        ctx.RSSetViewports(viewports.len() as _, viewports.as_ptr());

        Ok(())
    }

    pub fn client_size  (&self) -> (u32, u32)   { self.client_size }
    pub fn client_width (&self) -> u32          { self.client_size.0 }
    pub fn client_height(&self) -> u32          { self.client_size.1 }

    pub fn client_size_usize    (&self) -> (usize, usize)   { let (w, h) = self.client_size; (w as usize, h as usize) }
    pub fn client_width_usize   (&self) -> usize            { self.client_size.0 as usize }
    pub fn client_height_usize  (&self) -> usize            { self.client_size.1 as usize }
}



const DEFAULT_CREATE_FLAGS : D3D11_CREATE_DEVICE_FLAG = // https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_create_device_flag
    // D3D11_CREATE_DEVICE_DEBUG |
    D3D11_CREATE_DEVICE_BGRA_SUPPORT |
    // D3D11_CREATE_DEVICE_DEBUGGABLE |
    // D3D11_CREATE_DEVICE_PREVENT_ALTERING_LAYER_SETTINGS_FROM_REGISTRY |
    // D3D11_CREATE_DEVICE_DISABLE_GPU_TIMEOUT |
    0;

/// Implementation Details
impl ThreadLocal {
    fn new() -> Self {
        message::each_frame(|_|{
            ThreadLocal::with(|tl| tl.render_visible_windows());
            true
        });

        Self {
            dac:            Default::default(),
            windows:        Default::default()
        }
    }

    pub(crate) fn cull_destroyed_windows(&self) {
        self.windows.borrow_mut().retain(|&hwnd| hwnd::assoc::valid_window(hwnd));
    }

    pub(crate) fn try_create_device(&self) -> Result<(), Error> {
        let feature_levels = [
            D3D_FEATURE_LEVEL_12_1,
            D3D_FEATURE_LEVEL_12_0,
            D3D_FEATURE_LEVEL_11_1,
            // feature levels above this mark won't be selected if you merely pass null/0 to D3D11CreateDevice
            // https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-d3d11createdevice#parameters
            D3D_FEATURE_LEVEL_11_0,
            D3D_FEATURE_LEVEL_10_1,
            D3D_FEATURE_LEVEL_10_0,
            D3D_FEATURE_LEVEL_9_3,
            D3D_FEATURE_LEVEL_9_2,
            D3D_FEATURE_LEVEL_9_1,
        ];

        if self.dac.borrow().is_none() {
            let mut device = null_mut();
            let mut feature_level = 0;
            let mut immediate_context = null_mut();
            let mut hr = unsafe { D3D11CreateDevice(
                null_mut(), // adapter
                D3D_DRIVER_TYPE_HARDWARE,
                null_mut(), // software
                DEFAULT_CREATE_FLAGS,
                feature_levels.as_ptr(),
                feature_levels.len() as _,
                D3D11_SDK_VERSION,
                &mut device,
                &mut feature_level,
                &mut immediate_context,
            )};
            // NOTE:  If you request a D3D_FEATURE_LEVEL_11_1 device on a computer with only the Direct3D 11.0 runtime, D3D11CreateDeviceAndSwapChain immediately exits with E_INVALIDARG.
            // Ref: https://docs.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-devices-initialize
            // Probably applies to D3D11CreateDevice too, so fall back here
            if hr == E_INVALIDARG && device.is_null() && immediate_context.is_null() {
                hr = unsafe { D3D11CreateDevice(
                    null_mut(), // adapter
                    D3D_DRIVER_TYPE_HARDWARE,
                    null_mut(), // software
                    DEFAULT_CREATE_FLAGS,
                    null_mut(),
                    0,
                    D3D11_SDK_VERSION,
                    &mut device,
                    &mut feature_level,
                    &mut immediate_context,
                )};
            }
            let device              = unsafe { mcom::Rc::from_raw_opt(device            ) };
            let immediate_context   = unsafe { mcom::Rc::from_raw_opt(immediate_context ) };

            let device              = device            .ok_or(Error::new_hr("D3D11CreateDevice", hr, "ID3D11Device is null"))?;
            let immediate_context   = immediate_context .ok_or(Error::new_hr("D3D11CreateDevice", hr, "ID3D11DeviceContext is null"))?;
            *self.dac.borrow_mut() = Some(DeviceAndAssoc {
                feature_level,
                device,
                immediate_context,
            });
        }
        Ok(())
    }
}

fn get_back_buffer_size(swap_chain: &mcom::Rc<IDXGISwapChain>) -> (u32, u32) {
    let desc = swap_chain.get_desc().unwrap();
    (desc.BufferDesc.Width, desc.BufferDesc.Height)
}

lazy_static::lazy_static! {
    static ref D3D11_MWC_WNDCLASS : ATOM = unsafe { register_class_w(&WNDCLASSW {
        style:          0,
        lpfnWndProc:    Some(wndproc),
        cbClsExtra:     0,
        cbWndExtra:     0,
        hInstance:      get_module_handle_exe(),
        hIcon:          null_mut(),
        hCursor:        LoadCursorW(null_mut(), IDC_ARROW),
        hbrBackground:  null_mut(),
        lpszMenuName:   null_mut(),
        lpszClassName:  wch_c!("kakistocracy-d3d11-window").as_ptr(),
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
