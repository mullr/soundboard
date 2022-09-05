use kira::{
    manager::{
        backend::{cpal::CpalBackend, Backend},
        error::PlaySoundError,
        AudioManager, AudioManagerSettings,
    },
    sound::{
        static_sound::{PlaybackState, StaticSoundData, StaticSoundHandle, StaticSoundSettings},
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
    pending_events: Vec<PlayerEvent>,
}

impl Player {
    pub fn new() -> Result<Player, PlayerError> {
        let manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
            .map_err(PlayerError::CpalError)?;
        let player = Player {
            manager,
            playing: Default::default(),
            pending_events: Default::default(),
        };

        Ok(player)
    }

    pub fn poll_events(&mut self) -> Vec<PlayerEvent> {
        let mut to_remove = vec![];
        for (id, handle) in self.playing.iter() {
            if handle.state() == PlaybackState::Stopped {
                self.pending_events.push(PlayerEvent::Stopped {
                    coll_id: id.coll_id,
                    clip_id: id.clip_id,
                });
                to_remove.push((*id).clone());
            }
        }

        for id in to_remove.into_iter() {
            self.playing.remove(&id);
        }

        let mut res = vec![];
        std::mem::swap(&mut res, &mut self.pending_events);
        res
    }
}

pub enum PlayerEvent {
    Started {
        coll_id: u64,
        clip_id: u64,
        duration: f64,
    },
    Stopped {
        coll_id: u64,
        clip_id: u64,
    },
}

#[derive(Hash, Eq, PartialEq, Clone)]
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
        let duration = sound_data.duration();
        let handle = self.manager.play(sound_data)?;
        self.playing.insert(ClipId { coll_id, clip_id }, handle);
        self.pending_events.push(PlayerEvent::Started {
            coll_id,
            clip_id,
            duration: duration.as_secs_f64(),
        });

        Ok(())
    }

    pub fn stop_clip(&mut self, coll_id: u64, clip_id: u64) -> Result<(), PlayerError> {
        let id = ClipId { coll_id, clip_id };
        if let Some(handle) = self.playing.get_mut(&id) {
            handle.stop(Tween {
                duration: Duration::from_millis(200),
                ..Default::default()
            })?;

            self.pending_events
                .push(PlayerEvent::Stopped { coll_id, clip_id })
        }

        let _ = self.playing.remove(&id);

        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), PlayerError> {
        for (id, handle) in self.playing.iter_mut() {
            handle.stop(Tween {
                duration: Duration::from_millis(200),
                ..Default::default()
            })?;

            self.pending_events.push(PlayerEvent::Stopped {
                coll_id: id.coll_id,
                clip_id: id.clip_id,
            })
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
