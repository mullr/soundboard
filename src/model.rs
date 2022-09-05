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

    pub fn collection(&self, coll_id: u64) -> Option<&Collection> {
        self.collections.iter().find(|coll| coll.id == coll_id)
    }
}

#[derive(Clone, Debug)]
pub struct Collection {
    pub id: u64,
    pub name: String,
    pub directory: PathBuf,
    pub clips: Vec<Clip>,
    pub kind: CollectionKind,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum CollectionKind {
    Drops,
    Music,
    Fx,
    Ambience,
}
impl CollectionKind {
    pub fn loop_playback(&self) -> bool {
        match self {
            CollectionKind::Drops | CollectionKind::Fx => false,
            CollectionKind::Music | CollectionKind::Ambience => true,
        }
    }
}

impl Collection {
    pub fn from_dir(
        path: impl AsRef<std::path::Path>,
        kind: CollectionKind,
    ) -> std::io::Result<Self> {
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
            kind,
        })
    }

    pub fn clip(&self, clip_id: u64) -> Option<&Clip> {
        self.clips.iter().find(|clip| clip.id == clip_id)
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
