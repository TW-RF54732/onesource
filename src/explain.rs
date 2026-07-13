use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::configs::AppConfig;
use crate::filter_utils::{FileFilter, FilterDecision};
use crate::scan::{self, SkipReason};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplainDecision {
    Included,
    IncludedWithLossyUtf8,
    NotFound,
    OutsideRoot,
    OperationalOutput,
    NotContentFile,
    DisabledByNoTree,
    BlockedByBlacklist { rule: String },
    BlockedByIgnore,
    BlockedByExclude { rule: String },
    NotIncludedByInclude { rule: String },
    SkippedByMaxSize { max_kib: usize, actual_bytes: u64 },
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
    let root = scan::validate_root(&args.path)?;
    let input_path = if input.is_absolute() {
        input.to_path_buf()
    } else {
        root.join(input)
    };

    if !input_path.exists() {
        return Ok(ExplainReport {
            path: input.to_path_buf(),
            content: Some(ExplainSection {
                decision: ExplainDecision::NotFound,
            }),
            tree: None,
        });
    }

    let full_path = input_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", input.display()))?;
    let output_path = scan::absolute_output_path(&args.output_path)?;
    if scan::same_file_path(&input_path, &output_path) {
        let output = ExplainSection {
            decision: ExplainDecision::OperationalOutput,
        };
        return Ok(ExplainReport {
            path: input.to_path_buf(),
            content: Some(output.clone()),
            tree: Some(output),
        });
    }

    let Ok(canonical_rel_path) = full_path.strip_prefix(&root) else {
        if input_path
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            if let Ok(rel_path) = input_path.strip_prefix(&root) {
                return Ok(ExplainReport {
                    path: input.to_path_buf(),
                    content: Some(ExplainSection {
                        decision: ExplainDecision::OutsideRoot,
                    }),
                    tree: Some(ExplainSection {
                        decision: explain_tree(args, &root, &input_path, rel_path)?,
                    }),
                });
            }
        }
        let outside = ExplainSection {
            decision: ExplainDecision::OutsideRoot,
        };
        return Ok(ExplainReport {
            path: input.to_path_buf(),
            content: Some(outside.clone()),
            tree: Some(outside),
        });
    };
    let rel_path = if input_path
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        input_path.strip_prefix(&root).unwrap_or(canonical_rel_path)
    } else {
        canonical_rel_path
    };

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
    )?;

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

    if !full_path.is_file() {
        return Ok(ExplainDecision::NotContentFile);
    }

    Ok(match scan::inspect_file(full_path, args.max_size) {
        Ok(inspected) if inspected.lossy_utf8 => ExplainDecision::IncludedWithLossyUtf8,
        Ok(_) => ExplainDecision::Included,
        Err(SkipReason::TooLarge {
            max_kib,
            actual_bytes,
        }) => ExplainDecision::SkippedByMaxSize {
            max_kib,
            actual_bytes,
        },
        Err(SkipReason::Binary) => ExplainDecision::SkippedBinary,
        Err(SkipReason::Unreadable(error)) => ExplainDecision::Unreadable { error },
    })
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
    let filter = FileFilter::new(final_include, final_exclude, args.no_blacklist)?;

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
            max_kib,
            actual_bytes,
        } => println!(
            "  Rule    max-size = {} KiB, actual = {} bytes",
            max_kib, actual_bytes
        ),
        ExplainDecision::Unreadable { error } => println!("  Rule    read error = {}", error),
        _ => {}
    }
}

fn decision_text(decision: &ExplainDecision) -> &'static str {
    match decision {
        ExplainDecision::Included => "included",
        ExplainDecision::IncludedWithLossyUtf8 => {
            "included with lossy UTF-8 replacement characters"
        }
        ExplainDecision::NotFound => "not found",
        ExplainDecision::OutsideRoot => "outside the scan root",
        ExplainDecision::OperationalOutput => "excluded because it is the current output file",
        ExplainDecision::NotContentFile => "not a content file (directory or special path)",
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

    #[test]
    fn explains_directories_outside_paths_and_current_output() {
        let dir = temp_dir("special-paths");
        fs::create_dir(dir.join("nested")).unwrap();
        let output = dir.join("out.onesource");
        fs::write(&output, "old output").unwrap();
        let mut args = config(dir.clone());
        args.output_path = output;

        let directory = explain_path(&args, Path::new("nested")).unwrap();
        assert_eq!(
            content_decision(&directory),
            &ExplainDecision::NotContentFile
        );
        assert_eq!(tree_decision(&directory), &ExplainDecision::Included);

        let current_output = explain_path(&args, Path::new("out.onesource")).unwrap();
        assert_eq!(
            content_decision(&current_output),
            &ExplainDecision::OperationalOutput
        );

        let outside_dir = temp_dir("outside-root");
        let outside_path = outside_dir.join("outside.txt");
        fs::write(&outside_path, "outside").unwrap();
        let outside = explain_path(&args, &outside_path).unwrap();
        assert_eq!(content_decision(&outside), &ExplainDecision::OutsideRoot);
    }

    #[test]
    fn explains_lossy_utf8_and_exact_size_limit() {
        let dir = temp_dir("content-details");
        fs::write(dir.join("legacy.txt"), [0xff, b'a']).unwrap();
        fs::write(dir.join("over.txt"), vec![b'x'; 1025]).unwrap();
        let mut args = config(dir);
        args.max_size = 1;

        let lossy = explain_path(&args, Path::new("legacy.txt")).unwrap();
        assert_eq!(
            content_decision(&lossy),
            &ExplainDecision::IncludedWithLossyUtf8
        );
        let over = explain_path(&args, Path::new("over.txt")).unwrap();
        assert_eq!(
            content_decision(&over),
            &ExplainDecision::SkippedByMaxSize {
                max_kib: 1,
                actual_bytes: 1025,
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn explains_outside_symlink_as_content_only_block() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir("symlink-root");
        let outside_dir = temp_dir("symlink-outside");
        let outside = outside_dir.join("secret.txt");
        fs::write(&outside, "secret").unwrap();
        symlink(outside, dir.join("linked.txt")).unwrap();

        let report = explain_path(&config(dir), Path::new("linked.txt")).unwrap();
        assert_eq!(content_decision(&report), &ExplainDecision::OutsideRoot);
        assert_eq!(tree_decision(&report), &ExplainDecision::Included);
    }
}
