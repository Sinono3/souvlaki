Sources:

- [Windows Documentation - SystemMediaTransportControls](https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrols?view=winrt-26100)
- [Windows Documentation - SystemMediaTransportControlsButton](https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrolsbutton?view=winrt-26100)
- [Windows Documentation - SystemMediaTransportControlsDisplayUpdater](https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrolsdisplayupdater?view=winrt-26100)
- [Windows Documentation - SystemMediaTransportControlsTimelineProperties](https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrolstimelineproperties?view=winrt-26100)

## SystemMediaTransportControls

### Properties

#### AutoRepeatMode	

Gets or sets a value representing the current auto-repeat mode of the SystemMediaTransportControls.

#### DisplayUpdater	

Gets the display updater for the SystemMediaTransportControls which enable updating the information displayed about the currently playing song.

#### IsChannelDownEnabled	

Gets or sets a value that specifies if the channel down button is supported.

#### IsChannelUpEnabled	

Gets or sets a value that specifies if the channel up button is supported.

#### IsEnabled	

Enables and disables the system media transport controls for the app.

#### IsFastForwardEnabled	

Gets or sets a value that specifies if the fast forward button is supported.

#### IsNextEnabled	

Gets or sets a value that specifies if the next button is supported.

#### IsPauseEnabled	

Gets or sets a value that specifies if the pause button is supported.true if the pause button is supported; otherwise, false.

#### IsPlayEnabled	

Gets or sets a value that specifies if the play button is supported.

#### IsPreviousEnabled	

Gets or sets a value that specifies if the previous button is supported.

#### IsRecordEnabled	

Gets or sets a value that specifies if the record button is supported.true if the record button is supported; otherwise, false.

#### IsRewindEnabled	

Gets or sets a value that specifies if the rewind button is supported.

#### IsStopEnabled	

Gets or sets a value that specifies if the stop button is supported.

#### PlaybackRate	

Gets or sets the playback rate of the SystemMediaTransportControls.

#### PlaybackStatus	

Gets or sets the playback status of the media.

#### ShuffleEnabled	

Gets or sets a value representing the current shuffle state of the SystemMediaTransportControls.

#### SoundLevel	

Gets the sound level of the media for the capture and render streams.

#### Remarks

Music and media capture apps should monitor the SoundLevel to determine whether the audio streams on the app have been Muted. For apps using the MediaCapture object, capture will be automatically stopped when the capture streams of the app are muted. Capture is not re-started automatically when the audio streams are unmuted, so the SoundLevel changed notification can be used to restart capture. Use the PropertyChanged event to determine when the SoundLevel property changes.

### Methods

#### GetForCurrentView()	

The system media transport controls for the current view.

#### UpdateTimelineProperties(SystemMediaTransportControlsTimelineProperties)	

Updates the SystemMediaTransportControls timeline properties with the values in the provided object.

### Events

#### AutoRepeatModeChangeRequested	

Occurs when the user modifies the SystemMediaTransportControls auto-repeat mode.

##### Remarks

Registering for this event causes an app to be notified when the SystemMediaTransportControls auto-repeat mode changes. An app can change its auto-repeat behavior based on the request or ignore the request and update the SystemMediaTransportControls by setting the AutoRepeatMode property to a value that reflects the app's actual auto-repeat state.

#### ButtonPressed	

Occurs when a button is pressed on the SystemMediaTransportControls.

##### Remarks

Starting with Windows 10, version 1607, UWP apps that use the MediaPlayer class or AudioGraph class to play media are automatically integrated with the SMTC by default. For some scenarios, you may want to manually control the SMTC. In this case, you should ButtonPressed event to be notified that the user has pressed one of the SMTC buttons. For how-to guidance on manually controlling the SMTC, see Manual control of the System Media Transport Controls.

#### PlaybackPositionChangeRequested	

Occurs when the user modifies the playback position of the SystemMediaTransportControls.

##### Remarks

