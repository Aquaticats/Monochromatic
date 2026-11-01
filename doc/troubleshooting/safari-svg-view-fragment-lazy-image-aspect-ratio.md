# Safari (Technology Preview 252, Safari 27, Safari 17.5): an `<img>` SVG with a `#view` fragment and `loading="lazy"` is laid out at the whole canvas's aspect ratio, leaving small artwork in blank margins

Tool under test:
Safari Technology Preview Release 252 (`22626.1.6.19.1`, user agent `Version/27.0`) on macOS 27.0 (`26A428`),
driven through `safari-mcp-stp` on `m1`.
Chromium 149 (headless, Linux) as the positive control.
Surface trigger: `file.svg#view-id` as an `<img src>` or as a `<picture><source srcset>` candidate,
where the SVG has only a root `viewBox` and a `<view id="view-id" viewBox="...">` that trims blank export margins.
Failure mode: the image box takes the aspect ratio of the whole canvas instead of the view,
so the view is drawn letterboxed and the artwork looks small with blank bands at the sides.

This document corrects the first report of the bug (see "Correction to the first report").
The mechanism is only partly established: see "Root cause".

## Symptom

Seen on 2026-10-20 on a public portfolio site (`chen1054/chen1054.github.io`, commit `1b85496`).
Each Home project picture was a `<picture>` whose wide `<source srcset>` pointed at `art.svg#project-artwork-...`:

```html
<!-- Home page of the portfolio before commit 1b85496 -->
<picture class="illustration">
  <source media="(width>=40rem)" srcset="assets/tales-for-safer-trails-campaign-logo-and-speech-bubble.svg#project-artwork-without-blank-export-margins">
  <img src="assets/tales-for-safer-trails-campaign-logo-and-speech-bubble.svg" alt="" width="390" height="211"
    loading="lazy" decoding="async">
</picture>
```

