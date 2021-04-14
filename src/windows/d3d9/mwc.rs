use crate::windows::*;

use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::D3DBACKBUFFER_TYPE_MONO;
use winapi::shared::winerror::SUCCEEDED;

use std::any::*;
use std::cell::RefCell;
use std::collections::*;
use std::ops::Deref;
use std::ptr::null_mut;
use std::rc::Rc;



pub trait CreateFromDevice {
    fn new(device: &mcom::Rc<IDirect3DDevice9>) -> Self;
}



/// Shares [`IDirect3DDevice9`]s between multiple windows.
///
/// The device may be dropped and recreated in device lost scenarios.
/// "Additional" swap chains are created for each window, and recreated when the windows are resized.
/// The "primary" / focus window is a hidden 1x1 message-only stub window.
///
/// [`IDirect3DDevice9`]:   https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3ddevice9
pub struct MultiWindowContext {
    // NOTE: drop order might be important here!
    dac:            Option<DeviceAndAssoc>, // might be None for headless servers, some device lost scenarios, etc.
    d3d:            mcom::Rc<IDirect3D9>,
    stub_window:    OwnedWindow,
    windows:        Vec<Window>,
}

pub struct MultiWindowContextLock {
    pub d3d:        mcom::Rc<IDirect3D9>,
    pub device:     mcom::Rc<IDirect3DDevice9>,
    pub windows:    Vec<MultiWindowContextLockWindow>,
}

pub struct MultiWindowContextLockWindow {
    pub window:     Window,
    pub swap_chain: mcom::Rc<IDirect3DSwapChain9>,
}



struct DeviceAndAssoc {
    // NOTE: drop order might be important here!
    statics:    RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    device:     mcom::Rc<IDirect3DDevice9>,
}

#[derive(Default)]
struct WindowAssoc {
    // NOTE: drop order might be important here!
    swap_chain: RefCell<Option<mcom::Rc<IDirect3DSwapChain9>>>,
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
        self.windows.push(window.clone());
        Ok(window)
    }

    pub fn create_window_at(&mut self, title: &str, area: impl IntoRect) -> Result<Window, Error> {
        let window = Window::create_at(title, area)?;
        window.set(WindowAssoc::default());
        self.windows.push(window.clone());
        Ok(window)
    }

    pub fn if_device<R>(&mut self, f: impl FnOnce(&mcom::Rc<IDirect3DDevice9>) -> R) -> Option<R> {
        self.cull_destroyed_windows();
        let _ = self.try_create_device();
        if let Some(dac) = self.dac.as_ref() {
            Some(f(&dac.device))
        } else {
            None
        }
    }

    pub fn lock(&mut self, allow_no_rendered_windows: bool) -> Option<MultiWindowContextLock> {
        self.cull_destroyed_windows();
        let _ = self.try_create_device();
        let dac = self.dac.as_ref()?;

        let device = dac.device.clone();
        let d3d = self.d3d.clone();
        let windows = self.windows.iter().filter_map(|window|{
            if !window.should_render() { return None; }
            let hwnd = window.hwnd()?;
            let client_size = window.get_client_rect().ok()?.size();
            let wa = window.get_or_default::<WindowAssoc>()?;
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
            Some(MultiWindowContextLockWindow {
                window: window.clone(),
                swap_chain,
            })
        }).collect::<Vec<MultiWindowContextLockWindow>>();
        if windows.is_empty() && !allow_no_rendered_windows { return None; }
        Some(MultiWindowContextLock { d3d, device, windows })
    }
}

impl MultiWindowContextLockWindow {
    /// ### Safety
    /// * `device` must be the same device as the originating [`MultiWindowContext`]
    pub unsafe fn bind(&self, device: &mcom::Rc<IDirect3DDevice9>) -> Result<(), Error> {
        let mut bb = null_mut();
        let hr = self.swap_chain.GetBackBuffer(0, D3DBACKBUFFER_TYPE_MONO, &mut bb);
        let bb = mcom::Rc::from_raw_opt(bb).ok_or(Error::new_hr("IDirect3DSwapChain9::GetBackBuffer", hr, "IDirect3DSurface9 is null"))?;

        let hr = device.SetRenderTarget(0, bb.as_ptr());
        if !SUCCEEDED(hr) { return Err(Error::new_hr("IDirect3DDevice9::SetRenderTarget", hr, "")); }

        Ok(())
    }
}



/// Implementation Details
impl MultiWindowContext {
    fn new_raw() -> Result<Self, Error> {
        Ok(Self {
            dac:            None,
            d3d:            d3d9::create_d3d(cfg!(debug_assertions))?,
            stub_window:    OwnedWindow::create_stub("kakistocracy::windows::d3d9::MultiWindowContext::stub_window"),
            windows:        Default::default()
        })
    }

    pub(crate) fn cull_destroyed_windows(&mut self) {
        self.windows.retain(|pw| pw.is_alive())
    }

    pub(crate) fn try_create_device(&mut self) -> Result<(), Error> {
        if self.dac.is_none() {
            let device = unsafe { d3d9::create_device_windowed(&self.d3d, &self.stub_window)? };
            self.dac = Some(DeviceAndAssoc { device, statics: Default::default() });
        }
        Ok(())
    }
}

fn get_back_buffer_size(swap_chain: &mcom::Rc<IDirect3DSwapChain9>) -> (u32, u32) {
    let mut pp = unsafe { std::mem::zeroed() };
    let _hr = unsafe { swap_chain.GetPresentParameters(&mut pp) };
    (pp.BackBufferWidth, pp.BackBufferHeight)
}
