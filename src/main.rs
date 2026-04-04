use ignore::WalkBuilder;
use std::{fs::File, io::{BufWriter,Write}};
use configs::{Args};
use clap::Parser;
use crate::configs::AppConfig;

mod tree_utils;
mod io_utils;
mod filter_utils;
mod configs;


fn struct_tree<W: Write>(args:&AppConfig,writer: &mut W){
    let final_include = args.tree_include.as_deref().or(args.include.as_deref());
    let final_exclude = args.tree_exclude.as_deref().or(args.exclude.as_deref());
    let filter = filter_utils::FileFilter::new(final_include, final_exclude);
    let mut tree_root = tree_utils::Node::new(true);
    
    let walker = WalkBuilder::new(&args.path)
        .standard_filters(!args.tree_no_ignore)
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
fn rw_file<W: Write>(args:&AppConfig,writer:&mut W){
    let filter = filter_utils::FileFilter::new(args.include.as_deref(), args.exclude.as_deref());
    let walker = WalkBuilder::new(&args.path)
        .standard_filters(!args.no_ignore)
        .require_git(false)
        .build();
    let mut count:u32 = 0;
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
                        println!("[EXPECT] {}", rel_path.display());
                        count += 1;
                        continue;
                    }
                    if let Ok(bytes) = std::fs::read(entry.path()) {
                        let content = String::from_utf8_lossy(&bytes);
                        writeln!(writer, "<file path=\"{}\">", rel_path.display()).unwrap();
                        writeln!(writer, "{}", content).unwrap();
                        writeln!(writer, "</file>\n").unwrap();
                        println!("  + {}",rel_path.display());
                        count += 1;
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
    println!("Files Processed: {}",count);
}


fn main() {
    let mut args = Args::parse();
    
    /*
    Args priority :
    User input -> saves -> default
    
    Args usage:
    configs::Args -> use for user input.
    configs::AppConfig -> final config after all config, input logics that functions read.
    */

    // Args logic
    // 1. Get the user input and the configs.
    let base_path = args.path.as_deref().unwrap_or(std::path::Path::new("."));
    let config_path = base_path.join(".onesourcerc");

    // 2. Read the user input first then the saved configs
    if !args.no_config {
        if let Some(config) = Args::read_config(&config_path) {
            println!(".onesourcerc found, using settings.");
            args.output_path = args.output_path.or(config.output_path);
            args.no_ignore = args.no_ignore.or(config.no_ignore);
            args.include = args.include.or(config.include);
            args.exclude = args.exclude.or(config.exclude);
            args.tree_include = args.tree_include.or(config.tree_include);
            args.tree_exclude = args.tree_exclude.or(config.tree_exclude);
            args.no_tree = args.no_tree.or(config.no_tree);
            args.tree_no_ignore = args.tree_no_ignore.or(config.tree_no_ignore);
            args.max_size = args.max_size.or(config.max_size);
            
            // NOTE: args.path, args.show_arg, args.save, args.dry_run NOT inherit by saved config
        } else {println!("No .onesourcerc found, use default settings.")}
    }

    // 3. apply final settings.
    
    
    // Debug: show all args
    let is_show_arg = args.show_arg.unwrap_or(false);
    let is_save = args.save;   
    if is_save {
        if let Err(e) = args.save_config(&config_path) {
            eprintln!("WARNING: Fail to save configs ({})", e);
        } else {
            println!("Save Successfully {}", config_path.display());
        }
    }
    
    let app_config = args.resolve();

    
    //Start proccess
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

    } else {
        
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
