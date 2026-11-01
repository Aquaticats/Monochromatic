# Folded-cover folder picker: the D46 revisit

## Purpose and authority

D46 chose P4 as a temporary, pre-1.x design for the folded cover's folder picker:
the app-bar folder title and caret open the picker in the list area while the playback deck stays visible.
The human chose it while believing a better solution exists, and never named what is wrong with it.
On 2026-10-10 the human asked for the revisit now and left the angle to the agent:
build one version from the best of the agent's design judgment, then ask for approval.
No production implementation is authorized by this plan.

## What the existing P4 captures show

The measurements come from the native Fold captures `questions/render/cover-round-cover-picker-p4-s100.png`
and `cover-round-cover-picker-p4-s200.png` (1080 by 2424 pixels at 390dpi):

- The trigger does not change when the picker is open.
  The title and caret look the same closed and open, and the caret still points down.
  The only open-state cue is the picker itself, so the way to close it is not shown.
  The interactive study's code receives the open state and does not draw it.
- At 100% text the letter rail shows A to M and at 200% it shows A to H with the next letter clipped,
  and the last folder row is cut by the deck at 200%.
  The rail scrolls on its own by design (D17), so this is expected, not a defect.

## Constraints a better solution must respect

- D17: the picker is a filter, never a list of a thousand rows.
  No sub-letter segmentation in any form, no chip styling on names, and the wrapped several-per-line layout is required.
  The one-column rail scrolls on its own.
- K1 (discouraging back navigation) and K3 (a non-local sheet) stay rejected (D46).
- The deck keeps its content height (D44) and must not change size when the picker opens;
  earlier scan-indicator work (D13) ruled out reflow.
- Every state needs two visible channels, never color alone.

## The agent's version: P5

P5 keeps P4's calm title trigger and fixes its one acknowledged weakness, the weak affordance, in the open state only.
While the picker is open the title sits in a tonal container and its caret points up.
Closed, it is exactly P4.
The state shows by container and by caret direction, and the accessibility state description already reads expanded
and collapsed.

Larger changes were considered and not built:

- A full-height picker that collapses the deck gives the rail more room.
  It makes the deck change size and takes the playback controls away while choosing, which the deck rules above forbid.
- Putting the picker in a floating panel is P1 and P3, which D46 did not choose.
- Replacing the rail with sticky letter headers and one-column rows breaks D17.

## Queue

- [x] Study the existing P4 captures and the constraints.
- [x] Add P5 to the debug candidate host in `prototype/music-player-cover-picker`.
- [x] Capture P4 and P5 side by side on the owned Fold cover in light and dark at 100% and 200% text.
- [x] Inspect every capture, remove the status strip, and publish a review page.
- [x] Ask the human to approve P5, keep P4, or name a different problem, through the question tool.
  Answer (D104): approve P5 but remove the container; no further build and capture round;
  still unsatisfied, to be dealt with in another major version.

## Result

The first P5 build moved the title 16dp to the right when the picker opened, which the first captures showed.
The container now reaches 8dp past the title on each side, and the rebuilt captures draw the title within one pixel
of its place in P4 at both text sizes and in both themes.
The review is `package/music-player/design/questions/cover-picker-revisit.html`:
eight inspected native views (P4 and P5, light and dark, 100% and 200% text) from two emulator visits of prototype
`a61a4dd4761662acbe1adde0d64730d1d9f1ff49` on `prototype/music-player-cover-picker`.
`verify:cover-picker:revisit` and `test:cover-picker:revisit` pass, each of the four guards fails the test when
removed, and the page was checked in a browser in light and dark with no console errors.
P5's honest cost is that its container shares the Open action's tonal color,
and only its static open state was captured:
opening, closing, Back, focus return and TalkBack were not exercised for it.
