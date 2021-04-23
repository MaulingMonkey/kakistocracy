use crate::windows::error;

use winapi::shared::winerror::*;



pub(crate) const D3DERR_NOTFOUND : HRESULT = MAKE_D3DHRESULT(2150);

const _FACD3D : u16 = 0x876;
#[allow(non_snake_case)] const fn MAKE_D3DHRESULT(code: u16) -> HRESULT { error::MAKE_HRESULT(1, _FACD3D, code) }
//#[allow(non_snake_case)] const fn MAKE_D3DSTATUS (code: u16) -> HRESULT { error::MAKE_HRESULT(0, _FACD3D, code) }
