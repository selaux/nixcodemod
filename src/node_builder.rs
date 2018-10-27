#[derive(Debug)]
pub struct IsolatedIdentifier {
    pub name: String
}

#[derive(Debug)]
pub enum IsolatedNode {
    Identifier(IsolatedIdentifier)
}

pub fn build_identifier(name: &str) -> IsolatedNode {
    return IsolatedNode::Identifier(IsolatedIdentifier {
        name: String::from(name)
    });
}