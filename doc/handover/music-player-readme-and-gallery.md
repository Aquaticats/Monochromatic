# Music player README and gallery

## Status

The desktop gallery in `package/music-player/asset/readme/` is current.
It was retaken on 2026-10-13 from a release binary built at the repository head,
hosted in the nested Wayland compositor through the session helpers of the `desktop-e2e` package.
It holds twelve PNGs of the desktop app:
the two-pane window, Search, Settings and the track menu, each in light and dark,
plus the key hint badges (dark), the narrow player (dark), the narrow folder list (light) and the empty library (light).

On 2026-10-14 eight of the twelve were retaken from a release build of commit `4e71750a22`
(the branch `feat/desktop-gallery-retake`, which includes D125, the removal of the folders panel scrollbar):
the two-pane window, the track menu and Search in light and dark, the key hint badges and the narrow folder list.
Those eight showed the scrollbar on the right edge of the folder list (wide window) or of the folder picker (narrow
window), and the fresh frames differ from the committed ones only there and in the deck's elapsed time.
The track list scrollbar is unchanged and still shows.
The other four stay as committed:
a fresh Settings capture in light and in dark and a fresh empty-library capture were pixel-identical to the committed
files (zero differing pixels at a 3 percent color tolerance), which also shows that the regenerated library and the
method reproduce the first gallery,
and a fresh narrow player capture in dark differed only in the elapsed time and the seek knob (131 by 44 pixels).
Playback started from the keyboard this time (`Alt+2`, five `Down` taps, `Enter` on `03 Paper Lanterns`)
instead of a click on a row, then a click on the seek bar (to about 1:20) and a click on a folder row to move the focus
ring off the track row, so the two-pane, menu and Search frames carry no ring until the menu key or `Ctrl` brings it
back, as before.
The routing proof showed one output stream with one link, to the private null sink.
The library had to match the first gallery in three places that show on screen:
the root folder is named `Music`, the nine tracks of `Aster Vale` keep the durations and peaks in the captions
(for example `03 Paper Lanterns` is 4:03 at 0.703 of full scale for -3.5 dBTP), and the loose tracks `Amber Hour`
(29 seconds, -1.0 dBTP) and `Lantern Season` (36 seconds, -7.0 dBTP) feed the Settings preview.

The Android gallery holds eight PNGs of the Pixel 9 Pro Fold emulator (API 37, host GPU) with a generated library of
invented artists:
the cover display (library light, player dark, Search light, track menu dark, Settings light) and the inner display
(two panes light and dark, two-pane Settings dark).
Each was cropped to remove the status bar and gesture bar at native width, with demo mode on while capturing.
Every earlier Android capture was removed because it predated the focus ring, gray and menu decisions (D117 to D119)
or showed names from a private corpus and a carrier name.
Retake the Android gallery after any change to those surfaces, with the facts in "Android captures".

On 2026-10-14 all eight screens were captured again from a debug build of commit `e02e74edec`
(the list focus ring changes of D117 and D120, which put the first list row 3 dp higher),
with the same library, key sequences and crops as the first gallery.
Only `android-cover-menu-dark.png` was replaced, because it is the one image where a reader sees the change:
the ring on the playing row now sits just inside the row's edges, and the menu's focus ring moved with it.
The other seven were kept as committed from the first gallery (a debug build of commit `c6a3f9fde2`, 2026-10-13):
`android-cover-settings-light.png` and `android-inner-settings-dark.png` were pixel-identical to the new captures
(at a 3 percent color tolerance),
and the library, player, Search and two-pane captures differ only by rows 4 to 7 pixels higher, the letter rail and
the playing row's fill reaching the pane edge, with no change in content or rings.
So the eight PNGs do not all come from one commit:
the menu image is from `e02e74edec` and the rest from `c6a3f9fde2`.
If a reader-visible difference in row offset ever matters, the same-commit set is a retake of those seven.
No ring shows on the Search result or on any touch capture, as D117 intends.

All twenty oids in the README were checked against the sha256 of their files after the retake and agree.
The R2 URL check task has not run, since the objects reach R2 on push.

