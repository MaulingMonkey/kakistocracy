use crate::windows::*;

use mcom::AsIUnknown;
use winapi::Interface;
use winapi::shared::dxgi::*;
use winapi::shared::dxgiformat::{DXGI_FORMAT_B8G8R8A8_UNORM};
use winapi::shared::dxgitype::*;
use winapi::shared::minwindef::*;
use winapi::shared::winerror::{E_INVALIDARG, SUCCEEDED};
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;

use std::any::*;
use std::cell::RefCell;
use std::collections::*;
use std::ops::Deref;
use std::ptr::null_mut;
use std::rc::Rc;



pub trait CreateFromDevice {
    fn new(device: &mcom::Rc<ID3D11Device>) -> Self;
}



/// Shares [`ID3D11Device`]s between multiple windows.
///
/// The device may be dropped and recreated in device lost scenarios.
/// "Additional" swap chains are created for each window, and recreated when the windows are resized.
/// The "primary" window is a hidden 1x1 message-only stub window.
///
/// [`ID3D11Device`]:   https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nn-d3d11-id3d11device
pub struct MultiWindowContext {
    // NOTE: drop order might be important here!
    dac:            Option<DeviceAndAssoc>, // might be None for headless servers, some device lost scenarios, etc.
    windows:        Vec<OwnedWindow>,
}

pub struct MultiWindowContextLock {
    pub device:             mcom::Rc<ID3D11Device>,
    pub immediate_context:  mcom::Rc<ID3D11DeviceContext>,
    pub windows:            Vec<MultiWindowContextLockWindow>,
}

pub struct MultiWindowContextLockWindow {
    pub window:     Window,
    pub rtv:        mcom::Rc<ID3D11RenderTargetView>,
    pub swap_chain: mcom::Rc<IDXGISwapChain>,
}

struct DeviceAndAssoc {
    // NOTE: drop order might be important here!
    statics:            RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    #[allow(dead_code)] // TODO: expose somehow?
    feature_level:      D3D_FEATURE_LEVEL,
    immediate_context:  mcom::Rc<ID3D11DeviceContext>,
    device:             mcom::Rc<ID3D11Device>,
}

#[derive(Default)]
struct WindowAssoc {
    // NOTE: drop order might be important here!
    swap_chain_rtv: RefCell<Option<(
        mcom::Rc<IDXGISwapChain>,
        mcom::Rc<ID3D11RenderTargetView>,
    )>>,
}


/// Constructors
impl MultiWindowContext {
    pub fn new() -> Result<Self, Error> {
        let mut mwc = Self::new_raw()?;
        mwc.try_create_device()?; // TODO: variant which allows deviceless init? what would be the use case tho? d3d9 runtime installer? driver installer?
        Ok(mwc)
    }
}

/// Public Methods
impl MultiWindowContext {
    pub fn has_device(&self) -> bool { self.dac.is_some() }

    pub fn per_device<C: Any + CreateFromDevice>(&self) -> Option<impl Deref<Target = C>> {
        match self.dac.as_ref() {
            Some(dac) => {
                let rc_any = match dac.statics.borrow_mut().entry(TypeId::of::<C>()) {
                    hash_map::Entry::Occupied(o)    => o.get().clone(),
                    hash_map::Entry::Vacant(v)      => v.insert(Rc::new(C::new(&dac.device))).clone(),
                };
                Some(rc_any.downcast().unwrap())
            },
            None => None,
        }
    }

    pub fn create_fullscreen_window(&mut self, monitor: usize, title: &str) -> Result<Window, Error> {
        let window = Window::create_fullscreen(monitor, title)?;
        window.set(WindowAssoc::default());
        self.windows.push(OwnedWindow::new(window.clone()));
        Ok(window)
    }

    pub fn create_window_at(&mut self, title: &str, area: impl IntoRect) -> Result<Window, Error> {
        let window = Window::create_at(title, area)?;
        window.set(WindowAssoc::default());
        self.windows.push(OwnedWindow::new(window.clone()));
        Ok(window)
    }

