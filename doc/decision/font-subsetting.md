## Font subsetting pipeline (ssg/aquati.cat)

Records the technology choice behind `src/build/subset-fonts.ts` in
`package/ssg/aquati.cat/`.
 Future sessions consult this before
re-proposing rejected paths.

This document is appended to,
 not rewritten.
 When a downstream choice forces
re-evaluation of an earlier one,
 mark the earlier decision superseded;
 do not
delete it.

### Context

The static site ships four WOFF2 fonts (`inter`,
 `interItalic`,
 `monaspaceNeon`,
`materialSymbols`).
 Their full upstream files live in `fonts-source/`;
 the
build emits subsetted copies to `public/` covering only the code points
actually referenced in source files plus the Material Symbols PUA range for
`icon('...')` call sites.
 Subsetting runs at format time (`mise run
format:fonts`),
 not on every build,
 and the subsetted output is committed.

Constraints:

- Input format:
   WOFF2 (committed in `fonts-source/`).
- Output format:
   WOFF2 (consumed by `<link rel="preload">` and `@font-face`).
- Variable-font axes preserved (`wght`,
   `opsz`,
   `FILL`,
   `GRAD`) so the CSS
  `font-weight: 100 900` declarations keep working.
- Layout features retained (matches the prior `subset-font` default;
   required
  for Material Symbols ligatures,
   Inter contextual alternates,
   etc.).
- Bun + ESM runtime.
- Workspace policy:
   lodash is on the dependency blocklist
  (`doc/dependency-blocklist.md`,
   "use native ES + es-toolkit").

### Decision

Use `wawoff2` (WOFF2 → SFNT decode) → `hb-subset-wasm` (SFNT subset) →
`woff2-encode-wasm` (SFNT → WOFF2 encode).

The three packages compose as a one-shot pipeline;
 each stage is a function
that returns a `Uint8Array`.
 None pull lodash,
 none pull an old `p-limit`,
and `hb-subset-wasm` plus `woff2-encode-wasm` ship native TypeScript types.

### Rejected alternatives

#### `subset-font` (the incumbent)

Used previously.
 Rejected for three reasons surfaced during audit:

1.  **Transitive deps incompatible with workspace policy.
    ** Pulls
    `lodash@^4.17.21` for a single `_.once` call (`subset-font/index.js:6`)
    and `p-limit@^3.1.0` for a one-line concurrency-1 mutex
    (`subset-font/index.js:187-188`).
     lodash is on the workspace blocklist;
    `p-limit@3.1.0` is the last CJS release (4.
    x is ESM-only) so the pin is
    forced by `subset-font` being CJS.
