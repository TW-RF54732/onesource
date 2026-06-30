use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::configs::AppConfig;
use crate::filter_utils::{FileFilter, FilterDecision};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplainDecision {
    Included,
    NotFound,
    DisabledByNoTree,
    BlockedByBlacklist { rule: String },
    BlockedByIgnore,
    BlockedByExclude { rule: String },
    NotIncludedByInclude { rule: String },
    SkippedByMaxSize { max_size: usize, actual_size: u64 },
    SkippedBinary,
    Unreadable { error: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainSection {
    pub decision: ExplainDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainReport {
    pub path: PathBuf,
    pub content: Option<ExplainSection>,
    pub tree: Option<ExplainSection>,
}

pub fn explain_paths(args: &AppConfig, paths: &[PathBuf]) -> Result<Vec<ExplainReport>> {
    paths.iter().map(|path| explain_path(args, path)).collect()
}

fn explain_path(args: &AppConfig, input: &Path) -> Result<ExplainReport> {
    let full_path = if input.is_absolute() {
        input.to_path_buf()
    } else {
        args.path.join(input)
    };

    if !full_path.exists() {
        return Ok(ExplainReport {
            path: input.to_path_buf(),
            content: Some(ExplainSection {
                decision: ExplainDecision::NotFound,
            }),
            tree: None,
        });
    }

    let root = args
        .path
        .canonicalize()
        .with_context(|| format!("Failed to resolve root path: {}", args.path.display()))?;
    let full_path = full_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", input.display()))?;
    let rel_path = full_path.strip_prefix(&root).unwrap_or(&full_path);

    let content = ExplainSection {
        decision: explain_content(args, &root, &full_path, rel_path)?,
    };
    let tree = ExplainSection {
        decision: explain_tree(args, &root, &full_path, rel_path)?,
    };

    Ok(ExplainReport {
        path: input.to_path_buf(),
        content: Some(content),
        tree: Some(tree),
    })
}

fn explain_content(
    args: &AppConfig,
    root: &Path,
    full_path: &Path,
    rel_path: &Path,
) -> Result<ExplainDecision> {
    let filter = FileFilter::new(
        args.include.as_deref(),
        args.exclude.as_deref(),
        args.no_blacklist,
    );

    let filter_decision = filter.explain(rel_path);
    if let Some(decision) = explain_blacklist_decision(&filter_decision) {
        return Ok(decision);
    }

    if !args.no_ignore && is_blocked_by_ignore(root, full_path)? {
        return Ok(ExplainDecision::BlockedByIgnore);
    }

    if let Some(decision) = explain_filter_decision(filter_decision) {
        return Ok(decision);
    }

    if full_path.is_file() {
        let metadata = full_path
            .metadata()
            .with_context(|| format!("Failed to read metadata: {}", full_path.display()))?;
        let size_kb = metadata.len() / 1024;
        if size_kb > args.max_size as u64 {
            return Ok(ExplainDecision::SkippedByMaxSize {
                max_size: args.max_size,
                actual_size: size_kb,
            });
        }

        match is_text_file(full_path) {
            Ok(true) => {}
            Ok(false) => return Ok(ExplainDecision::SkippedBinary),
            Err(error) => {
                return Ok(ExplainDecision::Unreadable {
                    error: error.to_string(),
                })
            }
        }
    }

    Ok(ExplainDecision::Included)
}

fn explain_tree(
    args: &AppConfig,
    root: &Path,
    full_path: &Path,
    rel_path: &Path,
) -> Result<ExplainDecision> {
    if args.no_tree {
        return Ok(ExplainDecision::DisabledByNoTree);
    }

    let final_include = args.tree_include.as_deref().or(args.include.as_deref());
    let final_exclude = args.tree_exclude.as_deref().or(args.exclude.as_deref());
    let filter = FileFilter::new(final_include, final_exclude, args.no_blacklist);

    let filter_decision = filter.explain(rel_path);
    if let Some(decision) = explain_blacklist_decision(&filter_decision) {
        return Ok(decision);
    }

    if !args.tree_no_ignore && is_blocked_by_ignore(root, full_path)? {
        return Ok(ExplainDecision::BlockedByIgnore);
    }

    if let Some(decision) = explain_filter_decision(filter_decision) {
        return Ok(decision);
    }

    Ok(ExplainDecision::Included)
}

fn explain_filter_decision(decision: FilterDecision) -> Option<ExplainDecision> {
    match decision {
        FilterDecision::Included => None,
        FilterDecision::BlockedByBlacklist { rule } => {
            Some(ExplainDecision::BlockedByBlacklist { rule })
        }
        FilterDecision::BlockedByExclude { rule } => {
            Some(ExplainDecision::BlockedByExclude { rule })
        }
        FilterDecision::NotIncludedByInclude { rule } => {
            Some(ExplainDecision::NotIncludedByInclude { rule })
        }
    }
}

fn explain_blacklist_decision(decision: &FilterDecision) -> Option<ExplainDecision> {
    match decision {
        FilterDecision::BlockedByBlacklist { rule } => {
            Some(ExplainDecision::BlockedByBlacklist { rule: rule.clone() })
        }
        _ => None,
    }
}

fn is_text_file(path: &Path) -> std::io::Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 1024];
    let n = file.read(&mut buffer)?;
    Ok(!buffer[..n].contains(&0))
}

fn is_blocked_by_ignore(root: &Path, full_path: &Path) -> Result<bool> {
    if full_path == root {
        return Ok(false);
    }

    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .hidden(false)
        .require_git(false)
        .build();

    for result in walker {
        let entry = result?;
        if entry.path() == full_path {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn print_reports(reports: &[ExplainReport]) {
    for (index, report) in reports.iter().enumerate() {
        if index > 0 {
            println!();
        }

        println!("----------------");
        println!("{}", report.path.display());
        println!("----------------");
        println!();

        if let Some(content) = &report.content {
            if content.decision == ExplainDecision::NotFound {
                print_decision("Result", &content.decision);
                continue;
            }

            println!("Content");
            print_decision("  Result", &content.decision);
            print_rule(&content.decision);
            println!();
        }

        if let Some(tree) = &report.tree {
            println!("Tree");
            print_decision("  Result", &tree.decision);
            print_rule(&tree.decision);
        }
    }
}

fn print_decision(label: &str, decision: &ExplainDecision) {
    println!("{}  {}", label, decision_text(decision));
}

fn print_rule(decision: &ExplainDecision) {
    match decision {
        ExplainDecision::BlockedByBlacklist { rule } => println!("  Rule    blacklist = {}", rule),
        ExplainDecision::BlockedByExclude { rule } => println!("  Rule    exclude = {}", rule),
        ExplainDecision::NotIncludedByInclude { rule } => println!("  Rule    include = {}", rule),
        ExplainDecision::SkippedByMaxSize {
            max_size,
            actual_size,
        } => println!(
            "  Rule    max-size = {} KB, actual = {} KB",
            max_size, actual_size
        ),
        ExplainDecision::Unreadable { error } => println!("  Rule    read error = {}", error),
        _ => {}
    }
}

fn decision_text(decision: &ExplainDecision) -> &'static str {
    match decision {
        ExplainDecision::Included => "included",
        ExplainDecision::NotFound => "not found",
        ExplainDecision::DisabledByNoTree => "disabled by --no-tree",
        ExplainDecision::BlockedByBlacklist { .. } => "blocked by blacklist",
        ExplainDecision::BlockedByIgnore => "blocked by ignore filters",
        ExplainDecision::BlockedByExclude { .. } => "blocked by exclude",
        ExplainDecision::NotIncludedByInclude { .. } => "not included by include",
        ExplainDecision::SkippedByMaxSize { .. } => "skipped by max-size",
        ExplainDecision::SkippedBinary => "skipped because binary file",
        ExplainDecision::Unreadable { .. } => "skipped because unreadable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock went backwards")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("onesource-explain-{}-{}", test_name, unique));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    fn config(path: PathBuf) -> AppConfig {
        AppConfig {
            path,
            output_path: PathBuf::from("out.onesource"),
            no_ignore: false,
            include: None,
            exclude: None,
            tree_include: None,
            tree_exclude: None,
            no_tree: false,
            tree_no_ignore: false,
            dry_run: false,
            max_size: 500,
            no_blacklist: false,
            copy: false,
        }
    }

    fn content_decision(report: &ExplainReport) -> &ExplainDecision {
        &report.content.as_ref().unwrap().decision
    }

    fn tree_decision(report: &ExplainReport) -> &ExplainDecision {
        &report.tree.as_ref().unwrap().decision
    }

    #[test]
    fn explains_blacklist_and_no_blacklist() {
        let dir = temp_dir("blacklist");
        fs::write(dir.join(".env"), "SECRET=1").unwrap();

        let blocked = explain_path(&config(dir.clone()), Path::new(".env")).unwrap();
        assert_eq!(
            content_decision(&blocked),
            &ExplainDecision::BlockedByBlacklist {
                rule: ".env".to_string()
            }
        );

        let mut args = config(dir);
        args.no_blacklist = true;
        let included = explain_path(&args, Path::new(".env")).unwrap();
        assert_eq!(content_decision(&included), &ExplainDecision::Included);
    }

    #[test]
    fn exclude_wins_over_include() {
        let dir = temp_dir("exclude");
        fs::write(dir.join("main.rs"), "fn main() {}").unwrap();
        let mut args = config(dir);
        args.include = Some("*.rs".to_string());
        args.exclude = Some("main.rs".to_string());

        let report = explain_path(&args, Path::new("main.rs")).unwrap();
        assert_eq!(
            content_decision(&report),
            &ExplainDecision::BlockedByExclude {
                rule: "main.rs".to_string()
            }
        );
    }

    #[test]
    fn explains_content_and_tree_difference() {
        let dir = temp_dir("tree-difference");
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"").unwrap();
        let mut args = config(dir);
        args.include = Some("*.rs".to_string());
        args.tree_include = Some("*.toml".to_string());

        let report = explain_path(&args, Path::new("Cargo.toml")).unwrap();
        assert_eq!(
            content_decision(&report),
            &ExplainDecision::NotIncludedByInclude {
                rule: "*.rs".to_string()
            }
        );
        assert_eq!(tree_decision(&report), &ExplainDecision::Included);
    }

    #[test]
    fn explains_not_found_and_no_tree() {
        let dir = temp_dir("not-found-no-tree");
        let missing = explain_path(&config(dir.clone()), Path::new("missing.rs")).unwrap();
        assert_eq!(content_decision(&missing), &ExplainDecision::NotFound);
        assert!(missing.tree.is_none());

        fs::write(dir.join("main.rs"), "fn main() {}").unwrap();
        let mut args = config(dir);
        args.no_tree = true;
        let report = explain_path(&args, Path::new("main.rs")).unwrap();
        assert_eq!(tree_decision(&report), &ExplainDecision::DisabledByNoTree);
    }

    #[test]
    fn explains_gitignore_blocks() {
        let dir = temp_dir("gitignore");
        fs::write(dir.join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(dir.join("ignored.txt"), "ignore me").unwrap();

        let mut ignored_args = config(dir.clone());
        ignored_args.include = Some("*.rs".to_string());
        let report = explain_path(&ignored_args, Path::new("ignored.txt")).unwrap();
        assert_eq!(content_decision(&report), &ExplainDecision::BlockedByIgnore);

        let mut args = config(dir);
        args.no_ignore = true;
        let included = explain_path(&args, Path::new("ignored.txt")).unwrap();
        assert_eq!(content_decision(&included), &ExplainDecision::Included);
    }
}
