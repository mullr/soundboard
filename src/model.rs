use std::path::PathBuf;

#[derive(Default)]
pub struct Library {
    pub collections: Vec<Collection>,
}

impl Library {
    pub fn add_collection(&mut self, mut coll: Collection) {
        coll.id = self.collections.len() as u64;
        self.collections.push(coll);
    }

    pub fn clip_path(&self, coll_id: u64, clip_id: u64) -> Option<PathBuf> {
        Some(
            self.collections
                .get(coll_id as usize)?
                .clips
                .get(clip_id as usize)?
                .path
                .to_owned(),
        )
    }
}

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
        let mut next_id = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let mut clip = Clip::from_file(entry.path())?;
                clip.id = next_id;
                next_id += 1;

                clips.push(clip);
            }
        }

        Ok(Collection {
            id: 0,
            name: path
                .file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or_else(|| "<unknown>".to_string()),
            directory: path.to_owned(),
            clips,
        })
    }
}

pub struct Clip {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,
}

impl Clip {
    fn from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        Ok(Clip {
            id: 0,
            name: path
                .file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or_else(|| "<unknown>".to_string()),
            path: path.to_owned(),
        })
    }
}
