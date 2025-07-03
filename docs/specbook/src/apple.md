Sources:

- [Apple Developer Documentation - MPNowPlayingInfoCenter](https://developer.apple.com/documentation/mediaplayer/mpnowplayinginfocenter)
- [Apple Developer Documentation - MPRemoteCommandCenter](https://developer.apple.com/documentation/mediaplayer/mpremotecommandcenter#overview)
- [Apple Developer Documentation - MPMediaItem](https://developer.apple.com/documentation/mediaplayer/mpmediaitem)
- [Apple Developer Documentation - MPMediaItem#General media item property keys](https://developer.apple.com/documentation/mediaplayer/general-media-item-property-keys)

## Notes

- Loading images through data URLs is supported natively by Apple.

## MPNowPlayingInfoCenter

### Working with the default Now Playing info center

#### nowPlayingInfo: [String : Any]?

The current Now Playing information for the default Now Playing info center.

This is the dictionary that can contain properties with keys `NowPlayingInfoProperty*`
and the following subset of `MPMediaItem` properties:

- MPMediaItemPropertyAlbumTitle
- MPMediaItemPropertyAlbumTrackCount
- MPMediaItemPropertyAlbumTrackNumber
- MPMediaItemPropertyArtist
- MPMediaItemPropertyArtwork
- MPMediaItemPropertyComposer
- MPMediaItemPropertyDiscCount
- MPMediaItemPropertyDiscNumber
- MPMediaItemPropertyGenre
- MPMediaItemPropertyMediaType
- MPMediaItemPropertyPersistentID
- MPMediaItemPropertyPlaybackDuration
- MPMediaItemPropertyTitle

#### enum MPNowPlayingInfoMediaType

The type of media currently playing.

### Setting the playback state in macOS

#### playbackState: MPNowPlayingPlaybackState

The current playback state of the app.

#### enum MPNowPlayingPlaybackState

The playback state of the app.

### Now Playing metadata properties

#### MPNowPlayingInfoCollectionIdentifier: String

The identifier of the collection the Now Playing item belongs to.

#### MPNowPlayingInfoPropertyAdTimeRanges: String

A list of ad breaks in the Now Playing item.

#### MPNowPlayingInfoPropertyAvailableLanguageOptions: String

The available language option groups for the Now Playing item.

#### MPNowPlayingInfoPropertyAssetURL: String

The URL pointing to the Now Playing item’s underlying asset.

#### MPNowPlayingInfoPropertyChapterCount: String

The total number of chapters in the Now Playing item.

#### MPNowPlayingInfoPropertyChapterNumber: String

The number corresponding to the currently playing chapter.

#### MPNowPlayingInfoPropertyCreditsStartTime: String

The start time for the credits, in seconds, without ads, for the Now Playing item.

#### MPNowPlayingInfoPropertyCurrentLanguageOptions: String

The currently active language options for the Now Playing item.

#### MPNowPlayingInfoPropertyCurrentPlaybackDate: String

The date associated with the current elapsed playback time.

#### MPNowPlayingInfoPropertyDefaultPlaybackRate: String

The default playback rate for the Now Playing item.

#### MPNowPlayingInfoPropertyElapsedPlaybackTime: String

The elapsed time of the Now Playing item, in seconds.

#### MPNowPlayingInfoPropertyExcludeFromSuggestions: String

A number that denotes whether to exclude the Now Playing item from content suggestions.

#### MPNowPlayingInfoPropertyExternalContentIdentifier: String

The opaque identifier that uniquely identifies the Now Playing item, even through app relaunches.

#### MPNowPlayingInfoPropertyExternalUserProfileIdentifier: String

The opaque identifier that uniquely identifies the profile the Now Playing item plays from, even through app relaunches.

#### MPNowPlayingInfoPropertyInternationalStandardRecordingCode: String

The International Standard Recording Code (ISRC) of the Now Playing item.

#### MPNowPlayingInfoPropertyIsLiveStream: String

A number that denotes whether the Now Playing item is a live stream.

#### MPNowPlayingInfoPropertyMediaType: String

The media type of the Now Playing item.

#### MPNowPlayingInfoPropertyPlaybackProgress: String

The current progress of the Now Playing item.

#### MPNowPlayingInfoPropertyPlaybackRate: String

The playback rate of the Now Playing item.

#### MPNowPlayingInfoPropertyPlaybackQueueCount: String

The total number of items in the app’s playback queue.

#### MPNowPlayingInfoPropertyPlaybackQueueIndex: String

The index of the Now Playing item in the app’s playback queue.

#### MPNowPlayingInfoPropertyServiceIdentifier: String

The service provider associated with the Now Playing item.

#### MPNowPlayingInfoProperty1x1AnimatedArtwork: String (Beta)

1:1 (square) animated artwork for the current media item.

#### MPNowPlayingInfoProperty3x4AnimatedArtwork: String (Beta)

3:4 (tall) animated artwork for the current media item.

## MPRemoteCommandCenter

### Commands

#### pauseCommand: MPRemoteCommand

The command object for pausing playback of the current item.

#### playCommand: MPRemoteCommand

The command object for starting playback of the current item.

#### stopCommand: MPRemoteCommand

The command object for stopping playback of the current item.

#### togglePlayPauseCommand: MPRemoteCommand

The command object for toggling between playing and pausing the current item.

#### nextTrackCommand: MPRemoteCommand

The command object for selecting the next track.

#### previousTrackCommand: MPRemoteCommand

The command object for selecting the previous track.

#### changeRepeatModeCommand: MPChangeRepeatModeCommand

The command object for changing the repeat mode.

#### changeShuffleModeCommand: MPChangeShuffleModeCommand

The command object for changing the shuffle mode.

#### changePlaybackRateCommand: MPChangePlaybackRateCommand

The command object for changing the playback rate of the current media item.

#### seekBackwardCommand: MPRemoteCommand

The command object for seeking backward through a single media item.

#### seekForwardCommand: MPRemoteCommand

The command object for seeking forward through a single media item.

#### skipBackwardCommand: MPSkipIntervalCommand

The command object for playing a previous point in a media item.

#### skipForwardCommand: MPSkipIntervalCommand

The command object for playing a future point in a media item.

#### changePlaybackPositionCommand: MPChangePlaybackPositionCommand

The command object for changing the playback position in a media item.

#### ratingCommand: MPRatingCommand

The command object for rating a media item.

#### likeCommand: MPFeedbackCommand

The command object for indicating that a user likes what is currently playing.

#### dislikeCommand: MPFeedbackCommand

The command object for indicating that a user dislikes what is currently playing.

#### bookmarkCommand: MPFeedbackCommand

The command object for indicating that a user wants to remember a media item.

#### enableLanguageOptionCommand: MPRemoteCommand

The command object for enabling a language option.

#### disableLanguageOptionCommand: MPRemoteCommand

The command object for disabling a language option

## MPMediaItem

#### MPMediaItemPropertyAlbumTitle: String

The title of an album.

#### MPMediaItemPropertyAlbumTrackNumber: String

The track number of the media item, for a media item that is part of an album.

#### MPMediaItemPropertyAlbumTrackCount: String

The number of tracks for the album that contains the media item.

#### MPMediaItemPropertyDiscNumber: String

The disc number of the media item, for a media item that is part of a multidisc album.

#### MPMediaItemPropertyDiscCount: String

The number of discs for the album that contains the media item.

#### MPMediaItemPropertyArtwork: String

The artwork image for the media item.

#### MPMediaItemPropertyAlbumArtist: String

The primary performing artist for an album.


#### MPMediaItemPropertyArtist: String

The performing artists for a media item — which may vary from the primary artist for the album that a media item belongs to.

#### MPMediaItemPropertyGenre: String

The music or film genre of the media item.

#### MPMediaItemPropertyMediaType: String

The media type of the media item.

#### MPMediaItemPropertyPersistentID: String

The key for the persistent identifier for the media item.

#### MPMediaItemPropertyPlaybackDuration: String

The playback duration of the media item.

#### MPMediaItemPropertyTitle: String

The title or name of the media item.

The rest of the properties are omitted, because they can't be set by a Now Playable app.
Setting them in the `nowPlayingInfo` dictionary has no effect.
