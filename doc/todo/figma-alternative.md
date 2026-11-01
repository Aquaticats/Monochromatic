# Figma alternative

Long-term goal:
 a collaborative design tool built entirely in TypeScript with native DOM rendering.

## Architectural constraints

- **DOM/SVG rendering only**:
   no Canvas,
   WebGL,
   or WebGPU
  - Native text rendering,
     native accessibility,
     native CSS layout,
     native hit testing
  - Performance ceiling is lower than Canvas-based tools but rendering fidelity is exact
  - Investigate CSS `contain`,
     `content-visibility`,
     and virtual scrolling for large documents
- **TypeScript only**:
   no Clojure,
   no Rust WASM,
   no C++
  - Entire stack in one language:
     client,
     server,
     collaboration engine,
     build tools
  - Leverages the existing Monochromatic ecosystem (module-es,
     build-css,
     design tokens)
- **Real-time multiplayer**:
   concurrent editing with live cursors and presence

## Major subsystems

### Collaboration engine

- [ ] Evaluate Yjs vs Automerge for CRDT foundation
- [ ] Design document model as CRDT-compatible shared types
- [ ] WebSocket relay server (Bun) for real-time sync
- [ ] Presence system (cursors,
       selections,
       user avatars)
- [ ] Offline support with automatic reconnection and merge
- [ ] Undo/redo that respects collaborative context (local undo,
       not global)
- [ ] Room/document management (who is editing what)
- [ ] Investigate Liveblocks open-source sync engine as alternative to rolling own server

### Document model

- [ ] Define core object types (frame,
       group,
       rectangle,
       ellipse,
       line,
       text,
       vector path,
       image,
       component,
       instance)
- [ ] Tree structure (pages > frames > layers,
       arbitrary nesting)
- [ ] Property system (fills,
       strokes,
       effects,
       blend modes,
       opacity,
       constraints)
- [ ] Component/instance model with overrides
- [ ] Design token integration (colors,
       typography,
       spacing from Monochromatic design system)
- [ ] Serialization format (JSON-based,
       versionable,
       diffable)

### Rendering engine (DOM/SVG)

- [ ] Map document model to DOM/SVG elements
- [ ] Viewport with pan and zoom (CSS transforms on a container,
       not re-rendering)
- [ ] Virtualization;
       only render objects within the visible viewport
- [ ] CSS `contain: strict` and `content-visibility: auto` for off-screen layers
- [ ] SVG for vector shapes,
       native DOM for text,
       `<img>` for rasters
- [ ] Blend modes via CSS `mix-blend-mode`
- [ ] Effects (shadows,
       blurs) via CSS filters
- [ ] Measure performance ceiling:
       how many DOM nodes before degradation,
       on target hardware
- [ ] Investigate `display: none` vs `visibility: hidden` vs removal for hidden layers

### Layout engine

- [ ] Constraint-based positioning (pin to edges,
       center,
       stretch)
- [ ] Auto-layout (flexbox-like) for frames;
       map directly to CSS flexbox since rendering is DOM
- [ ] Responsive resize behavior
- [ ] Absolute positioning within auto-layout frames (the "absolute position" toggle Figma has)
- [ ] Grid layout (CSS grid mapping)
- [ ] Investigate whether CSS layout itself can be the layout engine (since rendering is already DOM,
       let the browser do layout natively instead of reimplementing a solver)

### Vector editing

