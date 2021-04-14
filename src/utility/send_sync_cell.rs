#![allow(dead_code)]

use std::ops::*;



/// Mark a type as Send + Sync, even if T isn't.
pub struct SendSyncCell<T>(T);

impl<T> SendSyncCell<T> {
    /// Mark a type as Send + Sync, even if T isn't.
    ///
    /// ### Safety
    ///
    /// `T` should be safe to send and share between threads.
    /// My main use case of this is for pointer-based types.
    /// Use extreme caution when using this type!
    pub const unsafe fn new(value: T) -> Self { Self(value) }
}

unsafe impl<T> Send for SendSyncCell<T> {}
unsafe impl<T> Sync for SendSyncCell<T> {}

impl<T> Deref for SendSyncCell<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

impl<T> DerefMut for SendSyncCell<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}
