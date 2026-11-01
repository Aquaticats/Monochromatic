# aquati.cat

Static site generator for the Aquaticat blog.
Converts MDX content into flat HTML pages served by Caddy with clean URLs.

## Architecture

The build pipeline runs as a sequence of mise tasks (`mise run build`):

1. **Client JS** (`build:js:client`):
    bundle client-side scripts via rolldown.
    Localized UI text is rendered at build time via `@monochromatic-dev/i18n-compose-core`
    (no codegen step),
    with per-locale label tables in `src/i18n/`
2. **Site generation** (`build:site` / `src/build.ts`):
    loads MDX from `src/content/{lang}/`,
    validates frontmatter with Valibot,
    processes changed files through a remark/rehype pipeline (with SHA-256 content caching),
    pre-computes syntax highlight ranges via Lezer,
    generates HTML pages from h-html templates,
    generates CSS from h-css declarations,
    generates RSS feeds per language via feedsmith,
    copies static assets from `public/`
3. **Post-processing** (`build:postprocess` / `src/build/postprocess.ts`):
    pagefind indexes `dist/` in parallel with fingerprint phases 1+2 (leaf assets + CSS);
    phase 3 (HTML reference rewriting) runs after both complete
4. **Compression** (`build:compress`):
    compresses `dist/` with zstd

## Commands

- `mise run build` -- full pipeline (client JS,
   site,
   postprocess,
   compress)
- `mise run build:site` -- site generation only (no postprocess or compression)
- `mise run build:site:clean` -- site generation from scratch (clears `.cache/`)
- `mise run build:postprocess` -- pagefind + asset fingerprinting (requires prior `build:site`)
- `mise run build:search` -- pagefind indexing only (requires prior `build:site`;
   standalone task for manual re-indexing)
- `mise run dev` -- full build,
   then serve with Caddy and rebuild on source changes
- `mise run format` -- run every format task (fonts + images)
- `mise run format:fonts` -- re-subset `fonts-source/*.woff2` into `public/`
- `mise run format:images` -- convert raster images to AVIF

## Content authoring

Add `.mdx` files under `src/content/{lang}/` with YAML frontmatter:

```yaml
---
title: Post Title
description: Short description
tags:
  - design
  - photography
---
```

The filename becomes the URL slug.
The parent directory name becomes the language code.

### Publication and update dates

`published` and `updated` are **not** authored in frontmatter.
They are derived from git history at build time in `src/lib/git-dates.ts`:

- `published`:
   author date of the oldest commit that touched the file (`git log --follow --reverse`)
- `updated`:
   author date of the newest commit that touched the file (`git log --follow`)

Full post pages render both dates with `<time datetime="...">` elements,
Open Graph article metadata exposes `article:published_time` and
`article:modified_time`,
 and RSS feeds use git-derived `updated` dates
for item `<pubDate>` plus channel `<pubDate>` and `<lastBuildDate>`.

Legacy `date`,
 `published`,
 or `updated` frontmatter fields are ignored
for rendering.
 When present,
 the build compares their calendar date against
the corresponding git-derived date and logs a warning if they diverge.
This preserves git history as the source of truth while surfacing stale
hand-authored metadata during migrations.

Both queries use `--follow` so renames preserve the original publication date.
Every commit touching the file bumps `updated`,
 including trivial edits;
there is no opt-out.

### Fallbacks

- Untracked or uncommitted files fall back to file mtime for both dates.
  This keeps dev previews of new posts rendering without git history.
- Authoring a post without committing is undefined behavior:
  published dates in dev previews will jump to the commit timestamp on first commit.
- Shallow clones (common in CI) lack the oldest commits needed for `published`.
  When `git rev-parse --is-shallow-repository` reports `true`,
  the oldest commit is fetched via the GitHub REST API through `gh api`
  (requires `gh` auth and an `origin` remote on github.
  com).
  `updated` still comes from local git because the tip is always present.

### Caching

Resolved dates are persisted in `.cache/build-manifest.json` alongside
the current `HEAD` commit SHA.
 On the next build,
 if `HEAD` has not moved,
cached dates are reused without re-spawning `git log`.
 When `HEAD` has moved,
dates are re-derived for every post.

## Syntax highlighting

Fenced code blocks are syntax-highlighted via the CSS Custom Highlight API.
Lezer parsers run **at build time** in a rehype plugin (`src/lib/rehype-highlight.ts`),
which embeds per-group character offsets as `data-hl-<group>` attributes on `<code>` elements.
The client script (`src/client/index.ts`) reads those offsets,
maps them to DOM Range objects,
 and registers CSS Custom Highlights.

No Lezer code ships to the browser.
The client bundle is ~1.8 KB (single file) versus ~313 KB (9 files) when parsers ran client-side.
The `data-hl-*` attributes add ~1.1 KB compressed across all pages:
a 99.4% net reduction in total transfer size for syntax highlighting.

## Output

Built files go to `dist/` as flat HTML with Caddy `try_files` providing clean URLs.
Cache manifest lives at `.cache/build-manifest.json`.

## CloudFront mirror

`amazon.aquati.cat` and `mirror.amazon.aquati.cat` reach the primary Caddy origin
through a native origin Host/SNI override.
One certificate covers both names.
The hostname was `aws.aquati.cat` until 2026-10-09.
The independently deployed function,
verification results,
and rollback boundary are documented in
[`cloudfront/README.md`](cloudfront/README.md).
The site build does not deploy this CloudFront function.
DNS and certificate authority records for both names are in
[`doc/troubleshooting/aws-cloudfront-mirror.md`](../../../doc/troubleshooting/aws-cloudfront-mirror.md).

