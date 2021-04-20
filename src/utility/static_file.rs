use std::fmt::{self, Debug, Formatter};



#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StaticFileRoot {
    Crate,
    ModDir,
}

pub struct StaticFile {
    pub cargo_manifest_dir: &'static str,
    pub cargo_pkg_name:     &'static str,
    pub module_path:        &'static str,

    pub root:               StaticFileRoot,
    pub path:               &'static str,
    pub data:               &'static [u8],
}

impl Debug for StaticFile {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("StaticFile").field(&self.root).field(&self.path).finish()
    }
}
