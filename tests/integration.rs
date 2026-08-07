use patchwork::{apply_patch, explain_diff, parse_diff};
use std::fs;
use tempfile::TempDir;

const SIMPLE_DIFF: &str = "diff --git a/hello.rs b/hello.rs\n--- a/hello.rs\n+++ b/hello.rs\n@@ -1,3 +1,3 @@\n fn main() {\n-    println!(\"hello\");\n+    println!(\"hello, world\");\n }\n";

const NEW_FILE_DIFF: &str = "diff --git a/new.rs b/new.rs\n--- /dev/null\n+++ b/new.rs\n@@ -0,0 +1,2 @@\n+fn foo() {}\n+fn bar() {}\n";

const DELETE_FILE_DIFF: &str = "diff --git a/old.rs b/old.rs\n--- a/old.rs\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-fn foo() {}\n-fn bar() {}\n";

#[test]
fn explain_simple_change() {
    let files = parse_diff(SIMPLE_DIFF).unwrap();
    let exps = explain_diff(&files);
    assert_eq!(exps.len(), 1);
    assert!(exps[0].summary.contains("+1"));
    assert!(exps[0].summary.contains("-1"));
}

#[test]
fn explain_new_file() {
    let files = parse_diff(NEW_FILE_DIFF).unwrap();
    let exps = explain_diff(&files);
    assert!(exps[0].summary.contains("New file"));
    assert!(exps[0].summary.contains("2"));
}

#[test]
fn apply_patch_modifies_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("hello.rs");
    fs::write(&path, "fn main() {\n    println!(\"hello\");\n}\n").unwrap();

    let applied = apply_patch(SIMPLE_DIFF, dir.path(), false).unwrap();
    assert_eq!(applied.len(), 1);
    assert!(applied[0].contains("patched"));

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("hello, world"));
}

#[test]
fn apply_patch_creates_new_file() {
    let dir = TempDir::new().unwrap();
    apply_patch(NEW_FILE_DIFF, dir.path(), false).unwrap();
    assert!(dir.path().join("new.rs").exists());
}

#[test]
fn apply_patch_dry_run_no_write() {
    let dir = TempDir::new().unwrap();
    let result = apply_patch(NEW_FILE_DIFF, dir.path(), true).unwrap();
    assert!(!result.is_empty());
    assert!(!dir.path().join("new.rs").exists());
}

#[test]
fn apply_patch_deletes_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("old.rs");
    fs::write(&path, "fn foo() {}\nfn bar() {}\n").unwrap();
    apply_patch(DELETE_FILE_DIFF, dir.path(), false).unwrap();
    assert!(!path.exists());
}
