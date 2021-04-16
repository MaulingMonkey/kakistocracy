//! A prelude for conveniently writing platform-specific code.
//!
//! Includes extension traits, and some important crates.



// crates

pub use ::mcom;
pub use ::winapi;



// extension traits

#[doc(no_inline)] pub use super::RectExt;
#[cfg(feature = "d3d9" )] #[doc(no_inline)] pub use super::d3d9 ::prelude::*;
#[cfg(feature = "d3d11")] #[doc(no_inline)] pub use super::d3d11::prelude::*;
#[cfg(feature = "dxgi" )] #[doc(no_inline)] pub use super::dxgi ::prelude::*;
