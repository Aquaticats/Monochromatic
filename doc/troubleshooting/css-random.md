# CSS `random()`: the `/TR/` snapshot's stale `by` step makes `random(a, b, by c)` silently dropped

`@monochromatic-dev/hyperscript-core`'s `cssRandom` emitted `random(1, 1000, by 1)`
from the day it was written until `8decc5d92`.
That spelling comes from the W3C `/TR/` snapshot of CSS Values 5,
which is nearly two years stale.
The current grammar removed the `by` keyword,
so every engine rejected the declaration,
including the ones that ship `random()`.

## Symptom

No error, no warning, nothing in the console.
The declaration is simply absent from the computed style,
because an unparseable value makes CSS drop the declaration at parse time.

Two consumers were affected:

-   `package/ssg/aquati.cat/src/component/shuffle-children.ts` emitted
    `shuffle-children > * { order: random(1, 1000, by 1) }`.
    Its CSS shuffle path never ran in any browser.
    The fallback at `package/ssg/aquati.cat/src/client/shuffle-children.ts`
    silently did all the work, which is why nothing looked broken.
-   The footer newsticker in `package/ssg/aquati.cat/src/component/site-footer.ts`
    would have inherited the same defect while implementing issue #477.

The failure is invisible precisely because the feature has a graceful-degradation story.
A dropped `random()` looks identical to a browser that does not support `random()`,
and today almost every browser is the latter, so the bug had no observable signature.

## Root cause

Two W3C URLs publish contradictory grammars for the same feature,
and the stale one ranks higher and looks more authoritative.

The published snapshot at `https://www.w3.org/TR/css-values-5/`,
dated 2024-11-11, states:

```text
<random()> = random( <random-caching-options>? , <calc-sum>, <calc-sum>, [by <calc-sum>]? )

<random-caching-options> = <dashed-ident> || per-element
```

The editor's draft at `https://drafts.csswg.org/css-values-5/`,
fetched 2026-09-06, states:

```text
<random()> = random( <random-key>? , <calc-sum>, <calc-sum>, <calc-sum>? )
<random-key> = auto | <random-cache-key> | fixed <number [0,1]>
<random-cache-key> =  <dashed-ident> || element-scoped
                       || [ property-scoped | property-index-scoped | <random-ua-ident> ]
<random-ua-ident> = <custom-ident>
```

Three changes matter:

-   The step lost its `by` prefix and became a bare positional fourth argument.
    This is what broke `cssRandom`.
-   `<random-caching-options>` became `<random-key>`,
    gaining `auto`, `property-scoped`, `property-index-scoped`, and `fixed <number>`.
-   `per-element` was renamed to `element-scoped`.

Counting occurrences in each fetched document confirms the split rather than inferring it.
In the editor's draft, `per-element` occurs 0 times and `element-scoped` occurs 34 times.
In the `/TR/` snapshot, `per-element` occurs 12 times and `element-scoped` occurs 0 times.

The offending code read:

```ts
// package/hyperscript/core/src/css/values.constructors.ts, before 8decc5d92
  return step === undefined
    ? `random(${min}, ${max})` as CssValue
    : `random(${min}, ${max}, by ${step})` as CssValue;
```

### An earlier reading of the caching default was wrong

While triaging `shuffle-children` this session, the working hypothesis was that a
bare `random()` shares one drawn value across every element matching the declaration,
which would have made a per-child shuffle impossible without an explicit keyword.
That is backwards.
The editor's draft defines the omitted key as `auto`, and says of it:

```text
This is equivalent to specifying element-scoped property-index-scoped,
```

So omitting the key already yields a distinct value per element,
which is exactly what `shuffle-children` wants.
Its intent was correct all along;
only the `by` spelling was wrong.
Do not re-derive the sharing hypothesis: the default is per-element, not shared.

## Verification

Editor's draft fetched 2026-09-06 from `https://drafts.csswg.org/css-values-5/`
(1087129 bytes).
Snapshot fetched the same day from `https://www.w3.org/TR/css-values-5/`.
Browser under test: Chrome 149, via `agent-browser`.

Grammar extraction, which is what actually proves the claim:

```bash
curl -sL https://drafts.csswg.org/css-values-5/ -o ed.html
curl -sL https://www.w3.org/TR/css-values-5/ -o tr.html
python3 -c "
import re, html
for name in ['ed.html', 'tr.html']:
    s = re.sub(r'<[^>]+>', '', open(name, encoding='utf-8').read())
    s = html.unescape(s)
    i = s.find('<random()> = random(')
    print(name, s[i:i + 110].strip())
    for kw in ['per-element', 'element-scoped']:
        print(' ', kw, s.count(kw))
"
```

