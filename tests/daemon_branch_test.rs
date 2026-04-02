use std::path::PathBuf;

#[test]
fn parse_git_head_branch() {
    assert_eq!(
        tokensave::daemon::parse_head_branch("ref: refs/heads/main"),
        Some("main".to_string())
    );
    assert_eq!(
        tokensave::daemon::parse_head_branch("ref: refs/heads/feature/foo-bar"),
        Some("feature/foo-bar".to_string())
    );
}

#[test]
fn parse_git_head_detached() {
    assert_eq!(
        tokensave::daemon::parse_head_branch("abc123def456"),
        None
    );
}

#[test]
fn sanitize_branch_name() {
    assert_eq!(tokensave::daemon::sanitize_branch("main"), "main");
    assert_eq!(tokensave::daemon::sanitize_branch("feature/foo"), "feature--foo");
    assert_eq!(tokensave::daemon::sanitize_branch("feature/deep/nest"), "feature--deep--nest");
    assert_eq!(tokensave::daemon::sanitize_branch(".hidden"), "_hidden");
}

#[test]
fn resolve_db_path_main() {
    let ts_dir = PathBuf::from("/project/.tokensave");
    assert_eq!(
        tokensave::daemon::resolve_branch_db_path(&ts_dir, "main"),
        ts_dir.join("tokensave.db")
    );
    assert_eq!(
        tokensave::daemon::resolve_branch_db_path(&ts_dir, "master"),
        ts_dir.join("tokensave.db")
    );
}

#[test]
fn resolve_db_path_feature_branch() {
    let ts_dir = PathBuf::from("/project/.tokensave");
    assert_eq!(
        tokensave::daemon::resolve_branch_db_path(&ts_dir, "feature/foo"),
        ts_dir.join("branches/feature--foo.db")
    );
}
