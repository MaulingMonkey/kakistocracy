use crate::io::StaticFile;
use crate::windows::Error;
use crate::windows::d3d11::Vertex;

use winapi::shared::dxgi::IDXGIDevice;
use winapi::shared::minwindef::*;
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;

use std::convert::*;
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

    /// [`ID3D11Device::CreateBuffer`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createbuffer)
    ///
    /// ### Safety
    /// Undefined behavior might result if:
    /// * `usage`/`bind` are invalid
    /// * `data` is too large
    /// * memory is exhausted
    unsafe fn create_buffer_from<D: 'static>(&self, usage: D3D11_USAGE, bind: D3D11_BIND_FLAG, data: &[D], debug_name: &str) -> Result<mcom::Rc<ID3D11Buffer>, Error>;

    /// [`ID3D11Device::CreateInputLayout`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createinputlayout)
    ///
    /// ### Safety
    /// Undefined behavior might result if:
    /// * `bytecode` contains invalid vertex shader bytecode
    unsafe fn create_input_layout_from<V: Vertex>(&self, bytecode: &[u8]) -> Result<mcom::Rc<ID3D11InputLayout>, Error>;

    /// [`ID3D11Device::CreateShaderResourceView`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createshaderresourceview)
    ///
    /// ### Safety
    /// Undefined behavior might result if:
    /// * `resource` belongs to another [`ID3D11Device`]
    unsafe fn create_shader_resource_view(&self, resource: &mcom::Rc<ID3D11Resource>) -> Result<mcom::Rc<ID3D11ShaderResourceView>, Error>;

    /// [`ID3D11Device::CreatePixelShader`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createpixelshader)
    unsafe fn create_pixel_shader(&self, file: &StaticFile) -> Result<mcom::Rc<ID3D11PixelShader>, Error>;

    /// [`ID3D11Device::CreateVertexShader`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createvertexshader)
    unsafe fn create_vertex_shader(&self, file: &StaticFile) -> Result<mcom::Rc<ID3D11VertexShader>, Error>;

    /// [`ID3D11Device::CreateSamplerState`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createsamplerstate)
    unsafe fn create_sampler_state(&self, desc: &D3D11_SAMPLER_DESC, debug_name: &str) -> Result<mcom::Rc<ID3D11SamplerState>, Error>;
}

/// Extension methods for [`ID3D11DeviceChild`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nn-d3d11-id3d11devicechild)
pub trait ID3D11DeviceChildExt {
    /// [`ID3D11DeviceChild::GetDevice`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11devicechild-getdevice)
    fn get_device(&self) -> mcom::Rc<ID3D11Device>;

    /// [`ID3D11DeviceChild::SetPrivateData`](https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11devicechild-setprivatedata)(...)
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error>;
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

