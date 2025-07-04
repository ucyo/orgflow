use orgflow::{Configuration, Note, OrgDocument, Task, TagSuggestions, Tag, TagCollection};
use std::io;
use std::io::Result as IoResult;
use std::str::FromStr;

mod session;
use session::{SessionManager, SessionState};

mod autocompletion;
use autocompletion::AutocompletionWidget;

use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Direction, Rect};
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
    let mut app = App::new()?;
    let app_result = app.run(&mut terminal);

    // Disable raw mode
    ratatui::restore();

    // Return application exit code
    app_result
}

#[derive(Debug)]
struct App {
    document: OrgDocument,
    exit: bool,
    note: TextArea<'static>,
    title: TextArea<'static>,
    note_focus: NoteFocus,
    scratchpad: TextArea<'static>,
    scratchpad_visible: bool,
    current_tab: AppTab,
    current_note_index: usize,
    current_task_index: usize,
    session_manager: SessionManager,
    document_path: String,
    has_unsaved_changes: bool,
    tag_suggestions: TagSuggestions,
    autocompletion: AutocompletionWidget,          // For scratchpad
    title_autocompletion: AutocompletionWidget,    // For note titles
}

#[derive(Debug)]
enum AppTab {
    Editor,
    Viewer,
    Tasks,
}

#[derive(Debug, PartialEq)]
enum NoteFocus {
    Title,
    Content,
}

