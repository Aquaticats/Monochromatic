# Android Emulator 37.1.11 guest input misses TalkBack gestures while gRPC touch and speech overlay expose announcements

## Symptom

The Pixel 9 Pro Fold emulator needed verifiable TalkBack evidence for a Compose design.
Three ordinary automation surfaces were insufficient:

- `adb shell input touchscreen swipe ...` did not reliably advance TalkBack accessibility
  focus.
- `uiautomator dump` exposed Compose names and roles but not TalkBack's spoken sentence.
- `adb logcat` showed Google TTS synthesis requests but not the synthesized text.

Hardware Tab and arrow events could move keyboard focus,
 but TalkBack's `Display speech
output` overlay remained on its preceding utterance.
 Those probes could not prove the
screen-reader announcement or traversal order.

The working composition is:

1.  Start Android Emulator 37.1.11 with its loopback gRPC controller.
2.  Enable installed TalkBack 17.0.0.889642762 and its `Display speech output` setting.
3.  Inject physical swipe gestures through `EmulatorController.sendTouch`.
4.  Capture the opaque panel while TalkBack shows both its focus rectangle and spoken text.

## Root cause

The failed probes addressed different evidence layers.
 UI Automator serializes the
application accessibility tree;
 it is not TalkBack's output surface.
 TTS log lines prove synthesis
occurred but do not preserve utterance text.
 Guest-shell input did not reproduce a physical
TalkBack navigation gesture in this emulator session.
 No stronger claim about the guest input
implementation is needed because the emulator exposes a dedicated host controller.

The installed emulator ships its controller contract at
`$ANDROID_SDK_ROOT/emulator/lib/emulator_controller.proto`.
 Lines 173 to 180 define
`sendTouch` as host-scheduled touch input and state that coordinates are scaled to display
resolution:

```protobuf
// Sends touch events to the emulator. This operation is asynchronous and executed on the main looper.
// Coordinates are scaled to the emulator's display resolution.
rpc sendTouch(TouchEvent) returns (google.protobuf.Empty) {}
```

The same shipped file's lines 974 to 1064 define physical x/y coordinates,
 a stable contact
identifier,
 nonzero pressure while touching,
 zero pressure on release,
 and the target display.
That is enough to construct a real swipe without guest-shell input.
 The upstream contract is
also published as
`platform/tools/base/emulator/proto/emulator_controller.proto`.

TalkBack's public source at commit
`229212fdf5842191d0a93fc95d9ca1423b346866` confirms the visible output is its actual
speech pipeline,
 not a separate debug transcript.
`talkback/src/main/java/com/google/android/accessibility/talkback/TalkBackService.java:2990-2996`
reads the `pref_tts_overlay` setting into the pipeline:

```java
pipeline.setOverlayEnabled(
    PreferencesActivityUtils.getDiagnosticPref(
        this, R.string.pref_tts_overlay_key, R.bool.pref_tts_overlay_default));
```

`utils/src/main/java/com/google/android/accessibility/utils/output/SpeechControllerImpl.java:2068-2075`
passes the same text submitted for speech to the overlay:

```java
if (ttsOverlay != null) {
  if (eventId != null) {
    ttsOverlay.displayText(text, eventId.getEventSubtype());
  } else {
    ttsOverlay.displayText(text);
  }
}
```

The preference is a normal two-state setting in
`talkback/src/main/res/xml/preferences.xml:54-59`,
 titled `Display speech output` by
`talkback/src/main/res/values/strings.xml:1061`.

## Verification

Versions and sources:

- Android Emulator 37.1.11.0,
  build 15917651.
- Installed TalkBack 17.0.0.889642762,
  version code 60201248.
- TalkBack source clone:
  `/home/user/temp/agent/talkback-2026-09-08`,
  commit `229212fdf5842191d0a93fc95d9ca1423b346866`.
- Emulator controller schema:
  `/home/user/Android/Sdk/emulator/lib/emulator_controller.proto`.

The emulator was started on a loopback-only control port:

```text
emulator -avd Pixel_9_Pro_Fold -port 5564 -grpc 8554 \
  -no-snapshot-save -no-audio -gpu host
