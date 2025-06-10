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

### Enhanced
- **TUI Application**: Improved user experience with seamless workflow continuity
  - No more lost work when accidentally closing the application
  - Resume exactly where you left off, including cursor position and focus
  - Automatic detection of unsaved changes and draft content
  - Background session saving without blocking the UI

### Technical
- Added `serde` and `serde_json` dependencies for session serialization
- New `session.rs` module with `SessionManager` and `SessionState` types
- Integrated session management into main application event loop
- Memory-efficient state tracking with minimal overhead

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