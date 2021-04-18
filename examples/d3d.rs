#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))] fn main() {}

#[cfg(windows)] fn main() {
    use kakistocracy::windows::*;
    use kakistocracy::windows::winapi::shared::d3d9types::D3DCLEAR_TARGET;

    use std::ptr::*;



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
            if let Some(mwc9) = mwc9.lock(false) {
                for window in mwc9.windows.iter() {
                    let tc = d3d9::BasicTextureCache::get(&mwc9.device);
                    let _d3d_16x9 = tc.get_texture_2d_static_file(include_bytes!("d3d-16x9.png"));

                    unsafe { window.bind(&mwc9.device) }.unwrap();
                    let _hr = unsafe { mwc9.device.Clear(0, null(), D3DCLEAR_TARGET, 0xFF112233, 0.0, 0) };
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