- [ ] Pen tool for Bezier path creation and editing
- [ ] Research vector networks (Figma's innovation over traditional paths):
       no known open-source implementation exists
- [ ] Boolean operations (union,
       subtract,
       intersect,
       exclude):
       investigate existing JS libraries or build on top of SVG clipPath/mask
- [ ] Path simplification and smoothing
- [ ] SVG import/export

### Text editing

- [ ] Rich text editing within design objects
- [ ] Leverage `contenteditable` or an existing editor (Tiptap,
       Lexical) since rendering is DOM;
       this is a major advantage over Canvas-based tools
- [ ] Typography controls (font family,
       size,
       weight,
       line height,
       letter spacing,
       paragraph spacing)
- [ ] Auto-sizing text frames (fixed width,
       auto width,
       auto height)
- [ ] Text styles as reusable tokens
- [ ] Web font loading

### Interaction and tools

- [ ] Selection tool (click,
       marquee,
       deep select)
- [ ] Move,
       resize,
       rotate handles
- [ ] Snapping and alignment guides (smart guides)
- [ ] Keyboard shortcuts
- [ ] Multi-select and group operations
- [ ] Copy/paste (internal and cross-document)
- [ ] Drag-and-drop from asset panel
- [ ] Zoom to fit,
       zoom to selection
- [ ] Hand tool (panning)
- [ ] Ruler and grid overlays
- [ ] Pixel-snapping toggle

### Component system

- [ ] Component definition (master component)
- [ ] Instance creation with override tracking
- [ ] Variant system (component sets with switchable properties)
- [ ] Nested instance overrides
- [ ] Component swap
- [ ] Detach instance
- [ ] Asset library panel

### Export and interop

- [ ] Export to PNG,
       SVG,
       PDF
- [ ] CSS code generation from design objects (trivial since internal representation is already DOM/CSS)
- [ ] Figma file import (.
      fig format,
       if documentation exists,
       or via Figma REST API)
- [ ] Sketch import (.
      sketch files are JSON in a zip)
- [ ] Copy as SVG,
       copy as CSS

### Prototyping

- [ ] Frame-to-frame navigation links
- [ ] Transition animations (CSS transitions/animations since rendering is DOM)
- [ ] Scroll behavior (overflow scrolling on frames)
- [ ] Presentation mode
- [ ] Hotspot overlays
- [ ] Device frame previews

### Infrastructure

- [ ] Authentication and user management
- [ ] Project/file organization
- [ ] Version history (snapshots,
       branching)
- [ ] Comments and annotations
- [ ] Asset library management (shared across files)
- [ ] File thumbnails
- [ ] Search across files and layers
- [ ] Permissions (view,
       edit,
       admin)

## Open questions

- Can DOM/SVG rendering scale to documents with 10,000+ objects,
   or is there a practical ceiling where the approach falls apart?
  Benchmark early.
- Is CSS layout (flexbox/grid) sufficient as the auto-layout engine,
   or will edge cases require a custom solver?
  Since rendering is DOM,
   using the browser's own layout engine is the simplest path,
   but Figma's auto-layout has behaviors that do not map 1:1 to CSS flexbox.
- Vector networks:
   implement from scratch based on the 2020 blog post describing the data structure,
   or start with traditional Bezier paths and add vector networks later?
- Should the document format be a single JSON blob or a normalized structure (closer
  to the file-per-property idea from earlier discussion)?
  Likely JSON for persistence,
   CRDT shared types in memory for live editing.
- What is the minimum viable product?
  Probably:
   frames,
   rectangles,
   text,
   selection/move/resize,
   real-time collaboration,
   export to PNG.
  Skip components,
   prototyping,
   and vector editing for v1.

## Prior art to study

- **Penpot**:
   open-source Figma alternative (Clojure/ClojureScript,
   SVG rendering),
   study architecture decisions and what works/does not
- **Excalidraw**:
   open-source whiteboard with Yjs collaboration,
   clean TypeScript codebase,
   good reference for multiplayer architecture
- **tldraw**:
   open-source infinite canvas library with multiplayer,
   well-designed shape/tool system
- **WeaveJS** (InditexTech):
   open-source library for real-time collaborative Canvas apps
- **Liveblocks**:
   recently open-sourced sync engine,
   potential collaboration infrastructure
- **Figma engineering blog posts**:
   vector networks,
   multiplayer architecture,
   rendering pipeline
