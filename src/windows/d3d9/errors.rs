use crate::windows::error;

use winapi::shared::winerror::*;



// C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\shared\d3d9.h

pub(crate) const D3D_OK                     : HRESULT = 0;

pub(crate) const D3DERR_DRIVERINTERNALERROR : HRESULT = MAKE_D3DHRESULT(2087);

pub(crate) const D3DERR_NOTFOUND            : HRESULT = MAKE_D3DHRESULT(2150);
pub(crate) const D3DERR_DEVICELOST          : HRESULT = MAKE_D3DHRESULT(2152);
pub(crate) const D3DERR_DEVICENOTRESET      : HRESULT = MAKE_D3DHRESULT(2153);
pub(crate) const D3DERR_NOTAVAILABLE        : HRESULT = MAKE_D3DHRESULT(2154);
pub(crate) const D3DERR_OUTOFVIDEOMEMORY    : HRESULT = MAKE_D3DHRESULT(380);
pub(crate) const D3DERR_INVALIDCALL         : HRESULT = MAKE_D3DHRESULT(2156);

const _FACD3D : u16 = 0x876;
#[allow(non_snake_case)] const fn MAKE_D3DHRESULT(code: u16) -> HRESULT { error::MAKE_HRESULT(1, _FACD3D, code) }
//#[allow(non_snake_case)] const fn MAKE_D3DSTATUS (code: u16) -> HRESULT { error::MAKE_HRESULT(0, _FACD3D, code) }