impl<'a> App {
    fn new() -> IoResult<Self> {
        let basefolder = Configuration::basefolder();

        // Ensure base folder exists with better error handling
        if let Err(e) = std::fs::create_dir_all(&basefolder) {
            eprintln!("Failed to create base folder '{}': {}", basefolder, e);
            eprintln!("Try setting ORGFLOW_BASEFOLDER to a writable directory:");
            eprintln!("  export ORGFLOW_BASEFOLDER=/tmp/orgflow");
            return Err(e);
        }

        let refile_path = std::path::Path::new(&basefolder).join("refile.org");
        let document_path = refile_path.to_str().unwrap().to_string();

        // Load document or create empty one if file doesn't exist
        let document = match OrgDocument::from(&document_path) {
            Ok(doc) => doc,
            Err(_) => OrgDocument::default(), // Create empty document if file doesn't exist
        };

        // Initialize session manager
        let session_file_path = std::path::Path::new(&basefolder).join("session.json");
        let mut session_manager =
            SessionManager::new(session_file_path.to_str().unwrap().to_string());

        // Load existing session or create default
        let session_state = match session_manager.load_session() {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Warning: Failed to load session, starting fresh: {}", e);
                SessionState::default()
            }
        };

        // Restore UI state from session
        let current_tab = session_state.current_tab;
        // Ensure indices are within bounds for current document
        let current_note_index = if session_state.current_note_index < document.notes.len() {
            session_state.current_note_index
        } else {
            0
        };
        let current_task_index = if session_state.current_task_index < document.tasks.len() {
            session_state.current_task_index
        } else {
            0
        };
        let note_focus = session_state.note_focus;
        let scratchpad_visible = session_state.scratchpad_visible;

        // Restore draft content from session with cursor positions
        let title = SessionManager::restore_textarea_with_cursor(
            &session_state.title_content,
            session_state.title_cursor_pos,
        );
        let note = SessionManager::restore_textarea_with_cursor(
            &session_state.note_content,
            session_state.note_cursor_pos,
        );
        let scratchpad = SessionManager::restore_textarea_with_cursor(
            &session_state.scratchpad_content,
            session_state.scratchpad_cursor_pos,
        );

        // Extract tag suggestions from document
        let tag_suggestions = document.collect_unique_tags();
        let autocompletion = AutocompletionWidget::new();
        let title_autocompletion = AutocompletionWidget::new();

        let app = App {
            document,
            exit: false,
            note,
            title,
            note_focus,
            scratchpad,
            scratchpad_visible,
            current_tab,
            current_note_index,
            current_task_index,
            session_manager,
            document_path,
            has_unsaved_changes: session_state.has_unsaved_changes,
            tag_suggestions,
            autocompletion,
            title_autocompletion,
        };
        Ok(app)
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
                    self.handle_key_event(key_event)?;

                    // Update session state after each keystroke
                    self.update_session_state();

                    // Check if we should save session (debounced)
                    if self.session_manager.should_save() {
                        let _ = self.session_manager.save_session();
                    }
                }
                _ => {}
            }
        }

        // Force save session on exit
        let _ = self.session_manager.force_save();
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
        match (
            key_event.kind,
            key_event.code,
            &self.current_tab,
            &self.note_focus,
        ) {
            // Tab switching with Ctrl+Tab (cycles through tabs) - only when scratchpad is NOT visible
            (KeyEventKind::Press, KeyCode::Char('r'), _, _)
                if key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && !self.scratchpad_visible =>
            {
                self.current_tab = match self.current_tab {
                    AppTab::Editor => {
                        // Reset note index if out of bounds when switching to Viewer
                        if self.current_note_index >= self.document.notes.len() {
                            self.current_note_index = 0;
                        }
                        AppTab::Viewer
                    }
                    AppTab::Viewer => {
                        // Reset task index if out of bounds when switching to Tasks
                        if self.current_task_index >= self.document.tasks.len() {
                            self.current_task_index = 0;
                        }
                        AppTab::Tasks
                    }
                    AppTab::Tasks => AppTab::Editor,
                };
            }
            // Arrow navigation in viewer tab
            (KeyEventKind::Press, KeyCode::Left, AppTab::Viewer, _) => {
                if self.current_note_index > 0 {
                    self.current_note_index -= 1;
                }
            }
            (KeyEventKind::Press, KeyCode::Right, AppTab::Viewer, _) => {
                if self.current_note_index < self.document.notes.len().saturating_sub(1) {
                    self.current_note_index += 1;
                }
            }
            // Arrow navigation in tasks tab
            (KeyEventKind::Press, KeyCode::Up, AppTab::Tasks, _) => {
                if self.current_task_index > 0 {
                    self.current_task_index -= 1;
                }
            }
            (KeyEventKind::Press, KeyCode::Down, AppTab::Tasks, _) => {
                if self.current_task_index < self.document.tasks.len().saturating_sub(1) {
                    self.current_task_index += 1;
                }
            }
            (KeyEventKind::Press, KeyCode::Char('t'), _, _)
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.scratchpad_visible = !self.scratchpad_visible;
            }
            // Ctrl+S save - put this early to ensure it's not intercepted
            (KeyEventKind::Press, KeyCode::Char('s'), _, _)
                if key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && !self.scratchpad_visible =>
            {
                self.save_note()?;
            }
            (KeyEventKind::Press, KeyCode::Esc, _, _) if self.scratchpad_visible && self.autocompletion.is_visible() => {
                // Hide autocompletion but don't close scratchpad
                self.autocompletion.hide();
            }
            (KeyEventKind::Press, KeyCode::Esc, AppTab::Editor, NoteFocus::Title) if self.title_autocompletion.is_visible() => {
                // Hide title autocompletion
                self.title_autocompletion.hide();
            }
            (KeyEventKind::Press, KeyCode::Esc, _, _) => {
                if self.scratchpad_visible {
                    // First ESC closes the scratchpad
                    self.scratchpad_visible = false;
                } else {
                    // Second ESC (or first ESC when scratchpad isn't visible) exits the app
                    self.exit = true;
                }
            }
            (KeyEventKind::Press, KeyCode::Enter, _, _) if self.scratchpad_visible => {
                let task = self.scratchpad.lines().first().unwrap();
                let t = Task::with_today(task);
                self.document.push_task(t);

                // Save to file immediately
                let _ = self.document.to(&self.document_path);

                self.scratchpad = TextArea::default();
                self.has_unsaved_changes = false;
                
                // Update tag suggestions after adding new task
                self.tag_suggestions = self.document.collect_unique_tags();
            }
            // Autocompletion handling in scratchpad
            (KeyEventKind::Press, KeyCode::Up, _, _) if self.scratchpad_visible && self.autocompletion.is_visible() => {
                self.autocompletion.select_previous();
            }
            (KeyEventKind::Press, KeyCode::Down, _, _) if self.scratchpad_visible && self.autocompletion.is_visible() => {
                self.autocompletion.select_next();
            }
            (KeyEventKind::Press, KeyCode::Tab, _, _) if self.scratchpad_visible && self.autocompletion.is_visible() => {
                // Apply the selected suggestion
                if let Some((new_text, _cursor_pos)) = self.autocompletion.apply_selected(&self.scratchpad.lines().join(" ")) {
                    // Replace the text content
                    self.scratchpad = TextArea::from(vec![new_text]);
                    // Move cursor to the end of the inserted tag
                    self.scratchpad.move_cursor(tui_textarea::CursorMove::End);
                    self.autocompletion.hide();
                }
            }
            (_, _, _, _) if self.scratchpad_visible => {
                self.scratchpad.input(key_event);
                // Update autocompletion suggestions after input
                let current_text = self.scratchpad.lines().join(" ");
                self.autocompletion.update_suggestions(&current_text, &self.tag_suggestions);
            }
            // Editor tab specific key handling
            (KeyEventKind::Press, KeyCode::BackTab, AppTab::Editor, NoteFocus::Content) => {
                self.note_focus = NoteFocus::Title
            }
            (KeyEventKind::Press, KeyCode::BackTab, AppTab::Editor, NoteFocus::Title) => {
                self.note_focus = NoteFocus::Content
            }
            (KeyEventKind::Press, KeyCode::Enter, AppTab::Editor, NoteFocus::Title) => {
                self.note_focus = NoteFocus::Content
            }
            // Title autocompletion handling
            (KeyEventKind::Press, KeyCode::Up, AppTab::Editor, NoteFocus::Title) if self.title_autocompletion.is_visible() => {
                self.title_autocompletion.select_previous();
            }
            (KeyEventKind::Press, KeyCode::Down, AppTab::Editor, NoteFocus::Title) if self.title_autocompletion.is_visible() => {
                self.title_autocompletion.select_next();
            }
            (KeyEventKind::Press, KeyCode::Tab, AppTab::Editor, NoteFocus::Title) if self.title_autocompletion.is_visible() => {
                // Apply the selected suggestion
                if let Some((new_text, _cursor_pos)) = self.title_autocompletion.apply_selected(&self.title.lines().join(" ")) {
                    self.title = TextArea::from(vec![new_text]);
                    self.title.move_cursor(tui_textarea::CursorMove::End);
                    self.title_autocompletion.hide();
                }
            }
            (KeyEventKind::Press, KeyCode::Tab, AppTab::Editor, NoteFocus::Title) => {
                self.note_focus = NoteFocus::Content
            }
            (_, _, AppTab::Editor, NoteFocus::Content) => _ = self.note.input(key_event),
            (_, _, AppTab::Editor, NoteFocus::Title) => {
                self.title.input(key_event);
                // Update autocompletion suggestions after input
                let current_text = self.title.lines().join(" ");
                self.title_autocompletion.update_suggestions(&current_text, &self.tag_suggestions);
            }
            // Ignore other inputs in viewer mode
            (_, _, AppTab::Viewer, _) => {}
            // Ignore other inputs in tasks mode
            (_, _, AppTab::Tasks, _) => {}
        }
        Ok(())
    }

    /// Extract tags from text (title or content)
    fn extract_tags_from_text(&self, text: &str) -> Vec<Tag> {
        let mut tags = Vec::new();
        
        // Split text into words and look for tag patterns
        for word in text.split_whitespace() {
            if let Ok(tag) = Tag::from_str(word) {
                tags.push(tag);
            }
        }
        
        tags
    }

    /// Remove tags from text, returning the cleaned text
    fn remove_tags_from_text(&self, text: &str) -> String {
        text.split_whitespace()
            .filter(|word| Tag::from_str(word).is_err()) // Keep words that are NOT valid tags
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn save_note(&mut self) -> io::Result<()> {
        let title = self.title.lines().join(" ");
        let content: Vec<String> = self.note.lines().iter().map(|s| s.to_string()).collect();

        // Check if we have any meaningful content
        let has_title = !title.trim().is_empty();
        let has_content = content.iter().any(|line| !line.trim().is_empty());

        if has_title || has_content {
            // Extract tags from title and content
            let mut extracted_tags = Vec::new();
            extracted_tags.extend(self.extract_tags_from_text(&title));
            for line in &content {
                extracted_tags.extend(self.extract_tags_from_text(line));
            }

            // Remove tags from title to get clean title
            let clean_title = self.remove_tags_from_text(&title);
            let final_title = if clean_title.trim().is_empty() {
                "Untitled Note".to_string()
            } else {
                clean_title
            };

            // Remove tags from content to get clean content
            let clean_content: Vec<String> = content
                .iter()
                .map(|line| self.remove_tags_from_text(line))
                .filter(|line| !line.trim().is_empty()) // Remove empty lines
                .collect();

            // Create note with extracted tags
            let note = if !extracted_tags.is_empty() {
                let tag_collection = TagCollection::from_tags(extracted_tags);
                Note::with_tags(final_title, clean_content, tag_collection)
            } else {
                Note::with(final_title, clean_content)
            };
            
            self.document.push_note(note);

            // Save to file
            self.document.to(&self.document_path)?;

            // Clear the text areas
            self.title = TextArea::default();
            self.note = TextArea::default();
            self.note_focus = NoteFocus::Title;
            self.has_unsaved_changes = false;
            
            // Update tag suggestions after adding new note
            self.tag_suggestions = self.document.collect_unique_tags();
        }
        Ok(())
    }

    /// Update session state with current application state
    fn update_session_state(&mut self) {
        // Check if there are unsaved changes in text areas
        let has_draft_content = !self.title.lines().is_empty()
            || !self.note.lines().is_empty()
            || !self.scratchpad.lines().is_empty();

        let has_unsaved = self.has_unsaved_changes || has_draft_content;

        self.session_manager.update_state(
            &self.current_tab,
            self.current_note_index,
            self.current_task_index,
            &self.note_focus,
            self.scratchpad_visible,
            &self.title,
            &self.note,
            &self.scratchpad,
            &self.document_path,
            has_unsaved,
        );
    }
}

