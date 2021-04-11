//! Microsoft Windows related crates, functions, and types

#![cfg(windows)]

pub use ::mcom;
pub use ::winapi;

#[path = "d3d9/_d3d9.rs"] pub mod d3d9;

mod errors;         pub use errors::*;
mod message_loop;   pub use message_loop::*;
mod misc;           pub use misc::*;
mod monitors;       pub use monitors::*;
mod window;         pub use window::*;
