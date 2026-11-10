# Real Gboard geometry on selected Fold Search A

## Purpose and boundary

D51 keeps Search and results on the right of the unfolded Pixel 9 Pro Fold,
with the playback deck at bottom-left.
D50 requires that deck to remain visible while typing,
except for real floating Gboard (D53) and the brief measured Gboard
font-update banner (D54).
The active review at `package/music-player/design/question/current.html` still shows a debug-only 300dp
system-managed input method,
not Gboard.
The captures linked here test selected A with real Gboard.
They **predate** the user's correction to retain the same folder browser
above the deck during typing and the subsequent debug-only height reflow.
The active review now shows that retained browser under the separate 300dp
debug input method;
these older Gboard PNGs must not be read as images of the corrected
upper-left composition.
They remain evidence of the measured keyboard modes,
not production implementation or a new choice between A,
B and C.

## Fixture and comparison limits

Both AVDs used Android Emulator 37.1.11.0,
Android 17 API 37,
390dpi,
an inner panel of 2076 × 2152px and a cover panel of 1080 × 2424px.
The installed debug Search APK had SHA-256
`d895072b4f232181c1f24d9db0bfd6b3afae2e25f869cbe21b0039e81f53`
on both AVDs.
The original `Pixel_9_Pro_Fold` AVD's active updated Gboard was
`18.2.7.969776716-release-x86_64`,
versionCode `175981944`.
Its hidden preloaded package was versionCode `175753756`;
the earlier troubleshooting reading mistook that for the active build.
The original floating screenshots were captured after its recorded
September 24 update.

The disposable `Fold_No_Hardware_Probe` AVD initially ran the preloaded
Gboard versionCode `175753756`.
Installing the original AVD's updated APK splits into the disposable AVD
changed its active versionCode to `175981944`,
but its settled split/full-width geometry remained the same.
This comparison **does not isolate a cause** for the original floating mode:
user data,
Gboard preferences and device settings differ.
The disposable AVD's `hw.keyboard=no` config also did not remove the guest's
`AT Translated Set 2 keyboard`;
Window Manager still reported `qwerty/v/v`.
Do not call it hardware-keyboard-free.

The disposable emulator booted under a 6 GiB RAM and 2 CPU container limit,
using a read-only copy of the host ADB **public** key,
`-skip-adb-auth`,
and clean disposable guest data.
No host private key,
production source or active AVD setting was changed.
See `doc/troubleshooting/android-17-fold-emulator-ime-probe.md`
for the failed lower-memory and ADB-auth probes.

## Settled Gboard typing that fits

- [Inner split keyboard at 100% light][inner-100]:
  real key taps entered `cam`.
  Its IME began at y `1352`;
  the complete deck and both right-side results remained readable.
- [Inner split keyboard at 200% light][inner-200]:
  `Shuffle all folders` ended at y `1313`,
  39px before the keyboard began at y `1352`.
  Its full 131px target height remained visible.
- [Inner split keyboard at 200% dark][inner-dark-200]
  and [after updating disposable Gboard][inner-updated-200]:
  real `cam` input and the same measured final-mode bounds remained visible.
- [Cover full-width keyboard at 100% light][cover-100]
  and [200% light][cover-200]:
  real key taps entered `cam`;
  both result labels remained above the keyboard beginning at y `1605`.
- [Cover full-width keyboard at 200% dark][cover-dark-200]
  and [after updating disposable Gboard][cover-updated-200]:
  the same result-visibility check passed.
  At 200%,
  `Camellia` occupied `[127,350][410,453]` and
  `Another Xronixle` occupied `[127,584][667,687]`.

On the disposable cover,
Android Back dismissed Gboard without clearing `cam` or its results.
Tapping the field reopened the keyboard.
Clear removed the query while retaining focus,
and real-key retyping restored both results.
These are bounded observations of the earlier debug Search composition,
not proof of result activation or rank behavior.
A later disposable-AVD probe of the corrected retained-browser candidate
used a different debug APK SHA-256
`89af26500614050e27cbf81178bc5b4711f3439b37617c79888ed172cdc2f444`
and again entered `cam` with real Gboard at 200% text:
`Folders` and Open remained at upper-left,
and the final mode occupied `[73,1201][965,1332]` before the keyboard's
y `1352` top.
Its raw current APK screenshot remains private;
it is **not** one of the linked sanitized Gboard PNGs.

