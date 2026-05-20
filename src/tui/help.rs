use ratatui::{
    layout::Rect,
    style::Color,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_help(area: Rect, f: &mut Frame) {
    let p = Paragraph::new(vec![
        Line::from("Keybinds:"),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("q", Style::default().fg(Color::Magenta)),
            Span::raw(" / "),
            Span::styled("Ctrl-C", Style::default().fg(Color::Magenta)),
            Span::raw("  Quit"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("r", Style::default().fg(Color::Magenta)),
            Span::raw("           Rerun"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("p", Style::default().fg(Color::Magenta)),
            Span::raw("           Pause/Resume"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("s", Style::default().fg(Color::Magenta)),
            Span::raw("           Save JSON"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("a", Style::default().fg(Color::Magenta)),
            Span::raw("           Toggle auto-save"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("H", Style::default().fg(Color::Magenta)),
            Span::raw("           Toggle redaction of identifying network info"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("tab", Style::default().fg(Color::Magenta)),
            Span::raw("         Switch tabs"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("?", Style::default().fg(Color::Magenta)),
            Span::raw("           Show this help"),
        ]),
        Line::from(""),
        Line::from("History tab:"),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("\u{2191}/\u{2193}", Style::default().fg(Color::Magenta)),
            Span::raw(" or "),
            Span::styled("j/k", Style::default().fg(Color::Magenta)),
            Span::raw("  Navigate"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Enter", Style::default().fg(Color::Magenta)),
            Span::raw("       Open JSON detail view"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Space", Style::default().fg(Color::Magenta)),
            Span::raw("       Open actions menu (view, comment, export, delete)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("/", Style::default().fg(Color::Magenta)),
            Span::raw("           Filter history"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("r", Style::default().fg(Color::Magenta)),
            Span::raw("           Refresh history"),
        ]),
        Line::from(""),
        Line::from("Repository (update your tool or report issues here):"),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "https://github.com/kavehtehrani/cloudflare-speed-cli",
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(p, area);
}
