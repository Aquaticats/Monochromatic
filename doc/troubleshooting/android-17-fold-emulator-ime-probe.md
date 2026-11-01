# Android 17 Fold Gboard 18.2.7 opens a physical-keyboard toolbar instead of keys on Search focus

## Symptom

On the original Pixel 9 Pro Fold emulator 37.1.11.0, Android 17 SDK 37,
the **active updated Gboard** `versionCode=175981944` can report an active
IME on Compose Search focus while the screenshot contains no on-screen keys.
The earlier `versionCode=175753756` reading named the preloaded system APK,
not the active updated package; `dumpsys package` shows the active code path
under `/data/app/`, with `lastUpdateTime=2026-09-24 12:32:53`.
Do not attribute the later floating-keyboard observations to the preload.
A later Google Messages Search probe also showed no keys.
Earlier in this design session, Messages did show a split keyboard,
so this is not a demonstrated permanent emulator capability limit.
The focused query and an `input_method` visibility report alone cannot prove
that the playback deck remains visible during software-keyboard use.
A subsequent probe found a **per-editor Gboard workaround**:
open the side toolbar's bottom menu and choose `Show on-screen keyboard`.
Real, floating keys then appeared and a key tap changed the Search query.
A separate disposable Fold subsequently established bounded split and
full-width Gboard behavior;
it does not erase the measured floating overlap.
Design decisions D53 and D54 accept that unfolded floating-deck overlap and
the distinct brief font-update banner,
respectively.
D55 separately accepts the measured floating cover keyboard obscuring both
matching labels while `cam` remains visible.
None approves obscuring the query or clipping the deck beneath ordinary
settled split or docked keyboards.

This is distinct from a separate, confirmed app-layout finding:
under a visible system-managed keyboard, a bottom-anchored Search prototype
covered the lower portion of its playback deck at 200% text.
Do not attribute that layout failure to Gboard.

## Root cause and limits of the diagnosis

The reason Gboard did not draw its keys in these later probes is **unresolved**.
Gboard's implementation is not available in the local Android SDK sources;
we cannot identify which Gboard decision or system state produced the discrepancy.
Changing `show_ime_with_hard_keyboard`, restarting Gboard, recreating Search,
and reproducing in Messages did not establish a cause.
Gboard's `Physical keyboard` preferences were also tested:
`Show on-screen keyboard` changed from off to on, and `Show toolbar` changed
from on to off after temporarily turning the former off.
Neither setting combination, nor a Gboard restart, made keys appear on
fresh Search focus.
The separate **toolbar-menu action** did make keys appear, but the next
fresh-focus probe returned to the side toolbar.
Repeating that action after restoring the original off/on preference switches
again produced the floating keys, so the preference changes were unnecessary
for this workaround.
Do not equate the preference switch with the per-editor action.
Google's [Pixel Tablet keyboard guidance][pixel-tablet-keyboard]
describes a Gboard toolbar with a physical keyboard and a Floating keyboard
mode; it does not establish which layout this Fold emulator should choose.

[pixel-tablet-keyboard]: https://support.google.com/googlepixeltablet/answer/13555948?hl=en

Android's public framework does establish why a replacement IME is a valid
occlusion probe.
The local SDK source,
`/home/user/Android/Sdk/sources/android-37.0/android/inputmethodservice/InputMethodService.java:2389-2400`,
checks whether an IME should show its input view:

```java
if (mSettingsObserver.shouldShowImeWithHardKeyboard()) {
    return true;
}
Configuration config = getResources().getConfiguration();
return config.keyboard == Configuration.KEYBOARD_NOKEYS
        || config.hardKeyboardHidden == Configuration.HARDKEYBOARDHIDDEN_YES;
```

This is the framework's *default* decision, not a source trace of Gboard.
At `InputMethodService.java:2347-2357`, the framework shows or hides the
input frame according to the IME's decision and creates its view:

```java
boolean isShown = mShowInputRequested && onEvaluateInputViewShown();
mInputFrame.setVisibility(isShown ? View.VISIBLE : View.GONE);
if (mInputView == null) {
    View v = onCreateInputView();
    if (v != null) {
        setInputView(v);
    }
}
```

The view supplied by the debug IME needs its own measured minimum height.
`InputMethodService.java:2521-2524` supplies a fresh wrap-content parent layout:

```java
public void setInputView(View view) {
    mInputFrame.removeAllViews();
    mInputFrame.addView(view, new FrameLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT));
    mInputView = view;
}
```

