use crate::windows::*;

use winapi::shared::winerror::*;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::d3d11::*;

use std::any::Any;
use std::marker::PhantomData;
use std::ops::*;
use std::ptr::null_mut;



pub(crate) fn device_private_data_get_or_insert<T: Any>(device: &mcom::Rc<ID3D11Device>, f: impl FnOnce() -> T) -> UnkWrapRc<T> {
    struct DevicePrivateData<T: Any>(PhantomData<T>);
    let pdguid = type_guid::<DevicePrivateData::<T>>();

    let mut unknown : *mut IUnknown = null_mut();
    let mut size = std::mem::size_of_val(&unknown) as _;
    let hr = unsafe { device.GetPrivateData(&pdguid, &mut size, (&mut unknown as *mut *mut IUnknown).cast()) };
    if SUCCEEDED(hr) {
        assert_eq!(size, std::mem::size_of_val(&unknown) as _, "ID3D11Device::GetPrivateData returned the wrong amount of data for an IUnknown pointer, UB likely");
        let unknown = unsafe { mcom::Rc::borrow_ptr(&unknown) };
        UnkWrapRc::from_com_unknown(unknown).unwrap()
    } else if hr == DXGI_ERROR_NOT_FOUND {
        let btc = UnkWrapRc::new(f());
        let hr = unsafe { device.SetPrivateDataInterface(&pdguid, btc.to_com_unknown().as_ptr()) };
        assert!(SUCCEEDED(hr), "ID3D11Device::SetPrivateDataInterface failed");
        btc
    } else {
        panic!("ID3D11Device::GetPrivateData failed unexpectedly with HRESULT == 0x{:08x}", hr as u32);
    }
}
