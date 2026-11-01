# Penpot File Format Research

## Overview

Penpot exports files in a ZIP-based format (binfile-v3) with JSON entries.
The `.penpot` extension is just a ZIP file containing structured JSON data
and binary media objects.

## ZIP Structure

```text
manifest.json
files/{fileId}.json
files/{fileId}/pages/{pageId}.json
files/{fileId}/pages/{pageId}/{shapeId}.json
files/{fileId}/media/{mediaId}.json
files/{fileId}/components/{componentId}.json
files/{fileId}/colors/{colorId}.json
files/{fileId}/typographies/{typographyId}.json
files/{fileId}/tokens.json
files/{fileId}/thumbnails/{tag}/{pageId}/{frameId}.json
files/{fileId}/plugin-data.json
objects/{storageObjectId}.json
objects/{storageObjectId}.{ext}   (binary: png, jpg, svg, woff, etc.)
```

## manifest.json

```json
{
  "type": "penpot/export-files",
  "version": 1,
  "generatedBy": "penpot/2.14.3",
  "referer": "penpot",
  "files": [
    {
      "id": "uuid-v4",
      "name": "File Name",
      "features": ["fdata/path-data", "design-tokens/v1", ...]
    }
  ],
  "relations": []
}
```

## File JSON (`files/{fileId}.json`)

Contains file-level metadata (not the actual design data):

```json
{
  "id": "uuid",
  "name": "File Name",
  "revn": 968,
  "modifiedAt": "2026-04-27T00:05:53.501447Z",
  "createdAt": "2026-04-23T21:31:42.274555Z",
  "isShared": false,
  "hasMediaTrimmed": false,
  "teamId": "uuid",
  "projectId": "uuid",
  "version": 67,
  "features": ["fdata/path-data", "design-tokens/v1", "variants/v1", "layout/grid", "components/v2", "fdata/shape-data-type"],
  "options": {
    "componentsV2": true,
    "baseFontSize": "16px"
  },
  "migrations": ["legacy-2", ..., "0017-fix-layout-flex-dir"]
}
```

Key fields from source (`schema:file` in `common/src/app/common/types/file.cljc`):

- `id`,
   `name`,
   `revn`,
   `vern`,
   `created-at`,
   `modified-at`,
   `deleted-at`
- `project-id`,
   `team-id`,
   `is-shared`,
   `has-media-trimmed`
- `data` (optional):
   the actual design data,
   not stored in file JSON
- `version` (int),
   `features` (feature flags),
   `migrations` (set of strings)

## Page JSON (`files/{fileId}/pages/{pageId}.json`)

```json
{
  "id": "uuid",
  "name": "Page 1",
  "background": "#000000",
  "index": 0
}
```

From source (`schema:page` in `common/src/app/common/types/page.cljc`):

- `id`,
   `name`,
   `background` (hex color string)
- Optional:
   `flows`,
   `options`

## Shape JSON (`files/{fileId}/pages/{pageId}/{shapeId}.json`)

Each shape is a separate JSON file.
 This is the core data model.

### Common fields (all shape types)

```json
{
  "id": "uuid",
  "name": "Shape Name",
  "type": "frame|group|bool|rect|path|text|circle|svg-raw|image",
  "x": 0,
  "y": 0,
  "width": 100,
  "height": 100,
  "rotation": 0,
  "selrect": {
    "x": 0,
    "y": 0,
    "width": 100,
    "height": 100,
    "x1": 0,
    "y1": 0,
    "x2": 100,
    "y2": 100
  },
  "points": [
    { "x": 0, "y": 0 },
    { "x": 100, "y": 0 },
    { "x": 100, "y": 100 },
    { "x": 0, "y": 100 }
  ],
  "transform": { "a": 1.0, "b": 0.0, "c": 0.0, "d": 1.0, "e": 0.0, "f": 0.0 },
  "transformInverse": {
    "a": 1.0,
    "b": 0.0,
    "c": 0.0,
    "d": 1.0,
    "e": 0.0,
    "f": 0.0
  },
  "parentId": "uuid",
  "frameId": "uuid",
  "flipX": null,
  "flipY": null,
  "proportion": 1,
  "proportionLock": false,
  "opacity": 1.0,
  "fills": [],
  "strokes": [],
  "pageId": "uuid"
}
```

### Type-specific fields

