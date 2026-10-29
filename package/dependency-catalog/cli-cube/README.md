# `@monochromatic-dev/dependency-catalog-cli-cube`

Renders an interactive 3D scatter of every entry in the `pnpm-workspace.yaml` catalog.
 Each catalog package becomes a glyph positioned in a 6-dimensional feature space (3 spatial + color + shape + size);
 the audit-target octant,
 small + stale + small-footprint + non-TS + leaf + niche,
 is the corner you scroll the camera toward.

Naming note:
 the package name preserves user's original `depsUpset` intent.
 The actual visualisation is a 3D scatter rendered via deck.
gl (WebGL),
 not an UpSet plot.
 UpSet binarises continuous attributes that are more useful at full resolution.
 See `../../../doc/decision/deps-cube.md` for the design rationale.

## Usage

```sh
# bin entry
deps-cube

# mise task
mise run //package/dependency-catalog/cli-cube:run
```

Output:
 `<package>/dist/deps-cube-<YYYY-MM-DDTHH-MM-SSZ>.html` (filename carries ISO 8601 UTC down to seconds,
 with `:` rewritten as `-` for filesystem safety;
 each run produces a distinct artifact).
 The package directory is located by walking up from `import.meta.dirname`
 until this package's own `package.json` is found,
 so the path is the same in source mode (`node src/cli.ts`) and built mode (`node dist/final/node/cli.mjs`).
 The package's `dist/` is gitignored.
 Stdout prints exactly `Saved to <abs-path>`.
 Open the HTML in any modern browser (Firefox ESR 140+ baseline).

## Dimensions

Default mapping (changeable via the in-page dim picker):

- **x = log(source bytes)**:
   replacement cost
- **y = log(days since last commit)**:
   staleness
- **z = log(install size,
   transitive)**:
   actual footprint
- **color**:
   TS ratio,
   interpolated in OKLCH from red (low) through amber (mid) to green (high)
- **shape**:
   sphere (leaf) / octahedron (non-leaf)
- **size**:
   log(weekly downloads)

Available additional dims selectable per channel:

- TS ratio (continuous 0 to 1)
- Runtime dep count
- Transitive dep count
- Package age (days since first publish)
- License class (permissive / copyleft / non-OSS / unknown)

## Control panel

- **6 dim dropdowns**:
   remap any data attribute to any channel (type-filtered)
- **7 three-state toggles** (must / must-not / any) for boolean attributes:
   leaf,
   TS-majority,
   large,
   recent,
   permissive,
   copyleft,
   has-known-GH-repo
- **6 range sliders**:
   fine-grained continuous filtering per active channel
- **Name search box**:
   substring / regex
- **Display toggles**:
   coordinate planes,
   threshold guide lines,
   name labels,
   unknown cluster
- **Visibility counter**:
   `X of 120 visible`
- **Reset filters** button

Camera (deck.
gl `OrbitController`):
 drag rotates,
 shift-drag pans,
 scroll zooms,
 double-click resets.

Click any glyph to pin a tooltip beside the chart;
 click background to unpin.

URL hash encodes view state.
 Copy URL to share the exact camera + dim mapping + filter view.

## Cache

`~/.cache/monochromatic/deps-cube/<name>@<version>.json` per catalog entry.
 TTL:
 indefinite for language/SLOC bytes (immutable per published version);
 30 days for last-commit data.

To force refresh of one entry:
 `rm ~/.cache/monochromatic/deps-cube/<name>@*.json`.

## Data acquisition

- npm registry `https://registry.npmjs.org/{name}/{version}`:
   repository,
   dependencies,
   dist.
  unpackedSize,
   license,
   first-publish time
- npm downloads `https://api.npmjs.org/downloads/point/last-week/{name}`:
   weekly download count
- `gh api repos/{owner}/{repo}/languages`:
   Linguist bytes per language
- `gh api repos/{owner}/{repo}` or `gh api 'repos/.../commits?path={directory}'`:
   maintenance signal (path-scoped for monorepo-housed packages)

Packages without a parseable GitHub `repository.url` (gitlab-hosted,
 private,
 missing) render with their TS/size/maintenance dims marked unknown.
 Their glyph lives in a separate "unknown" cluster at the edge of the scene.

## Verification checklist (manual)

1. Run `deps-cube`;
    confirm stdout is exactly one line.
2. Open the HTML in Firefox;
    confirm initial render:
    coordinate planes,
    arrowed axes with tick marks,
    ~120 glyphs,
    unknown cluster,
    control panel.
3. Drag to rotate;
    shift-drag to pan;
    scroll to zoom;
    double-click to reset.
4. Change a dim dropdown;
    confirm glyphs reposition without page reload.
5. Drag a range slider;
    confirm filtered-out glyphs fade to 5% opacity;
    visibility counter updates.
6. Type in the search box;
    confirm only matching glyphs are full-opacity.
7. Set the three-state toggles to identify the audit-target pattern
    (TS-majority must-not / Leaf must / Recent must-not);
    confirm visibility counter shrinks to the audit target list.
8. Hover a glyph;
    confirm tooltip shows pkg name + every dim value.
9. Click a glyph;
    confirm pinned tooltip appears.
10. Toggle threshold guide lines on/off;
     confirm visual update.
11. Copy URL;
     reload;
     confirm view restored.
