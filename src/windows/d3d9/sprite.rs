//! Sprite rendering utilities

use crate::io::StaticFile;
use crate::windows::*;
use crate::windows::d3d9::{BasicTextureCache, Vertex};

use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::UINT;

use std::convert::*;
use std::ops::Range;
use std::ptr::*;



struct SpriteRenderer<'d> {
    device:     &'d mcom::Rc<IDirect3DDevice9>,
    viewport:   [Range<f32>; 2],
    textures:   UnkWrapRc<BasicTextureCache>,
    resources:  UnkWrapRc<Resources>,
}

impl<'d> SpriteRenderer<'d> {
    pub unsafe fn new(device: &'d mcom::Rc<IDirect3DDevice9>) -> Self {
        let resources   = d3d9::device_private_data_get_or_insert(device, || Resources::new(device));
        let textures    = d3d9::device_private_data_get_or_insert(device, || BasicTextureCache::new(device.clone()));
        let mut viewport = std::mem::zeroed();
        let _hr = device.GetViewport(&mut viewport);
        let vx = viewport.X as f32 - 0.5;
        let vy = viewport.Y as f32 - 0.5;
        let viewport = [
            vx .. (vx + viewport.Width  as f32),
            vy .. (vy + viewport.Height as f32),
        ];
        Self { device, viewport, textures, resources }
    }

    pub unsafe fn draw(&mut self, texture: &StaticFile, instances: &[Instance]) {
        if instances.is_empty() { return } // Early out optimization

        // Common state

        let texture = self.textures.get_texture_2d_static_file(texture);

        let _hr = self.device.SetRenderState(D3DRS_LIGHTING, false.into());
        let _hr = self.device.SetIndices(self.resources.quads_ib.as_ptr());
        let _hr = self.device.SetVertexDeclaration(self.resources.sprite_vertex_vdecl.as_ptr());
        let _hr = self.device.SetTexture(0, texture.up_ref().as_ptr());
        let _hr = self.device.SetPixelShader(null_mut());
        let _hr = self.device.SetVertexShader(null_mut());

        // Instances

        let [view_x, view_y] = self.viewport.clone();
        let view_w = view_x.end - view_x.start;
        let view_h = view_y.end - view_y.start;
        let two_view_w = 2.0 / view_w;
        let two_view_h = 2.0 / view_h;

        // limit of shared quads_ib
        const MAX_QUADS_PER_DRAW : u16 = std::u16::MAX / 4;

        for instances in instances.chunks(MAX_QUADS_PER_DRAW.into()) {
            let verts = {
                let mut verts = Vec::new();

                for instance in instances.iter() {
                    let [ax, ay, az] = instance.anchor;

                    let [u, v] = instance.texcoords.clone();
                    let [x, y] = instance.dimensions.clone();
                    let (sin, cos) = instance.rotation.sin_cos();

                    for [x, y, u, v] in [
                        [x.start, y.start, u.start, v.start],
                        [x.end  , y.start, u.end  , v.start],
                        [x.end  , y.end  , u.end  , v.end  ],
                        [x.start, y.end  , u.start, v.end  ],
                    ].iter().copied() {
                        let [x, y] = [ax + x * cos - y * sin, ay + y * cos + x * sin];
                        let nx = (x - view_x.start) * two_view_w - 1.0;
                        let ny = 1.0 - (y - view_y.start) * two_view_h;
                        verts.push(SpriteVertex { position: [nx, ny, az, 1.0], texcoord: [u,v] });
                    }
                }
                self.device.create_vertex_buffer_from(D3DUSAGE_DYNAMIC, D3DPOOL_DEFAULT, &verts[..], "kakistocracy::windows::d3d9::sprite::SpriteRenderer::draw").unwrap()
            };

            let ninstances = instances.len() as UINT;
            let _hr = self.device.SetStreamSource(0, verts.as_ptr(), 0, SpriteVertex::stride());
            let _hr = self.device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, ninstances * 4, 0, ninstances * 2);
        }
    }
}

/// ### Safety
/// * ???
pub unsafe fn draw(device: &mcom::Rc<IDirect3DDevice9>, texture: &StaticFile, instances: &[Instance]) {
    SpriteRenderer::new(device).draw(texture, instances)
}



#[repr(C)]
#[derive(Clone)]
pub struct Instance {
    pub anchor:     [f32; 3],
    pub rotation:   f32,
    pub dimensions: [Range<f32>; 2],
    pub texcoords:  [Range<f32>; 2],
}



#[repr(C)]
#[derive(Clone, Copy)]
struct SpriteVertex {
    pub position: [f32; 4],
    pub texcoord: [f32; 2],
}

unsafe impl d3d9::Vertex for SpriteVertex {
    type Decl = &'static [D3DVERTEXELEMENT9];
    fn elements() -> Self::Decl { &[
        D3DVERTEXELEMENT9 { Stream: 0, Offset:  0, Method: D3DDECLMETHOD_DEFAULT as _, Type: D3DDECLTYPE_FLOAT4 as _, Usage: D3DDECLUSAGE_POSITION as _, UsageIndex: 0 },
        D3DVERTEXELEMENT9 { Stream: 0, Offset: 16, Method: D3DDECLMETHOD_DEFAULT as _, Type: D3DDECLTYPE_FLOAT2 as _, Usage: D3DDECLUSAGE_TEXCOORD as _, UsageIndex: 0 },
        D3DDECL_END,
    ][..] }
}



struct Resources {
    quads_ib:               mcom::Rc<IDirect3DIndexBuffer9>,
    sprite_vertex_vdecl:    mcom::Rc<IDirect3DVertexDeclaration9>,
}

impl Resources {
    fn new(device: &mcom::Rc<IDirect3DDevice9>) -> Self {
        let mut indicies = Vec::new();
        for quad in 0 ..= std::u16::MAX/4 {
            indicies.push(4 * quad + 0);
            indicies.push(4 * quad + 1);
            indicies.push(4 * quad + 2);

            indicies.push(4 * quad + 0);
            indicies.push(4 * quad + 2);
            indicies.push(4 * quad + 3);
        }
        let quads_ib            = unsafe { device.create_index_buffer_from(D3DUSAGE_DYNAMIC, D3DPOOL_DEFAULT, &indicies[..], "kakistocracy::windows::d3d9::sprite::Resources::quads_ib") }.unwrap();
        let sprite_vertex_vdecl = device.create_vertex_decl_from::<SpriteVertex>().unwrap();
        Self { quads_ib, sprite_vertex_vdecl }
    }
}
