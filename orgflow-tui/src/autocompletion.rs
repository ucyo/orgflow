use orgflow::TagSuggestions;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

/// Autocompletion popup widget for tag suggestions
#[derive(Debug, Clone)]
pub struct AutocompletionWidget {
    suggestions: Vec<String>,
    selected_index: usize,
    visible: bool,
    current_input: String,
    current_tag_type: TagType,
}

#[derive(Debug, Clone)]
enum TagType {
    Context, // @context
    Project, // +project
    Person,  // p:person
    Custom,  // key:value
    OneOff,  // !oneoff
    Mixed,   // Multiple types or unknown
}

impl AutocompletionWidget {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            selected_index: 0,
            visible: false,
            current_input: String::new(),
            current_tag_type: TagType::Mixed,
        }
    }

    /// Update suggestions based on current input and available tags
    pub fn update_suggestions(&mut self, input: &str, tag_suggestions: &TagSuggestions) {
        self.current_input = input.to_string();

        // Find the last word that looks like a tag (starts with @, +, p:, !, or contains :)
        let words: Vec<&str> = input.split_whitespace().collect();
        let last_word = words.last().unwrap_or(&"");

        if self.is_tag_prefix(last_word) {
            self.suggestions = tag_suggestions.suggestions_for_prefix(last_word);
            self.current_tag_type = self.determine_tag_type(last_word);
            self.visible = !self.suggestions.is_empty();
            self.selected_index = 0;
        } else {
            self.visible = false;
            self.suggestions.clear();
            self.current_tag_type = TagType::Mixed;
        }
    }

    /// Check if a word looks like the start of a tag
    fn is_tag_prefix(&self, word: &str) -> bool {
        if word.is_empty() {
            return false;
        }

        // Check for various tag prefixes
        word.starts_with('@')       // @context
            || word.starts_with('+') // +project
            || word.starts_with('!')  // !oneoff
            || (word.starts_with('p') && (word.contains(':') || word.len() >= 2)) // p:person
            || (word.contains(':') && word.len() > 1) // custom:value
    }

    /// Determine the tag type based on the prefix
    fn determine_tag_type(&self, word: &str) -> TagType {
        if word.starts_with('@') {
            TagType::Context
        } else if word.starts_with('+') {
            TagType::Project
        } else if word.starts_with('!') {
            TagType::OneOff
        } else if word.starts_with("p:") {
            // Person tags must start with exactly "p:"
            TagType::Person
        } else if word.contains(':') && word.len() > 1 {
            // Any other tag containing ':' is a custom tag
            TagType::Custom
        } else {
            TagType::Mixed
        }
    }

    /// Get the display name for the current tag type
    fn get_tag_type_display(&self) -> &'static str {
        match self.current_tag_type {
            TagType::Context => "Context",
            TagType::Project => "Project",
            TagType::Person => "Person",
            TagType::Custom => "Custom",
            TagType::OneOff => "OneOff",
            TagType::Mixed => "Tags",
        }
    }

    /// Move selection up in the suggestions list
    pub fn select_previous(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.suggestions.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Move selection down in the suggestions list
    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    /// Get the currently selected suggestion
    pub fn get_selected(&self) -> Option<&String> {
        self.suggestions.get(self.selected_index)
    }

    /// Apply the selected suggestion to the input
    /// Returns (new_text, cursor_position) where cursor_position is the character index
    pub fn apply_selected(&self, input: &str) -> Option<(String, usize)> {
        if let Some(selected) = self.get_selected() {
            let mut words: Vec<&str> = input.split_whitespace().collect();
            if let Some(last_word) = words.last_mut() {
                if self.is_tag_prefix(last_word) {
                    // Replace the last word with the selected suggestion
                    words.pop();
                    words.push(selected);
                    let new_text = words.join(" ") + " ";
                    let cursor_pos = new_text.len(); // Position cursor at the end
                    return Some((new_text, cursor_pos));
                }
            }
        }
        None
    }

    /// Check if the autocompletion popup is visible
    pub fn is_visible(&self) -> bool {
        self.visible && !self.suggestions.is_empty()
    }

    /// Hide the autocompletion popup
    pub fn hide(&mut self) {
        self.visible = false;
        self.suggestions.clear();
        self.selected_index = 0;
        self.current_tag_type = TagType::Mixed;
    }

    /// Render the autocompletion popup at a specific position
    pub fn render(&self, area: Rect, buf: &mut Buffer, cursor_pos: (u16, u16)) {
        if !self.is_visible() {
            return;
        }

        // Calculate popup position (below cursor, but within screen bounds)
        let popup_height = (self.suggestions.len() as u16 + 2).min(8); // Max 6 suggestions + borders
        let popup_width = self
            .suggestions
            .iter()
            .map(|s| s.len() as u16)
            .max()
            .unwrap_or(20)
            .max(20)
            .min(40); // Min 20, max 40 chars wide

        let popup_x = cursor_pos.0.min(area.width.saturating_sub(popup_width));
        let popup_y = (cursor_pos.1 + 1).min(area.height.saturating_sub(popup_height));

        let popup_area = Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Ensure popup is within the provided area
        if popup_area.x + popup_area.width > area.x + area.width
            || popup_area.y + popup_area.height > area.y + area.height
        {
            return; // Don't render if it would go outside bounds
        }

        // Create list items
        let items: Vec<ListItem> = self
            .suggestions
            .iter()
            .enumerate()
            .map(|(i, suggestion)| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default()
                };
                ListItem::new(suggestion.as_str()).style(style)
            })
            .collect();

        // Create the list widget
        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.get_tag_type_display())
                .style(Style::default().bg(Color::DarkGray)),
        );

        // Render the popup
        ratatui::widgets::Clear.render(popup_area, buf);
        ratatui::prelude::Widget::render(list, popup_area, buf);
    }
}

