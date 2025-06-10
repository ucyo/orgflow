# Orgflow

A modern Rust ecosystem for managing notes and tasks with a smooth, efficient workflow.

Orgflow consists of two main components:
- **`orgflow`** - Core library for document management with support for tasks and notes
- **`orgflow-tui`** - Terminal user interface for intuitive note and task management

## 🚀 Quick Start

### Install the TUI Application

```bash
# Install from crates.io
cargo install orgflow-tui

# Run the application
orgflow-tui
```

### Use the Library

Add to your `Cargo.toml`:

```toml
[dependencies]
orgflow = "0.1.0"
```

## 📋 Features

### Orgflow Library (`orgflow`)

- **Document Management**: Read, write, and manipulate structured documents
- **Task Handling**: Create, manage, and track tasks with metadata
- **Note Support**: Rich note creation with titles, content, and tags
- **Flexible Storage**: Configurable file locations and formats
- **Rust API**: Type-safe, memory-efficient document operations

### Orgflow TUI (`orgflow-tui`)

- **Three-Tab Interface**: Editor, Viewer, and Tasks views
- **Session Management**: Automatic state persistence and recovery
- **Real-time Editing**: Create and edit notes with immediate saving
- **Task Management**: Visual task list with status tracking
- **Note Browser**: Navigate through saved notes with metadata display
- **Keyboard-driven**: Efficient navigation without mouse dependency
- **Cross-platform**: Works on Linux, macOS, and Windows

## 🎯 Use Cases

- **Personal Knowledge Management**: Organize notes, ideas, and research
- **Task Tracking**: Manage todos, projects, and deadlines
- **Documentation**: Create and maintain structured documentation
- **Note-taking**: Capture thoughts with rich formatting and metadata
- **Productivity**: Streamline workflow with keyboard-driven interface

## 📦 Components

### Core Library: `orgflow`

The foundational library that provides:

```rust
use orgflow::{OrgDocument, Task, Note, Configuration};

// Load a document
let doc = OrgDocument::from("path/to/file.org")?;

// Create a task
let task = Task::with_today("Complete project documentation");

// Create a note
let note = Note::with("Meeting Notes".to_string(), vec![
    "Discussed project timeline".to_string(),
    "Next steps identified".to_string(),
]);

// Add to document and save
doc.push_task(task);
doc.push_note(note);
doc.to("path/to/file.org")?;
```

### Terminal Interface: `orgflow`

A beautiful, responsive terminal interface featuring:

#### Editor Tab
- Create notes with titles and rich content
- Quick task entry with `Ctrl+T`
- Auto-save functionality with `Ctrl+S`
- Smart field navigation
- Draft content automatically preserved between sessions

#### Viewer Tab
- Browse all saved notes
- Split-panel layout (content + metadata)
- Arrow key navigation
- Rich metadata display

#### Tasks Tab
- Visual task list with status indicators
- Detailed task information panel
- Highlighted selection with underlines
- Priority and date tracking

## 📖 Documentation

### Installation

#### From Crates.io

```bash
# Install the TUI application
cargo install orgflow-tui

# Use the library in your project
cargo add orgflow
```

#### From Source

```bash
git clone https://github.com/ucyo/orgflow
cd orgflow

# Build everything
cargo build --release

# Build just the library
cargo build -p orgflow --release

# Build just the TUI
cargo build -p orgflow-tui --release
```

### Configuration

Set your preferred storage location:

```bash
export ORGFLOW_BASEFOLDER=/path/to/your/notes
```

Default location: `/home/sweet/home`

### Session Management

Orgflow TUI automatically manages your session state:

- **Automatic Saving**: Session state is saved every 500ms after changes or every 50 keystrokes
- **Draft Recovery**: Unsaved notes, tasks, and scratchpad content are preserved between sessions
- **UI State Persistence**: Current tab, focus, and selection positions are restored on startup
- **Session File**: State is stored in `session.json` in your base folder
- **No Data Loss**: Even if the application crashes, your work is automatically recovered

**Session includes:**
- Current tab and focus position
- Unsaved draft content (title, note content, scratchpad)
- Navigation state (selected note/task indices)
- UI preferences (scratchpad visibility)

The session file is automatically created and managed - no manual intervention required.

### File Format

Orgflow uses a structured text format:

```org
## Tasks
[ ] Implement new feature
[x] 2024-01-15 Write documentation
(A) 2024-01-10 High priority task @work +project

## Notes

### Meeting Notes
> cre:2024-01-15 mod:2024-01-15 guid:abc123... @meeting +work
Discussed quarterly objectives and timeline.

Next steps:
- Review current progress
- Set new milestones
- Schedule follow-up meeting

### Project Ideas
> cre:2024-01-10 mod:2024-01-12 guid:def456... @ideas +innovation
Ideas for improving the user experience:
1. Better navigation
2. Faster search
3. Mobile support
```

## 🎮 Usage Examples

### TUI Application

