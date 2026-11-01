# Debug IME transition and unfolded deck fit

## Boundary

D50 requires the complete unfolded deck throughout Search typing,
except for the measured real floating Gboard overlap (D53) and brief
real font-update banner (D54).
The accepted [Search A review](../questions/current.html) uses a 300dp debug IME.
The earlier [Gboard evidence](gboard-geometry.md) records a real transient
font-update banner that raised the IME top to y `1140` and clipped the final mode.
The separate 415dp `FoldBannerInputMethod` approximates that reported height,
but **does not reproduce Gboard's banner**.
The upper-left keeps the same folder browser in every candidate here.

## Disposable fixture and controls

A private Pixel 9 Pro Fold AVD (`Fold_No_Hardware_Probe`) ran in a 6 GiB,
2 CPU container on its 2076 × 2152px inner panel at 390dpi and 200% font scale.
The tested debug APK SHA-256 was
`ac9a9570028aec916d2df704b403dac032165daf2cbb1daded78960b1362f7b6`,
built from prototype commit `d98a890b2`.
Both candidate visits used that APK,
the same system-managed 415dp debug IME,
a focused `cam` query,
and a physical-display `screenrecord` at 1038 × 1076px.
This is half the physical pixel count in each dimension;
frame geometry must be doubled before comparison with native PNGs.
The video captures remain private because their status region was not sanitized.

- **Failure control:**
  `search-deck-right-lift-bannerfit-retain-results-light` retained the
  earlier fixed 1000px Compose-bottom trigger.
  App logs showed the platform IME bottom already at `1011px` while
  Compose reported `681px` and `973px`;
  the compact-deck branch was still false.
  It became true when Compose reported `1011px`.
  In the 24-frame recording,
  two visible-IME frames lacked the final mode's bottom outline.
  Frame 016 showed `Shuffle all` cut at the dark IME edge.
  The same five-outline scan detected the complete mode before and after
  those frames,
  so the scan can distinguish a failure from a passing state.
- **First measured-fit attempt:**
  `search-deck-right-lift-autofit-retain-results-light` measured the closed
  deck,
  compared it with available height using the platform IME target,
  and selected the compact branch at the first logged `681px` Compose
  bottom.
  It also added `platform bottom - Compose bottom` as ordinary padding
  after `imePadding()`.
  In the 27-frame recording,
  frames 017 and 018 lacked most of the mode group,
  with a blank strip between app content and the keyboard.
  The same geometry scan reported only two of the five mode outlines in
  those frames.
  This **failed** D50 more visibly despite the earlier branch decision.

The app logs describe values sampled during composition;
`imePadding()` is resolved during layout.
The recordings demonstrate that combining the sampled difference with
`imePadding()` did **not** yield a reliable reservation in this fixture.
They do not establish an AndroidX bug or prove the precise callback order.
The experiment's raw logs are in private scratch as
`fixed-before-preclear-ime-transition.log` and
`auto-reserve-failed-ime-transition.log`;
full videos and extracted frames have matching names there.

## Layout-time-only retest

Prototype commit `b16c667d4` removed the ordinary extra padding.
The platform target now selects only whether to use the compact deck;
`imePadding()` alone owns bottom reservation.
The rebuilt debug APK SHA-256 was
`e5b998268d5e28d65006e73ffa642e1aee58c4b0818f0511d88a3a19a3f19f9f`.
In a new 26-frame recording at approximately 10fps,
13 sampled frames included the visible debug IME;
the five-outline check found the full final mode in each of those frames.
The platform already reported a `1011px` target when Compose first reported
`44px`,
and the compact branch was true at that sample.
The earlier fixed-trigger control had two clipped visible-IME frames,
but was recorded with the previous APK;
it proves the image check can expose a clip,
not a same-APK performance comparison.
The new recording jumped from an IME top around y `2108` to y `1140`
between captured frames,
so it **does not prove** visibility at every intermediate animation frame.
The corrected video and logs remain private scratch files named
`auto-ime-transition.mp4` and `auto-ime-transition.log`.

## In-place height steps

Prototype commit `dccedba55` added a separate debug-only input method whose
buttons changed its actual system-managed height without losing Search focus.
The APK SHA-256 for this step study was
`838f0f11294e5d824768a1d57dd5b471a8aa1c93f9b7dbca6f8b7ba8adcf375c`.
On the same unfolded 200% Fold,
its 330,
360,
370,
372,
375,
400 and 415dp states reported IME tops of y `1348`,
`1275`,
`1251`,
`1246`,
`1238`,
`1177` and `1141`,
respectively.
The first 330 to 360dp step changed the platform bottom inset from
`804` to `877px`,
a positive control that the button altered the actual IME window.

