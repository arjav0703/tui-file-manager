use super::*;

impl App {
    pub fn select_next(&mut self) {
        let items_len = self.dir.entries().len();
        if items_len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                // in case we reach the botton, go to top
                if i >= items_len - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        let items_len = self.dir.entries().len();
        if items_len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                // in case we reach the top, go to the bottom
                if i == 0 { items_len - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub async fn enter_directory(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                // Check if it's a directory (ends with '/')
                if selected_entry.ends_with('/') {
                    let dir_name = selected_entry.trim_end_matches('/');
                    // Find the subdirectory and navigate into it
                    if let Some(subdir) =
                        self.dir.subdirectories.iter().find(|d| d.name == dir_name)
                    {
                        let new_path = subdir.path.clone();
                        let new_name = subdir.name.clone();
                        self.dir = Directory::new(new_name, new_path);

                        // Scan the new directory
                        self.dir.scan_and_add().await.unwrap();
                        self.list_state.select(Some(0));
                        self.update_subdir_preview_async().await;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn go_to_parent(&mut self) -> Result<()> {
        use std::path::Path;

        let current_path = Path::new(&self.dir.path);
        if let Some(parent) = current_path.parent()
            && let Some(parent_str) = parent.to_str()
        {
            let parent_name = parent
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            self.dir = Directory::new(parent_name, parent_str.to_string());

            // Scan the parent directory
            self.dir.scan_and_add().await.unwrap();
            self.list_state.select(Some(0));
            self.update_subdir_preview_async().await;
        }
        Ok(())
    }

    pub async fn update_subdir_preview_async(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i)
                && selected_entry.ends_with('/')
            {
                let dir_name = selected_entry.trim_end_matches('/');

                if let Some(subdir) = self.dir.subdirectories.iter().find(|d| d.name == dir_name) {
                    let mut preview_dir = Directory::new(subdir.name.clone(), subdir.path.clone());
                    // Scan asynchronously
                    if preview_dir.scan_and_add().await.is_ok() {
                        self.subdir = Some(preview_dir);
                    } else {
                        self.subdir = None;
                    }
                    return;
                }
            }
        }
        self.subdir = None;
    }
}