Later on 2026-10-14 `android-cover-search-light.png` was retaken from the debug build of branch
`feat/android-search-restyle` (the D126 restyle of the Search page: header and status strip in the canvas tone,
a thin `outlineVariant` divider, results with a round tonal glyph disc),
with the same library, the key sequence `Ctrl+F`, `lan` and `Back`, and the cover crop `1080x2240+0+125`.
It is the only Android image that shows the Search page (checked with `rg` over the README and by viewing the other
seven: they show the library, player, track menu and Settings pages, where Search is only a header button).
The other seven were not retaken, because none of them draws a surface that D126 changed.
The README oid of the new file equals its sha256 (`b83ee72ae828`), and all twenty embedded oids were checked again.
The desktop Search images (`desktop-search-light.png` and `desktop-search-dark.png`) showed the old page until the
Slint port of D126 landed.

On 2026-10-14 both were retaken from the release build of branch `feat/desktop-search-restyle` (the D126 desktop port,
on top of `a228fca025`), in the nested compositor, with the invented 35-track library and accent `4E5E8B`.
The sequence was the one in "Sequence": `Alt+2`, five `Down` taps and `Enter` on `03 Paper Lanterns`, which since D124
only selects the row, so a second `Enter` on the current row started playback (a transport Play click would do the
same; no row click), then a click on the seek bar (to 1:21 and 1:23), a click on the folder row to clear the focus ring,
the track menu (`Menu`, `Escape`), the hint badges (dark only), `Ctrl+F` and `lan`.
The routing proof showed one output stream with one link, to the private null sink.
The new frames differ from the committed ones only in the Search page (the right half) and in a 156 by 44 pixel
box of the deck (the elapsed time and the seek knob), at a 3 percent color tolerance.
Two details of the method changed since the first retake: the ui_state log line does not follow typing, so wait with a
sleep after `type` and read the screenshot instead of `search_query`; and the compositor's process lookup is
`pgrep --exact music-player`, so a release binary copied under another name starts but is never found, and the routing
proof then reports zero streams (keep the binary named `music-player`, in its own directory).
The new files are `31bce84b95e2` (light, 58 502 bytes) and `3a69f0b9b347` (dark, 59 450 bytes) by their sha256, the
README oids were changed to match (`git cli-git fix --policy markdown/autofix` did not rewrite anything), and all
twenty embedded oids were checked against the files again (zero mismatches).
The R2 check task has not run, since the objects reach R2 on push.

Later on 2026-10-14 two branches changed Android and desktop images that show the Settings list or the track rows.
The record below says who retook what, and the final state of each image.

The branch `feat/auto-play-on-select` (D129, the Auto-play on select switch row) retook the four Settings images,
the only images that show the Settings list (checked with `rg` over the README and by viewing the others):
`desktop-settings-light.png` and `desktop-settings-dark.png` from a release build in the nested compositor (1000 by 800,
accent `4E5E8B`, an invented 35-track library with root `Music`, loose tracks `Amber Hour` and `Lantern Season`,
`Ctrl+Alt+S`, keyboard ring on Track rows, switch off), and `android-cover-settings-light.png` and
`android-inner-settings-dark.png` from the debug build on a private copy of the Pixel 9 Pro Fold emulator
(headful, host GPU, demo mode on, an invented 13-track library `ApsLibrary`, the gear on each display, crops
`1080x2240+0+125` and `2076x1964+0+110`, default off on a fresh install).
The Android library is smaller than the first gallery's, so the track counts differ in those two images.
The new sha256 values were `2848bc4d7c2d` (desktop light), `44d7735d2401` (desktop dark), `eafa459ad9fd` (cover) and
`ae30ec73e56a` (inner).