In Safari the artwork was drawn small inside its box, with the export's blank margins.
The commit message records a natural size of 277 by 150 (Chromium: the view's 190 by 150)
and boxes 221 px tall at 1920 px wide against 323 px after the fix.
The session notes record the symptom in Safari Technology Preview and Safari 17.5 (macOS 14.5);
the commit message names Safari 27 and Technology Preview.
Neither Safari 17.5 nor a release Safari 27 was available for this write-up,
so only Technology Preview 252 was measured.

The same fragment on the case-study hero `<img>` elements drew correctly.
That difference is explained by the measurements in "Verification", not by `<picture>` itself.

## Correction to the first report

The first report said Safari "ignores the `#view` fragment when the URL is in a `<picture><source srcset>`" and
"draws the whole SVG canvas".
Measured in Technology Preview 252, both halves are wrong:

- The view is applied when drawing, in `<picture>` and in plain `<img>` alike, eager or lazy.
  The drawn picture is the view, letterboxed into a box whose aspect ratio came from somewhere else.
  For the lazy case this was checked on the synthetic fixture in "Verification":
  box 400 by 200, and a full-page capture (taken after the box was read, box unchanged afterwards) showed one red
  square 200 by 200 CSS pixels centered in the box and no blue frame, so the fragment was applied.
  With the real Tales artwork the two readings differ by under 10 px, so it cannot settle this by eye.
- `<picture>` makes no difference in any measured case.
  The variable that flips the result is `loading="lazy"` on the `<img>`.
  The Home pictures had it; the case-study heroes (`menume.html:140-146`) did not, and also set an explicit
  `aspect-ratio` (`styles.css:1313-1315`, rule `.project-preview>button>img`, with
  `--case-study-preview-aspect-ratio: 258 / 204` at `styles.css:1098`),
  which makes the box independent of the image's own ratio.

## Root cause

Established by measurement: the box height of an SVG `<img>` with a `#view` fragment depends on `loading="lazy"` and
on what else on the page already used the same SVG file, in Technology Preview 252.
Chromium 149 follows the view in every case measured.

Not established: why Safari's layout ratio differs between eager and lazy images.
The WebKit source read (checkout `d1eca9768abe8426bde2a0c6be2ad10008972fdb`, 2026-10-20, `Source/WebCore`) shows
the following, and none of it separates eager from lazy.
Whether Technology Preview 252 was built from this tree is unknown.

### What reaches the drawing code is the same for `<picture>` and `<img src>`

`HTMLImageElement::imageSourceURL()` returns the best-fit candidate string, fragment included,
whether the candidate came from a `<picture>` `<source>` or from `src`:

```cpp
// Source/WebCore/html/HTMLImageElement.cpp:242
String HTMLImageElement::imageSourceURL() const
{
    return m_bestFitImageURL.isEmpty() ? attributeWithoutSynchronization(srcAttr) : m_bestFitImageURL;
}
```

`RenderImageResource::setCachedImage` turns that into the authored URL of the style image,
and `CachedImage::drawingExtrasForRenderer` hands it to `SVGImage` at draw time:

```cpp
// Source/WebCore/rendering/RenderImageResource.cpp:112
authoredURL = protect(m_renderer->document())->encodingParseURL(element->imageSourceURL());
// Source/WebCore/style/values/images/kinds/StyleCachedImage.cpp:279
return { m_url.resolved, linkParametersForResource(/* ... */) };
// Source/WebCore/svg/graphics/SVGImage.cpp:209
state.fragmentURL = styleExtras->fragmentURL();
```

So a fragment from `srcset` is not dropped before drawing.
This matches the measurement that `<picture>` and `<img src>` behave identically.

### The fragment is applied at draw time, the natural ratio comes from the authored `viewBox`

```cpp
// Source/WebCore/svg/graphics/SVGImage.cpp:248
if (state.fragmentURL != m_appliedDocumentState.fragmentURL || state.ignoreRootPreserveAspectRatio != m_appliedDocumentState.ignoreRootPreserveAspectRatio) {
    if (state.fragmentURL != m_appliedDocumentState.fragmentURL)
        protect(view)->scrollToFragment(state.fragmentURL);
```

```cpp
// Source/WebCore/svg/SVGSVGElement.cpp:830
else if (auto viewBoxSize = viewBox().size(); !viewBoxSize.isEmpty())
    naturalDimensions.aspectRatio = viewBoxSize;
```

`SVGSVGElement::naturalDimensions()` reads `viewBox()`, the root's authored attribute,
not `currentViewBoxRect()` (which honors an active view, `SVGSVGElement.cpp:714`).
Applying a `<view>` does not change `viewBox()`:
`inheritViewAttributes` writes the view's box into the separate view spec object and sets `m_useCurrentView`:

```cpp
// Source/WebCore/svg/SVGSVGElement.cpp:986
void SVGSVGElement::inheritViewAttributes(const SVGViewElement& viewElement)
{
    Ref view = currentView();
    m_useCurrentView = true;

    if (viewElement.hasValidViewBox())
        view->setViewBox(viewElement.viewBox());
```

Reading only this, an SVG with a `<view>` always reports the whole canvas's ratio, which is the lazy result
(277 by 150 is the whole canvas of the Tales artwork).
It does not explain the eager result, where the box follows the view.
One unverified reading is that an eager image is drawn, and so has the fragment applied, before layout reads its
ratio, while a lazy image is laid out first;
a second is that the shared state described under "The fragment is applied at draw time" decides it.

`SVGImage.cpp:241` carries this comment:
`// FIXME: webkit.org/b/322833 - the resource document holds one container's parameters at a time`.
Images of one SVG file share one resource document, and the measurements show that sharing:
a no-fragment `<img>` of the same file took the view's ratio when it sat on a page with fragment images.

## Verification

Harness: any static server on `127.0.0.1`, a page of `<img>` elements,
and a script writing the box size into `document.title`.
The fixture below is self-contained and reproduces the lazy-versus-eager split.

```svg
<!-- art.svg: whole canvas 400 by 200, view of the 100 by 100 square -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 200">
<view id="v" viewBox="150 50 100 100"/>
<rect x="150" y="50" width="100" height="100" fill="#d00"/>
<rect x="0" y="0" width="400" height="200" fill="none" stroke="#06c" stroke-width="4"/>
</svg>
```

```html
<!-- single.html?pic=1&lazy=1: one image per page load, so no other image shares the SVG -->
<style>body{margin:0;width:400px}img{display:block;inline-size:100%;block-size:auto}</style>
<div id="host"></div>
<script>
  const q = new URLSearchParams(location.search);
  const lazy = q.get('lazy') === '1' ? ' loading="lazy"' : '';
  document.getElementById('host').innerHTML = q.get('pic') === '1'
    ? `<picture><source media="(width>=40rem)" srcset="art.svg#v"><img id="t" src="art.svg"${lazy}></picture>`
    : `<img id="t" src="art.svg#v"${lazy}>`;
  setTimeout(() => {
    const i = document.getElementById('t'), b = i.getBoundingClientRect();
    document.title = `nat ${i.naturalWidth}x${i.naturalHeight} box ${b.width}x${Math.round(b.height)}`;
  }, 900);
</script>
```

The box is 400 wide.
The view is 100 by 100 (a 400 by 400 box), the whole canvas is 400 by 200.

Measured with that fixture (saved copies: `art-noattr.svg` and `single-synth.html` in
`${HOME}/temp/agent/svg-view-fixture-2026-10-20/`; `${HOME}/temp/agent` is scratch and not durable):

- Technology Preview 252, `loading="lazy"` on a plain `<img>`: box 400 by 200, natural size 300 by 150.
  This is the failing case: the view is letterboxed into a whole-canvas-ratio box.
- Technology Preview 252, no `loading` attribute: box 400 by 400.
- Chromium 149, with and without `loading="lazy"`: box 400 by 400, natural size 150 by 150.

Measured with the portfolio's real artwork (the Tales SVG as of `1b85496^`, whose root `viewBox` is `0 943 390 211`
and whose view is `62 943 258 204`), one image per page load, `width="390" height="211"` attributes,
box 400 wide, Technology Preview 252 (natural size 277 by 150 in every row):

