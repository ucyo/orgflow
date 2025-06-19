# Changelog

All notable changes to Orgflow will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Session Management**: Complete session persistence for the TUI application
  - Automatic state saving on every keystroke with intelligent debouncing
  - Session recovery on application restart
  - Preservation of UI state (current tab, focus, selection indices)
  - Draft content recovery (unsaved notes, tasks, and scratchpad content)
  - Smart IO optimization (saves after 500ms inactivity or 50 keystrokes)
  - Session data stored in `session.json` in the base folder
  - Atomic file operations for reliable session persistence
- **Tag Autocompletion System**: Intelligent tag suggestions with real-time completion
  - Smart prefix detection for all tag types (@context, +project, p:person, !oneoff, key:value)
  - Type-specific popup titles (Context, Project, Person, Custom, OneOff)
  - Keyboard navigation (↑/↓ to navigate, Tab to select, Esc to close)
  - Real-time suggestion updates based on existing document tags
  - Available in both task scratchpad (Ctrl+T) and note title fields
- **Enhanced Note Creation**: Automatic tag extraction and metadata integration
  - Tags typed in note titles are automatically extracted and added to note metadata
  - Clean separation: tags removed from display text, preserved in structured metadata
  - Support for mixed tag types in single title/content
  - Automatic tag database updates when new notes are saved

### Enhanced
- **TUI Application**: Improved user experience with seamless workflow continuity
  - No more lost work when accidentally closing the application
  - Resume exactly where you left off, including cursor position and focus
  - Automatic detection of unsaved changes and draft content
  - Background session saving without blocking the UI
- **Note Loading**: Robust handling of notes without content
  - Fixed startup crashes when loading existing documents with contentless notes
  - Relaxed validation to allow notes with only title and metadata
  - Better error handling for malformed or incomplete note structures
- **Navigation**: Streamlined tab switching with cycling interface
  - Replaced individual 1,2,3 keys with Ctrl+R cycling navigation
  - Single key combination cycles through Editor → Viewer → Tasks → Editor
  - Frees up number keys for potential future features

### Technical
- Added `serde` and `serde_json` dependencies for session serialization
- New `session.rs` module with `SessionManager` and `SessionState` types
- Integrated session management into main application event loop
- Memory-efficient state tracking with minimal overhead
- New `autocompletion.rs` module with `AutocompletionWidget` and `TagType` system
- Extended `TagCollection` with tag extraction methods and `from_tags` constructor
- Added `Note::with_tags()` method for creating notes with embedded tag metadata
- Enhanced `OrgDocument` with `collect_unique_tags()` for suggestion generation

### Fixed
- **Document Loading**: Fixed panic when loading notes without content from refile.org files
- **Note Parsing**: Relaxed validation requirements to allow notes with only title and metadata
- **Keyboard Input**: Fixed Ctrl+S save functionality to work from any tab (not just Editor)
- **Scratchpad Input**: Fixed tab switching (1-3) interfering with text input in scratchpad
- **Base Path Configuration**: Improved default path logic using HOME environment variable
- **File Operations**: Replaced unsafe try_into().unwrap() with safe into() conversions
- **Content Validation**: Enhanced logic to distinguish meaningful content from whitespace
- **Session Loading**: Added graceful error handling for corrupted session files

## [0.1.1] - 2024-01-XX

### Added
- Initial release of Orgflow workspace
- **orgflow**: Core library for document management with tasks and notes
- **orgflow-tui**: Terminal user interface with three-tab layout
- Support for org-mode inspired document format
- Task management with priorities, dates, and completion tracking
- Note creation with metadata (creation/modification dates, GUIDs, tags)
- Configurable storage location via `ORGFLOW_BASEFOLDER` environment variable

### Features
- **Editor Tab**: Create and edit notes with title and content fields
- **Viewer Tab**: Browse and view existing notes with metadata display
- **Tasks Tab**: Manage and view tasks with status indicators
- **Quick Task Entry**: Ctrl+T popup for rapid task creation
- **Keyboard Navigation**: Fully keyboard-driven interface
- **Cross-platform**: Works on Linux, macOS, and Windows

### Dependencies
- `ratatui` for terminal UI framework
- `tui-textarea` for text input handling
- `crossterm` for cross-platform terminal support
- `chrono` for date/time handling
- `uuid` for unique identifier generation