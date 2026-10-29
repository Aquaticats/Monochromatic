# Image processing library vet

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/wc` is now `package/wc/app`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Why this handover exists

The user asked whether there is anything better than `sharp` after a detailed `sharp` versus `jimp`
discussion.
They explicitly invoked the `choosing-technology` skill and asked for this handover because context was filling up.
The user later said to update this handover more aggressively.
This file is intentionally detailed so the next agent can resume without trusting compacted chat memory.

Do not recommend a library from memory.
The `choosing-technology` skill requires context-fork questions,
 alternative survey,
 source audit,
maintenance audit,
 validation,
 human-auditability comparison,
 and a decision doc after the user picks.

## Context forks already answered

The user answered the required context-fork questions:

- Need two recommendations:
  - Node.
  - Modern browsers where Wasm is unavailable.
- Workload:
   production pipeline.
- Formats:
   broad ingest.
- Native risk:
   prebuilt binaries are acceptable,
   but not preferred.

Important implication:
for Node production broad ingest,
 pure JavaScript and browser-canvas options are not substitutes for `sharp`.
For browsers without Wasm,
 broad ingest is not realistically available inside the browser.
Browser choices reduce to browser-decodable formats and canvas or JS resizing,
or moving broad ingest to the server.

## Current repo usage

Searches run in the repo found current `sharp` imports in:

- `package/webapp-productivity/wc/src/favicon.ts`
- `package/webapp-productivity/wc/src/favicon.unit.test.ts`
- `package/webapp-productivity/wc/src/page.browser.test.ts`
- `package/ssg/aquati.cat/src/images/convert.ts`
- `package/ssg/aquati.cat/src/build/render.ts`

Some of those files were already modified or untracked when searched.
Treat them as concurrent work unless the current task explicitly requires editing them.

Repo configuration already found:

- `pnpm-workspace.yaml` contains `allowBuilds.sharp: true`.
- `mise.toml` task `prepare:pnpm:others:approve-builds` runs `pnpm approve-builds sharp`.
- `pnpm-workspace.yaml` catalog entry includes `'sharp': '>=0.35.2'`.
- `allowBuilds` also marks `@vscode/ripgrep`,
   `core-js`,
   and `protobufjs` as false.

Prior scratch install showed `sharp@0.35.2` worked on current Linux x64 using prebuilt optional packages
when `allowBuilds.sharp: false`.
So the build approval is best understood as source-build fallback coverage,
 not the normal path on this host.

## Earlier `sharp` evidence to preserve

Keep these prior findings:

- `sharp@0.35.3` has `build: node install/build.js`.
- `sharp` normal installs use optional prebuilt `@img/sharp-*` and `@img/sharp-libvips-*` packages
  on common platforms.
- Source builds use C++17,
   `node-addon-api`,
   `node-gyp`,
   and optionally a globally installed libvips.
- Sharp install docs require Node-API v9,
   for example Node.js `>=20.9.0`.
- Prebuilt support includes JPEG,
   PNG,
   Ultra HDR,
   WebP,
   AVIF,
   TIFF,
   GIF,
   and SVG input.
- `sharp` benchmark docs for `sharp v0.35.0`,
   `libvips v8.18.2`,
   and `jimp v1.6.1` reported
  AMD64 JPEG throughput of Jimp buffer `3.44 ops/sec` and sharp buffer `89.63 ops/sec`,
   shown as `26.1x`.
- The same page showed PNG AMD64 sharp buffer `47.74 ops/sec` versus Jimp buffer `17.04 ops/sec`,
   about `2.8x`.
- libvips describes itself as demand-driven,
   horizontally threaded,
   low-memory,
   with roughly three hundred operations.
- `sharp` is mature despite `0.x`.
  Maintainer issues `lovell/sharp#1448` and `lovell/sharp#1754` say bug fixes and additions are patch releases,
  while deprecations and breaking changes are minor releases.
- `sharp v0.35.0` had breaking changes:
  Node 18 dropped,
   install script removed,
   AVIF tuning changed,
   deprecated APIs removed,
  and `format.jp2k` renamed.
- A caret range such as `^0.35.2` stays inside `0.35.x`,
   so it does not admit `0.36.0`.

## Surveyed package candidates

Registry and GitHub searches run:

- `npm search image-processing --searchlimit 50`
- `npm search image resize browser --searchlimit 50`
- `npm search image manipulation --searchlimit 50`
- `npm search sharp alternative --searchlimit 50`
- `gh search repos 'topic:image-processing language:JavaScript'`
- `gh search repos 'topic:image-processing language:TypeScript'`
- `gh search repos 'topic:image-resize language:JavaScript'`
- `npm view` metadata for all serious candidates.

Meaningful candidates surfaced:

- `sharp`:
   incumbent,
   Node native/libvips,
   broad ingest,
   strongest production fit.
- `jimp`:
   pure JavaScript,
   zero native dependencies,
   browser-capable,
   narrower format and performance envelope.
- `@napi-rs/image`:
   native Rust/N-API image pipeline with optional platform packages,
  serious Node alternative but younger and less battle-tested than `sharp`.
- `imgkit`:
   native Rust/N-API image pipeline,
  high feature count,
   Bun-oriented testing and packaging signals,
   younger than `sharp`.
- `rastermill`:
   small TypeScript API wrapping Photon and optional external native tools,
  designed for Node agents,
   useful but not a broad standalone replacement for `sharp`.
- `@silvia-odwyer/photon-node` and Photon:
  Rust and Wasm image library underneath Rastermill,
   but Node package is Wasm/native-adjacent and format support is narrower.
- `image-js`:
   scientific or analysis-oriented TypeScript image library,
  broad algorithms but limited production web-image format output.
- `imagescript`:
   zero dependency JavaScript image manipulation with Wasm/native codec helpers,
  useful for browser and pure-ish use,
   not broad production ingest.
- `pica`:
   high-quality browser resize,
   can run without Wasm using JS features,
  but it is not an image ingest or codec library.
- `browser-image-compression`:
   browser upload compression via canvas and workers,
  not a general image pipeline.
- `compressorjs`:
   browser compressor via canvas,
  not a general image pipeline.
- `gm`:
   wrapper around GraphicsMagick or ImageMagick CLIs,
  broad external tool power but depends on installed binaries and has very stale backlog.
- `canvas`:
   Cairo-backed Canvas API for Node,
  rendering primitive rather than production image pipeline replacement.
- `imagemagick` npm package:
   old wrapper,
   not a serious direct dependency.

## Cloned repositories and source audit paths

All serious candidates were cloned under `/tmp/agent/` using shallow GitHub clones where available.

Cloned paths:

- `lovell/sharp`:
   `/tmp/agent/lovell-sharp-20260702`
- `jimp-dev/jimp`:
   `/tmp/agent/jimp-dev-jimp-20260702`
- `Brooooooklyn/Image`:
   `/tmp/agent/brooooooklyn-image-20260702`
- `openclaw/rastermill`:
   `/tmp/agent/openclaw-rastermill-20260702`
- `nexus-aissam/imgkit`:
   `/tmp/agent/nexus-aissam-imgkit-20260702`
- `image-js/image-js`:
   `/tmp/agent/image-js-image-js-20260702`
- `matmen/ImageScript`:
   `/tmp/agent/matmen-imagescript-20260702`
- `nodeca/pica`:
   `/tmp/agent/nodeca-pica-20260702`
- `Donaldcwl/browser-image-compression`:
   `/tmp/agent/donaldcwl-browser-image-compression-20260702`
- `fengyuanchen/compressorjs`:
   `/tmp/agent/fengyuanchen-compressorjs-20260702`
- `aheckmann/gm`:
   `/tmp/agent/aheckmann-gm-20260702`
- `Automattic/node-canvas`:
   `/tmp/agent/automattic-node-canvas-20260702`
- `silvia-odwyer/photon`:
   `/tmp/agent/silvia-odwyer-photon-20260702`

Spot-read paths and behavior:

- `sharp`
  - `lib/input.mjs`:
     validates input descriptors,
     buffers,
     raw input,
     SVG/PDF/TIFF options,
    default pixel and channel safety limits.
  - `src/common.cc`:
     maps libvips loaders to image types,
     sniffs buffers and files,
    opens buffers/files/raw/text images,
     enforces pixel/channel limits.
  - `src/pipeline.cc`:
     async worker invokes libvips pipeline,
     shrink-on-load for JPEG,
     WebP,
     SVG,
     and PDF,
    orientation,
     resize,
     extract,
     composite,
     metadata,
     output.
  - `lib/output.mjs`:
     format selection,
     buffer/file output APIs,
     JP2 checks,
     metadata behavior.
  - `install/build.js`:
     source-build fallback through `node-gyp`.
  - `.github/workflows/ci.yml`:
     lint,
     build,
     unit tests,
     and packaging across Linux glibc,
     Linux musl,
    macOS,
     Windows,
     ARM,
     x64,
     and Wasm-related targets.
