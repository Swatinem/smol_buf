#![doc = include_str!("../README.md")]
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod buf16;
mod buf24;
mod str16;
mod str24;

pub use buf16::*;
pub use buf24::*;
pub use str16::*;
pub use str24::*;

#[cfg(feature = "intern")]
mod intern;
#[cfg(feature = "intern")]
pub use intern::*;
