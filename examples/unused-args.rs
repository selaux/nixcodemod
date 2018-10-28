extern crate glob;
extern crate rnix;
extern crate nixcodemod;

use std::env;
use std::fs;
use std::path::Path;
use std::collections::HashSet;
use glob::glob;
use rnix::parser::{ASTKind, AST, Arena, NodeId, ASTNode, Data};

fn get_name_from_identifier_node(node: &ASTNode) -> String {
    match &node.data {
        Data::Ident(_, name) => name.clone(),
        _ => unreachable!(),
    }
}

fn find_pattern(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
    node.kind == ASTKind::Pattern
}

fn find_pattern_entry(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
    node.kind == ASTKind::PatEntry
}

fn find_identifier(_: &Arena, _: NodeId, node: &ASTNode) -> bool {
    node.kind == ASTKind::Ident
}

fn collect_package_argument_identifiers(ast: &AST<'static>) -> HashSet<(NodeId, String)> {
    nixcodemod::find_children(&find_pattern, &ast, ast.root)
        .into_iter()
        .flat_map(|(pattern_id, _)| nixcodemod::find_children(&find_pattern_entry, &ast, pattern_id))
        .filter_map(|(pattern_entry_id, _)| {
            let identifiers = nixcodemod::find_children(&find_identifier, &ast, pattern_entry_id);
            identifiers.get(0).map(|(node_id, node)| (*node_id, get_name_from_identifier_node(&node)))
        })
        .collect()
}

fn collect_used_identifiers(ast: &AST<'static>, package_arguments: &HashSet<(NodeId, String)>) -> HashSet<(NodeId, String)> {
    nixcodemod::find_all(&find_identifier, &ast)
        .into_iter()
        .map(|(node_id, node)| (node_id, get_name_from_identifier_node(&node)))
        .filter(|pair| !package_arguments.contains(pair))
        .collect()
}

fn remove_unused(identifiers: &HashSet<&String>, ast: &AST<'static>) -> AST<'static> {
    let operations: Vec<nixcodemod::Operation> = nixcodemod::find_children(&find_pattern, &ast, ast.root)
        .into_iter()
        .flat_map(|(pattern_id, _)| nixcodemod::find_children(&find_pattern_entry, &ast, pattern_id))
        .filter(|(pattern_entry_id, _)| {
            let child_identifiers = nixcodemod::find_children(&find_identifier, &ast, *pattern_entry_id);
            let name = child_identifiers.get(0).map(|(_, node)| get_name_from_identifier_node(&node)).unwrap();

            identifiers.contains(&name)
        })
        .map(|(pattern_entry_id, _)| nixcodemod::Operation::Remove(pattern_entry_id, nixcodemod::Remove {}))
        .collect();

    nixcodemod::apply_operations(&ast, &operations)
}

fn not_applicable_to_ast(ast: &AST<'static>) -> Option<String> {
    let root_node = &ast.arena[ast.root];

    if root_node.kind != ASTKind::Lambda {
        return Some("Root node is not a function".to_string());
    }

    None
}

fn patch_default_nix_file(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|err| format!("Error reading file {:?}: {}", path, err))?;
    let ast = rnix::parse(&content).map_err(|err| format!("Error parsing file {:?}: {}", path, err))?;

    if let Some(reason) = not_applicable_to_ast(&ast) {
        println!("Skipping {:?} because: {}", path, reason);
        return Ok(());
    }

    let arguments = collect_package_argument_identifiers(&ast);
    let used = collect_used_identifiers(&ast, &arguments);

    let argument_names: HashSet<&String> = arguments.iter().map(|(_, s)| s).collect();
    let used_names: HashSet<&String> = used.iter().map(|(_, s)| s).collect();
    let diff: HashSet<&String> = argument_names.difference(&used_names).map(|s| *s).collect();

    if !diff.is_empty() {
        println!("Removing {:?} from {:?}", diff, path);
        let new_ast = remove_unused(&diff, &ast);
        fs::write(path, format!("{}", new_ast)).map_err(|err| format!("Error writing file {:?}: {}", path, err))?;
    }

    Ok(())
}

fn patch_default_nix_files(nixpkgs_arg: &str) -> Result<(), String> {
        let nixpkgs_pkgs_path = Path::new(nixpkgs_arg);
        let nixpkgs_pkgs_str = nixpkgs_pkgs_path.as_os_str().to_str().ok_or_else(|| "Unable to decode path".to_string())?;
        let glob_str = format!("{}/**/default.nix", nixpkgs_pkgs_str);
        let glob_result = glob(&glob_str).map_err(|err| format!("Failed to glob: {:?}", err))?;

        println!("Fixing unused package arguments in {}", glob_str);

        for result in glob_result {
            let path = result.map_err(|err| format!("Glob err {:?}", err))?;
            patch_default_nix_file(&path)?;
        }

        Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(nixpkgs_arg) = args.get(1) {
        patch_default_nix_files(nixpkgs_arg).unwrap();
    } else {
        println!("Usage:\n\tunused-args <path-to-nixpkgs>");
    }
}