The branch `fix/android-row-text-alignment` (D130: the track rows' box spans the pane and their text starts in the
column of the headings) retook `android-inner-two-pane-light.png` and `android-inner-two-pane-dark.png` from its debug
build, on the invented 35-track library in demo mode, with the same crop (`2076x1964+0+110`).
The committed pair showed the defect itself: the row text 11 dp right of the subfolder headings and the playing fill
starting at the crease clearance.
`android-cover-player-dark.png` was compared with the fixed build instead of retaken: the cover list region of the
fixed build is pixel-identical to the build before the change (zero differing pixels in four captures), and the
committed image differs from it by the 4 to 7 px row offset and the playing fill starting about 8 px in from the
pane edge, both recorded by the earlier retake and neither caused by this change.
`android-inner-settings-dark.png` was retaken there too, for the template editor change of D130 (the field rows' text
moved 3 dp left under the `Fields` heading, and the editor's hairlines now start at the pane edge), with the same crop;
`android-cover-settings-light.png` shows only the Settings list, which D130 does not touch, so it was kept.
The new sha256 values were `164afb146209` (two-pane light), `bdfe6255fecb` (two-pane dark) and `4155db09c791`
(Settings dark).
The elapsed time on the deck differs from the earlier captures because the invented track resumes where the previous
session left it.

After the two branches met, `android-inner-settings-dark.png` showed only one of the two changes in each branch's
version (the D129 file had the switch row beside the old template editor, the D130 file had the aligned editor and no
switch row), so it was retaken once more from the debug build of the merged head `2285fca73c`, with the invented
13-track library `ApsLibrary`, demo mode, dark appearance, the gear on the inner display and the same crop.
It shows the Playback group with the Auto-play on select switch off, and the aligned template editor (the field text
under the `Fields` heading, the hairlines from the pane edge).
Its sha256 is `0d57342b59f5`, so that is the final value for the inner Settings image; the other three Settings images
and the two two-pane images keep the values above.
Every other Settings and track-row image of both branches was left as its branch committed it.
For all of them the README oids equal the sha256 of their files (all twenty checked after the merged retake, zero
mismatches), and `git cli-git fix --policy markdown/autofix` rewrote nothing.
The R2 check task has not run, since the objects reach R2 on push.

Facts learned: a file pushed to an emulator needs `scan_volume --arg external_primary` (no trailing dot) and a second
scan before `is_music` and `duration` are filled, or the app reports an empty library; an adb server started with
another `HOME` needs that `HOME` for `adb emu kill` (the console token lives there); a debug APK built in another
Gradle home is signed with another key, so `adb uninstall` first; after `cmd device_state base-state 0` the cover
display needs `KEYCODE_WAKEUP` and `wm dismiss-keyguard` before a capture is not black.

On 2026-10-15, after every gallery branch had been pushed to `main`, `mise run
//package/config/lfs-r2-worker:check:markdown-urls` ran and printed `26/26 Worker object URLs verified`, with every
embedded README gallery URL answering `ok`, so the R2 check noted above has now run. Opening the README on github.com
in a logged-out browser was not done.

Last updated: 2026-10-15.

## Goal

`package/music-player/README.md` is the product and contributor entry point.
The component READMEs keep exhaustive commands and implementation detail.
The gallery shows the current committed applications only, never a Claude Design export,
and it covers every discrete user-visible branch at least once rather than every combination of dimensions.
The README states the availability as under active development, build from source.
It must not imply that the repository extraction in `doc/planning/music-player-repository-extraction.md` has happened.

## Desktop capture method

The capture scripts lived in `${HOME}/temp/agent/readme-shots/`, which is scratch and is not kept.
Rebuild them from this description.

### Library

A small script writes mono 16-bit WAV tones at 22 050 Hz (a sine with its fifth and a slow tremolo) into invented
artist folders, thirty-five files in all:
one artist with two albums (nine tracks), twelve more artist folders of one to three tracks each,
and three loose tracks directly in the root so the virtual letter folders appear.
A second, empty directory serves the empty-library capture.
No real library, name or path is ever used.

### Session and audio

`startSession` in `package/music-player/desktop-e2e/src/session.ts` takes the window size, the color scheme, an
accent, the library and a label.
It starts the nested compositor with the release binary in a transient systemd user unit and returns helpers:
`waitState`, `click`, `tap`, `chord`, `down` and `up`, `type`, `last` and `shot`.
The accent was `4E5E8B`.
Its config names the app directory, the compositor path, the release binary, a scratch root and an output root.
`createNullSink` and `currentRouting` in `src/pipewire.ts` give a private PipeWire null sink and prove that the app's
output streams link only to it.
Check the routing once after starting a track.

### Sequence

Wide window, 1000 by 800 pixels:
wait for the player state, wait about four seconds for the analysis to settle, play `03 Paper Lanterns` (the first
gallery clicked its row; the 2026-10-14 retakes used `Alt+2`, five `Down` taps and `Enter`, because a click on an
unselected row only selects it, then, since D124 made Enter on an unselected row select as well, a second `Enter` on the
current row to play), click the seek bar at (190, 648) for about 1:20, click the
folder row at (100, 118) to clear the focus ring, and take the two-pane shot.
Wait for the intended `ui_state` (`waitState`, or a poll of `last` for the latest line, since `waitState` without
`after` also matches older lines) before every shot.
Then `Alt+2`, five `Down` taps and `Menu` for the track menu shot, `Escape`, hold `Ctrl` for 900 ms for the badge shot
(dark only), `Ctrl+F` and type a short query for Search, `Escape`, and `Ctrl+Alt+S` for Settings.

Narrow window, 443 by 900 pixels:
play a row for the player shot, then press the folder button for the folder list shot.
The 2026-10-14 retake played from the keyboard, clicked the seek bar at (172, 748), clicked the folder button at
(60, 28) for the folder list shot, then clicked the `Aster Vale` row at (110, 118), which closes the picker with no
focus ring on the track row, for a player frame to compare with the committed one.

Empty library, 1000 by 800 pixels, light only.

Click coordinates were fixed pixels fitted to the layout at capture time, so find them again after a layout change.

### Finishing and limits

Convert each shot with `magick <in> -strip -define png:exclude-chunks=date,time <out>` into
`desktop-<state>-<scheme>.png`, and read every image in full before committing it.

The nested compositor draws no desktop title bar and no KDE taskbar progress,
so those shell-owned states are not in the gallery.
The Slint embedded MCP server returns a black screenshot from a live winit FemtoVG session,
so use compositor screenshots, as recorded in `doc/troubleshooting/slint-mcp-winit-femtovg-screenshot.md`.
Starting and loading indicators do not last long enough for a stable frame and are left out as intermediate frames.

## Android captures

These facts held for the earlier gallery and are the starting point for the next one.

- The SDK lists one AVD, `Pixel_9_Pro_Fold`.
  Emulator 37.1.11 crashes with software rendering during boot, so launch it with host GPU rendering, read-only, with no
  snapshot and disposable state, and never wipe the retained AVD data.
  Details: `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`.
  The generated `hardware-qemu.ini` can record a stale renderer, so pass the verified arguments explicitly.
- The inner display is 2076 by 2152 at density 390 and the cover display is 1080 by 2424.
  `cmd device_state` lists `CLOSED`, `HALF_OPENED`, `OPENED` and `REAR_DISPLAY_MODE`.
  Read it before each fold capture to confirm the posture.
- A synthetic FLAC library of 120 tracks in ten folders was accepted by MediaStore and by the folder grant.
  Remove stale copies in another format first.
- The media notification and the lock screen need `POST_NOTIFICATIONS` granted in the disposable state.
  Crop those captures to the application-owned media cards.
- Crop app captures to exclude the status bar and the navigation bar, which can carry a carrier name.
- The earlier note that unfolded and tabletop looked the same came from an app that ignored the fold.
  The app now reads the fold through WindowManager (D110), so capture both postures again.

### Method of the gallery

These facts held for the 2026-10-13 gallery and its 2026-10-14 retake.
The scripts were scratch and are not kept, so rebuild them from this description.

- Launch the emulator headful on the host GPU as its own systemd user unit, from a private reflink copy of the owned
  AVD (never the owned AVD), with its own emulator ports and adb server port.
  Delete `/sdcard/Music/RealAppTest` from the copy, push the invented library, and rescan with
  `content call --uri content://media/external_primary --method scan_volume --arg external_primary`.
  Query MediaStore afterwards: only the 35 invented rows may exist.
- The library is the generator of the desktop gallery (35 mono WAV tones, `Aster Vale` with `Salt Garden` of six
  tracks and `Night Ferry` of three, twelve more artist folders, three loose tracks).
- Build the native engine with `mise run //package/music-player/android-app:build:native` on the host, then
  `./gradlew :app:assembleDebug --no-daemon` in a capped container.
  Install the APK, and grant `READ_MEDIA_AUDIO` and `POST_NOTIFICATIONS`.
- Demo mode: `sysui_demo_allowed 1`, then the `com.android.systemui.demo` broadcasts for a 12:00 clock, full battery,
  one Wi-Fi icon, no mobile, no notifications.
  The crop below removes the bars anyway, so demo mode is the second layer.
- Switch panels with `cmd device_state base-state 2` (inner) or `0` (cover), and the scheme with `cmd uimode night`.
  Capture each display with `screencap -d <display id>` (inner `4619827259835644672`, cover `4619827551948147201`),
  force-stop and cold start the app before each screen, and take the capture only after a wait.
- Per screen: select `Aster Vale` and tap `01 Open Window` to play it;
  cover library is the folder button after that; track menu is `Alt+2`, `Menu`, `Down` three times and `Up` once,
  which puts the ring on Start shuffle from here; Search is `Ctrl+F` and the text `lan`, then `Back` to close the
  keyboard; Settings is the gear.
- Crops at native width: the cover display `1080x2240+0+125` of the 1080 by 2424 capture and the inner display
  `1964` high from `+110` of the 2152 capture.
  The status bar's frame is 136 pixels tall on the inner display and its icons end near row 105;
  the navigation bar starts at row 2074 on the inner display.
- Finish with `magick <raw> -crop <geometry> +repage -strip -define png:exclude-chunks=date,time <out>` and confirm
  that the file has only `IHDR`, `IDAT` and `IEND` chunks.
- Compare a new capture with the committed one at a 3 percent color tolerance:
  the deck's elapsed time always differs, so judge by where the differing pixels are.
- Restore the guest afterwards (demo mode off, night mode and posture back to the recorded values), then `sync`, wait
  8 seconds, `sync`, `adb emu kill`.

## Privacy and capture rules

- Use generated names and audio, disposable app state and storage, and never the user's real library.
- Inspect every region of every image, and mask or remove notifications, account names, host paths, device
  identifiers and unrelated windows.
- Strip image metadata, and do not capture audio.
- Confirm the intended rendered state before each capture: finished input is not a finished frame.
- End the nested compositor units and any emulator processes after the captures.

## Images on GitHub

The PNGs are Git LFS objects stored on the R2 worker, and GitHub cannot render those from a relative link.
The full record is `doc/planning/lfs-readme-image-rendering.md`.

- The raw URL of an R2-only object returns the pointer text, and `media.githubusercontent.com` returns 404.
  The README therefore embeds `https://monochromatic-lfs.aquaticat.workers.dev/<oid>/<repo path>`.
  The host changed from `an1298.workers.dev` on 2026-10-01.
- Write a relative image link and commit.
  The `markdown/autofix` policy of `git-policy-cli` (rule `lfs-image-url` of the markdown linter) puts the Worker URL
  into the committed file, and a replaced image gets its new oid the same way.
  The working tree keeps the relative link until you check the file out again.
- The objects reach R2 on push, so the URLs resolve only afterwards.
  `mise run //package/config/lfs-r2-worker:check:markdown-urls` sends a `HEAD` request for every embedded URL and
  asserts status 200, an image content type, and the pointer's size.
- On 2026-10-13 the oid in each of the twelve desktop README image URLs equals the sha256 of its file,
  and so does each of the eight Android ones.
  On 2026-10-14, after the menu image was replaced, all twenty embedded URLs were checked against their files again.
  The markdown autofix did not rewrite the link inside the commit, so running
  `git cli-git fix --policy markdown/autofix -- package/music-player/README.md`
  wrote the new oid into the working tree, and a second commit recorded it.

## Next actions

1. Push, run the URL check task, and open the README on github.com in a logged-out browser to confirm that all twenty
   images load (`naturalWidth` above zero).
2. Retake the Android gallery after the next visible change, with the facts in "Android captures".
3. Keep the image store of the extracted repository undecided until the extraction plan settles it.

## References

- [MusicBee library navigation elements](https://musicbee.fandom.com/wiki/Library_Navigation_Elements),
  [Roon folder browsing](https://blog.roonlabs.net/folder-browsing/) and
  [foobar2000 Facets](https://www.foobar2000.org/components/view/foo_facets) are the navigation precedent that the
  library design weighed.
- IntelliJ IDEA treats Control twice as Run Anything and Shift twice as Search Everywhere.
  The app's double Shift opens Search, as decided in `package/music-player/design/decisions.md` (D106).
