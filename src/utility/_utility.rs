//! Misc. utility types and functions

mod frame_rate_counter;         pub use frame_rate_counter::*;
mod send_sync_cell;             pub(crate) use send_sync_cell::*;
mod static_bytes_ref;           pub(crate) use static_bytes_ref::*;
