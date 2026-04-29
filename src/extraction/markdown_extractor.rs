/// Markdown source code extractor.
///
/// Parses Markdown source files and emits nodes and edges for the code graph.
/// Uses line-based parsing rather than tree-sitter since markdown is simple.
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{
    generate_node_id, Edge, EdgeKind, ExtractionResult, Node, NodeKind, Visibility,
};

/// Extracts code graph nodes and edges from Markdown source files.
pub struct MarkdownExtractor;

impl MarkdownExtractor {
    /// Extract code graph nodes and edges from a Markdown source file.
    pub fn extract_markdown(file_path: &str, source: &str) -> ExtractionResult {
        let start = std::time::Instant::now();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let errors = Vec::new();

        let mut node_stack: Vec<(String, String)> = Vec::new();

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
            updated_at: timestamp,
        };
        let file_node_id = file_node.id.clone();
        nodes.push(file_node);
        node_stack.push((file_path.to_string(), file_node_id));

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            if let Some((level, title)) = parse_header(trimmed) {
                while node_stack.len() > 1 {
                    let last_name = &node_stack[node_stack.len() - 1].0;
                    if get_header_level(last_name) >= level {
                        node_stack.pop();
                    } else {
                        break;
                    }
                }

                let kind = NodeKind::Module;
                let parent_name = &node_stack[node_stack.len() - 1].0;
                let qualified_name = format!("{parent_name}::{title}");
                let id = generate_node_id(file_path, &kind, title, line_num);

                let node = Node {
                    id: id.clone(),
                    kind,
                    name: title.to_string(),
                    qualified_name: qualified_name.clone(),
                    file_path: file_path.to_string(),
                    start_line: line_num,
                    end_line: line_num,
                    start_column: 0,
                    end_column: line.len() as u32,
                    signature: Some(format!("{} {}", "#".repeat(level), title)),
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
                    updated_at: timestamp,
                };

                if let Some((_, parent_id)) = node_stack.last() {
                    edges.push(Edge {
                        source: parent_id.clone(),
                        target: id.clone(),
                        kind: EdgeKind::Contains,
                        line: Some(line_num),
                    });
                }

                nodes.push(node);
                node_stack.push((title.to_string(), id));
            }

            for (link_text, link_url) in extract_links(line) {
                if link_url.starts_with("http://") || link_url.starts_with("https://") {
                    continue;
                }

                let target_path = link_url.trim_start_matches("file:");
                let target_ext = target_path.rsplit('.').next().unwrap_or("");

                if !is_code_extension(target_ext) {
                    continue;
                }

                let target_id = generate_node_id(target_path, &NodeKind::Use, link_text, 0);

                if let Some((_, parent_id)) = node_stack.last() {
                    edges.push(Edge {
                        source: parent_id.clone(),
                        target: target_id,
                        kind: EdgeKind::Uses,
                        line: Some(line_num),
                    });
                }
            }
        }

        ExtractionResult {
            nodes,
            edges,
            unresolved_refs: Vec::new(),
            errors,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

fn parse_header(line: &str) -> Option<(usize, &str)> {
    if !line.starts_with('#') {
        return None;
    }
    let level = line.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let title = line[level..].trim();
    if title.is_empty() {
        return None;
    }
    Some((level, title))
}

fn extract_links(line: &str) -> Vec<(&str, &str)> {
    let mut links = Vec::new();
    let mut i = 0;
    let bytes = line.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'[' {
            let text_start = i + 1;
            let mut text_end = text_start;
            while text_end < bytes.len() && bytes[text_end] != b']' {
                text_end += 1;
            }
            if text_end >= bytes.len() || bytes[text_end] != b']' {
                i += 1;
                continue;
            }
            if text_end + 1 >= bytes.len() || bytes[text_end + 1] != b'(' {
                i += 1;
                continue;
            }
            let url_start = text_end + 2;
            let mut url_end = url_start;
            while url_end < bytes.len() && bytes[url_end] != b')' {
                url_end += 1;
            }
            if url_end >= bytes.len() || bytes[url_end] != b')' {
                i += 1;
                continue;
            }

            let text = &line[text_start..text_end];
            let url = &line[url_start..url_end];
            links.push((text, url));

            i = url_end + 1;
        } else {
            i += 1;
        }
    }

    links
}

fn get_header_level(name: &str) -> usize {
    name.chars().take_while(|c| *c == '#').count().max(1)
}

fn is_code_extension(ext: &str) -> bool {
    matches!(
        ext,
        "rs"
            | "py"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "go"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "rb"
            | "php"
            | "swift"
            | "kt"
            | "scala"
            | "R"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ps1"
            | "ex"
            | "exs"
            | "erl"
            | "hrl"
            | "fs"
            | "fsx"
            | "ml"
            | "mli"
            | "hs"
            | "lhs"
            | "lua"
            | "pl"
            | "pm"
            | "t"
            | "nix"
            | "yaml"
            | "yml"
            | "toml"
            | "json"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "md"
            | "markdown"
            | "sql"
            | "db"
            | "proto"
            | "v"
            | "vhd"
            | "vhdl"
            | "sage"
            | "sagews"
            | "ipynb"
    )
}

impl crate::extraction::LanguageExtractor for MarkdownExtractor {
    fn extensions(&self) -> &[&str] {
        &["md", "markdown"]
    }

    fn language_name(&self) -> &'static str {
        "Markdown"
    }

    fn extract(&self, file_path: &str, source: &str) -> ExtractionResult {
        Self::extract_markdown(file_path, source)
    }
}