Setting only the returned view's parent layout parameters did **not** give
our first probe the requested 300dp height; the rendered window was shorter.
Setting `minimumHeight` on the returned view made the dark input surface fill
approximately 300dp on this 390dpi AVD.
That correction is in throwaway prototype commit `60e01dfab`.

The floating Gboard also explains why the selected deck did not lift in that
probe.
`adb -s emulator-5554 shell dumpsys window` reported a visible IME
`InsetsSource` frame `[0,2152][2076,2152]` and an IME bottom-inset hint
of `0`, despite its separately reported keyboard touchable region.
`/home/user/Android/Sdk/sources/android-37.0/android/view/InsetsSource.java:441-457`
returns no inset without an intersection, and otherwise maps IME intersection
height to a bottom inset:

```java
final boolean hasIntersection = relativeFrame.isEmpty()
        ? getIntersection(frame, relativeFrame, mTmpFrame)
        : mTmpFrame.setIntersect(frame, relativeFrame);
if (!hasIntersection) {
    return Insets.NONE;
}
if (getType() == WindowInsets.Type.ime()) {
    return Insets.of(0, 0, 0, mTmpFrame.height());
}
```

The zero-height server source and the unlifted deck agree for this device
state.

Android 17 also exposes a flagged `WindowInsets.getBoundingRects(typeMask)`
API for obscuring regions (`WindowInsets.java:525-553`).
`InsetsState.java:383-391` computes that public map from each source:

```java
final Insets insets = source.calculateInsets(relativeFrame, hostBounds, ignoreVisibility);
final Rect[] boundingRects = source.calculateBoundingRects(relativeFrame, hostBounds,
        ignoreVisibility);
processSourceAsPublicType(source, typeInsetsMap, idSideMap, typeVisibilityMap,
        typeBoundingRectsMap, insets, boundingRects, type);
```

The observed IME source had a zero-height frame and no `boundingRects` or
`insetsBoundingRects` entries in `dumpsys window`.
`InsetsSource.java:595-608` returns no rectangle when those arrays are absent
and the frame does not intersect the app window:

```java
if (mBoundingRects == null && mInsetsBoundingRects == null) {
    return mTmpFrame2.setIntersect(mTmpFrame, relativeFrame)
            ? new Rect[]{
                    new Rect(
                            mTmpFrame2.left - relativeFrame.left,
                            mTmpFrame2.top - relativeFrame.top,
                            mTmpFrame2.right - relativeFrame.left,
                            mTmpFrame2.bottom - relativeFrame.top
                    )
            }
            : EMPTY_RECTS;
}
```

The empty branch is the relevant one for the measured source.
`InsetsSource.java:839-848` prints either array when present, which is why
its absence in the server dump matters.
Thus this **observed IME source** offers neither a bottom inset nor a
floating-keyboard bounding rectangle through these framework paths.

A separate real-Gboard check on the disposable `emulator-5580` reproduced
this geometry with the **retained-browser Search prototype** at 200% text.
Google's [Pixel Tablet keyboard instructions][pixel-tablet-keyboard]
identify the keyboard-over-line toolbar icon as **Floating keyboard**.
Tapping that icon in the disposable Gboard toolbar switched its visible
full-width keys to a floating key surface over Search.
The app's `SearchInsetProbe` then logged `visible=true`,
`platformBottom=0`,
`composeBottom=0`,
and `boundingRects=[]`.
No new IME animation callback appeared in the captured app log for this toggle.
Privileged `dumpsys window windows`,
which is **not an app API**,
reported the Gboard window's touchable region at `[936,1147][1842,1918]`
plus the navigation region.
UI Automator placed the deck's `4:35` duration at
`[842,1263][965,1341]`,
intersecting the floating keys in x `[936,965)`.
The resulting screenshot visibly obscured the duration's ending and a strip
of the playback-mode container.
This was a real-Gboard counterexample to D50's unqualified deck-visibility
rule on the disposable AVD,
not a synthetic height-step result;
D53 now accepts this specific floating-keyboard overlap.
The exact launch variant for that first toggle was not recorded;
the captured public inset data must not be attributed to a specific
accepted-layout intent extra.

A second check explicitly launched
`search-deck-right-lift-retain-results-light`,
the corrected **vertical-deck A layout** without the experimental tall-IME
reflow or anticipatory reservation.
The installed debug APK and local build both had SHA-256
`261e608e67029f6361b79a2b26f79851099df9d98f3cdbed637b2e14a4529636`.
After the AVD rebooted and its sleeping screen was woken,
focusing the editor restored real floating Gboard.
Privileged Window Manager reported a key touch region
`[482,1006][1388,1777]`;
UI Automator placed the deck title at `[258,1120][781,1223]`.
Their intersection is x `[482,781)` and y `[1120,1223]`,
and the screenshot confirms the title,
seek display and transport were obscured.
A real Gboard key tap changed the focused query from `cam` to `cadm`;
its cursor had been between `a` and `m`.
The app log for this specific baseline launch did not retain a noninitial
public-insets sample,
so its floating rectangle and zero inset are **not** independently
app-logged for this exact variant.
The first corrected-prototype public probe and the second exact-variant
occlusion are separate evidence layers.
Both screenshots and logs remain unsanitized private scratch evidence,
not published review assets.

