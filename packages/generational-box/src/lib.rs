#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub use error::*;
pub use gen_box::{GenerationalBox, GenerationalBoxId};
pub use references::*;
pub use storage::Storage;
pub use sync::SyncStorage;
pub use unsync::UnsyncStorage;

mod error;
mod gen_box;
mod mem_location;
mod references;
mod storage;
mod sync;
mod unsync;

pub(crate) mod innerlude {
    pub(crate) use crate::error::*;
    pub(crate) use crate::mem_location::*;
    pub(crate) use crate::references::*;
    pub(crate) use crate::storage::*;
}