The measured-fit candidate selected the compact branch by 360dp.
At every stepped endpoint,
all four modes had full 131px final-row bounds ending 20px before the
actual IME,
and title,
metadata,
seek text,
transport icons and both right-side results remained in their safe areas.
This is **endpoint** evidence,
not an animation proof.
The unchanged vertical-deck control stayed complete at 360,
370,
372 and 375dp;
it first clipped at 400dp,
where its final mode was only 120px tall and ended exactly at the IME top.
It switched to compact at 415dp.
Thus the closed-deck height used by the first fit estimate caused an
earlier-than-needed design change.

The passing vertical deck's app `ScrollView` spanned 1033px at 360 and
375dp;
its 16dp separator occupied another approximately 39px.
At 400dp,
the available area from y `136` to IME top y `1177` was 1041px,
less than that measured combined demand.
The clipped `ScrollView` was only 1002px tall there.
Those measurements motivated prototype commit `ad53f5685`,
which preferred a noncompact keyboard-open deck sample when it had positive
space above it.
The next same-APK step run disproved this version:
it learned a `1033px` open deck at 330dp and retained it through 375dp,
then reported only `1002px` at 400dp.
That input selected the noncompact branch even though the final mode was
only `[73,1057][965,1177]`,
a visible 120px ending at the y `1177` IME top.
The browser header was absent from that hierarchy.
The callback apparently combined a smaller constrained viewport with an
older available-height value;
its exact scheduling was not captured,
so this is an inference,
not a proven AndroidX cause.
The APK SHA-256 for this failing revision was
`ba9925858eb04d51515647a93f3a22654c78dbbc3d80c3533f1ac8f3b75156a9`.
Prototype commit `ae10caf0d` retains the largest fitting height for
the unchanged content,
width and text scale rather than overwriting it with a shorter sample.
With APK SHA-256
`74d145d5cea646c022b81ae6c325c14b990494313016b2b24ec17254da7a0b51`,
the app learned `1033px` of open deck demand at 330dp and retained it
through the height steps.
The complete final mode remained 131px high and ended 20px before the
IME at every ascending **and descending** endpoint.
The normal vertical deck stayed selected through 375dp;
the compact branch became active at 400 and 415dp and reverted at 375dp.
At 375dp the `1033px` deck plus 39px divider fit within 1102px;
at 400dp the same demand exceeded the 1041px available area.
This is an in-place **endpoint** result,
not proof of visibility on every intervening rendered frame or on
cold entry directly into an intermediate keyboard height.

## Recorded in-place height jump

On the same debug APK used for the ascending and descending steps,
the adjustable IME moved from 375dp (y `1238`) to 400dp (y `1177`)
while the `cam` editor remained focused.
A native half-resolution `screenrecord` sampled the unchanged fixed-trigger
control in 24 frames and the measured-fit variant in 28 frames.
The control was missing the final mode's bottom outline in 20 recorded
IME-visible frames;
the measured-fit variant was missing it in 10.
In the latter,
frame 001 showed the full mode with IME top y `619` in video pixels,
frames 004 to 013 showed only earlier mode boundaries after the IME top
reached y `588`,
and frame 014 showed the full mode again.
The current branch was logged as compact with platform bottom `975px`,
but **that log does not prove it was drawn before the clipped frames**.
The same candidate therefore still fails D50 during this in-place resize,
even though both endpoint hierarchies pass.

The recordings and extracted frames stay in private scratch as
`fixed-before-preclear-height-jump.mp4`,
`auto-apply-logging-only-height-jump.mp4` and their matching private
frame directories.
FFmpeg's `image2` muxer reported non-monotonically increasing timestamps
while extracting each full frame count;
do not infer the duration of the failure from nominal frame rate.
The five-outline check's fixed control establishes it detects a real clipped
mode,
but no sampled-frame pass would prove every unrecorded instant.
A response must act before or alongside IME geometry movement,
not only after the app recomposes from its new bottom inset.

## Animation callback control

