use crate::windows::*;

use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::windef::*;
use winapi::shared::winerror::SUCCEEDED;

use std::ptr::null_mut;



pub trait Render {
    fn render(&self, args: &RenderArgs);
}

pub struct RenderArgs {
    pub d3d:        mcom::Rc<IDirect3D9>,
    pub device:     mcom::Rc<IDirect3DDevice9>,
    pub window:     HWND,
    pub swap_chain: mcom::Rc<IDirect3DSwapChain9>,
    pub(crate) client_size: (u32, u32),
}

impl RenderArgs {
    /// Binds the next back buffer of the window's swap chain as the render target, and sets the viewport to the entire window.
    pub fn bind(&self) -> Result<(), Error> {
        let device = &self.device;

        let mut bb = null_mut();
        let hr = unsafe { self.swap_chain.GetBackBuffer(0, D3DBACKBUFFER_TYPE_MONO, &mut bb) };
        let bb = unsafe { mcom::Rc::from_raw_opt(bb) }.ok_or(Error::new_hr("IDirect3DSwapChain9::GetBackBuffer", hr, "IDirect3DSurface9 is null"))?;

        let mut desc = unsafe { std::mem::zeroed() };
        let hr = unsafe { bb.GetDesc(&mut desc) };
        if !SUCCEEDED(hr) { return Err(Error::new_hr("IDirect3DSurface9::GetDesc", hr, "when binding RenderArgs")); }

        let hr = unsafe { device.SetRenderTarget(0, bb.as_ptr()) };
        if !SUCCEEDED(hr) { return Err(Error::new_hr("IDirect3DDevice9::SetRenderTarget", hr, "when binding RenderArgs")); }

        let hr = unsafe { device.SetViewport(&D3DVIEWPORT9 {
            X:      0,
            Y:      0,
            Width:  desc.Width,
            Height: desc.Height,
            MinZ:   0.0,
            MaxZ:   1.0,
        })};
        if !SUCCEEDED(hr) { return Err(Error::new_hr("IDirect3DDevice9::SetViewport", hr, "when binding RenderArgs")); }

        Ok(())
    }

    pub fn client_size  (&self) -> (u32, u32)   { self.client_size }
    pub fn client_width (&self) -> u32          { self.client_size.0 }
    pub fn client_height(&self) -> u32          { self.client_size.1 }

    pub fn client_size_usize    (&self) -> (usize, usize)   { let (w, h) = self.client_size; (w as usize, h as usize) }
    pub fn client_width_usize   (&self) -> usize            { self.client_size.0 as usize }
    pub fn client_height_usize  (&self) -> usize            { self.client_size.1 as usize }
}
