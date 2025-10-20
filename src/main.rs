use anyhow::Result;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut current_dir = Directory {
        name: "tui-file-manager".to_string(),
        path: ".".to_string(),
        files: Vec::new(),
        subdirectories: Vec::new(),
    };

    current_dir.scan_and_add().await.unwrap();
    dbg!(current_dir);
}

#[derive(Debug, Clone)]
struct Directory {
    name: String,
    path: String,
    files: Vec<FileEntry>,
    subdirectories: Vec<Directory>,
}

#[derive(Debug, Clone, PartialEq)]
enum FileType {
    Executable,
    Media,
    Document,
    Zip,
    Code,
    Unknown,
}

#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    filetype: FileType,
}

impl FileEntry {
    pub fn enumerate_filetype(&self) -> FileType {
        if self.filetype != FileType::Unknown {
            return self.filetype.clone();
        }
        let extension = self
            .name
            .chars()
            .rev()
            .collect::<String>()
            .split('.')
            .next()
            .unwrap_or("")
            .chars()
            .rev()
            .collect::<String>()
            .to_lowercase();

        let filetype: FileType = match extension.as_str() {
            "exe" | "bat" | "sh" => FileType::Executable,
            "mp3" | "mp4" | "wav" | "flac" | "jpg" | "png" | "gif" => FileType::Media,
            "pdf" | "doc" | "docx" | "txt" | "odt" | "json" | "toml" => FileType::Document,
            "zip" | "rar" | "7z" | "tar" | "gz" => FileType::Zip,
            "rs" | "py" | "js" | "java" | "c" | "cpp" | "html" | "css" => FileType::Code,
            _ => FileType::Unknown,
        };

        filetype
    }
}

impl Directory {
    fn add_file(&mut self, file: FileEntry) {
        self.files.push(file);
    }

    fn add_subdirectory(&mut self, subdirectory: Directory) {
        self.subdirectories.push(subdirectory);
    }

    pub async fn scan_and_add(&mut self) -> Result<()> {
        use std::fs;

        let entries = fs::read_dir(&self.path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let mut file = FileEntry {
                        name: file_name.to_string(),
                        filetype: FileType::Unknown,
                    };
                    let filetype = file.enumerate_filetype();
                    file.filetype = filetype;

                    self.add_file(file);
                }
            } else if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    let mut subdirectory = Directory {
                        name: dir_name.to_string(),
                        path: path.to_str().unwrap().to_string(),
                        files: Vec::new(),
                        subdirectories: Vec::new(),
                    };
                    self.add_subdirectory(subdirectory);
                }
            }
        }

        Ok(())
    }
}
