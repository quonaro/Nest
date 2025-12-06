use super::path::is_config_file;
use crate::constants::INDENT_SIZE;
use std::fs;
use std::io;
use std::path::Path;

pub fn read_config_file(path: &Path) -> io::Result<String> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    }
    if !is_config_file(path.file_name().unwrap().to_str().unwrap()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Not a config file",
        ));
    }
    fs::read_to_string(path)
}

pub fn get_indent_size(line: &str, indent_size: Option<u8>) -> u8 {
    let indent_size = indent_size.unwrap_or(INDENT_SIZE);
    let mut spaces = 0;
    for char in line.chars() {
        if char == ' ' {
            spaces += 1;
        } else {
            break;
        }
    }
    spaces / indent_size
}

pub fn type_of_line(line: &str) -> &str {
    let trimmed = line.trim();

    // Пустая строка
    if trimmed.is_empty() {
        return "null";
    }

    // Комментарии
    if trimmed.starts_with("#") {
        return "comment";
    }

    // Директивы
    if trimmed.starts_with("@func") {
        return "func";
    } else if trimmed.starts_with("@desc") {
        return "desc";
    } else if trimmed.starts_with("@args") {
        return "args";
    } else if trimmed.starts_with("@flags") {
        return "flags";
    } else if trimmed.starts_with("@cwd") {
        return "cwd";
    } else if trimmed.starts_with("@depends") {
        return "depends";
    } else if trimmed.starts_with("@before") {
        return "before";
    } else if trimmed.starts_with("@after") {
        return "after";
    } else if trimmed.starts_with("@fallback") {
        return "fallback";
    } else if trimmed.starts_with("@env-file") {
        return "env-file";
    } else if trimmed.starts_with("@env") {
        return "env";
    } else if trimmed.starts_with("@defaults") {
        return "defaults";
    } else if trimmed.starts_with("@call") {
        return "call";
    }

    // Директива script (может быть с двоеточием или без)
    if trimmed.starts_with("script:") || trimmed == "script" {
        return "script";
    }

    // Код скрипта или команда (не начинается с @ и не пустая)
    "script_line"
}