- `jimp`
  - `packages/jimp/src/index.ts`:
     default formats are BMP,
     GIF,
     JPEG,
     PNG,
     TIFF;
    default plugins include resize,
     crop,
     cover,
     contain,
     rotate,
     blur,
     color,
     print,
     quantize,
     etc.
  - `packages/core/src/index.ts`:
     `Jimp.read` accepts Buffer,
     ArrayBuffer,
     path,
     or URL;
    buffer format detection uses `file-type`;
     decoding delegates to configured formats.
  - `plugins/plugin-resize/src/index.ts`:
     resize option validation via zod and JS resize implementation.
  - `plugins/js-png/src/index.ts`:
     PNG uses `pngjs` sync encode/decode.
  - `.github/workflows/build.yml`:
     build,
     lint,
     test,
     and browser tests.
- `@napi-rs/image`
  - `packages/binding/index.js`:
     generated NAPI-RS native binding loader with platform optional packages
    and version checks.
  - `packages/binding/src/transformer.rs`:
     decodes HEIC,
     AVIF,
     SVG,
     and image-crate formats,
    tracks metadata,
     EXIF orientation,
     resize,
     fast resize,
     encode paths.
  - `packages/binding/src/jpeg.rs`:
     JPEG optimize/compress via `mozjpeg_sys` with unsafe FFI and cleanup.
  - `packages/binding/src/png.rs`:
     PNG encode,
     oxipng lossless compression,
     quantization.
  - `packages/binding/src/avif.rs`:
     AVIF encode via libavif.
  - `packages/binding/__test__/transformer.spec.mjs`:
     metadata,
     PNG/JPEG/SVG,
     orientation,
    staged transform regression tests.
  - `.github/workflows/CI.yml`:
     broad build matrix across macOS,
     Windows,
     Linux glibc,
     Linux musl,
    Android,
     and Wasm WASI.
- `rastermill`
  - `src/index.ts`:
     header probes for common formats,
     Photon internal path,
     external fallback via `execFile`,
    explicit pixel limits,
     metadata stripping policy,
     byte-budget encoding,
     backend selection.
  - `test/rastermill.test.ts`:
     header parsers,
     alpha detection,
     fake native tool scripts,
     HEIC-like boxes,
    metadata stripping,
     budgets,
     limits.
  - `.github/workflows/ci.yml`:
     Node 22 and 24,
     check,
     lint,
     format,
     coverage,
     build,
     docs,
     package smoke.
- `imgkit`
  - `src/index.ts`:
     exported API surface includes metadata,
     resize,
     crop,
     encode,
     hash,
     smart crop,
    dominant colors,
     thumbnails,
     composite.
  - `rust/src/lib.rs`:
     NAPI functions,
     sync and async wrappers,
     blocking task timeout note says Rust thread
    keeps running after timeout because of `spawn_blocking` limitation.
  - `rust/src/decode/mod.rs`:
     optimized JPEG/WebP shrink-on-load plus HEIC and image crate fallback.
  - `rust/src/decode/generic.rs`:
     image crate fallback with hard-coded one hundred megapixel protection.
  - `rust/src/metadata/mod.rs`:
     header-oriented metadata,
     HEIC detection,
     JPEG fast metadata.
  - `Cargo.toml`:
     dependencies include `image`,
     `fast_image_resize`,
     `turbojpeg`,
     `mozjpeg`,
     `webp`,
    `libwebp-sys2`,
     `tokio`,
     `image_hasher`,
     `smartcrop2`,
     `dominant_color`,
     `img-parts`,
     optional `libheif-rs`.
  - `test/local/index.test.ts`:
     Bun tests downloading `picsum.photos`,
     then metadata,
     resize,
     crop,
     encode.
  - `.github/workflows/ci.yml`:
     Bun setup,
     NAPI builds on several targets,
     tests after downloading artifacts.
- `image-js`
  - `src/load/decode.ts`:
     decodes PNG,
     JPEG,
     TIFF,
     and BMP by MIME sniffing.
  - `src/save/encode.ts`:
     encodes PNG,
     JPEG,
     and BMP only.
  - `src/geometry/resize.ts`:
     transform-based resize,
     not production codec pipeline.
  - `src/Image.ts`:
     large central image abstraction with analysis and processing methods.
  - `.github/workflows/nodejs.yml`:
     delegates to shared Zakodium workflow with lint,
     type check,
     and tests.
