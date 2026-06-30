mod configs;
mod filter_utils;
mod io_utils;
mod tree_utils;

use std::fs::File;
use std::io::{BufWriter, Write};

use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches};
use ignore::WalkBuilder;
use tiktoken_rs::cl100k_base;

use crate::configs::{AppConfig, Args, ProfileConfig};

#[derive(Debug, Default)]
struct FileStats {
    file_count: u32,
    total_tokens: usize,
}

fn generate_tree<W: Write>(args: &AppConfig, writer: &mut W) -> Result<()> {
    let final_include = args.tree_include.as_deref().or(args.include.as_deref());
    let final_exclude = args.tree_exclude.as_deref().or(args.exclude.as_deref());
    let filter = filter_utils::FileFilter::new(final_include, final_exclude, args.no_blacklist);
    let mut tree_root = tree_utils::Node::new(true);

    let walker = WalkBuilder::new(&args.path)
        .standard_filters(!args.tree_no_ignore)
        .hidden(false)
        .require_git(false)
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                let rel_path = entry
                    .path()
                    .strip_prefix(&args.path)
                    .unwrap_or(entry.path());
                if !filter.is_match(rel_path) {
                    continue;
                }
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                tree_root.insert_path(rel_path, is_dir);
            }
            Err(error) => {
                eprintln!("Error walking tree: {}", error);
            }
        }
    }
    let root_name = args
        .path
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| ".".into());

    writeln!(writer, "{}/", root_name).context("Failed to write root name to tree")?;
    tree_root
        .print("", writer)
        .context("Failed to print tree")?;
    Ok(())
}

fn process_files<W: Write>(args: &AppConfig, writer: &mut W) -> Result<FileStats> {
    let bpe = cl100k_base().context("Failed to load tokenizer")?;
    let filter = filter_utils::FileFilter::new(
        args.include.as_deref(),
        args.exclude.as_deref(),
        args.no_blacklist,
    );
    let walker = WalkBuilder::new(&args.path)
        .standard_filters(!args.no_ignore)
        .hidden(false)
        .require_git(false)
        .build();
    let mut stats = FileStats::default();

    for result in walker {
        match result {
            Ok(entry) => {
                let rel_path = entry
                    .path()
                    .strip_prefix(&args.path)
                    .unwrap_or(entry.path());
                if !filter.is_match(rel_path) {
                    continue;
                }

                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                if is_dir {
                    continue;
                }

                let metadata = entry.metadata().ok();
                let size_kb = metadata.map(|m| m.len() / 1024).unwrap_or(0);
                if size_kb > args.max_size as u64 {
                    continue;
                }

                let is_text_file = if let Ok(mut f) = File::open(entry.path()) {
                    use std::io::Read;
                    let mut buffer = [0; 1024];
                    let n = f.read(&mut buffer).unwrap_or(0);
                    !buffer[..n].contains(&0)
                } else {
                    false
                };
                if !is_text_file {
                    continue;
                }

                if let Ok(bytes) = std::fs::read(entry.path()) {
                    let content = String::from_utf8_lossy(&bytes);
                    let tokens = bpe.encode_with_special_tokens(&content);
                    let token_count = tokens.len();
                    stats.total_tokens += token_count;
                    stats.file_count += 1;

                    if args.dry_run {
                        println!("[EXPECT] {} ({} tokens)", rel_path.display(), token_count);
                        continue;
                    }

                    writeln!(writer, "<file path=\"{}\">", rel_path.display())?;
                    writeln!(writer, "{}", content)?;
                    writeln!(writer, "</file>\n")?;
                    println!("  + {} ({} tokens)", rel_path.display(), token_count);
                }
            }
            Err(error) => {
                eprintln!("Error processing file: {}", error);
            }
        }
    }

    if !args.dry_run {
        writer.flush().context("Failed to flush writer")?;
    }

    println!("======File processing completed======");
    println!("Files Processed: {}", stats.file_count);
    println!("Total Tokens: {}", stats.total_tokens);

    Ok(stats)
}