**frame**:
 `shapes` (child IDs),
 `hideFillOnExport`,
 `showContent`,
 `hideInViewer`,
 `r1`-`r4` (border radius),
 layout fields (`layout`,
 `layoutGap`,
 `layoutPadding`,
 `layoutFlexDir`,
 `layoutAlignItems`,
 `layoutJustifyContent`,
 `layoutAlignContent`,
 `layoutWrapType`,
 `layoutItemHSizing`,
 `layoutItemVSizing`,
 `layoutGapType`,
 `layoutPaddingType`),
 constraints (`constraintsH`,
 `constraintsV`)

**group**:
 `shapes` (child IDs),
 `maskedGroup` (bool)

**bool**:
 `shapes` (child IDs),
 `boolType` ("union"|"difference"|"exclude"|"intersection"),
 `content` (SVG path data string)

**rect**:
 `r1`-`r4` (individual corner radii)

**circle**:
 no extra fields (x/y/width/height + selrect suffice)

**path**:
 `content` (SVG path data string),
 `growType` ("auto-width"|"auto-height"|"fixed"),
 `strokeCapStart`,
 `strokeCapEnd`

**text**:
 `content` (rich text tree),
 `growType`,
 `positionData` (array),
 text layout fields

**image**:
 `metadata` object with `{ name, width, height, mtype, id }`

**svg-raw**:
 `content` (parsed XML tree `{tag, attrs, content}`),
 `svgAttrs`,
 `svgViewbox`,
 `svgDefs`,
 `svgTransform`

### Fill object

```json
{ "fillColor": "#FFFFFF", "fillOpacity": 1 }
{ "fillColorGradient": { ... gradient object ... }, "fillOpacity": 1 }
{ "fillImage": { "name": "...", "width": 100, "height": 100, "mtype": "image/png", "id": "mediaId" }, "fillOpacity": 1 }
```

### Stroke object

```json
{
  "strokeStyle": "solid|dotted|dashed|mixed",
  "strokeAlignment": "center|inner|outer",
  "strokeWidth": 1,
  "strokeColor": "#000000",
  "strokeOpacity": 1,
  "strokeCapStart": "round|square|line-arrow|triangle-arrow|...",
  "strokeCapEnd": "round|square|..."
}
```

### Text content tree

```json
{
  "type": "root",
  "children": [{
    "type": "paragraph-set",
    "children": [{
      "type": "paragraph",
      "lineHeight": "1.2",
      "fontStyle": "normal",
      "textTransform": "none",
      "textAlign": "left",
      "fontId": "gfont-inter",
      "fontSize": "36",
      "fontWeight": "300",
      "textDirection": "ltr",
      "fontVariantId": "300",
      "textDecoration": "none",
      "letterSpacing": "0",
      "fills": [{ "fillColor": "#ffffff", "fillOpacity": 1 }],
      "fontFamily": "Inter",
      "children": [{
        "text": "Hello World"
      }]
    }]
  }]
}
```

### Transform matrix

2D affine transform as `{a, b, c, d, e, f}`:

```text
| a c e |
| b d f |
| 0 0 1 |
```

Identity = `{a:1, b:0, c:0, d:1, e:0, f:0}`

### Constraints

- Horizontal:
   "left"|"right"|"leftright"|"center"|"scale"
- Vertical:
   "top"|"bottom"|"topbottom"|"center"|"scale"

### Blend modes

"normal"|"darken"|"multiply"|"color-burn"|"lighten"|"screen"|
"color-dodge"|"overlay"|"soft-light"|"hard-light"|"difference"|
"exclusion"|"hue"|"saturation"|"color"|"luminosity"

## Media JSON (`files/{fileId}/media/{mediaId}.json`)

```json
{
  "id": "uuid",
  "name": "image",
  "width": 2880,
  "height": 6,
  "mtype": "image/png",
  "mediaId": "uuid (storage object ref)",
  "thumbnailId": "uuid (storage object ref)",
  "isLocal": true,
  "createdAt": "2026-04-26T20:37:43.686292Z"
}
```

## Component JSON (`files/{fileId}/components/{componentId}.json`)

```json
{
  "id": "uuid",
  "name": "Component Name",
  "path": "",
  "modifiedAt": "2026-04-26T20:42:33.034Z",
  "mainInstanceId": "uuid",
  "mainInstancePage": "uuid"
}
```