`WindowInsets.java:413-430` describes `isVisible(Type.ime())` as
independent of overlap with the app window.
Its implementation checks a boolean map:

```java
public boolean isVisible(@InsetsType int typeMask) {
    for (@InsetsType int type : TYPES) {
        if ((typeMask & type) == 0) {
            continue;
        }
        if (!mTypeVisibilityMap[indexOf(type)]) {
            return false;
        }
    }
    return true;
}
```

`InsetsState.java:435-437` sets that map from source visibility:

```java
if (typeVisibilityMap != null) {
    typeVisibilityMap[index] = source.isVisible();
}
```

The original Gboard toolbar-only and floating-key states both exposed a
visible IME source with no usable bottom inset.
Changing `SearchDeckRight`'s `keyboardShown` test from
`WindowInsets.ime.getBottom(...) > 0` to a visibility flag could detect
that an IME window exists,
but cannot distinguish a toolbar from real keys or locate floating keys.
`WindowInsetsAnimation.java:187-201` stores lower and upper **Insets**
bounds for an animation,
not a floating-window rectangle:

```java
private final Insets mLowerBound;
private final Insets mUpperBound;
```

`InputMethodManager.java:5025-5041` preserves
`getInputMethodWindowVisibleHeight()` only as a hidden compatibility API
and calls its result not well-defined:

```java
/**
 * This is kept due to {@link android.compat.annotation.UnsupportedAppUsage}.
 *
 * <p>TODO(Bug 113914148): Check if we can remove this.  We have accidentally exposed
 * WindowManagerInternal#getInputMethodWindowVisibleHeight to app developers and some of them
 * started relying on it.</p>
 *
 * @return Something that is not well-defined.
 * @hide
 */
@UnsupportedAppUsage(trackingBug = 204906124, maxTargetSdk = Build.VERSION_CODES.TIRAMISU,
        publicAlternatives = "Use {@link android.view.WindowInsets} instead")
public int getInputMethodWindowVisibleHeight() {
    return IInputMethodManagerGlobalInvoker.getInputMethodWindowVisibleHeight(mClient);
}
```

It does not provide a supported floating rectangle for this app,
whose `package/music-player/android-app/app/build.gradle.kts:25-26`
sets `minSdk=26` and `targetSdk=36`.
`WindowManager.java:2879-2885` describes a nonfocusable window that can
layer above the IME and cover it:

```java
* gets Z-ordered on top of the input method, so it can use the full
* screen for its content and cover the input method if needed.  You
```

A debug-only test of that layer exists on prototype commit `d9a5c549e`.
The tested APK SHA-256 was
`e4bfec8e8eb98187a0afc06ef11d587fd3c87623fe45a37da7b0ffd862c6461e`.
The same-APK vertical-A control
`search-deck-right-lift-retain-results-light` showed real floating Gboard
obscuring the deck title at x `[482,781)` and y `[1120,1223]`.
The experimental
`search-deck-right-lift-layerprobe-retain-results-light` added a
**nonfocusable and not-touchable app panel** over the left pane.
Window Manager reported that panel at `[0,717][1038,1793]` with
`NOT_FOCUSABLE NOT_TOUCHABLE LAYOUT_IN_SCREEN` and the real floating
Gboard touch region at `[482,1006][1388,1777]`.
The screenshot verified the panel paints **above** the keys,
but it also washed out both keyboard keys and app content.
It was only a marker,
not a duplicate deck or accepted design.

The same SDK's
`android/view/WindowManager.java:2893-2914` documents the relevant
cross-UID touch rule for a `FLAG_NOT_TOUCHABLE` window:

```java
* Starting from Android {@link Build.VERSION_CODES#S}, for security reasons, touch
* events that pass through windows containing this flag (ie. are within the bounds of the
* window) will only be delivered to the touch-consuming window if one (or more) of the
* items below are true:
* <li><b>Same UID</b>: This window belongs to the same UID that owns the touch-consuming
*   window.
* <li><b>Trusted windows</b>: This window is trusted.
* <li><b>Invisible windows</b>: This window is {@link View#GONE} or
*   {@link View#INVISIBLE}.
* <li><b>Fully transparent windows</b>: This window has {@link LayoutParams#alpha}
*   equal to 0.
```

