#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))] fn main() {}

#[cfg(windows)] fn main() {
    use kakistocracy::windows::*;
    use kakistocracy::windows::winapi::shared::d3d9types::D3DCLEAR_TARGET;

    use std::ptr::*;



    let mut mwc = d3d9::MultiWindowContext::new().unwrap();
    mwc.create_window_at("wide", [10..10+320,  10.. 10+230]).unwrap();
    mwc.create_window_at("tall", [10..10+240, 250..250+320]).unwrap();
    mwc.create_fullscreen_window(2, "fullscreen").unwrap();

    let mut frames = 60 * 5;
    messages::each_frame(move |_| {
        frames -= 1;
        if frames == 0 {
            messages::post_quit(0);
        } else if let Some(mwc) = mwc.lock(false) {
            for window in mwc.windows.iter() {
                unsafe { window.bind(&mwc.device) }.unwrap();
                let _hr = unsafe { mwc.device.Clear(0, null(), D3DCLEAR_TARGET, 0xFF112233, 0.0, 0) };
                let _hr = unsafe { window.swap_chain.Present(null(), null(), null_mut(), null(), 0) };
                // XXX: error checking?
                // XXX: if different windows have different refresh rates, should D3DPRESENT_DONOTWAIT be attempted first?
            }
        }
        true
    });
    messages::loop_until_wm_quit();
}
