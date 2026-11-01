# agent-browser 0.39.1: `screenshot --full` showed lazy SVG previews smaller than viewport shots (once)

## Symptom

Observed once by a parent session on 2026-10-20 on this Linux host,
viewport 1440 by 950:
on a page whose images are
`<picture><source media="(width>=40rem)" srcset="x-trimmed.svg"><img src="x.svg" loading="lazy">`,
and after those images had loaded,
`agent-browser screenshot --full <path>` drew the SVG images smaller than a normal viewport
screenshot of the same scrolled position.
The viewport screenshots matched the layout box;
the full-page capture showed the art small inside its box.

This could not be reproduced as written.
What did reproduce,
on the same page,
is a different and flaky difference between the two capture modes:
in some runs the case-study preview images are absent from the full-page capture
although the viewport screenshots show them.
Both are recorded here,
because the conditions of the original observation (exact page state,
device scale factor) were not captured.

The page was served at `http://127.0.0.1:4174/index.html` by a local no-store server (read-only for this investigation).
At 1440 by 950 it measures `document.documentElement.scrollHeight` 5797 and `devicePixelRatio` 1.
The full-page PNG is 1440 by 5797.

## Root cause

Not established.
What the source does,
`vercel-labs/agent-browser` tag `v0.39.1`,
commit `1165952d74ceed98fdc2dd97190ad41f44dd0e24`:

`cli/src/native/screenshot.rs:156-205` (`capture_screenshot_base64`) builds a full-page capture from two settings:

```rust
capture_beyond_viewport: if options.full_page { Some(true) } else { None },
// ...
let metrics: Value = client
    .send_command_no_params("Page.getLayoutMetrics", Some(session_id))
    .await?;

let content_size = metrics
    .get("contentSize")
    .or_else(|| metrics.get("cssContentSize"));
// ...
params.clip = Some(Viewport { x: 0.0, y: 0.0, width, height, scale: 1.0 });
```

There is no scroll step,
no wait for lazy images,
and no device-metrics override.
The clip comes from the deprecated `contentSize` first and `cssContentSize` only as a fallback,
and its `scale` is fixed at `1.0`.

That `contentSize` preference is a real upstream defect at a device scale factor other than 1:
[issue 2064][issue-2064] reports a full-page image twice too large,
with the page in the top-left quarter,
at a scale factor of 2,
and [PR 2125][pr-2125] switches to `cssContentSize`.
It does not explain this incident:
the page here reports `devicePixelRatio` 1,
the output size matched the page exactly,
and the art was smaller inside its box,
not the page smaller inside the image.
The parent's device scale factor during the original observation was not recorded,
so it is not excluded for that run.

### Hypotheses that were tested and did not hold

- Viewport units re-laid out under `captureBeyondViewport`:
  a fixture with a `height: 50vh` box holding an SVG with
  `object-fit: contain` rendered the box at 600 by 475 in both modes,
  and the top 950 rows of the full-page PNG were
  pixel-identical to the viewport PNG (0 differing pixels,
  threshold 8 of 255).
- The page's own `100dvh` rule (`styles.css:186`) shrinking the art:
  the top screen of the real page was identical in both modes
  (0 of 1,368,000 pixels differ).
- A time-based intro animation:
  it did cause a false difference once (a fading band,
  region near 1056 by 541 at scroll 300).
  `styles.css:443-473` animates the home introduction with delays up to 6.8 seconds and a 1.8 second reveal,
  so wait at least nine seconds after load before comparing.
  After that wait the remaining differences were the missing previews described in the Verification section.

### Not established

- Why the previews are absent in some full-page captures
  while every `<img>` reports `complete` with a nonzero `naturalWidth`.
  The page uses `loading="lazy"` with `decoding="async"` on these images
  (lines 242-243 and 277-278 of the served `index.html`);
  an incomplete decode at capture time is a guess,
  not a measurement.
  A forced `img.decode()` call timed out in this harness (`Runtime.evaluate` timeout)
  because the phone-width variants have no source,
  so that guess is untested.

## Verification

Version under test:
agent-browser 0.39.1,
tag `v0.39.1`,
commit `1165952`,
headless Chrome 149.0.7827.54,
1440 by 950,
scale factor 1.
Comparison harness:
take `screenshot --full`,
then for each offset scroll,
read back `window.scrollY` (the last offset is clamped to
4847,
so the crop must use the real value),
take a viewport `screenshot`,
and compare it with the same rows of the full image.
A pixel counts as different when its grayscale difference is above 8 of 255.

Commands,
per session `s` (the harness itself was a scratch Python script around these calls):