- `ImageScript`
  - `ImageScript.js`:
     large single JS API,
     uses wasm/node codec modules for SVG,
     GIF,
     PNG,
     JPEG,
     TIFF,
    and exposes image manipulation methods.
  - `tests/run.js`:
     custom runner spawning each test with a one second timeout.
  - `.github/workflows/node.yml`:
     Node 20,
     22,
     23,
     24 across macOS,
     Ubuntu,
     Windows.
- `pica`
  - `src/pica_main.ts`:
     browser resize pipeline,
     feature detection,
     workers,
     JS/Wasm math,
     tiling,
    canvas extraction and putImageData.
  - `src/supported_features.ts`:
     capability checks for canvas,
     OffscreenCanvas,
     workers,
    createImageBitmap,
     browser orientation region bugs,
     and canvas readback reliability.
  - `src/mm_resize/index.ts`:
     exports JS and Wasm resize kernels.
  - `test/unit/pica_api.test.ts`:
     JS and Wasm resizeBuffer checks.
  - `.github/workflows/ci.yml`:
     installs Playwright Chromium and runs full npm test.
- `browser-image-compression`
  - `lib/index.js`:
     main compress function,
     worker fallback,
     EXIF preservation path for JPEG.
  - `lib/utils.js`:
     browser and worker helpers,
     canvas limits,
     OffscreenCanvas fallback,
    browser sniffing,
     EXIF orientation parsing.
  - `test/index.spec.js`:
     mocha tests for data URLs,
     canvas drawing,
     JPEG/PNG/BMP compression and resize.
- `compressorjs`
  - `src/index.js`:
     browser `File` or `Blob` validation,
     `Image` plus canvas draw path,
    orientation and EXIF handling,
     size constraints,
     `toBlob` output.
  - `src/utilities.js`:
     canvas availability,
     EXIF orientation parsing,
     size adjustment.
  - `.github/workflows/ci.yml`:
     npm install,
     lint,
     build,
     karma tests,
     codecov.
- `gm`
  - `index.js`:
     wrapper API around streams,
     buffers,
     paths,
     and new image creation.
  - `lib/command.js`:
     resolves `gm`,
     `convert`,
     or `magick`,
     spawns external process,
    streams buffers,
     timeout handling.
  - `.github/workflows/node.js.yml`:
     Ubuntu and Windows tests against installed GraphicsMagick and ImageMagick,
    old Node 14,
     16,
     18 matrix.
- `canvas`
  - `index.js`:
     Canvas API exports,
     `loadImage`,
     font registration,
     stream exports.
  - `src/Image.cc`:
     C++ image loading from file,
     URL-ish string,
     buffer,
     JPEG,
     PNG,
     GIF,
     SVG if compiled.
  - `.github/workflows/ci.yaml`:
     build from source and test on Linux,
     Windows,
     macOS across many Node versions.
- `Photon`
  - `crate/src/lib.rs`:
     `PhotonImage`,
     image load from bytes via `image::load_from_memory().unwrap()`,
    encode to PNG,
     JPEG,
     WebP,
     many pixel operations.
  - `.github/workflows/ci.yml`:
     cargo check,
     test,
     fmt,
     clippy,
     wasm32 check.

Fuzzing or mutation-testing evidence:
no meaningful fuzz,
 property,
 or mutation harness was found in the cloned repos by broad searches for
`fuzz`,
 `libFuzzer`,
 `AFL`,
 `quickcheck`,
 `proptest`,
 `fast-check`,
 `arbitrary`,
 `mutation`,
 `Stryker`,
 and `mutmut`.
A few incidental matches appeared in comments,
 fixtures,
 lockfiles,
 and ordinary words.
Report absence as a finding.

## Maintenance signals measured

Issue and PR samples used recent created or updated items from the last twelve months.
Open issue counts were measured with `gh issue list`.
Do not use raw open issue count alone as the conclusion.

High-level findings:

- `sharp`
  - Latest clone commit:
     `1018449`,
     2026-07-01.
  - Recent issue sample:
     ten issues,
     six closed,
     nine with maintainer comments,
     ten with maintainer action.
  - Recent PR sample:
     ten PRs,
     eight merged,
     eight with maintainer comments.
  - Open backlog:
     one hundred seven open issues,
     ninety stale by last update before 2025-07-02,
     three open PRs.
  - Interpretation:
     active releases and strong owner triage,
     but a real long-lived backlog.