A debug-only `WindowInsetsAnimation.Callback` was attached to the
`ComposeView` parent with subtree dispatch preserved.
On first showing the 330dp debug IME,
its `SearchAnimationProbe` logged `prepare`,
`start lower=0 upper=804`,
progress bottoms `0`,
`652`,
`785`,
`804`,
and `end`.
That positive control proves the callback was installed and could report a
normal show animation.
When the same focused input view increased in place from 330 to 360dp,
`SearchInsetProbe` reported the actual IME bottom change from `804` to
`877px`,
but no `SearchAnimationProbe` event appeared.
The same absence held for 375 to 400dp,
when the bottom changed from `914` to `975px` and the final mode clipped
briefly in recorded frames.
This fixture did **not** dispatch a public animation callback for those
in-place height steps;
it is not evidence that a real Gboard banner behaves identically.

Android's `WindowInsetsAnimation.java:331-363` specifies the sequence
**when an insets animation occurs**:

```java
* <li>onPrepare is called on the view hierarchy listeners</li>
* <li>{@link View#onApplyWindowInsets} will be called with the end state of the
*     animation</li>
* <li>View hierarchy gets laid out according to the changes the application has
*     requested due to the new insets being dispatched</li>
* <li>{@link #onStart} is called <em>before</em> the view
*     hierarchy gets drawn in the new laid out state</li>
```

`WindowInsetsAnimation.java:202-208` describes size-change bounds:

```java
* However, if the size of a window that causes insets is changing, these are the
* lower/upper bounds of that size animation.
```

Those provisions do not require every in-place resize to start an
animation.
The observed step cannot be fixed by an `onStart` callback that never ran.

A separate debug-only `OnApplyWindowInsetsListener` on that parent received
bottom `804px` during keyboard show before the animation's `onStart` log;
the complete final mode still ended before the keyboard.
For the non-animated 330 to 360dp step,
it logged bottom `877px` before `SearchInsetProbe` logged the new Compose
bottom.
For the critical 375 to 400dp step,
it logged bottom `975px` before `SearchInsetProbe` selected compact layout.
These log timestamps do **not** prove that Compose could redraw before the
keyboard moved.
`View.java:13048-13086` makes an insets listener replace the parent
view's `onApplyWindowInsets` policy:

```java
public void setOnApplyWindowInsetsListener(OnApplyWindowInsetsListener listener) {
    getListenerInfo().mOnApplyWindowInsetsListener = listener;
}
```

Its dispatch branch at `View.java:13081-13089` calls the listener
**instead of** the view default:

```java
if (mListenerInfo != null && mListenerInfo.mOnApplyWindowInsetsListener != null) {
    return mListenerInfo.mOnApplyWindowInsetsListener.onApplyWindowInsets(this, insets);
} else {
    return onApplyWindowInsets(insets);
}
```

The throwaway listener calls `view.onApplyWindowInsets(insets)` and preserves
subtree dispatch in this measured fixture.
That is not evidence that an unrelated existing parent listener could be
safely replaced in production.
A forked debug revision (`7d17bda51`) also wrote a remembered Compose state
from that parent listener when it received the new bottom inset.
Its APK SHA-256 was
`86b4b7f7750304c735ee12b8f77e57859cfb315169a5c1fcb7e59a7294695a3b`.
The system delivered bottom `975px` before `SearchInsetProbe` logged compact
selection,
but the same-AVD in-place 375 to 400dp recording still showed a
partly hidden final mode in frames 005 to 007.
Frame 008 showed its lower outline restored.
The recording sampled 23 frames;
the fixed-trigger control recording sampled 24 frames under a previous
APK,
so their different failed-frame counts do **not** isolate a causal
improvement from the state write.
The current `auto-height-jump.mp4` and its frame directory remain private;
the earlier listener-logging-only run was moved to
`auto-apply-logging-only-height-jump.mp4` and its matching frame directory.
An earlier app callback does not establish that a new Compose layout is
presented before the keyboard surface moves.
The full deck is **still not continuously visible** under this synthetic
in-place resize.

## Anticipatory clearance prototype

The reactive branch still lost recorded mode borders during the in-place
height jump.
A separate **unaccepted**,
debug-only candidate from commit `02be4b162` reserved 416dp at the bottom whenever the Search editor gained focus;
it used that single reservation instead of `imePadding()` and selected the
inline deck before the keyboard rose.
The 416dp band is a measured test envelope for the y-`1140` Gboard banner,
**not** a maximum keyboard size.
In one visit,
`SearchInsetProbe` recorded focus and the reserved deck while both IME
bottoms were zero,
then recorded the 330dp debug IME bottom of `804px`.
This establishes early selection for that visit,
not uninterrupted visual presentation.