fn format_field<T: std::fmt::Display>(value: &Option<T>) -> String {
    value
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "(unset)".to_string())
}

fn format_path_field(value: &Option<std::path::PathBuf>) -> String {
    value
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "(unset)".to_string())
}

fn print_profile_show(profile_name: &str, profile: &ProfileConfig) {
    println!("Profile: {}", profile_name);
    println!(
        "Description: {}",
        profile.description.as_deref().unwrap_or("(unset)")
    );
    println!();
    println!("Content");
    println!(
        "  output-path     {}",
        format_path_field(&profile.output_path)
    );
    println!("  include         {}", format_field(&profile.include));
    println!("  exclude         {}", format_field(&profile.exclude));
    println!("  max-size        {}", format_field(&profile.max_size));
    println!();
    println!("Tree");
    println!("  tree-include    {}", format_field(&profile.tree_include));
    println!("  tree-exclude    {}", format_field(&profile.tree_exclude));
    println!("  no-tree         {}", format_field(&profile.no_tree));
    println!(
        "  tree-no-ignore  {}",
        format_field(&profile.tree_no_ignore)
    );
    println!();
    println!("Behavior");
    println!("  no-ignore       {}", format_field(&profile.no_ignore));
    println!("  no-blacklist    {}", format_field(&profile.no_blacklist));
}

