use crate::windows::Error;
use crate::windows::d3d9::{Index, Vertex};

use mcom::AsIUnknown;
use winapi::Interface;
use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::guiddef::GUID;
use winapi::shared::minwindef::{DWORD, UINT};
use winapi::um::unknwnbase::IUnknown;

use std::convert::*;
use std::ptr::*;



/// Extension methods for [`IDirect3DDevice9`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nn-d3d9-idirect3ddevice9)
pub trait IDirect3DDevice9Ext {
    /// [`IDirect3DDevice9::GetBackBuffer`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-getbackbuffer)
    ///
    /// ### Safety
    /// Undefined behavior might result unless:
    /// * `swap_chain` is a valid swap chain
    /// * `back_buffer` is a valid back buffer
    unsafe fn get_back_buffer(&self, swap_chain: u32, back_buffer: u32) -> Result<mcom::Rc<IDirect3DSurface9>, Error>;

    /// [`IDirect3DDevice9::CreateIndexBuffer`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-createindexbuffer)
    /// + `Lock` + `memcpy` + `Unlock`
    ///
    /// ### Safety
    /// Undefined behavior might result unless:
    /// * `usage` is a valid usage for index buffers
    /// * `pool` is a valid pool for index buffers
    /// * `data` contains a reasonably limited number of indicies
    ///     * Running out of memory may result in UB
    ///     * Running out of address space may result in UB
    ///     * Overflowing D3D-internal u32s based on the length of `data` may result in UB
    unsafe fn create_index_buffer_from<I: Index>(&self, usage: DWORD, pool: D3DPOOL, data: &[I]) -> Result<mcom::Rc<IDirect3DIndexBuffer9>, Error>;

    /// [`IDirect3DDevice9::CreateVertexBuffer`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-createvertexbuffer)
    /// + `Lock` + `memcpy` + `Unlock`
    ///
    /// ### Safety
    /// Undefined behavior might result unless:
    /// * `usage` is a valid usage for vertex buffers
    /// * `pool` is a valid pool for vertex buffers
    /// * `data` contains a reasonably limited number of indicies
    ///     * Running out of memory may result in UB
    ///     * Running out of address space may result in UB
    ///     * Overflowing D3D-internal u32s based on the length of `data` may result in UB
    unsafe fn create_vertex_buffer_from<V: Vertex>(&self, usage: DWORD, pool: D3DPOOL, data: &[V]) -> Result<mcom::Rc<IDirect3DVertexBuffer9>, Error>;

    /// [`IDirect3DDevice9::CreateVertexDeclaration`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-createvertexdeclaration)
    fn create_vertex_decl_from<V: Vertex>(&self) -> Result<mcom::Rc<IDirect3DVertexDeclaration9>, Error>;

    /// [`IDirect3DDevice9::CreateVertexBuffer`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-createvertexbuffer)
    /// + `Lock` + `memcpy` + `Unlock`
    /// + [`CreateVertexDeclaration`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-createvertexdeclaration)
    ///
    /// ### Safety
    /// Undefined behavior might result unless:
    /// * `usage` is a valid usage for vertex buffers
    /// * `pool` is a valid pool for vertex buffers
    /// * `data` contains a reasonably limited number of verticies
    ///     * Running out of memory may result in UB
    ///     * Running out of address space may result in UB
    ///     * Overflowing D3D-internal u32s based on the length of `data` may result in UB
    unsafe fn create_vertex_buffer_decl_from<V: Vertex>(&self, usage: DWORD, pool: D3DPOOL, data: &[V]) -> Result<(mcom::Rc<IDirect3DVertexBuffer9>, mcom::Rc<IDirect3DVertexDeclaration9>), Error> {
        Ok((self.create_vertex_buffer_from(usage, pool, data)?, self.create_vertex_decl_from::<V>()?))
    }
}

/// Extension methods for [`IDirect3DSwapChain9`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nn-d3d9-idirect3dswapchain9)
pub trait IDirect3DSwapChain9Ext {
    /// [`IDirect3DSwapChain9::GetBackBuffer`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3dswapchain9-getbackbuffer)
    ///
    /// ### Safety
    /// Undefined behavior might result unless:
    /// * `back_buffer` is a valid back buffer
    unsafe fn get_back_buffer(&self, back_buffer: u32) -> Result<mcom::Rc<IDirect3DSurface9>, Error>;
}

