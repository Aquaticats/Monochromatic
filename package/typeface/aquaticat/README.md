# typeface-aquaticat

Custom geometric typeface.
 Regular weight,
 uppercase only (`A` to `Q`,
 `X` to `Z`).

## Build pipeline

1. Letter shapes are authored as SVG paths in a single glyph strip (`src/glyphs.svg`)
2. `src/build-font.ts` parses the strip,
    extracts per-cell paths,
    expands stroked
   outlines into filled contours,
    and assembles an OpenType font via opentype.
   js
3. The OTF is converted to WOFF2 using fonttools (invoked through `uv`)

## Outputs

- `dist/Aquaticat-Regular.otf`:
   OpenType format
- `dist/Aquaticat-Regular.woff2`:
   compressed web font
- `src/aquaticat.css`:
   `@font-face` declaration referencing both formats

## Usage

```css
@import '@monochromatic-dev/typeface-aquaticat/aquaticat.css';

h1 {
  font-family: 'Aquaticat', sans-serif;
}
```
