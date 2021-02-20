pub mod platform;
mod platform_impl;

pub struct MediaControls {
    controls: platform_impl::MediaControls
}

impl MediaControls {
    pub fn set_playback(&mut self, playing: bool) {
        self.controls.set_playback(playing);
    }
    pub fn set_metadata(&mut self, metadata: MediaMetadata) {
        self.controls.set_metadata(metadata);
    }
    pub fn poll<'f, F>(&mut self, handler: F)
    where
        F: 'f + FnMut(MediaControlEvent) 
    {
        self.controls.poll(handler);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MediaMetadata<'s> {
    pub title: &'s str,
    pub album: &'s str,
    pub artist: &'s str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Next,
    Previous,
}
