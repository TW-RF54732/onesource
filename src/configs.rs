use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::fs;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(name = "onesource", author = "lolLeo", version = "3.0.0")]
pub struct Args {
    // File setting
    #[serde(skip)]
    #[arg(help = "The root directory to scan")]
    pub path: Option<PathBuf>,

    #[arg(short, long, help = "The output file path")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<PathBuf>,

    // Content setting
    #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Ignore .gitignore rules when scanning file content")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_ignore: Option<bool>,

    #[arg(short, long, help = "Comma-separated list of patterns to include (gitignore syntax, e.g. '*.rs,src/')")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,

    #[arg(short = 'x', long, help = "Comma-separated list of patterns to exclude (gitignore syntax, e.g. 'target/,*.log')")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<String>,

    // Tree setting
    #[arg(long, visible_alias = "ti", help = "Custom include patterns for tree view (overrides global include)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_include: Option<String>,

    #[arg(long, visible_alias = "tx", help = "Custom exclude patterns for tree view (overrides global exclude)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_exclude: Option<String>,

    #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Disable the directory tree visualization")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_tree: Option<bool>,

    #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Ignore .gitignore rules specifically for the tree view")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_no_ignore: Option<bool>,

    // Behavior setting
    #[serde(skip)]
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Preview mode: List files without generating the output file")]
    pub dry_run: bool,

    #[arg(short, long, help = "Max file size in KB")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<usize>,

    #[serde(skip)]
    #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Show all argument (DEBUG)")]
    pub show_arg: Option<bool>,

    #[serde(skip)]
    #[arg(long, default_missing_value = "default", num_args = 0..=1, help = "Save all argument into .onesourcerc (JSON) under specified profile")]
    pub save: Option<String>,

    #[serde(skip)]
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Ignore the .onesourcerc configuration file")]
    pub no_config: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Disable the hardcoded blacklist (e.g. .git/)")]
    pub no_blacklist: Option<bool>,
    
    #[serde(skip)]
    #[arg(long, short, action = clap::ArgAction::SetTrue, help = "Output into clipboard. (no file)")]
    pub copy: bool,

    #[arg(short, long, help = "Load a specific profile")]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum Commands {
    /// Profile related commands
    Profile {
        #[command(subcommand)]
        subcommand: ProfileSubcommands,
    },
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum ProfileSubcommands {
    /// List all available profiles
    Ls {
        #[arg(long, help = "Output in JSON format")]
        json: bool,
    },
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDocument {
    pub profiles: HashMap<String, ProfileConfig>,
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
    pub fn read_config<P: AsRef<Path>>(path: P) -> Option<ConfigDocument> {
        let content = fs::read_to_string(path).ok()?;
        
        // Try parsing new format first
        if let Ok(config) = serde_json::from_str::<ConfigDocument>(&content) {
            return Some(config);
        }

        // Try parsing old format to provide migration warning
        if let Ok(_old_config) = serde_json::from_str::<ProfileConfig>(&content) {
            eprintln!("\n[ERROR] Incompatible configuration format detected.");
            eprintln!("The .onesourcerc file uses an old format. Profiles are now required.");
            eprintln!("Please delete or migrate your .onesourcerc to the new structure:");
            eprintln!("{{\n  \"profiles\": {{\n    \"default\": {{ ... }}\n  }}\n}}");
            std::process::exit(1);
        }

        None
    }

    pub fn save_config<P: AsRef<Path>>(&self, path: P, profile_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut config_doc = Self::read_config(&path).unwrap_or_else(|| ConfigDocument {
            profiles: HashMap::new(),
        });

        let profile = ProfileConfig {
            description: None, // Keep existing description if we were updating, but for now we just create/overwrite fields
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
        };

        // If updating an existing profile, we might want to preserve the description.
        let mut final_profile = profile;
        if let Some(existing) = config_doc.profiles.get(profile_name) {
            final_profile.description = existing.description.clone();
        }

        config_doc.profiles.insert(profile_name.to_string(), final_profile);

        let json_string = serde_json::to_string_pretty(&config_doc)?;
        fs::write(path, json_string)?;
        Ok(())
    }

    pub fn merge_saved_config(&mut self, path: &Path) {
        if self.no_config { return; }

        let profile_name = self.profile.as_deref().unwrap_or("default");
        
        if let Some(config_doc) = Self::read_config(path) {
            if let Some(profile) = config_doc.profiles.get(profile_name) {
                println!(".onesourcerc found, using profile: {}", profile_name);
                self.output_path = self.output_path.take().or(profile.output_path.clone());
                self.no_ignore = self.no_ignore.take().or(profile.no_ignore);
                self.include = self.include.take().or(profile.include.clone());
                self.exclude = self.exclude.take().or(profile.exclude.clone());
                self.tree_include = self.tree_include.take().or(profile.tree_include.clone());
                self.tree_exclude = self.tree_exclude.take().or(profile.tree_exclude.clone());
                self.no_tree = self.no_tree.take().or(profile.no_tree);
                self.tree_no_ignore = self.tree_no_ignore.take().or(profile.tree_no_ignore);
                self.max_size = self.max_size.take().or(profile.max_size);
                self.no_blacklist = self.no_blacklist.take().or(profile.no_blacklist);
            } else if self.profile.is_some() {
                eprintln!("[WARNING] Profile '{}' not found in .onesourcerc. Using default settings.", profile_name);
            } else {
                 println!("No 'default' profile in .onesourcerc, using CLI defaults.");
            }
        } else if self.profile.is_some() {
             eprintln!("[ERROR] .onesourcerc not found, but profile '{}' was requested.", profile_name);
             std::process::exit(1);
        } else {
            // No config file and no profile requested - this is fine, just use defaults.
        }
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