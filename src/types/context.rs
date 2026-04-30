use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::graph::{Node, Subgraph};
use super::kinds::{EdgeKind, NodeKind};

/// Output format for CLI results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Markdown,
    Json,
}

/// A block of source code extracted from a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub content: String,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub node_id: Option<String>,
}

/// A search result pairing a node with a relevance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub node: Node,
    pub score: f64,
}

/// Direction for graph traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraversalDirection {
    Outgoing,
    Incoming,
    Both,
}

/// Options controlling graph traversal behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalOptions {
    pub max_depth: u32,
    pub edge_kinds: Option<Vec<EdgeKind>>,
    pub node_kinds: Option<Vec<NodeKind>>,
    pub direction: TraversalDirection,
    pub limit: u32,
    pub include_start: bool,
}

impl Default for TraversalOptions {
    fn default() -> Self {
        TraversalOptions {
            max_depth: 3,
            edge_kinds: None,
            node_kinds: None,
            direction: TraversalDirection::Outgoing,
            limit: 100,
            include_start: true,
        }
    }
}

/// Options for building an LLM context from the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildContextOptions {
    pub max_nodes: usize,
    pub max_code_blocks: usize,
    pub max_code_block_size: usize,
    pub include_code: bool,
    pub format: OutputFormat,
    pub search_limit: usize,
    pub traversal_depth: usize,
    pub min_score: f64,
    /// Additional keywords to search for beyond those extracted from the query.
    /// Enables agent-driven synonym expansion (e.g. `"authentication"` → `["login", "session"]`).
    pub extra_keywords: Vec<String>,
    /// Node IDs to exclude from results (for session deduplication across calls).
    pub exclude_node_ids: HashSet<String>,
    /// When true, merge code blocks from the same file whose line ranges are
    /// adjacent or overlapping into a single block.
    pub merge_adjacent: bool,
    /// Maximum symbols from a single file in context results. Prevents one
    /// large file from dominating the output. `None` means no cap (defaults
    /// to `max_nodes`).
    pub max_per_file: Option<usize>,
    /// When set, only nodes whose `file_path` starts with this prefix are
    /// considered as entry points. Graph expansion may still traverse outside
    /// the prefix (traversals are unscoped).
    pub path_prefix: Option<String>,
}

impl Default for BuildContextOptions {
    fn default() -> Self {
        BuildContextOptions {
            max_nodes: 20,
            max_code_blocks: 5,
            max_code_block_size: 1500,
            include_code: true,
            format: OutputFormat::Markdown,
            search_limit: 3,
            traversal_depth: 1,
            min_score: 0.0,
            extra_keywords: Vec::new(),
            exclude_node_ids: HashSet::new(),
            merge_adjacent: false,
            max_per_file: None,
            path_prefix: None,
        }
    }
}

/// Context assembled for a task, combining graph data with code blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub query: String,
    pub summary: String,
    pub subgraph: Subgraph,
    pub entry_points: Vec<Node>,
    pub code_blocks: Vec<CodeBlock>,
    pub related_files: Vec<String>,
    /// IDs of all nodes returned as entry points (pass to next call's `exclude_node_ids` for dedup).
    pub seen_node_ids: Vec<String>,
}
