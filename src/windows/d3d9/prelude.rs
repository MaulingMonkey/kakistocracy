use crate::windows::Error;

use mcom::AsIUnknown;
use winapi::Interface;
use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::guiddef::GUID;
use winapi::shared::minwindef::DWORD;
use winapi::um::unknwnbase::IUnknown;

use std::convert::*;
use std::ptr::*;



pub trait D3d9DeviceExt {
    fn get_back_buffer(&self, swap_chain: u32, back_buffer: u32) -> Result<mcom::Rc<IDirect3DSurface9>, Error>;
}

pub trait D3d9SwapChainExt {
    fn get_back_buffer(&self, back_buffer: u32) -> Result<mcom::Rc<IDirect3DSurface9>, Error>;
}

pub trait D3d9ResourceExt {
    /// [`IDirect3DResource9::FreePrivateData`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3dresource9-freeprivatedata)
    fn free_private_data(&self, guid: &GUID) -> Result<(), Error>;

    /// [`IDirect3DResource9::GetPrivateData`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3dresource9-getprivatedata)
    fn get_private_data_raw<'d>(&self, guid: &GUID, data: &'d mut [u8]) -> Result<&'d [u8], Error>;

    /// [`IDirect3DResource9::GetPrivateData`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3dresource9-getprivatedata),
    /// cast to `*const IUnknown`, and `QueryInterface(...)`ed to `I`.
    ///
    /// ### Safety
    /// If any private data is associated with `guid`, it *must* be a "valid" (or null) `*IUnknown`, or this will result in undefined behavior.
    /// It is recommended that you only reuse the `guid` with `set_private_data_com`, which should ensure soundness.
    unsafe fn get_private_data_com<I: Interface>(&self, guid: &GUID) -> Result<mcom::Rc<I>, Error>;

    /// [`IDirect3DResource9::SetPrivateData`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3dresource9-setprivatedata) with `Flags = 0`
    fn set_private_data_raw(&self, guid: &GUID, data: &       [u8]) -> Result<(),       Error>;

    /// [`IDirect3DResource9::SetPrivateData`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3dresource9-setprivatedata) with `Flags = D3DSPD_IUNKNOWN`
    fn set_private_data_com<I: Interface>(&self, guid: &GUID, data: &mcom::Rc<I>) -> Result<(), Error>;
}



impl D3d9DeviceExt for mcom::Rc<IDirect3DDevice9> {
    fn get_back_buffer(&self, swap_chain: u32, back_buffer: u32) -> Result<mcom::Rc<IDirect3DSurface9>, Error> {
        let mut bb = null_mut();
        let hr = unsafe { self.GetBackBuffer(swap_chain, back_buffer, D3DBACKBUFFER_TYPE_MONO, &mut bb) };
        unsafe { mcom::Rc::from_raw_opt(bb).ok_or(Error::new_hr("IDirect3DSwapChain9::GetBackBuffer", hr, "IDirect3DSurface9 is null")) }
    }
}

impl D3d9SwapChainExt for mcom::Rc<IDirect3DSwapChain9> {
    fn get_back_buffer(&self, back_buffer: u32) -> Result<mcom::Rc<IDirect3DSurface9>, Error> {
        let mut bb = null_mut();
        let hr = unsafe { self.GetBackBuffer(back_buffer, D3DBACKBUFFER_TYPE_MONO, &mut bb) };
        unsafe { mcom::Rc::from_raw_opt(bb).ok_or(Error::new_hr("IDirect3DSwapChain9::GetBackBuffer", hr, "IDirect3DSurface9 is null")) }
    }
}

impl D3d9ResourceExt for mcom::Rc<IDirect3DResource9> {
    fn free_private_data(&self, guid: &GUID) -> Result<(), Error> {
        let hr = unsafe { self.FreePrivateData(guid) };
        Error::check_hr("IDirect3DResource9::FreePrivateData", hr, "")
    }

