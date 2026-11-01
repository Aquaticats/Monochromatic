# On-device VoiceOver fidelity sweep for the surviving iOS frameworks

## What this proves

Accessibility is an owner hard rule (the exact criterion that disqualified Slint).
 This sweep closes the one
a11y dimension that cannot be automated:
 that VoiceOver actually speaks each control with its correct label,
role,
 and value,
 in a sensible focus order,
 on the real iPhone X (iOS 16.7.16).
 Two a11y questions are distinct
and only the second is open:

- Crash-survival (does the framework's a11y code run on 16.7 or die on an iOS-17 symbol like Slint).
   Already
  closed for every survivor:
   all 18 render on the device with no dyld or objc2 a11y death.
- VoiceOver fidelity (does VoiceOver read each control correctly and navigate sanely).
   This runbook.

The element-tree dumps in `../decision/ios-iphone-x-vet-report/vet-ui-automation.md` are the headless half of
this:
 they show what each rendering model exposes (labels,
 roles,
 traits).
 This runbook is the audible half:
whether VoiceOver speaks those exposures and traverses them in order.
 The one load-bearing case is Compose
Multiplatform,
 the self-drawn renderer that is a live contender for both apps and whose interactive a11y is
unverified,
 so a render PASS can still hide a gap there;
 the WebKit-native and native-UIKit groups are
confirmations.
 The other self-drawn renderers,
 Avalonia and Uno-Skia,
 are .
NET:
 ruled out as the kopia host by
the Go-versus-Mono SIGKILL finding and bottom-ranked for the music-player,
 so their a11y does not gate a likely
decision and they are optional here,
 not a must.

Bridges tried and confirmed insufficient (so this handoff is a real obstacle,
 not an unconsidered one):
 the
Appium / XCUITest accessibility-tree dump captures labels,
 roles,
 and traits but not the audible speech,
 the
spoken order,
 or whether focus can reach and escape every element;
 a simulator Accessibility Audit (Xcode
**Accessibility Inspector**) would run on iOS 26.5,
 not the 16.7 target,
 and on a simulator where Slint's killer
symbols exist,
 so it is not a 16.7-device substitute.
 VoiceOver speech and live focus traversal genuinely need a
human listening on the device.

The phone work is all owner-driven (hold the iPhone X,
 listen,
 swipe).
 The Mac is used only to install each
gate;
 the agent can do those installs over SSH on request,
 so in practice the owner just holds the phone and
says which framework to load next.

## Setup

Status:
 TODO

- The iPhone X (`iPhone10,3`,
   iOS 16.7.16) set up per `ios-iphone-x-codesign-setup.md` (Developer Mode on,
   this
  Mac trusted,
   the `anchor` app installed so churning gate apps never drops trust).
   The gate apps reuse one
  bundle id `dev.monochromatic.iosvet.hellodevice`,
   so only one framework's gate is installed at a time;
   the
  agent swaps them with `ideviceinstaller -n upgrade <app>`.
- A quiet room,
   because the check is what you hear.
- Set the Accessibility Shortcut so you can toggle VoiceOver fast (essential:
   once VoiceOver is on,
   single tap
  selects-and-speaks and you must double-tap to activate,
   so a hardware toggle is how you get back out).
   On the
  phone open **Settings > Accessibility > Accessibility Shortcut** and tap **VoiceOver** so it has a checkmark.
  Expected outcome:
   triple-clicking the **side button** now toggles VoiceOver.
- Learn the four gestures you need:
   single tap selects and speaks an item;
   swipe right moves to the next item;
  swipe left moves to the previous;
   double-tap anywhere activates the selected item;
   two-finger swipe up reads
  everything from the top.
- Ask the agent to install the first framework's gate (it runs `ideviceinstaller -n install <gate>.app` from the
  build under `/Volumes/MacData/ios-vet/`),
   or install it yourself from a Mac terminal.
   Expected outcome:
  `Install: Complete` and the gate's icon on the home screen.

## Steps

Status:
 TODO

Do Compose Multiplatform first (the one load-bearing self-drawn contender),
 then Flutter (the best interactive
test),
 then one representative of each remaining group;
 Avalonia and Uno are optional (see What this proves).
Repeat steps 2 to 8 for each framework in the checklist under What to check.

1. Open the framework's gate app (tap its icon).
    Expected outcome:
    the gate's render screen appears (for
   example **Compose Multiplatform on iOS**,
    or **WKWebView OK**).
2. Turn VoiceOver on:
    **triple-click the side button**.
    Expected outcome:
    you hear "VoiceOver on",
    a focus
   rectangle appears around the first element,
    and that element is spoken.
3. Read the whole screen top to bottom:
    **two-finger swipe up**.
    Expected outcome:
    VoiceOver speaks every visible
   text item and control in order,
    then stops.
    Listen for any item that is silent or spoken as a bare type with
   no name (for example "button" with no label).
4. Walk each element one at a time:
    **swipe right** repeatedly from the top.
    Expected outcome:
    each swipe moves
   the focus rectangle to the next element and speaks it;
    you can reach the last element and then the status bar
   without the focus getting stuck (no trap).
