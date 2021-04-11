use crate::windows::*;

use winapi::shared::d3d9::*;

use std::ptr::null_mut;



/// Create a [`IDirect3D9`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3d9) via [`Direct3DCreate9`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-direct3dcreate9)
pub fn create_d3d(allow_debug: bool) -> Result<mcom::Rc<IDirect3D9>, Error> {
    let mut d3d = null_mut();
    if allow_debug      { d3d = unsafe { Direct3DCreate9(D3D_SDK_VERSION | D3D_SDK_DEBUG) }; }
    if d3d.is_null()    { d3d = unsafe { Direct3DCreate9(D3D_SDK_VERSION | 0            ) }; }
    unsafe { mcom::Rc::from_raw_opt(d3d) }.ok_or(Error::new("Direct3DCreate9", "", 0, "IDirect3D9 is null"))
}

/// Create a [`IDirect3D9Ex`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nn-d3d9-idirect3d9ex) via [`Direct3DCreate9Ex`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-direct3dcreate9ex)
pub fn create_d3d_ex(allow_debug: bool) -> Result<mcom::Rc<IDirect3D9Ex>, Error> {
    let mut d3d = null_mut();
    let mut hr = 0;
    if allow_debug      { hr = unsafe { Direct3DCreate9Ex(D3D_SDK_VERSION | D3D_SDK_DEBUG, &mut d3d) }; }
    if d3d.is_null()    { hr = unsafe { Direct3DCreate9Ex(D3D_SDK_VERSION | 0            , &mut d3d) }; }
    unsafe { mcom::Rc::from_raw_opt(d3d) }.ok_or(Error::new_hr("Direct3DCreate9Ex", hr, "IDirect3D9Ex is null"))
}

const D3D_SDK_DEBUG : u32 = 0x80000000;



#[test] fn test_create_d3d() {
    let _ = create_d3d(true).unwrap();
    let _ = create_d3d(true).unwrap();
}

#[test] fn test_create_d3d_ex() {
    let _ = create_d3d_ex(true).unwrap();
    let _ = create_d3d_ex(true).unwrap();
}
