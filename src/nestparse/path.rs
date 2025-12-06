use crate::constants::CONFIG_NAMES;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

pub fn is_config_file(file_name: &str) -> bool {
    CONFIG_NAMES.iter().any(|&name| name == file_name)
}

pub fn find_config_file() -> Option<PathBuf> {
    let current_dir = match current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Ошибка получения текущей директории: {}", e);
            return None;
        }
    };

    let entries = match fs::read_dir(&current_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Ошибка чтения директории {}: {}", current_dir.display(), e);
            return None;
        }
    };

    for entry in entries.flatten() {
        if entry.file_type().ok()?.is_file() {
            let file_path = entry.path();
            if let Some(file_name) = file_path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if is_config_file(name_str) {
                        return Some(file_path);
                    }
                }
            }
        }
    }

    None
}
