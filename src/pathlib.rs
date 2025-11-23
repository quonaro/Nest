use std::env::current_dir;
use std::path::PathBuf;
use std::{fs, io};
use crate::constants::CONFIG_NAMES;

fn pwd() -> String {
    match current_dir() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(e) => {
            eprintln!("Error getting current directory: {}", e);
            String::new()
        }
    }
}

fn list_files(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.push(entry.path());
        }
    }
    Ok(files)
}

fn is_config_file(path: &str) -> bool {
    CONFIG_NAMES.contains(&path)
}

pub fn find_config_file() -> Option<PathBuf>  {
    let pwd_path = pwd();
    match list_files(&pwd_path){
        Ok(files) => {
            for file_path in files {
                if let Some(file_name) = file_path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if is_config_file(name_str) {
                            return Some(file_path);
                        }
                    }
                }
            }
            None
        }
        Err(e) => {
            eprintln!("Error reading directory {}: {}", pwd_path, e);
            None
        }
    }
}