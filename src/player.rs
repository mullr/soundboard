use kira::{
    manager::{
        backend::{cpal::CpalBackend, Backend},
        error::PlaySoundError,
        AudioManager, AudioManagerSettings,
    },
    sound::{
        static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
        SoundData,
    },
    tween::Tween,
    CommandError,
};
use std::{collections::HashMap, path::PathBuf, time::Duration};
use thiserror::Error;

pub struct Player {
    manager: AudioManager,
    playing: HashMap<ClipId, StaticSoundHandle>,
}

impl Player {
    pub fn new() -> Result<Player, PlayerError> {
        let manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
            .map_err(PlayerError::CpalError)?;
        let player = Player {
            manager,
            playing: Default::default(),
        };

        Ok(player)
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
        let sound_data = StaticSoundData::from_file(path, StaticSoundSettings::default())?;
        let handle = self.manager.play(sound_data)?;
        self.playing.insert(ClipId { coll_id, clip_id }, handle);

        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), PlayerError> {
        for (_, handle) in self.playing.iter_mut() {
            handle.stop(Tween {
                duration: Duration::from_millis(200),
                ..Default::default()
            })?;
        }

        self.playing.clear();

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    FromFileError(#[from] kira::sound::FromFileError),

    #[error(transparent)]
    CpalError(<CpalBackend as Backend>::Error),

    #[error(transparent)]
    PlaySound(#[from] PlaySoundError<<StaticSoundData as SoundData>::Error>),

    #[error(transparent)]
    Command(#[from] CommandError),
}
