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

#[derive(Copy, Clone, Debug, serde::Serialize, Eq, PartialEq)]
pub enum CollectionKind {
    Drops,
    BackgroundMusic,
    BattleMusic,
    Fx,
    Ambience,
}
impl CollectionKind {
    pub fn loop_playback(&self) -> bool {
        match self {
            CollectionKind::Drops | CollectionKind::Fx => false,
            CollectionKind::BackgroundMusic
            | CollectionKind::Ambience
            | CollectionKind::BattleMusic => true,
        }
    }

    pub fn is_exclusive(&self) -> bool {
        match self {
            CollectionKind::Drops
            | CollectionKind::BackgroundMusic
            | CollectionKind::BattleMusic => true,
            CollectionKind::Fx | CollectionKind::Ambience => false,
        }
    }

    // Audio of higher priority will automatically pause anything of
    // lower priority as long as it's playing. None means it doesn't
    // participate in the priority system.
    pub fn priority(&self) -> Option<i8> {
        match self {
            CollectionKind::BackgroundMusic => Some(0),
            CollectionKind::BattleMusic => Some(1),
            CollectionKind::Drops => Some(2),
            CollectionKind::Fx | CollectionKind::Ambience => None,
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

        clips.sort_by_key(|clip| clip.name.clone());

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
