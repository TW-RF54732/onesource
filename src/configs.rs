use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{PathBuf,Path};
use std::fs;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(name = "onesource", author = "lolLeo", version = "2.2.0")]
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
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Save all argument into .onesourcerc(JSON)")]
    pub save: bool,

    #[serde(skip)]
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Ignore the .onesourcerc configuration file")]
    pub no_config: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Disable the hardcoded blacklist (e.g. .git/)")]
    pub no_blacklist: Option<bool>,
    
    #[serde(skip)]
    #[arg(long,short, action = clap::ArgAction::SetTrue, help = "Out put into clipboard. (no file)")]
    pub copy: bool,
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
    pub copy:bool,
}

impl Args {
    pub fn read_config<P:AsRef<Path>>(path:P)->Option<Self>{
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()?
    }
    pub fn save_config<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json_string = serde_json::to_string_pretty(self)?;
        fs::write(path, json_string)?;
        Ok(())
    }
    pub fn merge_saved_config(&mut self, path: &Path) {
        if self.no_config { return; }

        if let Some(config) = Self::read_config(path) {
            println!(".onesourcerc found, using settings.");
            self.output_path = self.output_path.take().or(config.output_path);
            self.no_ignore = self.no_ignore.take().or(config.no_ignore);
            self.include = self.include.take().or(config.include);
            self.exclude = self.exclude.take().or(config.exclude);
            self.tree_include = self.tree_include.take().or(config.tree_include);
            self.tree_exclude = self.tree_exclude.take().or(config.tree_exclude);
            self.no_tree = self.no_tree.take().or(config.no_tree);
            self.tree_no_ignore = self.tree_no_ignore.take().or(config.tree_no_ignore);
            self.max_size = self.max_size.take().or(config.max_size);
            self.no_blacklist = self.no_blacklist.take().or(config.no_blacklist);
            // NOTE: args.path, args.show_arg, args.save, args.dry_run NOT inherit by saved config
        } else {
            println!("No .onesourcerc found, use default settings.");
        }
    }
    pub fn resolve(self) -> AppConfig {
        let target_path = self.path.unwrap_or_else(|| PathBuf::from("."));
        let final_output_path = self.output_path.unwrap_or_else(|| {// Why just read project folder name so complex??
            let folder_name = target_path
                .canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_os_string()))
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".to_string());
                
            PathBuf::from(format!("{}.onesource", folder_name))
        });
        AppConfig {

            // If None is encountered, the final preset value will be assigned.
            path: target_path,
            output_path: final_output_path,
            no_ignore: self.no_ignore.unwrap_or(false),
            max_size: self.max_size.unwrap_or(500),
            no_tree: self.no_tree.unwrap_or(false),
            tree_no_ignore: self.tree_no_ignore.unwrap_or(false),
            no_blacklist: self.no_blacklist.unwrap_or(false),
            
            // These are allowed to be empty (None means no filtering condition is set), so they are directly transferred.
            include: self.include,
            exclude: self.exclude,
            tree_include: self.tree_include,
            tree_exclude: self.tree_exclude,
            
            copy: self.copy,
            dry_run: self.dry_run,
        }
    }
}