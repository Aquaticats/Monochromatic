# axe-core 4.12.1 and 4.13.0 report modal-dialog text contrast as incomplete

## Symptom

The self-contained music-player questionnaire passes axe-core WCAG A and AA checks
while its native `<dialog>` is closed.
 After the same dialog opens with `showModal()`,
axe-core reports no violation but returns `color-contrast` as incomplete for the dialog
heading,
 dimensions,
 scale label,
 and buttons:

```text
Element's background color could not be determined because it is overlapped by another element
```

Running axe against the full document or only `#preview-dialog` produces the same
incomplete result.
 Browser `elementsFromPoint()` probes at each reported element's
center put the element itself first,
 followed by its dialog ancestors.
 The user-visible
content is not actually covered.

## Root cause

This is axe-core's documented conservative uncertainty path,
 not evidence that the
questionnaire fails contrast.

`lib/commons/color/get-background-stack.js:16-29` builds a rendered-element stack for
each text rectangle.
 If axe's first virtual-grid element is not the text node,
 it stores
`bgOverlap` and returns no background:

```js
export default function getBackgroundStack(node) {
  const stacks = getTextElementStack(node).map(stack => {
    stack = reduceToElementsBelowFloating(stack, node);
    stack = sortPageBackground(stack);
    return stack;
  });

  for (let index = 0; index < stacks.length; index++) {
    const stack = stacks[index];

    if (stack[0] !== node) {
      incompleteData.set('bgColor', 'bgOverlap');
      return null;
    }
```

`lib/checks/color/color-contrast-evaluate.js:84-92,127-133,152-181` asks that helper for
the background,
 records its missing-data key,
 and returns `undefined` when the
background remains unknown.
 That `undefined` becomes an incomplete result rather than
a pass or violation.

Axe's modal detector admits that browser APIs do not expose enough top-layer ordering
information.
 `lib/commons/dom/get-modal-dialog.js:7-10` says its detection is a heuristic
with known issues and cannot identify which dialog is visually on top:

```js
/**
 * Determine if a dialog element is opened as a modal.
 * Currently there are no APIs to determine this so we'll use a bit of a
 * hacky solution that has known issues.
 * This can tell us that a dialog element is open but it cannot tell us which
 * one is the top layer, nor which one is visually on top. Nested dialogs that
 * are opened using both `.show` and `.showModal` can cause issues as well.
 * @see https://github.com/dequelabs/axe-core/issues/3463
 */
```

Its visual sorter models CSS stacking levels and document order in
`lib/commons/dom/visually-sort.js:11-37,79-97`;
 it has no top-layer branch.
 The browser's
live hit-test and axe's virtual stacking model can therefore disagree for a modal.

This conservative outcome matches `README.md:15-21`:
 axe-core reports elements as
incomplete when it cannot be certain and requires manual review.

## Verification

The run used axe-core tag `v4.13.0`,
 commit
`1cc54b900413660610180d631feb73c9e74f4dc9`.
 The injected `axe.min.js` has SHA-256
`c24f097bd2f451d4f933e8bc7d8d539f8672a2ebcb5cc9f9f3eec8ca9470a0c1`.
The browser was Helium 0.15.6.1 using Chromium 151.0.7922.169.

Open the questionnaire through Helium CDP,
 inject axe-core,
 and compare closed and open
states:

```console
agent-browser --session music-player-md3 --cdp 9224 open \
  'file:///var/home/user/Monochromatic/package/music-player/design/questions/current.html'
agent-browser --session music-player-md3 --cdp 9224 eval --stdin \
  < /home/user/temp/agent/axe-core-4.13.0/axe.min.js
agent-browser --session music-player-md3 --cdp 9224 eval \
  'axe.run(document).then(result => ({ violations: result.violations, incomplete: result.incomplete }))'
agent-browser --session music-player-md3 --cdp 9224 eval \
  'document.querySelector(".preview-open").click()'
agent-browser --session music-player-md3 --cdp 9224 eval \
  'axe.run(document).then(result => ({ violations: result.violations, incomplete: result.incomplete }))'
```

### Cleanly automated cases

- Closed questionnaire page:
   no WCAG A or AA violations and no incomplete checks.
- Open dialog checks unrelated to foreground/background inference:
   pass.
- Visible targets,
   names,
   labels,
   alternative text,
   IDs,
   and focus:
   pass direct probes.

### Incomplete case

- The original theme-comparison modal has no violations;
   `color-contrast` is incomplete
  for eight toolbar text elements with message key `bgOverlap`.
- Restricting `axe.run()` to `#preview-dialog` leaves the same eight incomplete nodes.
- The replacement divider-clarification modal reports the same `bgOverlap` path for
  six visible nodes.
   Its toolbar wraps differently,
   so axe's virtual overlap set is a
  subset rather than a stable node count.