```sh
agent-browser --session s set viewport 1440 950
agent-browser --session s open http://127.0.0.1:4174/index.html
agent-browser --session s wait 8000
agent-browser --session s screenshot --full "${HOME}/temp/agent/ab-full/full.png"
agent-browser --session s eval "window.scrollTo(0,2300); new Promise(r=>setTimeout(()=>r(window.scrollY),500))"
agent-browser --session s screenshot "${HOME}/temp/agent/ab-full/vp-2300.png"
```

### Runs where the two modes match

- Top screen only,
  no scrolling,
  viewport set before `open`:
  0 of 1,368,000 pixels differ.
- Scroll through the whole page first so every wide image reports loaded,
  then full capture,
  then viewport captures at five offsets:
  4 of 9 such runs matched with 0 differing pixels at every offset
  (one run at offsets 950 to 4750,
  two of three repeats of one resize-after-open sequence,
  and one run that first loaded the page at a 390 by 950 viewport and then resized).

### Runs where the previews are missing from the full-page capture

- Open,
  resize,
  and take the full capture without scrolling:
  the lazy previews are absent because they were never fetched
  (only 10 of 27 images had loaded);
  the viewport screenshots after scrolling show them.
- Scroll through the whole page so every wide image reports loaded,
  scroll back to the top,
  then full capture:
  previews absent at the four preview offsets (54,000 to 191,000 differing pixels per 1440 by 950 frame) in 5 of 9 runs.
  All four runs of the second harness and its variants (wait eight seconds,
  step 700 pixels,
  500 ms pauses;
  one variant without the wait,
  one resizing after `open`) differed,
  and one of the three repeats of the first harness differed,
  so the outcome is not deterministic for one sequence.

### Never seen

- Any run where the preview art was present but drawn smaller than in the viewport capture.

## Verified workarounds

### Scroll and take viewport screenshots, then stitch

Scroll in steps no larger than the viewport height,
take a viewport screenshot at each step,
and stitch the crops.
Every viewport screenshot in these runs showed the previews.
Tradeoffs:
fixed or sticky elements repeat in every frame;
the last step is clamped to the maximum scroll (4847 here),
so crop that frame by the real `scrollY`,
not the requested one;
and animated content is captured at different times in different frames.

### Verify a full-page capture against viewport frames before trusting it

Compare at least two offsets (as in the harness) after waiting for animations to finish.
Tradeoff:
it costs extra screenshots,
and it detects the difference but does not fix it.

## What does not work

- Scrolling through the page before `screenshot --full` to load lazy images:
  5 of 9 such runs still lost the previews.
- Waiting for network idle only:
  the images were loaded;
  the full capture still lacked them in some runs.
- Reading `devicePixelRatio` 1 as proof the original observation was unrelated to scale:
  it was not measured then.

## Upstream filing decision

`.out-of-scope/` was searched with `rg -n -i "agent-browser|browser" .out-of-scope` (exit code 1,
no match);
no exemption applies.
Duplicate searches on `vercel-labs/agent-browser` (issues and PRs,
open and closed) for lazy loading,
missing images,
blank full-page output and `captureBeyondViewport` found related but different reports:
[issue 2064][issue-2064] and [PR 2125][pr-2125] (scale factor 2,
wrong clip size),
[issue 1964][issue-1964] and
[PR 2126][pr-2126] (selector screenshots blank below the first viewport),
and [issue 1282][issue-1282]
(`--full` misses internal scroll containers).
None describes missing or smaller `<picture>` previews.

No issue is drafted.

1.  **Upstream fault?**
    Not assessable:
    the reported symptom (smaller art) was not reproduced,
    and the flaky missing previews could equally be Chrome's decode and lazy-load behavior
    under `captureBeyondViewport`.
2.  **Can upstream fix it?**
    Not assessed;
    there is no demonstrated cause.
3.  **Supported use case?**
    Full-page screenshots are supported (`README.md:139`);
    lazy images are not mentioned.
4.  **Contribution welcome?**
    No `CONTRIBUTING.md`,
    templates or filing policy was found;
    no AI-assistance ban.
5.  **Will they likely fix it?**
    Unknown;
    sibling full-page reports are open with PRs pending.
6.  **Minimal fix prototyped?**
    No. Constraints 1 and 2 cannot be answered for an unreproduced symptom,
    so the prototype gate is not entered.
    A new report would need a deterministic reproduction first,
    ideally a minimal page and the device scale factor.

[issue-1282]: https://github.com/vercel-labs/agent-browser/issues/1282
[issue-1964]: https://github.com/vercel-labs/agent-browser/issues/1964
[issue-2064]: https://github.com/vercel-labs/agent-browser/issues/2064
[pr-2125]: https://github.com/vercel-labs/agent-browser/pull/2125
[pr-2126]: https://github.com/vercel-labs/agent-browser/pull/2126
