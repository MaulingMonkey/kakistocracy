#![allow(dead_code)] // XXX

use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::*;
use winapi::um::errhandlingapi::GetLastError;

use std::fmt::{self, Debug, Display, Formatter};



/// [`GetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror), but safe.
pub fn get_last_error() -> DWORD {
    unsafe { GetLastError() }
}



/// A Win32 error of some sort.
///
/// This might be an [`HRESULT`](https://www.hresult.info/), or this might be an `ERROR_*` VALUE.
/// A proper API might segregate the two cases completely, but even Win32 itself sometimes mixes and matches by accident.
/// As such, this type tries to handle the combined muddle.
///
/// ### See Also
/// * [System Error Codes](https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes)
/// * [hresult.info](https://www.hresult.info/)
#[derive(Clone, Debug)]
pub struct Error {
    pub(crate) method:          &'static str,
    pub(crate) error_source:    &'static str,
    pub(crate) error:           u32,
    pub(crate) note:            &'static str,
}

impl Error {
    pub fn as_u32(&self) -> u32 { self.error as _ }
    pub fn as_hresult(&self) -> HRESULT { self.error as _ }
    pub fn as_decomposed_hresult(&self) -> DecomposedHResult { DecomposedHResult::from(self.as_hresult()) }
}

impl Error {
    pub(crate) fn check_hr(method: &'static str, hr: HRESULT, note: &'static str) -> Result<(), Self> {
        if SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Self::new_hr(method, hr, note))
        }
    }

    pub(crate) fn last<T>(method: &'static str, note: &'static str) -> Result<T, Self> {
        Err(Error::new_gle(method, get_last_error(), note))
    }

    pub(crate) fn new(method: &'static str, error_source: &'static str, error: u32, note: &'static str) -> Self {
        Self { method, error_source, error, note }
    }

    pub(crate) fn new_gle(method: &'static str, error: u32, note: &'static str) -> Self {
        Self::new(method, "GetLastError()", error, note)
    }

    pub(crate) fn new_hr(method: &'static str, hr: HRESULT, note: &'static str) -> Self {
        Self::new(method, "HRESULT", hr as _, note)
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "{} failed", self.method)?;
        if !self.error_source.is_empty()    { write!(fmt, " with {} == 0x{:08x}", self.error_source, self.error)?; }
        if !self.note.is_empty()            { write!(fmt, " ({})", self.note)?; }
        Ok(())
    }
}

impl std::error::Error for Error {}




pub struct DecomposedHResult {
    pub sev:        ErrorSeverity,
    pub customer:   bool,
    pub reserved:   bool,
    pub facility:   u16,
    pub code:       u16,
}

impl From<HRESULT> for DecomposedHResult {
    fn from(hresult: HRESULT) -> Self { Self {
        sev: match (hresult as u32) >> 30 {
            0b00    => ErrorSeverity::Success,
            0b01    => ErrorSeverity::Informational,
            0b10    => ErrorSeverity::Warning,
            0b11    => ErrorSeverity::Error,
            sev     => panic!("BUG: DecomposedHResult::sev == 0x{:x}\nreport this to {}", sev, "https://github.com/MaulingMonkey/kakistocracy/issues"),
        },
        customer:   hresult & (1 << 29) != 0,
        reserved:   hresult & (1 << 28) != 0,
        facility:   ((hresult >> 16) & 0xFFF) as u16,
        code:       hresult as u16,
    }}
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    Success         = 0b00,
    Informational   = 0b01,
    Warning         = 0b10,
    Error           = 0b11,
}
