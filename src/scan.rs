use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use ignore::WalkBuilder;

use crate::configs::AppConfig;
use crate::filter_utils::FileFilter;
use crate::tree_utils::Node;

#[derive(Debug)]
pub struct Candidate {
    pub full_path: PathBuf,
    pub rel_path: PathBuf,
}

#[derive(Debug)]
pub struct ScanSelection {
    pub output_path: PathBuf,
    pub tree: Option<String>,
    pub candidates: Vec<Candidate>,
    pub walk_errors: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    TooLarge { max_kib: usize, actual_bytes: u64 },
    Binary,
    Unreadable(String),
}

#[derive(Debug)]
pub struct InspectedFile {
    pub content: String,
    pub lossy_utf8: bool,
}

pub fn validate_root(path: &Path) -> Result<PathBuf> {
    let root = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve scan root: {}", path.display()))?;
    if !root.is_dir() {
        return Err(anyhow!("Scan path must be a directory: {}", path.display()));
    }
    Ok(root)
}

pub fn absolute_output_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("Failed to resolve current directory")?
            .join(path))
    }
}

pub fn build_selection(args: &AppConfig) -> Result<ScanSelection> {
    let root = validate_root(&args.path)?;
    let output_path = absolute_output_path(&args.output_path)?;
    let content_filter = FileFilter::new(
        args.include.as_deref(),
        args.exclude.as_deref(),
        args.no_blacklist,
    )?;
    let tree_filter = FileFilter::new(
        args.tree_include.as_deref().or(args.include.as_deref()),
        args.tree_exclude.as_deref().or(args.exclude.as_deref()),
        args.no_blacklist,
    )?;

    let (candidates, content_errors) = collect_content(args, &root, &output_path, &content_filter);
    let (tree, tree_errors) = if args.no_tree {
        (None, 0)
    } else {
        let (tree, errors) = collect_tree(args, &root, &output_path, &tree_filter)?;
        (Some(tree), errors)
    };

    Ok(ScanSelection {
        output_path,
        tree,
        candidates,
        walk_errors: content_errors + tree_errors,
    })
}

fn collect_content(
    args: &AppConfig,
    root: &Path,
    output_path: &Path,
    filter: &FileFilter,
) -> (Vec<Candidate>, usize) {
    let mut candidates = Vec::new();
    let mut errors = 0;
    let walker = WalkBuilder::new(root)
        .standard_filters(!args.no_ignore)
        .hidden(false)
        .require_git(false)
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                if same_file_path(entry.path(), output_path) {
                    continue;
                }
                let rel_path = entry.path().strip_prefix(root).unwrap_or(entry.path());
                if !filter.is_match(rel_path) {
                    continue;
                }
                if entry.file_type().is_some_and(|kind| kind.is_dir()) {
                    continue;
                }
                if entry.file_type().is_some_and(|kind| kind.is_symlink()) {
                    match entry.path().canonicalize() {
                        Ok(target) if target.starts_with(root) => {}
                        Ok(target) => {
                            errors += 1;
                            eprintln!(
                                "[WARNING] Skipping symlink outside scan root: {} -> {}",
                                rel_path.display(),
                                target.display()
                            );
                            continue;
                        }
                        Err(error) => {
                            errors += 1;
                            eprintln!(
                                "[WARNING] Skipping unreadable symlink {}: {}",
                                rel_path.display(),
                                error
                            );
                            continue;
                        }
                    }
                }
                candidates.push(Candidate {
                    full_path: entry.path().to_path_buf(),
                    rel_path: rel_path.to_path_buf(),
                });
            }
            Err(error) => {
                errors += 1;
                eprintln!("[WARNING] Failed while scanning content: {}", error);
            }
        }
    }

    candidates.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    (candidates, errors)
}

fn collect_tree(
    args: &AppConfig,
    root: &Path,
    output_path: &Path,
    filter: &FileFilter,
) -> Result<(String, usize)> {
    let mut tree_root = Node::new(true);
    let mut errors = 0;
    let walker = WalkBuilder::new(root)
        .standard_filters(!args.tree_no_ignore)
        .hidden(false)
        .require_git(false)
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                if same_file_path(entry.path(), output_path) {
                    continue;
                }
                let rel_path = entry.path().strip_prefix(root).unwrap_or(entry.path());
                if !filter.is_match(rel_path) {
                    continue;
                }
                let is_dir = entry.file_type().is_some_and(|kind| kind.is_dir());
                tree_root.insert_path(rel_path, is_dir);
            }
            Err(error) => {
                errors += 1;
                eprintln!("[WARNING] Failed while scanning tree: {}", error);
            }
        }
    }

    let root_name = root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    let mut bytes = Vec::new();
    use std::io::Write;
    writeln!(&mut bytes, "{}/", root_name)?;
    tree_root.print("", &mut bytes)?;
    let tree = String::from_utf8(bytes).context("Generated tree was not valid UTF-8")?;
    Ok((tree, errors))
}

