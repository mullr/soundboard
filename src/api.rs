use serde::Serialize;

use crate::model;

#[derive(Serialize)]
pub struct Library {
    pub collections: Vec<Collection>,
}

impl From<model::Library> for Library {
    fn from(m: model::Library) -> Self {
        Library {
            collections: m.collections.into_iter().map(|c| c.into()).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub clips: Vec<Clip>,
}

impl From<model::Collection> for Collection {
    fn from(m: model::Collection) -> Self {
        Collection {
            id: m.id.to_string(),
            name: m.name,
            clips: m.clips.into_iter().map(|c| c.into()).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct Clip {
    pub id: String,
    pub name: String,
}

impl From<model::Clip> for Clip {
    fn from(m: model::Clip) -> Self {
        Clip {
            id: m.id.to_string(),
            name: m.name,
        }
    }
}
