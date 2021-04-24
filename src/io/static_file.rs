use std::fmt::{self, Debug, Formatter};



/// The result of [`include_file!`].
pub struct StaticFile {
    #[doc(hidden)] pub path: &'static str,
    #[doc(hidden)] pub data: &'static [u8],
    #[doc(hidden)] pub _non_exhaustive_init_via_macros_only:   (),
}

impl StaticFile {
    pub fn path_str(&self) -> &'static str { self.path }
    pub fn as_bytes(&self) -> &'static [u8] { self.data }
    pub fn len(&self) -> usize { self.data.len() }
}

impl Debug for StaticFile {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("StaticFile").field(&self.path).finish()
    }
}

// TODO: comparison traits?



/// Include a file.  Similar to [`include_bytes!`](std::include_bytes), but results in a [`StaticFile`] containing additional metadata.
#[macro_export]
macro_rules! include_file {
    ( $path:literal ) => {
        $crate::io::StaticFile {
            path: $path,
            data: ::std::include_bytes!($path),
            _non_exhaustive_init_via_macros_only: (),
        }
    };
    ( CARGO_MANIFEST_DIR / $path:literal ) => {
        $crate::io::StaticFile {
            path: $path,
            data: ::std::include_bytes!(::std::concat!(::std::env!("CARGO_MANIFEST_DIR"), "/", $path)),
            _non_exhaustive_init_via_macros_only: (),
        }
    };
}
