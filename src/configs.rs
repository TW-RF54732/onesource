use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::parser::ValueSource;
use clap::{ArgMatches, Args as ClapArgs, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(name = "onesource", author = "lolLeo", version = "3.4.0")]
pub struct Args {
    // File setting
    #[serde(skip)]
    #[arg(help = "The root directory to scan")]
    pub path: Option<PathBuf>,

    #[arg(short, long, help = "The output file path")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<PathBuf>,

    // Content setting
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Ignore .gitignore rules when scanning file content"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_ignore: Option<bool>,

    #[arg(
        short,
        long,
        help = "Comma-separated list of patterns to include (gitignore syntax, e.g. '*.rs,src/')"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,

    #[arg(
        short = 'x',
        long,
        help = "Comma-separated list of patterns to exclude (gitignore syntax, e.g. 'target/,*.log')"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<String>,

    // Tree setting
    #[arg(
        long,
        visible_alias = "ti",
        help = "Custom include patterns for tree view (overrides global include)"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_include: Option<String>,

    #[arg(
        long,
        visible_alias = "tx",
        help = "Custom exclude patterns for tree view (overrides global exclude)"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_exclude: Option<String>,

    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Disable the directory tree visualization"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_tree: Option<bool>,

    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Ignore .gitignore rules specifically for the tree view"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_no_ignore: Option<bool>,

    // Behavior setting
    #[serde(skip)]
    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Preview mode: List files without generating the output file"
    )]
    pub dry_run: bool,

    #[arg(short, long, help = "Max file size in KB")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<usize>,

    #[serde(skip)]
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Show all argument (DEBUG)"
    )]
    pub show_arg: Option<bool>,

    #[serde(skip)]
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Save this command's explicit options back to the active profile")]
    pub save: bool,

    #[serde(skip)]
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Replace the active profile when saving instead of merging")]
    pub replace: bool,

    #[serde(skip)]
    #[arg(long, help = "Description for the profile")]
    pub desc: Option<String>,

    #[serde(skip)]
    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Ignore the .onesourcerc configuration file"
    )]
    pub no_config: bool,

    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Disable the hardcoded blacklist (e.g. .git/)"
    )]
    pub no_blacklist: Option<bool>,

    #[serde(skip)]
    #[arg(
        long,
        short,
        action = clap::ArgAction::SetTrue,
        help = "Output into clipboard. (no file)"
    )]
    pub copy: bool,

    #[arg(short, long, help = "Load a specific profile")]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
pub enum Commands {
    /// Profile related commands
    Profile {
        #[command(subcommand)]
        subcommand: Box<ProfileSubcommands>,
    },
    /// Download the latest release and replace this executable in place
    Update,
    /// Explain why specific paths are included or blocked
    Explain {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        options: Box<ExplainOptions>,
    },
}

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
pub enum ProfileSubcommands {
    /// List all available profiles
    #[command(alias = "ls")]
    List {
        #[arg(long, help = "Output in JSON format")]
        json: bool,
    },
    /// Show a single profile
    Show {
        profile: String,
        #[arg(long, help = "Output in JSON format")]
        json: bool,
    },
    /// Create a profile
    Create {
        profile: String,
        #[command(flatten)]
        options: ProfileOptions,
    },
    /// Update a profile
    Update {
        profile: String,
        #[arg(long, action = clap::ArgAction::SetTrue, help = "Replace instead of merging")]
        replace: bool,
        #[command(flatten)]
        options: ProfileOptions,
    },
    /// Delete a profile
    #[command(alias = "rm")]
    Delete { profile: String },
    /// Rename a profile
    Rename { old: String, new: String },
    /// Set or update the description of a profile (legacy; use profile update --desc)
    #[command(hide = true)]
    Desc {
        #[arg(help = "The new description")]
        description: String,
        #[arg(short, long, help = "The profile name to update")]
        profile: String,
    },
}

