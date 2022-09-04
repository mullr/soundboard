use rodio::{OutputStream, OutputStreamHandle};
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

pub struct Player {
    playing: HashMap<ClipId, rodio::Sink>,
    stream_handle: OutputStreamHandle,
}

impl Player {
    pub fn new() -> Result<(Player, OutputStream), PlayerError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let player = Player {
            playing: Default::default(),
            stream_handle,
        };

        Ok((player, stream))
    }
}

#[derive(Hash, Eq, PartialEq)]
struct ClipId {
    coll_id: u64,
    clip_id: u64,
}

impl Player {
    pub fn play_clip(
        &mut self,
        coll_id: u64,
        clip_id: u64,
        path: PathBuf,
    ) -> Result<(), PlayerError> {
        use std::fs::File;
        use std::io::BufReader;

        let file = BufReader::new(File::open(path)?);
        let sink = self.stream_handle.play_once(file)?;
        self.playing.insert(ClipId { coll_id, clip_id }, sink);

        Ok(())
    }

    pub fn stop_all(&mut self) {
        self.playing.clear();
    }
}

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    PlayError(#[from] rodio::PlayError),

    #[error(transparent)]
    StreamError(#[from] rodio::StreamError),
}
