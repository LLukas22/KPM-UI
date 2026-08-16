pub mod kpm;
mod manifest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub version: Option<[u64; 3]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub url: String,
}

impl Package {
    pub fn version_text(&self) -> String {
        self.version
            .map(|version| format!("{}.{}.{}", version[0], version[1], version[2]))
            .unwrap_or_default()
    }
}
