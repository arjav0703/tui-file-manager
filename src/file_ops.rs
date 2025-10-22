use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Directory {
    pub name: String,
    pub path: String,
    pub files: Vec<FileEntry>,
    pub subdirectories: Vec<Directory>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Executable,
    Media,
    Document,
    Zip,
    Code,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub filetype: FileType,
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

    pub fn assign_symbol(&self) -> String {
        let symbol = match self.filetype {
            FileType::Executable => "⚙️ ",
            FileType::Media => "🎵 ",
            FileType::Document => "📄 ",
            FileType::Zip => "🗜️ ",
            FileType::Code => r"</> ",
            FileType::Unknown => "❓",
        };

        format!("{} {}", symbol, self.name)
    }
}

impl FromIterator<FileEntry> for Directory {
    fn from_iter<T: IntoIterator<Item = FileEntry>>(iter: T) -> Self {
        let mut dir = Directory::new(String::new(), String::new());
        for file in iter {
            dir.add_file(file);
        }
        dir
    }
}

impl Directory {
    pub fn new(name: String, path: String) -> Self {
        Directory {
            name,
            path,
            files: Vec::new(),
            subdirectories: Vec::new(),
        }
    }
    fn add_file(&mut self, file: FileEntry) {
        self.files.push(file);
    }

    fn add_subdirectory(&mut self, subdirectory: Directory) {
        self.subdirectories.push(subdirectory);
    }

    pub async fn scan_and_add(&mut self) -> Result<()> {
        self.files.clear();
        self.subdirectories.clear();

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
            } else if path.is_dir()
                && let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
            {
                let subdirectory = Directory {
                    name: dir_name.to_string(),
                    path: path.to_str().unwrap().to_string(),
                    files: Vec::new(),
                    subdirectories: Vec::new(),
                };
                self.add_subdirectory(subdirectory);
            }
        }

        Ok(())
    }

    pub fn entries(&self) -> Vec<String> {
        let mut entries = Vec::new();

        let sorted_dir = self.sort_subdirectories();
        for subdir in &sorted_dir.subdirectories {
            entries.push(format!("{}/", subdir.name));
        }

        let sorted_files = self.sort_files();
        for file in &sorted_files.files {
            entries.push(file.name.clone());
        }

        entries
    }

    pub fn entries_with_symbols(&self) -> Vec<String> {
        let mut entries = Vec::new();

        let sorted_dir = self.sort_subdirectories();
        for subdir in &sorted_dir.subdirectories {
            entries.push(format!("📂 {}/", subdir.name));
        }

        let sorted_files = self.sort_files();
        for file in &sorted_files.files {
            entries.push(file.assign_symbol())
        }

        entries
    }

    fn sort_subdirectories(&self) -> Directory {
        let mut sorted_dir = self.clone();
        sorted_dir
            .subdirectories
            .sort_by(|a, b| a.name.cmp(&b.name));
        sorted_dir.files.sort_by(|a, b| a.name.cmp(&b.name));
        sorted_dir
    }

    fn sort_files(&self) -> Directory {
        let mut sorted_dir = self.clone();
        sorted_dir.files.sort_by(|a, b| a.name.cmp(&b.name));
        sorted_dir
            .subdirectories
            .sort_by(|a, b| a.name.cmp(&b.name));
        sorted_dir
    }
}

pub async fn get_current_directory() -> Result<Directory> {
    use std::env;

    let current_path = env::current_dir()?;
    let dir_name = current_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let path_str = current_path.to_str().unwrap_or("").to_string();

    let directory = Directory::new(dir_name, path_str);

    Ok(directory)
}