## Measured occlusions and open failures

- [Original AVD floating Gboard at 200% inner][floating-inner]:
  its touchable key region x `[274,1180)`,
   y `[310,1081)` overlapped the
  playback title `[258,1042][781,1145]`.
  Moving that floating keyboard lower covered more controls.
  The public IME source reported a zero-height bottom inset in this state.
- A later **disposable AVD** test used the corrected retained-browser
  Search prototype at 200% text.
  The actual Gboard Floating keyboard toolbar action displayed keys over
  the deck's `4:35` duration.
  UI Automator placed that duration at `[842,1263][965,1341]`;
  privileged Window Manager reported the floating touch region at
  `[936,1147][1842,1918]`,
  so the right edge of the time was obscured.
  The **app-visible** data was `visible=true`,
  `platformBottom=0`,
  `composeBottom=0`,
  and `boundingRects=[]`.
  No new IME animation callback appeared for this toggle in the captured log.
  The exact debug variant for this first toggle was not recorded.
  Its unsanitized screenshot remains a private scratch file,
  not one of the linked review PNGs.
- An **explicit A-layout retest** launched
  `search-deck-right-lift-retain-results-light` after reboot.
  It used debug APK SHA-256
  `261e608e67029f6361b79a2b26f79851099df9d98f3cdbed637b2e14a4529636`.
  Real floating Gboard occupied privileged touch region
  `[482,1006][1388,1777]` over the deck title
  `[258,1120][781,1223]`;
  their x `[482,781)` and y `[1120,1223]` intersection visibly hid
  title ink and other deck controls in the private screenshot.
  A real key tap changed the focused query from `cam` to `cadm` at its
  mid-word cursor.
  This exact-variant visit did **not** retain a noninitial app-insets log,
  so do not transplant the first probe's public geometry into its row.
  See `doc/troubleshooting/android-17-fold-emulator-ime-probe.md`
  for the source trace and boundary between these visits.
- A later **debug-only layering control on the same APK** used SHA-256
  `e4bfec8e8eb98187a0afc06ef11d587fd3c87623fe45a37da7b0ffd862c6461e`.
  A nonfocusable app panel visibly painted over real floating Gboard.
  An uncovered real key changed the focused query to `m`;
  tapping a key beneath the panel left it at `m`.
  `InputDispatcher` logged a dropped untrusted touch due to the app window.
  After Back hid Gboard,
  the sampled panel remained over the deck.
  This is a **rejected integration probe**,
  not a usable replacement Search design or a public review PNG.
  Its source and exact screen/window evidence are in the same troubleshooting
  document.
- The separate debug-only `-keepclear-` candidate used APK SHA-256
  `e755bf76ed65e45dc4e4ec57f4f55bdbd948902ca6e1940ddf911474ee8a4dd7`.
  App logging and privileged Window Manager agreed that a keep-clear area
  `[0,717][1038,2152]` was registered.
  Gboard's floating keys at `[482,1006][1388,1777]` **partially**
  overlapped it in x `[482,1038)` and y `[1006,1777)`.
  Dragging the keyboard right moved the observed key region to
  `[1025,1006][1931,1777]`,
  still overlapping the requested area by 13px horizontally.
  Dragging it back left returned a larger overlap and visibly covered
  the deck title.
  Those manual drags validate region detection,
  not system cooperation with the hint.
  A real key tap entered `d`.
  This validates the geometry detector and rejects **this best-effort
  hint on this fixture** as automatic floating-key avoidance;
  it does not describe all possible window placements.
  Raw captures remain private.
- [Original AVD floating Gboard at 100% cover][floating-cover]:
  real key taps entered `cam`,
  but its x `[0,830)`,
   y `[304,1025)` key surface covered both result labels.
- [Disposable inner Gboard font-update banner at 200%][font-banner]:
  immediately after a font-scale change,
  `Keyboard font size updated` raised the IME top to y `1140`.
  The final mode occupied only `[73,1076][965,1140]`,
  a visible height of 64px instead of the measured 117px minimum for 48dp.
  Tapping `OK` returned the settled IME top to y `1352`
  and restored the full mode.
  Automatic dismissal and recurrence were not established.