Browser probe for support and fallback behaviour:

```js
CSS.supports('order', 'random(0, 10, by 1)')  // false in Chrome 149
CSS.supports('order', 'random(0, 10)')        // false in Chrome 149
```

Both return false, so this probe does **not** discriminate the `by` spelling from
plain absence of support.
That distinction rests on the two fetched specifications, not on a local parse test.
No engine installed on this machine implements `random()`,
and `agent-browser` is Chromium-only with no engine-selection flag,
so a discriminating parse test could not be run here.
Issue #488 tracks running one on WebKit.

Patterns that are valid under the current grammar:

```css
order: random(1, 1000, 1);                    /* auto key, positional step */
order: random(0, 8);                          /* auto key, no step */
--seed: random(--ticker-seed, 0, 8, 1);       /* dashed-ident key, document-global */
width: random(element-scoped, 100px, 200px);  /* explicit per-element */
animation-delay: random(--a, 0s, 31.5s);      /* times are fine, any consistent type */
```

Patterns that are rejected:

```css
order: random(1, 1000, by 1);          /* `by` removed from the grammar */
width: random(per-element, 1px, 2px);  /* renamed to element-scoped */
width: random(element-shared, 1px, 2px); /* never in the ED; see MDN note below */
width: random(50px, 180deg);           /* arguments must share one type */
```

Fallback behaviour, measured in Chrome 149 with eight sibling elements:

```js
// @property --seed { syntax: "<integer>"; inherits: true; initial-value: 7 }
// #rt > i { --seed: random(0, 1000, by 1); order: random(0, 1000, by 1); }
// All eight elements computed --seed: "7" and order: "0".
```

That is the mechanism the newsticker fix relies on:
a registered property's `initial-value` is what an unparseable or unsupported
`random()` degrades to.

## The second victim: a support probe written in the stale grammar

The more instructive failure is not the emitted CSS but the feature detection.

`package/ssg/aquati.cat/src/client/shuffle-children.ts` decides whether to run its
DOM-reordering fallback by probing for support:

```ts
// before abf711ee6
const RANDOM_PROBE = 'random(1, 1000, by 1)';

if (CSS.supports('order', RANDOM_PROBE)) {
  return;
}
```

`CSS.supports` is a syntactic check, so it answers "is this string parseable",
not "does this feature exist".
A probe written in a grammar no engine accepts returns false forever,
in engines that implement `random()` exactly as loudly as in engines that do not.

The failure mode is a ratchet:

-   The probe reports "unsupported" in every browser.
-   So the scripted fallback always runs.
-   So the shuffle always visibly works.
-   So nothing looks broken, and the dead CSS path is never noticed.

The bug hid itself.
It only became harmful once the emitted CSS was corrected:
from Safari 26.2 and Chrome 155 the CSS path applies while the probe still says
"unsupported", so both the CSS shuffle and the DOM reorder run.
The file's own documentation claimed the script "is a no-op" in supporting
browsers, which had never been true.

Fixed in `abf711ee6` by spelling the probe the way `cssRandom` emits.
The general rule: a feature probe and the code it guards must be generated from
one spelling, or verified against each other, because a probe in the wrong
grammar fails safe-looking and silent.

## Verified in WebKit 26.6

Chrome cannot answer the question that matters, because it rejects
`random(1, 1000, 1)` and `random(1, 1000, by 1)` alike.
WebKit 26.6, via the Playwright container, discriminates them:

```json
{"supportsCurrentGrammar": true, "supportsStaleByGrammar": false}
```

That is the direct proof that `by` was the defect rather than absent support.

The same run confirmed three further things, all previously spec-read rather than
observed:

-   **The omitted key really is per-element.** All 16 `shuffle-children` children
    drew distinct `order` values (`distinctOrders: 16`), so `auto` behaves as
    `element-scoped property-index-scoped`.
-   **`random()` with `@property` works end to end.** The footer ticker's
    `--ticker-seed` computed to `6`, not the registered `initial-value` of `0`,
    so a real draw happened.
-   **The probe fix removes the double-shuffle**, shown against a positive
    control rather than asserted:

```json
{"shipped": {"domMatchesSource": true,  "distinctCssOrders": 16},
 "positiveControl": {"domMatchesSource": false, "distinctCssOrders": 16}}
```