The visible app panel belonged to UID `10249`,
while Gboard belonged to UID `10170`.
A real tap on an **uncovered** floating key entered `m` in the focused
Search query.
A tap on a key **under the panel** left the query at `m`.
`InputDispatcher` reported
`Dropping untrusted touch event due to occlusion by dev.monochromatic.musicplayer/10249`.
That positive and negative pair rejects this panel as a solution:
painting over the IME prevented typing through the covered area.
After Android Back hid Gboard,
the debug marker also remained painted over the browser and deck in the
sampled frame;
the candidate's visibility-based cleanup did not run for that state.
These unsanitized captures are private scratch evidence.
Do not transplant this panel into production or treat it as D50 compliance.

`View.java:13353-13370` separately offers
`setPreferKeepClearRects()` as a **best-effort preference** for floating
windows above an app view;
the source says the system may ignore it when the request cannot be met.
`View.java:13449-13454` forwards changed rectangles to its attached
`ViewRootImpl`,
and `ViewRootImpl.java:6687-6709` reports changed areas to Window Manager.
This path does **not** by itself promise movement of Gboard's internal keys.
The **Android 17 AOSP release branch**,
rather than a source-identical mapping to this Google system image,
provides one concrete downstream consumer trace.
Its
[DisplayContent.java][aosp-display-content] `:6468-6503`
collects keep-clear areas from visible windows,
adds the IME window's touchable region to the **unrestricted** set,
and dispatches changed areas:

```java
getKeepClearAreas(restrictedKeepClearAreas, unrestrictedKeepClearAreas);
mWmService.mDisplayNotificationController.dispatchKeepClearAreasChanged(
        this, restrictedKeepClearAreas, unrestrictedKeepClearAreas);
// Inside getKeepClearAreas, for a visible IME window:
w.getEffectiveTouchableRegion(touchableRegion);
RegionUtils.forEachRect(touchableRegion, rect -> outUnrestricted.add(rect));
```

[DisplayWindowListenerController.java][aosp-display-listener] `:124-133`
forwards those sets to registered display listeners,
and the Shell [DisplayController.java][aosp-shell-display] `:478-490`
forwards the callback to its display-change listeners.
The Android 17 [PipController.java][aosp-pip-controller] `:349-359`
is one such consumer:

```java
public void onKeepClearAreasChanged(int displayId, Set<Rect> restricted,
        Set<Rect> unrestricted) {
    if (mPipDisplayLayoutState.getDisplayId() == displayId) {
        mPipBoundsState.setKeepClearAreas(restricted, unrestricted);
        mMainExecutor.executeDelayed(
                mMovePipInResponseToKeepClearAreasChangeCallback,
                PIP_KEEP_CLEAR_AREAS_DELAY);
    }
}
```

`PipController.java:187-205` calls its PiP keep-clear algorithm and,
when the destination differs,
animates the **PiP task** to new bounds.
These source paths establish a PiP use of the hint,
not a Gboard subscription or a command to move Gboard's internal keys.
No PiP movement was tested on this fixture,
and no complete consumer search establishes that Gboard cannot participate
through another path.

[aosp-display-content]: https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android17-release/services/core/java/com/android/server/wm/DisplayContent.java
[aosp-display-listener]: https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android17-release/services/core/java/com/android/server/wm/DisplayWindowListenerController.java
[aosp-shell-display]: https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android17-release/libs/WindowManager/Shell/src/com/android/wm/shell/common/DisplayController.java
[aosp-pip-controller]: https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android17-release/libs/WindowManager/Shell/src/com/android/wm/shell/pip/phone/PipController.java

A second debug-only candidate,
`search-deck-right-lift-keepclear-retain-results-light`,
used prototype commit `b9c05342f` and APK SHA-256
`e755bf76ed65e45dc4e4ec57f4f55bdbd948902ca6e1940ddf911474ee8a4dd7`.
The app logged a request for `[0,717][1038,2152]`;
Window Manager reported that exact restricted `keepClearAreas` rectangle
on the app window.
Its real floating Gboard still occupied
`[482,1006][1388,1777]`.
The key region **partially overlapped** the requested area in x
`[482,1038)` and y `[1006,1777)`,
and the screenshot showed its keys over the deck.
To validate that Window Manager's touch-region probe could show movement,
a deliberate drag moved Gboard to `[1025,1006][1931,1777]`,
which still overlapped the requested area by 13px horizontally.
A reverse drag moved it back to `[528,1006][1434,1777]` while the same
keep-clear area remained registered.
Manual movement validates the geometry detector,
**not** keep-clear cooperation or automatic movement.
The screenshot after that reverse drag still showed the deck title obscured.
A tap on a visible floating key entered `d` in the focused query,
so this was real usable Gboard input rather than a toolbar-only state.
This test rejects **this rectangle request on this Gboard fixture** as
an automatic D50 response.
It does not prove every keep-clear placement or other floating window
behaves the same way.
The captures remain private unsanitized scratch evidence.
The rejected panel and measured keep-clear request do not prove that every
app-observable keyboard API lacks geometry or that all overlay approaches fail.

