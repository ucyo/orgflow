use std::io;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
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
    focus: AppFocus,
}

#[derive(PartialEq)]
enum AppFocus {
    Title,
    Content,
}

impl<'a> App {
    fn new() -> Self {
        let note = TextArea::default();
        let title = TextArea::default();
        let focus = AppFocus::Title;
        let exit = false;
        App {
            exit,
            note,
            title,
            focus,
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
        match (key_event.kind, key_event.code, &self.focus) {
            (KeyEventKind::Press, KeyCode::Esc, _) => self.exit = true,
            (KeyEventKind::Press, KeyCode::Tab, AppFocus::Content) => self.focus = AppFocus::Title,
            (KeyEventKind::Press, KeyCode::Tab, AppFocus::Title) => self.focus = AppFocus::Content,
            (KeyEventKind::Press, KeyCode::Enter, AppFocus::Title) => {
                self.focus = AppFocus::Content
            }
            (_, _, AppFocus::Content) => _ = self.note.input(key_event),
            (_, _, AppFocus::Title) => _ = self.title.input(key_event),
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
        let title_block = match self.focus {
            AppFocus::Title => title_block.style(Style::default().fg(Color::Yellow)),
            _ => title_block,
        };

        // Define content for the note inputs: content (text_area), title (instructions), border (block)
        let mut text_area = TextArea::from(self.note.clone());
        let note_instructions =
            Line::from(vec![" Quit ".into(), "<ESC> ".blue().bold()]).centered();
        let note_block = Block::default()
            .borders(Borders::ALL)
            .title("Content")
            .title_bottom(note_instructions);
        let note_block = match self.focus {
            AppFocus::Content => note_block.style(Style::default().fg(Color::Yellow)),
            _ => note_block,
        };

        // Render each of the contents
        text_area.set_block(note_block);
        text_area.render(content_area, buf);

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
    }
}
