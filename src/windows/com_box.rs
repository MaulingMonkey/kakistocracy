#![allow(dead_code)] // XXX

use crate::windows::{Error, type_guid};

use winapi::Interface;
use winapi::ctypes::c_void;
use winapi::shared::guiddef::{GUID, REFIID, IsEqualGUID};
use winapi::shared::minwindef::ULONG;
use winapi::shared::winerror::*;
use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};

use std::any::*;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::convert::*;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::mem::forget;
use std::ops::*;
use std::ptr::*;
use std::sync::{Arc, Weak};

// TODO:
// - [ ] move into mcom?
// - [ ] put to actual use



pub(crate) struct UnkWrapRc  <T: Any>(Arc <UnkWrap<T>>);
pub(crate) struct UnkWrapWeak<T: Any>(Weak<UnkWrap<T>>);

impl<T: Any> UnkWrapRc<T> {
    pub fn new(value: T) -> Self { Self(Arc::new(UnkWrap::new(value))) }
    // try_unwrap
    // into_raw
    // as_ptr
    // from_raw
    pub fn downgrade(this: &Self) -> UnkWrapWeak<T> { UnkWrapWeak(Arc::downgrade(&this.0)) }
    pub fn weak_count(this: &Self) -> usize { Arc::weak_count(&this.0) }
    pub fn strong_count(this: &Self) -> usize { Arc::strong_count(&this.0) }
    // increment_strong_count
    // decrement_strong_count
    pub fn ptr_eq(this: &Self, other: &Self) -> bool { Arc::ptr_eq(&this.0, &other.0) }
    // make_mut
    // get_mut
    // get_mut_unchecked
    // downcast
}

impl<T: Any> UnkWrapWeak<T> {
    pub fn new() -> Self { Self(Weak::new()) }
    // as_ptr
    // into_raw
    // from_raw
    pub fn upgrade(&self) -> Option<UnkWrapRc<T>> { self.0.upgrade().map(|wrap| UnkWrapRc(wrap)) }
    pub fn strong_count(&self) -> usize { self.0.strong_count() }
    pub fn weak_count(&self) -> usize { self.0.weak_count() }
    pub fn ptr_eq(&self, other: &Self) -> bool { self.0.ptr_eq(&other.0) }
}

unsafe impl<T: Any + Send> Send for UnkWrapRc<T> {}
unsafe impl<T: Any + Sync> Sync for UnkWrapRc<T> {}
unsafe impl<T: Any + Send> Send for UnkWrapWeak<T> {}
unsafe impl<T: Any + Sync> Sync for UnkWrapWeak<T> {}

impl<T: Any> UnkWrapRc<T> {
    pub(crate) fn from_com_unknown(unk: &mcom::Rc<IUnknown>) -> Option<Self> { Self::try_from(unk).ok() }
    pub(crate) fn to_com_unknown(&self) -> mcom::Rc<IUnknown> { mcom::Rc::<IUnknown>::from(self.clone()) }
}

impl<T: Any> UnkWrapWeak<T> {
    // conversion?
}

impl<T: Any> AsRef<T>   for UnkWrapRc<T> { fn as_ref(&self) -> &T { &self.0.as_inner() } }
impl<T: Any> Borrow<T>  for UnkWrapRc<T> { fn borrow(&self) -> &T { &self.0.as_inner() } }
impl<T: Any> Deref      for UnkWrapRc<T> { fn deref(&self)  -> &T { &self.0.as_inner() } type Target = T; }
impl<T: Any> Clone      for UnkWrapRc<T> { fn clone(&self) -> Self { Self(self.0.clone()) } }

impl<T: Any> Clone      for UnkWrapWeak<T> { fn clone(&self) -> Self { Self(self.0.clone()) } }

impl<T: Any + Default   > Default       for UnkWrapRc<T> { fn default() -> Self { Self::new(T::default()) } }
impl<T: Any + Debug     > Debug         for UnkWrapRc<T> { fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result { Debug  ::fmt(&self.0.as_inner(), fmt) } }
impl<T: Any + Display   > Display       for UnkWrapRc<T> { fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result { Display::fmt(&self.0.as_inner(), fmt) } }
impl<T: Any + PartialEq > PartialEq     for UnkWrapRc<T> { fn eq(&self, other: &Self) -> bool { self.0.as_inner().eq(other.0.as_inner()) } }
impl<T: Any + Eq        > Eq            for UnkWrapRc<T> {}
impl<T: Any + PartialOrd> PartialOrd    for UnkWrapRc<T> { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.0.as_inner().partial_cmp(other.0.as_inner()) } }
impl<T: Any + Ord       > Ord           for UnkWrapRc<T> { fn cmp(&self, other: &Self) -> Ordering { self.0.as_inner().cmp(other.0.as_inner()) } }
impl<T: Any + Hash      > Hash          for UnkWrapRc<T> { fn hash<H: Hasher>(&self, state: &mut H) { self.0.as_inner().hash(state) } }

impl<T: Any             > Default       for UnkWrapWeak<T> { fn default() -> Self { Self::new() } }
impl<T: Any + Debug     > Debug         for UnkWrapWeak<T> { fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result { match self.upgrade() {
    None    => fmt.debug_tuple("UnkWrapWeak").field(&format_args!("None")).finish(),
    Some(c) => fmt.debug_tuple("UnkWrapWeak").field(&Some(&*c)).finish(),
}}}

// Not (yet?) implemented:
// - Unpin, Pointer, UnwindSafe, CoerceUnsized, DispatchFromDyn

// Do not implement:
// - AsMut      (mutable access when COM objects are always refcounted / typically shared)
// - BorrowMut  (mutable access when COM objects are always refcounted / typically shared)
// - DerefMut   (mutable access when COM objects are always refcounted / typically shared)

impl<T: Any> TryFrom<&mcom::Rc<IUnknown>> for UnkWrapRc<T> {
    type Error = Error;
    fn try_from(value: &mcom::Rc<IUnknown>) -> Result<Self, Self::Error> {
        let value = value.try_cast::<UnkWrap<T>>().ok_or(Error::new_hr("IUnknown::QueryInterface", E_NOINTERFACE, ""))?;
        let value = unsafe { Arc::from_raw(value.into_raw()) };
        Ok(Self(value))
    }
}

impl<T: Any> From<UnkWrapRc<T>> for mcom::Rc<IUnknown> {
    fn from(value: UnkWrapRc<T>) -> Self {
        let raw : *const UnkWrap<T> = Arc::into_raw(value.0);
        let raw = raw as *mut UnkWrap<T> as *mut IUnknown;
        unsafe { mcom::Rc::from_raw_unchecked(raw) }
    }
}



#[repr(C)] pub(crate) struct UnkWrap<T: Any> {
    vtable: *const IUnknownVtbl,
    value:  T,
}

impl<T: Any> UnkWrap<T> {
    fn new(value: T) -> Self {
        let vtable : &'static IUnknownVtbl = &IUnknownVtbl {
            AddRef:         Self::add_ref,
            Release:        Self::release,
            QueryInterface: Self::query_interface,
        };
        Self {
            vtable: vtable,
            value,
        }
    }

    fn as_inner(&self) -> &T { &self.value }

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

unsafe impl<T: Any + Send> Send for UnkWrap<T> {}
unsafe impl<T: Any + Sync> Sync for UnkWrap<T> {}

impl<T: Any> Interface  for UnkWrap<T> { fn uuidof() -> GUID { type_guid::<Self>() } }