#[derive(ClapArgs, Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExplainOptions {
    #[command(flatten)]
    pub profile_options: ProfileOptions,
    #[arg(short, long, help = "Load a specific profile")]
    pub profile: Option<String>,
    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Ignore the .onesourcerc configuration file"
    )]
    pub no_config: bool,
    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        help = "Preview mode: accepted for parity with normal runs"
    )]
    pub dry_run: bool,
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Accepted for parity with normal runs")]
    pub save: bool,
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Accepted for parity with normal runs")]
    pub replace: bool,
    #[arg(
        long,
        short,
        action = clap::ArgAction::SetTrue,
        help = "Accepted for parity with normal runs"
    )]
    pub copy: bool,
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Accepted for parity with normal runs"
    )]
    pub show_arg: Option<bool>,
}

#[derive(ClapArgs, Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileOptions {
    #[arg(short = 'o', long, help = "The output file path")]
    pub output_path: Option<PathBuf>,
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Ignore .gitignore rules when scanning file content"
    )]
    pub no_ignore: Option<bool>,
    #[arg(short, long, help = "Comma-separated list of patterns to include")]
    pub include: Option<String>,
    #[arg(
        short = 'x',
        long,
        help = "Comma-separated list of patterns to exclude"
    )]
    pub exclude: Option<String>,
    #[arg(
        long,
        visible_alias = "ti",
        help = "Custom include patterns for tree view"
    )]
    pub tree_include: Option<String>,
    #[arg(
        long,
        visible_alias = "tx",
        help = "Custom exclude patterns for tree view"
    )]
    pub tree_exclude: Option<String>,
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Disable the directory tree visualization"
    )]
    pub no_tree: Option<bool>,
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Ignore .gitignore rules specifically for the tree view"
    )]
    pub tree_no_ignore: Option<bool>,
    #[arg(short, long, help = "Max file size in KB")]
    pub max_size: Option<usize>,
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Disable the hardcoded blacklist"
    )]
    pub no_blacklist: Option<bool>,
    #[arg(long, help = "Description for the profile")]
    pub desc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_ignore: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_include: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_exclude: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_tree: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_no_ignore: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_blacklist: Option<bool>,
}

impl ProfileConfig {
    pub fn merge_from(&mut self, update: ProfileConfig) {
        if update.description.is_some() {
            self.description = update.description;
        }
        if update.output_path.is_some() {
            self.output_path = update.output_path;
        }
        if update.no_ignore.is_some() {
            self.no_ignore = update.no_ignore;
        }
        if update.include.is_some() {
            self.include = update.include;
        }
        if update.exclude.is_some() {
            self.exclude = update.exclude;
        }
        if update.tree_include.is_some() {
            self.tree_include = update.tree_include;
        }
        if update.tree_exclude.is_some() {
            self.tree_exclude = update.tree_exclude;
        }
        if update.no_tree.is_some() {
            self.no_tree = update.no_tree;
        }
        if update.tree_no_ignore.is_some() {
            self.tree_no_ignore = update.tree_no_ignore;
        }
        if update.max_size.is_some() {
            self.max_size = update.max_size;
        }
        if update.no_blacklist.is_some() {
            self.no_blacklist = update.no_blacklist;
        }
    }
}

