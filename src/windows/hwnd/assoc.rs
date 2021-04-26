use crate::windows::Error;
use super::hooks;

use winapi::shared::windef::HWND;
use winapi::shared::winerror::*;

use std::any::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;



pub(crate) fn valid_window(hwnd: HWND) -> bool {
    ThreadLocal::with(|tl|{
        let windows = tl.windows.borrow();
        windows.get(&hwnd).is_some()
    })
}

pub fn get<T: Any>(hwnd: HWND) -> Result<impl Deref<Target = T>, Error> {
    ThreadLocal::with(|tl|{
        let windows = tl.windows.borrow();
        let window = windows.get(&hwnd).ok_or(Error::new("kakistocracy::windows::hwnd::assoc::get", "", E_HANDLE as _, "no such handle"))?;
        let rc = window.assoc.get(&TypeId::of::<T>()).ok_or(Error::new("kakistocracy::windows::hwnd::assoc::get", "", E_FAIL as _, "no such type associated with handle"))?;
        let rc = rc.clone().downcast::<T>().unwrap();
        Ok(rc)
    })
}

pub fn set<T: Any>(hwnd: HWND, value: T) -> Result<Option<impl Deref<Target = T>>, Error> {
    let rc = Rc::new(value);
    ThreadLocal::with(|tl|{
        let mut windows = tl.windows.borrow_mut();
        let window = windows.get_mut(&hwnd).ok_or(Error::new("kakistocracy::windows::hwnd::assoc::get", "", E_HANDLE as _, "no such handle"))?;
        let prev = window.assoc.insert(TypeId::of::<T>(), rc);
        Ok(prev.map(|rc| rc.downcast::<T>().unwrap()))
    })
}

#[allow(dead_code)]
pub fn remove<T: Any>(hwnd: HWND) -> Result<Option<impl Deref<Target = T>>, Error> {
    ThreadLocal::with(|tl|{
        let mut windows = tl.windows.borrow_mut();
        let window = windows.get_mut(&hwnd).ok_or(Error::new("kakistocracy::windows::hwnd::assoc::get", "", E_HANDLE as _, "no such handle"))?;
        let prev = window.assoc.remove(&TypeId::of::<T>());
        Ok(prev.map(|rc| rc.downcast::<T>().unwrap()))
    })
}



#[derive(Default)] struct ThreadLocal {
    pub windows: RefCell<HashMap<HWND, Assoc>>,
}

#[derive(Default)] struct Assoc {
    pub assoc: HashMap<TypeId, Rc<dyn Any>>,
}



impl ThreadLocal {
    fn with<R>(f: impl FnOnce(&ThreadLocal) -> R) -> R {
        hooks::ensure_registered();
        TL.with(|tl| f(tl))
    }
}

pub(super) unsafe fn on_hwnd_creating(hwnd: HWND) {
    let _prev = TL.with(|tl| tl.windows.borrow_mut().insert(hwnd, Default::default()));
    debug_assert!(_prev.is_none(), "kakistocracy::windows::hwnd::assoc::ThreadLocal::windows: already contained an entry for the hwnd");
}

pub(super) unsafe fn on_hwnd_destroyed(hwnd: HWND) {
    let _entry = TL.with(|tl| tl.windows.borrow_mut().remove(&hwnd));
    debug_assert!(_entry.is_some(), "kakistocracy::windows::hwnd::assoc::ThreadLocal::windows: didn't contain an entry for the hwnd");
}

thread_local! { static TL : ThreadLocal = Default::default(); }
