//! Microsoft Windows related crates, functions, and types

#![cfg(windows)]

pub use ::winapi;

mod errors;         pub use errors::*;
mod message_loop;   pub use message_loop::*;
mod misc;           pub use misc::*;
mod window;         pub use window::*;
