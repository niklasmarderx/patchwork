use crate::diff::{parse_diff, HunkLine};
use crate::{PatchError, Result};
use std::path::Path;

/// Apply a unified diff string to files on disk.
pub fn apply_patch(diff_str: &str, root: &Path, dry_run: bool) -> Result<Vec<String>> {
    let files = parse_diff(diff_str)?;
    let mut applied = Vec::new();

    for file_diff in &files {
        if file_diff.is_deleted_file {
            let path = root.join(&file_diff.old_path);
            if !dry_run {
                std::fs::remove_file(&path).map_err(PatchError::Io)?;
            }
            applied.push(format!("deleted {}", file_diff.old_path));
            continue;
        }

        if file_diff.is_new_file {
            let path = root.join(&file_diff.new_path);
            let content: String = file_diff
                .hunks
                .iter()
                .flat_map(|h| &h.lines)
                .filter_map(|l| {
                    if let HunkLine::Added(s) = l {
                        Some(format!("{}\n", s))
                    } else {
                        None
                    }
                })
                .collect();
            if !dry_run {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, content)?;
            }
            applied.push(format!("created {}", file_diff.new_path));
            continue;
        }

        let path = root.join(&file_diff.old_path);
        if !path.exists() {
            return Err(PatchError::FileNotFound(file_diff.old_path.clone()));
        }

        let original = std::fs::read_to_string(&path)?;
        let mut result_lines: Vec<String> = original.lines().map(String::from).collect();

        // Apply hunks in reverse order so line numbers stay valid
        let mut hunks = file_diff.hunks.clone();
        hunks.sort_by(|a, b| b.old_start.cmp(&a.old_start));

        for hunk in &hunks {
            let start = hunk.old_start.saturating_sub(1);
            let end = start + hunk.old_count;

            if end > result_lines.len() {
                return Err(PatchError::ApplyFailed(format!(
                    "hunk out of range in {}",
                    file_diff.old_path
                )));
            }

            let new_lines: Vec<String> = hunk
                .lines
                .iter()
                .filter_map(|l| match l {
                    HunkLine::Added(s) | HunkLine::Context(s) => Some(s.clone()),
                    HunkLine::Removed(_) => None,
                })
                .collect();

            result_lines.splice(start..end, new_lines);
        }

        if !dry_run {
            std::fs::write(&path, result_lines.join("\n") + "\n")?;
        }
        applied.push(format!("patched {}", file_diff.old_path));
    }

    Ok(applied)
}