5. For each interactive control,
    confirm its role and value are spoken.
    On a control,
    VoiceOver should append the
   trait,
    for example "Increment,
    button" or a title spoken as a "heading".
    Expected outcome:
    the trait word is
   spoken,
    not just the label.
6. Activate an interactive control:
    with it selected,
    **double-tap** anywhere.
    Expected outcome:
    the action
   fires and VoiceOver announces the new state.
    On the Flutter gate,
    double-tapping **Increment** twice makes the
   counter speak "1" then "2".
7. Judge the focus order:
    was the swipe-right order a sensible reading order (top-to-bottom,
    leading-to-trailing),
   with nothing skipped and nothing unreachable.
    Expected outcome:
    a clear yes,
    or a noted specific defect.
8. Turn VoiceOver off:
    **triple-click the side button** (you hear "VoiceOver off"),
    then tell the agent to load
   the next framework's gate.
    Record the result for this framework in the checklist below.

## What to check

Status:
 TODO

A framework passes when,
 on its gate:
 every text item is spoken with its real label;
 every interactive control
is spoken with its trait ("button",
 "heading") and,
 where it has one,
 its value;
 and swipe-right traverses all
elements in a sensible order with no skip and no trap.
 A failure is a control spoken as a bare "button" with no
name,
 a wrong or missing trait,
 a silent element,
 a nonsensical order,
 or a focus trap.

Concrete spoken strings to expect per gate (you should hear these words):

- Capacitor gate:
   "Capacitor vet",
   "WKWebView OK".
- Cordova / Quasar gate:
   "Quasar in WKWebView",
   "JS:
   720",
   and the action spoken as "QUASAR BUTTON,
   button";
  double-tap fires it.
- Flutter gate:
   "Flutter Demo Home Page" spoken as a heading,
   "You have pushed the button this many times:
  ",
  the counter "0",
   and "Increment,
   button";
   double-tap Increment twice and the counter speaks "2".
- Compose Multiplatform gate:
   "Compose Gate",
   "Compose Multiplatform on iOS" (both as plain text;
   this gate has
  no interactive control,
   so it tests label and order only,
   see the limitation below).
- React Native gate:
   "RN Gate",
   "Hermes:
   ON".
   NativeScript gate:
   "NS Gate",
   "V8 JS:
   720",
   "CROSSING OK".
   Lynx
  gate:
   "Lynx Gate",
   "Rust:
   720",
   "CROSSING OK".
   UIKit gate:
   "UIKit Gate",
   "Pure UIKit,
   native a11y",
   "RENDER
  OK".

Checklist (do the MUST group first;
 tick PASS or write the defect):

- [ ] Compose Multiplatform (self-drawn,
       MUST:
       the one load-bearing self-drawn contender for both apps)
- [ ] Flutter (self-drawn but a mature UIKit bridge;
       the interactive counter makes it the best role/value test)
- [ ] Capacitor (WebKit,
       confirm)
- [ ] Cordova / Quasar (WebKit,
       has an interactive button,
       confirm role + activation)
- [ ] Ionic,
       Framework7,
       Onsen (WebKit,
       spot-check one is enough if Capacitor passed)
- [ ] Dioxus (WebKit via wry,
       confirm)
- [ ] React Native (native UIKit,
       confirm)
- [ ] NativeScript (native UIKit,
       confirm)
- [ ] Lynx (native UIKit via its own engine,
       confirm)
- [ ] MAUI,
       UIKit,
       SwiftUI,
       SnapKit (native baselines,
       spot-check one)
- [ ] Avalonia,
       Uno-Skia,
       Uno-native (OPTIONAL,
       music-player-only:
       the .
      NET trio is ruled out as the kopia host
  by the Go-versus-Mono SIGKILL and is bottom-ranked for the music-player,
       so sweep these only if .
      NET is being
  reconsidered for the music-player)

Known evidence limit (not a defect,
 record it as scope):
 the gate apps are render-proofs,
 and most carry no
interactive control;
 only the Flutter gate (counter) and the Cordova/Quasar gate (button) exercise interactive
role,
 value,
 and activation announcements.
 So the self-drawn MUST group can be checked here for label and
focus-order announcement,
 but a full interactive-control VoiceOver verdict for Compose,
 Avalonia,
 and Uno needs a
richer gate with real buttons,
 toggles,
 and a list.
 If the label-and-order pass is clean,
 build that richer gate
(or test the real app prototype) before declaring the self-drawn a11y fully cleared.

## Restore

Status:
 TODO

- Turn VoiceOver off:
   **triple-click the side button** until you hear "VoiceOver off".
- Optionally remove the Accessibility Shortcut:
   **Settings > Accessibility > Accessibility Shortcut**,
   tap
  **VoiceOver** to clear its checkmark.
   Expected outcome:
   triple-click no longer toggles VoiceOver.
- Leave the `anchor` app installed (never uninstall it;
   it holds developer trust).
   The last gate app under the
  shared bundle id can stay or be removed by the agent with
  `ideviceinstaller -n uninstall dev.monochromatic.iosvet.hellodevice`;
   removing it is safe because `anchor`
  keeps the certificate trusted.
- Record the checklist results in `../decision/ios-iphone-x-music-player-kopia-stack.md` under the accessibility
  section,
   replacing "owner-owed sweep" with the per-framework outcome.