With the shipped probe the DOM stays in source order, so only the CSS shuffles.
Forcing `CSS.supports` to report `random()` unsupported reproduces the pre-fix
probe, and the DOM order then diverges from source order while the CSS ordering
still applies: both shuffles running at once.
The control is what makes the passing arm meaningful; without it,
`domMatchesSource: true` is equally consistent with a harness that cannot detect
reordering at all, which is exactly the false null an earlier version of this
harness produced when its extraction regex silently matched nothing.

### Harness

Requires the container from `playwright.Dockerfile` and the site served on
`:4321` (`mise run //package/ssg/aquati.cat:dev`, or `caddy run --config Caddyfile`
from `package/ssg/aquati.cat` against a built `dist`).

```bash
podman run --rm --network=host --security-opt label=disable \
  -v "$PWD:/work" -w /work monochromatic-playwright node probe.mjs
```

Three host-specific details cost time and are worth keeping:
`:Z` relabelling fails here (`lsetxattr ... operation not permitted` on a
`node_modules` file), so the mount needs `--security-opt label=disable`;
the image deliberately ships only browsers, so the host `node_modules` must be
mounted rather than a narrow probe directory;
and `playwright` is not hoisted by pnpm, so the import must be `@playwright/test`.

```js
import { webkit } from '@playwright/test';

const url = 'http://localhost:4321/en/guess-the-tagline';
const raw = await (await fetch(url)).text();
const seg = raw.slice(raw.indexOf('<shuffle-children'), raw.indexOf('</shuffle-children>'));
const sourceIds = [...new Set([...seg.matchAll(/name="(q-[0-9a-f]+)"/g)].map((m) => m[1]))];

const run = async (label, forceUnsupported) => {
  const browser = await webkit.launch();
  const page = await browser.newPage();
  if (forceUnsupported) {
    await page.addInitScript(() => {
      const real = CSS.supports.bind(CSS);
      CSS.supports = (...a) =>
        (a[0] === 'order' && String(a[1]).includes('random') ? false : real(...a));
    });
  }
  await page.goto(url, { waitUntil: 'load' });
  const r = await page.evaluate(() => {
    const kids = [...document.querySelector('shuffle-children').children];
    return {
      domIds: kids.map((k) => k.querySelector('input[type=radio]')?.name ?? '?'),
      orders: kids.map((k) => getComputedStyle(k).order),
    };
  });
  await browser.close();
  return {
    label,
    domMatchesSource: JSON.stringify(r.domIds) === JSON.stringify(sourceIds),
    distinctCssOrders: new Set(r.orders).size,
  };
};

console.log(JSON.stringify({
  shipped: await run('shipped probe', false),
  positiveControl: await run('probe forced false', true),
}, null, 1));
```

This is not yet a standing test, but nothing structural blocks it any more.

An earlier revision of this document claimed the blocker was
`playwright.browser.config.ts` pointing `testDir` at a nonexistent `./packages`.
That was stale: the config already reads `./package`, corrected in `eac44c979`.
The real blocker was the container mount.
`mise.toml` mounted the repo as `:Z`, which relabels every file under the mount
with per-container SELinux MCS categories, and this tree holds 291,243 files
under `node_modules/.monochromatic` that the logger appends to while the relabel
walks them. The relabel failed with `lsetxattr ... operation not permitted` and
podman exited 126 before Playwright started, so no browser test could run in any
engine. Fixed in `3d004e60d` by using `label=disable` and a plain bind mount.

`mise run test:browser -- --list` now collects 66 tests in 5 files across
chromium, firefox and webkit.

## Verified workarounds

### Register the target custom property with `@property`

```css
@property --ticker-seed { syntax: "<integer>"; inherits: true; initial-value: 0 }
site-footer footer { --ticker-seed: random(--ticker-seed, 0, 8, 1) }
```

Tradeoff: the degraded value is the same for every visitor and every load,
so the randomness disappears entirely rather than degrading to weaker randomness.
For a decorative rotation that is fine;
for anything where repetition is a correctness problem it is not.

Registering also fixes a second, separate trap.
An **unregistered** custom property does not evaluate `random()` at the
declaration; it substitutes textually, so each use site draws its own value.
`--size: random(100px, 500px)` used in both `width` and `height` yields a
rectangle, not a square.
A non-universal `syntax` descriptor forces evaluation at the declaration.

### Pass an explicit `<dashed-ident>` key when one value must be shared