- Plain `<img>`, eager: 316 tall (the view's ratio, matching Chromium).
- `<picture>` with `<source media="(width>=40rem)">`, eager: 316.
- Plain `<img>` and `<picture>`, `loading="lazy"`: 216 (the whole canvas's ratio).
- `loading="lazy"` with `decoding="async"`: 216; `decoding="async"` alone: 316, so `decoding` is not the variable.
- Lazy with width and height attributes of 100 by 100: 216, so the box is not derived from the attributes.
- Lazy, no fragment (control): 216.
- Lazy, plus `style="aspect-ratio: 258 / 204"` on the image: 316, and a full-page screenshot showed the artwork
  filling the box with no side margins.
- The box stayed 216 for 3 s after load and after scrolling the image into view again.

Measured with several images of the same file on one page, Technology Preview 252:

- Five images, all with the fragment, some lazy and some eager: all 216, including the eager ones.
- Seven eager images, one of them with no fragment at all: all 316, including the one with no fragment.
  Chromium 149 gave 216 for that one.

Release Safari 27 and Safari 17.5, measured later on 2026-10-20 by the portfolio session on the case-study hero
viewers (a lazy fragment `<img>` in a closed popover, opened with `showPopover()`, sized by its natural ratio,
WebDriver at a 1440 px layout viewport, the WebDriver helper printing the answering browser: Safari 27.0 and
Safari 17.5): Technology Preview, Safari 27 and Safari 17.5 all opened them at 2.179, 1.789 and 1.848 wide per
unit of height against the views' 1.5, 1.541 and 1.265, which Chromium matched. A viewport capture of the open
MenuMe viewer in Safari 27 and 17.5 shows the artwork at about two thirds of the viewer, blank to its right and
below. With the trimmed copies (portfolio `6e97123`) all three gave the views' ratios and the artwork fills the
viewer. A first run of this measurement went to Technology Preview for every rig, because the helper ignored its
driver setting; the numbers here are from the re-run.

Not measured: iOS Safari, Firefox,
and a lazy image placed below the fold (every case in this section had the image in the first viewport).
Viewport screenshots from `safari-mcp-stp` came back as an all-black PNG in this setup;
only full-page captures were readable, so drawing was checked with full-page captures and box sizes.

## Verified workarounds

Trimmed copies, as the portfolio landed in `1b85496`.
A copy of each SVG whose root `viewBox` equals the view's `viewBox`, with the `<view>` removed,
referenced without a fragment.
The whole canvas is then the artwork, so every ratio source agrees.
Tradeoff: one extra file per view, and the two files must be kept in step when the artwork changes.
The portfolio's post-change Safari checks showed boxes as tall at 1920 px as at 1440 px (323 against 221 before).

An explicit `aspect-ratio` equal to the view's ratio on the `<img>`.
Measured in the real-artwork list: the lazy image became 316 tall with the artwork filling the box (Technology Preview 252).
This is what the case-study heroes already do.
Tradeoff: the ratio is duplicated in CSS and must be edited with the view;
the browser no longer reads it from the file.

## What does not work

- Treating `<picture>` as the cause.
  The measurements show no `<picture>` effect, and the source path for `srcset` and `src` is the same.
- Removing `decoding="async"`.
  It changes nothing in the measurements.
- Relying on eager loading alone.
  Eager images shared a file with lazy images in one page and still got the whole-canvas ratio (five-image page).
  Eager loading is not a safe switch while another image of the same file can be lazy.
- Reading `naturalWidth` and `naturalHeight` as evidence of the layout ratio.
  Technology Preview 252 reported the whole canvas's natural size in the eager case while laying the box out at the
  view's ratio.

## Upstream filing decision

Check of `.out-of-scope/` (`/var/home/user/Monochromatic/.out-of-scope/`):
11 files, none about WebKit, Safari, or SVG (`rg --ignore-case "safari|webkit|svg" .out-of-scope` returns nothing).
No exemption applies.

Duplicate search on `bugs.webkit.org` (REST quicksearch, 2026-10-20),
with terms drawn from the symptom and the source path:
`svg view fragment`, `svg #view img`, `svg fragment intrinsic`, `svg fragment aspect ratio img`,
`svgView img`, `svg view intrinsic aspect ratio`, `picture srcset svg fragment`, `picture source svg fragment`.
No report matches this behavior (lazy `<img>` with a `#view` fragment laid out at the whole canvas's ratio).
Related, none the same:

- [163812](https://bugs.webkit.org/show_bug.cgi?id=163812):
  a `<view>` fragment is not respected when the SVG is a CSS background (NEW since 2016).
- [322167](https://bugs.webkit.org/show_bug.cgi?id=322167):
  `drawImage()` ignores an `svgView()` fragment when the image has no renderer (NEW, with pull request 72294 linked).
- [326931](https://bugs.webkit.org/show_bug.cgi?id=326931):
  an `svgView()` fragment discards the root `viewBox` and `preserveAspectRatio`.
- [302908](https://bugs.webkit.org/show_bug.cgi?id=302908):
  media fragments with SVG documents (`#xywh`, `#t`).
- [315283](https://bugs.webkit.org/show_bug.cgi?id=315283):
  per-axis `viewBox` synthesis for an SVG used as `<img>` (sizing of SVG images, different trigger).

The six constraints:

1. Is it upstream's fault?
   Partly. A `#view` fragment on an `<img>` SVG is specified (SVG 2 linking) and Chromium 149 follows it in layout.
   Whether the spec requires the intrinsic ratio to follow the view was not read for this document,
   so that part is unverified.
2. Can upstream fix it?
   Probably, nothing architectural is evident:
   `SVGSVGElement::naturalDimensions()` reads the authored `viewBox()` where `currentViewBoxRect()` exists,
   but whether changing that fixes the failing path is untested.
3. Do they support the use case?
   The fragment path in `SVGImage` and the related reports listed here show a stated interest in fragments on SVG images.
4. Would the repo welcome the contribution?
   `webkit.org/reporting-bugs/` asks for the latest build, a Bugzilla account, a duplicate search,
   and sends Safari-only bugs to Feedback Assistant.
   Neither it nor `webkit.org/contributing-code/` mentions AI-assisted reports (fetched 2026-10-20),
   so no ban was found.
5. Will they likely fix it?
   No signal against.
   Bugs 322167 and 326931 were filed in August and October 2026, so fragment handling is being worked on.
6. Prototyped a minimal fix?
   No.
   The eager and lazy split is unexplained, so a change to `naturalDimensions()` alone might not touch the failing path.
   The root cause is not established and Technology Preview 252 was not matched to a source revision,
   so the audit is incomplete rather than failed.

Decision: do not file.
The draft is kept as an auditable record; it needs the cause of the eager and lazy split and a run on the latest
build before it is fileable.

~~~md
<!-- DRAFT for bugs.webkit.org, product WebKit, component SVG. Do not file as-is: cause of the lazy/eager split unknown. -->
Title: SVG image with a #view fragment is laid out at the whole canvas's aspect ratio when the img has loading="lazy"

With `art.svg` (root `viewBox="0 0 400 200"`, `<view id="v" viewBox="150 50 100 100"/>`) and
`<img src="art.svg#v" loading="lazy">` in a 400 px wide block with `block-size: auto`,
Safari Technology Preview 252 lays the box out 400 by 200 and draws the view letterboxed in it.
Without `loading="lazy"` the box is 400 by 400. Chromium 149 gives 400 by 400 in both cases.
`<picture><source srcset="art.svg#v">` behaves the same as `<img src>`.
With several images of one SVG file on a page, a lazy image can also change the box of eager images
(whole-canvas ratio on all five) and a no-fragment image of the same file took the view's ratio.

Source read at d1eca9768abe: SVGSVGElement::naturalDimensions() (SVGSVGElement.cpp:830) takes the aspect ratio from
the authored viewBox(), while SVGImage::applyDocumentState() (SVGImage.cpp:248) applies the fragment at draw time.
This does not by itself explain why eager images lay out at the view's ratio.

Related: bug 322833 FIXME in SVGImage.cpp:241 (one container's state per resource document), bugs 163812, 322167, 326931.
Reduced test case: art.svg and the page from the troubleshooting document.
~~~
