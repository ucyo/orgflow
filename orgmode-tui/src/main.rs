use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::Line,
    style::Stylize,
    widgets::{Widget, block::title},
};

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
}

impl App {
    fn new() -> Self {
        App { exit: false }
    }
    /// Start the application
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Infinite loop until variable set
        while !self.exit {
            // Iterate over frames and draw them one by one
            terminal.draw(|frame| self.draw(frame))?;

            // wait for key events and handle them locally in the application
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event)?,
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
    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match (key_event.kind, key_event.code) {
            (KeyEventKind::Press, KeyCode::Char('q')) => self.exit = true,
            _ => (),
        }
        Ok(())
    }
}

/// Give App itself the ability to be a Widget (if there is only one widget )
impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // Create a vertical layout via percentages
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);

        // Split input area in above layout
        let [title_area, content_area] = vertical_layout.areas(area);

        // Render contents in the verical areas
        Line::from("Orgmode").bold().render(title_area, buf);
        Line::from("Content").bold().render(content_area, buf);
    }
}
