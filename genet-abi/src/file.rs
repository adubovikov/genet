/// File type.
#[derive(Serialize, Deserialize, Debug)]
pub struct FileType {
    name: String,
    extensions: Vec<String>,
}

impl FileType {
    pub fn new(name: &str, extensions: &[&str]) -> Self {
        FileType {
            name: name.into(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
        }
    }
}
