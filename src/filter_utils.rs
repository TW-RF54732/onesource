use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

const BLACKLIST: &[&str] = &[
    //Not in .gitignore:
    ".git",
    ".gitignore",
    //Danger
    ".env",
    //Self
    ".onesourcerc",
    //Others
    ".svn",
    ".hg",
    "node_modules",
    "__pycache__",
    "venv",
    ".venv",
    ".pytest_cache",
    ".tox",
    "target",
    ".idea",
    ".vscode",
    ".DS_Store",
];

pub struct FileFilter {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
    no_blacklist: bool,
}

impl FileFilter {
    /// Determines whether a file path should be kept or discarded.
    ///
    /// # Priority Logic (Exclude-First)
    /// 1. **Exclude first**: If the path matches `exclude`, it is DISCARDED immediately.
    /// 2. **Include second**: If not excluded, the path must match `include` to be KEPT.
    /// 3. **Default**: If `include` is `None` (*), all non-excluded paths are KEPT.
    pub fn new(include: Option<&str>, exclude: Option<&str>, no_blacklist: bool) -> Self {
        Self {
            include: include.and_then(Self::build_set),
            exclude: exclude.and_then(Self::build_set),
            no_blacklist,
        }
    }

    fn build_set(patterns: &str) -> Option<GlobSet> {
        // Use .gitignore logic
        let mut builder = GlobSetBuilder::new();
        let mut has_pattern = false;

        for pattern in patterns.split(',') {
            let p = pattern.trim().replace('\\', "/");

            if p.is_empty() {
                continue;
            }
            let is_simple_name = !p.contains('/') && !p.contains('*') && !p.contains('.');

            if is_simple_name {
                builder.add(Glob::new(&format!("**/{}", p)).unwrap());
                builder.add(Glob::new(&format!("**/{}/**", p)).unwrap());
            } else {
                let final_p = if p.ends_with('/') {
                    format!("{}**", p)
                } else {
                    p
                };
                builder.add(Glob::new(&final_p).unwrap());
            }
            has_pattern = true;
        }

        if has_pattern {
            Some(builder.build().expect("GlobSet compile fail"))
        } else {
            None
        }
    }

    pub fn is_match(&self, path: &Path) -> bool {
        if !self.no_blacklist
            && path.components().any(|c| {
                c.as_os_str()
                    .to_str()
                    .is_some_and(|s| BLACKLIST.contains(&s))
            })
        {
            return false;
        }

        if let Some(ref ex_set) = self.exclude {
            if ex_set.is_match(path) {
                return false;
            }
        }

        match &self.include {
            Some(inc_set) => inc_set.is_match(path),
            None => true,
        }
    }
}