- The existing-screen refinement modal reports seven `color-contrast` incomplete
  nodes.
   Six use `bgOverlap`;
   `#zoom-dimensions` uses
  `elmPartiallyObscuring`.
   Direct computed-color checks resolve the dimensions text to
  9.344:1,
   the scale label to 17.075:1,
   and each affected button pair to 6.441:1.
  Axe reports no violation.
   The longer large-text segmented preview title wraps the
  toolbar differently and produces no incomplete nodes,
   confirming that node count is
  layout-sensitive rather than a stable property of the dialog.
- The redundant-cue matrix modal at a 390 × 844 CSS-pixel viewport reports two
  `color-contrast` incompletes,
   `#zoom-title` and `#zoom-dimensions`,
   through the
  same overlap path.
   Their flat computed pairs resolve to 17.075:1 and 9.344:1.
  It reports no axe violation.

### Replacement-form accessibility corrections

The first divider-clarification run produced one real axe violation on
`#zoom-stage`:
 `scrollable-region-focusable`.
 Giving the scroll region `tabindex="0"`
and a named `role="region"` removes it.
 An intermediate `aria-label` without a role
produced the separate `aria-prohibited-attr` incomplete diagnostic;
 adding the region
role removes that uncertainty too.

After those corrections,
 the closed page has no violations or incomplete checks.
 The
open modal has no violations;
 only the six `bgOverlap` contrast incompletes remain.
Direct computed-color checks resolve them to two pairs:

- `#1D1B20` on white:
   17.075:1.
- `#6750A4` and white in either foreground/background direction:
   6.441:1.

## Verified workarounds

### Resolve flat computed backgrounds and calculate ratios

For each incomplete node,
 read its computed foreground.
 Walk ancestors until reaching
the first non-transparent computed background,
 then calculate WCAG relative luminance.
For the questionnaire dialog this gives:

- Heading on white:
   17.07:1.
- Supporting dimensions and scale text on white:
   9.34:1.
- Secondary action text on white:
   6.44:1.
- White reset text on the primary button:
   6.44:1.

All exceed the 4.5:1 requirement.
 The tradeoff is scope:
 this fallback is valid for the
questionnaire's flat opaque colors,
 not gradients,
 images,
 opacity compositing,
 blend
modes,
 or unknown external content.

### Keep incomplete distinct from violations

Treat axe's incomplete output as a manual-review queue.
 Never silently convert it to a
pass,
 and never report it as an axe violation.
 The tradeoff is an explicit manual step
for each unresolved node.

## What does not work

- Re-running axe against only the dialog does not resolve the virtual stacking result.
- Browser `elementsFromPoint()` proving the text is topmost does not change axe's
  already built virtual grid.
- Treating `bgOverlap` as a failed contrast ratio is incorrect:
   axe has no ratio and
  deliberately returns incomplete.
- Treating the absence of a violation as a pass is also incorrect until the computed
  pair is measured.

## Upstream filing decision