Tradeoff: a bare dashed-ident is document-global,
so every element matching the declaration shares the draw.
That is desirable for one seed on one element and wrong for a shuffle.
Note that Safari changed this mid-series: the bare-name-is-global behaviour
arrived in Safari 26.5, which also removed `element-shared`.
Code written against Safari 26.2 through 26.4 changes meaning on 26.5 and later.

### Keep a scripted fallback when the effect is functional

`shuffle-children` pairs its CSS path with `src/client/shuffle-children.ts`.
Tradeoff: both paths can apply at once once engines start supporting `random()`,
double-shuffling the children.
That risk is now live and is tracked in issue #488.

## What does not work

-   **Trusting `https://www.w3.org/TR/css-values-5/`.**
    It is the URL search engines and most write-ups surface,
    and it has been wrong about this feature since 2024-11-11.
    Cite `drafts.csswg.org/css-values-5/` instead.
-   **Using `@supports` as a proxy for "this works".**
    `@supports (width: random(1px, 2px))` is a purely syntactic check.
    It passes on engines whose `random()` support is substantially incomplete,
    so it overstates support badly.
    Test the exact sub-feature relied on, for example
    `@supports (width: random(property-scoped, 1px, 2px))`.
-   **Assuming the omitted key shares one value.**
    Covered under "Root cause"; the default is per-element.
-   **Round-tripping a declaration through `cssText`.**
    Setting `height: random(100px, 200px)` serialises back as
    `height: random(element-scoped ua-height-1, 100px, 200px)`,
    because the specification requires the UA ident to survive serialisation.
    Any test that string-compares declarations will break.

## Upstream filing decision

`.out-of-scope/` was checked before considering any filing.
It contains `bun-install.md`, `cargo-workspace.md`, `claude-code-upstream-bugs.md`,
`codex-harness.md`, `jsr.md`, `lightningcss.md`, `low-impact-typescript-formatting.md`,
`module-es-monolith.md`, `pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`, and `typescript-project-references.md`.
None covers CSS specifications, MDN, or browser engines, so no exemption applies.

The defect in our tree was ours, not upstream's:
`cssRandom` transcribed a stale grammar.
That is fixed in `8decc5d92` and nothing about it is filable anywhere.

One genuinely upstream defect surfaced during the investigation.
MDN's `random()` page contradicts itself.
Its generated "Formal syntax" block correctly reads
`<random-cache-key> = <dashed-ident> || element-scoped || ...`,
while its hand-written prose and its runnable "Try it" demo use `element-shared`,
a keyword that occurs 0 times in the editor's draft and was removed from
Safari in 26.5.
Occurrence counts on the fetched page: `element-shared` 23, `element-scoped` 1.
The demo on that page therefore cannot work in any engine.

### Duplicate search

Searched `mdn/content` issues (open and closed) and pull requests.

-   `mdn/content#44761`, "docs(css): update random() syntax to match latest
    specification", **open**, filed 2026-07-17, touching
    `files/en-us/web/css/reference/values/random/index.md`.
    It renames the first parameter to `<random-key>` and documents `auto`,
    `element-scoped`, `property-scoped`, and `property-index-scoped`.
-   It states `Fixes #44502`, so the tracking issue exists too.
-   `mdn/content#44544`, "Fix random() CSS value documentation", closed.

### Six-constraint check

1.  **Really upstream's fault?** Yes, for the MDN page.
    No, for our `cssRandom` bug, which was our own transcription error.
2.  **Can upstream fix it?** Yes. It is a documentation edit to one file.
3.  **Supporting this use case?** Yes. MDN documents `random()` deliberately,
    including an interactive demo.
4.  **Would the repo welcome our contribution?** Not assessed, because the
    question is moot: a fix is already open.
5.  **Will they likely fix it?** Already in progress via `mdn/content#44761`.
6.  **Prototyped a minimal fix?** Not applicable. An open pull request already
    implements the fix, so prototyping a competing patch would be waste.

### Artifact: nothing to file

No new issue and no comment.
`mdn/content#44761` already covers the `element-shared` to `element-scoped`
correction and the `<random-key>` rename, which is the entirety of what this
investigation found wrong with the page.
A comment saying so would be a bare "+1" and is not worth making.
Recorded here explicitly so a future session does not re-derive the same empty comment.

If that pull request is closed without merging and the page still shows
`element-shared`, the finding above is enough to re-open the question.

## Related

-   Issue #477, the footer newsticker clipping fix that surfaced this.
-   Issue #488, verifying the `shuffle-children` CSS path on an engine that
    implements `random()`, and confirming the CSS and script paths do not both apply.
