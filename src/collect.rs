use rnix::parser::{NodeId, ASTNode, AST, Arena};

pub trait CollectFromAST {
    type Item;
    fn visit_node(&self, arena: &Arena, node_id: &NodeId, node: &ASTNode) -> Option<Self::Item>;
}

type MatchFn = Fn(&Arena, &NodeId, &ASTNode) -> bool;

struct CollectMatchingNodes {
    match_fn: &'static MatchFn
}

impl CollectFromAST for CollectMatchingNodes {
    type Item = (NodeId, ASTNode);

    fn visit_node(&self, arena: &Arena, node_id: &NodeId, node: &ASTNode) -> Option<Self::Item> {
        if (self.match_fn)(arena, node_id, node) {
            Some((*node_id, node.clone()))
        } else {
            None
        }
    }
}

pub fn collect_in_tree<T>(
    collect: &impl CollectFromAST<Item = T>,
    arena: &Arena,
    node_id: &NodeId,
    node: &ASTNode
) -> Vec<T> {
    let mut result = vec![];
    let current_node_match = collect.visit_node(&arena, node_id, node);

    if let Some(m) = current_node_match {
        result.push(m);
    }

    for child_id in node.children(arena) {
        let child = &arena[child_id];
        let mut changes_for_child = collect_in_tree(collect, arena, &child_id, child);

        result.append(&mut changes_for_child);
    }

    return result;
}

pub fn find_all(match_fn: &'static MatchFn, ast: &AST<'static>) -> Vec<(NodeId, ASTNode)> {
    let collect = CollectMatchingNodes { match_fn: match_fn };
    let root_id = ast.root;
    let arena = &ast.arena;
    let root_node = &arena[root_id];

    return collect_in_tree(&collect, &arena, &root_id, &root_node);
}