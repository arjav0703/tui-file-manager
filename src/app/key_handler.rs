use super::*;

impl App {
    pub async fn on_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.show_confirmation {
            match key.code {
                KeyCode::Char('y') => {
                    if let Some(file) = &self.file_to_delete {
                        let full_path = format!("{}/{}", self.dir.path, file);
                        if let Err(_e) = fs::remove_file(&full_path) {
                            fs::remove_dir_all(&full_path).unwrap_or_else(|err| {
                                eprintln!("Failed to delete directory: {err}");
                            });
                        }
                        self.dir.scan_and_add(self.show_hidden_files).await.unwrap();
                    }
                    self.show_confirmation = false;
                    self.file_to_delete = None;
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.show_confirmation = false;
                    self.file_to_delete = None;
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                self.update_subdir_preview_async().await;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                self.update_subdir_preview_async().await;
            }
            KeyCode::Right | KeyCode::Char('l') => self.enter_directory().await?,
            KeyCode::Left | KeyCode::Char('h') => self.go_to_parent().await?,
            KeyCode::Enter => self.open_file(),
            KeyCode::Delete | KeyCode::Char('d') | KeyCode::Backspace => self.delete_file().await,
            KeyCode::Char('r') => self.rename_file(),
            KeyCode::Char('y') => self.yank_file(),
            KeyCode::Char('a') => self.new_file(),
            KeyCode::Char('c') => {
                self.handle_copy_file();
            }
            KeyCode::Char('x') => {
                self.handle_cut_file();
            }
            KeyCode::Char('p') => {
                self.handle_paste().await;
            }
            _ => {}
        }
        Ok(())
    }
}
