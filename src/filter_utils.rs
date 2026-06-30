use std::path::Path;

use globset::{Glob, GlobMatcher};

pub const BLACKLIST: &[&str] = &[
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
    include: Option<PatternSet>,
    exclude: Option<PatternSet>,
    no_blacklist: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterDecision {
    Included,
    BlockedByBlacklist { rule: String },
    BlockedByExclude { rule: String },
    NotIncludedByInclude { rule: String },
}

struct PatternSet {
    patterns: Vec<String>,
    matchers: Vec<(String, GlobMatcher)>,
}

impl PatternSet {
    fn new(patterns: &str) -> Option<Self> {
        let mut raw_patterns = Vec::new();
        let mut matchers = Vec::new();

        for pattern in patterns.split(',') {
            let p = pattern.trim().replace('\\', "/");

            if p.is_empty() {
                continue;
            }

            let is_simple_name = !p.contains('/') && !p.contains('*') && !p.contains('.');
            raw_patterns.push(p.clone());

            if is_simple_name {
                matchers.push((
                    p.clone(),
                    Glob::new(&format!("**/{}", p)).unwrap().compile_matcher(),
                ));
                matchers.push((
                    p.clone(),
                    Glob::new(&format!("**/{}/**", p))
                        .unwrap()
                        .compile_matcher(),
                ));
            } else {
                let final_p = if p.ends_with('/') {
                    format!("{}**", p)
                } else {
                    p.clone()
                };
                matchers.push((p, Glob::new(&final_p).unwrap().compile_matcher()));
            }
        }

        if raw_patterns.is_empty() {
            None
        } else {
            Some(Self {
                patterns: raw_patterns,
                matchers,
            })
        }
    }

    fn matched_rule(&self, path: &Path) -> Option<&str> {
        self.matchers
            .iter()
            .find_map(|(raw, matcher)| matcher.is_match(path).then_some(raw.as_str()))
    }

    fn rules(&self) -> String {
        self.patterns.join(",")
    }
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
            include: include.and_then(PatternSet::new),
            exclude: exclude.and_then(PatternSet::new),
            no_blacklist,
        }
    }

    pub fn is_match(&self, path: &Path) -> bool {
        matches!(self.explain(path), FilterDecision::Included)
    }

    pub fn explain(&self, path: &Path) -> FilterDecision {
        let blacklist_match = path.components().find_map(|c| {
            c.as_os_str()
                .to_str()
                .filter(|s| BLACKLIST.contains(s))
                .map(str::to_string)
        });
        if !self.no_blacklist {
            if let Some(rule) = blacklist_match {
                return FilterDecision::BlockedByBlacklist { rule };
            }
        }

        if let Some(ref ex_set) = self.exclude {
            if let Some(rule) = ex_set.matched_rule(path) {
                return FilterDecision::BlockedByExclude {
                    rule: rule.to_string(),
                };
            }
        }

        match &self.include {
            Some(inc_set) => {
                if inc_set.matched_rule(path).is_some() {
                    FilterDecision::Included
                } else {
                    FilterDecision::NotIncludedByInclude {
                        rule: inc_set.rules(),
                    }
                }
            }
            None => FilterDecision::Included,
        }
    }
}
