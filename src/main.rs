mod pathlib;
mod constants;
use std::path::PathBuf;

use pathlib::{find_config_file};

fn main() {
    let x: Option<PathBuf> = find_config_file();
    
    // Вариант 1: с помощью match
    let result = match x {
        Some(path) => path.to_string_lossy().to_string(),
        None => "Не найдено".to_string(),
    };
    println!("{}", result);
}