## Verification

- Check the installed emulator and Gboard versions:

  ```sh
  /home/user/Android/Sdk/emulator/emulator -version
  adb -s emulator-5554 shell dumpsys package com.google.android.inputmethod.latin
  ```

- The debug-only implementation is on branch
  `prototype/music-player-theme-compose` at
  `package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/FoldProbeInputMethod.kt`.
  Build and install with
  `ANDROID_SERIAL=emulator-5554 mise run //package/music-player/android-app:prototype:install`
  from that worktree.
  `adb -s emulator-5554 shell ime enable dev.monochromatic.musicplayer/.FoldProbeInputMethod`
  and `adb -s emulator-5554 shell ime set dev.monochromatic.musicplayer/.FoldProbeInputMethod`
  select the probe after launching the activity.
  Selecting the probe **before** `am force-stop dev.monochromatic.musicplayer`
  may revert the selected input method to Gboard because this debug IME
  shares the activity's package.
- With the emulator unfolded at font scale `2.0`, launch
  `dev.monochromatic.musicplayer/.DesignCandidateActivity` with
  `--es candidate search-deck-inner-right-empty-light`, focus the query,
  tap the probe's `Type cam` key, then capture the inner HWC display 0.
  The query becomes `cam`; the result heading, result lines, all playback
  mode labels, transport controls, and the system-managed input pane are
  visible together.
  The mirrored candidate,
  `search-deck-inner-mirrored-empty-light`, also passes this visible-state
  check with Search on the left and the deck on the right.
  The committed
  throwaway matrix task `prototype:capture:choices` captured right,
  mirrored and IME-lifted placements in both schemes at 100% and 200%.
  At 200%, `dumpsys window windows` reported `InputMethod` frame
  `[0,1421][2076,2152]`, or 731px on the 390dpi inner panel.
  The task restored Gboard as the selected method and disabled the probe.
- A separate cover-panel check at 200% inserted `cam` while the same input
  method occupied `[0,1693][1080,2424]`.
  The query and both sample results
  remained visible above its input surface.
  The cover returned to the
  unfolded state and original 100% text setting after this check.
- The previously bottom-anchored right candidate, captured with the same
  initial probe before moving the deck, displayed its lower mode controls
  behind the input window at 200% text.
  Even the first, shorter probe was a positive control for occlusion;
  it was not a valid measurement of a 300dp keyboard.
- With the selected Search study focused on the 2076 × 2152px inner panel,
  the repeatable scratch harness
  `/home/user/temp/agent/probe-fold-gboard.mjs` verified a focused
  `EditText` and reported Gboard touchable regions
  `[19,790,157,1436]` and `[0,2074,2076,2152]`, without a key-sized
  region.
  Opening the side toolbar menu at `(90,1360)` and tapping
  `Show on-screen keyboard` at approximately `(510,1118)` at 100% text
  displayed a real floating Gboard at x `[274,1180)`, y `[288,1059)`.
  A tap on one letter changed the focused query to `x` and produced the
  corresponding no-results state.
  A fresh app launch and focus returned to the toolbar-only state.
  Repeating the toolbar-menu action with Gboard's original off/on physical
  keyboard preferences also produced real floating keys.
- At 200% text, the same menu action at approximately `(515,1080)` displayed
  floating Gboard keys over x `[274,1180)`, y `[310,1081)`.
  The deck title `Another Xronixle` occupied
  `[258,1042][781,1145]` in the UI hierarchy.
  Their overlap is x `[274,781)`, y `[1042,1081)`, and the screenshot
  visibly obscured part of the title.
  Dragging the floating keyboard lower moved its region to approximately
  x `[564,1470)`, y `[1245,2016)`, obscuring more controls;
  this was **not** a successful docking test.
  These observations are a counterexample to assuming the selected layout
  meets D50 under every real Gboard mode.
