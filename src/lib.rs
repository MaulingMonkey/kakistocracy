#![deny(unreachable_patterns)]

#[path = "io/_io.rs"            ] pub mod io;
#[path = "utility/_utility.rs"  ] pub(crate) mod utility;
#[path = "windows/_windows.rs"  ] pub mod windows;
