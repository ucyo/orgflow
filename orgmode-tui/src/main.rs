use std::io;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
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
}

impl<'a> App {
    fn new() -> Self {
        let ta = TextArea::default();
        let title = TextArea::default();
        App {
            exit: false,
            note: ta,
            title,
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
        match (key_event.kind, key_event.code) {
            (KeyEventKind::Press, KeyCode::Esc) => self.exit = true,
            _ => self.handle_other_key_event(key_event)?, // Handle all other key events by sending them to content
                                                          // TODO: Should check which layout is active
        }
        Ok(())
    }

    /// Handle content input to textarea
    fn handle_other_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        self.note.input(key_event);
        Ok(())
    }
}

/// Give App itself the ability to be a Widget (if there is only one widget )
impl<'a> Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // Create a vertical layout via percentages
        let vertical_layout = Layout::vertical([
            Constraint::Percentage(5),
            Constraint::Percentage(15),
            Constraint::Percentage(80),
        ]);

        // Split input area in above layout
        let [appname_area, title_area, content_area] = vertical_layout.areas(area);

        // Render title in the vertical area
        Line::from("Orgmode").bold().render(appname_area, buf);

        // Define title area and its content
        let mut title = TextArea::from(self.title.clone());
        let title_block = Block::default().borders(Borders::ALL).title("Titel");
        title.set_block(title_block);
        title.render(
            Rect {
                x: title_area.left(),
                y: title_area.top(),
                width: title_area.width,
                height: 3,
            },
            buf,
        );

        // Define content for the note inputs: content (text_area), title (instructions), border (block)
        let mut text_area = TextArea::from(self.note.clone());
        let note_instructions =
            Line::from(vec![" Quit ".into(), "<ESC> ".blue().bold()]).centered();
        let note_block = Block::default()
            .borders(Borders::ALL)
            .title("Content")
            .title_bottom(note_instructions);

        // Render each of the contents
        text_area.set_block(note_block);
        text_area.render(content_area, buf);
    }
}