/// Give App itself the ability to be a Widget (if there is only one widget )
impl<'a> Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.current_tab {
            AppTab::Editor => render_note_editor(self, area, buf),
            AppTab::Viewer => render_note_viewer(self, area, buf),
            AppTab::Tasks => render_task_viewer(self, area, buf),
        }
    }
}

fn render_note_editor(app: &App, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    // Create a vertical layout via length
    let vertical_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(0),
    ]);

    // Split input area in above layout
    let [appname_area, title_area, content_area] = vertical_layout.areas(area);

    // Render title in the vertical area
    Line::from("Orgflow - Editor | Viewer | Tasks (Ctrl+R to switch)")
        .bold()
        .centered()
        .render(appname_area, buf);

    // Define title area and its content
    let mut title = TextArea::from(app.title.clone());
    let title_block = Block::default().borders(Borders::ALL).title("Title");
    let title_block = match app.note_focus {
        NoteFocus::Title if !app.scratchpad_visible => {
            title_block.style(Style::default().fg(Color::Yellow))
        }
        _ => title_block,
    };

    // Define content for the note inputs: content (text_area), title (instructions), border (block)
    let mut text_area = TextArea::from(app.note.clone());
    let note_instructions = Line::from(vec![
        " Quit ".into(),
        "<ESC> ".blue().bold(),
        "Switch ".into(),
        "<SHIFT>+<TAB> ".blue().bold(),
        "Save Note ".into(),
        "<CTRL>+<S> ".blue().bold(),
        "Enter Task ".into(),
        "<CTRL>+<T> ".blue().bold(),
        "Switch ".into(),
        "<CTRL>+<R> ".blue().bold(),
    ])
    .centered();
    let note_block = Block::default()
        .borders(Borders::ALL)
        .title("Content")
        .title_bottom(note_instructions);
    let note_block = match app.note_focus {
        NoteFocus::Content if !app.scratchpad_visible => {
            note_block.style(Style::default().fg(Color::Yellow))
        }
        _ => note_block,
    };

    let mut scratchpad = TextArea::from(app.scratchpad.clone());
    let scratchpad_block = Block::default()
        .borders(Borders::ALL)
        .title("Task")
        .style(Style::default().fg(Color::Yellow));

    let scratchpad_area = centered_rect(60, 10, area);

    if app.scratchpad_visible {
        scratchpad.set_block(scratchpad_block);
        scratchpad.render(scratchpad_area, buf);
        
        // Render autocompletion popup if visible
        if app.autocompletion.is_visible() {
            // Calculate cursor position within the scratchpad
            let cursor_line = scratchpad.cursor().0;
            let cursor_col = scratchpad.cursor().1;
            let cursor_pos = (
                scratchpad_area.x + 1 + cursor_col as u16, // +1 for border
                scratchpad_area.y + 1 + cursor_line as u16, // +1 for border
            );
            app.autocompletion.render(area, buf, cursor_pos);
        }
    }

    // Render each of the contents
    text_area.set_block(note_block);
    text_area.render(content_area, buf);

    title.set_block(title_block);
    title.render(title_area, buf);
    
    // Render title autocompletion popup if visible
    if app.title_autocompletion.is_visible() && app.note_focus == NoteFocus::Title && !app.scratchpad_visible {
        // Calculate cursor position within the title
        let cursor_line = title.cursor().0;
        let cursor_col = title.cursor().1;
        let cursor_pos = (
            title_area.x + 1 + cursor_col as u16, // +1 for border
            title_area.y + 1 + cursor_line as u16, // +1 for border
        );
        app.title_autocompletion.render(area, buf, cursor_pos);
    }
}

