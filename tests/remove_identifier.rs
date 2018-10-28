#[cfg(test)]
mod test_integration_remove {
    extern crate nixcodemod;
    extern crate rnix;

    use self::nixcodemod::*;
    use self::rnix::parser::ASTKind;

    #[test]
    fn test_remove_pattern_entry() {
        fn pattern_entry(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
            node.kind == ASTKind::PatEntry
        }
        let code = r#"
            { some, function, arguments }:
            {
                foo = "bar";
            }
        "#;
        let expected = r#"
            { some, arguments }:
            {
                foo = "bar";
            }
        "#;

        let ast = rnix::parse(code).unwrap();
        let operations: Vec<Operation> =
            find_all(&pattern_entry, &ast)
                .get(1)
                .map(|(node_id, _)| Operation::Remove(*node_id, Remove {}))
                .into_iter()
                .collect();
        let new_ast = apply_operations(&ast, &operations);

        assert_eq!(format!("{}", new_ast), expected);
    }

    #[test]
    fn test_remove_pattern_entry_leading_comma() {
        fn pattern_entry(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
            node.kind == ASTKind::PatEntry
        }
        let code = r#"
            { some
            , function
            , arguments
            }:
            {
                foo = "bar";
            }
        "#;
        let expected = r#"
            { some
            , function
            }:
            {
                foo = "bar";
            }
        "#;

        let ast = rnix::parse(code).unwrap();
        let operations: Vec<Operation> =
            find_all(&pattern_entry, &ast)
                .get(2)
                .map(|(node_id, _)| Operation::Remove(*node_id, Remove {}))
                .into_iter()
                .collect();
        let new_ast = apply_operations(&ast, &operations);

        assert_eq!(format!("{}", new_ast), expected);
    }
}
