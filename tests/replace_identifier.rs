#[cfg(test)]
mod test_integration_replace {
    extern crate nixcodemod;

    use self::nixcodemod::*;

    #[test]
    fn test_replace_identifier_by_identifier() {
        fn some_identifier(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
            match &node.data {
                rnix::parser::Data::Ident(_, name) => name == "some",
                _ => false,
            }
        }
        let code = r#"
            { some, function }:
            {
                inherit some;
                foo = "bar";
            }
        "#;
        let expected = r#"
            { other, function }:
            {
                inherit other;
                foo = "bar";
            }
        "#;

        let ast = rnix::parse(code).unwrap();
        let nodes_to_replace = find_all(&some_identifier, &ast);
        let operations: Vec<Operation> = nodes_to_replace
            .into_iter()
            .map(|(node_id, _)| {
                Operation::Replace(
                    node_id,
                    Replacement {
                        node: build_identifier("other"),
                    },
                )
            })
            .collect();
        let new_ast = apply_operations(&ast, &operations);

        assert_eq!(format!("{}", new_ast), expected);
    }
}
