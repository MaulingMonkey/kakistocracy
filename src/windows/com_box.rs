#![allow(dead_code)]

use super::Error;

use winapi::Interface;
use winapi::ctypes::c_void;
use winapi::shared::guiddef::{GUID, REFIID, IsEqualGUID};
use winapi::shared::minwindef::ULONG;
use winapi::shared::winerror::*;
use winapi::um::combaseapi::CoCreateGuid;
use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};

use std::any::*;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::*;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem::forget;
use std::ptr::*;
use std::sync::{Arc, Mutex};

// TODO:
// - [ ] move into mcom?
// - [ ] put to actual use



#[repr(C)] pub struct ComBox<T: Any> {
    vtable: *const IUnknownVtbl,
    value:  T,
}

impl<T: Any> ComBox<T> {
    pub fn new(value: T) -> mcom::Rc<Self> {
        let vtable : &'static IUnknownVtbl = &IUnknownVtbl {
            AddRef:         Self::add_ref,
            Release:        Self::release,
            QueryInterface: Self::query_interface,
        };
        let cb = Arc::new(Self {
            vtable: vtable,
            value,
        });
        let cb : *const Self = Arc::into_raw(cb);
        let cb : *mut   Self = cb as *mut _;
        unsafe { mcom::Rc::from_raw_unchecked(cb) }
    }

    pub fn as_inner(&self) -> &T { &self.value }

    unsafe extern "system" fn add_ref(this: *mut IUnknown) -> ULONG {
        // https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-addref
        let this = Arc::from_raw(this as *mut Self);
        let rc = Arc::strong_count(&this) + 1;
        forget(this.clone());
        forget(this);
        rc.try_into().unwrap()
    }

    unsafe extern "system" fn release(this: *mut IUnknown) -> ULONG {
        // https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-release
        let this = Arc::from_raw(this as *mut Self);
        let rc = Arc::strong_count(&this) - 1;
        drop(this);
        rc.try_into().unwrap()
    }

    unsafe extern "system" fn query_interface(this: *mut IUnknown, riid: REFIID, ppv_object: *mut *mut c_void) -> HRESULT {
        // https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-queryinterface(refiid_void)
        if riid.is_null() || ppv_object.is_null() {
            E_POINTER
        } else if IsEqualGUID(&*riid, &IUnknown::uuidof()) || IsEqualGUID(&*riid, &Self::uuidof()) {
            Self::add_ref(this);
            *ppv_object = this.cast();
            S_OK
        } else {
            *ppv_object = null_mut();
            E_NOINTERFACE
        }
    }
}

unsafe impl<T: Any + Send> Send for ComBox<T> {}
unsafe impl<T: Any + Sync> Sync for ComBox<T> {}

impl<T: Any> Interface  for ComBox<T> { fn uuidof() -> GUID { type_guid::<Self>() } }
impl<T: Any> AsRef<T>   for ComBox<T> { fn as_ref(&self) -> &T { &self.value } }
impl<T: Any> Borrow<T>  for ComBox<T> { fn borrow(&self) -> &T { &self.value } }
// XXX: Should this deref to T (convenient), or to IUnknown (consistent, might be required for future COM pointers)
//impl<T: Any> Deref      for ComBox<T> { fn deref(&self)  -> &IUnknown { unsafe { std::mem::transmute(self) } } type Target = IUnknown; }
//impl<T: Any> Deref      for ComBox<T> { fn deref(&self)  -> &T { &self.value } type Target = T; }

impl<T: Any + Debug  > Debug   for ComBox<T> { fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result { Debug  ::fmt(&self.value, fmt) } }
impl<T: Any + Display> Display for ComBox<T> { fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result { Display::fmt(&self.value, fmt) } }

// Not (yet?) implemented:
// - [Partial]Eq
// - [Partial]Ord
// - Hash
// - From/Into
// - Unpin, Pointer, UnwindSafe, CoerceUnsized, DispatchFromDyn

// Cannot implement:
// - Default for mcom::Rc<ComBox<T: Default>>

// Do not implement:
// - AsMut      (mutable access when COM objects are always refcounted / typically shared)
// - BorrowMut  (mutable access when COM objects are always refcounted / typically shared)
// - DerefMut   (mutable access when COM objects are always refcounted / typically shared)



lazy_static::lazy_static! { static ref TYPE_GUIDS : Mutex<HashMap<TypeId, GUID>> = Default::default(); }

fn type_guid<T: Any>() -> GUID { *TYPE_GUIDS.lock().unwrap().entry(TypeId::of::<T>()).or_insert_with(|| co_create_guid().unwrap()) }

fn co_create_guid() -> Result<GUID, Error> {
    let mut guid = unsafe { std::mem::zeroed() };
    let hr = unsafe { CoCreateGuid(&mut guid) };
    Error::check_hr("CoCreateGuid", hr, "")?;
    Ok(guid)
}
