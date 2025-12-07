use super::path::is_config_file;
use std::fs;
use std::io;
use std::path::Path;

pub fn read_config_file(path: &Path) -> io::Result<String> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    }
    if let Some(file_name) = path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            if !is_config_file(file_name_str) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Not a config file",
                ));
            }
        }
    }
    fs::read_to_string(path)
}
