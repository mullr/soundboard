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

use crate::model::CollectionKind;

pub struct Player {
    manager: AudioManager,
    playing: HashMap<ClipId, PlayingSound>,
    pending_events: Vec<PlayerEvent>,
}

struct PlayingSound {
    sound_data: StaticSoundData,
    handle: StaticSoundHandle,
    kind: CollectionKind,
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
        for (id, playing_sound) in self.playing.iter_mut() {
            if playing_sound.handle.state() == PlaybackState::Stopped {
                if playing_sound.kind.loop_playback() {
                    playing_sound.handle =
                        self.manager.play(playing_sound.sound_data.clone()).unwrap();
                } else {
                    self.pending_events.push(PlayerEvent::Stopped {
                        coll_id: id.coll_id,
                        clip_id: id.clip_id,
                    });
                    to_remove.push((*id).clone());
                }
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
        kind: CollectionKind,
    ) -> Result<(), PlayerError> {
        if kind.is_exclusive() {
            self.stop_where(|_, playing_sound| playing_sound.kind == kind)?;
        }

        let sound_data = StaticSoundData::from_file(path, StaticSoundSettings::default())?;
        let duration = sound_data.duration();
        let handle = self.manager.play(sound_data.clone())?;
        self.playing.insert(
            ClipId { coll_id, clip_id },
            PlayingSound {
                sound_data,
                handle,
                kind,
            },
        );
        self.pending_events.push(PlayerEvent::Started {
            coll_id,
            clip_id,
            duration: duration.as_secs_f64(),
        });

        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), PlayerError> {
        self.stop_where(|_, _| true)
    }

    pub fn stop_coll(&mut self, coll_id: u64) -> Result<(), PlayerError> {
        self.stop_where(|id, _| id.coll_id == coll_id)
    }

    pub fn stop_clip(&mut self, coll_id: u64, clip_id: u64) -> Result<(), PlayerError> {
        self.stop_where(|id, _| id.coll_id == coll_id && id.clip_id == clip_id)
    }

    fn stop_where(
        &mut self,
        pred: impl Fn(&ClipId, &PlayingSound) -> bool,
    ) -> Result<(), PlayerError> {
        let mut to_remove = vec![];
        for (id, playing_sound) in self.playing.iter_mut() {
            if !(pred)(id, playing_sound) {
                continue;
            }

            playing_sound.handle.stop(Tween {
                duration: match playing_sound.kind {
                    CollectionKind::Fx | CollectionKind::Drops => Duration::from_millis(200),
                    CollectionKind::Music | CollectionKind::Ambience => Duration::from_millis(1000),
                },
                ..Default::default()
            })?;

            self.pending_events.push(PlayerEvent::Stopped {
                coll_id: id.coll_id,
                clip_id: id.clip_id,
            });
            to_remove.push((*id).clone());
        }

        for id in to_remove.into_iter() {
            self.playing.remove(&id);
        }

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