- `jimp`
  - Latest clone commit:
     `e1bfa93`,
     2026-04-07.
  - Recent issue sample:
     ten issues,
     one closed,
     one with maintainer comments/actions.
  - Recent PR sample:
     ten PRs,
     one merged,
     one with maintainer review/comment.
  - Open backlog:
     one hundred sixty-one open issues,
     one hundred forty-one stale,
     twenty-four open PRs.
  - Interpretation:
     active publishing but weak public issue support.
- `@napi-rs/image`
  - Latest clone commit:
     `ab7387a`,
     2026-07-01.
  - Recent issue sample:
     five issues,
     all closed,
     one with maintainer comment,
     all labeled or closed.
  - Recent PR sample:
     ten PRs,
     eight merged,
     two with maintainer reviews.
  - Open backlog:
     four open issues,
     two stale,
     three open PRs.
  - Interpretation:
     active and responsive,
     but much smaller public surface than `sharp`.
- `rastermill`
  - Latest clone commit:
     `3e61ff1`,
     2026-05-30.
  - Recent issue and PR samples:
     none.
  - Open backlog:
     zero issues and zero PRs.
  - Interpretation:
     tiny project,
     low public support signal either way.
- `imgkit`
  - Latest clone commit:
     `6614e37`,
     2026-06-23.
  - Recent issue sample:
     ten issues,
     all closed,
     nine with maintainer comments,
     all with maintainer action.
  - Recent PR sample:
     no PRs.
  - Open backlog:
     zero issues and zero PRs.
  - Interpretation:
     active but young and mostly single-maintainer signal.
- `image-js`
  - Latest clone commit:
     `2993eab`,
     2026-06-30.
  - Recent issue sample:
     ten issues,
     seven closed,
     two with maintainer comments,
     nine with action.
  - Recent PR sample:
     ten PRs,
     seven merged,
     one with maintainer review.
  - Open backlog:
     one hundred ninety-four open issues,
     one hundred eleven stale,
     eleven open PRs.
  - Interpretation:
     active releases and tests,
     but sizable backlog.
- `ImageScript`
  - Latest clone commit:
     `5d2a96f`,
     2025-06-01.
  - Recent issue sample:
     two open issues,
     both labeled,
     no maintainer comments.
  - Recent PR sample:
     none.
  - Open backlog:
     twenty-one issues,
     eighteen stale,
     zero PRs.
  - Interpretation:
     some maintenance,
     but weak public triage.
- `pica`
  - Latest clone commit:
     `60c7138`,
     2026-06-26.
  - Recent issue sample:
     two issues,
     both closed,
     both with maintainer comments.
  - Recent PR sample:
     one open PR with maintainer comments.
  - Open backlog:
     four issues,
     three stale,
     one open PR.
  - Interpretation:
     focused and maintained.
- `browser-image-compression`
  - Latest clone commit:
     `d933bc8`,
     2023-03-06.
  - Recent issue sample:
     eight issues,
     one closed,
     no maintainer comments.
  - Recent PR sample:
     one open PR,
     no maintainer review/comment.
  - Open backlog:
     fifty-one issues,
     thirty-seven stale,
     fourteen open PRs.
  - Interpretation:
     effectively weak maintenance despite package still being used.
- `compressorjs`
  - Latest clone commit:
     `081db76`,
     2026-04-06.
  - Recent issue sample:
     none.
  - Recent PR sample:
     one open PR,
     no maintainer review/comment.
  - Open backlog:
     six issues,
     two stale,
     one open PR.
  - Interpretation:
     some maintenance,
     but old browser testing stack.
- `gm`
  - Latest clone commit:
     `6e43846`,
     2025-02-24.
  - Recent issue sample:
     one closed issue,
     no maintainer comment.
  - Recent PR sample:
     one open PR,
     no maintainer review/comment.
  - Open backlog:
     three hundred twenty-six issues,
     three hundred twenty-five stale,
    forty-two open PRs.
  - Interpretation:
     not healthy as a new direct dependency.
- `canvas`
  - Latest clone commit:
     `556cb7c`,
     2026-06-29.
  - Recent issue sample:
     ten issues,
     five closed,
     two with maintainer comments,
     six with action.
  - Recent PR sample:
     ten PRs,
     six merged,
     one with maintainer comments.
  - Open backlog:
     four hundred twenty-six issues,
     three hundred ninety-two stale,
    forty open PRs.
  - Interpretation:
     active but huge,
     and not a direct image-pipeline replacement.