fn render_note_viewer(app: &App, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    // Create a vertical layout
    let vertical_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(0),
    ]);

    // Split input area in above layout
    let [appname_area, navigation_area, main_area] = vertical_layout.areas(area);

    // Render title in the vertical area
    Line::from("Orgflow - Editor | Viewer | Tasks (Ctrl+R to switch)")
        .bold()
        .centered()
        .render(appname_area, buf);

    // Show current note info and navigation
    let note_count = app.document.notes.len();
    let current_index = app.current_note_index;

    let navigation_content = if note_count == 0 {
        vec!["No notes available".to_string()]
    } else {
        vec![format!(
            "Note {} of {} (Use ←→ arrows to navigate)",
            current_index + 1,
            note_count
        )]
    };

    let navigation_block = Block::default()
        .borders(Borders::ALL)
        .title("Navigation")
        .style(Style::default().fg(Color::Yellow));

    let mut navigation_display = TextArea::from(navigation_content);
    navigation_display.set_block(navigation_block);
    navigation_display.render(navigation_area, buf);

    if note_count == 0 {
        // Show empty state
        let empty_block = Block::default()
            .borders(Borders::ALL)
            .title("No Notes")
            .title_bottom(
                Line::from(vec![
                    " Quit ".into(),
                    "<ESC> ".blue().bold(),
                    "Switch ".into(),
                    "<CTRL>+<TAB> ".blue().bold(),
                ])
                .centered(),
            );

        let mut empty_display = TextArea::from(vec!["No notes to display".to_string()]);
        empty_display.set_block(empty_block);
        empty_display.render(main_area, buf);
        return;
    }

    // Create horizontal layout for content and metadata
    let horizontal_layout =
        Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]);

    let [content_area, metadata_area] = horizontal_layout.areas(main_area);

    // Create vertical layout for content area (title + content)
    let content_vertical = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]);

    let [title_area, note_content_area] = content_vertical.areas(content_area);

    if let Some(note) = app.document.notes.get(current_index) {
        // Display note title
        let title_block = Block::default().borders(Borders::ALL).title("Title");

        let mut title_display = TextArea::from(vec![note.title().to_string()]);
        title_display.set_block(title_block);
        title_display.render(title_area, buf);

        // Display note content
        let content_block = Block::default()
            .borders(Borders::ALL)
            .title("Content")
            .title_bottom(
                Line::from(vec![
                    " Quit ".into(),
                    "<ESC> ".blue().bold(),
                    "Switch ".into(),
                    "<CTRL>+<TAB> ".blue().bold(),
                ])
                .centered(),
            );

        let content_lines: Vec<String> = note.content().iter().cloned().collect();
        let mut content_display = TextArea::from(content_lines);
        content_display.set_block(content_block);
        content_display.render(note_content_area, buf);

        // Display metadata
        let metadata_lines = vec![
            format!("Level: {}", note.level()),
            format!("Created: {}", note.creation_date()),
            format!("Modified: {}", note.modification_date()),
            format!("GUID: {}", note.guid()),
            format!("Tags: {}", note.tags()),
        ];

        let metadata_block = Block::default().borders(Borders::ALL).title("Metadata");

        let mut metadata_display = TextArea::from(metadata_lines);
        metadata_display.set_block(metadata_block);
        metadata_display.render(metadata_area, buf);
    }
}

