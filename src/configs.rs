use clap::{Parser};
use serde::{Serialize, Deserialize};
use std::{path::PathBuf};
use std::{fs, option};
use std::path::Path;


#[derive(Parser,Debug,Serialize, Deserialize)]
#[command(name = "onesource", author = "lolLeo", version = "0.2.0")]
pub struct Args {
    // File setting
    #[serde(skip)]
    #[arg(default_value = ".", help = "The root directory to scan")]
    pub path: PathBuf,

    #[arg(short, long, default_value = "allCode.txt", help = "The output file path")]
    pub output_path: PathBuf,

    // Content setting
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Ignore .gitignore rules when scanning file content")]
    pub no_ignore: bool,

    #[arg(short, long, help = "Comma-separated list of patterns to include (gitignore syntax, e.g. '*.rs,src/')")]
    pub include: Option<String>,

    #[arg(short = 'x', long, help = "Comma-separated list of patterns to exclude (gitignore syntax, e.g. 'target/,*.log')")]
    pub exclude: Option<String>,

    // Tree setting
    #[arg(long, visible_alias = "ti", help = "Custom include patterns for tree view (overrides global include)")]
    pub tree_include: Option<String>,

    #[arg(long, visible_alias = "tx", help = "Custom exclude patterns for tree view (overrides global exclude)")]
    pub tree_exclude: Option<String>,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Disable the directory tree visualization")]
    pub no_tree: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Ignore .gitignore rules specifically for the tree view")]
    pub tree_no_ignore: bool,

    // Behavior setting
    #[serde(skip)]
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Preview mode: List files without generating the output file")]
    pub dry_run: bool,
    
    #[arg(short, long, default_value_t = 500, help = "Max file size in KB")]
    pub max_size: usize,

    #[serde(skip)]
    #[arg(long,action = clap::ArgAction::SetTrue,help = "Show all argument (DEBUG)")]
    pub show_arg:Option<bool>,

    #[serde(skip)]
    #[arg(long,action = clap::ArgAction::SetTrue,help = "Save all argument into .onesourcerc(JSON)")]
    pub save: bool,

    #[serde(skip)]
    #[arg(long,action = clap::ArgAction::SetTrue,help = "Ignore the .onesourcerc configuration file")]
    pub no_config:bool,
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
}