mod configs;
mod explain;
mod filter_utils;
mod io_utils;
mod scan;
mod self_update;
mod tree_utils;

use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches};
use tiktoken_rs::{cl100k_base, CoreBPE};

use crate::configs::{AppConfig, Args, ProfileConfig};

#[derive(Debug, Default)]
struct FileStats {
    file_count: usize,
    skipped_count: usize,
    estimated_output_tokens: usize,
}

fn write_counted<W: Write>(
    writer: &mut W,
    bpe: &CoreBPE,
    value: &str,
    stats: &mut FileStats,
) -> Result<()> {
    writer
        .write_all(value.as_bytes())
        .context("Failed to write generated output")?;
    stats.estimated_output_tokens += bpe.encode_with_special_tokens(value).len();
    Ok(())
}

fn render_tree<W: Write>(
    selection: &scan::ScanSelection,
    writer: &mut W,
    bpe: &CoreBPE,
    stats: &mut FileStats,
) -> Result<()> {
    if let Some(tree) = &selection.tree {
        write_counted(writer, bpe, tree, stats)?;
    }
    Ok(())
}

fn process_files<W: Write>(
    args: &AppConfig,
    selection: &scan::ScanSelection,
    writer: &mut W,
    bpe: &CoreBPE,
    stats: &mut FileStats,
) -> Result<()> {
    for candidate in &selection.candidates {
        match scan::inspect_file(&candidate.full_path, args.max_size) {
            Ok(inspected) => {
                let content_tokens = bpe.encode_with_special_tokens(&inspected.content).len();
                stats.file_count += 1;
                if inspected.lossy_utf8 {
                    eprintln!(
                        "[WARNING] {} is not valid UTF-8; replacement characters were used",
                        candidate.rel_path.display()
                    );
                }

                let opening = format!(
                    "<file path=\"{}\">\n",
                    scan::escape_path_attribute(&candidate.rel_path)
                );
                write_counted(writer, bpe, &opening, stats)?;
                write_counted(writer, bpe, &inspected.content, stats)?;
                write_counted(writer, bpe, "\n</file>\n\n", stats)?;

                if args.dry_run {
                    let suffix = if inspected.lossy_utf8 {
                        ", lossy UTF-8"
                    } else {
                        ""
                    };
                    println!(
                        "[EXPECT] {} ({} content tokens{})",
                        candidate.rel_path.display(),
                        content_tokens,
                        suffix
                    );
                } else {
                    println!(
                        "  + {} ({} content tokens)",
                        candidate.rel_path.display(),
                        content_tokens
                    );
                }
            }
            Err(reason) => {
                stats.skipped_count += 1;
                let message = skip_reason_text(&reason);
                if args.dry_run {
                    println!("[SKIP] {} ({})", candidate.rel_path.display(), message);
                }
                if matches!(reason, scan::SkipReason::Unreadable(_)) {
                    eprintln!(
                        "[WARNING] Skipping {}: {}",
                        candidate.rel_path.display(),
                        message
                    );
                }
            }
        }
    }

    writer.flush().context("Failed to flush generated output")?;
    Ok(())
}

fn skip_reason_text(reason: &scan::SkipReason) -> String {
    match reason {
        scan::SkipReason::TooLarge {
            max_kib,
            actual_bytes,
        } => format!("larger than {} KiB: {} bytes", max_kib, actual_bytes),
        scan::SkipReason::Binary => "binary file".to_string(),
        scan::SkipReason::Unreadable(error) => format!("unreadable: {}", error),
    }
}

fn print_stats(stats: &FileStats) {
    println!("======File processing completed======");
    println!("Files Processed: {}", stats.file_count);
    println!("Files Skipped: {}", stats.skipped_count);
    println!("Estimated Output Tokens: {}", stats.estimated_output_tokens);
}

fn persist_output<F>(output_path: &Path, render: F) -> Result<()>
where
    F: FnOnce(&mut BufWriter<&mut std::fs::File>) -> Result<()>,
{
    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(anyhow::anyhow!(
            "Output directory does not exist: {}",
            parent.display()
        ));
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary_path = parent.join(format!(".onesource-tmp-{}-{}", std::process::id(), unique));
    let mut temporary = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary_path)
        .with_context(|| format!("Failed to create temporary output in {}", parent.display()))?;

    let render_result = (|| -> Result<()> {
        let mut writer = BufWriter::new(&mut temporary);
        render(&mut writer)?;
        writer.flush().context("Failed to flush temporary output")?;
        drop(writer);
        temporary
            .sync_all()
            .context("Failed to sync temporary output")
    })();
    drop(temporary);

    if let Err(error) = render_result {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(error);
    }

    replace_output(&temporary_path, output_path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary_path);
        anyhow::anyhow!(
            "Failed to replace output file {}: {}",
            output_path.display(),
            error
        )
    })?;
    Ok(())
}

