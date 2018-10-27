use rnix::parser::{ASTKind, ASTNode, Data, Node};

#[derive(Debug)]
pub struct IsolatedIdentifier {
    pub name: String,
}

#[derive(Debug)]
pub enum IsolatedNode {
    Identifier(IsolatedIdentifier),
}

pub trait ToAstNode {
    fn to_ast_node(&self, reference_node: &ASTNode) -> ASTNode;
}

impl ToAstNode for IsolatedIdentifier {
    fn to_ast_node(&self, reference_node: &ASTNode) -> ASTNode {
        ASTNode {
            kind: ASTKind::Ident,
            data: Data::Ident(
                match &reference_node.data {
                    Data::Ident(meta, _) => meta.clone(),
                    _ => unreachable!(),
                },
                self.name.clone(),
            ),
            span: reference_node.span.clone(),
            node: Node {
                child: None,
                sibling: reference_node.node.sibling,
            },
        }
    }
}

impl ToAstNode for IsolatedNode {
    fn to_ast_node(&self, reference_node: &ASTNode) -> ASTNode {
        match self {
            IsolatedNode::Identifier(id) => id.to_ast_node(reference_node),
        }
    }
}

pub fn build_identifier(name: &str) -> IsolatedNode {
    return IsolatedNode::Identifier(IsolatedIdentifier {
        name: String::from(name),
    });
}
