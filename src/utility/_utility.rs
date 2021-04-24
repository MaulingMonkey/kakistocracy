//! Misc. utility types and functions

mod frame_rate_counter;         #[allow(unused_imports)] pub(crate) use frame_rate_counter::*;
mod send_sync_cell;             #[allow(unused_imports)] pub(crate) use send_sync_cell::*;
mod static_bytes_ref;           #[cfg_attr(not(windows), allow(unused_imports))] pub(crate) use static_bytes_ref::*;
mod static_file;                pub use static_file::*;