- `Photon`
  - Latest clone commit:
     `1390383`,
     2026-07-02.
  - Recent issue sample:
     three issues,
     two closed,
     one with maintainer comment.
  - Recent PR sample:
     ten PRs,
     six merged,
     five with maintainer reviews.
  - Open backlog:
     forty-eight issues,
     forty-four stale,
     two open PRs.
  - Interpretation:
     active enough,
     but standalone Node broad-ingest story is weaker than `sharp`.

## Human-auditability surface measured

Approximate source-ish file and line counts were measured excluding `node_modules`,
 `dist`,
 coverage,
site/docs/demo folders where possible.
These are selection factors,
 not exact code-size claims for all published artifacts.

- `pica`:
   forty-four source-ish files,
   4,325 lines.
  Smallest serious browser resize surface.
- `compressorjs`:
   thirty-three source-ish files,
   2,880 lines.
  Small,
   but browser-only compressor and old test stack.
- `browser-image-compression`:
   thirty-four source-ish files,
   3,697 lines.
  Small,
   but weak maintenance and install conflict.
- `rastermill`:
   six source-ish files,
   5,575 lines.
  Very readable API,
   but it extends trust to Photon and optional external tools.
- `jimp`:
   one hundred sixty-two source-ish files,
   10,372 lines.
  Pure JS and modular,
   but many package/plugins.
- `ImageScript`:
   sixty-eight source-ish files,
   11,786 lines.
  Mostly one large API plus codec modules.
- `imgkit`:
   seventy-eight source-ish files,
   16,704 lines.
  Rust native surface plus many feature modules.
- `@napi-rs/image`:
   eighty-five source-ish files,
   20,694 lines.
  Rust native surface plus generated NAPI loader and native dependencies.
- `sharp`:
   one hundred nine source-ish files,
   32,011 lines.
  Larger codebase but backed by libvips and very broad tests.
- `canvas`:
   ninety-five source-ish files,
   32,562 lines.
  Large rendering/native surface,
   not direct replacement.
- `image-js`:
   five hundred twenty-six source-ish files,
   43,506 lines.
  Large algorithm library,
   not focused production ingest.

Interpretation:
for Node broad ingest,
 the more auditable smaller candidates do not satisfy the same constraints.
For browser no-Wasm resize,
 `pica` has the best fit and small surface.

## Validation results so far

A scratch package was created at `/tmp/agent/image-vet-scratch-20260702`.
Because the Bash tool ignored `cwd` in one run,
 `npm init --yes` accidentally rewrote the repo root `package.json`.
That was immediately restored with `git restore -- package.json`.
Do not repeat that pattern.
Use explicit `cd "${HOME}/temp/agent/..." && command` in Bash for current scratch validation.

Scratch install command:

```sh
cd /tmp/agent/image-vet-scratch-20260702 && npm install sharp@0.35.3 jimp@1.6.1 @napi-rs/image@1.14.0 pica@10.0.2 imagescript@1.3.1 image-js@1.6.2 browser-image-compression@2.0.2 compressorjs@1.3.0 rastermill@0.3.1 imgkit@2.3.0 --save-exact
```

Result:
installed one hundred thirty-seven packages,
 zero vulnerabilities.
NPM warned that `skia-canvas@3.0.8` had an install script not covered by allowScripts.

Integration script:
`/tmp/agent/image-vet-scratch-20260702/integration-check.mjs`.
It generated a valid two pixel by two pixel PNG with `pngjs`,
 then exercised library boundaries.

Passing integration checks:

- `sharp`:
   imported,
   resized PNG to one pixel by one pixel,
   metadata read back as PNG.
- `jimp`:
   read PNG,
   resized,
   emitted PNG buffer.
- `@napi-rs/image`:
   `Transformer` resized and emitted PNG,
   metadata read back.
- `pica`:
   JS-only `features: ['js']` `resizeBuffer` worked without Wasm.
- `imagescript`:
   decoded PNG,
   resized,
   encoded PNG.
- `image-js`:
   decoded PNG,
   resized,
   encoded PNG.
- `imgkit`:
   metadata and resize to PNG worked.
- `rastermill`:
   `execution: 'internal'` probe and PNG encode worked.

Upstream validation commands and outcomes:

- `jimp`
  - Initial `pnpm install --frozen-lockfile` failed because current pnpm `11.9.0` cannot read the repo lockfile.
  - Retried with `npx --yes pnpm@9.9.0 install --frozen-lockfile && npx --yes pnpm@9.9.0 test`.
  - Result:
     success.