Existing issue [dequelabs/axe-core#3463](https://github.com/dequelabs/axe-core/issues/3463),
“Support the `<dialog>` element,”
 covers modal and top-layer detection.
 It is closed,
but the 4.13.0 source still cites it and explicitly records known limitations.

1.  Upstream fault:
     No demonstrated defect.
     Returning incomplete when stacking is
    uncertain is documented axe-core behavior.
2.  Upstream can change it:
     Yes,
     if browser APIs or axe's top-layer model improve.
3.  Supported use case:
     Yes.
     Axe has modal-dialog detection and dialog integration
    tests.
4.  Contribution policy:
     `CONTRIBUTING.md:1-18` accepts contributions under a CLA
    and repository style.
     No AI-assistance prohibition was found in `CONTRIBUTING.md`,
    `README.md`,
     or `.github/` templates.
5.  Likely upstream action:
     Unclear.
     Issue 3463 is closed,
     while current source still
    labels the heuristic as limited.
6.  Compatible minimal fix:
     Not prototyped because constraint one does not hold and
    the conservative incomplete result is safe.

No new issue should be filed.
 There is no additive comment to post without a minimal
reproduction independent of this questionnaire and evidence that axe's conservative
classification itself is wrong.

## Filename review under axe-core 4.12.1

### Symptom and source boundary

`agent-browser` 0.38.1's A/AA audit reported axe-core 4.12.1.
The filename review's closed page had zero violations and incomplete checks
at desktop/mobile light/dark settings.
Opening its native dialog at 1440 × 1000 produced zero violations and one
`color-contrast` incomplete on `#preview-metadata`:

```text
Element's background color could not be determined because it partially overlaps other elements
```

The 390 × 844 open-dialog audits had zero violations and incomplete checks.
This is distinct from the earlier 4.13.0 `bgOverlap` observations.
No causal equivalence is inferred from the similar surface.

The read-only `dequelabs/axe-core` tag `v4.12.1`,
revision `5d002cca1f862a0699d9f1bb7b5a1ec334fa1b22`,
was inspected.
This matches the reported library version;
installed-bundle byte equivalence and the exact origin of the differing
virtual stacks were not established.

In that source,
`lib/commons/dom/get-text-element-stack.js:12` to `18` maps visible text
rectangles to the virtual grid's stacks:

```js
// lib/commons/dom/get-text-element-stack.js
const clientRects = getVisibleChildTextRects(node);
return clientRects.map(rect => getRectStack(grid, rect));
```

`lib/commons/color/get-background-stack.js:32` to `35` records uncertainty
when those stacks differ:

```js
// lib/commons/color/get-background-stack.js
if (index !== 0 && !shallowArraysEqual(stack, stacks[0])) {
  incompleteData.set('bgColor', 'elmPartiallyObscuring');
  return null;
}
```

`lib/checks/color/color-contrast.json:44` maps that key to the observed message.
`lib/checks/color/color-contrast-evaluate.js:163` to `176` returns `undefined`
for a missing background rather than establishing a contrast violation.
`README.md:19` requires manual review of uncertain results.
These source paths explain the reported classification,
not why this review's virtual stacks differ.

### Verification and bounded manual resolution

The policy-selection form was later withdrawn because supporting text is
templated and user-configurable in Settings.
The observations in this subsection belong to that archived form.
Fresh [evidence-only review checks][template-verification] repeated the
closed-page/modal audits and bounded manual review after removing votes;
they do not implement templates or establish native acceptance.

The [verification summary][filename-verification] retains the closed-page
and open-dialog results separately.
The checks ran offline in a 2GiB/2CPU container.
The native image cohort remained unchanged.
Modal controls,
zoom,
panning,
100% dp reset,
Escape/focus return and all four composed placement/visibility answers were
exercised in each desktop/mobile light/dark context.

For each incomplete node,
the consumer probe retained its exact diagnostic,
checked foreground and the first opaque ancestor background,
and rejected images,
opacity,
filters,
blend modes or text shadows in that color chain.
Live hit tests at the centers of its three text rectangles placed the metadata
node foremost.
The screenshot region was also inspected.
The computed flat pairs measured:

- Light:
  `rgb(32, 30, 38)` on `rgb(252, 249, 255)`,
  15.78947720932554:1.
- Dark:
  `rgb(238, 231, 246)` on `rgb(23, 21, 29)`,
  14.978445394818559:1.

Black/white and equal-color controls yielded 21:1 and 1:1.
Both measured review-text pairs exceed 4.5:1.
The axe incomplete results remain incomplete;
the direct measurement resolves this flat-pair manual review only.
It is not a native screenshot contrast assessment or a universal modal pass.

A reproduction uses the committed offline artifact.
Set `REPO_ROOT` to this checkout and use a fresh named browser session:

```console
# package/music-player/design/questions/search-filename-comparison.html
agent-browser --session filename-contrast --pin-tab open \
  "file://${REPO_ROOT}/package/music-player/design/questions/search-filename-comparison.html"
agent-browser --session filename-contrast set viewport 1440 1000
agent-browser --session filename-contrast set media light
agent-browser --session filename-contrast select '#scale' 2
agent-browser --session filename-contrast click 'figure[data-scene=placementsupport] button'
agent-browser --session filename-contrast a11y --tags wcag2a,wcag2aa,wcag21a,wcag21aa --json
agent-browser --session filename-contrast close
```

Requiring every modal audit to have zero incomplete checks rejected the first
coverage run.
Treating that rejection as a demonstrated contrast violation would be wrong.
No CSS workaround,
axe rule disable or upstream source edit was applied.
The manual fallback's tradeoff is an explicit,
flat-opaque-color-only review.
It accepts only axe 4.12.1's exact partial-overlap diagnostic on
`#preview-metadata` at the desktop width.
Other diagnostics,
selectors,
versions,
widths or color-chain effects fail verification.

### Upstream filing decision for the filename incident

The `.out-of-scope/` census had no matching axe exemption.
Issue 3463 and its comments were read again;
the thread is closed and covers historical modal detection,
not a demonstrated defect in this specific contrast result.
Exact `elmPartiallyObscuring dialog` issue and PR searches returned no matches;
that empty query is not proof of no comparable behavior,
so the existing modal issue and versioned source remain the comparison.

- Upstream fault:
  not demonstrated;
  this is a documented uncertainty classification with measured adequate flat contrast.
- Upstream ability:
  the source's virtual-stack inference is the relevant boundary;
  no architectural impossibility is claimed.
- Supported use case:
  the tag includes modal detection in `lib/commons/dom/get-modal-dialog.js`
  and the historical support thread.
- Contribution policy:
  `CONTRIBUTING.md:5` requires a CLA and points to
  `doc/code-submission-guidelines.md`.
  Searches of that document,
  `.github/`,
  `CONTRIBUTING.md` and `README.md` found no AI-assistance prohibition.
- Likely action:
  no decision on this specific result was found;
  the closed support issue does not prove a refusal to improve inference.
- Minimal fix:
  not prototyped because an upstream fault was not established.
  The read-only source clone was not modified.

Nothing additive establishing a bug or fix was identified.
No new issue,
comment or patch was sent.

[filename-verification]: ../../package/music-player/design/questions/evidence/search-filename-policy-withdrawn-verification.json
[template-verification]: ../../package/music-player/design/questions/evidence/search-filename-comparison-review-verification.json
