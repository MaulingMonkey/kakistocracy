#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))] fn main() {}

#[cfg(windows)] fn main() {
    use kakistocracy::windows::*;
    use kakistocracy::windows::winapi::shared::d3d9::*;
    use kakistocracy::windows::winapi::shared::d3d9types::*;

    use std::ptr::*;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vertex {
        pub position: [f32; 4],
        pub texcoord: [f32; 2],
    }

    unsafe impl d3d9::Vertex for Vertex {
        type Decl = &'static [D3DVERTEXELEMENT9];
        fn elements() -> Self::Decl { &[
            D3DVERTEXELEMENT9 { Stream: 0, Offset:  0, Method: D3DDECLMETHOD_DEFAULT as _, Type: D3DDECLTYPE_FLOAT4 as _, Usage: D3DDECLUSAGE_POSITION as _, UsageIndex: 0 },
            D3DVERTEXELEMENT9 { Stream: 0, Offset: 16, Method: D3DDECLMETHOD_DEFAULT as _, Type: D3DDECLTYPE_FLOAT2 as _, Usage: D3DDECLUSAGE_TEXCOORD as _, UsageIndex: 0 },
            D3DDECL_END,
        ][..] }
    }



    struct Data9 {
        quad_ib:    mcom::Rc<IDirect3DIndexBuffer9>,
        quad_vb:    mcom::Rc<IDirect3DVertexBuffer9>,
        quad_vdecl: mcom::Rc<IDirect3DVertexDeclaration9>,
    }

    impl From<mcom::Rc<IDirect3DDevice9>> for Data9 {
        fn from(device: mcom::Rc<IDirect3DDevice9>) -> Self {
            let quad_ib = unsafe { device.create_index_buffer_from(0, D3DPOOL_DEFAULT, &[0u16, 1, 2, 0, 2, 3]) }.unwrap();
            let (quad_vb, quad_vdecl) = unsafe { device.create_vertex_buffer_decl_from(0, D3DPOOL_DEFAULT, &[
                Vertex { position: [-0.5,  0.5, 0.0, 1.0], texcoord: [0.0, 0.0] },
                Vertex { position: [ 0.5,  0.5, 0.0, 1.0], texcoord: [1.0, 0.0] },
                Vertex { position: [ 0.5, -0.5, 0.0, 1.0], texcoord: [1.0, 1.0] },
                Vertex { position: [-0.5, -0.5, 0.0, 1.0], texcoord: [0.0, 1.0] },
            ])}.unwrap();
            Self { quad_ib, quad_vb, quad_vdecl }
        }
    }



    let mut mwc9 = d3d9::MultiWindowContext::new().unwrap();
    let mut mwc11 = d3d11::MultiWindowContext::new().unwrap();

    mwc9 .create_window_at("wide9",  [ 10.. 10+320,  10.. 10+230]).unwrap();
    mwc11.create_window_at("wide11", [330..330+320,  10.. 10+230]).unwrap();
    mwc9 .create_window_at("tall9",  [ 10.. 10+240, 250..250+320]).unwrap();
    mwc11.create_window_at("tall11", [250..250+240, 250..250+320]).unwrap();
    let _ = mwc9 .create_fullscreen_window(2, "fullscreen9");
    let _ = mwc11.create_fullscreen_window(3, "fullscreen11");

    let mut frames : u32 = 60 * 5;
    message::each_frame(move |_| {
        frames -= 1;
        if frames == 0 {
            message::post_quit(0);
            return false;
        } else {
            if let (Some(data), Some(mwc9)) = (mwc9.per_device::<Data9>(), mwc9.lock(false)) {
                for window in mwc9.windows.iter() {
                    let tc = d3d9::BasicTextureCache::get(&mwc9.device);
                    let _d3d_16x9 = tc.get_texture_2d_static_file(include_bytes!("d3d-16x9.png"));

                    unsafe { window.bind(&mwc9.device) }.unwrap();
                    let _hr = unsafe { mwc9.device.Clear(0, null(), D3DCLEAR_TARGET, 0xFF112233, 0.0, 0) };
                    let _hr = unsafe { mwc9.device.BeginScene() };
                    let _hr = unsafe { mwc9.device.SetRenderState(D3DRS_LIGHTING, false.into()) };
                    let _hr = unsafe { mwc9.device.SetIndices(data.quad_ib.as_ptr()) };
                    let _hr = unsafe { mwc9.device.SetVertexDeclaration(data.quad_vdecl.as_ptr()) };
                    let _hr = unsafe { mwc9.device.SetStreamSource(0, data.quad_vb.as_ptr(), 0, std::mem::size_of::<Vertex>() as u32) };
                    let _hr = unsafe { mwc9.device.SetTexture(0, _d3d_16x9.up_ref().as_ptr()) };
                    let _hr = unsafe { mwc9.device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, 4, 0, 2) };
                    let _hr = unsafe { mwc9.device.EndScene() };
                    let _hr = unsafe { window.swap_chain.Present(null(), null(), null_mut(), null(), 0) };
                    // XXX: error checking?
                }
            }

            if let Some(mwc11) = mwc11.lock(false) {
                for window in mwc11.windows.iter() {
                    unsafe { window.bind(&mwc11.immediate_context) }.unwrap();
                    unsafe { mwc11.immediate_context.ClearRenderTargetView(window.rtv.as_ptr(), &[0.3, 0.2, 0.1, 1.0]) };
                    unsafe { window.swap_chain.Present(1, 0) };
                    // XXX: error checking?
                }
            }

            // XXX: if different windows have different refresh rates, should D3DPRESENT_DONOTWAIT be attempted first?
        }
        true
    });
    message::loop_until_wm_quit();
}
