use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub dependencies: Vec<String>,
}

pub type HasteMap = Vec<SourceFile>;
