//! Sprite rendering utilities

pub use crate::sprite::*;

use crate::*;
use crate::io::StaticFile;
use crate::windows::*;
use crate::windows::d3d11::{BasicTextureCache, Vertex};

use winapi::shared::dxgiformat::*;
use winapi::shared::minwindef::UINT;
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;

use std::convert::*;
use std::ops::Range;
use std::ptr::*;



/// Render `instances` of `texture` to `context`
///
/// ### Safety
/// * `context`'s render target 0 is expected to be valid/bound
/// * `context`'s viewport 0 is expected to be valid/bound
pub unsafe fn draw(context: &mcom::Rc<ID3D11DeviceContext>, texture: &StaticFile, instances: &[Instance]) {
    SpriteRenderer::new(context).draw(texture, instances)
}



struct SpriteRenderer<'d> {
    context:    &'d mcom::Rc<ID3D11DeviceContext>,
    device:     mcom::Rc<ID3D11Device>,
    viewport:   [Range<f32>; 2],
    textures:   UnkWrapRc<BasicTextureCache>,
    resources:  UnkWrapRc<Resources>,
}

impl<'d> SpriteRenderer<'d> {
    pub unsafe fn new(context: &'d mcom::Rc<ID3D11DeviceContext>) -> Self {
        let device = context.get_device();
        let resources   = d3d11::device_private_data_get_or_insert(&device, || Resources::new(&device));
        let textures    = d3d11::device_private_data_get_or_insert(&device, || BasicTextureCache::new(device.clone()));
        let mut n_viewports = 1;
        let mut viewport = std::mem::zeroed();
        context.RSGetViewports(&mut n_viewports, &mut viewport);
        let vx = viewport.TopLeftX; // - 0.5;
        let vy = viewport.TopLeftY; // - 0.5;
        let viewport = [
            vx .. (vx + viewport.Width ),
            vy .. (vy + viewport.Height),
        ];
        Self { device, context, viewport, textures, resources }
    }

    pub unsafe fn draw(&mut self, texture: &StaticFile, instances: &[Instance]) {
        if instances.is_empty() { return } // Early out optimization

        // Common state

        let texture = self.textures.get_texture_2d_static_file(texture);
        let texture = self.device.create_shader_resource_view(texture.up_ref()).unwrap(); // XXX: Offload to BasicTextureCache?

        self.context.IASetIndexBuffer(self.resources.quads_ib.as_ptr(), DXGI_FORMAT_R16_UINT, 0);
        self.context.IASetInputLayout(self.resources.sprite_vertex_layout.as_ptr());
        self.context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        self.context.VSSetShader(self.resources.sprite_vertex_shader.as_ptr(), null(), 0);
        self.context.PSSetShader(self.resources.sprite_pixel_shader .as_ptr(), null(), 0);
        self.context.PSSetShaderResources(0, 1, [texture.as_ptr()].as_ptr());
        self.context.PSSetSamplers(0, 1, [self.resources.sampler_state.as_ptr()].as_ptr());

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
                        verts.push(sprite::Vertex { position: [nx, ny, az, 1.0], texcoord: [u,v] });
                    }
                }
                self.device.create_buffer_from(D3D11_USAGE_IMMUTABLE, D3D11_BIND_VERTEX_BUFFER, &verts[..], "kakistocracy::windows::d3d11::sprite::SpriteRenderer::draw").unwrap()
            };

            let ninstances = instances.len() as UINT;
            self.context.IASetVertexBuffers(0, 1, [verts.as_ptr()].as_ptr(), [sprite::Vertex::stride()].as_ptr(), [0].as_ptr());
            self.context.DrawIndexed(ninstances * 6, 0, 0);
        }
    }
}

unsafe impl d3d11::Vertex for sprite::Vertex {
    type Decl = [D3D11_INPUT_ELEMENT_DESC; 2];

    fn elements() -> Self::Decl {[
        D3D11_INPUT_ELEMENT_DESC { SemanticName: b"POSITION\0".as_ptr().cast(), SemanticIndex: 0, Format: DXGI_FORMAT_R32G32B32A32_FLOAT, InputSlot: 0, AlignedByteOffset:  0, InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA, InstanceDataStepRate: 0 },
        D3D11_INPUT_ELEMENT_DESC { SemanticName: b"TEXCOORD\0".as_ptr().cast(), SemanticIndex: 0, Format: DXGI_FORMAT_R32G32_FLOAT,       InputSlot: 0, AlignedByteOffset: 16, InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA, InstanceDataStepRate: 0 },
    ]}
}

struct Resources {
    quads_ib:               mcom::Rc<ID3D11Buffer>,
    sampler_state:          mcom::Rc<ID3D11SamplerState>,
    sprite_pixel_shader:    mcom::Rc<ID3D11PixelShader>,
    sprite_vertex_shader:   mcom::Rc<ID3D11VertexShader>,
    sprite_vertex_layout:   mcom::Rc<ID3D11InputLayout>,
}

impl Resources {
    fn new(device: &mcom::Rc<ID3D11Device>) -> Self {
        let indicies                = create_quads_index_data(std::u16::MAX/4);
        let quads_ib                = unsafe { device.create_buffer_from(D3D11_USAGE_IMMUTABLE, D3D11_BIND_INDEX_BUFFER, &indicies[..], "kakistocracy::windows::d3d9::sprite::Resources::quads_ib") }.unwrap();
        let sampler_state           = unsafe { device.create_sampler_state(&D3D11_SAMPLER_DESC { // https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ns-d3d11-d3d11_sampler_desc
            Filter:         D3D11_FILTER_MIN_MAG_MIP_POINT,
            AddressU:       D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressV:       D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressW:       D3D11_TEXTURE_ADDRESS_CLAMP,
            MipLODBias:     0.0,
            MaxAnisotropy:  0,
            ComparisonFunc: D3D11_COMPARISON_LESS_EQUAL, // ?
            BorderColor:    [0.0, 0.0, 0.0, 0.0],
            MinLOD:         0.0,
            MaxLOD:         D3D11_FLOAT32_MAX,
        }, "kakistocracy::windows::d3d9::sprite::Resources::sampler_state")}.unwrap();
        let sprite_pixel_shader     = unsafe { device.create_pixel_shader(&include_file!("sprite.bin.ps_4_0")) }.unwrap();
        let sprite_vertex_shader    = unsafe { device.create_vertex_shader(&include_file!("sprite.bin.vs_4_0")) }.unwrap();
        let sprite_vertex_layout    = unsafe { device.create_input_layout_from::<sprite::Vertex>(include_file!("sprite.bin.vs_4_0").as_bytes()) }.unwrap();
        Self { quads_ib, sampler_state, sprite_pixel_shader, sprite_vertex_shader, sprite_vertex_layout }
    }
}
