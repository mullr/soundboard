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
    CommandError, Volume,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::sync::{
    broadcast::{error::SendError, Sender},
    Mutex,
};

use crate::model::CollectionKind;

pub struct Player {
    manager: AudioManager,
    playing: HashMap<ClipId, PlayingSound>,
    pending_events: Vec<PlayerEvent>,
    coll_gain: HashMap<u64, f64>,
}

struct PlayingSound {
    sound_data: StaticSoundData,
    handle: StaticSoundHandle,
    kind: CollectionKind,
}

pub async fn poll_events(
    player_mutex: Arc<Mutex<Player>>,
    sender: Sender<PlayerEvent>,
) -> Result<(), PlayerError> {
    loop {
        let events = {
            let mut player = player_mutex.lock().await;
            player.poll_events()?
        };

        for event in events.into_iter() {
            sender.send(event)?;
        }

        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

#[derive(Debug, Clone)]
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

#[derive(Copy, Hash, Eq, PartialEq, Clone, Debug)]
struct ClipId {
    coll_id: u64,
    clip_id: u64,
}

fn pause_tween() -> Tween {
    Tween {
        duration: Duration::from_millis(500),
        ..Default::default()
    }
}

impl Player {
    pub fn new() -> Result<Player, PlayerError> {
        let manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
            .map_err(PlayerError::CpalError)?;
        let player = Player {
            manager,
            playing: Default::default(),
            pending_events: Default::default(),
            coll_gain: Default::default(),
        };

        Ok(player)
    }

    fn clips_to_play_and_pause(&self) -> (Vec<ClipId>, Vec<ClipId>) {
        let highest_playing = self
            .playing
            .iter()
            .filter(|(_id, ps)| {
                ps.handle.state() == PlaybackState::Playing && ps.kind.priority().is_some()
            })
            // SAFETY: this unwrap is okay because we just checked it with is_some()
            .max_by_key(|(_id, ps)| ps.kind.priority().unwrap());

        let highest_paused = self
            .playing
            .iter()
            .filter(|(_id, ps)| {
                ps.handle.state() == PlaybackState::Paused && ps.kind.priority().is_some()
            })
            // SAFETY: this unwrap is okay because we just checked it with is_some()
            .max_by_key(|(_id, ps)| ps.kind.priority().unwrap());

        match (highest_playing, highest_paused) {
            (None, None) | (Some(_), None) => (vec![], vec![]),
            (None, Some((paused_clip_id, _))) => (vec![*paused_clip_id], vec![]),
            (Some((playing_clip_id, playing_sound)), Some((paused_clip_id, paused_sound))) => {
                if paused_sound.kind.priority() > playing_sound.kind.priority() {
                    (vec![*paused_clip_id], vec![*playing_clip_id])
                } else {
                    (vec![], vec![])
                }
            }
        }
    }

    fn poll_events(&mut self) -> Result<Vec<PlayerEvent>, PlayerError> {
        let (to_play, to_pause) = self.clips_to_play_and_pause();
        for id in to_play.iter() {
            if let Some(playing_sound) = self.playing.get_mut(id) {
                playing_sound.handle.resume(pause_tween())?
            }
        }

        for id in to_pause.iter() {
            if let Some(playing_sound) = self.playing.get_mut(id) {
                playing_sound.handle.pause(pause_tween())?
            }
        }

        let mut to_remove = vec![];
        for (id, playing_sound) in self.playing.iter_mut() {
            match playing_sound.handle.state() {
                PlaybackState::Stopped if playing_sound.kind.loop_playback() => {
                    playing_sound.handle = self.manager.play(playing_sound.sound_data.clone())?;
                    let gain = self.coll_gain.get(&id.coll_id).unwrap_or(&1.0);
                    playing_sound
                        .handle
                        .set_volume(Volume::from(*gain), Tween::default())?;
                }
                PlaybackState::Stopped => {
                    self.pending_events.push(PlayerEvent::Stopped {
                        coll_id: id.coll_id,
                        clip_id: id.clip_id,
                    });
                    to_remove.push((*id).clone());
                }
                _ => (),
            }
        }

        for id in to_remove.into_iter() {
            self.playing.remove(&id);
        }

        let mut res = vec![];
        std::mem::swap(&mut res, &mut self.pending_events);
        Ok(res)
    }

    pub fn playing_clips(&self) -> Vec<(u64, u64)> {
        self.playing
            .keys()
            .map(|id| (id.coll_id, id.clip_id))
            .collect()
    }

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

        if let Some(priority) = kind.priority() {
            // pause any lower priority tracks
            for (_id, playing_sound) in self.playing.iter_mut() {
                if let Some(other_priority) = playing_sound.kind.priority() {
                    if other_priority < priority {
                        playing_sound.handle.pause(pause_tween())?;
                    }
                }
            }
        }

        let sound_data = StaticSoundData::from_file(path, StaticSoundSettings::default())?;
        let duration = sound_data.duration();
        let mut handle = self.manager.play(sound_data.clone())?;

        let gain = self.coll_gain.get(&coll_id).unwrap_or(&1.0);
        handle.set_volume(Volume::from(*gain), Tween::default())?;

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
                    CollectionKind::BackgroundMusic
                    | CollectionKind::Ambience
                    | CollectionKind::BattleMusic => Duration::from_millis(1000),
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

    pub fn set_gain(&mut self, coll_id: u64, gain: f64) -> Result<(), PlayerError> {
        for (id, ps) in self.playing.iter_mut() {
            if id.coll_id != coll_id {
                continue;
            }
            ps.handle.set_volume(Volume::from(gain), Tween::default())?;
        }

        self.coll_gain.insert(coll_id, gain);
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

    #[error(transparent)]
    SendError(#[from] SendError<PlayerEvent>),
}
