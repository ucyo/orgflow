use std::collections::HashMap;
use orgflow::{OrgDocument, Task};
use std::io::Cursor;

#[test]
fn read_document() {
    let mut source_exp = HashMap::new();
    source_exp.insert("tests/document.md", (2, 3));
    source_exp.insert("tests/document_with_post.md", (2, 1));

    for (s, exp) in source_exp {
        let od = OrgDocument::from(s).unwrap();
        let docs = od.len();
        assert_eq!(docs.0, exp.0, "Err w/ {:?}: [{:?}]", s, od);
        assert_eq!(docs.1, exp.1, "Err w/ {:?}: [{:?}]", s, od);
    }
}

#[test]
fn roundtrip() {
    let files = ["tests/document.md", "tests/document_with_post.md"];
    for file in files {
        let od = OrgDocument::from(file).unwrap();
        let res: Vec<u8> = Vec::new();
        let mut c = Cursor::new(res);
        od.write(&mut c).unwrap();
        let r = String::from_utf8(c.into_inner()).unwrap();
        let exp = std::fs::read_to_string(file).unwrap();
        assert_eq!(r[..r.len() - 1], exp); // TODO: Fix additional extra line at end
    }
}

#[test]
fn test_project_tag_collection() {
    let mut doc = OrgDocument::default();
    
    // Add tasks with project tags
    let task1 = Task::with_today("Fix login bug +webdev @work");
    let task2 = Task::with_today("Update docs +website +docs");
    let task3 = Task::with_today("Regular task without project tags");
    
    doc.push_task(task1);
    doc.push_task(task2);
    doc.push_task(task3);
    
    // Collect tags
    let tag_suggestions = doc.collect_unique_tags();
    
    // Should have 3 unique project tags
    assert_eq!(tag_suggestions.project.len(), 3);
    assert!(tag_suggestions.project.contains(&"+webdev".to_string()));
    assert!(tag_suggestions.project.contains(&"+website".to_string()));
    assert!(tag_suggestions.project.contains(&"+docs".to_string()));
}

#[test]
fn test_no_project_filter_logic() {
    let mut doc = OrgDocument::default();
    
    // Add tasks: some with projects, some without
    let task_with_project = Task::with_today("Fix bug +webdev @work");
    let task_without_project = Task::with_today("Regular task without project");
    let task_with_context_only = Task::with_today("Task with context only @work");
    let task_with_multiple_projects = Task::with_today("Update docs +website +docs");
    
    doc.push_task(task_with_project);
    doc.push_task(task_without_project);
    doc.push_task(task_with_context_only);
    doc.push_task(task_with_multiple_projects);
    
    // Test that we can identify tasks without project tags
    let tasks_without_projects: Vec<&Task> = doc.tasks.iter()
        .filter(|task| {
            if let Some(tags) = task.tags() {
                tags.project_tags().is_empty()
            } else {
                true // Tasks with no tags at all count as having no project
            }
        })
        .collect();
    
    // Should have 2 tasks without project tags
    assert_eq!(tasks_without_projects.len(), 2);
    
    // Verify the correct tasks are identified
    assert!(tasks_without_projects.iter().any(|task| task.description() == "Regular task without project"));
    assert!(tasks_without_projects.iter().any(|task| task.description() == "Task with context only"));
}
