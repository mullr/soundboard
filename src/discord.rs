use std::{collections::HashMap, error::Error, sync::Arc};

use futures::StreamExt;
use tokio::sync::RwLock;
use twilight_model::{
    channel::{Channel, ChannelType},
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
    user::CurrentUserGuild,
};

#[derive(Debug)]
pub struct DiscordConnection {
    http: twilight_http::Client,
    trackdata: RwLock<HashMap<twilight_model::id::Id<GuildMarker>, songbird::tracks::TrackHandle>>,
    songbird: songbird::Songbird,
    standby: twilight_standby::Standby,
}

impl DiscordConnection {
    /// Connect to a discord voice channel, and start relaying float pcm audio from audio_consumer.
    pub async fn connect(
        audio_consumer: ringbuf::HeapConsumer<u8>,
        token: String,
    ) -> Result<Arc<Self>, Box<dyn Error + Send + Sync>> {
        let (mut events, conn) = {
            let http = twilight_http::Client::new(token.clone());
            let current_user = http.current_user().exec().await?;
            let user_model = current_user.model().await?;

            let intents = twilight_gateway::Intents::GUILD_MESSAGES
                | twilight_gateway::Intents::GUILD_VOICE_STATES;
            let (cluster, events) = twilight_gateway::Cluster::new(token, intents).await?;
            cluster.up().await;

            let songbird = songbird::Songbird::twilight(Arc::new(cluster), user_model.id);

            (
                events,
                Arc::new(DiscordConnection {
                    http,
                    trackdata: Default::default(),
                    songbird,
                    standby: twilight_standby::Standby::new(),
                }),
            )
        };

        // spawn a task to service discord events, and relay them to the songbird lib
        let conn_for_loop = conn.clone();
        tokio::task::spawn(async move {
            while let Some((_, event)) = events.next().await {
                dbg!(&event);
                conn_for_loop.standby.process(&event);
                conn_for_loop.songbird.process(&event).await;
            }
        });

        // Join the first guild
        let guilds = conn.get_guilds().await?;
        let guild = guilds.get(0).expect("no guilds!");
        println!("Joining guild '{}' (id={})", guild.name, guild.id);

        let channels = conn.get_channels(guild.id).await?;
        let voice_channels = channels
            .iter()
            .filter(|ch| ch.kind == ChannelType::GuildVoice)
            .collect::<Vec<_>>();

        // Find the first voice channel
        let channel = voice_channels.get(0).expect("no voice channels!");
        println!(
            "Joining channel '{}' (id={})",
            channel.name.as_deref().unwrap_or_default(),
            channel.id
        );

        let (_handle, success) = conn.songbird.join(guild.id, channel.id).await;
        dbg!(success);

        conn.send_message(channel.id, "BLEEP BLOOP").await?;

        let reader = songbird::input::reader::Reader::Extension(Box::new(RingBufferMediaSource {
            audio_consumer,
        }));

        let input = songbird::input::Input::float_pcm(true, reader);

        if let Some(call_lock) = conn.songbird.get(guild.id) {
            let mut call = call_lock.lock().await;
            let handle = call.play_source(input);

            handle.play();

            let mut store = conn.trackdata.write().await;
            store.insert(guild.id, handle);
        }

        Ok(conn)
    }

    async fn get_guilds(&self) -> Result<Vec<CurrentUserGuild>, Box<dyn Error + Send + Sync>> {
        let guilds = self
            .http
            .current_user_guilds()
            .exec()
            .await?
            .model()
            .await?;
        Ok(guilds)
    }

    async fn get_channels(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<Channel>, Box<dyn Error + Send + Sync>> {
        let channels = self
            .http
            .guild_channels(guild_id)
            .exec()
            .await?
            .model()
            .await?;

        Ok(channels)
    }

    async fn send_message(
        &self,
        channel_id: Id<ChannelMarker>,
        message: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.http
            .create_message(channel_id)
            .content(message)?
            .exec()
            .await?;

        Ok(())
    }
}

struct RingBufferMediaSource {
    audio_consumer: ringbuf::HeapConsumer<u8>,
}

impl songbird::input::reader::MediaSource for RingBufferMediaSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}

impl std::io::Read for RingBufferMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read_bytes = self.audio_consumer.pop_slice(buf);
        if read_bytes == 0 {
            // HACK synthesize some silence to keep the audio channel running
            // TODO hit the play button only when we're actually playing
            for entry in buf.iter_mut().take(1920) {
                *entry = 0;
            }
            Ok(1920)
        } else {
            Ok(read_bytes)
        }
    }
}

impl std::io::Seek for RingBufferMediaSource {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Err(std::io::ErrorKind::Unsupported.into())
    }
}
