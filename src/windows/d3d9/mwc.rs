use crate::windows::*;

use wchar::wch_c;

use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::winuser::*;

use std::any::*;
use std::cell::RefCell;
use std::collections::*;
use std::ops::Deref;
use std::ptr::null_mut;
use std::rc::Rc;



pub trait CreateFromDevice {
    fn new(device: &mcom::Rc<IDirect3DDevice9>) -> Self;
}

impl<T: From<mcom::Rc<IDirect3DDevice9>>> CreateFromDevice for T {
    fn new(device: &mcom::Rc<IDirect3DDevice9>) -> Self { Self::from(device.clone()) }
}

pub type RenderArgs = MultiWindowContextLockWindow;

pub trait Render {
    fn render(&self, args: &RenderArgs);
}

trait Context : Render + message::Handler + 'static {}
impl<T: Render + message::Handler + 'static> Context for T {}


/// Shares [`IDirect3DDevice9`]s between multiple windows.
///
/// The device may be dropped and recreated in device lost scenarios.
/// "Additional" swap chains are created for each window, and recreated when the windows are resized.
/// The "primary" window is a hidden 1x1 message-only stub window.
///
/// [`IDirect3DDevice9`]:   https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3ddevice9
pub struct MultiWindowContext {
    // NOTE: drop order might be important here!
    dac:            Option<DeviceAndAssoc>, // might be None for headless servers, some device lost scenarios, etc.
    d3d:            mcom::Rc<IDirect3D9>,
    stub_window:    OwnedWindow,
    windows:        Vec<HWND>,
}

pub struct MultiWindowContextLock {
    pub d3d:        mcom::Rc<IDirect3D9>,
    pub device:     mcom::Rc<IDirect3DDevice9>,
    pub windows:    Vec<MultiWindowContextLockWindow>,
}

pub struct MultiWindowContextLockWindow {
    pub d3d:        mcom::Rc<IDirect3D9>,
    pub device:     mcom::Rc<IDirect3DDevice9>,
    pub window:     HWND,
    pub swap_chain: mcom::Rc<IDirect3DSwapChain9>,
    client_size:    (u32, u32),
}



struct DeviceAndAssoc {
    // NOTE: drop order might be important here!
    statics:        RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    device:         mcom::Rc<IDirect3DDevice9>,
}

struct WindowAssoc {
    // NOTE: drop order might be important here!
    context:        Box<dyn Context>,
    swap_chain:     RefCell<Option<mcom::Rc<IDirect3DSwapChain9>>>,
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

    pub fn create_fullscreen_window(&mut self, monitor: impl monitor::Selector, title: &str, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
        self.create_window_impl(title, monitor.monitor_area(), WS_POPUP | WS_VISIBLE, context)
    }

    pub fn create_window_at(&mut self, title: &str, area: impl IntoRect, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
        self.create_window_impl(title, area.into(), WS_OVERLAPPEDWINDOW | WS_VISIBLE, context)
    }

    fn create_window_impl(&mut self, title: &str, area: RECT, style: DWORD, context: impl Render + message::Handler + 'static) -> Result<(), Error> {
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
        self.windows.push(hwnd);
        Ok(())
    }

    pub fn lock(&mut self, allow_no_rendered_windows: bool) -> Option<MultiWindowContextLock> {
        self.cull_destroyed_windows();
        let _ = self.try_create_device();
        let dac = self.dac.as_ref()?;

        let device = dac.device.clone();
        let d3d = self.d3d.clone();
        let windows = self.windows.iter().filter_map(|&hwnd|{
            if unsafe { IsWindowVisible(hwnd) == FALSE } { return None; }
            if unsafe { IsIconic(hwnd)        != FALSE } { return None; }
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
            Some(MultiWindowContextLockWindow {
                d3d: d3d.clone(),
                device: device.clone(),
                window: hwnd,
                swap_chain,
                client_size,
            })
        }).collect::<Vec<MultiWindowContextLockWindow>>();
        if windows.is_empty() && !allow_no_rendered_windows { return None; }
        Some(MultiWindowContextLock { d3d, device, windows })
    }

    pub fn render_visible_windows(&mut self) {
        if let Some(lock) = self.lock(false) {
            for window in lock.windows.iter() {
                if let Ok(assoc) = hwnd::assoc::get::<WindowAssoc>(window.window) {
                    assoc.context.render(&window);
                }
            }
        }
    }
}

impl MultiWindowContextLockWindow {
    /// Binds the next back buffer of the window's swap chain as the render target, and sets the viewport to the entire window.
    pub fn bind(&self) -> Result<(), Error> {
        let device = &self.device;

        let mut bb = null_mut();
        let hr = unsafe { self.swap_chain.GetBackBuffer(0, D3DBACKBUFFER_TYPE_MONO, &mut bb) };
        let bb = unsafe { mcom::Rc::from_raw_opt(bb) }.ok_or(Error::new_hr("IDirect3DSwapChain9::GetBackBuffer", hr, "IDirect3DSurface9 is null"))?;

        let mut desc = unsafe { std::mem::zeroed() };
        let hr = unsafe { bb.GetDesc(&mut desc) };
        if !SUCCEEDED(hr) { return Err(Error::new_hr("IDirect3DSurface9::GetDesc", hr, "when binding MultiWindowContextLockWindow")); }

        let hr = unsafe { device.SetRenderTarget(0, bb.as_ptr()) };
        if !SUCCEEDED(hr) { return Err(Error::new_hr("IDirect3DDevice9::SetRenderTarget", hr, "when binding MultiWindowContextLockWindow")); }

        let hr = unsafe { device.SetViewport(&D3DVIEWPORT9 {
            X:      0,
            Y:      0,
            Width:  desc.Width,
            Height: desc.Height,
            MinZ:   0.0,
            MaxZ:   1.0,
        })};
        if !SUCCEEDED(hr) { return Err(Error::new_hr("IDirect3DDevice9::SetViewport", hr, "when binding MultiWindowContextLockWindow")); }

        Ok(())
    }

    pub fn client_size  (&self) -> (u32, u32)   { self.client_size }
    pub fn client_width (&self) -> u32          { self.client_size.0 }
    pub fn client_height(&self) -> u32          { self.client_size.1 }

    pub fn client_size_usize    (&self) -> (usize, usize)   { let (w, h) = self.client_size; (w as usize, h as usize) }
    pub fn client_width_usize   (&self) -> usize            { self.client_size.0 as usize }
    pub fn client_height_usize  (&self) -> usize            { self.client_size.1 as usize }
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
        self.windows.retain(|&hwnd| hwnd::assoc::valid_window(hwnd));
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
