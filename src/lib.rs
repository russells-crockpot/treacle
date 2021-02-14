//!

mod error;
pub mod learning;
pub(crate) mod tree;

pub use error::{Error, Result};

pub use tree::*;
