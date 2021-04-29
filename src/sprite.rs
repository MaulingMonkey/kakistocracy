//! [Sprite](https://en.wikipedia.org/wiki/Sprite_(computer_graphics)) rendering types/traits/functions

#![cfg_attr(not(all(windows, any(feature = "d3d9", feature = "d3d11"))), allow(dead_code))]

use crate::io::StaticFile;

use std::ops::*;



/// Render `instances` of `texture` to `target`
///
/// ### Safety
/// * `target` is expected to be "valid"
///     * render target 0 is expected to be valid/bound
///     * viewport is expected to be valid/bound
pub unsafe fn render1<RT: RenderTarget>(mut target: RT, texture: &StaticFile, instances: &[Instance]) {
    target.begin();
    target.render1(texture, instances);
    target.end();
}

/// A sprite instance
#[repr(C)]
#[derive(Clone)]
pub struct Instance {
    /// The X/Y/Z viewport position to render the sprite at.
    /// This is the center of rotation, and what "dimensions" is relative to.
    pub anchor:     [f32; 3],

    /// How much to rotate the sprite around the anchor, clockwise, in radians.
    pub rotation:   f32,

    /// The pixel coordinates (relative to the anchor, in the rotated frame) to render the sprite with.
    pub dimensions: [Range<f32>; 2],

    /// The UV coordinates to render the sprite with.
    pub texcoords:  [Range<f32>; 2],
}

/// [`IDirect3DDevice9`](winapi::shared::d3d9::IDirect3DDevice9) /
/// [`ID3D11DeviceContext`](winapi::um::d3d11::ID3D11DeviceContext):
/// targets sprites can be rendered to
pub trait RenderTarget : private::RenderTarget {}
impl<T: private::RenderTarget> RenderTarget for T {}



#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Vertex {
    pub position: [f32; 4],
    pub texcoord: [f32; 2],
}



pub(crate) fn create_quads_index_data<I>(quads: I) -> Vec<I> where
    I: Copy + From<u8> + Add<Output = I> + Mul<Output = I>,
    RangeInclusive<I> : IntoIterator<Item = I>,
{
    let mut indicies = Vec::new();
    for quad in I::from(0) ..= quads {
        indicies.push(I::from(4) * quad + I::from(0));
        indicies.push(I::from(4) * quad + I::from(1));
        indicies.push(I::from(4) * quad + I::from(2));

        indicies.push(I::from(4) * quad + I::from(0));
        indicies.push(I::from(4) * quad + I::from(2));
        indicies.push(I::from(4) * quad + I::from(3));
    }
    indicies
}

pub(crate) mod private {
    use super::*;

    pub trait RenderTarget {
        unsafe fn begin(&mut self) {}
        unsafe fn render1(&mut self, texture: &StaticFile, instances: &[Instance]);
        unsafe fn end(&mut self) {}
    }
}
