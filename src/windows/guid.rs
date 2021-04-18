#![allow(dead_code)] // XXX

use super::Error;

use winapi::shared::guiddef::GUID;
use winapi::um::combaseapi::CoCreateGuid;

use std::any::*;
use std::collections::HashMap;
use std::sync::Mutex;



pub(crate) fn type_guid<T: Any>() -> GUID { *TYPE_GUIDS.lock().unwrap().entry(TypeId::of::<T>()).or_insert_with(|| co_create_guid().unwrap()) }

lazy_static::lazy_static! { static ref TYPE_GUIDS : Mutex<HashMap<TypeId, GUID>> = Default::default(); }

fn co_create_guid() -> Result<GUID, Error> {
    let mut guid = unsafe { std::mem::zeroed() };
    let hr = unsafe { CoCreateGuid(&mut guid) };
    Error::check_hr("CoCreateGuid", hr, "")?;
    Ok(guid)
}
