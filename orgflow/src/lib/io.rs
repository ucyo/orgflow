use std::fs::File;
use std::io::Result as IoResult;
use std::io::{self, BufRead, Seek, Write};
use std::str::FromStr;
use std::vec;

use crate::Note;
use crate::Task;

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
            let t: Vec<String> = note.try_into().unwrap();
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