The first rendition also kept `TransportBlock`'s navigation-bar padding
inside the already-reserved band.
At 330dp,
`Shuffle all folders` ended at y `1040` while the reserved app boundary
was about y `1138`,
and the Folders text was clipped to `[112,136][394,164]` with Open absent
from the hierarchy.
The user accepted **minor** browser/Open cropping in the earlier review,
not this loss of the visible Open action throughout ordinary typing.
Prototype commit `5e07bdfbb` omits that additional navigation inset only
inside the pre-reserved debug candidate.
With debug APK SHA-256
`198a92671b912ca296d4ac41fa0f6e98cc4304b460d48e23a60c2724d74cd895`,
its 330dp state retained `Folders` and Open in the shortened upper-left
viewport,
and the final mode ended at y `1118`.
The same APK's in-place 375 to 400dp `screenrecord` sampled 21
IME-visible frames of the pre-reserved candidate;
the five-outline check found its complete final mode in every sampled frame.
A fixed-trigger control on that APK sampled 23 IME-visible frames and
lacked the last mode border in 19 of them.
During initial focus into a 415dp debug IME,
the pre-reserved branch was selected while both reported IME bottoms were
zero;
14 recorded IME-visible frames kept the full final mode.
The same APK's fixed-trigger control missed its bottom border in one of
15 recorded IME-visible frames.
The pre-reserved frames also show title,
metadata,
seek,
transport and Search results while the IME moves.
These counts identify **bounded** passing synthetic runs with positive
failure controls,
not every unrecorded frame or a real-Gboard banner guarantee.

Pre-reserving banner-sized space under the ordinary 330dp keyboard leaves
about 210 physical px unused above that keyboard and keeps the upper-left
browser at its shortened header.
Pressing Android Back hid that keyboard while retaining query focus.
Both Compose and platform bottoms reached zero,
but the focus-only reservation remained active:
`Shuffle all folders` still ended at y `1118` instead of returning to the
keyboard-closed deck near the bottom of the panel.
This conflicts with D51's accepted closed composition.
The user has **not** selected this visible tradeoff;
the active A-only review continues to show the accepted arrangement.
Prototype commit `771028179` then gated that reservation on the focused
editor's first keyboard request or a still-present IME.
With APK SHA-256
`261e608e67029f6361b79a2b26f79851099df9d98f3cdbed637b2e14a4529636`,
Android Back hid the 330dp debug IME while the `cam` editor retained focus.
Once both reported bottoms reached zero,
`Shuffle all folders` returned to `[73,1904][965,2035]`,
and the unchanged folder browser expanded.
Tapping the still-focused editor caused ordinary inset delivery of `804px`
before the show animation's first progress log;
the compact branch was true when Compose still reported bottom zero.
The settled final mode again ended at y `1118`.
A separate refocus `screenrecord` sampled 23 frames after this Back path.
The editor was still focused when tapped;
parent inset delivery reported the 415dp IME target before its first
animation progress event.
All 16 sampled IME-visible frames retained five complete mode boundaries,
with title,
seek,
transport and results visible in the inspected frames.
A same-APK fixed-trigger refocus control sampled 22 frames,
14 with the IME visible.
Frames 011 and 012 lacked the final mode's bottom outline;
the geometric check also rejected frame 010 because its margin to the
keyboard was smaller than the check's 8-video-pixel floor.
The pre-reserved run had no rejected visible-IME frame.
This is a bounded synthetic comparison,
not proof of every unrecorded instant or the cause of a frame-count
difference.
The reserved candidate remains unaccepted and does not address floating keys
or a real Gboard banner.

## Font-scale retest of the selected vertical deck

The later debug-only APK SHA-256
`0801ea3f44ef84163cbddb2b2f37fbddbb1c293148440381f6f899be7eaee7af`
kept the selected vertical deck unchanged while adding separate
Search-result stress and cover-viewport comparisons.
On the disposable inner panel at 390dpi and **100% text**,
a system-managed stepped debug IME reached 400dp (top y `1177`) and
415dp (top y `1141`).
At 400dp,
the unchanged complete final mode had accessibility bounds
`[518,1040][965,1157]`,
ending 20px above the keyboard;
Folders and Open remained visible.
At 415dp,
the final mode ended at y `1121`,
also 20px above its keyboard,
with the same browser header.
Native screenshots confirmed the complete four-mode group at the 400dp
endpoint.
This is a **settled endpoint** pass at 100%,
not continuous-animation or TalkBack evidence.

