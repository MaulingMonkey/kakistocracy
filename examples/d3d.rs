#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))] fn main() {}

#[cfg(windows)] fn main() {
    use kakistocracy::*;
    use kakistocracy::windows::*;
    use kakistocracy::windows::d3d9::sprite;
    use kakistocracy::windows::winapi::shared::d3d9types::*;
    use kakistocracy::windows::winapi::shared::minwindef::*;
    use kakistocracy::windows::winapi::shared::windef::*;
    use kakistocracy::windows::winapi::um::winuser::*;

    use instant::*;

    use std::ptr::*;

    #[derive(Clone)]
    struct Context {
        start: Instant,
    }

    impl Default for Context {
        fn default() -> Self { Self { start: Instant::now() } }
    }

    impl kakistocracy::windows::message::Handler for Context {
        unsafe fn wndproc(&self, hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
            match msg {
                WM_CLOSE    => { message::post_quit(0); 0 }, // quit if *any* window is closed
                _other      => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    impl kakistocracy::windows::d3d9::Render for Context {
        fn render(&self, ctx: &d3d9::RenderArgs) {
            let (cw, ch) = ctx.client_size();
            let (cw, ch) = (cw as f32, ch as f32);
            let dev = &ctx.device;
            let rot = (Instant::now() - self.start).as_secs_f32();

            let instances = [
                sprite::Instance { anchor: [     10.0,      10.0, 0.0], rotation: 0.0, dimensions: [  0.0 .. 16.0,  0.0 .. 9.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] },
                sprite::Instance { anchor: [cw - 10.0,      10.0, 0.0], rotation: 0.0, dimensions: [-16.0 ..  0.0,  0.0 .. 9.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] },
                sprite::Instance { anchor: [cw - 10.0, ch - 10.0, 0.0], rotation: 0.0, dimensions: [-16.0 ..  0.0, -9.0 .. 0.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] },
                sprite::Instance { anchor: [     10.0, ch - 10.0, 0.0], rotation: 0.0, dimensions: [  0.0 .. 16.0, -9.0 .. 0.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] },
                sprite::Instance { anchor: [cw / 2.0 , ch / 2.0 , 0.0], rotation: rot, dimensions: [-16.0 .. 16.0, -9.0 .. 9.0], texcoords: [0.0 .. 1.0, 0.0 .. 1.0] }, // 2x size
            ];

            ctx.bind().unwrap();
            let _hr = unsafe { dev.Clear(0, null(), D3DCLEAR_TARGET, 0xFF112233, 0.0, 0) };
            let _hr = unsafe { dev.BeginScene() };
            unsafe { d3d9::sprite::draw(dev, &include_file!("d3d-16x9.png"), &instances[..]) };
            let _hr = unsafe { dev.EndScene() };
            let _hr = unsafe { ctx.swap_chain.Present(null(), null(), null_mut(), null(), 0) };
            // XXX: error checking?
        }
    }

    impl kakistocracy::windows::d3d11::Render for Context {
        fn render(&self, ctx: &d3d11::RenderArgs) {
            ctx.bind_immediate_context().unwrap();
            unsafe { ctx.immediate_context.ClearRenderTargetView(ctx.rtv.as_ptr(), &[0.3, 0.2, 0.1, 1.0]) };
            unsafe { ctx.swap_chain.Present(1, 0) };
        }
    }

    let mut mwc9 = d3d9::MultiWindowContext::new().unwrap();
    let mut mwc11 = d3d11::MultiWindowContext::new().unwrap();

    let ctx = Context::default();
    mwc9 .create_window_at("wide9",  [ 10.. 10+320,  10.. 10+230], ctx.clone()).unwrap();
    mwc11.create_window_at("wide11", [330..330+320,  10.. 10+230], ctx.clone()).unwrap();
    mwc9 .create_window_at("tall9",  [ 10.. 10+240, 250..250+320], ctx.clone()).unwrap();
    mwc11.create_window_at("tall11", [250..250+240, 250..250+320], ctx.clone()).unwrap();
    mwc9 .create_fullscreen_window(monitor::ByOrderX( 1), "fullscreen9",  ctx.clone()).unwrap();
    mwc11.create_fullscreen_window(monitor::ByOrderX( 3), "fullscreen11", ctx.clone()).unwrap();

    message::each_frame(move |_| {
        mwc9 .render_visible_windows();
        mwc11.render_visible_windows();
        // XXX: if different windows have different refresh rates, should D3DPRESENT_DONOTWAIT be attempted first?
        true
    });

    message::loop_until_wm_quit();
}
