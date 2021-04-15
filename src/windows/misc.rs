use crate::windows::*;

use winapi::shared::minwindef::*;
use winapi::um::libloaderapi::*;

use std::ptr::null_mut;



/// <code>[GetModuleHandle](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew)(NULL)</code>
pub(crate) fn get_module_handle_exe() -> HMODULE {
    unsafe { GetModuleHandleW(null_mut()) }
}
