use rnix::parser::{AST, ASTNode, ASTKind, Data, NodeId, Node};
use node_builder::IsolatedNode;

pub trait OperationExt {
    fn apply(&self, ast: &AST<'static>, original_node_id: &NodeId) -> AST<'static>;
}

#[derive(Debug)]
pub struct Replacement {
    pub node: IsolatedNode
}

#[derive(Debug)]
pub enum Operation {
    Replace(NodeId, Replacement)
}

impl OperationExt for Replacement {
    fn apply(&self, ast: &AST<'static>, original_node_id: &NodeId) -> AST<'static> {
        let mut new_arena = ast.arena.clone();
        let node_ids = new_arena.get_ids();
        let original_node = &ast.arena[*original_node_id];
        let replacement_node = ASTNode {
            kind: ASTKind::Ident,
            data: Data::Ident(
                match &original_node.data {
                  Data::Ident(meta, _) => meta.clone(),
                  _ => unreachable!()
                },
                match &self.node {
                    IsolatedNode::Identifier(id) => id.name.clone()
                }
            ),
            span: original_node.span.clone(),
            // TODO: What about the children?
            node: Node {
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

        return AST {
            arena: new_arena,
            root: ast.root.clone()
        };
    }
}

pub fn apply_operations(ast: &AST<'static>, operations: &[Operation]) -> AST<'static> {
    match operations {
        [ operation, rest.. ] => {
            let new_ast = match operation {
                Operation::Replace(node_id, replacement) => replacement.apply(ast, node_id)
            };
            apply_operations(&new_ast, rest)
        },
        [] => ast.clone()
    }
}
