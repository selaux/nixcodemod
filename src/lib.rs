#![feature(slice_patterns)]
#[crate_type = "lib"]

extern crate rnix;

mod collect;
mod node_builder;
mod operations;

use std::path::PathBuf;

pub use collect::*;
pub use node_builder::*;
pub use operations::*;
pub use rnix::parser::{ASTNode, Arena, NodeId};

