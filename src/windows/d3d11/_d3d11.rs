//! Direct3D11 related crates, functions, and types

#![cfg(feature = "d3d11")]

pub(crate) mod prelude;         #[allow(unused_imports)] pub(crate) use prelude::*;
mod mwc;                        pub use mwc::*;
