use ignore::WalkBuilder;
use std::{fs::File, io::{BufWriter,Write}};
use configs::{Args};
use clap::Parser;
use crate::configs::AppConfig;
use tiktoken_rs::cl100k_base;

mod tree_utils;
mod io_utils;
mod filter_utils;
mod configs;

#[derive(Debug, Default)]
struct FileStats {
    file_count: u32,
    total_tokens: usize,
}


fn struct_tree<W: Write>(args:&AppConfig,writer: &mut W){
    let final_include = args.tree_include.as_deref().or(args.include.as_deref());
    let final_exclude = args.tree_exclude.as_deref().or(args.exclude.as_deref());
    let filter = filter_utils::FileFilter::new(final_include, final_exclude,args.no_blacklist);
    let mut tree_root = tree_utils::Node::new(true);
    
    let walker = WalkBuilder::new(&args.path)
        .standard_filters(!args.tree_no_ignore)
        .hidden(false)
        .require_git(false)
        .build();
    
    for result in walker{
        match result {
            Ok(entry)=>{
                let rel_path = entry.path().strip_prefix(&args.path).unwrap_or(entry.path());
                if !filter.is_match(rel_path){continue;}
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                tree_root.insert_path(rel_path, is_dir);
            }
            Err(error)=>{
                print!("{}",error)
            }
        }
    }
    let root_name = args.path.canonicalize() // 先轉為絕對路徑
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| ".".into());
    writeln!(writer,"{}/", root_name).expect("Write root failed");
    tree_root.print("",writer).expect("Error at print tree");
}
fn rw_file<W: Write>(args:&AppConfig,writer:&mut W) -> FileStats {
    let bpe = cl100k_base().expect("Failed to load tokenizer");
    let filter = filter_utils::FileFilter::new(args.include.as_deref(), args.exclude.as_deref(),args.no_blacklist);
    let walker = WalkBuilder::new(&args.path)
        .standard_filters(!args.no_ignore)
        .hidden(false)
        .require_git(false)
        .build();
    let mut stats = FileStats::default();
    
    for result in walker{
        match result {
            Ok(entry)=>{
                let rel_path = entry.path().strip_prefix(&args.path).unwrap_or(entry.path());
                if !filter.is_match(rel_path){continue;}
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                if !is_dir{
                    let metadata = entry.metadata().ok();
                    let size_kb = metadata.map(|m| m.len() / 1024).unwrap_or(0);
                    if size_kb > args.max_size as u64 { continue; }
                    let is_text_file = if let Ok(mut f) = File::open(entry.path()) {
                        use std::io::Read;
                        let mut buffer = [0; 1024];
                        let n = f.read(&mut buffer).unwrap_or(0);
                        !buffer[..n].contains(&0)
                    } else {
                        false
                    };
                    if !is_text_file { continue; }
                    if args.dry_run {
                        // In dry run mode, read the file to calculate tokens but don't write output
                        if let Ok(bytes) = std::fs::read(entry.path()) {
                            let content = String::from_utf8_lossy(&bytes);
                            let tokens = bpe.encode_with_special_tokens(&content);
                            let token_count = tokens.len();
                            stats.total_tokens += token_count;
                            println!("[EXPECT] {} ({} tokens)", rel_path.display(), token_count);
                        } else {
                            println!("[EXPECT] {}", rel_path.display());
                        }
                        stats.file_count += 1;
                        continue;
                    }
                    if let Ok(bytes) = std::fs::read(entry.path()) {
                        let content = String::from_utf8_lossy(&bytes);
                        
                        // Calculate tokens for this file
                        let tokens = bpe.encode_with_special_tokens(&content);
                        let token_count = tokens.len();
                        stats.total_tokens += token_count;
                        
                        writeln!(writer, "<file path=\"{}\">", rel_path.display()).unwrap();
                        writeln!(writer, "{}", content).unwrap();
                        writeln!(writer, "</file>\n").unwrap();
                        println!("  + {} ({} tokens)",rel_path.display(), token_count);
                        stats.file_count += 1;
                    }
                }
            }
            Err(error)=>{
                print!("{}",error)
            }
        }
    }
    if !args.dry_run {
        writer.flush().expect("last input flush fail");
    }
    println!("======File processing completed======");
    println!("Files Processed: {}", stats.file_count);
    println!("Total Tokens: {}", stats.total_tokens);
    
    stats
}


