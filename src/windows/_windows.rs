//! Microsoft Windows related crates, functions, and types

#![cfg(windows)]

pub use ::mcom;
pub use ::winapi;

#[path = "d3d9/_d3d9.rs"  ] pub mod d3d9;
#[path = "d3d11/_d3d11.rs"] pub mod d3d11;
#[path = "dxgi/_dxgi.rs"  ] pub(crate) mod dxgi;
#[path = "hwnd/_hwnd.rs"  ] pub(crate) mod hwnd;

mod error;          pub use error::Error;
pub mod message;
pub mod monitor;
pub mod prelude;    pub use prelude::*;

mod com_box;        pub(crate) use com_box::*;
mod guid;           pub(crate) use guid::*;
mod misc;           pub(crate) use misc::*;
mod rect;           pub use rect::*;
mod window;         pub(crate) use window::*;



// Backwards compatability with 0.1.0

#[doc(hidden)] pub use message::loop_until_wm_quit    as message_loop_until_wm_quit;
#[doc(hidden)] pub use message::post_quit             as post_quit_message;