Components also contain an `objects` map (shape tree) in the full internal representation,
but this is stripped to just the above fields in the export file JSON.

## Storage Objects (`objects/`)

Binary assets (images,
 fonts) stored as two files:

1. `objects/{id}.json`:
    metadata:
    `{id, size, contentType, bucket, hash}`
2. `objects/{id}.{ext}`:
    the actual binary data (png,
    jpg,
    svg,
    woff,
    woff2,
    ttf,
    otf)

The `hash` field uses format `"blake2b:{hex}"`.
The `bucket` is one of the valid storage buckets (e.g.,
 "file-object-thumbnail",
 "file-media-object").

## Thumbnail JSON

```json
{
  "fileId": "uuid",
  "pageId": "uuid",
  "frameId": "uuid",
  "tag": "frame|component",
  "mediaId": "uuid (storage object ref)"
}
```

## Design Tokens (`files/{fileId}/tokens.json`)

Standard DTCG format:

```json
{
  "Global": {
    "tokenName": { "$value": "#05ce78", "$type": "color", "$description": "" }
  },
  "$themes": [],
  "$metadata": {
    "tokenSetOrder": ["Global"],
    "activeThemes": [],
    "activeSets": ["Global"]
  }
}
```

## Color Library (`files/{fileId}/colors/{colorId}.json`)

Not present in the sample file.
 From source (`schema:library-color`):

- `id`,
   `name`,
   `path`,
   `color` (hex),
   `opacity`
- Optional gradient fields
- `file-id`,
   `created-at`,
   `modified-at`

## Typography Library (`files/{fileId}/typographies/{typographyId}.json`)

Not present in the sample file.
 From source (`schema:typography`):

- `id`,
   `name`,
   `path`,
   `font-id`,
   `font-family`,
   `font-variant-id`
- `font-size`,
   `font-weight`,
   `font-style`,
   `line-height`,
   `letter-spacing`
- `text-transform`,
   `text-decoration`

## JSON Key Convention

Penpot uses camelCase in its JSON export (via `json/write-camel-key`).
On import,
 keys are read as kebab-case (via `json/read-kebab-key`) and then decoded.
So the JSON files use camelCase:
 `parentId`,
 `frameId`,
 `fillColor`,
 etc.
Internally Penpot uses kebab-case:
 `:parent-id`,
 `:frame-id`,
 `:fill-color`,
 etc.

## Source Code References

- Export:
   `backend/src/app/binfile/v3.clj`:
   `write-entry!`,
   `export-file`,
   `export-files`
- Import:
   `backend/src/app/binfile/v3.clj`:
   `read-entry`,
   `import-file`,
   `import-storage-objects`
- Shape schema:
   `common/src/app/common/types/shape.cljc`
- File schema:
   `common/src/app/common/types/file.cljc`
- Page schema:
   `common/src/app/common/types/page.cljc`
- Color schema:
   `common/src/app/common/types/color.cljc`
- Component schema:
   `common/src/app/common/types/component.cljc`
- Cleaner:
   `backend/src/app/binfile/cleaner.clj`:
   pre/post decode fixes
- Common:
   `backend/src/app/binfile/common.clj`:
   `file-attrs`,
   shared utilities

## Conversion strategy

### .fig files (design)

Each CANVAS node becomes a Penpot page.
 CANVAS nodes named
"Internal Only Canvas" (with `internalOnly: true`) are skipped.
Child frames,
 groups,
 text,
 paths,
 etc. become Penpot shapes.

### .deck files (presentations)

Each SLIDE node becomes a Penpot page.
 The CANVAS,
 SLIDE_GRID,
and SLIDE_ROW structural nodes are skipped (they are just
Figma-internal containers).
 A deck with 10 slides produces
10 Penpot pages.

### .jam files (whiteboards)

Same as .
fig:
 each CANVAS becomes a page.
 STICKY notes are
converted as frames (Penpot has no sticky note type).
SECTION,
 VECTOR,
 and ROUNDED_RECTANGLE nodes map normally.

### Paint type resolution

Figma Paint objects have both `__type` (schema type name,
 e.g. "Paint")
and `type` (enum value,
 e.g. "PaintType.
SOLID").
 Always check `type`
first for the paint variant,
 not `__type`.
