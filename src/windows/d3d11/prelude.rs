use crate::windows::Error;

use winapi::shared::dxgi::IDXGIDevice;
use winapi::um::d3d11::*;

use std::ptr::*;



pub trait D3d11DeviceExt {
    fn to_dxgi_device(&self) -> mcom::Rc<IDXGIDevice>;
    fn create_render_target_view_from_resource(&self, resource: &mcom::Rc<ID3D11Resource>) -> Result<mcom::Rc<ID3D11RenderTargetView>, Error>;
    /// ### Safety
    /// * `format` should be a valid format?
    /// * `mip_slice` should be a valid mip slice?
    unsafe fn create_render_target_view_from_texture2d(&self, texture: &mcom::Rc<ID3D11Texture2D>, format: u32, mip_slice: u32) -> Result<mcom::Rc<ID3D11RenderTargetView>, Error>;
}

impl D3d11DeviceExt for mcom::Rc<ID3D11Device> {
    fn to_dxgi_device(&self) -> mcom::Rc<IDXGIDevice> {
        self.try_cast::<IDXGIDevice>().unwrap()
    }

    fn create_render_target_view_from_resource(&self, resource: &mcom::Rc<ID3D11Resource>) -> Result<mcom::Rc<ID3D11RenderTargetView>, Error> {
        let mut rtv = null_mut();
        let hr = unsafe { self.CreateRenderTargetView(resource.as_ptr(), null_mut(), &mut rtv) };
        let rtv = unsafe { mcom::Rc::from_raw_opt(rtv) };
        rtv.ok_or(Error::new_hr("ID3D11Device::CreateRenderTargetView", hr, "ID3D11RenderTargetView is null"))
    }

    unsafe fn create_render_target_view_from_texture2d(&self, texture: &mcom::Rc<ID3D11Texture2D>, format: u32, mip_slice: u32) -> Result<mcom::Rc<ID3D11RenderTargetView>, Error> {
        let mut desc = D3D11_RENDER_TARGET_VIEW_DESC {
            Format:         format,
            ViewDimension:  D3D11_RTV_DIMENSION_TEXTURE2D,
            .. std::mem::zeroed()
        };
        desc.u.Texture2D_mut().MipSlice = mip_slice;

        let mut rtv = null_mut();
        let hr = self.CreateRenderTargetView(texture.up_ref().as_ptr(), &mut desc, &mut rtv);
        let rtv = mcom::Rc::from_raw_opt(rtv);
        rtv.ok_or(Error::new_hr("ID3D11Device::CreateRenderTargetView", hr, "ID3D11RenderTargetView is null"))
    }
}