fn main() -> Result<()> {
    let matches = Args::command().get_matches();
    let mut args = Args::from_arg_matches(&matches)?;
    let explicit_profile = args.explicit_profile_config(&matches);
    let base_path = args
        .path
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = base_path.join(".onesourcerc");

    // 1. Handle subcommands early
    if let Some(command) = &args.command {
        match command {
            configs::Commands::Profile { subcommand } => match subcommand {
                configs::ProfileSubcommands::List { json } => {
                    if let Some(config_doc) = Args::read_config(&config_path)? {
                        if *json {
                            let profiles: Vec<_> = Args::sorted_profiles(&config_doc)
                                .into_iter()
                                .map(|(name, profile)| {
                                    serde_json::json!({
                                        "name": name,
                                        "profile": profile,
                                    })
                                })
                                .collect();
                            println!("{}", serde_json::to_string_pretty(&profiles)?);
                        } else {
                            println!("Available profiles in {}:", config_path.display());
                            for (name, profile) in Args::sorted_profiles(&config_doc) {
                                let desc =
                                    profile.description.as_deref().unwrap_or("No description");
                                println!("  - {:<15} : {}", name, desc);
                            }
                        }
                    } else {
                        return Err(anyhow::anyhow!(
                            "No .onesourcerc found or invalid format at {}",
                            config_path.display()
                        ));
                    }
                    return Ok(());
                }
                configs::ProfileSubcommands::Show { profile, json } => {
                    Args::validate_profile_name(profile)?;
                    let config_doc = Args::read_config(&config_path)?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No .onesourcerc found or invalid format at {}",
                            config_path.display()
                        )
                    })?;
                    let profile_config = Args::get_profile(&config_doc, profile)?;
                    if *json {
                        println!("{}", serde_json::to_string_pretty(profile_config)?);
                    } else {
                        print_profile_show(profile, profile_config);
                    }
                    return Ok(());
                }
                configs::ProfileSubcommands::Create { profile, options } => {
                    Args::create_profile(&config_path, profile, options.to_profile_config())?;
                    println!("Created profile '{}' at {}", profile, config_path.display());
                    return Ok(());
                }
                configs::ProfileSubcommands::Update {
                    profile,
                    replace,
                    options,
                } => {
                    Args::update_profile(
                        &config_path,
                        profile,
                        options.to_profile_config(),
                        *replace,
                    )?;
                    if *replace {
                        println!(
                            "Replaced profile '{}' at {}",
                            profile,
                            config_path.display()
                        );
                    } else {
                        println!("Updated profile '{}' at {}", profile, config_path.display());
                    }
                    return Ok(());
                }
                configs::ProfileSubcommands::Delete { profile } => {
                    Args::delete_profile(&config_path, profile)?;
                    println!(
                        "Deleted profile '{}' from {}",
                        profile,
                        config_path.display()
                    );
                    if profile == "default" {
                        println!("No-profile runs will now use built-in defaults.");
                    }
                    return Ok(());
                }
                configs::ProfileSubcommands::Rename { old, new } => {
                    Args::rename_profile(&config_path, old, new)?;
                    println!(
                        "Renamed profile '{}' to '{}' in {}",
                        old,
                        new,
                        config_path.display()
                    );
                    return Ok(());
                }
                configs::ProfileSubcommands::Desc {
                    description,
                    profile,
                } => {
                    let update = ProfileConfig {
                        description: Some(description.clone()),
                        ..ProfileConfig::default()
                    };
                    Args::update_profile(&config_path, profile, update, false)?;
                    println!("Updated profile '{}' at {}", profile, config_path.display());
                    return Ok(());
                }
            },
        }
    }

    // 2. Regular execution flow
    if !args.no_config {
        args.merge_saved_config(&config_path)?;
    }

    let is_show_arg = args.show_arg.unwrap_or(false);

    if args.replace && !args.save {
        return Err(anyhow::anyhow!("--replace can only be used with --save"));
    }

    if args.save {
        let profile_name = args.profile.as_deref().unwrap_or("default");
        Args::upsert_profile(&config_path, profile_name, explicit_profile, args.replace)?;
        if args.replace {
            println!(
                "Replaced profile '{}' at {}",
                profile_name,
                config_path.display()
            );
        } else {
            println!(
                "Saved profile '{}' at {}",
                profile_name,
                config_path.display()
            );
        }
    }

    let app_config = args.resolve();

    if app_config.dry_run {
        println!("\n[DRY RUN MODE] Previews only, no files will be written.\n");

        if !app_config.no_tree {
            generate_tree(&app_config, &mut std::io::stdout())?;
        }

        let mut sink = std::io::sink();
        process_files(&app_config, &mut sink)?;

        let full_output_path = std::env::current_dir()
            .map(|dir| dir.join(&app_config.output_path))
            .unwrap_or_else(|_| app_config.output_path.clone());

        println!(
            "Dry run finished. If executed, file would be saved at: {}",
            full_output_path.display()
        );
        if app_config.copy {
            println!("[WARNING] NO COPY WAS MADE WHILE DRY RUN")
        }
    } else if app_config.copy {
        let mut clipboard_writer = io_utils::ClipboardWriter::new().context(
            "Failed to initialize clipboard. Hint: Try running without -c flag to save to file instead",
        )?;

        if !app_config.no_tree {
            let mut stdout = std::io::stdout();
            let mut multi_writer = io_utils::tee(&mut clipboard_writer, &mut stdout);
            generate_tree(&app_config, &mut multi_writer)?;
        }
        process_files(&app_config, &mut clipboard_writer)?;

        clipboard_writer
            .flush()
            .context("Failed to copy to clipboard")?;
        println!("Output copied to clipboard successfully!");
    } else {
        let file = File::create(&app_config.output_path).with_context(|| {
            format!(
                "Failed to create output file: {}",
                app_config.output_path.display()
            )
        })?;
        let mut writer = BufWriter::new(file);

        let abs_path = app_config
            .output_path
            .canonicalize()
            .unwrap_or_else(|_| app_config.output_path.clone());

        let path_str = abs_path.display().to_string();
        let abs_path_display = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);

        if !app_config.no_tree {
            let mut stdout = std::io::stdout();
            let mut multi_writer = io_utils::tee(&mut writer, &mut stdout);
            generate_tree(&app_config, &mut multi_writer)?;
        }

        process_files(&app_config, &mut writer)?;

        writer.flush().context("Failed to flush output file")?;
        println!("Output saved to: {}", abs_path_display);
    }

    if is_show_arg {
        println!("======ARGS======");
        println!("Target path: {:#?}", app_config);
        println!("======Others======");
    }
    Ok(())
}
