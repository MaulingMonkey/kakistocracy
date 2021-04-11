use crate::windows::*;

use winapi::shared::d3d9::*;
use winapi::shared::d3d9caps::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;

use std::ptr::null_mut;



/// Create a [`IDirect3DDevice9`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3ddevice9) via [`IDirect3D9::CreateDevice`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3d9-createdevice)
///
/// ### Safety
///
/// * Using the resulting Device after `window` is destroyed might be UB
pub unsafe fn create_device_windowed(d3d: &mcom::Rc<IDirect3D9>, window: &Window) -> Result<mcom::Rc<IDirect3DDevice9>, Error> {
    let hwnd = window.hwnd().ok_or_else(|| Error::new("d3d9::create_device_windowed", "", 0, "window is not alive"))?;
    let mut pp = default_windowed_presentation_parameters(hwnd);
    let mut device = null_mut();
    let hr = d3d.CreateDevice(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL, hwnd, DEFAULT_BEHAVIOR_FLAGS, &mut pp, &mut device);
    mcom::Rc::from_raw_opt(device).ok_or(Error::new_hr("IDirect3D9::CreateDevice", hr, "IDirect3DDevice9 is null"))
}

/// Create a [`IDirect3DDevice9Ex`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nn-d3d9-idirect3ddevice9ex) via [`IDirect3D9Ex::CreateDeviceEx`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3d9ex-createdeviceex)
///
/// ### Safety
///
/// * Using the resulting Device after `window` is destroyed might be UB
pub unsafe fn create_device_ex_windowed(d3d: &mcom::Rc<IDirect3D9Ex>, window: &Window) -> Result<mcom::Rc<IDirect3DDevice9Ex>, Error> {
    let hwnd = window.hwnd().ok_or_else(|| Error::new("d3d9::create_device_ex_windowed", "", 0, "window is not alive"))?;
    let mut pp = default_windowed_presentation_parameters(hwnd);
    let mut device = null_mut();
    let hr = d3d.CreateDeviceEx(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL, hwnd, DEFAULT_BEHAVIOR_FLAGS, &mut pp, null_mut(), &mut device);
    mcom::Rc::from_raw_opt(device).ok_or(Error::new_hr("IDirect3D9Ex::CreateDeviceEx", hr, "IDirect3DDevice9Ex is null"))
}



/// https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3dcreate
const DEFAULT_BEHAVIOR_FLAGS : DWORD =
    // D3DCREATE_DISABLE_PRINTSCREEN | // d3d9ex only
    D3DCREATE_FPU_PRESERVE |
    D3DCREATE_HARDWARE_VERTEXPROCESSING |
    D3DCREATE_NOWINDOWCHANGES |
    // D3DCREATE_PUREDEVICE | // many Get* functions still work even with this set
    0;


/// https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3dpresentflag
const DEFAULT_PRESENT_FLAGS : DWORD =
    // D3DPRESENTFLAG_DISCARD_DEPTHSTENCIL |
    // D3DPRESENTFLAG_NOAUTOROTATE |
    // D3DPRESENTFLAG_RESTRICTED_CONTENT |
    0;

/// https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3dpresent-parameters
unsafe fn default_windowed_presentation_parameters(hwnd: HWND) -> D3DPRESENT_PARAMETERS {
    D3DPRESENT_PARAMETERS {
        BackBufferWidth:            0,
        BackBufferHeight:           0,
        BackBufferFormat:           0,
        BackBufferCount:            0,
        MultiSampleType:            0,
        MultiSampleQuality:         0,
        SwapEffect:                 D3DSWAPEFFECT_DISCARD,
        hDeviceWindow:              hwnd,
        Windowed:                   TRUE,
        EnableAutoDepthStencil:     FALSE,
        AutoDepthStencilFormat:     0,
        Flags:                      DEFAULT_PRESENT_FLAGS,
        FullScreen_RefreshRateInHz: 0,
        PresentationInterval:       D3DPRESENT_INTERVAL_DEFAULT,
    }
}



#[test] fn test_create_device() {
    {
        let win = Window::create_stub("test_create_device 1");
        let d3d = d3d9::create_d3d(true).unwrap();
        let device = unsafe { d3d9::create_device_windowed(&d3d, &win) }.unwrap();
        drop(device);
        drop(d3d);
        let _ = win.destroy();
    }
    {
        let win = Window::create_stub("test_create_device 2");
        let d3d = d3d9::create_d3d(true).unwrap();
        let device = unsafe { d3d9::create_device_windowed(&d3d, &win) }.unwrap();
        drop(device);
        drop(d3d);
        let _ = win.destroy();
    }
}

#[test] fn test_create_device_ex() {
    {
        let win = Window::create_stub("test_create_device_ex 1");
        let d3dex = d3d9::create_d3d_ex(true).unwrap();
        let deviceex = unsafe { d3d9::create_device_ex_windowed(&d3dex, &win) }.unwrap();
        drop(deviceex);
        drop(d3dex);
        let _ = win.destroy();
    }
    {
        let win = Window::create_stub("test_create_device_ex 2");
        let d3dex = d3d9::create_d3d_ex(true).unwrap();
        let deviceex = unsafe { d3d9::create_device_ex_windowed(&d3dex, &win) }.unwrap();
        drop(deviceex);
        drop(d3dex);
        let _ = win.destroy();
    }
}