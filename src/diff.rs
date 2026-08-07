use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum HunkLine {
    Context(String),
    Added(String),
    Removed(String),
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<HunkLine>,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub hunks: Vec<Hunk>,
    pub is_new_file: bool,
    pub is_deleted_file: bool,
}

impl FileDiff {
    pub fn added_lines(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| matches!(l, HunkLine::Added(_)))
            .count()
    }

    pub fn removed_lines(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| matches!(l, HunkLine::Removed(_)))
            .count()
    }
}

/// Parse a unified diff string into structured FileDiffs.
pub fn parse_diff(input: &str) -> Result<Vec<FileDiff>> {
    let mut files = Vec::new();
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        if !line.starts_with("diff ") {
            continue;
        }

        // Parse --- and +++ headers
        let old_path = lines
            .next()
            .and_then(|l| l.strip_prefix("--- "))
            .map(|p| p.trim_start_matches("a/").to_string())
            .unwrap_or_default();

        let new_path = lines
            .next()
            .and_then(|l| l.strip_prefix("+++ "))
            .map(|p| p.trim_start_matches("b/").to_string())
            .unwrap_or_default();

        let is_new_file = old_path == "/dev/null";
        let is_deleted_file = new_path == "/dev/null";

        let display_path = if is_new_file {
            new_path.clone()
        } else {
            old_path.clone()
        };

        let mut hunks = Vec::new();

        while lines.peek().map_or(false, |l| !l.starts_with("diff ")) {
            let hunk_header = match lines.next() {
                Some(l) if l.starts_with("@@") => l,
                _ => continue,
            };

            let (old_start, old_count, new_start, new_count) =
                parse_hunk_header(hunk_header).unwrap_or((1, 0, 1, 0));

            let mut hunk_lines = Vec::new();

            while lines
                .peek()
                .map_or(false, |l| !l.starts_with("@@") && !l.starts_with("diff "))
            {
                let l = lines.next().unwrap();
                let hunk_line = if let Some(rest) = l.strip_prefix('+') {
                    HunkLine::Added(rest.to_string())
                } else if let Some(rest) = l.strip_prefix('-') {
                    HunkLine::Removed(rest.to_string())
                } else {
                    HunkLine::Context(l.trim_start_matches(' ').to_string())
                };
                hunk_lines.push(hunk_line);
            }

            hunks.push(Hunk {
                old_start,
                old_count,
                new_start,
                new_count,
                lines: hunk_lines,
            });
        }

        files.push(FileDiff {
            old_path: display_path.clone(),
            new_path: display_path,
            hunks,
            is_new_file,
            is_deleted_file,
        });
    }

    Ok(files)
}

fn parse_hunk_header(header: &str) -> Option<(usize, usize, usize, usize)> {
    // @@ -old_start,old_count +new_start,new_count @@
    let inner = header.strip_prefix("@@ ")?.split(" @@").next()?;
    let mut parts = inner.split_whitespace();
    let old = parts.next()?.strip_prefix('-')?;
    let new = parts.next()?.strip_prefix('+')?;

    let (os, oc) = parse_range(old);
    let (ns, nc) = parse_range(new);
    Some((os, oc, ns, nc))
}

fn parse_range(s: &str) -> (usize, usize) {
    if let Some((start, count)) = s.split_once(',') {
        (start.parse().unwrap_or(1), count.parse().unwrap_or(0))
    } else {
        (s.parse().unwrap_or(1), 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_DIFF: &str = r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,4 +1,4 @@
 fn main() {
-    println!("hello");
+    println!("hello, world");
 }
"#;

    #[test]
    fn test_parse_simple_diff() {
        let files = parse_diff(SIMPLE_DIFF).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].added_lines(), 1);
        assert_eq!(files[0].removed_lines(), 1);
    }

    #[test]
    fn test_new_file() {
        let diff = "diff --git a/new.rs b/new.rs\n--- /dev/null\n+++ b/new.rs\n@@ -0,0 +1,2 @@\n+fn foo() {}\n+fn bar() {}\n";
        let files = parse_diff(diff).unwrap();
        assert!(files[0].is_new_file);
        assert_eq!(files[0].added_lines(), 2);
    }
}
