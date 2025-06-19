use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tui_textarea::TextArea;

use crate::{AppTab, NoteFocus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    // UI State
    pub current_tab: AppTab,
    pub current_note_index: usize,
    pub current_task_index: usize,
    pub note_focus: NoteFocus,
    pub scratchpad_visible: bool,
    
    // Draft Content (unsaved work)
    pub title_content: Vec<String>,
    pub note_content: Vec<String>,
    pub scratchpad_content: Vec<String>,
    
    // Cursor positions for text areas
    pub title_cursor_pos: (usize, usize),
    pub note_cursor_pos: (usize, usize),
    pub scratchpad_cursor_pos: (usize, usize),
    
    // File metadata
    pub document_path: String,
    pub last_save_timestamp: u64,
    pub has_unsaved_changes: bool,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            current_tab: AppTab::Editor,
            current_note_index: 0,
            current_task_index: 0,
            note_focus: NoteFocus::Title,
            scratchpad_visible: false,
            title_content: Vec::new(),
            note_content: Vec::new(),
            scratchpad_content: Vec::new(),
            title_cursor_pos: (0, 0),
            note_cursor_pos: (0, 0),
            scratchpad_cursor_pos: (0, 0),
            document_path: String::new(),
            last_save_timestamp: 0,
            has_unsaved_changes: false,
        }
    }
}

#[derive(Debug)]
pub struct SessionManager {
    state: SessionState,
    last_change_time: Instant,
    debounce_duration: Duration,
    keystroke_counter: u32,
    save_threshold: u32,
    session_file_path: String,
    needs_save: bool,
}

impl SessionManager {
    pub fn new(session_file_path: String) -> Self {
        Self {
            state: SessionState::default(),
            last_change_time: Instant::now(),
            debounce_duration: Duration::from_millis(500), // 500ms debounce
            keystroke_counter: 0,
            save_threshold: 50, // Save every 50 keystrokes
            session_file_path,
            needs_save: false,
        }
    }

    /// Load session state from file, or create default if file doesn't exist
    pub fn load_session(&mut self) -> io::Result<SessionState> {
        // Always start with a valid default state
        self.state = SessionState::default();
        
        if Path::new(&self.session_file_path).exists() {
            match fs::read_to_string(&self.session_file_path) {
                Ok(content) => {
                    match serde_json::from_str::<SessionState>(&content) {
                        Ok(state) => {
                            // Only use loaded state if it's valid
                            self.state = state.clone();
                        }
                        Err(e) => {
                            // If JSON parsing fails, delete corrupted file and start fresh
                            eprintln!("Warning: Corrupted session file, starting fresh: {}", e);
                            let _ = fs::remove_file(&self.session_file_path);
                        }
                    }
                }
                Err(_) => {
                    // If file read fails, continue with default state
                }
            }
        }
        Ok(self.state.clone())
    }

    /// Update session state from current app state
    pub fn update_state(&mut self, 
        current_tab: &AppTab,
        current_note_index: usize,
        current_task_index: usize,
        note_focus: &NoteFocus,
        scratchpad_visible: bool,
        title: &TextArea<'static>,
        note: &TextArea<'static>,
        scratchpad: &TextArea<'static>,
        document_path: &str,
        has_unsaved_changes: bool,
    ) {
        // Update UI state
        self.state.current_tab = current_tab.clone();
        self.state.current_note_index = current_note_index;
        self.state.current_task_index = current_task_index;
        self.state.note_focus = note_focus.clone();
        self.state.scratchpad_visible = scratchpad_visible;

        // Update draft content
        self.state.title_content = title.lines().iter().map(|s| s.to_string()).collect();
        self.state.note_content = note.lines().iter().map(|s| s.to_string()).collect();
        self.state.scratchpad_content = scratchpad.lines().iter().map(|s| s.to_string()).collect();
        
        // Update cursor positions
        self.state.title_cursor_pos = title.cursor();
        self.state.note_cursor_pos = note.cursor();
        self.state.scratchpad_cursor_pos = scratchpad.cursor();

        // Update metadata
        self.state.document_path = document_path.to_string();
        self.state.has_unsaved_changes = has_unsaved_changes;

        // Track change timing
        self.last_change_time = Instant::now();
        self.keystroke_counter += 1;
        self.needs_save = true;
    }