pub fn inspect_file(path: &Path, max_kib: usize) -> std::result::Result<InspectedFile, SkipReason> {
    let metadata = path
        .metadata()
        .map_err(|error| SkipReason::Unreadable(format!("metadata: {}", error)))?;
    let max_bytes = (max_kib as u64).saturating_mul(1024);
    if metadata.len() > max_bytes {
        return Err(SkipReason::TooLarge {
            max_kib,
            actual_bytes: metadata.len(),
        });
    }

    let bytes =
        std::fs::read(path).map_err(|error| SkipReason::Unreadable(format!("read: {}", error)))?;
    if bytes.contains(&0) {
        return Err(SkipReason::Binary);
    }

    match String::from_utf8(bytes) {
        Ok(content) => Ok(InspectedFile {
            content,
            lossy_utf8: false,
        }),
        Err(error) => Ok(InspectedFile {
            content: String::from_utf8_lossy(error.as_bytes()).into_owned(),
            lossy_utf8: true,
        }),
    }
}

pub fn same_file_path(path: &Path, output_path: &Path) -> bool {
    if path == output_path {
        return true;
    }
    match (path.canonicalize(), output_path.canonicalize()) {
        (Ok(path), Ok(output)) => path == output,
        _ => false,
    }
}

pub fn escape_path_attribute(path: &Path) -> String {
    let mut escaped = String::new();
    for character in path.to_string_lossy().chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value.is_control() => {
                escaped.push_str(&format!("\\u{{{:X}}}", value as u32));
            }
            value => escaped.push(value),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("onesource-scan-{}-{}", name, unique));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn config(path: PathBuf, output_path: PathBuf) -> AppConfig {
        AppConfig {
            path,
            output_path,
            no_ignore: false,
            include: None,
            exclude: None,
            tree_include: None,
            tree_exclude: None,
            no_tree: false,
            tree_no_ignore: false,
            dry_run: false,
            max_size: 500,
            no_blacklist: true,
            copy: false,
        }
    }

    #[test]
    fn max_size_is_an_exact_byte_boundary() {
        let dir = temp_dir("size");
        let exact = dir.join("exact.txt");
        let over = dir.join("over.txt");
        fs::write(&exact, vec![b'x'; 1024]).unwrap();
        fs::write(&over, vec![b'x'; 1025]).unwrap();
        assert!(inspect_file(&exact, 1).is_ok());
        assert!(matches!(
            inspect_file(&over, 1),
            Err(SkipReason::TooLarge {
                actual_bytes: 1025,
                ..
            })
        ));
    }

    #[test]
    fn nul_after_first_kib_is_binary() {
        let dir = temp_dir("binary");
        let path = dir.join("binary.dat");
        let mut bytes = vec![b'x'; 2048];
        bytes[1500] = 0;
        fs::write(&path, bytes).unwrap();
        assert!(matches!(inspect_file(&path, 10), Err(SkipReason::Binary)));
    }

    #[test]
    fn invalid_utf8_is_included_lossily() {
        let dir = temp_dir("lossy");
        let path = dir.join("legacy.txt");
        fs::write(&path, [0xff, b'a']).unwrap();
        let inspected = inspect_file(&path, 10).unwrap();
        assert!(inspected.lossy_utf8);
        assert!(inspected.content.contains('\u{fffd}'));
    }

    #[test]
    fn path_attributes_are_escaped() {
        assert_eq!(
            escape_path_attribute(Path::new("a&\"<b>\n.rs")),
            "a&amp;&quot;&lt;b&gt;\\n.rs"
        );
    }

    #[test]
    fn selection_is_sorted_and_always_excludes_current_output() {
        let dir = temp_dir("selection");
        let output = dir.join("result.onesource");
        fs::write(dir.join("z.txt"), "z").unwrap();
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(&output, "previous output").unwrap();

        let selection = build_selection(&config(dir.clone(), output.clone())).unwrap();
        let selected: Vec<_> = selection
            .candidates
            .iter()
            .map(|candidate| candidate.rel_path.clone())
            .collect();
        assert_eq!(
            selected,
            vec![PathBuf::from("a.txt"), PathBuf::from("z.txt")]
        );
        assert!(!selection.tree.unwrap().contains("result.onesource"));
    }

    #[cfg(unix)]
    #[test]
    fn selection_does_not_follow_content_symlinks_outside_root() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir("outside-symlink");
        let outside_dir = temp_dir("outside-target");
        let outside = outside_dir.join("secret.txt");
        fs::write(&outside, "secret").unwrap();
        symlink(&outside, dir.join("linked.txt")).unwrap();

        let selection =
            build_selection(&config(dir.clone(), dir.join("result.onesource"))).unwrap();
        assert!(selection.candidates.is_empty());
        assert_eq!(selection.walk_errors, 1);
    }
}
