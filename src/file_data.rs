use error::{OuterError, OuterResult};
use std::io::{Read, Write};
use std::{fs, io};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FileHandle(u32);

#[cfg(test)]
impl FileHandle {
    pub fn dummy() -> Self {
        FileHandle(0)
    }
}

pub struct FileData {
    file_names: Vec<String>,
    file_contents: Vec<String>,
}

fn read_file_inner(path: &str) -> io::Result<String> {
    let mut file = fs::OpenOptions::new().read(true).open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn read_file(path: &str) -> OuterResult<String> {
    match read_file_inner(path) {
        Ok(rv) => Ok(rv),
        Err(err) => {
            writeln!(
                io::stderr(),
                "error: Failed to read file '{}': {}",
                path,
                err
            )
            .unwrap();
            Err(OuterError::ReadInput)
        }
    }
}

impl FileData {
    pub fn new() -> Self {
        FileData {
            file_names: Vec::new(),
            file_contents: Vec::new(),
        }
    }
    pub fn read(&mut self, name: String) -> OuterResult<FileHandle> {
        let contents = read_file(&name)?;
        Ok(self.add(name, contents))
    }
    pub fn add(&mut self, name: String, contents: String) -> FileHandle {
        self.file_names.push(name);
        self.file_contents.push(contents);
        FileHandle((self.file_contents.len() - 1) as u32)
    }
    pub fn get_name(&self, handle: FileHandle) -> &str {
        &self.file_names[handle.0 as usize]
    }
    pub fn get_contents(&self, handle: FileHandle) -> &str {
        &self.file_contents[handle.0 as usize]
    }
}
