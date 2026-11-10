# Aquascope connector lines can disappear in Firefox

Status on 2026-07-29:
connector lines in the experimental Rust Book's opening Aquascope interpreter rendered in Chromium,
but the learner's Firefox view omitted them.
The exact Firefox version was not captured,
and Firefox was unavailable in the verification environment.
This is a browser rendering discrepancy,
not evidence that Aquascope produced a different Rust execution state.

## Symptom

The opening interpreter diagram still shows stack and heap values,
pointer dots,
and crossed-out values,
but curved lines between a pointer dot and its target are absent in Firefox.
The same widget shows those lines in Chromium.

This can make `s` and `hello` look disconnected from the heap bytes they address.
It does not change the underlying interpreter response embedded in the page.

## Affected surface and impact

The finding applies to the Aquascope interpreter widget on the experimental Rust Book introduction,
verified from the page's exact opening embed on 2026-07-29.
That embed requests only the `interpreter` operation and contains the interpreter trace as serialized data.
The browser draws connectors after loading that same data.

Treat each pointer dot as a pointer value and the intended target as the value it addresses.
For the opening example:

- At `L1`,
  `s` points to the heap bytes for `Hello world`.
- At `L2`,
  `hello` points to the first five bytes of that allocation.
- At `L3`,
  `s` points to the replacement allocation containing `Hello world!`;
  `hello` is crossed out because its borrow has ended before mutation.
- At `L4`,
  `drop(s)` has deallocated the buffer,
  so the pointer inside `s` is crossed out.

Use a Chromium browser when seeing connector geometry is necessary.
The pointer dots,
labels,
and crossed-out states remain the authoritative reading fallback when Firefox omits lines.

## Implementation path

Aquascope does not encode connector paths in the Rust interpreter trace.
At Aquascope commit `f2d0a06034b86765a91f10b7c8e40ec31ecb87a6`,
`frontend/packages/aquascope-editor/src/editor-utils/interpreter.tsx` does the following:

1.  Finds rendered `.pointer` elements and their `data-point-to` destinations.
2.  Creates an SVG connector through the workspace package `@aquascope/leader-line`.
3.  Moves generated `.leader-line` SVG elements from `document.body` into the diagram's arrow container.
4.  Calls `line.position()` and translates the arrow container as the diagram moves.

The workspace package vendors a minified LeaderLine implementation in
`frontend/packages/leader-line/src/leader-line.js`.
Aquascope's package metadata identifies its wrapper as `@aquascope/leader-line` version `0.1.0`;
it does not identify the vendored upstream LeaderLine release.

## Root-cause status

The precise Firefox failure mechanism is unconfirmed.
Do not describe a specific SVG,
CSS transform,
clipping,
or stacking bug as the cause without reproducing it in Firefox and inspecting the generated SVG.

The symptom matches archived upstream report
[anseki/leader-line#180](https://github.com/anseki/leader-line/issues/180):
Firefox 82.0.3 inserted LeaderLine SVGs in the expected place but did not paint them,
while Chrome and Safari did.
That issue was opened on 2020-11-26 and is closed without a visible resolution in its public thread.
The `anseki/leader-line` repository was archived on 2025-04-11.
The report establishes precedent,
not proof that the current Aquascope symptom has the same low-level cause.

## Verification

Chromium verification on 2026-07-29 used the rendered experimental Rust Book page,
not a reconstructed sample:

- The opening embed contained four annotated source locations and five interpreter states.
- The rendered widget contained four `.leader-line` SVG elements.
- Computed style for every connector was `display: block`,
  `visibility: visible`,
  and `opacity: 1`.
- The SVG bounding boxes had nonzero dimensions.
- A screenshot showed the connectors joining pointer dots to their targets.
- Expanding internal types and revealing hidden source lines did not remove the connectors.

Firefox evidence is currently the learner's direct observation only.
A complete Firefox diagnosis would need the browser version,
the generated `.leader-line` SVG nodes and computed styles,
and a screenshot from that same page load.

## Resolution and upstream status

The learning material now states that Chromium renders the connector lines and Firefox may omit them.
It tells the reader that missing lines do not represent a different Rust state.
No patch was made to the external experimental Rust Book or Aquascope.

Do not file against the archived LeaderLine repository.
An Aquascope report would be justified only after reproducing the current page in Firefox,
capturing the browser version and generated SVG state,
and reducing the failure to Aquascope's vendored connector path.
