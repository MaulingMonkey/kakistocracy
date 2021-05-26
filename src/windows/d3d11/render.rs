use crate::windows::*;

use winapi::shared::dxgi::*;
use winapi::shared::windef::*;
use winapi::shared::winerror::*;
use winapi::um::d3d11::*;

use std::ptr::null_mut;



pub trait Render {
    fn render(&self, args: &RenderArgs) -> RenderResult;
}

pub(super) struct RenderLock {
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
    pub(super) client_size: (u32, u32),
}

pub struct RenderResult(pub(super) Option<Error>);
impl    From<()>                  for RenderResult { fn from(_value: ()             ) -> Self { Self(None) } }
impl    From<HRESULT>             for RenderResult { fn from(value: HRESULT         ) -> Self { Self(if SUCCEEDED(value) { None } else { Some(Error::new_hr("", value, "")) }) } }
impl    From<Error>               for RenderResult { fn from(value: Error           ) -> Self { Self(Some(value)) } }
impl<T> From<Result<T, Error>>    for RenderResult { fn from(value: Result<T, Error>) -> Self { Self(value.err()) } }



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
