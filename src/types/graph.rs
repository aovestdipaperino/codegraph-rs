use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::kinds::{EdgeKind, NodeKind, Visibility};

/// A node in the code graph representing a code entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub name: String,
    pub qualified_name: String,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
    pub signature: Option<String>,
    pub docstring: Option<String>,
    pub visibility: Visibility,
    pub is_async: bool,
    /// Number of branching statements (if, match/switch arms, ternary).
    /// 0 for non-function nodes. Cyclomatic complexity = branches + 1.
    pub branches: u32,
    /// Number of loop constructs (for, while, loop).
    pub loops: u32,
    /// Number of early-exit statements (return, break, continue, throw).
    pub returns: u32,
    /// Maximum brace nesting depth within the function body.
    pub max_nesting: u32,
    /// Number of unsafe blocks/statements within the function body.
    pub unsafe_blocks: u32,
    /// Number of unchecked/force-unwrap calls (e.g. `.unwrap()`, `!!`, `.get()` on Optional).
    pub unchecked_calls: u32,
    /// Number of assertion calls (e.g. `assert!`, `assertEquals`, `expect`).
    pub assertions: u32,
    pub updated_at: u64,
}

/// An edge in the code graph representing a relationship between nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
    pub line: Option<u32>,
}

/// Record tracking an indexed file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileRecord {
    pub path: String,
    pub content_hash: String,
    pub size: u64,
    pub modified_at: i64,
    pub indexed_at: i64,
    pub node_count: u32,
}

/// An unresolved reference found during parsing, to be resolved later.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnresolvedRef {
    pub from_node_id: String,
    pub reference_name: String,
    pub reference_kind: EdgeKind,
    pub line: u32,
    pub column: u32,
    pub file_path: String,
}

/// Result of extracting code entities from a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub unresolved_refs: Vec<UnresolvedRef>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

impl ExtractionResult {
    /// Strip nodes with empty names and remove any edges or unresolved refs
    /// that reference their IDs. Tree-sitter can produce empty-name nodes
    /// from complex declarators (especially C/C++); if we skip the node at
    /// insert time but keep its edges, we get FK constraint violations.
    pub fn sanitize(&mut self) {
        let before = self.nodes.len();
        let bad_ids: std::collections::HashSet<String> = self
            .nodes
            .iter()
            .filter(|n| n.name.is_empty())
            .map(|n| n.id.clone())
            .collect();

        if bad_ids.is_empty() {
            return;
        }

        self.nodes.retain(|n| !n.name.is_empty());
        self.edges
            .retain(|e| !bad_ids.contains(&e.source) && !bad_ids.contains(&e.target));
        self.unresolved_refs
            .retain(|r| !bad_ids.contains(&r.from_node_id));

        let removed = before - self.nodes.len();
        if removed > 0 {
            self.errors
                .push(format!("stripped {removed} node(s) with empty names"));
        }
    }
}

/// A subgraph containing a subset of nodes and edges.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Subgraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub roots: Vec<String>,
}

/// Statistics about the code graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: u64,
    pub edge_count: u64,
    pub file_count: u64,
    pub nodes_by_kind: HashMap<String, u64>,
    pub edges_by_kind: HashMap<String, u64>,
    pub db_size_bytes: u64,
    pub last_updated: u64,
    /// Total bytes of all indexed source files.
    pub total_source_bytes: u64,
    /// Number of indexed files per language (e.g. "Rust" -> 42).
    pub files_by_language: HashMap<String, u64>,
    /// Timestamp of the most recent incremental sync (0 if never synced).
    pub last_sync_at: u64,
    /// Timestamp of the most recent full (re)index (0 if never indexed).
    pub last_full_sync_at: u64,
}

/// Result of resolving references in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    pub resolved: Vec<ResolvedRef>,
    pub unresolved: Vec<UnresolvedRef>,
    pub total: usize,
    pub resolved_count: usize,
}

/// A reference that has been resolved to a target node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRef {
    pub original: UnresolvedRef,
    pub target_node_id: String,
    pub confidence: f64,
    pub resolved_by: String,
}

/// Generates a deterministic node ID from file path, kind, name, and line number.
///
/// The ID format is `"kind:32hexchars"` where the hex portion is the first 32
/// characters of the SHA-256 hash of the input components.
pub fn generate_node_id(file_path: &str, kind: &NodeKind, name: &str, line: u32) -> String {
    debug_assert!(
        !name.is_empty(),
        "generate_node_id called with empty name for {file_path}:{line}"
    );
    let input = format!("{}:{}:{}:{}", file_path, kind.as_str(), name, line);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    let hex_str = hex::encode(hash);
    format!("{}:{}", kind.as_str(), &hex_str[..32])
}