#[cfg(not(windows))]
fn replace_output(temporary: &Path, output: &Path) -> std::io::Result<()> {
    std::fs::rename(temporary, output)
}

#[cfg(windows)]
fn replace_output(temporary: &Path, output: &Path) -> std::io::Result<()> {
    if !output.exists() {
        return std::fs::rename(temporary, output);
    }

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let backup = output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            ".onesource-backup-{}-{}",
            std::process::id(),
            unique
        ));
    std::fs::rename(output, &backup)?;
    match std::fs::rename(temporary, output) {
        Ok(()) => {
            let _ = std::fs::remove_file(backup);
            Ok(())
        }
        Err(error) => {
            let _ = std::fs::rename(backup, output);
            Err(error)
        }
    }
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
    let command = args.command.clone();
    if let Some(configs::Commands::Explain { options, .. }) = &command {
        args.apply_explain_options(options);
    }
    let base_path = args
        .path
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = base_path.join(".onesourcerc");

    // 1. Handle subcommands early
    if let Some(command) = &command {
        match command {
            configs::Commands::Update => {
                self_update::run()?;
                return Ok(());
            }
            configs::Commands::Explain { paths, .. } => {
                if !args.no_config {
                    args.merge_saved_config(&config_path)?;
                }
                let app_config = args.resolve();
                let reports = explain::explain_paths(&app_config, paths)?;
                explain::print_reports(&reports);
                return Ok(());
            }
            configs::Commands::Profile { subcommand } => match &**subcommand {
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
    let selection = scan::build_selection(&app_config)?;
    let bpe = cl100k_base().context("Failed to load tokenizer")?;
    let mut stats = FileStats {
        skipped_count: selection.walk_errors,
        ..FileStats::default()
    };

    if app_config.dry_run {
        println!("\n[DRY RUN MODE] Previews only, no files will be written.\n");

        render_tree(&selection, &mut std::io::stdout(), &bpe, &mut stats)?;

        let mut sink = std::io::sink();
        process_files(&app_config, &selection, &mut sink, &bpe, &mut stats)?;

        println!(
            "Dry run finished. If executed, file would be saved at: {}",
            selection.output_path.display()
        );
        if app_config.copy {
            println!("[WARNING] NO COPY WAS MADE WHILE DRY RUN")
        }
    } else if app_config.copy {
        let mut clipboard_writer = io_utils::ClipboardWriter::new().context(
            "Failed to initialize clipboard. Hint: Try running without -c flag to save to file instead",
        )?;

        let mut stdout = std::io::stdout();
        let mut multi_writer = io_utils::tee(&mut clipboard_writer, &mut stdout);
        render_tree(&selection, &mut multi_writer, &bpe, &mut stats)?;
        process_files(
            &app_config,
            &selection,
            &mut clipboard_writer,
            &bpe,
            &mut stats,
        )?;

        clipboard_writer
            .flush()
            .context("Failed to copy to clipboard")?;
        println!("Output copied to clipboard successfully!");
    } else {
        persist_output(&selection.output_path, |writer| {
            let mut stdout = std::io::stdout();
            let mut multi_writer = io_utils::tee(&mut *writer, &mut stdout);
            render_tree(&selection, &mut multi_writer, &bpe, &mut stats)?;
            process_files(&app_config, &selection, writer, &bpe, &mut stats)
        })?;

        let path_str = selection.output_path.display().to_string();
        println!(
            "Output saved to: {}",
            path_str.strip_prefix(r"\\?\").unwrap_or(&path_str)
        );
    }

    print_stats(&stats);

    if is_show_arg {
        println!("======ARGS======");
        println!("Target path: {:#?}", app_config);
        println!("======Others======");
    }
    Ok(())
}

#[cfg(test)]
mod main_tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("onesource-main-{}-{}", name, unique));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn failed_render_preserves_previous_output() {
        let dir = temp_dir("preserve-output");
        let output = dir.join("result.onesource");
        fs::write(&output, "previous").unwrap();

        let result = persist_output(&output, |writer| {
            writer.write_all(b"partial replacement")?;
            Err(anyhow::anyhow!("intentional render failure"))
        });

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&output).unwrap(), "previous");
        assert_eq!(fs::read_dir(dir).unwrap().count(), 1);
    }

    #[test]
    fn successful_render_replaces_previous_output() {
        let dir = temp_dir("replace-output");
        let output = dir.join("result.onesource");
        fs::write(&output, "previous").unwrap();

        persist_output(&output, |writer| {
            writer.write_all(b"replacement")?;
            Ok(())
        })
        .unwrap();

        assert_eq!(fs::read_to_string(output).unwrap(), "replacement");
    }
}