Registering for this event causes an app to be notified when the SystemMediaTransportControls playback position changes. An app can change its auto-repeat behavior based on the request or ignore the request and update the SystemMediaTransportControls by populating a SystemMediaTransportControlsTimelineProperties object with values indicating the actual playback position and calling SystemMediaTransportControls.UpdateTimelineProperties.

#### PlaybackRateChangeRequested	

Occurs when the user modifies the SystemMediaTransportControls playback rate.

##### Remarks

Registering for this event causes an app to be notified when the SystemMediaTransportControls playback rate changes. An app can change its playback rate based on the request or ignore the request and update the SystemMediaTransportControls by setting the PlaybackRate property to a value that reflects the app's actual playback rate.

#### PropertyChanged	

Occurs when a property on the SystemMediaTransportControls has changed.

##### Remarks

Use the Property value of the event SystemMediaTransportControlsPropertyChangedEventArgs to determine which property has changed.

#### ShuffleEnabledChangeRequested	

Occurs when the user modifies the SystemMediaTransportControls shuffle state.

##### Remarks

Registering for this event causes an app to be notified when the SystemMediaTransportControls shuffle state changes. An app can change its shuffle state based on the request or ignore the request and update the SystemMediaTransportControls by setting the ShuffleEnabled property to a value that reflects the app's actual shuffle state.

## SystemMediaTransportControlsButton

### Variants

- **Play	(0):** The play button.
- **Pause	(1):** The pause button.
- **Stop	(2):** The stop button.
- **Record	(3):** The record button.
- **FastForward	(4):** The fast forward button.
- **Rewind	(5):** The rewind button.
- **Next	(6):** The next button.
- **Previous	(7):** The previous button.
- **ChannelUp	(8):** The channel up button.
- **ChannelDown	(9):** The channel down button.

## SystemMediaTransportControlsDisplayUpdater

### Properties

#### AppMediaId

Gets or sets the media id of the app.

#### ImageProperties

Gets the image properties associated with the currently playing media.

#### MusicProperties

Gets the music properties associated with the currently playing media.

#### Thumbnail

Gets or sets thumbnail image associated with the currently playing media.

#### Type

Gets or sets the type of media.

#### VideoProperties

Gets the video properties associated with the currently playing media.

### Methods

#### ClearAll()	

Clears out all of the media metadata.

#### CopyFromFileAsync(MediaPlaybackType, StorageFile)	

Initialize the media properties using the specified file.

#### Update()	

Updates the metadata for the currently playing media.

## SystemMediaTransportControlsTimelineProperties

### Properties

#### EndTime	

Gets or sets a value representing the end time of the currently playing media item.

#### MaxSeekTime	

Gets or sets a value indicating the latest time within the currently playing media item to which the user can seek.

#### MinSeekTime	

Gets or sets a value indicating the earliest time within the currently playing media item to which the user can seek.

#### Position	

Gets or sets a value representing the current playback position within the currently playing media item.

#### StartTime	

Gets or sets a value representing the start time of the currently playing media item.

## SoundLevel

### Variants

- **Muted (0):** The sound level is muted.
- **Low (1):** The sound level is low.
- **Full (2):** The sound level is at full volume.

## ImageDisplayProperties

### Properties

#### Subtitle

Gets or sets the subtitle of the image.

#### Title	

Gets or sets the title of the image.

## MusicDisplayProperties

### Properties

#### AlbumArtist

Gets or sets the name of the album artist.

#### AlbumTitle

Gets or sets the album title.

#### AlbumTrackCount

Gets or sets the album track count.

#### Artist

Gets or set the name of the song artist.

#### Genres

Gets a modifiable list of strings representing genre names.

#### Title

Gets or set the title of the song.

#### TrackNumber

Gets or sets the track number.

## VideoDisplayProperties

### Properties

#### Genres	

Gets a modifiable list of strings representing genre names.

#### Subtitle

Gets or sets the subtitle of the video.

#### Title

Gets or sets the title of the video.
