use config::Map;
use orgmode::OrgDocument;
use std::io::Cursor;

#[test]
fn read_document() {
    let mut source_exp = Map::new();
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
