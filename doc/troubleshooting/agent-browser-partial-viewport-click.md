# agent-browser 0.36.0: a partly visible download button has an off-screen click point

## Symptom

The Promise lesson's `download #download-reference <path>` command timed out:

```text
Operation timed out. The page may still be loading or the element may not exist.
```

The page was complete,
the button existed,
and its containing disclosure was open.
Process `proc_a208` had passed the chat and workshop checks before reaching this download.
The failure was not evidence that the download listener or generated file was broken.

## Root cause

Inspected release:
`v0.36.0`,
commit `eb05921bad874cd2a1b4fa5d1149f1ed26576cae`.
The read-only source clone is `~/temp/agent/agent-browser-quality-explorer-2026-09-06`.
Paths in this section are relative to that clone.

`cli/src/native/actions.rs:6722` defines `handle_download`.
It configures download handling,
subscribes to events,
then calls `interaction::click`.
`cli/src/native/interaction.rs:37` obtains the point from `resolve_element_center`:

```rust
// cli/src/native/interaction.rs
let (x, y, effective_session_id) = resolve_element_center(
```

The CSS-selector path in `cli/src/native/element.rs:763` generates this visibility test:

```javascript
// JavaScript emitted by cli/src/native/element.rs
const inView = (r) => r.width > 0 && r.height > 0 &&
    r.bottom > 0 && r.right > 0 &&
    r.top < (window.innerHeight || document.documentElement.clientHeight) &&
    r.left < (window.innerWidth || document.documentElement.clientWidth);
```

It scrolls only when that test fails,
but subsequently chooses the rectangle's center:

```javascript
// JavaScript emitted by cli/src/native/element.rs
if (!inView(rect)) {
    el.scrollIntoView({ block: 'center', inline: 'center', behavior: 'instant' });
    rect = el.getBoundingClientRect();
}
const x = rect.x + rect.width / 2;
const y = rect.y + rect.height / 2;
```

The measured button rectangle after the failed command was:

- Top:
  `565.140625` CSS pixels.
- Height:
  `48` CSS pixels.
- Bottom:
  `613.140625` CSS pixels.
- Viewport height:
  `577` CSS pixels.

The rectangle intersects the viewport,
so `inView` is true.
Its center is at `589.140625`,
outside that viewport.
`document.elementFromPoint()` at the measured center returned no element.
The visibility predicate and the chosen interaction point answer different questions.

## Verification and workaround

Working catalog:

- `scrollintoview '#download-reference'` before the native download command.
- Read the resulting rectangle and assert that its center hits the intended element.
- The revised driver applies this to native clicks and downloads.
  `proc_28b9` passed the whole advanced suite,
  downloads,
  and an unfiltered empty-error check.
- `proc_f17b` reopened and exercised those downloaded files.

Failing catalog:

- The partially visible button in the default `1280 × 577` content viewport.
- Inferring a failed application listener from the tool's generic timeout.

Runnable local implementation:
`~/temp/agent/promises-revision/verify-advanced.mjs`,
invoked by `mise run test:advanced`.
The additional hit check is:

```javascript
// In the target document, after scrollintoview.
const element = document.querySelector("#download-reference");
const rect = element.getBoundingClientRect();
element.contains(document.elementFromPoint(
  rect.x + rect.width / 2,
  rect.y + rect.height / 2
));
```

Tradeoff:
explicit scrolling changes viewport position.
It does not establish a general fix for fixed off-screen elements,
wrapped inline fragments,
overlays,
or every frame coordinate system.
Those inputs were not substituted for this measured button.

## Verified selector-geometry prototype

A fresh private clone at `~/temp/agent/upstream-prototype.EsTqN4`
was checked against the origin URL and release commit before editing.
The installed tool and lesson were not patched.

`mise run probe:click-prototype`,
from `~/temp/agent/promises-revision/`,
compiles the actual Rust `build_selector_js` emitter and `BLOCKER_AT_JS` constant
before and after the change.
Only selector lookup is replaced with a fixture lookup for `#target`.
Compilation uses `rust:1.97-bookworm` with no network,
no proxy environment,
a 2 GiB memory limit,
and 2 CPUs.
The generated JavaScript computes points in a real disposable browser;
actual mouse move/down/up commands exercise those points.

Process `proc_9b2e` passed:

```text
PASS geometry fully visible
PASS geometry bottom intersection
PASS geometry right intersection
PASS geometry fixed bottom intersection
PASS geometry fixed top intersection
PASS geometry fixed left intersection
PASS geometry corner intersection
PASS geometry oversized fixed rectangle
PASS geometry wholly offscreen but scrollable
```

Every named intersection case and the oversized fixed rectangle missed before the patch and hit afterward.
The fully visible and scrollable cases hit in both versions.
The driver also asserted an empty page-error record.
The first setup attempt stopped before editing because `/home` and `/var/home` were compared without canonicalization;
that harness check was corrected with `realpathSync`.

