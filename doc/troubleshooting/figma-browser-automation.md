# Figma's WebGL canvas blocks DOM-based browser automation; agents see chrome but not design content

## Symptom

An AI agent driving a browser via Playwright,
 Puppeteer,
 BrowserOS,
or any DOM-based automation tool can interact with Figma's UI
**chrome** (toolbars,
 side panels,
 menus,
 layer-list entries) but
cannot read or manipulate anything **inside** the design canvas:

- Screenshots show pixels,
   not structure.
   The agent can save a PNG
  but cannot inspect,
   query,
   or programmatically read frames,
  layers,
   text,
   or interactive elements rendered inside the canvas.
- Clicking inside the canvas requires pixel-coordinate guessing
  from screenshots;
   there are no DOM targets to address.
- Keyboard shortcuts that Figma binds at the canvas level
  (Shift+1 zoom-to-fit,
   Shift+2 zoom-to-selection) cannot be
  triggered by dispatching keyboard events into the DOM;
   the WebGL
  canvas captures input before the DOM event system sees it.
- Prototype previews are also WebGL;
   the inline preview panel
  renders device frames as canvas content,
   not as DOM.
- Comments are rendered in the canvas overlay system and are not
  exposed as DOM elements.

## Root cause

Figma renders the entire design canvas to a single `<canvas>`
element via WebGL.
 The DOM has no representation of frames,
layers,
 text,
 or interactive elements that live on the canvas;
they are GPU-side draw calls,
 not nodes.
 Browser automation
operates on the DOM (and on the layout it produces),
 so it has
no handles to address canvas content.

This is the correct engineering decision for Figma:
 rendering
thousands of vector objects via DOM would be unusable.
 The trap is
that every accessibility and automation API depending on DOM
structure becomes blind to the design content.

Figma's accessibility features (screen-reader mode) provide
limited text alternatives,
 but those alternatives are not
sufficient for design-review automation:
 they expose names but
not geometry,
 transitions,
 or interaction targets.

## Verification

Versions under test:

- Figma web app (production),
   as of 2026-05.
- BrowserOS,
   Playwright,
   Puppeteer;
   all return empty results when
  querying for design-content selectors inside the canvas.

Reproduce:
 open any Figma file via Playwright;
 the page's DOM
includes the toolbar,
 the left sidebar (layer list,
 with the layer
names that Figma maintains separately),
 the right panel,
 and the
canvas `<canvas>` element.
 Inside that single canvas there is no
descendant structure.
 No `querySelectorAll('[data-figma-frame]')`
or similar selector will return canvas content because Figma does
not expose one.

## What each tool can and cannot do

### Figma MCP (REST API based)

Can do:
 read the node tree and metadata;
 render specific nodes as
static PNG;
 read component properties,
 styles,
 variables;
 access
file and page metadata.

Cannot do:
 read Figma comments;
 run or interact with prototypes;
see prototype transitions and interaction hotspots;
 observe
variable-mode switching in real time;
 access annotation or dev-mode
markup.

### Browser automation (BrowserOS, Playwright, Puppeteer)

Can do:
 see and click Figma's toolbar,
 panels,
 menus;
 navigate
between pages in the sidebar;
 open the prototype preview panel;
click "Next frame" / "Previous frame" in the preview;
 take
screenshots of whatever is visible at whatever zoom Figma chose.

Cannot do:
 read any text rendered on the canvas;
 click on
specific design frames or elements;
 reliably zoom to a specific
frame;
 read prototype interaction flows;
 extract design specs
(spacing,
 colours,
 fonts) from the canvas;
 read comments or
annotations.

### Claude-in-Chrome / human-in-the-loop

Best current option for full context.
 The human navigates to the
relevant frame;
 the agent analyses the screenshot visually.
Clunky,
 slow,
 requires manual effort,
 but actually works.

## Verified workarounds

### For design audits

1.  Use Figma MCP for structured node data and static frame
    screenshots.
2.  Have the human manually screenshot specific screens,
     prototype
    flows,
     and comment threads.
3.  Have the human paste comment text directly into the
    conversation.
4.  Combine both data sources in the agent's analysis.

Tradeoff:
 depends on human availability.
 No automated path closes
this gap.

### For prototype review

No automated path.
 The human must run the prototype manually and
either screen-record or screenshot each state.
 Tradeoff:
 agent
cannot iterate prototype review autonomously.

### For comment review

No automated path.
 The human must copy comment text into the
conversation or screenshot threads.
 Tradeoff:
 comments cannot be
correlated to canvas geometry programmatically.

## What does not work

- Sending synthesised keyboard events to trigger canvas shortcuts:
  the canvas captures input before the DOM event system runs.
- Reading the canvas via `canvas.toDataURL()` plus OCR:
   works as a
  fallback for visible text but loses structure (frame
  boundaries,
   layer hierarchy,
   prototype connections).
- Patching Figma's renderer through DevTools:
   Figma's bundle is
  minified and changes frequently;
   any patch breaks on the next
  release.
- Using Figma's accessibility mode as a structure proxy:
   provides
  names but not geometry,
   transitions,
   or interaction targets.

## Why we do not file this upstream

The behaviour is by design and serves Figma's primary use case
(fast vector rendering).
 The 5 constraints:

1.  **Is it really upstream's fault?
    ** Borderline.
     The WebGL
    architecture is correct for Figma's audience.
     The "fault" is
    that the API surface is narrower than design-review automation
    wants.
2.  **Can upstream fix it?
    ** Yes,
     by exposing comments,
     prototype
    flows,
     and dev-mode annotations through the REST/MCP API.
    Architecturally feasible;
     it expands the API contract.
3.  **Are they supporting this use case?
    ** Partially:
     REST/MCP
    covers node geometry and rendering.
     Prototype interaction and
    comments are not covered.
4.  **Will they likely fix it?
    ** Unknown.
     Demand from
    automation-first users is growing but Figma's roadmap has not
    committed.
5.  **Have we prototyped a minimal fix?
    ** N/A;
     closed source.

Decision:
 no upstream report from us.
 Track Figma roadmap for
API expansion to comments,
 prototypes,
 and annotations.
 None of
these features exist in the public API as of 2026-05.

## What would actually fix this

- Figma exposing comments,
   prototype flows,
   and annotation data
  through their REST API or MCP plugin.
- A dedicated Figma MCP that wraps the full API surface
  (comments,
   prototype links,
   dev-mode annotations),
   not just the
  node tree.
- A Figma "headless prototype runner" API that returns frame
  sequences for a given interaction flow.

None exist as of 2026-05.
