# agent-browser 0.38.1 selector click can miss a partly visible target's off-viewport center

## Symptom and incident boundaries

The offline first-run evidence verifier received a successful native
`click` command,
but the image dialog stayed closed.
The failed frame was the cover/declined/light/100% preview after changing
from inner-panel images.
The browser reported no JavaScript or console error;
the active element remained the body.
A screenshot showed only the target button's lower portion at the viewport top.

A separate minimal control reproduced the same class of missed input:

```text
click #target: ✓ Done
trusted target clicks: 0
```

This is separate from Android capture,
source access or notification design.
It is also separate from the diagnostic attempt that returned the entire
body's `outerHTML`,
including embedded images,
and exceeded the verifier's output buffer.
Node 24.20.0 emitted `spawnSync /opt/agent-browser/bin/agent-browser-linux-x64 ENOBUFS`.
That diagnostic was replaced with bounded tag/ID/geometry fields.

## Deciding source

The read-only `vercel-labs/agent-browser` tag `v0.38.1` was inspected at
`aff6125c023b810ea3f2e5deec5379e9a4270bdc`.
Installed-binary/source byte equivalence was not established.
The installed CLI reported 0.38.1 and reproduced the input failure.

`cli/src/native/interaction.rs:44` calls `resolve_element_center`,
then dispatches trusted pointer input at its returned coordinates.
For root CSS selectors,
`cli/src/native/element.rs:938` to `955` considers any rectangle overlap
sufficient to skip scrolling,
then chooses the whole rectangle's center:

```js
// cli/src/native/element.rs, JavaScript emitted by build_selector_js
const inView = (r) => r.width > 0 && r.height > 0 &&
    r.bottom > 0 && r.right > 0 &&
    r.top < (window.innerHeight || document.documentElement.clientHeight) &&
    r.left < (window.innerWidth || document.documentElement.clientWidth);
let rect = el.getBoundingClientRect();
if (!inView(rect)) {
    el.scrollIntoView({ block: 'center', inline: 'center', behavior: 'instant' });
    rect = el.getBoundingClientRect();
}
const x = rect.x + rect.width / 2;
const y = rect.y + rect.height / 2;
```

A rectangle can intersect the viewport while its center lies outside it.
The source's blocker helper at line 911 treats no hit as no blocker:

```js
// cli/src/native/element.rs, BLOCKER_AT_JS
if (!hit || hit === el) return null;
```

The minimal control measured a rectangle at x16/y−180,
288px wide and 200px high.
Its bottom was 20px inside the viewport,
but its center was y−80.
The CLI returned success with zero target callbacks.
The visual viewport offset was zero,
so issue 1971's nonzero-offset explanation does not explain this control.
The source shape and measured control establish this root-selector input
failure,
not the cause of every superficially similar missed click.

## Verification

Checks ran in an owned offline Chromium container capped at 2GiB/2CPU.
The minimal target was deliberately partly visible,
with an observable counter and `event.isTrusted` capture.
After the missed implicit click,
explicit `scrollintoview` moved it to y400 through y600.
The next native click produced exactly one trusted target callback.

A minimal page can reproduce the control:

```html
<!-- partial-target-control.html -->
<!doctype html>
<html lang="en">
<head><title>Partial target control</title>
<style>
body { min-block-size: 3200px; margin-block: 0; margin-inline: 0; }
button { display: block; margin-block-start: 800px; inline-size: 288px; block-size: 200px; }
</style></head>
<body><button id="target">Count target clicks</button>
<script>
let clicks = 0;
let trusted = false;
document.getElementById('target').addEventListener('click', event => {
  clicks++;
  trusted = event.isTrusted;
});
</script></body></html>
```

```console
# partial-target-control.html, owned browser session only
agent-browser --session partial-target --pin-tab open file:///absolute/path/partial-target-control.html
agent-browser --session partial-target set viewport 1440 1000
agent-browser --session partial-target eval 'window.scrollBy(0, document.getElementById("target").getBoundingClientRect().top + 180)'
agent-browser --session partial-target click '#target'
agent-browser --session partial-target eval 'JSON.stringify({clicks, trusted})'
agent-browser --session partial-target scrollintoview '#target'
agent-browser --session partial-target click '#target'
agent-browser --session partial-target eval 'JSON.stringify({clicks, trusted})'
agent-browser --session partial-target close
```

