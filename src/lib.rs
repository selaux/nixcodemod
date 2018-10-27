#![feature(slice_patterns)]

extern crate rnix;

mod collect;
mod node_builder;
mod operations;

pub use collect::*;
pub use node_builder::*;
pub use operations::*;
pub use rnix::parser::{ASTNode, Arena, NodeId};