- `pica`
  - Initial `npm install && npm test` failed because Playwright Chromium was missing.
  - `npx playwright install --with-deps chromium` failed because this host lacks `apt-get`.
  - Retried `npx playwright install chromium && npm test`.
  - Result:
     success.
  - Browser test `test/browser/fixture_resize_canvas.test.ts` directly exercises canvas resize with
    `picaFactory({ features: ['js'] })`,
     so the browser no-Wasm canvas path was covered by upstream tests.
- `sharp`
  - `npm install && npm test` failed at `tsd` because `tsd --files ./test/types/sharp.test-d.{cts,mts}`
    resolved package types oddly from the test file directory.
  - `npm run build:dist && npm test` failed the same way.
  - Manual equivalent passed:
    `npm run build:dist && npx tsd --typings ./dist/index.d.mts --files ./test/types/sharp.test-d.mts && npx tsd --typings ./dist/index.d.cts --files ./test/types/sharp.test-d.cts && npm run test-unit`.
  - Result:
     1,811 tests passed,
     zero failed,
     coverage printed one hundred percent.
- `rastermill`
  - `npx --yes pnpm@10.33.3 install --frozen-lockfile && npx --yes pnpm@10.33.3 test:coverage && npx --yes pnpm@10.33.3 build`.
  - Result:
     success.
- `ImageScript`
  - `npm install && npm test`.
  - Result:
     success.
- `image-js`
  - `npm install && npm test`.
  - Result:
     failed only in `src/extra/draw/__tests__/draw_text.test.ts`.
  - Cause:
     optional `skia-canvas` could not load `../skia.node` because the install script was not approved.
  - Other result:
     two hundred seven test files passed,
     one failed;
     ten draw text failures;
     1,314 tests passed;
    six expected failures;
     one todo.
- `browser-image-compression`
  - `npm install && npm test`.
  - Result:
     failed during install with peer dependency conflict:
    root has `rollup@3.15.0`,
     `rollup-plugin-terser@7.0.2` peers on `rollup@^2.0.0`.
  - This is a reproducibility finding against it.
- `compressorjs`
  - `npm install && npm test`.
  - Install completed but reported thirty-four vulnerabilities.
  - Test failed in Karma config:
    `ReferenceError: require is not defined in ES module scope` from `node_modules/yargs/yargs` under Node `v26.4.0`.
  - This is a modern-runtime validation finding against it,
     though it may pass under older Node.
- `@napi-rs/image`
  - Initial host build failed in `libaom-sys` because neither `yasm` nor `nasm` was installed.
  - A bounded Podman validation image was built from Rust bookworm plus Node 24,
     CMake,
     `g++`,
     `make`,
    `nasm`,
     Perl,
     `pkg-config`,
     and Python.
  - First container retry failed because SELinux labeling denied reading the mounted Yarn release file.
  - Retried with `--memory=2g --cpus=2 --userns=keep-id --security-opt label=disable`.
  - Native `@napi-rs/image` release build then succeeded.
  - Whole-monorepo `build:ts` failed on an unrelated TypeScript 6 diagnostic:
    `packages/rollup-plugin/tsconfig.json` still uses deprecated `moduleResolution=node10`.
  - Root AVA test command `node .yarn/releases/yarn-4.17.0.cjs test` passed after native build.
  - Scratch integration also passed.
- `imgkit`
  - Initial host build failed in `turbojpeg-sys` because no NASM assembler was installed.
  - Same bounded Podman image was used for retry.
  - First container retry failed before validation because the copied npm symlink was broken and login shell handling
    dropped Cargo from `PATH`.
  - Retried with host Bun mounted directly,
     `sh -c`,
     and `--memory=2g --cpus=2 --userns=keep-id --security-opt label=disable`.
  - Native build succeeded,
     `tsup` type and JS build succeeded,
     and `bun test test/local` ran.
  - Result:
     two hundred eight tests passed,
     one failed,
     across ten files.
  - The failing EXIF test was an unnamed `beforeAll` failure in `test/local/exif.test.ts`.
    Its remote Wikimedia fixture returned HTTP 400 HTML,
     not a PNG:
    status `400`,
     content type `text/html; charset=utf-8`,
     prefix `<!DOCTYPE html>`.
  - Direct EXIF smoke validation with repo benchmark JPEG passed:
    `writeExif`,
     `stripExif`,
     `toWebp`,
     `writeExif` on WebP,
     and metadata readback all worked.
  - Scratch integration also passed.
