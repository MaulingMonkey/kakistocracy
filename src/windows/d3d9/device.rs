use crate::windows::*;
use crate::windows::d3d9::D3DERR_NOTFOUND;

use winapi::shared::d3d9::*;
use winapi::shared::d3d9caps::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::unknwnbase::IUnknown;

use std::any::Any;
use std::marker::PhantomData;
use std::ops::*;
use std::ptr::null_mut;



/// Create a [`IDirect3DDevice9`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9helper/nn-d3d9helper-idirect3ddevice9) via [`IDirect3D9::CreateDevice`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3d9-createdevice)
///
/// ### Safety
///
/// * Using the resulting Device after `window` is destroyed might be UB
/// * This method [should not be run during the handling of `WM_CREATE`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3d9-createdevice#remarks).
/// * Any call to create, release, or reset the device must be done using the same thread as the window procedure of the focus window.
pub(crate) unsafe fn create_device_windowed(d3d: &mcom::Rc<IDirect3D9>, window: &Window) -> Result<mcom::Rc<IDirect3DDevice9>, Error> {
    let hwnd = window.hwnd().ok_or_else(|| Error::new("d3d9::create_device_windowed", "", 0, "window is not alive"))?;
    let mut pp = default_windowed_presentation_parameters(hwnd);
    let mut device = null_mut();
    let hr = d3d.CreateDevice(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL, null_mut(), DEFAULT_BEHAVIOR_FLAGS, &mut pp, &mut device);
    mcom::Rc::from_raw_opt(device).ok_or(Error::new_hr("IDirect3D9::CreateDevice", hr, "IDirect3DDevice9 is null"))
}

/// Create a [`IDirect3DDevice9Ex`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nn-d3d9-idirect3ddevice9ex) via [`IDirect3D9Ex::CreateDeviceEx`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3d9ex-createdeviceex)
///
/// ### Safety
///
/// * Using the resulting Device after `window` is destroyed might be UB
/// * This method [should not be run during the handling of `WM_CREATE`](https://docs.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3d9-createdevice#remarks).
/// * Any call to create, release, or reset the device must be done using the same thread as the window procedure of the focus window.
#[allow(dead_code)]
pub(crate) unsafe fn create_device_ex_windowed(d3d: &mcom::Rc<IDirect3D9Ex>, window: &Window) -> Result<mcom::Rc<IDirect3DDevice9Ex>, Error> {
    let hwnd = window.hwnd().ok_or_else(|| Error::new("d3d9::create_device_ex_windowed", "", 0, "window is not alive"))?;
    let mut pp = default_windowed_presentation_parameters(hwnd);
    let mut device = null_mut();
    let hr = d3d.CreateDeviceEx(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL, null_mut(), DEFAULT_BEHAVIOR_FLAGS, &mut pp, null_mut(), &mut device);
    mcom::Rc::from_raw_opt(device).ok_or(Error::new_hr("IDirect3D9Ex::CreateDeviceEx", hr, "IDirect3DDevice9Ex is null"))
}

pub(crate) fn device_private_data_get_or_insert<T: Any>(device: &mcom::Rc<IDirect3DDevice9>, f: impl FnOnce() -> T) -> UnkWrapRc<T> {
    struct DevicePrivateData<T: Any>(PhantomData<T>);
    let pdguid = type_guid::<DevicePrivateData::<T>>();
    let bb = unsafe { device.get_back_buffer(0, 0) }.unwrap();
    match unsafe { bb.get_private_data_com::<IUnknown>(&pdguid) } {
        Ok(btc) => UnkWrapRc::from_com_unknown(&btc).unwrap(),
        Err(err) if err.hresult() == D3DERR_NOTFOUND => {
            let btc = UnkWrapRc::new(f());
            bb.set_private_data_com(&pdguid, &btc.to_com_unknown()).unwrap();
            btc
        },
        Err(err) => panic!("{}", err),
    }
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
pub(crate) unsafe fn default_windowed_presentation_parameters(hwnd: HWND) -> D3DPRESENT_PARAMETERS {
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
        let win = OwnedWindow::create_stub("test_create_device 1");
        let d3d = d3d9::create_d3d(true).unwrap();
        let _device = unsafe { d3d9::create_device_windowed(&d3d, &win) }.unwrap();
    }
    {
        let win = OwnedWindow::create_stub("test_create_device 2");
        let d3d = d3d9::create_d3d(true).unwrap();
        let _device = unsafe { d3d9::create_device_windowed(&d3d, &win) }.unwrap();
    }
}

#[test] fn test_create_device_ex() {
    {
        let win = OwnedWindow::create_stub("test_create_device_ex 1");
        let d3dex = d3d9::create_d3d_ex(true).unwrap();
        let _deviceex = unsafe { d3d9::create_device_ex_windowed(&d3dex, &win) }.unwrap();
    }
    {
        let win = OwnedWindow::create_stub("test_create_device_ex 2");
        let d3dex = d3d9::create_d3d_ex(true).unwrap();
        let _deviceex = unsafe { d3d9::create_device_ex_windowed(&d3dex, &win) }.unwrap();
    }
}
