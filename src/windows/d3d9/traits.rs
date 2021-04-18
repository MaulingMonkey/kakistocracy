use winapi::shared::d3d9types::*;



/// ### Safety
/// Undefined behavior may result if:
/// * The returned D3DFORMAT is not a valid D3DFMT_INDEX*
/// * The returned D3DFORMAT does not match the size of `Self`
pub unsafe trait Index { fn d3dfmt() -> D3DFORMAT; }
unsafe impl Index for u16 { fn d3dfmt() -> D3DFORMAT { D3DFMT_INDEX16 } }
unsafe impl Index for u32 { fn d3dfmt() -> D3DFORMAT { D3DFMT_INDEX32 } }

/// ### Safety
/// Undefined behavior may result if:
/// * The array does not end with `D3DDECL_END`
/// * The array has an absurd amount of elements
/// * Elements have offsets/sizes that go out-of-bounds for the vertex type
/// * Elements have invalid types/methods/usages
/// * Elements have absurd/large Streams or UsageIndexes
pub unsafe trait Vertex { type Decl : AsRef<[D3DVERTEXELEMENT9]>; fn elements() -> Self::Decl; }
