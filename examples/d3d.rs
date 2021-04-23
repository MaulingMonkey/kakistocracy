#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))] fn main() {}

#[cfg(windows)] fn main() {
    use kakistocracy::*;
    use kakistocracy::windows::*;
    use kakistocracy::windows::d3d9::sprite;
    use kakistocracy::windows::winapi::shared::d3d9types::*;

    use instant::*;

    use std::ptr::*;



    let mut mwc9 = d3d9::MultiWindowContext::new().unwrap();
    let mut mwc11 = d3d11::MultiWindowContext::new().unwrap();

    mwc9 .create_window_at("wide9",  [ 10.. 10+320,  10.. 10+230]).unwrap();
    mwc11.create_window_at("wide11", [330..330+320,  10.. 10+230]).unwrap();
    mwc9 .create_window_at("tall9",  [ 10.. 10+240, 250..250+320]).unwrap();
    mwc11.create_window_at("tall11", [250..250+240, 250..250+320]).unwrap();
    let _ = mwc9 .create_fullscreen_window(2, "fullscreen9");
    let _ = mwc11.create_fullscreen_window(3, "fullscreen11");

    let start = Instant::now();

    message::each_frame(move |_| {
        if !any_current_thread_windows() {
            message::post_quit(0);
            return false;
        }

        let now = Instant::now();
        let rot = (now - start).as_secs_f32();

        if let Some(mwc9) = mwc9.lock(false) {
            let dev = &mwc9.device;

            for window in mwc9.windows.iter() {
                let (cw, ch) = window.client_size();
                let (cw, ch) = (cw as f32, ch as f32);

                let instances = [
                    sprite::Instance { anchor: [     10.0,      10.0, 0.0], rotation: 0.0, dimensions: [  0.0 .. 16.0,  0.0 .. 9.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] },
                    sprite::Instance { anchor: [cw - 10.0,      10.0, 0.0], rotation: 0.0, dimensions: [-16.0 ..  0.0,  0.0 .. 9.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] },
                    sprite::Instance { anchor: [cw - 10.0, ch - 10.0, 0.0], rotation: 0.0, dimensions: [-16.0 ..  0.0, -9.0 .. 0.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] },
                    sprite::Instance { anchor: [     10.0, ch - 10.0, 0.0], rotation: 0.0, dimensions: [  0.0 .. 16.0, -9.0 .. 0.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] },
                    sprite::Instance { anchor: [cw / 2.0 , ch / 2.0 , 0.0], rotation: rot, dimensions: [-16.0 .. 16.0, -9.0 .. 9.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] }, // 2x size
                ];

                window.bind().unwrap();
                let _hr = unsafe { dev.Clear(0, null(), D3DCLEAR_TARGET, 0xFF112233, 0.0, 0) };
                let _hr = unsafe { dev.BeginScene() };
                unsafe { d3d9::sprite::draw(dev, &include_file!("d3d-16x9.png"), &instances[..]) };
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
