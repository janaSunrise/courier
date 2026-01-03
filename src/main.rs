use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(draw)?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                break;
            }
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame) {
    let area = frame.area();

    let layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .split(area);

    let hello = Paragraph::new(Text::raw("Hello, World!"))
        .style(Style::default().fg(Color::Cyan))
        .centered()
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(hello, layout[1]);

    let help = Paragraph::new("Press 'q' to quit")
        .style(Style::default().fg(Color::DarkGray))
        .centered();

    frame.render_widget(help, layout[2]);
}
