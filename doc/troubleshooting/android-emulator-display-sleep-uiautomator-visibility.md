# Emulator display sleep and uiautomator viewport visibility break capture runs

## Symptom

Long native capture runs over the folded cover display fail partway with
`Compose did not replace the Android launch splash on the cover display.`
even though the same candidate launches fine by hand.
The dumped hierarchy contains
only empty window bounds,
and a screencap of the cover panel is fully black.

Separately,
at 200% text a capture's expected markers vanish even while the screen is
awake:
names that sit below the visible scroll viewport are absent from the hierarchy,
so marker lists that passed at 100% fail at 200%.

## Root cause

Two independent emulator behaviors:

1.  The virtual display sleeps on its own timeout while a capture run is still working.
    `dumpsys power` reports `mWakefulness=Asleep`,
    Compose never draws,
    and
    `uiautomator dump` returns window shells without content.
    Waking the display
    (`input keyevent 224` plus `cmd window dismiss-keyguard`) makes the very same activity
    compose and dump correctly,
    which isolates sleep as the cause.
    `settings put global stay_on_while_plugged_in 7` alone did not hold across night-mode
    and font-scale changes during a run.
2.  `uiautomator dump` omits nodes whose bounds fall outside the visible viewport of a
    scrolled container.
    At 200% text the picker's name wall scrolls,
    so names beyond the
    first screens are simply not in the XML,
    while at 100% everything fit and appeared.

## Verification

- With the display asleep,
  `uiautomator dump` yields only `bounds="[0,0][1080,2424]"`
  shells and screencap returns a black panel;
  after wake plus dismiss-keyguard the same
  dump contains the expected `text` and `content-desc` nodes.
- At 200% the hierarchy lists names only up to the visible viewport edge (for example
  through `Crystal Castles`) and then jumps to the deck,
  proving viewport culling rather
  than a composition failure.

## Verified workarounds

- Wake per capture inside the capture loop:
  `input keyevent 224`,
  then
  `cmd window dismiss-keyguard`,
  before `am start`;
  also pin
  `stay_on_while_plugged_in` to 7 at run start and restore the original value in the
  run's finally block.
- Choose capture markers that name nodes visible at the capture's font scale;
  at 200%
  prefer top-row and deck nodes over scrolled list names.
- Place pixel-sample coordinates inside the visible band for the capture's font scale
  (for the cover picker at 200%,
  the in-slot picker band sits around y 310 to 970 px,
  so
  samples use y 600 rather than y 1200).

## Upstream filing decision

None.
 Both behaviors are the emulator and uiautomator working as designed;
the fixes
belong in the capture harness,
not upstream.
