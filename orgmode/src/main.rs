use orgmode::{Configuration, Note, OrgDocument, Task};

fn main() {
    // Read config from loc
    let loc = "/workspaces/org-mode/tests/config.toml";
    let config = Configuration::with(loc);

    // Get location of org files from config
    let basefolder = config.get_or("basefolder", "/home/sweet/home");
    let refile = std::path::Path::new(&basefolder).join("refile.org");
    println!("{:?}", refile);

    // Parse refile.org file
    let mut od = OrgDocument::from(refile.to_str().unwrap()).unwrap();

    // Create buffer string
    let mut buf = String::new();

    // Read in Task
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read task");

    // Remove trailing line break
    buf = buf.trim().to_string();
    println!("Buffer '{}'", buf);

    // Only create task if not empty
    if !buf.is_empty() {
        let t = Task::with_task(std::mem::take(&mut buf));
        println!("Task: {}", t);
        od.push_task(t);
    }

    // Read in Note w/ first element as title and all following as content
    // until `exit`
    loop {
        let b = buf.clone();
        let lines: Vec<&str> = b.trim().split("\n").collect();
        println!("Lines {:?}", lines);
        if let Some(s) = lines.last() {
            if s.to_string() == "exit".to_string() {
                break;
            }
        }
        std::io::stdin().read_line(&mut buf).expect("note_line");
    }
    let lines: Vec<String> = buf.split("\n").map(|x| x.to_string()).collect();
    let (title, content) = lines.split_first().unwrap();
    let n = Note::with(title.clone(), content.to_vec());
    od.push_note(n);

    // Write back to refile.org
    println!("OrgD: {:?}", od);
    let _ = od.to(refile.to_str().unwrap());
}
