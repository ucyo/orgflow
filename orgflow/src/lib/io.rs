use std::fs::File;
use std::io::Result as IoResult;
use std::io::{self, BufRead, Seek, Write};
use std::str::FromStr;
use std::vec;

use std::collections::HashSet;

use crate::{Note, Task};

#[derive(PartialEq, Debug, Default)]
pub struct OrgDocument {
    preample: Vec<String>,
    pub tasks: Vec<Task>,
    between: Vec<String>,
    pub notes: Vec<Note>,
    post: Vec<String>,
}

impl OrgDocument {
    pub fn push_task(&mut self, task: Task) {
        self.tasks.push(task);
    }
    pub fn push_note(&mut self, note: Note) {
        self.notes.push(note);
    }
    pub fn write<W: Write + Seek>(&self, mut buf: W) -> Result<(), io::Error> {
        for line in self.preample.iter() {
            writeln!(buf, "{}", line)?;
        }
        writeln!(buf, "")?;
        writeln!(buf, "## Tasks")?;
        for line in self.tasks.iter() {
            writeln!(buf, "{}", line)?;
        }
        writeln!(buf, "")?;
        if !self.between.is_empty() {
            for line in self.between.iter() {
                writeln!(buf, "{}", line)?;
            }
            writeln!(buf, "")?;
        }
        writeln!(buf, "## Notes")?;
        writeln!(buf, "")?;
        for note in self.notes.iter() {
            let t: Vec<String> = note.into();
            for nline in t.iter() {
                writeln!(buf, "{}", nline)?;
            }
            writeln!(buf, "")?;
        }
        if !self.post.is_empty() {
            for line in self.post.iter() {
                writeln!(buf, "{}", line)?;
            }
        }
        Ok(buf.flush()?)
    }
    pub fn to(&self, path: &str) -> Result<(), io::Error> {
        let file = File::options().write(true).open(path)?;
        let buf = io::BufWriter::new(file);
        self.write(buf)
    }
    pub fn from(path: &str) -> IoResult<Self> {
        let mut parser = OrgDocumentParser::default();
        let mut doc = OrgDocument::default();
        let file = File::open(path)?;
        let lines = io::BufReader::new(file).lines();
        for line in lines.map_while(Result::ok) {
            if !line.is_empty() {
                parser.parse(&line, &mut doc)?;
            }
        }
        parser.finish(&mut doc)?;
        Ok(doc)
    }
    pub fn len(&self) -> (usize, usize) {
        (self.tasks.len(), self.notes.len())
    }

    /// Collect all unique tags from tasks and notes for autocompletion
    pub fn collect_unique_tags(&self) -> TagSuggestions {
        let mut context_tags = HashSet::new();
        let mut project_tags = HashSet::new();
        let mut person_tags = HashSet::new();
        let mut custom_tags = HashSet::new();
        let mut oneoff_tags = HashSet::new();

        // Collect tags from tasks
        for task in &self.tasks {
            if let Some(tag_collection) = task.tags() {
                context_tags.extend(tag_collection.context_tags());
                project_tags.extend(tag_collection.project_tags());
                person_tags.extend(tag_collection.person_tags());
                custom_tags.extend(tag_collection.custom_tags());
                oneoff_tags.extend(tag_collection.oneoff_tags());
            }
        }

        // Collect tags from notes
        for note in &self.notes {
            let tag_collection = note.tags();
            context_tags.extend(tag_collection.context_tags());
            project_tags.extend(tag_collection.project_tags());
            person_tags.extend(tag_collection.person_tags());
            custom_tags.extend(tag_collection.custom_tags());
            oneoff_tags.extend(tag_collection.oneoff_tags());
        }

        // Convert HashSets to sorted Vecs
        let mut context: Vec<String> = context_tags.into_iter().collect();
        let mut project: Vec<String> = project_tags.into_iter().collect();
        let mut person: Vec<String> = person_tags.into_iter().collect();
        let mut custom: Vec<String> = custom_tags.into_iter().collect();
        let mut oneoff: Vec<String> = oneoff_tags.into_iter().collect();

        context.sort();
        project.sort();
        person.sort();
        custom.sort();
        oneoff.sort();

        TagSuggestions {
            context,
            project,
            person,
            custom,
            oneoff,
        }
    }
}

/// Collection of tag suggestions for autocompletion
#[derive(Debug, Clone)]
pub struct TagSuggestions {
    pub context: Vec<String>,   // @context
    pub project: Vec<String>,   // +project
    pub person: Vec<String>,    // p:person
    pub custom: Vec<String>,    // key:value
    pub oneoff: Vec<String>,    // !oneoff
}

