/// Tree-sitter based TOML source code extractor.
///
/// Parses TOML source files and emits nodes and edges for the code graph.
/// Each `[table]` and `[[table_array]]` header becomes a `Module` node with
/// a `Contains` edge from its parent (file or enclosing table).
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use tree_sitter::{Node as TsNode, Parser, Tree};

use crate::types::{
    generate_node_id, Edge, EdgeKind, ExtractionResult, Node, NodeKind, Visibility,
};

pub struct TomlExtractor;

struct ExtractionState {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    file_path: String,
    source: Vec<u8>,
    timestamp: u64,
    node_stack: Vec<(String, String, usize)>,
}

impl ExtractionState {
    fn new(file_path: &str, source: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            file_path: file_path.to_string(),
            source: source.as_bytes().to_vec(),
            timestamp,
            node_stack: Vec::new(),
        }
    }

    fn node_text(&self, node: TsNode<'_>) -> String {
        node.utf8_text(&self.source)
            .unwrap_or("<invalid utf8>")
            .to_string()
    }
}

impl TomlExtractor {
    pub fn extract_toml(file_path: &str, source: &str) -> ExtractionResult {
        let start = Instant::now();
        let mut state = ExtractionState::new(file_path, source);

        let file_node = Node {
            id: generate_node_id(file_path, &NodeKind::File, file_path, 0),
            kind: NodeKind::File,
            name: file_path.to_string(),
            qualified_name: file_path.to_string(),
            file_path: file_path.to_string(),
            start_line: 0,
            end_line: source.lines().count().saturating_sub(1) as u32,
            start_column: 0,
            end_column: 0,
            signature: None,
            docstring: None,
            visibility: Visibility::Pub,
            is_async: false,
            branches: 0,
            loops: 0,
            returns: 0,
            max_nesting: 0,
            unsafe_blocks: 0,
            unchecked_calls: 0,
            assertions: 0,
            updated_at: state.timestamp,
        };
        let file_node_id = file_node.id.clone();
        state.nodes.push(file_node);
        state
            .node_stack
            .push((file_path.to_string(), file_node_id, 0));

        match Self::parse_source(source) {
            Ok(tree) => {
                let root = tree.root_node();
                Self::visit_children(&mut state, root);
            }
            Err(_msg) => {
                // Parse failed; skip extraction rather than creating a self-loop
            }
        }

        state.node_stack.pop();

        ExtractionResult {
            nodes: state.nodes,
            edges: state.edges,
            unresolved_refs: Vec::new(),
            errors: Vec::new(),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn parse_source(source: &str) -> Result<Tree, String> {
        // TODO(toml): switch to `tokensave_large_treesitters::toml::LANGUAGE`
        // once tokensave-large-treesitters >= 0.3.3 ships the vendored grammar.
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_toml_ng::LANGUAGE.into())
            .map_err(|e| format!("failed to load toml grammar: {e}"))?;
        parser
            .parse(source, None)
            .ok_or_else(|| "tree-sitter parse returned None".to_string())
    }

    fn visit_children(state: &mut ExtractionState, node: TsNode<'_>) {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                Self::visit_node(state, child);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    fn visit_node(state: &mut ExtractionState, node: TsNode<'_>) {
        match node.kind() {
            "table" | "table_array_element" => {
                Self::visit_table(state, node);
            }
            _ => {
                Self::visit_children(state, node);
            }
        }
    }

    fn visit_table(state: &mut ExtractionState, node: TsNode<'_>) {
        let key_node = node
            .children(&mut node.walk())
            .find(|n| matches!(n.kind(), "dotted_key" | "bare_key" | "quoted_key"));

        let Some(key_node) = key_node else {
            return;
        };

        let raw_name = state.node_text(key_node).trim().to_string();
        if raw_name.is_empty() {
            return;
        }

        let level = raw_name.split('.').count();

        while state.node_stack.len() > 1 {
            let last_level = state.node_stack[state.node_stack.len() - 1].2;
            if last_level >= level {
                state.node_stack.pop();
            } else {
                break;
            }
        }

        let kind = NodeKind::Module;
        let parent_name = &state.node_stack[state.node_stack.len() - 1].0;
        let qualified_name = format!("{parent_name}::{raw_name}");
        let id = generate_node_id(
            &state.file_path,
            &kind,
            &raw_name,
            node.start_position().row as u32,
        );

        let signature = if node.kind() == "table_array_element" {
            format!("[[{raw_name}]]")
        } else {
            format!("[{raw_name}]")
        };

        let node_obj = Node {
            id: id.clone(),
            kind,
            name: raw_name.clone(),
            qualified_name,
            file_path: state.file_path.clone(),
            start_line: node.start_position().row as u32,
            end_line: node.end_position().row as u32,
            start_column: node.start_position().column as u32,
            end_column: node.end_position().column as u32,
            signature: Some(signature),
            docstring: None,
            visibility: Visibility::Pub,
            is_async: false,
            branches: 0,
            loops: 0,
            returns: 0,
            max_nesting: 0,
            unsafe_blocks: 0,
            unchecked_calls: 0,
            assertions: 0,
            updated_at: state.timestamp,
        };

        if let Some((_, parent_id, _)) = state.node_stack.last() {
            state.edges.push(Edge {
                source: parent_id.clone(),
                target: id.clone(),
                kind: EdgeKind::Contains,
                line: Some(node.start_position().row as u32),
            });
        }

        state.nodes.push(node_obj);
        state.node_stack.push((raw_name, id, level));
    }
}

impl crate::extraction::LanguageExtractor for TomlExtractor {
    fn extensions(&self) -> &[&str] {
        &["toml"]
    }

    fn language_name(&self) -> &'static str {
        "TOML"
    }

    fn extract(&self, file_path: &str, source: &str) -> ExtractionResult {
        Self::extract_toml(file_path, source)
    }
}
