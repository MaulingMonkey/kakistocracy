use crate::windows::Error;

use winapi::Interface;
use winapi::shared::dxgi::*;
use winapi::shared::winerror::SUCCEEDED;

use std::ptr::null_mut;



pub trait DxgiDeviceExt {
    fn get_parent<P: Interface>(&self) -> Result<mcom::Rc<P>, Error>;
    fn get_parent_dxgi_adapter(&self) -> Result<mcom::Rc<IDXGIAdapter>, Error>;
}

pub trait DxgiAdapterExt {
    fn get_parent<P: Interface>(&self) -> Result<mcom::Rc<P>, Error>;
    fn get_parent_dxgi_factory(&self) -> Result<mcom::Rc<IDXGIFactory>, Error>;
}

pub trait DxgiSwapChainExt {
    fn get_buffer<B: Interface>(&self, index: u32) -> Result<mcom::Rc<B>, Error>;
    fn get_desc(&self) -> Result<DXGI_SWAP_CHAIN_DESC, Error>;
}



impl DxgiDeviceExt for mcom::Rc<IDXGIDevice> {
    fn get_parent<P: Interface>(&self) -> Result<mcom::Rc<P>, Error> {
        let mut parent = null_mut();
        let hr = unsafe { self.GetParent(&P::uuidof(), &mut parent) };
        let parent = unsafe { mcom::Rc::from_raw_opt(parent as *mut P) };
        parent.ok_or(Error::new_hr("IDXGIDevice::GetParent", hr, "parent is null"))
    }

    fn get_parent_dxgi_adapter(&self) -> Result<mcom::Rc<IDXGIAdapter>, Error> { self.get_parent() }
}

impl DxgiAdapterExt for mcom::Rc<IDXGIAdapter> {
    fn get_parent<P: Interface>(&self) -> Result<mcom::Rc<P>, Error> {
        let mut parent = null_mut();
        let hr = unsafe { self.GetParent(&P::uuidof(), &mut parent) };
        let parent = unsafe { mcom::Rc::from_raw_opt(parent as *mut P) };
        parent.ok_or(Error::new_hr("IDXGIAdapter::GetParent", hr, "parent is null"))
    }

    fn get_parent_dxgi_factory(&self) -> Result<mcom::Rc<IDXGIFactory>, Error> { self.get_parent() }
}

impl DxgiSwapChainExt for mcom::Rc<IDXGISwapChain> {
    fn get_buffer<B: Interface>(&self, index: u32) -> Result<mcom::Rc<B>, Error> {
        let mut bb = null_mut();
        let hr = unsafe { self.GetBuffer(index, &B::uuidof(), &mut bb) };
        let bb = unsafe { mcom::Rc::from_raw_opt(bb as *mut B) };
        bb.ok_or(Error::new_hr("IDXGISwapChain::GetBuffer", hr, "buffer is null"))
    }

    fn get_desc(&self) -> Result<DXGI_SWAP_CHAIN_DESC, Error> {
        let mut desc = unsafe { std::mem::zeroed() };
        let hr = unsafe { self.GetDesc(&mut desc) };
        if SUCCEEDED(hr) {
            Ok(desc)
        } else {
            Err(Error::new_hr("IDXGISwapChain::GetDesc", hr, ""))
        }
    }
}
