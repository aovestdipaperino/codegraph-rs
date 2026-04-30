use serde::{Deserialize, Serialize};

/// Result of a single string replacement edit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditResult {
    pub success: bool,
    pub file_path: String,
    pub matched_str: String,
    pub new_str: String,
    pub message: String,
}

/// Result of a multi-string replacement edit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiEditResult {
    pub success: bool,
    pub file_path: String,
    pub applied_count: usize,
    pub message: String,
}

/// Result of an insert-at operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertResult {
    pub success: bool,
    pub file_path: String,
    pub anchor_line: u32,
    pub content: String,
    pub before: bool,
    pub message: String,
}

/// Result of an ast-grep rewrite operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstGrepResult {
    pub success: bool,
    pub file_path: String,
    pub pattern: String,
    pub rewrite: String,
    pub message: String,
}
