use crate::diff::{FileDiff, HunkLine};

pub struct Explanation {
    pub file: String,
    pub summary: String,
    pub details: Vec<String>,
}

/// Produce a human-readable explanation of what a diff does.
pub fn explain_diff(files: &[FileDiff]) -> Vec<Explanation> {
    files.iter().map(explain_file).collect()
}

fn explain_file(file: &FileDiff) -> Explanation {
    let mut details = Vec::new();

    let summary = if file.is_new_file {
        format!("New file — {} line(s) added", file.added_lines())
    } else if file.is_deleted_file {
        format!("File deleted — {} line(s) removed", file.removed_lines())
    } else {
        format!(
            "+{} / -{} line(s) across {} hunk(s)",
            file.added_lines(),
            file.removed_lines(),
            file.hunks.len()
        )
    };

    for (i, hunk) in file.hunks.iter().enumerate() {
        let added: Vec<&str> = hunk
            .lines
            .iter()
            .filter_map(|l| {
                if let HunkLine::Added(s) = l {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect();

        let removed: Vec<&str> = hunk
            .lines
            .iter()
            .filter_map(|l| {
                if let HunkLine::Removed(s) = l {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect();

        let mut hunk_desc = format!("Hunk {} (line {}): ", i + 1, hunk.new_start);

        if added.is_empty() && !removed.is_empty() {
            hunk_desc.push_str(&format!("removes {} line(s)", removed.len()));
        } else if removed.is_empty() && !added.is_empty() {
            hunk_desc.push_str(&format!("adds {} line(s)", added.len()));
        } else {
            hunk_desc.push_str(&format!(
                "replaces {} line(s) with {}",
                removed.len(),
                added.len()
            ));
        }

        // Detect rename/signature changes
        for (r, a) in removed.iter().zip(added.iter()) {
            let r_trim = r.trim();
            let a_trim = a.trim();
            if (r_trim.starts_with("fn ") || r_trim.starts_with("pub fn "))
                && (a_trim.starts_with("fn ") || a_trim.starts_with("pub fn "))
            {
                details.push(format!("  Signature change: `{}` → `{}`", r_trim, a_trim));
            }
        }

        details.push(hunk_desc);
    }

    Explanation {
        file: file.new_path.clone(),
        summary,
        details,
    }
}
