//! Direct3D11 related crates, functions, and types

#![cfg(feature = "d3d11")]

mod device;                     pub(crate) use device::*;
pub(crate) mod prelude;         #[allow(unused_imports)] pub(crate) use prelude::*;
mod render;                     pub use render::*;
pub(crate) mod sprite;
mod texture_cache;              pub(crate) use texture_cache::*;
mod thread_local;               pub use thread_local::*;
mod traits;                     pub use traits::*;
