use rodio::{OutputStreamHandle, OutputStream};
use std::{collections::HashMap, path::PathBuf};

pub struct Player {
    playing: HashMap<ClipId, rodio::Sink>,
    stream_handle: OutputStreamHandle,
}

impl Player {
    pub fn new() -> (Player, OutputStream) {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let player = Player {
            playing: Default::default(),
            stream_handle,
        };

        (player, stream)
    }
}

#[derive(Hash, Eq, PartialEq)]
struct ClipId {
    coll_id: u64,
    clip_id: u64,
}

impl Player {
    pub fn play_clip(&mut self, coll_id: u64, clip_id: u64, path: PathBuf) {
        use std::fs::File;
        use std::io::BufReader;

        let file = BufReader::new(File::open(path).unwrap());
        let sink = self.stream_handle.play_once(file).unwrap();
        self.playing.insert(ClipId { coll_id, clip_id }, sink);
    }

    pub fn stop_all(&mut self) {
        self.playing.clear();
    }
}