- The cover-panel fresh-focus probe also showed only the toolbar:
  touchable x `[19,157)`, y `[944,1590)` plus the navigation area.
  Its toolbar-menu action exposed real floating keys at approximately
  x `[0,830)`, y `[304,1025)`.
  Tapping the real keys entered `cam` in a focused cover Search field.
  The UI hierarchy placed the matching folder label `Camellia` at
  `[127,374][286,433]` and track title `Another Xronixle` at
  `[127,569][432,628]`, both fully under Gboard's key surface.
  The captured Search screenshot showed the query but no visible result
  data in that observed floating-keyboard position.
  D55 accepts this separate cover result overlap,
  not the absence of actual result data or an obscured query.

### Disposable keyboard-free Fold attempt

A separate Pixel 9 Pro Fold API 37 AVD was created under
`/home/user/temp/agent/fold-no-hardware-avd/` without copying the active
user AVD.
Its generated `config.ini` says `hw.keyboard=no`,
`hw.ramSize=2G`, and `hw.lcd.density=390`.
The Android SDK and this private AVD were mounted respectively read-only and
writable in a rootless Fedora 44 Podman container capped at 2 GiB RAM and
2 CPUs.
The emulator reported `Increasing RAM size to 4096MB` at startup.
This establishes the effective requested guest size for **this invocation**,
not a universal minimum across Fold system images.

The first `-gpu host -no-window` launch inside the container had no X display;
Android Emulator reported `Failed to get EGL display` and
`Could not start renderer! (Error: -2)`.
A second attempt used a container-local Xvfb display while retaining
`-gpu host` and the same memory/CPU limits.
Its renderer proceeded further,
but the process exited with status 137 before Android booted.
The retained kernel journal records the deciding emitter and memory boundary:

```text
Sep 24 20:11:18 bazzite kernel: Memory cgroup out of memory: Killed process 2055528 (qemu-system-x86) total-vm:8001484kB, anon-rss:2062252kB, file-rss:53928kB, shmem-rss:0kB, UID:1000 pgtables:9928kB oom_score_adj:200
```

`journalctl -k --since '2026-09-24 20:08:00' --until
'2026-09-24 20:12:00'` reproduces the diagnostic while that journal is
retained.
This is a **container-limit failure**, not evidence that a keyboard-free
Fold cannot display Gboard or that the app passes D50.
The `mvm` alternative was checked without creating a VM:
`mvm list` failed with `spawn virsh ENOENT`;
`package/mvm/cli/README.md` names `virsh`, `qemu-img` and libvirt as
prerequisites, and none of those binaries/libraries was present locally.
No host packages were installed.

After the user authorized a larger cap,
a 6 GiB/2 CPU retry first stopped at Android Emulator's
`Running multiple emulators with the same AVD` fatal diagnostic.
No container or process held the private AVD's lock files;
removing only those stale scratch locks allowed another retry to launch.
That retry did not become usable:
`adb devices -l` reported `emulator-5580 unauthorized`,
while the emulator's own log repeatedly reported
`adb: device unauthorized` and `No adb private key exists`.
After the user noted the wait,
Podman measured `5.496GB / 6.442GB` for the container.
I killed that disposable container;
its exit status 137 was the result of this **intentional stop**,
not evidence of a second cgroup out-of-memory event.
No Gboard or deck measurement was obtained.

The installed Android Emulator 37.1.11 help lists `-skip-adb-auth`.
The [Google emulator container launcher][emulator-container-launcher] at commit
`0654f694b46794fae4b178f1e1a17cb60c5d2d34`
uses it in `emu/templates/launch-emulator.sh:177-180`:

```sh
LAUNCH_CMD+=("-skip-adb-auth" "-no-snapshot-save" "-wipe-data" "-no-boot-anim")
```

A 6 GiB retry did pass this flag,
confirmed in the disposable AVD's `emu-launch-params.txt`,
but `adb devices -l` still reported `emulator-5580 unauthorized`.
Android Emulator continued to emit `No adb private key exists` and
`adb: device unauthorized`.
The container was intentionally stopped before the bounded monitor's
five-minute deadline;
this attempt did not establish whether Android completed startup.
The flag alone is **not** an observed authorization workaround on this fixture.

The same launcher at `emu/templates/launch-emulator.sh:67-69`
has a public-key input path:

```sh
elif [ ! -z "${ADBKEY_PUB}" ]; then
  echo "emulator: Using provided adb public key"
  echo $ADBKEY_PUB >>/root/.android/adbkey.pub
```