    /// Check if session should be saved based on debounce logic
    pub fn should_save(&self) -> bool {
        if !self.needs_save {
            return false;
        }

        // Save if enough time has passed since last change
        let time_elapsed = self.last_change_time.elapsed();
        if time_elapsed >= self.debounce_duration {
            return true;
        }

        // Save if keystroke threshold reached (prevent data loss)
        if self.keystroke_counter >= self.save_threshold {
            return true;
        }

        false
    }

    /// Save session state to file (async-safe)
    pub fn save_session(&mut self) -> io::Result<()> {
        if !self.needs_save {
            return Ok(());
        }

        // Update timestamp
        self.state.last_save_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Serialize to JSON
        let json_content = serde_json::to_string_pretty(&self.state)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Atomic write: write to temp file first, then rename
        let temp_path = format!("{}.tmp", self.session_file_path);
        
        {
            let mut file = fs::File::create(&temp_path)?;
            file.write_all(json_content.as_bytes())?;
            file.flush()?;
        }

        // Atomic rename
        fs::rename(&temp_path, &self.session_file_path)?;

        // Reset counters
        self.needs_save = false;
        self.keystroke_counter = 0;

        Ok(())
    }

    /// Force save session (for app exit)
    pub fn force_save(&mut self) -> io::Result<()> {
        self.needs_save = true;
        self.save_session()
    }

    /// Get current session state
    pub fn get_state(&self) -> &SessionState {
        &self.state
    }

    /// Check if there are unsaved drafts that would be lost
    pub fn has_unsaved_drafts(&self) -> bool {
        !self.state.title_content.is_empty() || 
        !self.state.note_content.is_empty() || 
        !self.state.scratchpad_content.is_empty() ||
        self.state.has_unsaved_changes
    }

    /// Create TextArea from saved content
    pub fn restore_textarea(content: &[String]) -> TextArea<'static> {
        if content.is_empty() {
            TextArea::default()
        } else {
            TextArea::from(content.to_vec())
        }
    }
    
    /// Create TextArea from saved content and restore cursor position
    pub fn restore_textarea_with_cursor(content: &[String], cursor_pos: (usize, usize)) -> TextArea<'static> {
        let mut textarea = if content.is_empty() {
            TextArea::default()
        } else {
            TextArea::from(content.to_vec())
        };
        
        // Restore cursor position using CursorMove::Jump
        textarea.move_cursor(tui_textarea::CursorMove::Jump(
            cursor_pos.0 as u16, 
            cursor_pos.1 as u16
        ));
        textarea
    }
}

/// Helper trait to add session support to existing enums
impl Clone for AppTab {
    fn clone(&self) -> Self {
        match self {
            AppTab::Editor => AppTab::Editor,
            AppTab::Viewer => AppTab::Viewer,
            AppTab::Tasks => AppTab::Tasks,
        }
    }
}

impl Clone for NoteFocus {
    fn clone(&self) -> Self {
        match self {
            NoteFocus::Title => NoteFocus::Title,
            NoteFocus::Content => NoteFocus::Content,
        }
    }
}

// Serde implementations for enums
impl Serialize for AppTab {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AppTab::Editor => serializer.serialize_str("Editor"),
            AppTab::Viewer => serializer.serialize_str("Viewer"),
            AppTab::Tasks => serializer.serialize_str("Tasks"),
        }
    }
}

impl<'de> Deserialize<'de> for AppTab {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Editor" => Ok(AppTab::Editor),
            "Viewer" => Ok(AppTab::Viewer),
            "Tasks" => Ok(AppTab::Tasks),
            _ => Ok(AppTab::Editor), // Default fallback
        }
    }
}

impl Serialize for NoteFocus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            NoteFocus::Title => serializer.serialize_str("Title"),
            NoteFocus::Content => serializer.serialize_str("Content"),
        }
    }
}

impl<'de> Deserialize<'de> for NoteFocus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Title" => Ok(NoteFocus::Title),
            "Content" => Ok(NoteFocus::Content),
            _ => Ok(NoteFocus::Title), // Default fallback
        }
    }
}