impl Default for AutocompletionWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orgflow::TagSuggestions;

    fn create_test_suggestions() -> TagSuggestions {
        TagSuggestions {
            context: vec![
                "@work".to_string(),
                "@home".to_string(),
                "@phone".to_string(),
            ],
            project: vec!["+project1".to_string(), "+urgent".to_string()],
            person: vec!["p:john".to_string(), "p:alice".to_string()],
            custom: vec!["priority:high".to_string(), "status:done".to_string()],
            oneoff: vec!["!important".to_string(), "!reminder".to_string()],
        }
    }

    #[test]
    fn test_context_tag_suggestions() {
        let mut widget = AutocompletionWidget::new();
        let suggestions = create_test_suggestions();

        widget.update_suggestions("This is a task @w", &suggestions);

        assert!(widget.is_visible());
        assert_eq!(widget.suggestions, vec!["@work"]);
    }

    #[test]
    fn test_project_tag_suggestions() {
        let mut widget = AutocompletionWidget::new();
        let suggestions = create_test_suggestions();

        widget.update_suggestions("Task +p", &suggestions);

        assert!(widget.is_visible());
        assert_eq!(widget.suggestions, vec!["+project1"]);
    }

    #[test]
    fn test_no_suggestions_for_regular_text() {
        let mut widget = AutocompletionWidget::new();
        let suggestions = create_test_suggestions();

        widget.update_suggestions("This is regular text", &suggestions);

        assert!(!widget.is_visible());
    }

    #[test]
    fn test_apply_selected() {
        let mut widget = AutocompletionWidget::new();
        let suggestions = create_test_suggestions();

        widget.update_suggestions("Task @w", &suggestions);
        let result = widget.apply_selected("Task @w");

        assert_eq!(result, Some(("Task @work ".to_string(), 11))); // 11 is the length "Task @work "
    }

    #[test]
    fn test_apply_selected_cursor_position() {
        let mut widget = AutocompletionWidget::new();
        let suggestions = create_test_suggestions();

        // Test with context tag
        widget.update_suggestions("Do something @h", &suggestions);
        let result = widget.apply_selected("Do something @h");
        let expected_text = "Do something @home ";
        assert_eq!(
            result,
            Some((expected_text.to_string(), expected_text.len()))
        );

        // Test with project tag
        widget.update_suggestions("Fix bug +p", &suggestions);
        let result = widget.apply_selected("Fix bug +p");
        let expected_text = "Fix bug +project1 ";
        assert_eq!(
            result,
            Some((expected_text.to_string(), expected_text.len()))
        );

        // Test with person tag
        widget.update_suggestions("Ask p:a", &suggestions);
        let result = widget.apply_selected("Ask p:a");
        let expected_text = "Ask p:alice ";
        assert_eq!(
            result,
            Some((expected_text.to_string(), expected_text.len()))
        );
    }

    #[test]
    fn test_navigation() {
        let mut widget = AutocompletionWidget::new();
        let suggestions = create_test_suggestions();

        widget.update_suggestions("@", &suggestions);

        assert_eq!(widget.selected_index, 0);

        widget.select_next();
        assert_eq!(widget.selected_index, 1);

        widget.select_previous();
        assert_eq!(widget.selected_index, 0);
    }

    #[test]
    fn test_tag_type_detection() {
        let mut widget = AutocompletionWidget::new();
        let suggestions = create_test_suggestions();

        // Test context tag type
        widget.update_suggestions("Task @w", &suggestions);
        assert_eq!(widget.get_tag_type_display(), "Context");

        // Test project tag type
        widget.update_suggestions("Fix +p", &suggestions);
        assert_eq!(widget.get_tag_type_display(), "Project");

        // Test person tag type
        widget.update_suggestions("Ask p:a", &suggestions);
        assert_eq!(widget.get_tag_type_display(), "Person");

        // Test oneoff tag type
        widget.update_suggestions("Note !i", &suggestions);
        assert_eq!(widget.get_tag_type_display(), "OneOff");

        // Test custom tag type (would need custom suggestions to actually show)
        widget.determine_tag_type("priority:"); // Direct test of method
        // Since update_suggestions filters, we test the method directly
        assert!(matches!(
            widget.determine_tag_type("priority:high"),
            TagType::Custom
        ));
        assert!(matches!(
            widget.determine_tag_type("@work"),
            TagType::Context
        ));
        assert!(matches!(
            widget.determine_tag_type("+project"),
            TagType::Project
        ));
        assert!(matches!(
            widget.determine_tag_type("p:john"),
            TagType::Person
        ));
        assert!(matches!(
            widget.determine_tag_type("!urgent"),
            TagType::OneOff
        ));
    }
}
