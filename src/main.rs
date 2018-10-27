#![feature(slice_patterns)]

extern crate rnix;

mod collect;
mod node_builder;
mod operations;

use std::path::PathBuf;

use std::fs::File;
use std::io::Read;

pub use node_builder::*;
pub use operations::*;
pub use rnix::parser::{ASTNode, Arena, NodeId};

fn stdenv_identifier(_: &Arena, _: &NodeId, node: &ASTNode) -> bool {
    match &node.data {
        rnix::parser::Data::Ident(_, name) => name == "stdenv",
        _ => false,
    }
}

fn main() {
    let path = PathBuf::from("../nixpkgs/pkgs/build-support/rust/default.nix");
    let mut code = String::new();

    File::open(&path)
        .unwrap()
        .read_to_string(&mut code)
        .unwrap();

    let ast = rnix::parse(&code).unwrap();
    let nodes_to_replace = collect::find_all(&stdenv_identifier, &ast);
    let operations: Vec<Operation> = nodes_to_replace
        .into_iter()
        .map(|(node_id, _)| {
            Operation::Replace(
                node_id,
                Replacement {
                    node: build_identifier("foo"),
                },
            )
        })
        .collect();

    println!("Replacing stdenv with foo in {:?}", path);
    println!("Changes: {:?}", operations);

    let new_ast = apply_operations(&ast, &operations);
    print!("New AST: {}", new_ast);
}
