#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]
#![warn(missing_docs)]
#![allow(clippy::type_complexity)]

mod macros;
pub use macros::*;

mod rt;
pub use rt::*;

mod effect;
pub use effect::*;

// mod selector;
// pub use selector::*;

pub(crate) mod signal;
pub use signal::*;

pub mod readonly_signal;
pub use readonly_signal::*;

// mod dependency;
// pub use dependency::*;

// mod map;
// pub use map::*;

mod copyvalue;
pub use copyvalue::*;

// mod comparer;
// pub use comparer::*;

// mod global;
// pub use global::*;

mod impls;

pub use generational_box::{Storage, SyncStorage, UnsyncStorage};