- `gm`
  - Full upstream test not run.
  - It requires GraphicsMagick or ImageMagick binaries.
- `canvas`
  - Full upstream test not run.
  - It requires native Cairo/Pango/etc. build dependencies.

## Preliminary interpretation, not final recommendation

For Node production pipeline plus broad ingest:

- `sharp` is still the front-runner.
  It is the only audited candidate with broad format ingest,
   mature maintenance,
   broad platform CI,
  comprehensive tests,
   and proven local integration.
  Its downside remains native/prebuilt dependency trust and source-build fallback complexity.
- `@napi-rs/image` is the most serious direct alternative.
  It has active maintenance,
   a broad native build matrix,
   passed scratch integration,
  built successfully in the bounded container,
   and passed upstream AVA tests.
  It still has a smaller public maturity record than `sharp`,
   and source audit found more direct Rust/native codec surface
  for this repo to trust.
- `imgkit` is interesting but should not outrank `sharp` for this repo yet.
  It is younger,
   Bun-heavy in tests,
   has a very broad feature API,
  and its async timeout note admits work continues in the background after timeout.
  It built in the bounded container and passed direct EXIF smoke validation.
  Its upstream local tests were one remote fixture failure short of green.
- `rastermill` is auditable and agent-friendly,
   but it is not a broad standalone production replacement.
  Its internal mode inherits Photon limitations;
   its external mode shells out to native tools.
- `jimp`,
   `image-js`,
   and `ImageScript` are not production broad-ingest replacements for `sharp`.
  Their fit is tests,
   fixtures,
   pure-JS manipulation,
   or narrower browser-compatible workflows.
- `gm` and `canvas` should not be selected as direct replacements.
  `gm` adds external binary dependencies and stale backlog.
  `canvas` is a rendering API with a large native surface,
   not an image pipeline.

For modern browsers where Wasm is unavailable:

- If the browser task is resize/downscale before upload,
   `pica` JS-only mode is the current front-runner.
  It passed `features: ['js']` integration and full upstream tests after installing Playwright Chromium.
  It is small,
   focused,
   maintained,
   and explicitly handles browser canvas capability bugs.
- If the browser task is compress user uploads using native browser decoders/encoders,
   `compressorjs` and
  `browser-image-compression` remain possible but currently have validation and maintenance concerns.
- If the requirement truly means broad ingest in-browser without Wasm,
   no surveyed package satisfies it.
  The recommendation should say the broad ingest pipeline belongs on Node/server side,
  with browser-side best-effort preprocessing only for browser-decodable image formats.

## Required next work

Continue from task `Synthesize recommendation`.
Do not restart from scratch unless needed.

Remaining validation choices:

- Treat `gm` and `canvas` as disqualified without full validation because they are not serious finalists
  for the stated package role.
- Do not spend more time on `browser-image-compression` unless the user specifically wants a browser compressor.
  Its `npm install` peer conflict is already a negative finding.
- Do not spend more time on `compressorjs` unless the user specifically wants a browser compressor.
  Its Node 26 Karma failure and install vulnerability count are already negative findings.

For synthesis:

- Produce a recommendation with two separate rankings:
  - Node production broad ingest.
  - Browser without Wasm.
- Include pros and cons for each finalist and a fully sorted personal ranking.
- Name at least two rejected alternatives with concrete reasons.
- If the user picks,
   write `doc/decision/image-processing-library.md` or another clearly named decision doc.

## Cautions for future agent

- Do not treat this handover as final recommendation.
  It is strong evidence toward `sharp` for Node and `pica` for browser resize without Wasm.
- Do not say browser no-Wasm can do broad ingest unless you can point to browser codecs or JS decoders covering it.
- Do not treat `0.x` alone as evidence that `sharp` is immature.
- Do not claim `sharp` requires source builds on normal platforms.
  The evidence says prebuilt optional packages are the usual path.
- Do not ignore the source-build fallback.
  It is why `allowBuilds.sharp: true` can still be reasonable.
- Do not cite open issue count alone as maintenance evidence.
  Use the sampled maintainer actions and backlog shape.
- Do not run scratch commands relying on Bash tool `cwd` alone.
  Use explicit `cd "${HOME}/temp/agent/..." && command`.
- Do not stage unrelated repo changes.
  Many unrelated files were already modified or untracked while this investigation ran.
