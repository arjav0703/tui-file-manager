use super::*;

pub struct ConfirmationDialog {
    pub message: String,
}

impl Widget for ConfirmationDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::style::Color;
        use ratatui::widgets::BorderType;

        let block = Block::bordered()
            .title(" ⚠️  Confirm Deletion ")
            .title_style(Style::new().bold().red())
            .border_type(BorderType::Rounded)
            .border_style(Style::new().red())
            .style(Style::new().bg(Color::Rgb(40, 20, 20)));
        let inner = block.inner(area);
        block.render(area, buf);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(self.message.as_str())
                .centered()
                .style(Style::new().bold().white()),
            Line::from(""),
            Line::from(" Press Y to confirm, N to cancel ")
                .centered()
                .style(Style::new().fg(Color::Rgb(150, 150, 150))),
        ]);
        Paragraph::new(text).centered().render(inner, buf);
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    horizontal[1]
}