2.  **WASM-memory-growth bug.
    ** Captures `heapu8 = new Uint8Array(harfbuzzJsWasm.memory.buffer)`
    once at init (`subset-font/index.js:13`) and never re-reads.
     When the
    harfbuzz heap grows during a subset call,
     subsequent reads against the
    stale view target detached memory.
     `hb-subset-wasm` re-reads `memory.buffer`
    after every wasm call (`hb-subset-wasm/src/api.ts:484-485`,
     "Re-read
    buffer in case memory grew during subset").
3.  **Resource leak on `variationAxes` error paths.
    ** Throws at
    `subset-font/index.js:107-110, 119-122, 136-139` free `fontBuffer` and
    destroy `face` before throwing but never call `hb_subset_input_destroy(input)`.
    `hb-subset-wasm`'s C wrapper destroys both `input` and `face` on every
    error path (`hb-subset-wasm/wasm/wrapper.c:115-138`).

CJS-only with no `"type": "module"` is a secondary cost (forces an ambient
type shim and a CJS interop hop).
 Maintained,
 not abandoned:
 129,900
downloads/month,
 last commit 2026-04-02.

#### Hand-rolled wrapper over `harfbuzzjs@1.x`

`harfbuzzjs@1.0.0` shipped 2026-05-11 with ESM,
 TypeScript types,
 and
`FinalizationRegistry`-based automatic cleanup.
 Tempting to wrap directly.
Rejected because `harfbuzzjs` exposes the entire HarfBuzz API (shaping,
layout,
 font metrics,
 buffer APIs) and the wasm import surface is
correspondingly large.
 `hb-subset-wasm` is purpose-built for the subset
case,
 with a wasm-import surface limited to
`env.emscripten_notify_memory_growth` (no-op) and
`wasi_snapshot_preview1.proc_exit` (throws).
 No filesystem,
 no syscalls.
Writing our own wrapper would reproduce essentially the same code that
`hb-subset-wasm` already audits cleanly,
 against a larger trust surface.

#### `@web-alchemy/fonttools`

Node adapter that runs Python `fonttools` (incl.
 `pyftsubset`) via Pyodide.
This is the upstream tool Google Fonts uses;
 the most feature-complete
subsetter that exists.
 Rejected because our call site needs `text` +
`targetFormat: 'woff2'` plus retained layout features;
 a small fraction of
fonttools' surface.
 The install carries the full CPython 3.
x WASM runtime
plus the fonttools wheels;
 cold start is multi-second.
 Re-evaluate if a
future requirement needs pyftsubset-specific features
(e.g. `--desubroutinize`,
 hint preservation,
 name-table edits).

#### `fontkit` (foliojs)

Disqualified for web fonts.
 From the foliojs README:
 subsets "may not work
as standalone files.
 They have no cmap tables and other essential tables for
standalone use.
" Fontkit is a PDF-embedding subsetter,
 not a web-font
subsetter.
 Confirmed by reading `foliojs/fontkit/README.md`.

#### `fontmin`

Gulp-stream plugin architecture (last major release years ago);
 mismatch
with the workspace's mise-task / Bun-script pipeline.
 Rejected for
toolchain fit before the audit reached the source-quality layer.

#### `wawoff2` for both decode and encode (drop `woff2-encode-wasm`)

`wawoff2` already has `compress()` and `decompress()`;
 using it for both
sides would remove one dep.
 Rejected because `woff2-encode-wasm` ships
SHA-256-checksum-pinned upstream sources
(`woff2-encode-wasm/scripts/fetch-deps.sh:11-15`:
 `google/woff2 v1.0.2` and
`google/brotli v1.1.0` verified at fetch time),
 runs Node + Cloudflare
Workers + Deno tests,
 and is the package we already audited line-by-line.
`wawoff2` is older and the build provenance is less audited.
 Trade is
worth it for the encode step;
 for decode,
 `wawoff2` is fine because it has
many years of production use behind it.

### Audit notes (carried for future review)

`hb-subset-wasm@0.4.0`,
 `woff2-encode-wasm@0.1.1`.
 Both ~1 month old as of
2026-05-12,
 single maintainer (Kyosuke Nakamura / PixelGrid Inc.
,
 GitHub
since 2008,
 30 public repos,
 prior `cloudflare-pages-glyphhanger` work).
Low download counts (99 + 40 per month respectively):
 community oversight
risk is real and the audit cited above is the workspace's primary
correctness guarantee.
 Re-audit if upstream releases a major version that
changes the wasm import surface or the build flags
(`hb-subset-wasm/scripts/build-wasm.sh:62-107`).

Pipeline-internal serialization:
 both `hb-subset-wasm.subset()` and
`woff2-encode-wasm.encode()` are declared `async` but their bodies are
synchronous (no `await`) after the initial wasm-export lookup.
 JS's
single-threaded event loop therefore keeps concurrent calls atomic without
an explicit mutex;
 `Promise.all` over the four fonts is safe.
 Verified by
reading `hb-subset-wasm/src/api.ts:251-504` and
`woff2-encode-wasm/src/index.ts:143-194`.

### Output sizes after migration (2026-05-12)

Font,
subset-font,
hb-subset-wasm pipeline,
delta:

- inter.woff2:
  76 468,
  76 448,
  −20
- interItalic.woff2:
  82 444,
  82 856,
  +412
- monaspaceNeon.woff2:
  78 344,
  78 980,
  +636
- materialSymbols.woff2:
  7 372,
  7 516,
  +144

All within ~1%;
 differences attributable to HarfBuzz version (`10.4.0` vs
the `harfbuzzjs@0.10.3` build) and Google woff2 encoder settings.
