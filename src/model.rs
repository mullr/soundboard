use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
};

#[derive(Default, Clone, Debug)]
pub struct Library {
    pub collections: Vec<Collection>,
}

impl Library {
    pub fn add_collection(&mut self, coll: Collection) {
        self.collections.push(coll);
    }

    pub fn clip_path(&self, coll_id: u64, clip_id: u64) -> Option<PathBuf> {
        Some(
            self.collections
                .iter()
                .find(|coll| coll.id == coll_id)?
                .clips
                .iter()
                .find(|clip| clip.id == clip_id)?
                .path
                .to_owned(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Collection {
    pub id: u64,
    pub name: String,
    pub directory: PathBuf,
    pub clips: Vec<Clip>,
}

impl Collection {
    pub fn from_dir(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let mut clips = vec![];
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                clips.push(Clip::from_file(entry.path())?);
            }
        }

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let id = hasher.finish();

        Ok(Collection {
            id,
            name: path
                .file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or_else(|| "<unknown>".to_string()),
            directory: path.to_owned(),
            clips,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Clip {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,
}

impl Clip {
    fn from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref();

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let id = hasher.finish();

        Ok(Clip {
            id,
            name: path
                .file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or_else(|| "<unknown>".to_string()),
            path: path.to_owned(),
        })
    }
}