The prototype chooses the center of the visible intersection when one exists:

```diff
--- a/cli/src/native/element.rs
+++ b/cli/src/native/element.rs
@@ -779,8 +779,14 @@ fn build_selector_js(selector: &str) -> String {
                 el.scrollIntoView({{ block: 'center', inline: 'center', behavior: 'instant' }});
                 rect = el.getBoundingClientRect();
             }}
-            const x = rect.x + rect.width / 2;
-            const y = rect.y + rect.height / 2;
+            // A visible intersection does not guarantee that the full box's center is visible.
+            const left = Math.max(0, rect.left);
+            const right = Math.min(document.documentElement.clientWidth || window.innerWidth, rect.right);
+            const top = Math.max(0, rect.top);
+            const bottom = Math.min(document.documentElement.clientHeight || window.innerHeight, rect.bottom);
+            const hasVisibleArea = right > left && bottom > top;
+            const x = hasVisibleArea ? (left + right) / 2 : rect.x + rect.width / 2;
+            const y = hasVisibleArea ? (top + bottom) / 2 : rect.y + rect.height / 2;
             const blockerAt = {BLOCKER_AT_JS};
             return {{ x: x, y: y, blocker: blockerAt(document, el, x, y) }};
         }})()"#,
```

Coverage limits:
this compiles and exercises the changed emitter,
not the whole CLI.
It does not test every selector parser,
ref path,
iframe,
clipping ancestor,
or wrapped inline element.
A wholly unreachable rectangle retains its previous fallback behavior.
The consumer's explicit-scroll workaround remains in use regardless of this prototype.

## What does not work

- Treating intersection with the viewport as proof that the rectangle's center is visible.
- Treating command submission as proof that the button received the click.
- Changing lesson download code to compensate for an off-screen input point.
- Classifying this as the wrapped-inline-element problem without measuring client rectangles.

## Upstream filing decision

Initial conjunctive searches returned no matches.
Broader issue searches for `scroll` and PR searches for `click` found related work.
The bodies and comments of [issue #1044][issue],
[PR #1073][scroll-pr],
and [PR #1756][fragment-pr] were read in full.

The current source already has conditional scrolling.
The older wholly-off-screen report is not proof that this version lacks it.
The wrapped-inline PR concerns another geometry problem and is not treated as this incident's fix.

1.  Fault:
    the inspected CSS-selector predicate permits the measured off-screen center.
2.  Fixability:
    scrolling and point selection are implemented upstream;
    the related PR proposes changes at that boundary.
3.  Supported use:
    native clicks and downloads are documented commands.
4.  Contribution policy:
    no contribution file or issue/PR template exists in the inspected clone.
    `AGENTS.md` addresses AI contributors;
    a tracker search for `AI-generated` found no matching policy report.
5.  Direction:
    collaborator responses on PR #1073 support improving scroll-before-interaction behavior.
6.  Prototype:
    the changed Rust geometry emitter was compiled and its generated points were exercised
    before and after the patch as recorded in `Verified selector-geometry prototype`.
    PR #1073 itself was not applied or independently verified.

The skill-relative `.out-of-scope/` directory was absent.
No new issue is warranted for this related symptom.
The partial-intersection evidence is additional to the wholly-off-screen case discussed in issue #1044.
The comment draft remains local;
external posting has not been authorized.
The local workaround does not depend on upstream acceptance.

### Additive comment draft

~~~md
There is a bounded partial-visibility case in v0.36.0 which is distinct from the older absence of scrolling.

In `cli/src/native/element.rs`, `build_selector_js` checks whether any part of the rectangle intersects
with the viewport, but then dispatches at the entire rectangle's center.
A button at top `565.140625`, height `48`, in a `577`-pixel viewport passes that intersection check;
its center is at `589.140625`, outside the viewport.
A download driven by that selector timed out without the button receiving the click.
Explicit `scrollintoview` followed by the native download command worked.

A local prototype computes the center of the viewport-visible intersection instead.
The actual Rust emitter was compiled before and after the change,
with only selector lookup replaced by `document.querySelector("#target")`.
Its generated points were exercised using real browser mouse events.
Bottom, right, fixed bottom/top/left, corner, and oversized fixed cases missed before and hit afterward.
Fully visible and wholly off-screen-but-scrollable controls continued to work.

This is component verification, not a full CLI regression run.
It does not validate wrapped inline elements, every frame/ref path, or wholly unreachable rectangles.
The installed CLI was not modified.
The source trace and tests were AI-assisted and executed by automation;
no human manual verification is claimed.
~~~

[issue]: https://github.com/vercel-labs/agent-browser/issues/1044
[scroll-pr]: https://github.com/vercel-labs/agent-browser/pull/1073
[fragment-pr]: https://github.com/vercel-labs/agent-browser/pull/1756
