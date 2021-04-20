#[macro_export]
macro_rules! include_file {
    ( $path:literal ) => {
        $crate::utility::StaticFile {
            cargo_manifest_dir: ::std::env!("CARGO_MANIFEST_DIR"),
            cargo_pkg_name:     ::std::env!("CARGO_PKG_NAME"),
            module_path:        ::std::module_path!(),

            root:               $crate::utility::StaticFileRoot::ModDir,
            path:               $path,
            data:               ::std::include_bytes!($path),
        }
    };
}

#[macro_export]
macro_rules! include_crate_file {
    ( $path:literal ) => {
        $crate::utility::StaticFile {
            cargo_manifest_dir: ::std::env!("CARGO_MANIFEST_DIR"),
            cargo_pkg_name:     ::std::env!("CARGO_PKG_NAME"),
            module_path:        ::std::module_path!(),

            root:               $crate::utility::StaticFileRoot::Crate,
            path:               $path,
            data:               ::std::include_bytes!(::std::concat!(::std::env!("CARGO_MANIFEST_DIR"), "/", $path)),
        }
    };
}
