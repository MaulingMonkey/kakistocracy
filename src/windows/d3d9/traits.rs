use winapi::shared::d3d9types::*;



/// [`u16`]/[`u32`]: Valid types for [`IDirect3DIndexBuffer9`] contents.
///
/// ### Safety
/// Undefined behavior may result if:
/// * The returned `D3DFORMAT` is not a valid `D3DFMT_INDEX*`
/// * The returned `D3DFORMAT` does not match the size of `Self`
///
/// [`IDirect3DIndexBuffer9`]:      https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3dindexbuffer9
pub unsafe trait Index {
    /// Returns `D3DFMT_INDEX16` or `D3DFMT_INDEX32`
    fn d3dfmt() -> D3DFORMAT;
}

/// `D3DFMT_INDEX16`
unsafe impl Index for u16 { fn d3dfmt() -> D3DFORMAT { D3DFMT_INDEX16 } }

/// `D3DFMT_INDEX32`
unsafe impl Index for u32 { fn d3dfmt() -> D3DFORMAT { D3DFMT_INDEX32 } }



/// Describe a Vertex for a [`IDirect3DVertexBuffer9`] / [`IDirect3DVertexDeclaration9`].
///
/// ### Safety
/// Undefined behavior may result if:
/// * The array does not end with `D3DDECL_END`
/// * The array has an absurd amount of elements
/// * Elements have offsets/sizes that go out-of-bounds for the vertex type
/// * Elements have invalid types/methods/usages
/// * Elements have absurd/large Streams or UsageIndexes
///
/// [`IDirect3DVertexBuffer9`]:         https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3dvertexbuffer9
/// [`IDirect3DVertexDeclaration9`]:    https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3dvertexdeclaration9
pub unsafe trait Vertex {
    type Decl : AsRef<[D3DVERTEXELEMENT9]>;

    /// Returns an array or vec of [`D3DVERTEXELEMENT9`]s, ending with [`D3DDECL_END`], suitable for passing to [`IDirect3DDevice9::CreateVertexDeclaration`].
    ///
    /// [`D3DVERTEXELEMENT9`]:                          https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3dvertexelement9
    /// [`D3DDECL_END`]:                                https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3ddecl-end
    /// [`IDirect3DDevice9::CreateVertexDeclaration`]:  https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nf-d3d9helper-idirect3ddevice9-createvertexdeclaration
    fn elements() -> Self::Decl;
}