fn render_task_viewer(app: &App, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    // Create a vertical layout
    let vertical_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);

    // Split input area in above layout
    let [appname_area, main_area] = vertical_layout.areas(area);

    // Render title in the vertical area
    Line::from("Orgflow - Editor | Viewer | Tasks (Ctrl+R to switch)")
        .bold()
        .centered()
        .render(appname_area, buf);

    let task_count = app.document.tasks.len();
    let current_index = app.current_task_index;

    if task_count == 0 {
        // Show empty state
        let empty_block = Block::default()
            .borders(Borders::ALL)
            .title("No Tasks")
            .title_bottom(
                Line::from(vec![
                    " Quit ".into(),
                    "<ESC> ".blue().bold(),
                    "Switch ".into(),
                    "<CTRL>+<TAB> ".blue().bold(),
                ])
                .centered(),
            );

        let mut empty_display = TextArea::from(vec!["No tasks to display".to_string()]);
        empty_display.set_block(empty_block);
        empty_display.render(main_area, buf);
        return;
    }

    // Create horizontal layout for task list and metadata
    let horizontal_layout =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]);

    let [task_list_area, metadata_area] = horizontal_layout.areas(main_area);

    // Display task list with current selection highlighted
    let task_list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Tasks ({} total)", task_count))
        .title_bottom(
            Line::from(vec![
                " Quit ".into(),
                "<ESC> ".blue().bold(),
                "Navigate ".into(),
                "<↑↓> ".blue().bold(),
                "Switch ".into(),
                "<CTRL>+<TAB> ".blue().bold(),
            ])
            .centered(),
        );

    // Create content area for the task list
    let inner_area = task_list_block.inner(task_list_area);
    task_list_block.render(task_list_area, buf);

    // Render each task line with appropriate styling
    for (i, task) in app.document.tasks.iter().enumerate() {
        if i >= inner_area.height as usize {
            break; // Don't render beyond the available space
        }

        let y = inner_area.y + i as u16;
        let prefix = if i == current_index { "► " } else { "  " };
        let status = if task.is_completed() { "[x]" } else { "[ ]" };
        let text = format!("{}{} {}", prefix, status, task.description());

        let style = if i == current_index {
            Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED)
        } else {
            Style::default()
        };

        Line::from(text).style(style).render(
            ratatui::layout::Rect {
                x: inner_area.x,
                y,
                width: inner_area.width,
                height: 1,
            },
            buf,
        );
    }

    // Display metadata for current task
    if let Some(task) = app.document.tasks.get(current_index) {
        let mut metadata_lines = vec![format!(
            "Status: {}",
            if task.is_completed() {
                "Completed"
            } else {
                "Pending"
            }
        )];

        if let Some(priority) = task.priority_level() {
            metadata_lines.push(format!("Priority: {}", priority));
        } else {
            metadata_lines.push("Priority: None".to_string());
        }

        if let Some(creation_date) = task.creation_date() {
            metadata_lines.push(format!("Created: {}", creation_date));
        } else {
            metadata_lines.push("Created: Unknown".to_string());
        }

        if let Some(completion_date) = task.completion_date() {
            metadata_lines.push(format!("Completed: {}", completion_date));
        } else {
            metadata_lines.push("Completed: N/A".to_string());
        }

        if let Some(tags) = task.tags() {
            metadata_lines.push(format!("Tags: {}", tags));
        } else {
            metadata_lines.push("Tags: None".to_string());
        }

        metadata_lines.push("".to_string());
        metadata_lines.push("Description:".to_string());
        metadata_lines.push(task.description().to_string());

        let metadata_block = Block::default().borders(Borders::ALL).title("Task Details");

        let mut metadata_display = TextArea::from(metadata_lines);
        metadata_display.set_block(metadata_block);
        metadata_display.render(metadata_area, buf);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Length(3),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