## Images

Each photo lives under `src/content/` as a raster source paired with an AVIF counterpart
of the same basename.
 `mise run format:images` scans for any raster format sharp can decode
(PNG,
 JPEG,
 TIFF,
 WebP,
 GIF,
 HEIC,
 JXL,
 JP2,
 PPM,
 PFM,
 EXR,
 HDR;
 see `RASTER_GLOB` in
`src/image/format.ts`) and generates missing AVIFs via `src/image/convert.ts`.

Both the source raster and its AVIF are copied to `dist/` and fingerprinted independently.
Pages reference the AVIF for transfer efficiency;
 the source stays reachable under
its fingerprinted URL so readers who want the original
(for download,
 print,
 or closer inspection) can retrieve it directly.

When committing an AVIF source by hand,
 encode with `yuv420p` chroma subsampling.
`yuv444p` balloons file size with no perceivable benefit for photographic content:
`winter-tree.avif` was originally shipped at `yuv444p` quality 100 and was 2.2 MB
for a 2048x1365 image;
 re-encoding at CRF 28 yuv420p produced a 385 KB file with no
visible quality loss.

## Fonts

Four variable woff2 fonts ship with the site:

- `inter.woff2` + `interItalic.woff2`:
   body text
- `monaspaceNeon.woff2`:
   code blocks
- `materialSymbols.woff2`:
   icon font

The full upstream files live in `fonts-source/` (committed,
 not copied to
`dist/`).
 `mise run format:fonts` reads each upstream file,
 scans
`src/**/*.{ts,mdx,md}` for the characters actually in use,
 runs each font
through the three-stage `wawoff2` (WOFF2 decode) → `hb-subset-wasm`
(harfbuzz subset) → `woff2-encode-wasm` (WOFF2 re-encode) pipeline,
 and
writes the result to `public/` where `build:site` picks it up as a regular
static asset.
 Variable axes (`wght`,
 `opsz`,
 `FILL`,
 `GRAD`) are preserved.

Subsetting is **not** part of `build`.
 It takes a few seconds and its input
(source files + upstream fonts) changes infrequently,
 so it is a format
task run on demand.
 Re-run `format:fonts` when:

- you add an `icon('name')` call for a new Material Symbols icon
- you add non-ASCII characters (e.g. CJK) to MDX content or i18n strings
  that need to render in Inter or Inter Italic
- you replace an upstream font in `fonts-source/`

The subsetted `public/*.woff2` files are the artifacts the site ships and
are committed alongside source.

### Material Symbols

Icons render by **PUA codepoint**,
 not by ligature.
 A site-local helper
at `src/lib/icon/icon.ts` resolves a ligature name (e.g. `'info'`) to
the single-codepoint string it maps to in the upstream Material Symbols
codepoints table (`src/lib/icon/material-symbols-outlined.codepoints`,
one `name hex` pair per line):

```ts
import { icon, } from '../lib/icon/icon.ts';

h({
  tag: 'span',
  class: 'material-symbols-outlined',
  text: icon('info',),
},);
```

Rendering by codepoint (rather than letting the browser shape a ligature
from the icon name) is what makes tight subsetting possible:
 harfbuzz's
layout closure would otherwise retain every icon whose name can be
spelled from the letters present in source,
 which is essentially the
whole font.
 With PUA codepoints as input,
 only the specific glyphs
requested are retained;
 the subsetted icon font is a few KB regardless
of how many icons the upstream font provides.

`format:fonts` enumerates icons in use by scanning source for the
literal pattern `icon('NAME')`.
 Keep call-site arguments as string
literals (never `icon(variable)`) so the regex picks them up.
 An
unknown icon name throws at `format:fonts` time.

## Asset fingerprinting

All static assets in `dist/` are renamed with a 10-character content hash
before their extension (e.g. `styles.f1da372f3a.css`,
 `inter.693b77d4f3.woff2`).
References in HTML,
 CSS,
 and `manifest.webmanifest` are rewritten to match.

This runs as a post-processing step (`src/build/postprocess.ts`) in three phases
to respect the dependency chain between assets:

1. **Leaf assets**:
    images,
    fonts,
    JS,
    PDFs,
    favicons (no outgoing references to other hashable assets)
2. **CSS**:
    rewrite font `url()` references with hashed names from phase 1,
    then hash the CSS itself
3. **Reference rewriting**:
    replace original basenames with hashed basenames in all HTML files and `manifest.webmanifest`

Phase 3 uses basename-level `replaceAll` (e.g. `inter.woff2` -> `inter.693b77d4f3.woff2`),
which handles both absolute paths (`/inter.woff2`) and relative paths (`../glass-collection.avif`)
without needing to parse HTML.

**Excluded from fingerprinting**:
HTML (entry points),
 MDX source files,
 `pagefind/` (manages its own hashing),
`robots.txt`,
 RSS feeds,
 `manifest.webmanifest` (rewritten but not renamed,
 since PWA expects a stable URL).

**Cache headers** (configured in `Caddyfile`):

- Fingerprinted assets:
   `Cache-Control: public, max-age=31536000, immutable`
- HTML:
   `Cache-Control: no-cache` (revalidate with ETag every request)

Stale fingerprinted files from previous builds are cleaned up automatically
before each fingerprinting run.
