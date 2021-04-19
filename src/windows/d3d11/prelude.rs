use crate::windows::Error;

use winapi::shared::dxgi::IDXGIDevice;
use winapi::um::d3d11::*;

use std::ptr::*;



/// Extension methods for [`ID3D11Device`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nn-d3d11-id3d11device)
pub trait ID3D11DeviceExt {
    /// <code>[IUnknown::QueryInterface](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-queryinterface(refiid_void))(__uuidof([IDXGIDevice](https://docs.microsoft.com/en-us/windows/win32/api/dxgi/nn-dxgi-idxgidevice)), ...)</code>
    fn to_dxgi_device(&self) -> mcom::Rc<IDXGIDevice>;

    /// [`ID3D11Device::CreateRenderTargetView`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createrendertargetview)
    fn create_render_target_view_from_resource(&self, resource: &mcom::Rc<ID3D11Resource>) -> Result<mcom::Rc<ID3D11RenderTargetView>, Error>;

    /// [`ID3D11Device::CreateRenderTargetView`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createrendertargetview)
    ///
    /// ### Safety
    /// Undefined behavior might result unless:
    /// * `format` is a valid format
    /// * `mip_slice` is a valid mip slice
    unsafe fn create_render_target_view_from_texture2d(&self, texture: &mcom::Rc<ID3D11Texture2D>, format: u32, mip_slice: u32) -> Result<mcom::Rc<ID3D11RenderTargetView>, Error>;
}

impl ID3D11DeviceExt for mcom::Rc<ID3D11Device> {
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
