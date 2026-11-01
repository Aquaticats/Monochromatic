# Music player on Android pauses for a notification sound instead of ducking

## Symptom

The human reported (2026-10-22, in Chinese) that other apps lower their volume for about a second when a message
sound plays, while the music player pauses, plays only the message sound, and resumes after it ends.

## Cause found by reading the source

Not reproduced with sound on a device: the human asked for no loudspeaker playback (see Not verified).
Two pieces of `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/RustEngine.kt`
produce the pause, and both are needed for it:

- The focus request is built with `.setWillPauseWhenDucked(true)`, whose comment says a music player prefers a clean
  pause over playing quietly.
  Android documents that setting as telling the system to use the app's callback instead of ducking the app by itself.
- `onFocusChange` handles `AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK` in the same arm as `AUDIOFOCUS_LOSS_TRANSIENT`: it
  remembers whether it was playing, calls `nativeEnginePause`, and resumes on `AUDIOFOCUS_GAIN`.

A notification sound asks for transient focus that may duck, so the app receives the can-duck loss and pauses.

## What the platform documents

From the Android audio focus guide (`developer.android.com/media/optimize/audio-focus`, fetched 2026-10-22):

- Automatic ducking (Android 8.0 and later) applies when the app that has focus is not playing speech, did not set
  `setWillPauseWhenDucked(true)`, and another app requests transient focus that may duck.
  The system then lowers the first app's players and restores them when the second app abandons focus.
- In that case the first app "does not usually receive an `onAudioFocusChange()` callback".
- Without automatic ducking, the guide says the app "should duck ... or pause" on a transient can-duck loss.

## Why the framework cannot duck this app

Read from the Android 16 source (`android16-release` branch, fetched 2026-10-23), not measured on a device.

- `services/core/java/com/android/server/audio/PlaybackActivityMonitor.java` lists
  `UNDUCKABLE_PLAYER_TYPES` as `PLAYER_TYPE_AAUDIO` and `PLAYER_TYPE_JAM_SOUNDPOOL`, with the comment
  "TODO support VolumeShaper on those players".
  `duckPlayers` returns false for the whole app when any of its started players is of those types.
- The engine plays through the NDK AAudio C API (`package/music-player/android-app/rust/Cargo.toml` links the sysroot
  `libaaudio`), so automatic ducking would not lower it even without the flag.
- `FocusRequester.java` reports the loss to the app's listener whenever the framework does not duck, and always when
  the request carries `AUDIOFOCUS_FLAG_PAUSES_ON_DUCKABLE_LOSS`, which `setWillPauseWhenDucked(true)` sets, unless
  the winner forces a duck.

So the listener has to duck, and the flag is what guarantees it is told.
The reason the commit that added audio focus (`6ec84c8419`, 2026-06-13) chose pausing is not recorded; it says only
that it mirrors what ExoPlayer gave the Media3 flavor; no compatibility problem is recorded anywhere in the repository.
The source reading above shows only that the framework would not have ducked this engine.

## Fix

Applied as D134 in `package/music-player/design/decisions.md`: the flag stays, the can-duck arm of `onFocusChange` in
`RustEngine.kt` lowers the engine volume to a fifth (the level Android's own duck ramps to) and the gain arm restores
it; a call or a permanent loss still pauses.
The rule is `volumeWhileDucked` in `core/FocusDuck.kt` with `FocusDuckTest`.

## Not verified

The human asked for no loudspeaker playback (tenants in the building), and the microphone was too insensitive to
hear the phone anyway, so nothing was played.
The check left for the human is to play music and let a message sound arrive: the music should drop for the length of
the sound and come back, and a phone call should still pause it.