/// Extension methods for [`IDirect3DResource9`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nn-d3d9-idirect3dresource9)
pub trait IDirect3DResource9Ext {
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



impl IDirect3DDevice9Ext for mcom::Rc<IDirect3DDevice9> {
    unsafe fn get_back_buffer(&self, swap_chain: u32, back_buffer: u32) -> Result<mcom::Rc<IDirect3DSurface9>, Error> {
        let mut bb = null_mut();
        let hr = self.GetBackBuffer(swap_chain, back_buffer, D3DBACKBUFFER_TYPE_MONO, &mut bb);
        mcom::Rc::from_raw_opt(bb).ok_or(Error::new_hr("IDirect3DSwapChain9::GetBackBuffer", hr, "IDirect3DSurface9 is null"))
    }

    unsafe fn create_index_buffer_from<I: Index>(&self, usage: DWORD, pool: D3DPOOL, data: &[I]) -> Result<mcom::Rc<IDirect3DIndexBuffer9>, Error> {
        let size : UINT = std::mem::size_of_val(data).try_into().map_err(|_| Error::new("mcom::Rc<IDirect3DDevice9>::create_index_buffer_from", "", 0, "size_of_val(data) exceeded UINT"))?;
        let mut ib = null_mut();
        let hr = self.CreateIndexBuffer(size, usage, I::d3dfmt(), pool, &mut ib, null_mut());
        let ib = mcom::Rc::from_raw_opt(ib).ok_or(Error::new_hr("IDirect3DDevice9::CreateIndexBuffer", hr, "IDirect3DIndexBuffer9 is null"))?;

        let mut lock = null_mut();
        let hr = ib.Lock(0, 0, &mut lock, 0);
        Error::check_hr("IDirect3DIndexBuffer9::Lock", hr, "")?;

        std::ptr::copy_nonoverlapping(data.as_ptr(), lock as *mut I, data.len());

        let hr = ib.Unlock();
        Error::check_hr("IDirect3DIndexBuffer9::Unlock", hr, "")?;

        Ok(ib)
    }

    unsafe fn create_vertex_buffer_from<V: Vertex>(&self, usage: DWORD, pool: D3DPOOL, data: &[V]) -> Result<mcom::Rc<IDirect3DVertexBuffer9>, Error> {
        let size : UINT = std::mem::size_of_val(data).try_into().map_err(|_| Error::new("mcom::Rc<IDirect3DDevice9>::create_vertex_buffer_decl_from", "", 0, "size_of_val(data) exceeded UINT"))?;
        let mut vb = null_mut();
        let hr = self.CreateVertexBuffer(size, usage, 0, pool, &mut vb, null_mut());
        let vb = mcom::Rc::from_raw_opt(vb).ok_or(Error::new_hr("IDirect3DDevice9::CreateVertexBuffer", hr, "IDirect3DVertexBuffer9 is null"))?;

        let mut lock = null_mut();
        let hr = vb.Lock(0, 0, &mut lock, 0);
        Error::check_hr("IDirect3DVertexBuffer9::Lock", hr, "")?;

        std::ptr::copy_nonoverlapping(data.as_ptr(), lock as *mut V, data.len());

        let hr = vb.Unlock();
        Error::check_hr("IDirect3DVertexBuffer9::Unlock", hr, "")?;

        Ok(vb)
    }

