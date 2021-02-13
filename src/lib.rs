//!

mod error;
pub mod learning;
mod tree;

pub use error::Error;

pub use tree::{nodes, Decision, Node, Tree};