    unsafe fn create_buffer_from<D: 'static>(&self, usage: D3D11_USAGE, bind: D3D11_BIND_FLAG, data: &[D], debug_name: &str) -> Result<mcom::Rc<ID3D11Buffer>, Error> {
        let stride : UINT = std::mem::size_of::<D>()    .try_into().map_err(|_| Error::new("ID3D11DeviceExt::create_buffer_from", "", 0, "size_of::<D>() exceeds UINT max"))?;
        let size   : UINT = std::mem::size_of_val(data) .try_into().map_err(|_| Error::new("ID3D11DeviceExt::create_buffer_from", "", 0, "size_of_val(data) exceeds UINT max"))?;
        let desc = D3D11_BUFFER_DESC { ByteWidth: size, Usage: usage, BindFlags: bind, CPUAccessFlags: 0, MiscFlags: 0, StructureByteStride: stride };
        let initial_data = D3D11_SUBRESOURCE_DATA { pSysMem: data.as_ptr().cast(), SysMemPitch: stride, SysMemSlicePitch: 0 };
        let mut buffer = null_mut();
        let hr = self.CreateBuffer(&desc, &initial_data, &mut buffer);
        let buffer = mcom::Rc::from_raw_opt(buffer).ok_or(Error::new_hr("ID3D11Device::CreateBuffer", hr, "ID3D11Buffer is null"))?;
        let _ = buffer.set_debug_name(debug_name);
        Ok(buffer)
    }

    unsafe fn create_input_layout_from<V: Vertex>(&self, bytecode: &[u8]) -> Result<mcom::Rc<ID3D11InputLayout>, Error> {
        let elements = V::elements();
        let elements = elements.as_ref();
        let nelements : UINT = elements.len().try_into().map_err(|_| Error::new("ID3D11DeviceExt::create_input_layout_from", "", 0, "V::elements().len() exceeds UINT max"))?;
        let mut layout = null_mut();
        let hr = self.CreateInputLayout(elements.as_ptr(), nelements, bytecode.as_ptr().cast(), bytecode.len(), &mut layout);
        let layout = mcom::Rc::from_raw_opt(layout).ok_or(Error::new_hr("ID3D111Device::CreateInputLayout", hr, "ID3D11InputLayout is null"))?;
        let _ = layout.set_debug_name(std::any::type_name::<V>());
        Ok(layout)
    }

    unsafe fn create_shader_resource_view(&self, resource: &mcom::Rc<ID3D11Resource>) -> Result<mcom::Rc<ID3D11ShaderResourceView>, Error> {
        let mut view = null_mut();
        let hr = self.CreateShaderResourceView(resource.as_ptr(), null(), &mut view);
        let view = mcom::Rc::from_raw_opt(view).ok_or(Error::new_hr("ID3D11Device::CreateShaderResourceView", hr, "ID3D11ShaderResourceView is null"))?;
        // debug name?
        Ok(view)
    }

    unsafe fn create_pixel_shader(&self, file: &StaticFile) -> Result<mcom::Rc<ID3D11PixelShader>, Error> {
        let bytes = file.as_bytes();
        let mut shader = null_mut();
        let hr = self.CreatePixelShader(bytes.as_ptr().cast(), bytes.len(), null_mut(), &mut shader);
        let shader = mcom::Rc::from_raw_opt(shader).ok_or(Error::new_hr("ID3D11Device::CreatePixelShader", hr, "ID3D11PixelShader is null"))?;
        let _ = shader.set_debug_name(file.path_str());
        Ok(shader)
    }

    unsafe fn create_vertex_shader(&self, file: &StaticFile) -> Result<mcom::Rc<ID3D11VertexShader>, Error> {
        let bytes = file.as_bytes();
        let mut shader = null_mut();
        let hr = self.CreateVertexShader(bytes.as_ptr().cast(), bytes.len(), null_mut(), &mut shader);
        let shader = mcom::Rc::from_raw_opt(shader).ok_or(Error::new_hr("ID3D11Device::CreateVertexShader", hr, "ID3D11VertexShader is null"))?;
        let _ = shader.set_debug_name(file.path_str());
        Ok(shader)
    }

    unsafe fn create_sampler_state(&self, desc: &D3D11_SAMPLER_DESC, debug_name: &str) -> Result<mcom::Rc<ID3D11SamplerState>, Error> {
        let mut ss = null_mut();
        let hr = self.CreateSamplerState(desc, &mut ss);
        let ss = mcom::Rc::from_raw_opt(ss).ok_or(Error::new_hr("ID3D11Device::CreateSamplerState", hr, "ID3D11SamplerState is null"))?;
        let _ = ss.set_debug_name(debug_name);
        Ok(ss)
    }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11DeviceChild> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> {
        let mut device = null_mut();
        unsafe { self.GetDevice(&mut device) };
        unsafe { mcom::Rc::from_raw(device) }
    }

    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> {
        if cfg!(debug_assertions) && !debug_name.is_empty() {
            let debug_name = debug_name.as_bytes();
            let hr = unsafe { self.SetPrivateData(&WKPDID_D3DDebugObjectName, debug_name.len() as _, debug_name.as_ptr().cast()) };
            Error::check_hr("ID3D11DeviceChild::SetPrivateData", hr, "setting WKPDID_D3DDebugObjectName")
        } else {
            Ok(())
        }
    }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11DeviceContext> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> { self.up_ref().get_device() }
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> { self.up_ref().set_debug_name(debug_name) }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11Resource> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> { self.up_ref().get_device() }
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> { self.up_ref().set_debug_name(debug_name) }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11Buffer> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> { self.up_ref().get_device() }
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> { self.up_ref().set_debug_name(debug_name) }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11Texture2D> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> { self.up_ref().get_device() }
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> { self.up_ref().set_debug_name(debug_name) }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11PixelShader> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> { self.up_ref().get_device() }
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> { self.up_ref().set_debug_name(debug_name) }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11VertexShader> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> { self.up_ref().get_device() }
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> { self.up_ref().set_debug_name(debug_name) }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11InputLayout> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> { self.up_ref().get_device() }
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> { self.up_ref().set_debug_name(debug_name) }
}

impl ID3D11DeviceChildExt for mcom::Rc<ID3D11SamplerState> {
    fn get_device(&self) -> mcom::Rc<ID3D11Device> { self.up_ref().get_device() }
    fn set_debug_name(&self, debug_name: &str) -> Result<(), Error> { self.up_ref().set_debug_name(debug_name) }
}