impl ProfileOptions {
    pub fn to_profile_config(&self) -> ProfileConfig {
        ProfileConfig {
            description: self.desc.clone(),
            output_path: self.output_path.clone(),
            no_ignore: self.no_ignore,
            include: self.include.clone(),
            exclude: self.exclude.clone(),
            tree_include: self.tree_include.clone(),
            tree_exclude: self.tree_exclude.clone(),
            no_tree: self.no_tree,
            tree_no_ignore: self.tree_no_ignore,
            max_size: self.max_size,
            no_blacklist: self.no_blacklist,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDocument {
    pub profiles: HashMap<String, ProfileConfig>,
}

impl ConfigDocument {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub path: PathBuf,
    pub output_path: PathBuf,
    pub no_ignore: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub tree_include: Option<String>,
    pub tree_exclude: Option<String>,
    pub no_tree: bool,
    pub tree_no_ignore: bool,
    pub dry_run: bool,
    pub max_size: usize,
    pub no_blacklist: bool,
    pub copy: bool,
}

impl Args {
    fn is_valid_profile_name(profile_name: &str) -> bool {
        !profile_name.is_empty()
            && profile_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    }

    pub fn validate_profile_name(profile_name: &str) -> Result<()> {
        if Self::is_valid_profile_name(profile_name) {
            Ok(())
        } else {
            Err(anyhow!(
                "Invalid profile name '{}'. Use only letters, numbers, '.', '_' and '-'.",
                profile_name
            ))
        }
    }

    fn read_config_or_empty<P: AsRef<Path>>(path: P) -> Result<ConfigDocument> {
        Ok(Self::read_config(path)?.unwrap_or_else(ConfigDocument::new))
    }

    pub fn write_config<P: AsRef<Path>>(path: P, config_doc: &ConfigDocument) -> Result<()> {
        let path = path.as_ref();
        let json_string = serde_json::to_string_pretty(config_doc)
            .context("Failed to serialize configuration to JSON")?;
        fs::write(path, json_string)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }

    pub fn sorted_profiles(config_doc: &ConfigDocument) -> Vec<(&String, &ProfileConfig)> {
        let mut profiles: Vec<_> = config_doc.profiles.iter().collect();
        profiles.sort_by_key(|(name, _)| *name);
        profiles
    }

    pub fn get_profile<'a>(
        config_doc: &'a ConfigDocument,
        profile_name: &str,
    ) -> Result<&'a ProfileConfig> {
        config_doc
            .profiles
            .get(profile_name)
            .ok_or_else(|| anyhow!("Profile '{}' not found in .onesourcerc", profile_name))
    }

    pub fn create_profile<P: AsRef<Path>>(
        path: P,
        profile_name: &str,
        profile: ProfileConfig,
    ) -> Result<()> {
        Self::validate_profile_name(profile_name)?;
        let mut config_doc = Self::read_config_or_empty(&path)?;
        if config_doc.profiles.contains_key(profile_name) {
            return Err(anyhow!("Profile '{}' already exists", profile_name));
        }
        config_doc
            .profiles
            .insert(profile_name.to_string(), profile);
        Self::write_config(path, &config_doc)
    }

    pub fn update_profile<P: AsRef<Path>>(
        path: P,
        profile_name: &str,
        profile: ProfileConfig,
        replace: bool,
    ) -> Result<()> {
        Self::validate_profile_name(profile_name)?;
        let mut config_doc = Self::read_config_or_empty(&path)?;
        let existing = config_doc
            .profiles
            .get_mut(profile_name)
            .ok_or_else(|| anyhow!("Profile '{}' not found in .onesourcerc", profile_name))?;

        if replace {
            *existing = profile;
        } else {
            existing.merge_from(profile);
        }

        Self::write_config(path, &config_doc)
    }

    pub fn upsert_profile<P: AsRef<Path>>(
        path: P,
        profile_name: &str,
        profile: ProfileConfig,
        replace: bool,
    ) -> Result<()> {
        Self::validate_profile_name(profile_name)?;
        let mut config_doc = Self::read_config_or_empty(&path)?;

        if replace {
            config_doc
                .profiles
                .insert(profile_name.to_string(), profile);
        } else {
            config_doc
                .profiles
                .entry(profile_name.to_string())
                .and_modify(|existing| existing.merge_from(profile.clone()))
                .or_insert(profile);
        }

        Self::write_config(path, &config_doc)
    }

    pub fn delete_profile<P: AsRef<Path>>(path: P, profile_name: &str) -> Result<()> {
        Self::validate_profile_name(profile_name)?;
        let mut config_doc = Self::read_config_or_empty(&path)?;
        if config_doc.profiles.remove(profile_name).is_none() {
            return Err(anyhow!(
                "Profile '{}' not found in .onesourcerc",
                profile_name
            ));
        }
        Self::write_config(path, &config_doc)
    }

    pub fn rename_profile<P: AsRef<Path>>(path: P, old: &str, new: &str) -> Result<()> {
        Self::validate_profile_name(old)?;
        Self::validate_profile_name(new)?;
        let mut config_doc = Self::read_config_or_empty(&path)?;
        if config_doc.profiles.contains_key(new) {
            return Err(anyhow!("Profile '{}' already exists", new));
        }
        let profile = config_doc
            .profiles
            .remove(old)
            .ok_or_else(|| anyhow!("Profile '{}' not found in .onesourcerc", old))?;
        config_doc.profiles.insert(new.to_string(), profile);
        Self::write_config(path, &config_doc)
    }

    fn explicit(matches: &ArgMatches, id: &str) -> bool {
        matches
            .value_source(id)
            .is_some_and(|source| source == ValueSource::CommandLine)
    }

    pub fn explicit_profile_config(&self, matches: &ArgMatches) -> ProfileConfig {
        ProfileConfig {
            description: Self::explicit(matches, "desc")
                .then(|| self.desc.clone())
                .flatten(),
            output_path: Self::explicit(matches, "output_path")
                .then(|| self.output_path.clone())
                .flatten(),
            no_ignore: Self::explicit(matches, "no_ignore")
                .then_some(self.no_ignore)
                .flatten(),
            include: Self::explicit(matches, "include")
                .then(|| self.include.clone())
                .flatten(),
            exclude: Self::explicit(matches, "exclude")
                .then(|| self.exclude.clone())
                .flatten(),
            tree_include: Self::explicit(matches, "tree_include")
                .then(|| self.tree_include.clone())
                .flatten(),
            tree_exclude: Self::explicit(matches, "tree_exclude")
                .then(|| self.tree_exclude.clone())
                .flatten(),
            no_tree: Self::explicit(matches, "no_tree")
                .then_some(self.no_tree)
                .flatten(),
            tree_no_ignore: Self::explicit(matches, "tree_no_ignore")
                .then_some(self.tree_no_ignore)
                .flatten(),
            max_size: Self::explicit(matches, "max_size")
                .then_some(self.max_size)
                .flatten(),
            no_blacklist: Self::explicit(matches, "no_blacklist")
                .then_some(self.no_blacklist)
                .flatten(),
        }
    }

    pub fn apply_explain_options(&mut self, options: &ExplainOptions) {
        let profile_options = &options.profile_options;

        if let Some(profile) = &options.profile {
            self.profile = Some(profile.clone());
        }
        if options.no_config {
            self.no_config = true;
        }
        if options.dry_run {
            self.dry_run = true;
        }
        if options.copy {
            self.copy = true;
        }
        if options.show_arg.is_some() {
            self.show_arg = options.show_arg;
        }

        if profile_options.output_path.is_some() {
            self.output_path = profile_options.output_path.clone();
        }
        if profile_options.no_ignore.is_some() {
            self.no_ignore = profile_options.no_ignore;
        }
        if profile_options.include.is_some() {
            self.include = profile_options.include.clone();
        }
        if profile_options.exclude.is_some() {
            self.exclude = profile_options.exclude.clone();
        }
        if profile_options.tree_include.is_some() {
            self.tree_include = profile_options.tree_include.clone();
        }
        if profile_options.tree_exclude.is_some() {
            self.tree_exclude = profile_options.tree_exclude.clone();
        }
        if profile_options.no_tree.is_some() {
            self.no_tree = profile_options.no_tree;
        }
        if profile_options.tree_no_ignore.is_some() {
            self.tree_no_ignore = profile_options.tree_no_ignore;
        }
        if profile_options.max_size.is_some() {
            self.max_size = profile_options.max_size;
        }
        if profile_options.no_blacklist.is_some() {
            self.no_blacklist = profile_options.no_blacklist;
        }
    }

    pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Option<ConfigDocument>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        // Try parsing new format first
        if let Ok(config) = serde_json::from_str::<ConfigDocument>(&content) {
            return Ok(Some(config));
        }

        // Try parsing old format to provide migration warning
        if let Ok(_old_config) = serde_json::from_str::<ProfileConfig>(&content) {
            let msg = format!(
                "\n[ERROR] Incompatible configuration format detected in {}.\n\
                The .onesourcerc file uses an old format. Profiles are now required.\n\
                Please delete or migrate your .onesourcerc to the new structure:\n\
                {{\n  \"profiles\": {{\n    \"default\": {{ ... }}\n  }}\n}}",
                path.display()
            );
            return Err(anyhow!(msg));
        }

        Err(anyhow!(
            "Invalid configuration format in {}. Please fix or delete the file before saving.",
            path.display()
        ))
    }