    fn create_vertex_decl_from<V: Vertex>(&self) -> Result<mcom::Rc<IDirect3DVertexDeclaration9>, Error> {
        let elements = V::elements();
        let elements = elements.as_ref();
        if elements.is_empty() { return Err(Error::new("mcom::Rc<IDirect3DDevice9>::create_vertex_decl_from", "", 0, "elements is empty")) }
        let (mid, end) = elements.split_at(elements.len()-1);
        for mid in mid.iter() { if  eq(mid, &D3DDECL_END) { return Err(Error::new("mcom::Rc<IDirect3DDevice9>::create_vertex_decl_from", "", 0, "V::elements() contains non-terminal D3DDECL_END")) } }
        for end in end.iter() { if !eq(end, &D3DDECL_END) { return Err(Error::new("mcom::Rc<IDirect3DDevice9>::create_vertex_decl_from", "", 0, "V::elements() does not end with D3DDECL_END")) } }

        fn eq(a: &D3DVERTEXELEMENT9, b: &D3DVERTEXELEMENT9) -> bool {
            (a.Method == b.Method) &&
            (a.Offset == b.Offset) &&
            (a.Stream == b.Stream) &&
            (a.Type   == b.Type  ) &&
            (a.Usage  == b.Usage ) &&
            (a.UsageIndex == b.UsageIndex)
        }

        let mut vd = null_mut();
        let hr = unsafe { self.CreateVertexDeclaration(elements.as_ptr(), &mut vd) };
        unsafe { mcom::Rc::from_raw_opt(vd) }.ok_or(Error::new_hr("IDirect3DDevice9::CreateVertexDeclaration", hr, "IDirect3DVertexDeclaration9 is null"))
    }
}

impl IDirect3DSwapChain9Ext for mcom::Rc<IDirect3DSwapChain9> {
    unsafe fn get_back_buffer(&self, back_buffer: u32) -> Result<mcom::Rc<IDirect3DSurface9>, Error> {
        let mut bb = null_mut();
        let hr = self.GetBackBuffer(back_buffer, D3DBACKBUFFER_TYPE_MONO, &mut bb);
        mcom::Rc::from_raw_opt(bb).ok_or(Error::new_hr("IDirect3DSwapChain9::GetBackBuffer", hr, "IDirect3DSurface9 is null"))
    }
}

impl IDirect3DResource9Ext for mcom::Rc<IDirect3DResource9> {
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
        assert!(n == 0 || n == sizeof_iunknown_ptr, "IDirect3DResource9Ext::get_private_data_com called on private data that wasn't pointer sized.  This probably isn't a COM pointer.  This probably *is* a serious bug that may lead to undefined behavior if the private data ever has the same size as a COM pointer.");
        Error::check_hr("IDirect3DResource9::GetPrivateData", hr, "")?;
        let data = usize::from_ne_bytes(data) as *mut IUnknown;
        let data = mcom::Rc::from_raw_opt(data).ok_or(Error::new("IDirect3DResource9Ext::get_private_data_com", "", 0, "instance is null"))?;
        let data = data.try_cast::<I>().ok_or(Error::new("IDirect3DResource9Ext::get_private_data_com", "", 0, "instance doesn't implement the interface"))?;
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

impl IDirect3DResource9Ext for mcom::Rc<IDirect3DSurface9> {
    fn free_private_data(&self, guid: &GUID) -> Result<(), Error> { self.up_ref().free_private_data(guid) }
    fn get_private_data_raw<'d>(&self, guid: &GUID, data: &'d mut [u8]) -> Result<&'d [u8], Error> { self.up_ref().get_private_data_raw(guid, data) }
    fn set_private_data_raw    (&self, guid: &GUID, data: &       [u8]) -> Result<(),       Error> { self.up_ref().set_private_data_raw(guid, data) }
    unsafe fn get_private_data_com<I: Interface>(&self, guid: &GUID) -> Result<mcom::Rc<I>, Error> { self.up_ref().get_private_data_com(guid) }
    fn set_private_data_com<I: Interface>(&self, guid: &GUID, data: &mcom::Rc<I>) -> Result<(), Error> { self.up_ref().set_private_data_com(guid, data) }
}

impl IDirect3DResource9Ext for mcom::Rc<IDirect3DBaseTexture9> {
    fn free_private_data(&self, guid: &GUID) -> Result<(), Error> { self.up_ref().free_private_data(guid) }
    fn get_private_data_raw<'d>(&self, guid: &GUID, data: &'d mut [u8]) -> Result<&'d [u8], Error> { self.up_ref().get_private_data_raw(guid, data) }
    fn set_private_data_raw    (&self, guid: &GUID, data: &       [u8]) -> Result<(),       Error> { self.up_ref().set_private_data_raw(guid, data) }
    unsafe fn get_private_data_com<I: Interface>(&self, guid: &GUID) -> Result<mcom::Rc<I>, Error> { self.up_ref().get_private_data_com(guid) }
    fn set_private_data_com<I: Interface>(&self, guid: &GUID, data: &mcom::Rc<I>) -> Result<(), Error> { self.up_ref().set_private_data_com(guid, data) }
}

impl IDirect3DResource9Ext for mcom::Rc<IDirect3DTexture9> {
    fn free_private_data(&self, guid: &GUID) -> Result<(), Error> { self.up_ref().free_private_data(guid) }
    fn get_private_data_raw<'d>(&self, guid: &GUID, data: &'d mut [u8]) -> Result<&'d [u8], Error> { self.up_ref().get_private_data_raw(guid, data) }
    fn set_private_data_raw    (&self, guid: &GUID, data: &       [u8]) -> Result<(),       Error> { self.up_ref().set_private_data_raw(guid, data) }
    unsafe fn get_private_data_com<I: Interface>(&self, guid: &GUID) -> Result<mcom::Rc<I>, Error> { self.up_ref().get_private_data_com(guid) }
    fn set_private_data_com<I: Interface>(&self, guid: &GUID, data: &mcom::Rc<I>) -> Result<(), Error> { self.up_ref().set_private_data_com(guid, data) }
}
