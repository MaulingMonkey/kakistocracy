use winapi::shared::dxgiformat::*;
use winapi::um::d3d11::*;



/// [`u16`]/[`u32`]: Valid types for Index [`ID3D11Buffer`] contents.
///
/// ### Safety
/// Undefined behavior may result if:
/// * The returned `DXGI_FORMAT` is not `DXGI_FORMAT_R{16,32}_UINT`
/// * The returned `DXGI_FORMAT` does not match the size of `Self`
///
/// [`IDirect3DIndexBuffer9`]:      https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3dindexbuffer9
pub unsafe trait Index {
    /// Returns `DXGI_FORMAT_R16_UINT` or `DXGI_FORMAT_R32_UINT`
    fn dxgi_format() -> DXGI_FORMAT;
}

/// `DXGI_FORMAT_R16_UINT`
unsafe impl Index for u16 { fn dxgi_format() -> DXGI_FORMAT { DXGI_FORMAT_R16_UINT } }

/// `DXGI_FORMAT_R32_UINT`
unsafe impl Index for u32 { fn dxgi_format() -> DXGI_FORMAT { DXGI_FORMAT_R32_UINT } }



/// Describe a Vertex for a [`ID3D11Buffer`] / [`ID3D11InputLayout`].
///
/// ### Safety
/// Undefined behavior may result if:
/// * The array has an absurd amount of elements
/// * Elements have offsets/sizes that go out-of-bounds for the vertex type
/// * Elements have invalid semantics/formats/slots/step rates/...
/// * Elements have absurd/large SemanticIndexs/...
///
/// [`ID3D11Buffer`]:       https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nn-d3d11-id3d11buffer
/// [`ID3D11InputLayout`]:  https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nn-d3d11-id3d11inputlayout
pub unsafe trait Vertex : Sized {
    type Decl : AsRef<[D3D11_INPUT_ELEMENT_DESC]>;

    /// Returns an array or vec of [`D3D11_INPUT_ELEMENT_DESC`]s, suitable for passing to [`ID3D11Device::CreateInputLayout`].
    ///
    /// [`D3D11_INPUT_ELEMENT_DESC `]:          https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ns-d3d11-d3d11_input_element_desc
    /// [`ID3D11Device::CreateInputLayout`]:    https://docs.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createinputlayout
    fn elements() -> Self::Decl;

    /// Returns the stride of a given vertex.
    fn stride() -> u32 { std::mem::size_of::<Self>() as _ }
}