    fn get_private_data_raw<'d>(&self, guid: &GUID, data: &'d mut [u8]) -> Result<&'d [u8], Error> {
        let max = DWORD::try_from(data.len()).map_err(|_| Error::new("<D3d9DResourceExt for mcom::Rc<IDirect3DResource9>>::get_private_data", "", 0, "data length exceeds DWORD"))?;
        let mut n : DWORD = max;
        let hr = unsafe { self.GetPrivateData(guid, data.as_mut_ptr().cast(), &mut n) };
        let read = n.min(max) as usize;
        Error::check_hr("IDirect3DResource9::GetPrivateData", hr, "")?;
        Ok(&data[..read])
    }

    fn set_private_data_raw(&self, guid: &GUID, data: &[u8]) -> Result<(), Error> {
        let n : DWORD = data.len().try_into().map_err(|_| Error::new("<D3d9DResourceExt for mcom::Rc<IDirect3DResource9>>::set_private_data", "", 0, "data length exceeds DWORD"))?;
        let hr = unsafe { self.SetPrivateData(guid, data.as_ptr().cast(), n, 0) };
        Error::check_hr("IDirect3DResource9::GetPrivateData", hr, "")
    }

    unsafe fn get_private_data_com<I: Interface>(&self, guid: &GUID) -> Result<mcom::Rc<I>, Error> {
        // NOTE: GetPrivateData does call data->AddRef() if it was set via D3DSPD_IUNKNOWN
        let sizeof_iunknown_ptr = std::mem::size_of::<*mut I>() as DWORD;
        let mut n = sizeof_iunknown_ptr;
        let mut data = 0usize.to_ne_bytes();
        let hr = self.GetPrivateData(guid, data.as_mut_ptr().cast(), &mut n);
        assert!(n == 0 || n == sizeof_iunknown_ptr, "D3d9ResourceExt::get_private_data_com called on private data that wasn't pointer sized.  This probably isn't a COM pointer.  This probably *is* a serious bug that may lead to undefined behavior if the private data ever has the same size as a COM pointer.");
        Error::check_hr("IDirect3DResource9::GetPrivateData", hr, "")?;
        let data = usize::from_ne_bytes(data) as *mut IUnknown;
        let data = mcom::Rc::from_raw_opt(data).ok_or(Error::new("D3d9ResourceExt::get_private_data_com", "", 0, "instance is null"))?;
        let data = data.try_cast::<I>().ok_or(Error::new("D3d9ResourceExt::get_private_data_com", "", 0, "instance doesn't implement the interface"))?;
        Ok(data)
    }

    fn set_private_data_com<I: Interface>(&self, guid: &GUID, data: &mcom::Rc<I>) -> Result<(), Error> {
        // NOTE: SetPrivateData does call data->AddRef()
        const D3DSPD_IUNKNOWN : DWORD = 0x00000001; // C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\shared\d3d9.h
        let hr = unsafe { self.SetPrivateData(guid, data.as_iunknown_ptr().cast(), std::mem::size_of::<*mut IUnknown>() as DWORD, D3DSPD_IUNKNOWN) };
        Error::check_hr("IDirect3DResource9::SetPrivateData", hr, "")?;
        Ok(())
    }
}

impl D3d9ResourceExt for mcom::Rc<IDirect3DSurface9> {
    fn free_private_data(&self, guid: &GUID) -> Result<(), Error> { self.up_ref().free_private_data(guid) }
    fn get_private_data_raw<'d>(&self, guid: &GUID, data: &'d mut [u8]) -> Result<&'d [u8], Error> { self.up_ref().get_private_data_raw(guid, data) }
    fn set_private_data_raw    (&self, guid: &GUID, data: &       [u8]) -> Result<(),       Error> { self.up_ref().set_private_data_raw(guid, data) }
    unsafe fn get_private_data_com<I: Interface>(&self, guid: &GUID) -> Result<mcom::Rc<I>, Error> { self.up_ref().get_private_data_com(guid) }
    fn set_private_data_com<I: Interface>(&self, guid: &GUID, data: &mcom::Rc<I>) -> Result<(), Error> { self.up_ref().set_private_data_com(guid, data) }
}
