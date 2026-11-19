# Single-item repeat in installed media players

Research draft checked by Codex on 2026-11-18T20:51:59-05:00.
The question was which installed graphical music or video players can repeat one file indefinitely.

## Dedicated players

- **VLC 3.0.24**. Cycle the repeat button to **Loop one**.
  The Qt interface distinguishes loop one, loop all and no loop.
  [VLC Qt controller](https://github.com/videolan/vlc/blob/3.0.24/modules/gui/qt/components/controller.cpp#L448-L457)
- **Haruna 1.9.0**. Choose **Repeat item** in the playlist playback menu
  or under **Settings**, **Playlist**, **Playback behavior**.
  The setting repeats the current item indefinitely.
  [Haruna playlist settings](https://github.com/KDE/haruna/blob/v1.9.0/src/qml/Settings/PlaylistSettings.qml#L117-L150)
- **mpv 0.42.0**. Press **Shift+L** in the player window or launch with `mpv --loop-file=inf FILE`.
  The binding toggles single-file repeat.
  [mpv key binding](https://github.com/mpv-player/mpv/blob/v0.42.0/etc/input.conf#L173-L177)
  [mpv option](https://github.com/mpv-player/mpv/blob/v0.42.0/DOCS/man/options.rst#L490-L498)
- **MPlayer 38542**. Launch with `mplayer -loop 0 FILE`.
  The package supplies a graphical video window and a hidden desktop entry.
  The separate `gmplayer` executable is absent.
  [MPlayer manual](https://mplayerhq.hu/DOCS/man/en/mplayer.1.html#:~:text=0%20means%20forever)
- **FFplay 9.0.2**. Launch with `ffplay -loop 0 FILE`.
  The FFmpeg package supplies the graphical playback window without an application-menu entry.
  [FFplay manual](https://ffmpeg.org/ffplay.html#:~:text=0%20means%20forever)
- **Blackmagic RAW Player**, supplied by DaVinci Resolve 21.1.1.
  The transport overlay has a loop button to the right of the forward control.
  The installed binary contains `toggleLoop`, `setLooping` and active and inactive loop icons.
  It opened the bundled `profile.braw` sample in a private compositor and displayed the control.
  The compositor prompted for input-control permission, so the probe did not observe an end-to-start transition.
  [Blackmagic RAW Player](https://www.blackmagicdesign.com/products/blackmagicraw)

## Playback in other applications

- **Telegram Desktop 7.4.4**. Its chat music player cycles through repeat off, repeat one and repeat all.
  The end-of-track handler restarts the same song in repeat-one mode.
  [Repeat button](https://github.com/telegramdesktop/tdesktop/blob/v7.4.4/Telegram/SourceFiles/media/player/media_player_widget.cpp#L149-L159)
  [End-of-track handler](https://github.com/telegramdesktop/tdesktop/blob/v7.4.4/Telegram/SourceFiles/media/player/media_player_instance.cpp#L1478-L1491)
- **Audacity** loops the selected region when Loop is on. Select the whole imported track to play it repeatedly.
  [Audacity Transport Toolbar](https://manual.audacityteam.org/man/playing_and_recording.html#:~:text=loop%2Dplay%20the%20current%20looping%20region%20over%20and%20over)
- **DaVinci Resolve** has **Playback → Loop/Unloop**.
  Its shipped reference manual, chapter 40, page 815, describes continuously looping a source clip or timeline.
- **OBS Studio** can loop a local file through a Media Source using the **Loop** option.
  [OBS FFmpeg source properties](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-ffmpeg/obs-ffmpeg-source.c#L167-L175)

## Inventory and checks

- Inspected `pacman --query --search ''` package descriptions and the desktop entries under
  `/usr/share/applications` and the user application directory, including MIME handlers and hidden launchers.
- Checked system and user Flatpak installs with `flatpak list --app`, which returned no applications.
  Checked Snap and Nix launcher locations, local binary and application directories, `/opt`,
  desktop web-app shortcuts, and installed Steam app manifests.
- Played a disposable 1.17-second MP4 once and with single-item repeat in a private compositor.
  VLC and Haruna reported MPRIS `LoopStatus=Track` and their reported positions restarted repeatedly.
  Both reported `Stopped` at the end with looping disabled.
  mpv, MPlayer and FFplay each reported playback positions starting over in the repeat arm.
  The play-once arms reached the end without restarting the clip.
- Wine 11.22 also ships an executable named `wmplayer.exe`. Its entry point logs a stub message and returns.
  [Wine wmplayer source](https://github.com/wine-mirror/wine/blob/wine-11.22/programs/wmplayer/main.c#L20-L31)
- The list covers conventional installed app locations and discovered players with a built-in repeat mode.
  The browser inventory did not include arbitrary websites or extensions.
