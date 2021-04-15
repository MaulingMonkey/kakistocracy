//! Direct3D11 related crates, functions, and types

#![cfg(feature = "d3d11")]

pub mod prelude;                pub use prelude::*;
mod mwc;                        pub use mwc::*;