The split/full-width passes do **not** cancel the observed occlusions.
D53 explicitly accepts real floating Gboard hiding the **unfolded deck**;
D51's A selection is unchanged.
D55 separately accepts the measured cover floating-keyboard overlap with
both result labels while the query remains visible.
D54 accepts the observed brief Gboard font-update-banner clip.
D53 and D55 accept their measured floating-Gboard overlays even if that
floating mode remains on screen until the user dismisses it.
D54 accepts only the brief measured font-update banner,
not a persistent or different banner.
Ordinary settled docked keyboards and other input methods remain outside
these scoped exceptions.

## Banner-height fit bound retained as evidence

At 200% font scale,
the settled light capture first paints the deck at y `281` and the
keyboard begins at y `1352`.
That deck takes 1071px.
The banner capture first paints it at y `175`,
just after the 136px status inset and a 16dp,
approximately 39px divider.
With the banner IME beginning at y `1140`,
the safe vertical area from status inset to keyboard is 1004px.
The current divider plus deck needs 1110px,
a 106px shortfall;
the screenshot and final-mode bounds show the resulting clip.

In prototype commit `36f8a8b8e`,
`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/DesignCandidateActivity.kt:982-999`
keeps that divider even when Search hides the folder browser.
`TransportBlock` at the same file's lines 1472 to 1501 also applies
16dp vertical outer padding.
Removing the now-unneeded divider would save about 39px but leave about
67px of the measured shortfall.
Reducing outer padding from 16dp to 8dp on both edges would save about
39px more but still leave about 28px **if the other components kept their
settled heights**.
These are before-state arithmetic,
not a built variant or after-state fit proof.
Do not implement a partial compact fallback,
shrink a 48dp target,
undercut the accepted 8dp group spacing or 12dp horizontal mode padding,
or hide controls to silence this diagnostic.
D54 accepts this measured brief banner overlap without choosing a reflow.
If a different tall keyboard still requires a complete deck,
validate any proposed reflow in a debug-only Compose build with real input
before another design question.

## Evidence and regeneration

Each linked PNG is a full physical-panel capture,
with the left status area replaced by a generic `9:41` clock and image metadata
stripped.
The right status icons,
app content and navigation handle were inspected;
they carry no account,
path,
notification text or other personal identifier in these samples.
Each same-stem JSON file in `package/music-player/design/question/evidence/` contains only whitelisted
synthetic app labels,
focused-field state,
node bounds,
keyboard bounds,
APK hash and AVD/version provenance.
Full raw UI hierarchies and screenshots remain in private agent scratch,
not the repository.

`package/music-player/design/gboard-evidence.mjs` owns the sanitizer and geometry checks.
Regenerate only with the private raw capture directory set as
`MUSIC_PLAYER_GBOARD_INPUT` and run
`mise run //package/music-player/design:evidence:gboard`.
The committed outputs validate without private inputs through
`mise run //package/music-player/design:lint:gboard:evidence`.
Disposable negative controls confirmed that validator rejects a clipped
settled mode,
a falsely passing banner,
result labels moved outside the floating keyboard,
and an unmasked status-corner pixel;
the original records passed after each restoration.

A later debug-only Compose probe measured docked Gboard insets and a
bounding rectangle;
with real floating Gboard it observed visibility but neither a bottom inset
nor a usable floating-key rectangle.
The system `InsetsSource` path is traced in
`doc/troubleshooting/android-17-fold-emulator-ime-probe.md`.
D53 and D54 accept the measured unfolded-deck overlays;
D55 accepts the specific folded-cover floating overlap with both result
labels while `cam` remains visible.
Other keyboard states,
result activation,
long result names and scrolling remain open.

[inner-100]: ../question/evidence/gboard-inner-split-light-s100.png
[inner-200]: ../question/evidence/gboard-inner-split-light-s200.png
[inner-dark-200]: ../question/evidence/gboard-inner-split-dark-s200.png
[inner-updated-200]: ../question/evidence/gboard-inner-updated-split-dark-s200.png
[cover-100]: ../question/evidence/gboard-cover-full-light-s100.png
[cover-200]: ../question/evidence/gboard-cover-full-light-s200.png
[cover-dark-200]: ../question/evidence/gboard-cover-full-dark-s200.png
[cover-updated-200]: ../question/evidence/gboard-cover-updated-full-dark-s200.png
[floating-inner]: ../question/evidence/gboard-inner-floating-overlap-light-s200.png
[floating-cover]: ../question/evidence/gboard-cover-floating-overlap-light-s100.png
[font-banner]: ../question/evidence/gboard-inner-font-banner-light-s200.png
