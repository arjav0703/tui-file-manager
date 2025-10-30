use super::*;

impl App {
    pub fn open_file(&mut self) {
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
        use std::process::Command;

        if let Some(file_index) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(file_index)
            // && !selected_entry.ends_with('/')
            {
                let full_path = format!("{}/{}", self.dir.path, selected_entry);

                // leave TUI mode (temporarily)
                if let Err(e) = disable_raw_mode() {
                    eprintln!("Failed to disable raw mode: {e}");
                }

                #[cfg(target_os = "macos")]
                let mut cmd = Command::new("open");
                #[cfg(target_os = "linux")]
                let mut cmd = Command::new("xdg-open");
                #[cfg(target_os = "windows")]
                let mut cmd = Command::new("cmd");

                #[cfg(target_os = "windows")]
                {
                    cmd.args(["/C", "start", "", &full_path])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                }
                #[cfg(not(target_os = "windows"))]
                {
                    cmd.arg(&full_path)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                }

                match cmd.status() {
                    Ok(status) => {
                        if !status.success() {
                            // eprintln!("Failed to open file: {:?}", status);
                        }
                    }
                    Err(_err) => {
                        // eprintln!("Error launching file: {err}");
                    }
                }

                if let Err(e) = enable_raw_mode() {
                    eprintln!("Failed to enable raw mode: {e}");
                }
            }
        }
    }

    pub async fn delete_file(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                self.show_confirmation = true;
                self.file_to_delete = Some(selected_entry.clone());
            }
        }
    }

    pub fn rename_file(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                self.show_rename = true;
                self.file_to_rename = Some(selected_entry.to_string());

                // Pre-populate the input with the current filename
                let current_name = selected_entry.trim_end_matches('/');
                self.rename_input = TextArea::from([current_name]);
                self.rename_input
                    .set_block(Block::bordered().title("New name"));
            }
        }
    }

    pub fn yank_file(&mut self) {
        use std::process::Command;

        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                let full_path = format!("{}/{}", self.dir.path, selected_entry);

                #[cfg(target_os = "macos")]
                let mut cmd = Command::new("pbcopy");
                #[cfg(target_os = "linux")]
                let mut cmd = Command::new("xclip");
                #[cfg(target_os = "windows")]
                let mut cmd = Command::new("clip");

                #[cfg(target_os = "linux")]
                {
                    cmd.args(["-selection", "clipboard"]);
                }

                let mut process = cmd
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .expect("Failed to spawn clipboard command");

                if let Some(stdin) = process.stdin.as_mut() {
                    use std::io::Write;
                    stdin
                        .write_all(full_path.as_bytes())
                        .expect("Failed to write to clipboard");
                }

                let _ = process.wait();
            }
        }
    }

    pub fn handle_copy_file(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                let full_path = format!("{}/{}", self.dir.path, selected_entry);
                self.clipboard = Some(Clipboard {
                    cut: false,
                    path: full_path,
                });
            }
        }
    }

    pub fn handle_cut_file(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                let full_path = format!("{}/{}", self.dir.path, selected_entry);
                self.clipboard = Some(Clipboard {
                    cut: true,
                    path: full_path,
                });
            }
        }
    }

    pub async fn handle_paste(&mut self) {
        if let Some(clipboard_path) = &self.clipboard.as_ref().map(|c| &c.path) {
            let filename = clipboard_path.rsplit('/').next().unwrap_or("pasted_file");
            let new_path = format!("{}/{}", self.dir.path, filename);
            if let Err(err) = fs::copy(clipboard_path, &new_path) {
                eprintln!("Failed to paste file: {err}");
            } else {
                self.dir.scan_and_add(self.show_hidden_files).await.unwrap();
            }
        }

        if let Some(to_cut) = &self.clipboard {
            if to_cut.cut {
                if let Err(_r) = fs::remove_file(&to_cut.path) {
                    fs::remove_dir_all(&to_cut.path).unwrap_or_else(|err| {
                        eprintln!("Failed to delete original directory after cut: {err}");
                    });
                }
                self.dir.scan_and_add(self.show_hidden_files).await.unwrap();
            }
            self.clipboard = None;
        }
    }

    pub fn new_file(&mut self) {
        self.show_new_file = true;
        self.rename_input = TextArea::default();
        self.rename_input
            .set_block(Block::bordered().title("New name"));
    }

    pub async fn handle_rename_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if let Some(old_name) = &self.file_to_rename {
                    let new_name = self.rename_input.lines().join("").trim().to_string();
                    if !new_name.is_empty() {
                        // Remove trailing slash if it's a directory
                        let old_name_clean = old_name.trim_end_matches('/');
                        let old_path = format!("{}/{}", self.dir.path, old_name_clean);
                        let new_path = format!("{}/{}", self.dir.path, new_name);
                        if let Err(err) = fs::rename(&old_path, &new_path) {
                            eprintln!("Failed to rename file: {err}");
                        } else {
                            self.dir.scan_and_add(self.show_hidden_files).await.unwrap();
                        }
                    }
                }
                self.rename_input = TextArea::default();
                self.rename_input
                    .set_block(Block::bordered().title("New name"));
                self.show_rename = false;
                self.file_to_rename = None;
            }
            KeyCode::Esc => {
                self.rename_input = TextArea::default();
                self.rename_input
                    .set_block(Block::bordered().title("New name"));
                self.show_rename = false;
                self.file_to_rename = None;
            }
            _ => {
                // Pass the event to the text area input
                self.rename_input.input(Event::Key(key));
            }
        }
        Ok(())
    }

    pub async fn handle_new_file_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                let new_name = self.new_file_input.lines().join("").trim().to_string();
                if !new_name.is_empty() {
                    let new_path = format!("{}/{}", self.dir.path, new_name);
                    if let Err(err) = fs::File::create(&new_path) {
                        eprintln!("Failed to create file: {err}");
                    } else {
                        self.dir.scan_and_add(self.show_hidden_files).await.unwrap();
                    }
                }
                self.new_file_input = TextArea::default();
                self.new_file_input
                    .set_block(Block::bordered().title("New name"));
                self.show_new_file = false;
            }
            KeyCode::Esc => {
                self.new_file_input = TextArea::default();
                self.new_file_input
                    .set_block(Block::bordered().title("New name"));
                self.show_new_file = false;
            }
            _ => {
                // Pass the event to the text area input
                self.new_file_input.input(Event::Key(key));
            }
        }
        Ok(())
    }
}
