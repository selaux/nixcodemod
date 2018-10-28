use rnix::parser::{ASTNode, Arena, NodeId, AST};

pub trait CollectFromAST {
    type Item;
    fn visit_node(&self, arena: &Arena, node_id: NodeId, node: &ASTNode) -> Option<Self::Item>;
}

type MatchFn = Fn(&Arena, NodeId, &ASTNode) -> bool;

struct CollectMatchingNodes {
    match_fn: &'static MatchFn,
}

impl CollectFromAST for CollectMatchingNodes {
    type Item = (NodeId, ASTNode);

    fn visit_node(&self, arena: &Arena, node_id: NodeId, node: &ASTNode) -> Option<Self::Item> {
        if (self.match_fn)(arena, node_id, node) {
            Some((node_id, node.clone()))
        } else {
            None
        }
    }
}

pub fn collect_in_subtree<T>(
    collect: &impl CollectFromAST<Item = T>,
    arena: &Arena,
    node_id: NodeId,
    node: &ASTNode,
) -> Vec<T> {
    let mut result = vec![];
    let current_node_match = collect.visit_node(&arena, node_id, node);

    if let Some(m) = current_node_match {
        result.push(m);
    }

    for child_id in node.children(arena) {
        let child = &arena[child_id];
        let mut changes_for_child = collect_in_subtree(collect, arena, child_id, child);

        result.append(&mut changes_for_child);
    }

    result
}

pub fn find_children(
    match_fn: &'static MatchFn,
    ast: &AST<'static>,
    node_id: NodeId,
) -> Vec<(NodeId, ASTNode)> {
    let mut result = vec![];
    let collect = CollectMatchingNodes { match_fn };
    let arena = &ast.arena;
    let node = &arena[node_id];

    for child_id in node.children(arena) {
        let child = &arena[child_id];

        if let Some(r) = collect.visit_node(arena, child_id, child) {
            result.push(r)
        }
    }

    result
}

pub fn find_all(match_fn: &'static MatchFn, ast: &AST<'static>) -> Vec<(NodeId, ASTNode)> {
    let collect = CollectMatchingNodes { match_fn };
    let root_id = ast.root;
    let arena = &ast.arena;
    let root_node = &arena[root_id];

    collect_in_subtree(&collect, &arena, root_id, &root_node)
}

#[cfg(test)]
mod tests {
    use super::{find_all, find_children};
    use rnix::parser::{ASTKind, ASTNode, Arena, Data, NodeId};

    const CODE: &str = r#"
        { foo, bar }:
        let
            baz = bar;
        in
        {
            inherit bar;
        }
    "#;

    fn find_pattern(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
        node.kind == ASTKind::Pattern
    }

    fn find_pattern_entry(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
        node.kind == ASTKind::PatEntry
    }

    fn find_identifier(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
        node.kind == ASTKind::Ident
    }

    fn get_name_from_identifier(pair: (NodeId, ASTNode)) -> Option<String> {
        let (_, node) = pair;
        match &node.data {
            Data::Ident(_, name) => Some(name.clone()),
            _ => None,
        }
    }

    fn get_name_from_identifier_ref(pair: &(NodeId, ASTNode)) -> Option<String> {
        let (_, node) = pair;
        match &node.data {
            Data::Ident(_, name) => Some(name.clone()),
            _ => None,
        }
    }

    #[test]
    fn find_all_identifiers() {
        let ast = rnix::parse(CODE).unwrap();
        let identifiers: Vec<String> = find_all(&find_identifier, &ast)
            .into_iter()
            .filter_map(get_name_from_identifier)
            .collect();

        assert_eq!(
            identifiers,
            vec![
                "foo".to_string(),
                "bar".to_string(),
                "baz".to_string(),
                "bar".to_string(),
                "bar".to_string()
            ]
        );
    }

    #[test]
    fn find_children_identifiers() {
        let ast = rnix::parse(CODE).unwrap();
        let identifiers: Vec<String> = find_children(&find_pattern, &ast, ast.root)
            .into_iter()
            .flat_map(|(pattern_id, _)| find_children(&find_pattern_entry, &ast, pattern_id))
            .filter_map(|(pattern_entry_id, _)| {
                let identifiers = find_children(&find_identifier, &ast, pattern_entry_id);
                identifiers.get(0).and_then(get_name_from_identifier_ref)
            })
            .collect();

        assert_eq!(identifiers, vec!["foo".to_string(), "bar".to_string()]);
    }
}