impl TagSuggestions {
    /// Get all tags as a flat list for general autocompletion
    pub fn all_tags(&self) -> Vec<String> {
        let mut all = Vec::new();
        all.extend(self.context.clone());
        all.extend(self.project.clone());
        all.extend(self.person.clone());
        all.extend(self.custom.clone());
        all.extend(self.oneoff.clone());
        all.sort();
        all
    }

    /// Get suggestions that match a given prefix
    pub fn matching_prefix(&self, prefix: &str) -> Vec<String> {
        self.all_tags()
            .into_iter()
            .filter(|tag| tag.to_lowercase().starts_with(&prefix.to_lowercase()))
            .collect()
    }

    /// Get suggestions for a specific tag type based on prefix
    pub fn suggestions_for_prefix(&self, prefix: &str) -> Vec<String> {
        if prefix.starts_with('@') {
            // Context tags
            self.context
                .iter()
                .filter(|tag| tag.to_lowercase().starts_with(&prefix.to_lowercase()))
                .cloned()
                .collect()
        } else if prefix.starts_with('+') {
            // Project tags
            self.project
                .iter()
                .filter(|tag| tag.to_lowercase().starts_with(&prefix.to_lowercase()))
                .cloned()
                .collect()
        } else if prefix.starts_with('p') && prefix.contains(':') {
            // Person tags
            self.person
                .iter()
                .filter(|tag| tag.to_lowercase().starts_with(&prefix.to_lowercase()))
                .cloned()
                .collect()
        } else if prefix.starts_with('!') {
            // One-off tags
            self.oneoff
                .iter()
                .filter(|tag| tag.to_lowercase().starts_with(&prefix.to_lowercase()))
                .cloned()
                .collect()
        } else if prefix.contains(':') {
            // Custom tags
            self.custom
                .iter()
                .filter(|tag| tag.to_lowercase().starts_with(&prefix.to_lowercase()))
                .cloned()
                .collect()
        } else {
            // Fallback to all tags
            self.matching_prefix(prefix)
        }
    }
}

enum OrgDocumentParser {
    BeforeTasks,
    InTasks,
    BetweenTasksAndNotes,
    InNotes(Vec<String>),
    AfterNotes,
}

impl Default for OrgDocumentParser {
    fn default() -> Self {
        Self::BeforeTasks
    }
}

impl OrgDocumentParser {
    fn parse(&mut self, line: &str, doc: &mut OrgDocument) -> IoResult<()> {
        match (&self, line) {
            (OrgDocumentParser::BeforeTasks, "## Tasks") => *self = OrgDocumentParser::InTasks,
            (OrgDocumentParser::InTasks, "## Notes") => {
                *self = OrgDocumentParser::InNotes(Vec::new())
            }
            (OrgDocumentParser::InTasks, l) if l.starts_with("## ") => {
                doc.between.push(line.to_string().clone());
                *self = OrgDocumentParser::BetweenTasksAndNotes;
            }
            (OrgDocumentParser::BetweenTasksAndNotes, "## Notes") => {
                *self = OrgDocumentParser::InNotes(Vec::new())
            }
            (OrgDocumentParser::InNotes(note_vec), l)
                if (l.starts_with("## ") | l.starts_with("### ")) =>
            {
                if !note_vec.is_empty() {
                    doc.notes.push(Note::from(note_vec.clone()));
                }
                if l.starts_with("## ") {
                    doc.post.push(l.to_string().clone());
                    *self = OrgDocumentParser::AfterNotes
                } else {
                    *self = OrgDocumentParser::InNotes(vec![line.to_string()])
                }
            }
            (OrgDocumentParser::BeforeTasks, _) => doc.preample.push(line.to_string().clone()),
            (OrgDocumentParser::InTasks, _) => doc.tasks.push(Task::from_str(line).unwrap()),
            (OrgDocumentParser::BetweenTasksAndNotes, _) => doc.between.push(line.to_string()),
            (OrgDocumentParser::InNotes(notes_vec), _) => {
                let mut t = notes_vec.clone();
                t.push(line.to_string());
                *self = OrgDocumentParser::InNotes(t)
            }
            (OrgDocumentParser::AfterNotes, _) => {
                doc.post.push(line.to_string());
            }
        }
        Ok(())
    }
    fn finish(&mut self, doc: &mut OrgDocument) -> IoResult<()> {
        match self {
            OrgDocumentParser::InNotes(vec) => {
                if !vec.is_empty() {
                    doc.notes.push(Note::from(vec.clone()));
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}