    pub fn merge_saved_config(&mut self, path: &Path) -> Result<()> {
        if self.no_config {
            return Ok(());
        }

        let profile_name = self.profile.as_deref().unwrap_or("default");

        match Self::read_config(path)? {
            Some(config_doc) => {
                if let Some(profile) = config_doc.profiles.get(profile_name) {
                    println!(".onesourcerc found, using profile: {}", profile_name);
                    self.output_path = self
                        .output_path
                        .take()
                        .or_else(|| profile.output_path.clone());
                    self.no_ignore = self.no_ignore.take().or(profile.no_ignore);
                    self.include = self.include.take().or_else(|| profile.include.clone());
                    self.exclude = self.exclude.take().or_else(|| profile.exclude.clone());
                    self.tree_include = self
                        .tree_include
                        .take()
                        .or_else(|| profile.tree_include.clone());
                    self.tree_exclude = self
                        .tree_exclude
                        .take()
                        .or_else(|| profile.tree_exclude.clone());
                    self.no_tree = self.no_tree.take().or(profile.no_tree);
                    self.tree_no_ignore = self.tree_no_ignore.take().or(profile.tree_no_ignore);
                    self.max_size = self.max_size.take().or(profile.max_size);
                    self.no_blacklist = self.no_blacklist.take().or(profile.no_blacklist);
                } else if self.profile.is_some() {
                    return Err(anyhow!(
                        "Profile '{}' not found in .onesourcerc",
                        profile_name
                    ));
                } else {
                    println!("No 'default' profile in .onesourcerc, using CLI defaults.");
                }
            }
            None => {
                if self.profile.is_some() {
                    return Err(anyhow!(
                        ".onesourcerc not found, but profile '{}' was requested.",
                        profile_name
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn resolve(self) -> AppConfig {
        let target_path = self.path.unwrap_or_else(|| PathBuf::from("."));
        let final_output_path = self.output_path.unwrap_or_else(|| {
            let folder_name = target_path
                .canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_os_string()))
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".to_string());

            PathBuf::from(format!("{}.onesource", folder_name))
        });
        AppConfig {
            path: target_path,
            output_path: final_output_path,
            no_ignore: self.no_ignore.unwrap_or(false),
            max_size: self.max_size.unwrap_or(500),
            no_tree: self.no_tree.unwrap_or(false),
            tree_no_ignore: self.tree_no_ignore.unwrap_or(false),
            no_blacklist: self.no_blacklist.unwrap_or(false),
            include: self.include,
            exclude: self.exclude,
            tree_include: self.tree_include,
            tree_exclude: self.tree_exclude,
            copy: self.copy,
            dry_run: self.dry_run,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock went backwards")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("onesource-{}-{}", test_name, unique));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir.join(".onesourcerc")
    }

    fn profile(include: Option<&str>, exclude: Option<&str>) -> ProfileConfig {
        ProfileConfig {
            include: include.map(str::to_string),
            exclude: exclude.map(str::to_string),
            ..ProfileConfig::default()
        }
    }

    #[test]
    fn create_profile_fails_when_duplicate() {
        let path = temp_config_path("duplicate-create");
        Args::create_profile(&path, "backend", profile(Some("src/backend/**"), None)).unwrap();

        let error = Args::create_profile(&path, "backend", profile(Some("src/frontend/**"), None))
            .unwrap_err();

        assert!(error.to_string().contains("already exists"));
    }

    #[test]
    fn update_profile_merges_unspecified_fields() {
        let path = temp_config_path("merge-update");
        Args::create_profile(
            &path,
            "backend",
            ProfileConfig {
                include: Some("src/backend/**".to_string()),
                max_size: Some(300),
                ..ProfileConfig::default()
            },
        )
        .unwrap();

        Args::update_profile(
            &path,
            "backend",
            ProfileConfig {
                exclude: Some("*.db".to_string()),
                max_size: Some(500),
                ..ProfileConfig::default()
            },
            false,
        )
        .unwrap();

        let config = Args::read_config(&path).unwrap().unwrap();
        let backend = config.profiles.get("backend").unwrap();
        assert_eq!(backend.include.as_deref(), Some("src/backend/**"));
        assert_eq!(backend.exclude.as_deref(), Some("*.db"));
        assert_eq!(backend.max_size, Some(500));
    }

    #[test]
    fn update_profile_replace_removes_unspecified_fields() {
        let path = temp_config_path("replace-update");
        Args::create_profile(
            &path,
            "backend",
            ProfileConfig {
                include: Some("src/backend/**".to_string()),
                exclude: Some("*.db".to_string()),
                ..ProfileConfig::default()
            },
        )
        .unwrap();

        Args::update_profile(&path, "backend", profile(Some("*.py"), None), true).unwrap();

        let config = Args::read_config(&path).unwrap().unwrap();
        let backend = config.profiles.get("backend").unwrap();
        assert_eq!(backend.include.as_deref(), Some("*.py"));
        assert_eq!(backend.exclude, None);
    }

    #[test]
    fn delete_and_rename_profile_cover_failure_cases() {
        let path = temp_config_path("delete-rename");
        Args::create_profile(&path, "old", profile(Some("old/**"), None)).unwrap();
        Args::create_profile(&path, "taken", profile(Some("taken/**"), None)).unwrap();

        let error = Args::rename_profile(&path, "old", "taken").unwrap_err();
        assert!(error.to_string().contains("already exists"));

        Args::rename_profile(&path, "old", "new").unwrap();
        let error = Args::delete_profile(&path, "old").unwrap_err();
        assert!(error.to_string().contains("not found"));

        Args::delete_profile(&path, "new").unwrap();
        let config = Args::read_config(&path).unwrap().unwrap();
        assert!(!config.profiles.contains_key("new"));
    }

    #[test]
    fn invalid_config_is_not_overwritten() {
        let path = temp_config_path("invalid-config");
        fs::write(&path, "{ definitely not json").unwrap();

        let error =
            Args::upsert_profile(&path, "backend", profile(Some("*.rs"), None), false).unwrap_err();
        assert!(
            error.to_string().contains("Failed to write")
                || error.to_string().contains("not json")
                || error.to_string().contains("invalid")
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ definitely not json");
    }

    #[test]
    fn profile_name_validation_rejects_path_like_names() {
        assert!(Args::validate_profile_name("backend.api-1").is_ok());
        assert!(Args::validate_profile_name("backend/api").is_err());
        assert!(Args::validate_profile_name("backend api").is_err());
        assert!(Args::validate_profile_name("").is_err());
    }
}