    pub fn lock(&mut self, allow_no_rendered_windows: bool) -> Option<MultiWindowContextLock> {
        self.cull_destroyed_windows();
        let _ = self.try_create_device();
        let dac = self.dac.as_ref()?;

        let device = dac.device.clone();
        let immediate_context = dac.immediate_context.clone();
        //let d3d = self.d3d.clone();
        let windows = self.windows.iter().filter_map(|window|{
            if !window.should_render() { return None; }
            let hwnd = window.hwnd()?;
            let client_size = window.get_client_rect().ok()?.size();
            let wa = window.get_or_default::<WindowAssoc>()?;
            let mut wa_swap_chain_rtv = wa.swap_chain_rtv.borrow_mut();
            let wa_swap_chain_rtv = &mut *wa_swap_chain_rtv;
            let (swap_chain, rtv) = match wa_swap_chain_rtv.clone() {
                Some(swap_chain_rtv) if get_back_buffer_size(&swap_chain_rtv.0) == client_size => swap_chain_rtv,
                Some(_) | None => {
                    *wa_swap_chain_rtv = None; // release previous swap chain before creating a new one

                    let dxgi_device = dac.device.try_cast::<IDXGIDevice>()?;

                    let dxgi_adapter = {
                        let mut dxgi_adapter = null_mut();
                        let _hr = unsafe { dxgi_device.GetParent(&IDXGIAdapter::uuidof(), &mut dxgi_adapter) };
                        unsafe { mcom::Rc::from_raw_opt(dxgi_adapter as *mut IDXGIAdapter)? }
                    };

                    let dxgi_factory = {
                        let mut dxgi_factory = null_mut();
                        let _hr = unsafe { dxgi_adapter.GetParent(&IDXGIFactory::uuidof(), &mut dxgi_factory) };
                        unsafe { mcom::Rc::from_raw_opt(dxgi_factory as *mut IDXGIFactory)? }
                    };

                    let bb_format = DXGI_FORMAT_B8G8R8A8_UNORM;

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
                        assert!(SUCCEEDED(_hr), "IDXGIFactory::CreateSwapChain failed with HRESULT == 0x{:08x}", _hr as u32);
                        unsafe { mcom::Rc::from_raw_opt(swap_chain)? } // panic on null?
                    };

                    let bb = {
                        let mut bb = null_mut();
                        let _hr = unsafe { swap_chain.GetBuffer(0, &ID3D11Resource::uuidof(), &mut bb) };
                        assert!(SUCCEEDED(_hr), "IDXGISwapChain::GetBuffer failed with HRESULT == 0x{:08x}", _hr as u32);
                        unsafe { mcom::Rc::from_raw_opt(bb as *mut ID3D11Resource)? } // panic on null?
                    };

                    let rtv = {
                        let mut rtv = null_mut();
                        let _hr = unsafe { dac.device.CreateRenderTargetView(bb.as_ptr(), null_mut(), &mut rtv) };
                        assert!(SUCCEEDED(_hr), "ID3D11Device::CreateRenderTargetView failed with HRESULT == 0x{:08x}", _hr as u32);
                        unsafe { mcom::Rc::from_raw_opt(rtv)? } // panic on null?
                    };

                    *wa_swap_chain_rtv = Some((swap_chain.clone(), rtv.clone()));
                    (swap_chain, rtv)
                },
            };
            Some(MultiWindowContextLockWindow {
                window: (*window).clone(),
                swap_chain,
                rtv,
            })
        }).collect::<Vec<MultiWindowContextLockWindow>>();
        if windows.is_empty() && !allow_no_rendered_windows { return None; }
        Some(MultiWindowContextLock { device, immediate_context, windows })
    }
}

impl MultiWindowContextLockWindow {
    /// Binds the next back buffer of the window's swap chain as the render target, and sets the viewport to the entire window.
    ///
    /// ### Safety
    /// * `device` must be the same device as the originating [`MultiWindowContext`]
    pub unsafe fn bind(&self, ctx: &mcom::Rc<ID3D11DeviceContext>) -> Result<(), Error> {
        let rect = self.window.get_client_rect()?;

        let rtvs = [self.rtv.as_ptr()];
        ctx.OMSetRenderTargets(rtvs.len() as _, rtvs.as_ptr(), null_mut());

        let viewports = [D3D11_VIEWPORT{
            TopLeftX:   0.0,
            TopLeftY:   0.0,
            Width:      f64::from(rect.width() ) as f32,
            Height:     f64::from(rect.height()) as f32,
            MinDepth:   0.0,
            MaxDepth:   1.0,
        }];
        ctx.RSSetViewports(viewports.len() as _, viewports.as_ptr());

        Ok(())
    }
}



const DEFAULT_CREATE_FLAGS : D3D11_CREATE_DEVICE_FLAG = // https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_create_device_flag
    // D3D11_CREATE_DEVICE_DEBUG |
    D3D11_CREATE_DEVICE_BGRA_SUPPORT |
    // D3D11_CREATE_DEVICE_DEBUGGABLE |
    // D3D11_CREATE_DEVICE_PREVENT_ALTERING_LAYER_SETTINGS_FROM_REGISTRY |
    // D3D11_CREATE_DEVICE_DISABLE_GPU_TIMEOUT |
    0;

/// Implementation Details
impl MultiWindowContext {
    fn new_raw() -> Result<Self, Error> {
        Ok(Self {
            dac:            None,
            windows:        Default::default()
        })
    }

    pub(crate) fn cull_destroyed_windows(&mut self) {
        self.windows.retain(|pw| pw.is_alive())
    }

    pub(crate) fn try_create_device(&mut self) -> Result<(), Error> {
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

        if self.dac.is_none() {
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
            self.dac = Some(DeviceAndAssoc {
                statics:        Default::default(),
                feature_level,
                device,
                immediate_context,
            });
        }
        Ok(())
    }
}

fn get_back_buffer_size(swap_chain: &mcom::Rc<IDXGISwapChain>) -> (u32, u32) {
    let mut desc = unsafe { std::mem::zeroed() };
    let _hr = unsafe { swap_chain.GetDesc(&mut desc) };
    (desc.BufferDesc.Width, desc.BufferDesc.Height)
}
