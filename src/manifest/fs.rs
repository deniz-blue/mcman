use std::path::PathBuf;

use crate::core::kdl::{KdlRead, Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyFile {
    pub from: PathBuf,
    pub to: PathBuf,
    pub overwrite: bool,
}

impl KdlRead for CopyFile {
    fn read(reader: &mut Reader) -> Self {
        Self {
            from: reader.required_path_argument("source path"),
            to: reader.required_path_argument("destination path"),
            overwrite: reader.flag_property("overwrite"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymlinkFile {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl KdlRead for SymlinkFile {
    fn read(reader: &mut Reader) -> Self {
        Self {
            from: reader.required_path_argument("source path"),
            to: reader.required_path_argument("destination path"),
        }
    }
}
