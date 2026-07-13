use std::path::Path;

use anyhow::{Context, Result};
use globset::{Glob, GlobBuilder, GlobMatcher};

pub const BLACKLIST_COMPONENTS: &[&str] = &[
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
    ".npmrc",
    ".pypirc",
    ".netrc",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
];

pub const BLACKLIST_PATTERNS: &[&str] = &[
    ".env.*",
    "credentials.json",
    "service-account*.json",
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "*.onesource",
    ".onesource-tmp-*",
    ".onesource-backup-*",
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
    fn new(patterns: &str) -> Result<Option<Self>> {
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
                matchers.push((p.clone(), compile_glob(&format!("**/{}", p), &p)?));
                matchers.push((p.clone(), compile_glob(&format!("**/{}/**", p), &p)?));
            } else {
                let final_p = if p.ends_with('/') {
                    format!("{}**", p)
                } else {
                    p.clone()
                };
                matchers.push((p.clone(), compile_glob(&final_p, &p)?));
            }
        }

        if raw_patterns.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self {
                patterns: raw_patterns,
                matchers,
            }))
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
    pub fn new(include: Option<&str>, exclude: Option<&str>, no_blacklist: bool) -> Result<Self> {
        Ok(Self {
            include: include.map(PatternSet::new).transpose()?.flatten(),
            exclude: exclude.map(PatternSet::new).transpose()?.flatten(),
            no_blacklist,
        })
    }

    pub fn is_match(&self, path: &Path) -> bool {
        matches!(self.explain(path), FilterDecision::Included)
    }

    pub fn explain(&self, path: &Path) -> FilterDecision {
        if !self.no_blacklist {
            if let Some(rule) = blacklist_rule(path) {
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

fn compile_glob(pattern: &str, original: &str) -> Result<GlobMatcher> {
    Glob::new(pattern)
        .with_context(|| format!("Invalid glob pattern '{}'", original))
        .map(|glob| glob.compile_matcher())
}

fn blacklist_rule(path: &Path) -> Option<String> {
    if let Some(component) = path.components().find_map(|component| {
        let value = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        BLACKLIST_COMPONENTS
            .iter()
            .find(|rule| value == rule.to_ascii_lowercase())
    }) {
        return Some((*component).to_string());
    }

    let normalized = path.to_string_lossy().replace('\\', "/");
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    BLACKLIST_PATTERNS.iter().find_map(|pattern| {
        GlobBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .ok()
            .filter(|glob| {
                let matcher = glob.compile_matcher();
                matcher.is_match(&normalized) || matcher.is_match(Path::new(file_name.as_ref()))
            })
            .map(|_| (*pattern).to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_glob_is_a_user_error_instead_of_a_panic() {
        let error = FileFilter::new(Some("["), None, false).err().unwrap();
        assert!(error.to_string().contains("Invalid glob pattern '['"));
    }

    #[test]
    fn sensitive_patterns_are_case_insensitive() {
        let filter = FileFilter::new(None, None, false).unwrap();
        assert!(matches!(
            filter.explain(Path::new("config/.ENV.PRODUCTION")),
            FilterDecision::BlockedByBlacklist { .. }
        ));
        assert!(matches!(
            filter.explain(Path::new("keys/DEPLOY.PEM")),
            FilterDecision::BlockedByBlacklist { .. }
        ));
        assert!(matches!(
            filter.explain(Path::new(".onesource-tmp-123")),
            FilterDecision::BlockedByBlacklist { .. }
        ));
    }

    #[test]
    fn no_blacklist_disables_sensitive_patterns() {
        let filter = FileFilter::new(None, None, true).unwrap();
        assert_eq!(
            filter.explain(Path::new("context.onesource")),
            FilterDecision::Included
        );
    }
}
