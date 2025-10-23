use super::*;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        use ratatui::style::Color;
        use ratatui::widgets::BorderType;

        let items = self.dir.entries_with_symbols();

        // Main directory list with styled border
        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(format!(" 📁 {} ", self.dir.path))
                    .title_style(Style::new().bold().cyan())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().cyan()),
            )
            .style(Style::new().white())
            .highlight_style(
                Style::new()
                    .bg(Color::Rgb(60, 60, 80))
                    .fg(Color::Rgb(255, 215, 0))
                    .bold(),
            )
            .highlight_symbol("▶ ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        let helper_text = Text::from(
            Line::from(vec![
                " q:Quit ".into(),
                "│".dark_gray(),
                " ↑↓/jk:Nav ".into(),
                "│".dark_gray(),
                " ←→/hl:Dir ".into(),
                "│".dark_gray(),
                " Enter:Open ".into(),
                "│".dark_gray(),
                " d:Del ".into(),
                "│".dark_gray(),
                " r:Rename ".into(),
                " y:Yank ".into(),
                "│".dark_gray(),
                " a:New ".into(),
                "│".dark_gray(),
                " c:Copy ".into(),
                "│".dark_gray(),
                " x:Cut ".into(),
                "│".dark_gray(),
                " p:Paste ".into(),
            ])
            .style(Style::new().fg(Color::Rgb(200, 200, 200))),
        );

        // Render main list
        frame.render_stateful_widget(
            list,
            Rect {
                x: 0,
                y: 0,
                width: frame.area().width / 2,
                height: frame.area().height - 3,
            },
            &mut self.list_state,
        );

        // Preview panel
        let items2 = if let Some(subdir) = &self.subdir {
            subdir.entries_with_symbols()
        } else {
            vec!["   No preview available".to_string()]
        };

        let preview_title = if let Some(subdir) = &self.subdir {
            format!(" 👁  Preview: {} ", subdir.name)
        } else {
            " 👁  Preview ".to_string()
        };

        let list2 = List::new(items2)
            .block(
                Block::bordered()
                    .title(preview_title)
                    .title_style(Style::new().bold().magenta())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().magenta()),
            )
            .style(Style::new().fg(Color::Rgb(180, 180, 200)))
            .direction(ListDirection::TopToBottom);

        frame.render_widget(
            list2,
            Rect {
                x: frame.area().width / 2,
                y: 0,
                width: frame.area().width / 2,
                height: frame.area().height - 3,
            },
        );

        // Status bar at bottom
        frame.render_widget(
            Paragraph::new(helper_text).centered().block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().green()),
            ),
            Rect {
                x: 0,
                y: frame.area().height - 3,
                width: frame.area().width,
                height: 3,
            },
        );

        // Render confirmation overlay if active
        if self.show_confirmation {
            let area = centered_rect(50, 20, frame.area());
            let msg = if let Some(file) = &self.file_to_delete {
                format!("Delete '{}'? (y/n)", file)
            } else {
                "Delete file? (y/n)".to_string()
            };

            let dialog = ConfirmationDialog { message: msg };

            frame.render_widget(dialog, area);
        }

        if self.show_rename {
            use ratatui::style::Color;
            use ratatui::widgets::BorderType;

            let area = centered_rect(60, 25, frame.area());
            let block = Block::bordered()
                .title(" ✏️  Rename File ")
                .title_style(Style::new().bold().yellow())
                .border_type(BorderType::Rounded)
                .border_style(Style::new().yellow())
                .style(Style::new().bg(Color::Rgb(30, 30, 40)));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            frame.render_widget(&self.rename_input, inner);
        }

        if self.show_new_file {
            use ratatui::style::Color;
            use ratatui::widgets::BorderType;

            let area = centered_rect(60, 25, frame.area());
            let block = Block::bordered()
                .title(" ➕ New File ")
                .title_style(Style::new().bold().green())
                .border_type(BorderType::Rounded)
                .border_style(Style::new().green())
                .style(Style::new().bg(Color::Rgb(30, 30, 40)));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            frame.render_widget(&self.new_file_input, inner);
        }
    }
}