The complete first-run evidence verifier also passed after explicit
center scrolling before preview clicks:
34 distinct images decoded/opened,
four responsive/theme contexts,
optional observations and representative modal/zoom/pan/focus controls.
The page HTML and native images were not changed by this workaround.

## Verified consumer workaround

Use explicit center scrolling before a selector click and verify the
resulting application state,
not merely the CLI exit status.
The source's `scroll_into_view` at
`cli/src/native/interaction.rs:835` unconditionally centers the target.

This keeps trusted native input but may change page scroll position.
It does not fix visual-viewport offsets,
iframe coordinates,
blocking overlays or every unreachable target.
The passing control had a reachable center and zero visual offset.

## Disposable source-compatible prototype

A fresh private,
unpredictable clone was created for the patch.
Its origin and exact tag revision were checked;
push was disabled.
The read-only inspection clone was not edited.
The [prototype patch](agent-browser-partial-target-center.patch) changes
only the root CSS-selector point calculation:
use the center of the visible rectangle intersection and return no point
for an empty intersection.
The existing scroll and blocker machinery stays in place.

The least-trusting harness extracted the JavaScript emitted by that
source function before and after the patch,
executed it in Chromium,
then used trusted native mouse events at the returned point.
Ten cases covered a full target,
partial top/bottom/left/right,
a corner,
an oversized target,
hidden input and zero width/height.
The original source missed all partial-edge target callbacks.
The patched emission produced one trusted callback for every eligible
case and no point/callback for every ineligible case.

This verifies the changed emitted program and its input coordinates.
A Rust binary rebuild,
upstream full suite,
iframe paths,
visual-offset cases and complete release acceptance were not performed.
The prototype is not installed as the local workaround.

## What did not work

- Treating `✓ Done` as target activation failed the minimal control.
- Assuming any rectangle overlap made its whole center visible was
  disproved by y−180 through y20 geometry.
- Issue 1971 describes mobile visual-viewport offset and already has a
  separately tested patch;
  the zero-offset desktop control excludes that hypothesis here.
- Increasing the body-HTML diagnostic buffer was unnecessary;
  bounded fields avoid dumping all embedded image data.

## Upstream filing decision

The `.out-of-scope/` census had no agent-browser exemption.
The related issue 1971 and all its comments were read;
it was open with no comments at inspection.
`partially visible click` issue search and
`partial click viewport center` PR search returned no matches.
Those exact empty queries do not establish universal tracker absence;
issue 1971 supplies the comparable case.

- Fault:
  current installed input reports success without activating a partly
  visible target in the zero-offset control.
- Ability:
  the root selector's generated point calculation is separable from
  trusted input dispatch;
  no architectural impossibility was found.
- Supported input:
  selectors and partly scrolled pages are existing supported surfaces.
- Contribution policy:
  upstream `AGENTS.md` explicitly addresses AI coding agents and requires
  source,
  command,
  schema and skill documentation updates when applicable.
  No submission has been made.
- Likely action:
  no refusal of this case was found;
  issue 1971 is related evidence,
  not an upstream decision about this zero-offset case.
- Prototype:
  the source-compatible emitted-program patch and nontrivial controls
  passed,
  with the binary/full-suite limitations stated explicitly.

No issue,
comment or pull request was sent.
A report remains local;
external-submission authorization is separate from the local source audit.
The consuming design review is already verified via the scoped workaround.

### Local issue draft, not sent

Title:
`Selector click reports success when a partly visible target's center is outside the viewport (0.38.1)`.

A zero-offset desktop Chromium control leaves a button at y−180 through y20.
`click #target` reports `✓ Done` but no target callback fires.
`scrollintoview #target` followed by the same click yields one trusted callback.
The root selector helper skips scrolling for any rectangle intersection,
then sends the whole rectangle's off-viewport center.
This differs from issue 1971's nonzero visual-viewport offset case.

A source-compatible prototype against tag `v0.38.1` uses the visible
intersection center and rejects empty intersections.
Before/after emitted-program controls covered the full target,
partial edges,
corner,
oversized target,
hidden target and zero dimensions with trusted native mouse input.
The prototype was not used to build a replacement Rust binary,
and iframe/visual-offset paths were not included.
The reproduction,
source trace,
patch and consumer workaround are recorded in this document.
AI assistance was used for this investigation;
no claim of human verification is made.
This draft remains local and requests no action until separately submitted.
