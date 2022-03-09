use std::io::Stdout;
use std::sync::{Arc, Mutex};
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, BorderType, Paragraph, Row, Table};
use crate::Output;

pub(crate) enum Event<I> {
    Input(I),
    Tick,
}

pub(crate) fn draw_home(rect: &mut Frame<CrosstermBackend<Stdout>>, chunks: &Vec<Rect>, output: &Arc<Mutex<Output>>) {
    let home_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [Constraint::Percentage(70), Constraint::Percentage(30)].as_ref(),
        )
        .split(chunks[0]);

    let home = {
        let output_temp = output.lock().unwrap();
        Paragraph::new(vec![
            Spans::from(vec![Span::styled(
                "Swords And Death",
                Style::default().fg(Color::LightYellow),
            )]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw(output_temp.get(0).unwrap().clone())]),
            Spans::from(vec![Span::raw(output_temp.get(1).unwrap().clone())]),
            Spans::from(vec![Span::raw(output_temp.get(2).unwrap().clone())]),
            Spans::from(vec![Span::raw(output_temp.get(3).unwrap().clone())]),
            Spans::from(vec![Span::raw(output_temp.get(4).unwrap().clone())]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw("Press [Enter] to type an action.")]),
            Spans::from(vec![Span::raw("Actions:")]),
            Spans::from(vec![Span::raw("* Type 'step' to take a step. *")]),
            Spans::from(vec![Span::raw("* Type 'inv' to view your inventory. *")]),
            Spans::from(vec![Span::raw("* Type 'inspect <item>' to inspect an item. *")]),
            Spans::from(vec![Span::raw("* Type 'drop <item>' to drop an item. (THIS CAN'T BE UNDONE) *")]),
            Spans::from(vec![Span::raw("Press 'q' to quit")]),
        ])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("SND")
                    .border_type(BorderType::Plain),
            )
    };

    let home_details = Table::new(vec![Row::new(vec![
        Span::raw(format!("{}", "?")), // Player's Username
        Span::raw(format!("{}", "?")), // Player's Level
        Span::raw(format!("{} / {}", "?", "?")), // Player's EXP
        Span::raw(format!("{} / {}", "?", "?")), // Health
        Span::raw(format!("{}", "?")), // Current Region
        Span::raw(format!("{}", "?")), // Total Steps
    ])])
        .header(Row::new(vec![
            Span::styled(
                "Username",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Level",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Exp",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Health",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Current Region",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Total Steps",
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Stats")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ]);
    rect.render_widget(home, home_chunks[0]);
    rect.render_widget(home_details, home_chunks[1]);
}