```

The Python client used `grpc.insecure_channel("127.0.0.1:8554")` and the generic unary
method path:

```python
send_touch = channel.unary_unary(
    "/android.emulation.control.EmulatorController/sendTouch",
    request_serializer=lambda payload: payload,
    response_deserializer=lambda payload: payload,
)
```

Each swipe sent several `TouchEvent` messages with one contact,
 increasing x coordinates,
and pressure 1024,
 followed by the same identifier at pressure zero.
 A full runnable copy
was used from `/home/user/temp/agent/emulator-grpc-swipe.py`.

### Working catalog

- gRPC right swipe advanced TalkBack from `Folders` to `Open` and displayed
  `Open. Button`.
- Seventy sequential gRPC swipes produced a complete ordered transcript for the accepted
  100% layout.
- TalkBack exposed the current row as
  `Current track. Another Xronixle. 4:35 · −1.2 dBTP. Button` after the state-description
  correction.
- The repaired mode group announced `1 of 4` through `4 of 4` at 85%,
  100%,
  and
  150%.
- Opaque screenshots retained the green TalkBack focus rectangle and visible speech text.

### Failing catalog

- Guest `input touchscreen swipe` sometimes left focus unchanged.
- Guest Tab moved keyboard focus but did not update TalkBack speech output.
- UI Automator XML contained application semantics but no speech-overlay text.
- Google TTS logcat contained synthesis lifecycle lines without the utterance.

## Folded-cover positive-control gap

The later disposable `Fold_No_Hardware_Probe` Search baseline used the
same installed emulator and TalkBack versions,
font scale `2.0`,
and no keyboard input.
The inner physical-swipe path advanced actual TalkBack focus and speech.
On the folded 1080 × 2424px cover,
UI Automator showed the Search controls and an overlay showed
“Back to player.
 Button”,
but the swipe path was not validated.

The shipped controller schema at
`emulator/lib/emulator_controller.proto:1061-1063` defines the target:

```protobuf
// The display device where the touch event occurred.
// Omitting or using the value 0 indicates the main display.
int32 display = 2;
```

The helper had defaulted to display `0`.
Encoding a host `TouchEvent.display = 1` also left the shown cover Back
focus unchanged in the sampled probe.
The controller's `getDisplayConfigurations` reported its main
2076 × 2152 configuration while Android's active cover logical display
was `0`.
A guest `input touchscreen -d 0 swipe` did not move the shown focus
either.
There was no positive control establishing that any sampled cover path
could advance TalkBack focus.
Repeated OCR of app text is therefore **not** a cover speech transcript.
The exact controller-to-folded-display mapping remains unresolved;
these null probes establish neither a cover traversal order nor an
upstream defect or inability.

The bounded Search observations and inspected inner witnesses live at
`package/music-player/design/evidence/search-talkback-native-baseline.md`.
The inner success does not remove this separate cover harness gate.
No additional keyboard or original-AVD experiment was performed.

## Verified workarounds

### Use emulator gRPC touch on loopback

Start the emulator with `-grpc 8554` and send down,
 move,
 and release events through
`sendTouch`.
 Tradeoffs:
 the emulator must be restarted to add the controller port;
 an
unauthenticated controller is acceptable only on loopback in a private local session.
 Use
JWT authorization for any non-loopback exposure.

### Enable TalkBack display speech output

Open TalkBack settings and enable `Display speech output` before capture.
 Disable usage
hints when the core announcement must remain visible instead of being replaced by the later
usage hint.
 Tradeoff:
 the overlay covers lower screen content,
 so it is evidence of speech and
focus,
 not a clean visual-design raster.
 Keep separate clean captures for pixel review.

### Keep UI Automator as structural evidence

Use XML for names,
 roles,
 bounds,
 and full text that may be visually ellipsized.
 Use the
TalkBack overlay for actual spoken order.
 Tradeoff:
 both artifacts are required because
neither alone proves the other layer.

## What does not work

- Increasing guest swipe distance or changing swipe duration did not make
  `adb shell input` a reliable TalkBack navigation source.
- Keyboard Tab and DPAD probes exercised hardware focus rather than proving TalkBack swipe
  traversal.
- TTS log filtering proved engine activity but did not expose utterance text.
- A custom accessibility service would log its own interpretation,
  not production TalkBack's
  speech,
  so it was not used as substitute evidence.

## Upstream filing decision

1.  **Is it really upstream's fault?**
     No.
    The emulator gRPC controller and TalkBack speech
    overlay both work as documented.
    Guest input,
    UI Automator,
    and logcat answer different
    questions.
2.  **Can upstream fix it?**
     Not applicable because no upstream defect is established.
3.  **Are they supporting this use case?**
     The emulator explicitly supports host touch injection,
    and TalkBack explicitly supports visible speech output.
4.  **Would the repositories welcome our contribution?**
     Not evaluated because no defect or
    missing source behavior remains.
5.  **Will they likely fix it?**
     Not applicable.
6.  **Have we prototyped a minimal fix?**
     No upstream fix is needed;
    the verified composition is
    entirely at the consumer boundary.

No matching exemption exists under `.out-of-scope/`.
 Searches did not identify an issue
whose resolution would add evidence beyond this working composition.
 There is no upstream
issue or comment to file.
