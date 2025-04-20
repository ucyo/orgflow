use std::fmt::Display;
use std::str::FromStr;

use super::tags::guid::Guid;

use super::dates::Date;
use super::tags::TagCollection;

#[derive(PartialEq, Debug)]
pub struct Note {
    lvl: usize,
    title: String,
    creation_date: Date,
    modification_date: Date,
    guid: Guid,
    tags: TagCollection,
    content: Vec<String>,
}

impl Note {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with(title: String, content: Vec<String>) -> Self {
        Self {
            title,
            content,
            ..Default::default()
        }
    }
    pub fn with_content(content: Vec<String>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}

impl Default for Note {
    fn default() -> Self {
        Self {
            lvl: 3,
            title: String::new(),
            creation_date: Date::now(),
            modification_date: Date::now(),
            tags: TagCollection::new(),
            guid: Guid::new(),
            content: Vec::new(),
        }
    }
}

impl Into<Vec<String>> for &Note {
    fn into(self) -> Vec<String> {
        let lvl = '#'.to_string().repeat(self.lvl);
        let title = format!("{} {}", lvl, self.title.trim());
        let metadata = format!(
            "> cre:{} mod:{} guid:{} {}",
            self.creation_date.to_string(),
            self.modification_date.to_string(),
            self.guid.to_string(),
            self.tags.to_string()
        )
        .trim()
        .to_string();
        let mut content = self.content.clone();
        let mut result = vec![title, metadata];
        result.append(&mut content);
        result
    }
}

impl Note {
    fn from_vec(value: Vec<String>) -> Result<Self, String> {
        if value.len() < 3 {
            return Err(format!(
                "There should be at least a title, some metadata and content [{:?}]",
                value
            ));
        }
        let (title, body) = value.split_first().unwrap();

        // First element is title w/ level information
        let (lvl_str, title) = title
            .split_once(" ")
            .ok_or(format!("Title of only a word is not allowed [{:?}]", value))?;

        let check_lvl = '#'.to_string().repeat(lvl_str.trim().len());
        if check_lvl != lvl_str {
            return Err("Title must start with '#' defining the levels in document".to_string());
        };
        let lvl = lvl_str.len();
        let title = title.trim().to_string();

        let (metadata, remainder) = body.split_first().unwrap();

        // Second element is the metadata
        assert!(
            metadata.starts_with("> "),
            "Wrong metadata start for [{:?}]",
            value
        );
        let metadata = metadata.to_string().replace("> ", "");

        // First metadata is creation date
        let (creation_date_str, metadata) =
            metadata.split_once(" ").ok_or("Creation date not found")?;
        let creation_date = Date::from_str(&creation_date_str.replace("cre:", ""))?;

        // Second metadata is modification date
        let (modification_date_str, metadata) = metadata
            .trim()
            .split_once(" ")
            .ok_or("Modification date not found")?;
        let modification_date = Date::from_str(&modification_date_str.replace("mod:", ""))?;

        // Third metadata is note id
        let (guid_str, tag_str) = metadata.split_once(" ").unwrap_or((&metadata.trim(), ""));
        let guid = Guid::from_str(&guid_str.replace("guid:", ""))?;

        let tags = if tag_str.is_empty() {
            TagCollection::new()
        } else {
            TagCollection::from_str(&tag_str.trim())?
        };

        // The remainder is the content
        let content = remainder.to_vec();

        let result = Note {
            lvl,
            title,
            creation_date,
            modification_date,
            guid,
            tags,
            content,
        };
        Ok(result)
    }
}

impl From<Vec<String>> for Note {
    fn from(value: Vec<String>) -> Self {
        Self::from_vec(value).unwrap()
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: Vec<String> = self.into();
        write!(f, "{}", result.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let cases = vec![
            vec![
                "# Title",
                "> cre:2022-03-03 mod:2021-03-01 guid:a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8 @aid +project",
                "- This is the content",
                "- A lot of data",
            ],
            vec![
                "### Title",
                "> cre:2022-03-03 mod:2021-03-01 guid:a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8 @aid +project",
                "- This is the content",
                "- A lot of data",
            ],
            vec![
                "# Title",
                "> cre:2022-03-03 mod:2021-03-01 guid:a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
                "- This is the content",
            ],
        ];
        for case in cases {
            let case: Vec<String> = case.iter().map(|&s| s.to_string()).collect();
            let note = Note::from_vec(case.clone());
            println!("{:?} v {:?}", case, &note);
            let roundtrip: Vec<String> = (&note.unwrap()).into();
            assert_eq!(case, roundtrip);
        }
    }
    #[test]
    fn roundtrip_bad() {
        let cases = vec![
            // No content
            vec!["# Title", "> cre:2022-03-03 mod:2021-03-01 @aid +project"],
            // no creation date
            vec![
                "# Title",
                "> mod:2021-03-01 @aid +project",
                "- This is the content",
            ],
            // no lvl info
            vec![
                "Title",
                "> cre:2022-03-03 mod:2021-03-01 @aid +project",
                "- This is the content",
                "- A lot of data",
            ],
            // wrong title lvl format
            vec![
                ">>> Title",
                "> cre:2022-03-03 mod:2021-03-01 @aid +project",
                "- This is the content",
                "- A lot of data",
            ],
            // bad month
            vec![
                "# Title",
                "> cre:2022-13-03 mod:2021-03-01 @aid +project",
                "- This is the content",
                "- A lot of data",
            ],
            // wrong guid format
            vec![
                "# Title",
                "> cre:2022-03-03 mod:2021-03-01 guid:7d8",
                "- This is the content",
            ],
        ];
        for case in cases {
            let case: Vec<String> = case.iter().map(|&s| s.to_string()).collect();
            let note = Note::from_vec(case.clone());
            assert!(note.is_err(), "{:?}", case);
        }
    }
}
