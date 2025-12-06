mod constants;
mod nestparse;

use nestparse::blocks::{get_indent_size, read_config_file, type_of_line};
use nestparse::path::find_config_file;

fn main() {
    let mut config_content = String::new();
    match find_config_file() {
        Some(config_path) => {
            println!("{}", config_path.display());

            match read_config_file(&config_path) {
                Ok(content) => {
                    config_content = content;
                }
                Err(e) => {
                    eprintln!("Ошибка чтения файла: {}", e);
                }
            }
        }
        None => println!("Не найдено"),
    }
    for line in config_content.lines() {
        let indent_size = get_indent_size(line, None);
        println!("[{}] [{}] {}", indent_size, type_of_line(line), line);
    }
}
