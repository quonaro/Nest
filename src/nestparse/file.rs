use super::path::is_config_file;
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
