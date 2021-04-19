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
        corner_quads_ib:    mcom::Rc<IDirect3DIndexBuffer9>,
    }

    impl From<mcom::Rc<IDirect3DDevice9>> for Data9 {
        fn from(device: mcom::Rc<IDirect3DDevice9>) -> Self {
            let corner_quads_ib = unsafe { device.create_index_buffer_from::<u16>(0, D3DPOOL_DEFAULT, &[
                 0, 1, 2, 0, 2, 3,
                 4, 5, 6, 4, 6, 7,
                 8, 9,10, 8,10,11,
                12,13,14,12,14,15,
            ])}.unwrap();
            Self { corner_quads_ib }
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

    message::each_frame(move |_| {
        if !any_current_thread_windows() {
            message::post_quit(0);
            return false;
        }
        if let (Some(data), Some(mwc9)) = (mwc9.per_device::<Data9>(), mwc9.lock(false)) {
            let dev = &mwc9.device;
            let tc = d3d9::BasicTextureCache::get(dev);
            let d3d_logo_16x9 = tc.get_texture_2d_static_file(include_bytes!("d3d-16x9.png"));

            for window in mwc9.windows.iter() {
                let (cw, ch) = window.client_size();
                let (sx, sy) = (2.0 / (cw as f32), -2.0 / (ch as f32));
                let pos = |x: u32, y: u32| [((x as f32 - 0.5) * sx) - 1.0, ((y as f32 - 0.5) * sy) + 1.0, 0.0, 1.0];

                let (corner_quads_vb, corner_quads_vdecl) = unsafe { dev.create_vertex_buffer_decl_from(0, D3DPOOL_DEFAULT, &[
                    Vertex { position: pos(10 +  0, 10 + 0), texcoord: [0.0, 0.0] },
                    Vertex { position: pos(10 + 16, 10 + 0), texcoord: [1.0, 0.0] },
                    Vertex { position: pos(10 + 16, 10 + 9), texcoord: [1.0, 1.0] },
                    Vertex { position: pos(10 +  0, 10 + 9), texcoord: [0.0, 1.0] },

                    Vertex { position: pos(cw - 26 +  0, 10 + 0), texcoord: [0.0, 0.0] },
                    Vertex { position: pos(cw - 26 + 16, 10 + 0), texcoord: [1.0, 0.0] },
                    Vertex { position: pos(cw - 26 + 16, 10 + 9), texcoord: [1.0, 1.0] },
                    Vertex { position: pos(cw - 26 +  0, 10 + 9), texcoord: [0.0, 1.0] },

                    Vertex { position: pos(cw - 26 +  0, ch - 19 + 0), texcoord: [0.0, 0.0] },
                    Vertex { position: pos(cw - 26 + 16, ch - 19 + 0), texcoord: [1.0, 0.0] },
                    Vertex { position: pos(cw - 26 + 16, ch - 19 + 9), texcoord: [1.0, 1.0] },
                    Vertex { position: pos(cw - 26 +  0, ch - 19 + 9), texcoord: [0.0, 1.0] },

                    Vertex { position: pos(10 +  0, ch - 19 + 0), texcoord: [0.0, 0.0] },
                    Vertex { position: pos(10 + 16, ch - 19 + 0), texcoord: [1.0, 0.0] },
                    Vertex { position: pos(10 + 16, ch - 19 + 9), texcoord: [1.0, 1.0] },
                    Vertex { position: pos(10 +  0, ch - 19 + 9), texcoord: [0.0, 1.0] },
                ])}.unwrap();

                window.bind().unwrap();
                let _hr = unsafe { dev.Clear(0, null(), D3DCLEAR_TARGET, 0xFF112233, 0.0, 0) };
                let _hr = unsafe { dev.BeginScene() };
                let _hr = unsafe { dev.SetRenderState(D3DRS_LIGHTING, false.into()) };
                let _hr = unsafe { dev.SetIndices(data.corner_quads_ib.as_ptr()) };
                let _hr = unsafe { dev.SetVertexDeclaration(corner_quads_vdecl.as_ptr()) };
                let _hr = unsafe { dev.SetStreamSource(0, corner_quads_vb.as_ptr(), 0, std::mem::size_of::<Vertex>() as u32) };
                let _hr = unsafe { dev.SetTexture(0, d3d_logo_16x9.up_ref().as_ptr()) };
                let _hr = unsafe { dev.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, 4*4, 0, 4*2) };
                let _hr = unsafe { dev.EndScene() };
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
        true
    });
    message::loop_until_wm_quit();
}