fn main() {
    let mut args = Args::parse();
    
    // 1. Handle subcommands early
    if let Some(command) = &args.command {
        match command {
            configs::Commands::Profile { subcommand } => {
                match subcommand {
                    configs::ProfileSubcommands::Ls { json } => {
                        let base_path = args.path.as_deref().unwrap_or(std::path::Path::new("."));
                        let config_path = base_path.join(".onesourcerc");
                        
                        if let Some(config_doc) = Args::read_config(&config_path) {
                            if *json {
                                println!("{}", serde_json::to_string_pretty(&config_doc.profiles).unwrap());
                            } else {
                                println!("Available profiles in {}:", config_path.display());
                                for (name, profile) in &config_doc.profiles {
                                    let desc = profile.description.as_deref().unwrap_or("No description");
                                    println!("  - {:<15} : {}", name, desc);
                                }
                            }
                        } else {
                            eprintln!("No .onesourcerc found or invalid format.");
                            std::process::exit(1);
                        }
                        return;
                    }
                }
            }
        }
    }

    // 2. Regular execution flow
    let base_path = args.path.as_deref().unwrap_or(std::path::Path::new("."));
    let config_path = base_path.join(".onesourcerc");
    
    if !args.no_config {
        args.merge_saved_config(&config_path);
    }

    let is_show_arg = args.show_arg.unwrap_or(false);
    
    if let Some(profile_name) = &args.save {
        if let Err(e) = args.save_config(&config_path, profile_name) {
            eprintln!("WARNING: Fail to save configs ({})", e);
        } else {
            println!("Save Successfully to profile '{}' at: {}", profile_name, config_path.display());
        }
    }
    
    let app_config = args.resolve();

    if app_config.dry_run {
        println!("\n[DRY RUN MODE] Previews only, no files will be written.\n");
        
        if !app_config.no_tree {
            struct_tree(&app_config, &mut std::io::stdout());
        }

        // if dry run, no writer
        let mut sink = std::io::sink();
        rw_file(&app_config, &mut sink);

        println!("Dry run finished. If executed, file would be saved at: {}",std::env::current_dir()
                                                                                .map(|dir| dir.join(&app_config.output_path))
                                                                                .unwrap_or_else(|_| app_config.output_path.clone()) 
                                                                                .display());
        if app_config.copy{
            println!("[WARNING] NO COPY WAS MADE WHILE DRY RUN")
        }
    } 
    else if app_config.copy {

        let mut clipboard_writer = match io_utils::ClipboardWriter::new() {
            Ok(writer) => writer,
            Err(e) => {
                eprintln!("[ERROR] Failed to initialize clipboard: {}", e);
                eprintln!("Hint: Try running without -c flag to save to file instead");
                std::process::exit(1);
            }
        };
        if !app_config.no_tree {
            let mut stdout = std::io::stdout();
            let mut multi_writer = io_utils::tee(&mut clipboard_writer, &mut stdout);
            struct_tree(&app_config, &mut multi_writer);
        }
        rw_file(&app_config, &mut clipboard_writer);

        if let Err(e) = clipboard_writer.flush() {
            eprintln!("[ERROR] Failed to copy to clipboard: {}", e);
            std::process::exit(1);
        }
        
        println!("Output copied to clipboard successfully!");
    }
    else {
        
        // Only not dry will creat file
        let file = File::create(&app_config.output_path).expect("Create output file failed");
        let mut writer = BufWriter::new(file);
        
        let abs_path = app_config.output_path.canonicalize()
            .unwrap_or_else(|_| app_config.output_path.clone()); // 

        let path_str = abs_path.display().to_string();

        let abs_path_display = path_str
            .strip_prefix(r"\\?\")
            .unwrap_or(&path_str);
                if !app_config.no_tree {
                    let mut stdout = std::io::stdout();
                    let mut multi_writer = io_utils::tee(&mut writer, &mut stdout);
                    struct_tree(&app_config, &mut multi_writer);
                }

        rw_file(&app_config, &mut writer);

        writer.flush().expect("Flush failed");
        
        println!("Output saved to: {}", abs_path_display);
    }


    if is_show_arg { show_args(&app_config); } 
    // println!("=====================================");
}

fn show_args(args:&AppConfig){
    println!("======ARGS======");
    println!("Target path: {:#?}",args);    
    println!("======Others======")    
}
