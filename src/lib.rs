pub mod platform;
mod platform_impl;

pub struct MediaControls {
    controls: platform_impl::MediaControls,
}

impl MediaControls {
    pub fn attach<F>(&mut self, event_handler: F)
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.controls.attach(event_handler);
    }

    pub fn detach(&mut self) {
        self.controls.detach();
    }

    pub fn set_playback(&mut self, playback: MediaPlayback) {
        self.controls.set_playback(playback);
    }

    pub fn set_metadata(&mut self, metadata: MediaMetadata) {
        self.controls.set_metadata(metadata);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MediaPlayback {
    Stopped,
    Paused,
    Playing,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MediaMetadata<'s> {
    pub title: Option<&'s str>,
    pub album: Option<&'s str>,
    pub artist: Option<&'s str>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
}
