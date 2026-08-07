use clap::{Parser, Subcommand};
use colored::Colorize;
use patchwork::{apply_patch, explain_diff, parse_diff};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "patchwork", about = "Understand and apply diffs", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show a colored, human-readable explanation of a diff
    Explain {
        /// Diff file to explain (use - for stdin)
        #[arg(default_value = "-")]
        file: PathBuf,
    },
    /// Show a colored unified diff between two files
    Diff {
        /// Original file
        old: PathBuf,
        /// New file
        new: PathBuf,
    },
    /// Apply a patch file to files on disk
    Apply {
        /// Patch file to apply (use - for stdin)
        #[arg(default_value = "-")]
        file: PathBuf,
        /// Root directory to apply patch in
        #[arg(short, long, default_value = ".")]
        root: PathBuf,
        /// Preview changes without writing to disk
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Explain { file } => cmd_explain(&file),
        Command::Diff { old, new } => cmd_diff(&old, &new),
        Command::Apply {
            file,
            root,
            dry_run,
        } => cmd_apply(&file, &root, dry_run),
    }
}

fn read_input(path: &PathBuf) -> String {
    if path.to_str() == Some("-") {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s).unwrap();
        s
    } else {
        std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("{} {}: {}", "error:".red().bold(), path.display(), e);
            std::process::exit(1);
        })
    }
}

fn cmd_explain(file: &PathBuf) {
    let input = read_input(file);
    let files = parse_diff(&input).unwrap_or_else(|e| {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    });

    if files.is_empty() {
        println!("{}", "No changes found.".dimmed());
        return;
    }

    let explanations = explain_diff(&files);
    for exp in &explanations {
        println!("\n{} {}", "●".cyan().bold(), exp.file.bold());
        println!("  {}", exp.summary.white());
        for detail in &exp.details {
            println!("  {}", detail.dimmed());
        }
    }
}

fn cmd_diff(old: &PathBuf, new: &PathBuf) {
    let old_content = std::fs::read_to_string(old).unwrap_or_else(|e| {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    });
    let new_content = std::fs::read_to_string(new).unwrap_or_else(|e| {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    });

    use similar::{ChangeTag, TextDiff};
    let diff = TextDiff::from_lines(&old_content, &new_content);

    println!(
        "{} {} → {}",
        "diff".bold(),
        old.display().to_string().yellow(),
        new.display().to_string().green()
    );

    for group in diff.grouped_ops(3) {
        for op in &group {
            for change in diff.iter_changes(op) {
                let line: String = match change.tag() {
                    ChangeTag::Delete => {
                        format!("{} {}", "-".red().bold(), change.to_string_lossy().red())
                    }
                    ChangeTag::Insert => format!(
                        "{} {}",
                        "+".green().bold(),
                        change.to_string_lossy().green()
                    ),
                    ChangeTag::Equal => format!("  {}", change.to_string_lossy().dimmed()),
                };
                print!("{}", line);
            }
        }
    }
}

fn cmd_apply(file: &PathBuf, root: &PathBuf, dry_run: bool) {
    let input = read_input(file);

    if dry_run {
        println!("{}", "Dry run — no files will be modified.\n".yellow());
    }

    match apply_patch(&input, root, dry_run) {
        Ok(applied) => {
            for msg in &applied {
                let colored = if msg.starts_with("created") {
                    format!("{} {}", "✚".green().bold(), msg)
                } else if msg.starts_with("deleted") {
                    format!("{} {}", "✖".red().bold(), msg)
                } else {
                    format!("{} {}", "✔".cyan().bold(), msg)
                };
                println!("{}", colored);
            }
            println!(
                "\n{} {} file(s) processed.",
                "Done.".green().bold(),
                applied.len()
            );
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
