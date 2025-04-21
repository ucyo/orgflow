use std::io;

use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::Color;
use ratatui::style::Style;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::Line,
    style::Stylize,
    widgets::{Block, Borders, Widget},
};
use tui_textarea::TextArea;

fn main() -> io::Result<()> {
    // Initialise terminal and move to raw mode
    let mut terminal = ratatui::init();

    // Create app and run for infinite loop
    let mut app = App::new();
    let app_result = app.run(&mut terminal);

    // Disable raw mode
    ratatui::restore();

    // Return application exit code
    app_result
}

struct App {
    exit: bool,
    note: TextArea<'static>,
    title: TextArea<'static>,
    note_focus: NoteFocus,
    tab_index: usize,
}

enum NoteFocus {
    Title,
    Content,
}

impl<'a> App {
    fn new() -> Self {
        let note = TextArea::default();
        let title = TextArea::default();
        let focus = NoteFocus::Title;
        let exit = false;
        App {
            exit,
            note,
            title,
            note_focus: focus,
            tab_index: 0,
        }
    }
    /// Start the application
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Infinite loop until variable set
        while !self.exit {
            // Iterate over frames and draw them one by one
            terminal.draw(|frame| self.draw(frame))?;

            // wait for key events and handle them locally in the application
            match ratatui::crossterm::event::read()? {
                ratatui::crossterm::event::Event::Key(key_event) => {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            }
        }
        Ok(())
    }
    /// Routine about how to draw each frame in application
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Look for key presses and handle event
    fn handle_key_event(
        &mut self,
        key_event: ratatui::crossterm::event::KeyEvent,
    ) -> io::Result<()> {
        match (key_event.kind, key_event.code, &self.note_focus) {
            (KeyEventKind::Press, KeyCode::Char('t'), _)
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.tab_index = (self.tab_index + 1) % 2
            }
            (KeyEventKind::Press, KeyCode::Esc, _) => self.exit = true,
            (KeyEventKind::Press, KeyCode::BackTab, NoteFocus::Content) => {
                self.note_focus = NoteFocus::Title
            }
            (KeyEventKind::Press, KeyCode::BackTab, NoteFocus::Title) => {
                self.note_focus = NoteFocus::Content
            }
            (KeyEventKind::Press, KeyCode::Enter, NoteFocus::Title) => {
                self.note_focus = NoteFocus::Content
            }
            (_, _, NoteFocus::Content) => _ = self.note.input(key_event),
            (_, _, NoteFocus::Title) => _ = self.title.input(key_event),
        }
        Ok(())
    }
}

/// Give App itself the ability to be a Widget (if there is only one widget )
impl<'a> Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.tab_index {
            _ => render_note(self, area, buf),
        }
    }
}

fn render_note(app: &App, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    // Create a vertical layout via length
    let vertical_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(0),
    ]);

    // Split input area in above layout
    let [appname_area, title_area, content_area] = vertical_layout.areas(area);

    // Render title in the vertical area
    Line::from(format!("{}", app.tab_index))
        .bold()
        .centered()
        .render(appname_area, buf);

    // Define title area and its content
    let mut title = TextArea::from(app.title.clone());
    let title_block = Block::default().borders(Borders::ALL).title("Titel");
    let title_block = match app.note_focus {
        NoteFocus::Title => title_block.style(Style::default().fg(Color::Yellow)),
        _ => title_block,
    };

    // Define content for the note inputs: content (text_area), title (instructions), border (block)
    let mut text_area = TextArea::from(app.note.clone());
    let note_instructions = Line::from(vec![
        " Quit ".into(),
        "<ESC> ".blue().bold(),
        "Switch ".into(),
        "<SHIFT>+<TAB> ".blue().bold(),
        "Switch Tabs ".into(),
        "<CTRL>+<T> ".blue().bold(),
    ])
    .centered();
    let note_block = Block::default()
        .borders(Borders::ALL)
        .title("Content")
        .title_bottom(note_instructions);
    let note_block = match app.note_focus {
        NoteFocus::Content => note_block.style(Style::default().fg(Color::Yellow)),
        _ => note_block,
    };

    // Render each of the contents
    text_area.set_block(note_block);
    text_area.render(content_area, buf);

    title.set_block(title_block);
    title.render(title_area, buf);
}
