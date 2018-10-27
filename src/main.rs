#![feature(slice_patterns)]

extern crate rnix;
extern crate arenatree;

use std::path::PathBuf;

use std::fs::File;
use std::io::Read;

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
            // TODO: What about the children? And what about exposing this in rnix
            node: arenatree::Node {
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

fn visit_node(node_id: rnix::parser::NodeId, node: &rnix::parser::ASTNode) -> Vec<Change> {
    match &node.data {
        rnix::parser::Data::Ident(meta, name) => {
            match name.as_str() {
                "stdenv" => vec![
                    Change::Replace(
                        node_id,
                        Replacement {
                            kind: rnix::parser::ASTKind::Ident,
                            data: rnix::parser::Data::Ident(
                                meta.clone(),
                                "foo".to_string()
                            )
                        }
                    )
                ],
                _ => vec![]
            }
        },
        _ => vec![]
    }
}

fn walk_node(arena: &rnix::parser::Arena, node_id: rnix::parser::NodeId, node: &rnix::parser::ASTNode) -> Vec<Change> {
    let mut changes = vec![];
    let mut changes_for_current_node = visit_node(node_id, node);

    changes.append(&mut changes_for_current_node);
    for child_id in node.children(arena) {
        let child = &arena[child_id];
        let mut changes_for_child = walk_node(arena, child_id, child);

        changes.append(&mut changes_for_child);
    }

    return changes;
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

fn main() {
    let path = PathBuf::from("../nixpkgs/pkgs/build-support/rust/default.nix");
    let mut code = String::new();

    File::open(&path).unwrap().read_to_string(&mut code).unwrap();

    let ast = rnix::parse(&code).unwrap();
    let root_node = &ast.arena[ast.root];

    println!("Replacing stdenv with foo in {:?}", path);

    let changes = walk_node(&ast.arena, ast.root, root_node);
    println!("Changes: {:?}", changes);

    let new_ast = apply_changes(&ast, &changes);
    print!("New AST: {}", new_ast);
}
