use winapi::shared::d3d9types::*;



pub unsafe trait Index { fn d3dfmt() -> D3DFORMAT; }
unsafe impl Index for u16 { fn d3dfmt() -> D3DFORMAT { D3DFMT_INDEX16 } }
unsafe impl Index for u32 { fn d3dfmt() -> D3DFORMAT { D3DFMT_INDEX32 } }
