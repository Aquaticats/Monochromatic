# Safari Technology Preview: a raster inside a `clip-path` group of an SVG used as `<img>` went missing after scrolling away and back

Tool under test: Safari Technology Preview on macOS (release number not recorded for the original trials),
with Safari 27 and Safari 17.5 (macOS 14.5) as comparisons.
Surface trigger: an SVG drawn through `<img>` or `<picture>` whose content is a base64 JPEG `<image>`
inside a `<g clip-path="url(#...)">`.
Failure mode: the photo is blank (the vector parts still draw) once the picture has been scrolled out of view and back.

Status: the trigger is narrowed, the cause is not established, and no upstream filing is justified yet.
Every statement about the engine is a hypothesis unless it names a measurement.

## Symptom

Seen on 2026-10-20 on a public portfolio site (`chen1054/chen1054.github.io`),
in the Home picture `youth-in-limbo-photograph-and-film-frame-trimmed.svg`.
The SVG draws a film frame around a JPEG photograph.
Before the workaround its root held the photo `<image>` (2048 by 1365 JPEG) inside
`<g clip-path="url(#clip-44)">`, plus a second clipped `<image>` of the grunge frame texture (2176 by 2000 JPEG).
After the picture had been scrolled past and back, the photo was blank;
the commit message of `bda96e8` records it still blank 20 s later.

Measured by the session that found it, same day:

- Safari Technology Preview, clipped version, scroll away and back: photo blank in 3 of 4 trials.
- Safari 27 and Safari 17.5 (macOS 14.5), clipped version: photo drawn in 4 of 4 trials each.
- Full-page passes with the clipped version drew the photo in Safari 27 and Safari 17.5.
- Technology Preview, version without the clip (frame redrawn as vectors): blank in 0 of 4 trials;
  full-page passes later drew it in 3 of 3.
- Across all three Safari builds, the vector version showed the photo in 21 of 22 trials;
  the one miss was a single Technology Preview full-page pass.

The first message on `bda96e8` said "In Safari the photo ... disappeared".
That was corrected by a commit comment the same day (read with
`gh api repos/chen1054/chen1054.github.io/commits/bda96e82021850bdc8e3665542bdd0b3475e9577/comments`):
the loss was seen only in Technology Preview, and "removing the clip path improved the preview
but is not shown to be the whole cause".
Anyone reading the commit message alone would over-state the scope.

## Root cause

Not established.
No WebKit source was read for this bug, so no code path is cited.
Reading the `SVGImage` drawing path was not attempted because the failure could not be reproduced in a fixture
(see "Verification").

Hypotheses, all unverified:

- A decoded-image purge or a tile that is not repainted for an SVG image that left the viewport,
  specific to a raster inside a clipped group.
  Supporting observation only: the loss needs a scroll away and back.
- A Technology Preview regression absent from Safari 27 and Safari 17.5.
  Supporting observation only: both release builds drew the clipped version in 4 of 4 trials.
- The clip path was a trigger and not the whole cause.
  Supporting observation only: the vector version still missed once in Technology Preview.

## Verification

What was measured is in "Symptom".
It is a human-driven scroll test with 4 trials per cell, which is too small to give a rate;
3 of 4 against 0 of 4 is a signal, not a measurement of the bug's frequency.

An attempt to reproduce it on 2026-10-20 failed for tool reasons, not because the bug is absent:

- Machine: macOS 27.0 (`26A428`), Safari Technology Preview Release 252 (`22626.1.6.19.1`),
  driven through `safari-mcp-stp`.
- Fixture: the portfolio's clipped and vector SVGs (`bda96e8^` and `bda96e8`) as `<img>` above a 4000 px spacer,
  scrolled to the bottom, left 3 s, scrolled back to the top.
- Result: the viewport screenshot call returned an all-black PNG every time on this setup,
  so the blank-photo state could not be read.
- One full-page capture after the scroll back (clipped version) drew the photo and the frame, in 1 of 1 trial.
  A full-page capture repaints everything, and the session that found the bug reported that full-page passes drew
  the photo too, so this is not evidence that the on-screen viewport was drawn.
  The bug was neither reproduced nor ruled out on Technology Preview 252.

A future attempt needs a way to read the on-screen pixels of the scrolled-back viewport,
for example a screen-recording or window-capture path, plus 20 or more trials per cell.
Without that, treat the trial counts in "Symptom" as the only evidence.

## Verified workarounds

Remove the clip and draw the frame as vector shapes over the unclipped photo, as `bda96e8` did.
The two strips and twenty sprocket holes became flat `<rect>` shapes (strip `#343b4a`, holes `#b0b5bf`),
the JPEG bytes stayed unchanged, and the grunge texture image was dropped (178 KB less).
Result in the portfolio: the photo stayed at 1, 4 and 12 s after scrolling back, and Chromium changed only inside the
picture's box.
Tradeoffs: the frame is no longer the exported texture, so its look is a measured approximation;
the fix removes a trigger without a proven cause, so a different clipped raster could still be affected.

## What does not work

Both were tried before removing the clip, with the photo still going missing in Technology Preview:

- Wrapping the content in a nested `<svg>` viewport.
- Re-encoding the raster smaller.

## Upstream filing decision

Check of `.out-of-scope/`: 11 files, none about WebKit, Safari, or SVG.
No exemption applies.

Duplicate search on `bugs.webkit.org` (REST quicksearch, 2026-10-20), no match:
`svg image clip-path scroll disappears`, `svg image clip-path blank scroll`, `svg embedded image missing scroll`,
`svg image blank after scrolling`, `SVG image disappears scroll`, `svg img raster image missing`,
`svg clip-path image not rendered`, `svg embedded jpeg disappears`, `svg in img tag image element missing`,
`SVGImage decoded data purged`, `svg image not repainted offscreen`, `svg image decoded purge blank`.
Only bugs on other topics matched the words (for example 125289, patterns and masks in SVG images at screen resolution).
Absence from these queries is weak evidence: the symptom wording may differ from a filed report's.

The six constraints:

1. Is it upstream's fault?
   Plausibly, but unproven: it appears only in Technology Preview, and the cause may be the page.
2. Can upstream fix it?
   Unknown without a cause.
3. Do they support the use case?
   Yes, an SVG with an embedded raster in a clipped group is ordinary SVG-as-image content.
4. Would the repo welcome the contribution?
   `webkit.org/reporting-bugs/` asks for the latest build and a duplicate search,
   and mentions no AI policy (2026-10-20).
5. Will they likely fix it?
   No signal either way.
6. Prototyped a minimal fix?
   No, and none is possible before a cause is found.

Decision: do not file.
A report would need a reduced test case that fails reliably, a Technology Preview release number,
and a run on the latest build.
The draft is kept so a later session can fill the gaps.

~~~md
<!-- DRAFT for bugs.webkit.org, product WebKit, component SVG. Do not file as-is: no reduced test case, no cause. -->
Title: Raster <image> inside a clip-path group of an SVG used as <img> goes blank after scrolling away and back (Safari Technology Preview)

An SVG used through <img>/<picture> contains a base64 JPEG <image> inside <g clip-path="url(#c)">.
After the image is scrolled out of the viewport and back, the JPEG is not drawn; the vector parts are.
Safari Technology Preview: blank in 3 of 4 trials. Safari 27 and Safari 17.5 (macOS 14.5): drawn in 4 of 4 trials.
Removing the clip path (frame drawn as vectors): blank in 0 of 4 trials in Technology Preview.
Nested <svg> viewport and a smaller JPEG did not help.

Missing before filing: Technology Preview release number for the original trials, a self-contained reduced SVG,
and a run on the latest build.
~~~