```bash
# Start the application
orgflow

# Keyboard shortcuts:
# 1 - Editor tab    2 - Viewer tab    3 - Tasks tab
# Ctrl+T - Quick task entry    Ctrl+S - Save note
# Esc - Exit (session auto-saved)    Tab - Navigate fields
# Session state automatically preserved on every keystroke
```

### Library Usage

```rust
use orgflow::{Configuration, OrgDocument, Task, Note};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get base folder from environment or use default
    let base_folder = Configuration::basefolder();
    let file_path = format!("{}/refile.org", base_folder);

    // Load existing document or create new
    let mut doc = OrgDocument::from(&file_path)
        .unwrap_or_else(|_| OrgDocument::default());

    // Add a new task
    let task = Task::with_today("Review pull requests");
    doc.push_task(task);

    // Add a new note
    let note = Note::with(
        "Daily Standup".to_string(),
        vec![
            "Team discussed current sprint progress".to_string(),
            "Identified blockers and solutions".to_string(),
            "Planned next steps for the week".to_string(),
        ]
    );
    doc.push_note(note);

    // Save the document
    doc.to(&file_path)?;

    println!("Document saved with {} tasks and {} notes",
             doc.len().0, doc.len().1);

    Ok(())
}
```

## 🛠️ Development

### Prerequisites

- Rust 1.70+ (edition 2024)
- Git

### Setup

```bash
# Clone the repository
git clone https://github.com/ucyo/orgflow
cd orgflow

# Build the workspace
cargo build

# Run tests
cargo test

# Run the TUI in development
cargo run -p orgflow-tui

# Run the CLI tool
cargo run -p orgflow
```

### Project Structure

```
orgflow/
├── orgflow/           # Core library
│   ├── src/
│   │   ├── lib/       # Library modules
│   │   └── main.rs    # CLI binary
│   └── tests/         # Integration tests
├── orgflow-tui/       # Terminal interface
│   └── src/
│       ├── main.rs    # TUI application
│       └── session.rs # Session management
├── Cargo.toml         # Workspace configuration
└── README.md          # This file
```

### Testing

```bash
# Run all tests
cargo test

# Test specific package
cargo test -p orgflow
cargo test -p orgflow-tui

# Run with output
cargo test -- --nocapture
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Check for issues
cargo check
```

## 🎨 Interface Preview

### Editor Tab
```
┌─────────────────────────────────────────────────────────────┐
│              Orgflow - Editor (1) | Viewer (2) | Tasks (3)  │
├─────────────────────────────────────────────────────────────┤
│ Title                                                       │
│ Weekly Planning Session                                     │
├─────────────────────────────────────────────────────────────┤
│ Content                                                     │
│ ## Agenda                                                   │
│ 1. Review last week's accomplishments                      │
│ 2. Set priorities for upcoming week                        │
│ 3. Identify potential blockers                             │
│                                                             │
│ ## Action Items                                             │
│ - Schedule team review meeting                              │
│ - Update project documentation                              │
│                                                             │
│ Quit <ESC> Switch <SHIFT>+<TAB> Save Note <CTRL>+<S>      │
└─────────────────────────────────────────────────────────────┘
```

### Tasks Tab
```
┌─────────────────────────────────────────────────────────────┐
│              Orgflow - Editor (1) | Viewer (2) | Tasks (3)  │
├──────────────────────────────────────┬──────────────────────┤
│ Tasks (5 total)                      │ Task Details         │
│ ► [ ] Review pull request #123       │ Status: Pending      │
│   [x] Update documentation           │ Priority: High       │
│   [ ] Fix login bug                  │ Created: 2024-01-15  │
│   [ ] Plan next sprint               │ Completed: N/A       │
│   [x] Team standup meeting           │ Tags: @dev +urgent   │
│                                      │                      │
│                                      │ Description:         │
│                                      │ Review pull request  │
│                                      │ #123 for the new     │
│                                      │ authentication...    │
│ Navigate <↑↓> Quit <ESC>            │                      │
└──────────────────────────────────────┴──────────────────────┘
```

**Session Management**: All UI state and draft content is automatically preserved in `session.json`. Exit anytime with `ESC` and resume exactly where you left off!

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes**: Follow the existing code style
4. **Add tests**: Ensure your changes are well-tested
5. **Run tests**: `cargo test`
6. **Submit a PR**: Create a pull request with a clear description

### Contribution Guidelines

- Follow Rust conventions and idioms
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic
- Write clear commit messages

### Areas for Contribution

- **Features**: New functionality for the library or TUI
- **Performance**: Optimization opportunities
- **Documentation**: Improve examples and guides
- **Testing**: Increase test coverage
- **UI/UX**: Enhance the terminal interface
- **Bug fixes**: Address reported issues

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) - Excellent TUI framework
- [org-mode](https://orgmode.org/) - Inspiration for the document format
- The Rust community for amazing tools and libraries

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/ucyo/orgflow/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ucyo/orgflow/discussions)
- **Documentation**: [docs.rs/orgflow](https://docs.rs/orgflow)

---

**Start organizing your workflow with Orgflow today!** 🚀

---

*Made with ❤️ in Rust*