A further private-AVD retry mounted **only the host ADB public key**
read-only at that path,
passed `-skip-adb-auth`,
and wiped interrupted disposable guest data.
The bounded monitor reported `ADB_BOOT_READY emulator-5580` after 122 seconds;
`getprop ro.boot.qemu.avd_name` returned `Fold_No_Hardware_Probe`.
No private ADB key was copied and the shared host ADB server was not restarted.
The combination worked;
this run did **not** isolate which step corrected authorization.

The fresh AVD's generated configuration said `hw.keyboard=no`,
yet `dumpsys input` still listed an enabled `AT Translated Set 2 keyboard`,
and Window Manager reported `qwerty/v/v`.
Do not call this fixture hardware-keyboard-free or infer that removing
hardware keyboard alone changed Gboard's layout.
The installed debug app APK on both AVDs had the same SHA-256
`d895072b4f232181c1f24d9db0bfd6b3af7cafae2e25f869cbe21b0039e81f53`.
The original AVD's active Gboard was version
`18.2.7.969776716-release-x86_64` (`versionCode=175981944`),
while the fresh AVD initially used its preloaded
`17.2.2.895242737-preload-x86_64` (`versionCode=175753756`).
I pulled only the updated Gboard APKs from the original AVD,
installed them into the disposable AVD,
and verified its active `/data/app/` package became the same updated version.
The disposable AVD still showed split inner and full-width cover keys,
so **Gboard version alone did not reproduce the original floating mode**.
User data,
physical-keyboard configuration and other AVD settings still differ.

### Actual Gboard typing and visibility

On the disposable inner panel,
actual taps on Gboard's split keys entered `cam` into the focused Search field.
At settled 100% and 200% font scales,
the system IME source occupied `[0,1352][2076,2152]`.
At 200%,
all four unfolded mode options were visible;
`Shuffle all folders` occupied `[73,1182][965,1313]`,
ending 39 physical px before the keyboard.
The folder and track result labels also remained visible beside the deck.
At 200%,
light and dark captures showed this bounded arrangement,
including after installing the updated Gboard.

On the disposable cover panel,
actual Gboard key taps again entered `cam` at 100% and 200% font scales.
The full-width IME source occupied `[0,1605][1080,2424]`.
At 200%,
folder `Camellia` occupied `[127,350][410,453]` and track
`Another Xronixle` occupied `[127,584][667,687]`,
well above that keyboard.
Light and dark 200% captures agreed;
updating Gboard did not change this measured layout.
Android Back hid the keyboard while retaining `cam` results;
tapping the query reopened it,
and Clear followed by real-key retyping restored the results.

A separate **transient counterexample** appeared immediately after changing
Android's font scale to 200% while Search was focused.
Gboard showed a `Keyboard font size updated` banner with an `OK` action;
the IME source temporarily began at y `1140`.
The final mode's accessibility bounds were only
`[73,1076][965,1140]`,
with part of its label and container visibly covered.
Tapping `OK` removed the banner,
returned the IME source to y `1352`,
and restored the complete mode at `[73,1182][965,1313]`.
This measured temporary clip does **not** become a visibly complete deck
because the settled screenshot passes D50.
D54 explicitly accepts this brief Gboard banner overlap as an exception;
automatic dismissal and recurrence were not established.

A later **bounded recurrence attempt** on the disposable AVD used the
corrected `search-deck-right-lift-retain-results-light` debug candidate
with APK SHA-256
`261e608e67029f6361b79a2b26f79851099df9d98f3cdbed637b2e14a4529636`.
With floating Gboard visible,
`settings put system font_scale 1.0` recreated Search and hid the keys;
changing the font scale back to `2.0` recreated it again.
After a settled refocus,
Gboard displayed **docked split keys** with touchable region
`[0,1352][2076,2152]` and no font-update banner.
UI Automator placed the complete final mode at `[0,1201][1038,1332]`.
This sequence differs from the first banner's focused scale change.
It neither reproduces the banner nor proves it cannot recur while typing;
its unsanitized screenshot remains private scratch evidence.

The debug input method is deliberately synthetic, **not Gboard**.
Its measured overlap tests bottom-window occlusion and text input integration;
it does not establish Gboard's own geometry.
The real-keyboard evidence now covers the specific settled split/full-width
arrangements and the original floating failures,
not every keyboard or user configuration.
The [Gboard geometry evidence](../../package/music-player/design/evidence/gboard-geometry.md)
indexes sanitized physical-panel PNGs and whitelisted app-node JSON;
raw screenshots and full hierarchies stayed in private agent scratch.
The committed evidence validator rejected altered deck,
banner,
floating-result and status-mask fixtures,
then passed their restored originals.

## Verified workaround

