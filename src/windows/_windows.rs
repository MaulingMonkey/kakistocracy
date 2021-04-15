//! Microsoft Windows related crates, functions, and types

#![cfg(windows)]

pub use ::mcom;
pub use ::winapi;

#[path = "d3d9/_d3d9.rs"] pub mod d3d9;

pub mod error;      pub use error::Error;
pub mod message;
pub(crate) mod monitor;
pub mod prelude;

mod misc;           pub(crate) use misc::*;
mod rect;           pub use rect::*;
mod window;         pub use window::*;



// Backwards compatability with 0.1.0

#[doc(hidden)] pub use message::loop_until_wm_quit    as message_loop_until_wm_quit;
#[doc(hidden)] pub use message::post_quit             as post_quit_message;
