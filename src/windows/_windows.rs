//! Microsoft Windows related crates, functions, and types

#![cfg(windows)]

pub use ::winapi;

mod message_loop;   pub use message_loop::*;