The **same APK at 200% text** selected the identical A branch.
After the same debug IME stepped from 330dp to 400dp,
Window Manager reported the input window starting at y `1177`.
The final `Shuffle all folders` box was only
`[73,1057][965,1177]`,
against its 131px full row at the lower keyboard height.
The native screenshot showed the mode group's rounded bottom cut by the
dark keyboard edge;
Folders and Open had disappeared from the accessibility hierarchy.
This reproduces a non-exempt **200% endpoint failure** while the same
keyboard height fits at 100%.
D54's accepted brief real-Gboard font-update banner does not excuse this
separate synthetic steady state.
The opt-in cover result viewport does not alter either inner result.

## Early inline layout without advance clearance

Prototype commit `f674da786` adds a debug-only `-earlyinline-` branch.
It keeps the accepted vertical Search layout at 100% text and requests
the compact inline deck at 200%
**before** a focused keyboard first becomes visible.
Unlike the older `-preclear-` study,
it uses ordinary layout-time `imePadding()` rather than a fixed 416dp
bottom reservation.
The tested APK SHA-256 was
`b9d7a7cec4dcea956b57cd694e5c964cf4d70ef1c96a4a0b92518501145556b0`.
It was installed only on the disposable AVD after its earlier debug
certificate was found to differ.
The same artifact still provided the unmodified selected vertical control
and the pre-reserved comparison.

At 200% text,
the stepped debug IME moved in place from 375dp (top y `1238`) to
400dp (top y `1177`) without losing the `cam` editor's focus.
A half-resolution native video of the **early inline** candidate contained
22 sampled IME-visible frames.
Its first four frames showed all five mode-group outlines at video keyboard
y `619`.
Frames 005 to 007 showed the keyboard at y `588` and only four outlines:
the complete final mode's bottom border was behind the keyboard.
Frame 008 showed five outlines again after the layout caught up.
The actual frame 005 shows Folders,
Open,
query,
results and the deck's text;
the mode-group bottom edge is visibly cut at the keyboard boundary.
That is a D50 counterexample during a **synthetic in-place height change**,
even though the compact deck was already selected at 375dp.
It does not establish the reason layout lost those sampled frames.

On the **same APK**,
the advance-reserved `-preclear-` candidate showed all five outlines
in each of its 24 IME-visible sampled frames across the same 375dp to
400dp step.
Its last border stayed at video y `558` while the keyboard edge moved
from y `619` to y `588`.
The earlier fixed-height study measured the reservation's unused space
above a shorter 330dp keyboard at about 210 physical px;
this same-APK run confirms the clearance tradeoff at 375dp,
but does not retest every intermediate instant or real Gboard.
The two recordings sampled different frame counts,
so their counts are categorical observed pass/fail evidence,
not a timing comparison.
After Android Back hid the pre-reserved debug IME,
the focused `cam` editor remained and the full closed deck returned to
`[73,1904][965,2035]` with Folders and Open visible.
At **100% text** under the 330dp debug keyboard (top y `1348`),
the selected vertical and the new early-inline-gated candidates kept their
last mode at y `1328` with Folders/Open above it.
The pre-reserved candidate instead ended its complete last mode at y `1118`,
leaving 230px between the mode and the keyboard and shortening the same
upper-left browser so only its first rows showed.
A native screenshot showed this blank strip under the deck;
no substitute browser or production layout was introduced.
At 100%,
the selected vertical deck also remained complete at the 400dp and
415dp settled endpoints on this same device,
so reserving 416dp under a 330dp keyboard at that text scale was not
required by the tested static fit.
The early-inline candidate deliberately retained the selected vertical
100% branch at 330dp;
its **200%** sampled-frame failure remains decisive for that alternative.

TalkBack was enabled on the disposable AVD for a bounded 100% pre-reserved
probe.
A native screenshot showed its green focus rectangle on the query while
the debug keyboard was present;
a later synthetic tap showed focus on the debug IME's `Type cam` button.
The captured UI hierarchy exposed Folders,
Open and the four mode descriptions,
but synthetic swipe/Tab attempts did not establish that TalkBack could
traverse and activate each mode during keyboard-open Search.
After those inputs,
a later hierarchy showed the keyboard closed and the full closed deck.
The disposable guest's original accessibility settings were restored.
At 200% text and the 400dp debug IME (top y `1177`),
a pre-reserved accessibility hierarchy exposed Folders,
Open,
query and all four mode descriptions;
`Shuffle all folders` occupied its full `[73,987][965,1118]` node box.
After a first-run notification-permission dialog was dismissed on the
disposable guest,
a native screenshot showed TalkBack's focus rectangle on the query with the
400dp debug keyboard visibly present.
That is evidence of TalkBack being active in this composition,
**not** a completed mode-by-mode focus,
speech or activation sequence.
The dialog initially interrupted the probe,
and synthetic gestures did not establish traversal.
The guest's prior accessibility settings and notification permission were
restored after this bounded check.
Node presence is not a screen-reader interaction test;
full 100%/200% Search traversal remains open for the accessibility review.

