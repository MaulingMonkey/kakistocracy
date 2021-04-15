//! Microsoft Windows related crates, functions, and types

#![cfg(windows)]

pub use ::mcom;
pub use ::winapi;

#[path = "d3d9/_d3d9.rs"] pub mod d3d9;

pub mod messages;
pub mod prelude;

mod errors;         pub use errors::*;
mod misc;           pub use misc::*;
mod monitors;       pub use monitors::*;
mod rect;           pub use rect::*;
mod window;         pub use window::*;



// Backwards compatability with 0.1.0

#[doc(hidden)] pub use messages::loop_until_wm_quit    as message_loop_until_wm_quit;
#[doc(hidden)] pub use messages::post_quit             as post_quit_message;
