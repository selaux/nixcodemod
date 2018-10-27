#![feature(slice_patterns)]

extern crate rnix;

mod collect;

use std::path::PathBuf;

use std::fs::File;
use std::io::Read;
use rnix::parser::{NodeId, ASTNode};

#[derive(Debug)]
struct Replacement {
    kind: rnix::parser::ASTKind,
    data: rnix::parser::Data
}

impl Replacement {
    fn apply_on_ast(&self, ast: &rnix::parser::AST<'static>, original_node_id: &rnix::parser::NodeId) -> rnix::parser::AST<'static> {
        let mut new_arena = ast.arena.clone();
        let node_ids = new_arena.get_ids();
        let original_node = &ast.arena[*original_node_id];
        let replacement_node = rnix::parser::ASTNode {
            kind: self.kind,
            data: self.data.clone(),
            span: original_node.span.clone(),
            // TODO: What about the children?
            node: rnix::parser::Node {
                child: None,
                sibling: original_node.node.sibling
            }
        };

        let replacement_node_id = new_arena.insert(replacement_node);

        for node_id in node_ids {
            let mut astnode_to_update = &mut new_arena[node_id];
            let mut node_to_update = &mut astnode_to_update.node;

            if Some(*original_node_id) == node_to_update.child {
                node_to_update.child = Some(replacement_node_id);
            }
            if Some(*original_node_id) == node_to_update.sibling {
                node_to_update.sibling = Some(replacement_node_id);
            }
        }

        new_arena.take(*original_node_id);

        return rnix::parser::AST {
            arena: new_arena,
            root: ast.root.clone()
        };
    }
}

#[derive(Debug)]
enum Change {
    Replace(rnix::parser::NodeId, Replacement)
}

fn apply_changes(ast: &rnix::parser::AST<'static>, changes: &[Change]) -> rnix::parser::AST<'static> {
    match changes {
        [ change, rest.. ] => {
            let new_ast = match change {
                Change::Replace(node_id, replacement) => replacement.apply_on_ast(ast, node_id)
            };
            apply_changes(&new_ast, rest)
        },
        [] => ast.clone()
    }
}

fn stdenv_identifier(_: &NodeId, node: &ASTNode) -> bool {
    match &node.data {
        rnix::parser::Data::Ident(_, name) => name == "stdenv",
        _ => false
    }
}

fn main() {
    let path = PathBuf::from("../nixpkgs/pkgs/build-support/rust/default.nix");
    let mut code = String::new();

    File::open(&path).unwrap().read_to_string(&mut code).unwrap();

    let ast = rnix::parse(&code).unwrap();
    let nodes_to_replace = collect::find_all(&stdenv_identifier, &ast);
    let operations: Vec<Change> = nodes_to_replace.into_iter().map(|(node_id, node)| {
        match &node.data {
            rnix::parser::Data::Ident(meta, _) => Change::Replace(
                node_id,
                Replacement {
                    kind: rnix::parser::ASTKind::Ident,
                    data: rnix::parser::Data::Ident(
                        meta.clone(),
                        "foo".to_string()
                    )
                }
            ),
            _ => unreachable!()
        }
    }).collect();

    println!("Replacing stdenv with foo in {:?}", path);
    println!("Changes: {:?}", operations);

    let new_ast = apply_changes(&ast, &operations);
    print!("New AST: {}", new_ast);
}