The native videos,
raw frame directories,
logs and hierarchies remain private.
Neither inline reflow nor the 416dp reservation is accepted.

## Scale-scoped reservation comparison and stop point

Prototype commit `3541587ec` added an **unaccepted** `-scalereserve-` comparison to the debug-only Search study.
Its debug branch reserves 416dp when `fontScale >= 2f`;
the captured 200% text scale takes that branch,
while captured 100% keeps the selected vertical deck and its ordinary IME inset.
The installed debug APK matched the built SHA-256
`ecc1ed1a68331564e6925a0d0d65d2be826af5143f4decd0f272a98bb01ee0bb`.
This fixture does not change production Search or the selected A review.

At 100% text with the stepped debug keyboard at 330dp (top y `1348`),
the complete vertical mode control ended at y `1328` with Folders and Open visible,
rather than leaving the old pre-reservation's 230px blank band.
At the settled 400dp and 415dp endpoints,
the last mode ended at y `1157` and y `1121`.
Each was 20px above its keyboard edge;
Folders and Open remained visible.
The same upper-left folder browser was shortened at 415dp,
not replaced by a caption.
At 200% text under 330dp,
the reserved inline deck ended at y `1118` while the keyboard began at y `1348`,
retaining the 230px clearance **at that scale**.
A native image labeled 400dp showed a keyboard top at approximately y `1177`
and the complete final mode ending at y `1118`;
Folders,
Open,
query and both short results were visible in that image.
However,
the companion 200% hierarchy dumps at 330dp and purported 400dp reported identical app bounds;
those dumps alone do not independently establish a distinct **settled** 400dp endpoint.
After Back hid the keyboard at 200%,
the last mode returned to `[73,1904][965,2035]` and the upper-left browser expanded.

The 200% in-place 375dp to 400dp recording kept the keyboard visible in 23 sampled frames;
an existing five-outline image check found the complete mode in all 23,
with its last border at video y `558` while the keyboard edge moved from y `619` to y `588`.
An inspected frame shows the actual painted border,
not only accessibility node bounds.
On the earlier early-inline APK,
that variant lost the border in 3 of 22 sampled IME-visible frames;
its same-APK pre-reserved control retained all borders in 24 of 24.
The later scale-scoped APK is a separate recording.
Detected outlines do not establish mode readability or activation.
These categorical observations do not measure clip duration,
every unrecorded frame,
or any real Gboard transition.
The 416dp reservation remains a measured synthetic envelope,
not a maximum keyboard height.
The comparison removes an unnecessary **100%** clearance cost in these states
but retains an upper-left space cost at **200%**.
No inline or reserved variant is selected.
The user requested an end to further IME experiments;
the design tradeoff is recorded for a later decision without widening D53 or D54.

## Remaining boundary

The closed deck's first measured heights changed from `762` to `891` to
`1149px` as Compose settled in one visit.
A later focused transition logged a stored `1071px` height,
but this capture did not isolate when that earlier sample was taken.
No single early `onSizeChanged` value proves full-content demand.
For a distinct non-exempt keyboard that can grow in place,
a D50-compliant design would need to reserve deck space **before** its
height increase or use a supported integration point that synchronizes
layout with IME presentation;
the parent callback experiment did not prevent the first clipped draw.
D54 does not require this synthetic remedy for the accepted brief real
font-update banner.
Do not treat a settled synthetic fit as real-banner verification.
Keyboard dismissal,
refocus,
and cold entry at a height where both deck arrangements fit remain open.
Check title paint and accessibility bounds separately;
acceptance of a cropped **browser** does not allow a cropped deck.
D53 and D54 accept those specific observed real-Gboard overlays without
selecting a synthetic reflow or reservation.
The custom height-step IME,
other untested keyboard geometries and accessibility bounds remain separate
evidence needs before claiming broader D50 compliance.
A Gboard banner recurrence was not established,
but D54 does not require one to accept the already measured brief state.
