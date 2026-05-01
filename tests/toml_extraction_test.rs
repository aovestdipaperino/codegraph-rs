#![cfg(feature = "lang-toml")]

use tokensave::extraction::LanguageExtractor;
use tokensave::extraction::TomlExtractor;
use tokensave::types::*;

#[test]
fn test_toml_file_node_is_root() {
    let source = "[package]\nname = \"foo\"\n";
    let result = TomlExtractor.extract("Cargo.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let files: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::File)
        .collect();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "Cargo.toml");
}

#[test]
fn test_toml_extracts_tables() {
    let source = "[package]\nname = \"foo\"\n\n[dependencies]\nserde = \"1\"\n\n[features]\ndefault = []\n";
    let result = TomlExtractor.extract("Cargo.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let modules: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .collect();
    assert_eq!(modules.len(), 3);
    assert_eq!(modules[0].name, "package");
    assert_eq!(modules[1].name, "dependencies");
    assert_eq!(modules[2].name, "features");
}

#[test]
fn test_toml_dotted_table_hierarchy() {
    let source = "[a]\nx = 1\n\n[a.b]\ny = 2\n\n[a.b.c]\nz = 3\n\n[a.d]\nw = 4\n";
    let result = TomlExtractor.extract("config.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let a = result.nodes.iter().find(|n| n.name == "a").unwrap();
    let ab = result.nodes.iter().find(|n| n.name == "a.b").unwrap();
    let abc = result.nodes.iter().find(|n| n.name == "a.b.c").unwrap();
    let ad = result.nodes.iter().find(|n| n.name == "a.d").unwrap();

    let contains: Vec<_> = result
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Contains)
        .collect();

    // `a` (level 1) contains `a.b` and `a.d` (level 2)
    let a_contains: Vec<_> = contains.iter().filter(|e| e.source == a.id).collect();
    assert!(a_contains.iter().any(|e| e.target == ab.id));
    assert!(a_contains.iter().any(|e| e.target == ad.id));

    // `a.b` (level 2) contains `a.b.c` (level 3)
    let ab_contains: Vec<_> = contains.iter().filter(|e| e.source == ab.id).collect();
    assert!(ab_contains.iter().any(|e| e.target == abc.id));
}

#[test]
fn test_toml_table_array_element() {
    let source = "[[items]]\nname = \"a\"\n\n[[items]]\nname = \"b\"\n";
    let result = TomlExtractor.extract("config.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let modules: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .collect();
    assert_eq!(modules.len(), 2);
    assert!(modules.iter().all(|m| m.name == "items"));
    assert!(modules
        .iter()
        .all(|m| m.signature.as_deref() == Some("[[items]]")));
}

#[test]
fn test_toml_table_signature_format() {
    let source = "[package.metadata]\nfoo = 1\n";
    let result = TomlExtractor.extract("Cargo.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let m = result
        .nodes
        .iter()
        .find(|n| n.name == "package.metadata")
        .unwrap();
    assert_eq!(m.signature.as_deref(), Some("[package.metadata]"));
}

#[test]
fn test_toml_handles_empty_file() {
    let source = "";
    let result = TomlExtractor.extract("config.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let files: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::File)
        .collect();
    assert_eq!(files.len(), 1);
    let modules: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .collect();
    assert!(modules.is_empty());
}

#[test]
fn test_toml_handles_top_level_keys_only() {
    let source = "name = \"foo\"\nversion = \"1.0.0\"\n";
    let result = TomlExtractor.extract("config.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let modules: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .collect();
    assert!(modules.is_empty(), "should have no Module nodes");
}

#[test]
fn test_toml_unrelated_table_after_nested() {
    // After `[a.b.c]`, a fresh `[x]` (level 1) must re-parent to the file,
    // not nest under `a.b.c`.
    let source = "[a.b.c]\nv = 1\n\n[x]\ny = 2\n";
    let result = TomlExtractor.extract("config.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let file = result
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::File)
        .unwrap();
    let x = result.nodes.iter().find(|n| n.name == "x").unwrap();
    let abc = result.nodes.iter().find(|n| n.name == "a.b.c").unwrap();

    let contains: Vec<_> = result
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Contains)
        .collect();

    assert!(contains
        .iter()
        .any(|e| e.source == file.id && e.target == x.id));
    assert!(!contains
        .iter()
        .any(|e| e.source == abc.id && e.target == x.id));
}

#[test]
fn test_toml_quoted_table_key() {
    let source = "[\"weird key\"]\nv = 1\n";
    let result = TomlExtractor.extract("config.toml", source);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let modules: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .collect();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "\"weird key\"");
}

#[test]
fn test_toml_extensions() {
    let ext = TomlExtractor;
    assert!(ext.extensions().contains(&"toml"));
}

#[test]
fn test_toml_language_name() {
    let ext = TomlExtractor;
    assert_eq!(ext.language_name(), "TOML");
}