Use the debug-only `FoldProbeInputMethod`, which overrides
`onEvaluateInputViewShown()` to return `true`, gives its view a measured
300dp minimum height, and commits the sample query via
`currentInputConnection.commitText("cam", 1)`.
Capture after verifying the query has changed and the input view is rendered.
The probe uses a generic dark slab and one sample key; it cannot validate
Gboard-specific spacing or user-facing keyboard behavior.

The Gboard side-toolbar menu's `Show on-screen keyboard` action exposes
real keys for the focused editor without switching to the debug IME.
It is a temporary, floating-keyboard action, not a persistent default
or a docked-keyboard geometry test.
At 200% text,
it covers part of the selected Search deck.
D53 permits that measured floating overlap;
this action does not prove any docked-keyboard layout passes D50.

For repeatable geometry research without changing the active AVD,
the disposable Fold can be booted in a 6 GiB/2 CPU container with Xvfb,
a read-only copy of the host ADB **public** key,
`-skip-adb-auth`,
and clean disposable guest data.
This verified combination enables real Gboard input,
but its resource cost is higher than the earlier 2 GiB probe,
and it does not make the original AVD's floating mode go away.

For bottom-keyboard design comparisons, anchor the unfolded deck above a
visible keyboard rather than treating a keyboard-closed capture as D50 evidence.
Do not change the production app based solely on either probe.

## What does not work

- Query focus, a true IME visibility flag, or an XML node alone:
  none proves that an actual keyboard surface occludes app content.
- Toggling `show_ime_with_hard_keyboard` between `0` and `1`, changing
  Gboard's physical-keyboard preference switches, restarting Gboard,
  or trying Messages again: these probes did not restore keys
  reliably on a fresh Search focus.
- Dragging floating Gboard toward the bottom did not dock it in this probe;
  the overlay moved over the deck without generating a bottom keyboard inset.
- Assigning `layoutParams` with a 300dp height to the debug IME's returned
  view: the framework supplies wrap-content parent parameters;
  the first probe appeared shorter than its label.
- The old `capture-search-deck.mjs` screenshots: they were taken before
  the deck-first revisions and have no visible keyboard.
- The 2 GiB capped disposable AVD run: the container-local Xvfb allowed
  graphics initialization, but the kernel killed `qemu-system-x86` for
  container memory exhaustion before a keyboard could be tested.
- Passing `-skip-adb-auth` alone in the 6 GiB container still left the fresh
  guest `unauthorized` to the host ADB server.
- Treating the generated `hw.keyboard=no` config as absence of a physical
  keyboard: `dumpsys input` and Window Manager contradict that assumption.
- Treating the banner-state `bottom <= imeTop` as a deck-visibility proof:
  the final mode was clipped to 64px while ending exactly at the IME top.

## Upstream filing decision

The `.out-of-scope/` entries do not exempt this Android symptom.
A `gh search issues` query for
`Android emulator Gboard physical keyboard not showing soft keyboard`
returned no matching GitHub issues, but GitHub is not the Gboard issue tracker.
No upstream report is ready:

1. **Upstream fault:** Unknown. A disposable AVD running the same updated
   Gboard build drew split/full-width keys; the original differs in user data
   and settings, so the floating mode is not an isolated regression.
2. **Fixability:** Unknown without the responsible path.
3. **Supported use case:** Android documents custom IMEs and soft-keyboard use,
   but that does not identify a Gboard regression.
4. **Contribution policy:** Gboard is not an identified public source repository
   with reviewed contribution or AI-assistance policy for this path.
5. **Expected upstream response:** No comparable maintainer signal was established.
6. **Tested upstream fix:** None. The custom IME is a consumer-side measurement
   workaround, not a patch for Gboard.

### Draft, do not file as-is

~~~md
Android 17 Fold emulator: Gboard's physical-keyboard toolbar lacks keys on fresh Search focus

On emulator 37.1.11.0, Pixel 9 Pro Fold, Android 17 SDK 37,
Active Gboard versionCode 175981944, focusing the music-player debug Search
shows a side toolbar without keys.
The toolbar's Show on-screen keyboard action brings up a usable floating
keyboard, but another fresh focus returns to the toolbar-only state.
At 200% text the floating keyboard obscures the selected player's deck title.
A disposable Fold with the same updated Gboard drew split inner and
full-width cover keyboards,
so version alone did not reproduce the original state.
No upstream attribution or fix is proposed.
~~~

The [Android IME guide](https://developer.android.com/develop/ui/views/touch-and-input/creating-input-method)
explains the supported input-method extension point used for the probe.

[emulator-container-launcher]: https://github.com/google/android-emulator-container-scripts/blob/master/emu/templates/launch-emulator.sh
