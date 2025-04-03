use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::Path;

pub fn rvm_fopen(filename: &str, _extension: &str, mode: &str) -> io::Result<File> {
    match mode {
        "r" => File::open(filename),
        "w" => File::create(filename),
        "a" => OpenOptions::new().append(true).open(filename),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid file mode")),
    }
}

pub fn rvm_fcopy(src: &mut File) -> io::Result<String> {
    let mut content = String::new();
    src.read_to_string(&mut content)?;
    Ok(content)
}

pub fn rvm_flength(file: &mut File) -> io::Result<u64> {
    let current_pos = file.seek(SeekFrom::Current(0))?;
    let length = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(current_pos))?;
    Ok(length)
}
