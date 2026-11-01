# doodle-widget

Generates a single self-contained HTML file with a canvas-based doodling widget.

## Features

- Freehand pencil drawing on an HTML canvas (red,
   10px)
- SVG background support:
   doodle strokes render **behind** SVG paths
- Raster background support (JPEG,
   PNG,
   etc.)
- Upload custom background images via the toolbar
- Ships with a default SVG background (`src/asset/default-bg.svg`)
- Strokes survive window resizes (stored in normalized coordinates)

## How it works

The build script reads the default SVG,
 removes its white background rect,
and inlines everything (HTML via h-html,
 CSS via h-css,
 JavaScript) into a single
`dist/final/index.html` file.

For SVG backgrounds,
 the SVG is rendered as a `pointer-events: none` overlay
above the canvas so that SVG strokes visually sit on top of doodle strokes.
For raster backgrounds,
 the image is set as a CSS background-image beneath
the canvas.

## Build

```sh
mise run //package/doodle-widget/app:build
```

Output:
 `dist/final/index.html`
