<!--
This document uses HTML <table> elements for every numeric grid, by explicit
request, overriding the repo's usual MD5 "no markdown tables" convention.
The data is dense and tabular; HTML tables render it far more legibly than
nested lists or code fences. This is a deliberate, documented exception.
-->

# Replace the zstd CLI with node:zlib's built-in zstd

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Context

Two places in the workspace shell out to the external `zstd` command-line tool:

- `package/figma/kiwi/src/index.ts`,
   function `decompressZstd`,
   decompresses the
  zstd-compressed document section of a Figma `.fig` / `.deck` / `.jam` export.
   It first tries an
  optional native module (`@bokuwatch/zstd`,
   never declared as a dependency) and otherwise spawns
  `zstd -d` through `nano-spawn`,
   shuttling data through temp files.
- `package/ssg/aquati.cat/mise.toml`,
   task `build:compress`,
   runs
  `zstd -z -f --no-check -T0 --exclude-compressed --no-content-size -r --adapt dist` to write a
  `<file>.zst` companion next to every compressible asset in `dist/`.
   The companions are read by
  `package/page-weight/cli/src/size.ts` (`wireSize`) and exist for precompressed serving.

Both depend on the `zstd` binary being installed,
 which the workspace declares as a mise tool
(`"github:facebook/zstd" = "latest"` in `mise.toml` and `mise.no-env.toml`).

Node 22.15+ and Bun 1.3+ ship native Zstandard support directly in `node:zlib`
(`zstdCompressSync`,
 `zstdDecompressSync`,
 `zstdCompress`,
 `zstdDecompress`,
 and the streaming
`createZstd*` constructors).
 The workspace runs Bun 1.3.14 and Node 26.3.0,
 both of which expose
the full API.
 Replacing the subprocess and the external tool dependency with the platform built-in
removes a tool from the install set,
 removes the temp-file dance and the undeclared optional
dependency in the Figma parser,
 and makes the compression step a normal in-process build script.

The decompression swap is mechanically trivial and is treated as an independent change.
 The
compression swap raises real choices (compression level,
 threading model,
 which `.zst` to keep),
because the CLI flags being replaced (`--adapt`,
 `-T0`,
 `--exclude-compressed`) have no one-to-one
`node:zlib` equivalent.
 Those choices were settled by benchmarking,
 documented in full below.

This document records the decision and the complete evidence behind it.
 It does not itself land the
change;
 the code edits await go-ahead.

## Decision

1. **Figma `decompressZstd`** becomes a one-line call to `node:zlib`'s `zstdDecompressSync`.
    The
   optional `@bokuwatch/zstd` path and the `zstd` CLI fallback are removed,
    and `nano-spawn` is
   dropped from the package's dependencies (it has no other use there).
    This is correctness-only:
   one-shot decompression of a single frame,
    no level or threading concerns.
    Validated by full
   interop (every produced frame round-trips byte-identically) and by the package's existing
   integration test that decodes real `.fig` / `.deck` / `.jam` files.

2. **The ssg `build:compress` task** becomes a build script (`src/build/compress.ts`,
    alongside the
   existing `postprocess.ts`) that walks `dist/`,
    compresses each file with `node:zlib`,
    and writes
   `<file>.zst`.
    The workspace has migrated off Bun,
    so this script must run under **Node**;
    the
   engine choice is made for Node,
    with Bun numbers kept only as reference.
    Specifics,
    each justified
   by the benchmark:
   - **Engine + threading:
     ** shard the file list across **`node:worker_threads` (about 8 workers,
     matching the physical-core count)**,
      each worker running synchronous `zstdCompressSync`.
      This is
     the fastest Node approach measured,
      and at matched ratio (level 19) it is faster than the zstd
     CLI itself (168 ms versus 217 ms on tmpfs for the real `dist/`).
      Three other approaches were
     measured and rejected:
      Node's **async** `zstdCompress` via `Promise.all` is pathologically slow
     (about 1 second at level 19,
      7.8 seconds at level 22) because Node's async zlib carries a large
     fixed per-call overhead;
      the CLI's in-frame `-T0` multithreading is useless for many small
     files;
      the CLI's cross-file process parallelism is catastrophic (process-spawn bound,
      seconds to
     tens of seconds).
      Node sequential `zstdCompressSync` is the simple fallback (243 ms) if the
     worker_threads machinery is judged not worth it,
      but it does not scale with file count the way
     worker_threads does (2,000 files:
      787 ms sequential versus 253 ms with 8 workers).
      Over-
     subscribing to all 16 logical threads is slightly slower than 8 for CPU-bound zstd.
      Bun's async
     parallel path is faster still (72 ms) but Bun is no longer the target runtime.
   - **Level:
     ** fixed **level 19**.
      `node:zlib` has no equivalent of `--adapt`,
      so a fixed level is
     required,
      and a fixed level also makes the build's output reproducible (`--adapt` is not;
      see
     caveats).
      Level 19 is the best practical ratio;
      under the chosen Node worker_threads engine its
     full wall time on the real `dist/` is ~168 ms (tmpfs) / ~177 ms (SSD).
      Level 15 is the knee of
     the ratio curve (within 0.1 percentage points of level 19's ratio at ~20% less time:
      ~141 ms /
     ~146 ms) and is the better pick only if dev-rebuild latency is ever a felt pain;
      since
     precompressed assets are paid for once and served repeatedly,
      level 19 is preferred.
   - **Exclusion policy ("keep if smaller" + extension skip):
     ** write `<file>.zst` only when it is
     strictly smaller than the original,
      and skip known-incompressible extensions
     (images,
      fonts,
      video,
      already-compressed) without reading them.
      This is stricter and more
     correct than the CLI's `--exclude-compressed` (which is extension-only and can leave a `.zst`
     larger than its source);
      incompressible input always grows under zstd (frame overhead),
      so
     keeping the raw bytes is the right behavior for `wireSize` and for any precompressed server.
     The set of produced `.zst` files therefore differs from the CLI's by design.

3. **Remove the external tool.
   ** Drop `"github:facebook/zstd" = "latest"` from `mise.toml` and
   `mise.no-env.toml` once both call sites are migrated.
    No other workspace code invokes the `zstd`
   binary (the lone remaining textual reference is a tool-name entry in the bash-output-filter
   allowlist,
    which is generic and harmless).

The decompression change (1) is unconditionally safe.
 The compression change (2) is the one the
benchmark exists to justify.

## Head-to-head: the current task versus the proposal

The single most decision-relevant measurement,
 framed for the **Node** target (the workspace is
after the Bun-to-Node migration.
 Real `dist/` text assets (138 files,
 1,257,418 bytes raw),
 full wall time
including process startup and file writes,
 output dir cleared before each run.
 Times are mean +/-
stddev.
 The key comparison is at *matched* compression (level 19),
 since the current `--adapt`
command's speed comes entirely from it choosing a weak level.

<table>
  <thead>
    <tr><th>Configuration</th><th>tmpfs (RAM) mean</th><th>SSD (btrfs+LUKS) mean</th><th>Output bytes (kept)</th><th>Ratio vs raw</th></tr>
  </thead>
  <tbody>
    <tr><td>current: <code>zstd -T0 --adapt</code> (CLI)</td><td>13.8 ms &plusmn; 0.4</td><td>33.4 ms &plusmn;
34.6</td><td>404,644 (level-3-equiv, nondeterministic)</td><td>32.18%</td></tr>
    <tr><td>zstd CLI <code>-19 -T0</code> (matched ratio)</td><td>216.6 ms &plusmn; 4.9</td><td>232.1 ms &plusmn;
1.9</td><td>374,726</td><td>29.80%</td></tr>
    <tr><td><b>proposed: node worker_threads (8), level 19</b></td><td><b>168.1 ms &plusmn;
9.4</b></td><td><b>177.3 ms &plusmn; 4.2</b></td><td>374,726</td><td>29.80%</td></tr>
    <tr><td>proposed: node worker_threads (8), level 15</td><td>141.4 ms &plusmn; 10.5</td><td>146.4 ms &plusmn;
4.0</td><td>375,774</td><td>29.88%</td></tr>
    <tr><td>node sequential, level 19 (simple fallback)</td><td>243.4 ms &plusmn; 3.4</td><td>257.4 ms &plusmn;
3.6</td><td>374,726</td><td>29.80%</td></tr>
    <tr><td>node async parallel, level 19 (rejected)</td><td>980.9 ms &plusmn; 38.2</td><td>987.3 ms &plusmn;
24.7</td><td>374,726</td><td>29.80%</td></tr>
    <tr><td>bun parallel, level 19 (reference only; Bun being removed)</td><td>74.1 ms &plusmn;
2.8</td><td>88.3 ms &plusmn; 2.2</td><td>374,726</td><td>29.80%</td></tr>
  </tbody>
</table>

Reading this table:

- The current `--adapt` task looks fastest (14 ms / 33 ms) only because `--adapt` settled on a low
  effort level (its output,
   404,644 bytes,
   equals level 3 exactly) and is nondeterministic (the SSD
  run has stddev 35 ms because `--adapt` tunes its level to observed I/O speed).
   It is not a fair
  comparison:
   it produces 7.4% larger,
   irreproducible output.
- At **matched ratio (level 19)** the proposed Node worker_threads engine (168 ms / 177 ms) is
  **faster than the zstd CLI at the same level** (217 ms / 232 ms),
   while removing the external tool
  and producing reproducible output.
- worker_threads (8 workers) beats Node sequential (243 ms / 257 ms) by ~1.5x here and by ~3x on
  larger file counts (see the Node parallelism section),
   and beats Node's async parallel path (~1 s)
  by ~6x.
- Bun's async parallel path (74 ms) is the fastest of all,
   but Bun is no longer the
  target;
   it is listed only to anchor the Node numbers.

## Environment

All measurements were taken on one machine.
 Numbers are specific to it;
 see caveats.

<table>
  <tbody>
    <tr><th>CPU</th><td>AMD Ryzen 7 8700F, 8 cores / 16 threads, base/boost to 5.055 GHz max, 0.413 GHz min</td></tr>
    <tr><th>CPU scaling at capture</th><td>governor reported 83% scaling (not pinned to performance)</td></tr>
    <tr><th>RAM</th><td>62 GiB total, ~41 GiB available, ~43 GiB in buff/cache during runs</td></tr>
    <tr><th>OS / kernel</th><td>Fedora 44, Linux 7.0.9-ogc3.2.fc44.x86_64</td></tr>
    <tr><th>tmpfs backend</th><td><code>/tmp</code>, tmpfs (RAM-backed), 32 GiB</td></tr>
    <tr><th>SSD backend</th><td><code>/var/home</code>, btrfs on LUKS (<code>/dev/mapper/luks-...</code>), 882 GiB,
opts <code>rw,relatime,seclabel,ssd,discard=async,space_cache=v2</code>; no <code>compress=</code> mount option,
so btrfs transparent compression does not interfere</td></tr>
    <tr><th>zstd CLI</th><td>Zstandard CLI v1.5.7 (Yann Collet), <code>/usr/bin/zstd</code></td></tr>
    <tr><th>Bun</th><td>1.3.14</td></tr>
    <tr><th>Node</th><td>26.3.0</td></tr>
    <tr><th>hyperfine</th><td>1.20.0</td></tr>
  </tbody>
</table>

The SSD path is encrypted (LUKS) and copy-on-write (btrfs).
 Writing many small `.zst` files there
costs real metadata,
 CoW,
 and encryption work that tmpfs does not incur.
 Running on both backends
isolates compute cost (tmpfs) from realistic build cost (SSD,
 where the real `dist/` lives).

The `node:zlib` advanced-parameter constants are identical across Bun and Node:
`ZSTD_c_compressionLevel = 100`,
 `ZSTD_c_windowLog = 101`,
 `ZSTD_c_enableLongDistanceMatching = 160`,
`ZSTD_c_nbWorkers = 400`,
 `ZSTD_c_contentSizeFlag = 200`,
 `ZSTD_c_checksumFlag = 201`.
 Neither
runtime exposes a `ZSTD_VERSION` constant,
 so the libzstd version was characterized empirically by
comparing output bytes (see correctness).

## Methodology

### Datasets

Four datasets,
 generated deterministically (seeded PRNG) so the tmpfs and SSD copies are
byte-identical.
 Per-file content verified identical across backends by path-sorted SHA-256.

<table>
  <thead><tr><th>Dataset</th><th>Files</th><th>Raw bytes</th><th>Purpose</th></tr></thead>
  <tbody>
    <tr><td><code>real</code></td><td>138</td><td>1,257,418</td><td>the actual ssg <code>dist/</code> text assets (HTML/CSS/JS/SVG/JSON/XML);
the workload the decision is about</td></tr>
    <tr><td><code>many</code></td><td>2,000</td><td>5,791,137</td><td>synthetic small HTML files;
stresses cross-file parallelism and filesystem metadata (creating many files)</td></tr>
    <tr><td><code>large</code></td><td>1</td><td>33,554,432</td><td>one 32 MiB compressible file (repeated blog text);
exposes in-frame multithreading scaling</td></tr>
    <tr><td><code>incompressible</code></td><td>1</td><td>8,388,608</td><td>8 MiB seeded random bytes;
worst-case compress time and the keep-if-smaller guard</td></tr>
  </tbody>
</table>

### Engines and the threading vocabulary

"Threading" means different mechanisms for different engines;
 the benchmark separates them
deliberately because they behave very differently.

- **zstd CLI,
   in-frame multithreading** (`-T<n>`,
   with `-T0` meaning all cores):
   splits a *single*
  file into jobs across worker threads.
   When compressing a directory recursively,
   the CLI processes
  files one at a time,
   each with in-frame MT.
   This is exactly what the current `build:compress`
  does (`-T0 -r`).
   For files smaller than zstd's job size it gives no benefit.
- **zstd CLI,
   cross-file process parallelism** (`xargs -P<n>`):
   launches one `zstd` process per
  file,
   up to `n` at a time.
   This is the only way to parallelize the CLI across many files.
- **node:
  zlib sequential** (`zstdCompressSync` in a loop):
   one thread.
- **node:
  zlib async parallel** (`Promise.all` over async `zstdCompress`,
   bounded concurrency `n`):
  the async calls run on the runtime's internal thread pool,
   giving cross-file parallelism
  in-process.
   This is the path that turns out pathological under Node.
- **node:
  zlib worker_threads** (shard files across `n` `node:worker_threads`,
   each running
  synchronous `zstdCompressSync`):
   real OS-thread,
   cross-file parallelism that bypasses the async
  zlib path entirely.
   This is the chosen Node engine.
   It was added to the matrix as a follow-up once
  the constraint that the build must run under Node made the async path's
  pathology disqualifying.
- **node:
  zlib in-frame multithreading** (`ZSTD_c_nbWorkers`):
   the direct analogue of the CLI's
  `-T<n>`,
   splitting a single file.

Bun does not honor `UV_THREADPOOL_SIZE` (verified:
 4,
 8,
 16 are indistinguishable);
 it uses its own
pool.
 Node does honor it,
 and the async-parallel Node runs set it to match the concurrency.

The async/sequential candidate is `compress-worker.ts` (run under both `bun` and `node`);
 the chosen
Node engine is `compress-node-wt.ts` (worker_threads).
 Both are reproduced below and use only
`node:` APIs so the bun-vs-node comparison is fair.

### Measurement

- **Ratio** (compressed size) is storage-independent and deterministic,
   so it was measured once,
   in
  memory,
   on the tmpfs copy,
   under both Bun and Node,
   and cross-checked against the system CLI.
- **Time** was measured with hyperfine on both backends.
   Each hyperfine benchmark clears its output
  directory before every run via `--prepare`,
   mirroring the real pipeline (the `build` task runs
  `build:clean` = `rm -rf dist` before each build,
   so `.zst` files are always freshly created,
   never
  warm overwrites).
   Run counts:
   Exp A (level sweep) `--warmup 2 --runs 5`;
   Exp B (threading)
  `--warmup 1 --runs 5` (3 for the 2000-process `many` + cli-xargs case);
   Exp C/D `--warmup 1
  --runs 4`;
   Exp E `--warmup 2 --runs 6`;
   Exp F (head-to-head) `--warmup 3 --runs 12`.

All compression used `--no-check` / `ZSTD_c_checksumFlag = 0` (zstd's default is already no
checksum) and `--no-content-size` / `ZSTD_c_contentSizeFlag = 0` to match the current task's frame
options.
 Ultra levels (20 to 22) require `--ultra` on the CLI;
 `node:zlib` reaches them directly via
the level parameter.

## Correctness and engine equivalence

Before any timing,
 the engines were checked for byte-level agreement.
 These are the load-bearing
correctness facts for the swap.

- **Bun and Node produce byte-identical output at every level on every dataset.
  ** All 110+ rows of
  the ratio sweep matched exactly between the two runtimes.
- **node:
  zlib matches the system zstd CLI v1.5.7 byte-for-byte at every level 1 to 19** on the real
  dataset (per-file totals:
   L1 419,904;
   L3 404,644;
   L6 387,591;
   L9 384,350;
   L12 381,095;
   L15
  375,774;
   L19 374,726).
   The Figma round-trip was also exhaustively verified:
   every one of the 138
  files,
   compressed by the Bun worker and by the CLI,
   decompresses to the exact original bytes (0
  mismatches across all files and both producers).
- **At ultra levels the CLI silently caps at 19 without `--ultra`** (CLI "level 22" returned 374,726,
  identical to its level 19),
   whereas `node:zlib` applies the ultra level directly and reaches
  374,698.
   So `node:zlib` is not merely equivalent;
   it can produce marginally smaller output than a
  bare CLI invocation.

The decompression direction is symmetric:
 a frame produced by the system CLI decodes correctly under
Bun's `zstdDecompressSync`,
 and a frame produced by `node:zlib` decodes correctly under the CLI.

## Ratio results

### real (1,257,418 bytes raw)

All 138 files compress smaller at every level,
 so "kept if smaller" equals the compressed total here.

<table>
  <thead><tr><th>Level</th><th>Compressed bytes</th><th>Ratio</th><th>Level</th><th>Compressed bytes</th><th>Ratio</th></tr></thead>
  <tbody>
    <tr><td>-7</td><td>675,774</td><td>53.74%</td><td>9</td><td>384,350</td><td>30.57%</td></tr>
    <tr><td>-5</td><td>623,679</td><td>49.60%</td><td>10</td><td>383,982</td><td>30.54%</td></tr>
    <tr><td>-3</td><td>564,301</td><td>44.88%</td><td>11</td><td>382,235</td><td>30.40%</td></tr>
    <tr><td>-1</td><td>404,644</td><td>32.18%</td><td>12</td><td>381,095</td><td>30.31%</td></tr>
    <tr><td>1</td><td>419,904</td><td>33.39%</td><td>13</td><td>379,868</td><td>30.21%</td></tr>
    <tr><td>2</td><td>415,136</td><td>33.01%</td><td>14</td><td>376,666</td><td>29.96%</td></tr>
    <tr><td>3</td><td>404,644</td><td>32.18%</td><td>15</td><td>375,774</td><td>29.88%</td></tr>
    <tr><td>4</td><td>398,381</td><td>31.68%</td><td>16</td><td>375,050</td><td>29.83%</td></tr>
    <tr><td>5</td><td>391,757</td><td>31.16%</td><td>17</td><td>374,927</td><td>29.82%</td></tr>
    <tr><td>6</td><td>387,591</td><td>30.82%</td><td>18</td><td>374,922</td><td>29.82%</td></tr>
    <tr><td>7</td><td>386,272</td><td>30.72%</td><td>19</td><td>374,726</td><td>29.80%</td></tr>
    <tr><td>8</td><td>385,192</td><td>30.63%</td><td>20</td><td>374,712</td><td>29.80%</td></tr>
    <tr><td></td><td></td><td></td><td>21</td><td>374,712</td><td>29.80%</td></tr>
    <tr><td></td><td></td><td></td><td>22</td><td>374,698</td><td>29.80%</td></tr>
  </tbody>
</table>

The curve drops steeply through level 6,
 flattens after level 12,
 and is essentially flat from level
16 onward.
 Level 15 (375,774) is within 1,048 bytes (0.08 percentage points) of level 19 (374,726).
Negative ("fast") levels trade a lot of ratio for speed and are not useful for precompressed,
served-many-times assets.
 Note that the committed `dist/.zst` baseline of 404,644 bytes equals level
3,
 but that figure came from a prior `--adapt` run and is environment-specific,
 not a fixed property
of `--adapt`.

### many (5,791,137 bytes raw, 2,000 files)

<table>
  <thead><tr><th>Level</th><th>Compressed bytes</th><th>Ratio</th></tr></thead>
  <tbody>
    <tr><td>1</td><td>1,835,464</td><td>31.69%</td></tr>
    <tr><td>3</td><td>1,781,447</td><td>30.76%</td></tr>
    <tr><td>6</td><td>1,767,604</td><td>30.52%</td></tr>
    <tr><td>9</td><td>1,760,041</td><td>30.39%</td></tr>
    <tr><td>12</td><td>1,710,352</td><td>29.53%</td></tr>
    <tr><td>15</td><td>1,697,316</td><td>29.31%</td></tr>
    <tr><td>16 to 22</td><td>1,689,927</td><td>29.18%</td></tr>
  </tbody>
</table>

Same shape:
 a knee around level 12 to 16,
 flat thereafter.

### large (33,554,432 bytes raw, 1 file) and long-distance matching

The large file is repeated blog text,
 so it is hugely compressible.
 This dataset is where
long-distance matching (LDM) can matter,
 because LDM needs a large window over a large input.

<table>
  <thead><tr><th>Level</th><th>No LDM bytes</th><th>No LDM ratio</th><th>LDM bytes</th><th>LDM ratio</th></tr></thead>
  <tbody>
    <tr><td>-5</td><td>7,547,185</td><td>22.49%</td><td>276,636</td><td>0.82%</td></tr>
    <tr><td>1</td><td>5,016,925</td><td>14.95%</td><td>190,078</td><td>0.57%</td></tr>
    <tr><td>3</td><td>175,217</td><td>0.52%</td><td>174,173</td><td>0.52%</td></tr>
    <tr><td>6</td><td>160,534</td><td>0.48%</td><td>164,371</td><td>0.49%</td></tr>
    <tr><td>12</td><td>156,407</td><td>0.47%</td><td>160,430</td><td>0.48%</td></tr>
    <tr><td>19</td><td>147,097</td><td>0.44%</td><td>147,046</td><td>0.44%</td></tr>
    <tr><td>22</td><td>146,701</td><td>0.44%</td><td>146,702</td><td>0.44%</td></tr>
  </tbody>
</table>

LDM dramatically helps *low* levels on a large redundant file (level 1:
 14.95% to 0.57%) but is
neutral or slightly worse at high levels (whose windows are already large),
 and it does nothing for
small files.
 Since the ssg assets are small per-file,
 LDM is irrelevant to the decision;
 it is
measured here only to rule it out.

### incompressible (8,388,608 bytes raw, 1 file)

At every level the compressed output is 8,388,806 bytes,
 which is 198 bytes *larger* than the raw
input (zstd frame overhead on already-random data).
 The keep-if-smaller policy therefore keeps the
raw 8,388,608 bytes and writes no `.zst`.
 This is the concrete justification for keep-if-smaller:
without it,
 the build would emit `.zst` companions that are larger than their sources,
 which
`wireSize` would then report as the (inflated) wire size.

## Timing results

### Experiment A: level sweep on real, per engine

Mean wall time in milliseconds,
 real dataset,
 output cleared before each run.
 This experiment
predates the Bun-removal constraint,
 so it sweeps `bun-par` and `node-par` (async);
 the chosen Node
worker_threads engine is benchmarked separately in the Node parallelism section.
 `cli-rec-T0` is the
current style,
 `node-par` (async) is shown to be pathological,
 and `node-seq` is the Node sequential
baseline that worker_threads improves on.
 Bun numbers are reference only.

tmpfs backend:

<table>
  <thead><tr><th>Level</th><th>cli-rec-T0</th><th>bun-par-c16</th><th>bun-seq</th><th>node-par-c16</th><th>node-seq</th></tr></thead>
  <tbody>
    <tr><td>1</td><td>12.2</td><td>37.2</td><td>34.1</td><td>71.4</td><td>65.9</td></tr>
    <tr><td>2</td><td>12.3</td><td>38.2</td><td>34.4</td><td>66.5</td><td>64.1</td></tr>
    <tr><td>3</td><td>13.5</td><td>35.4</td><td>34.4</td><td>71.7</td><td>62.6</td></tr>
    <tr><td>4</td><td>16.2</td><td>36.5</td><td>35.3</td><td>78.1</td><td>65.9</td></tr>
    <tr><td>5</td><td>17.6</td><td>35.3</td><td>37.4</td><td>80.0</td><td>66.9</td></tr>
    <tr><td>6</td><td>20.1</td><td>36.0</td><td>40.5</td><td>82.9</td><td>71.6</td></tr>
    <tr><td>7</td><td>20.8</td><td>36.8</td><td>41.4</td><td>97.5</td><td>70.6</td></tr>
    <tr><td>8</td><td>22.4</td><td>35.4</td><td>41.2</td><td>98.4</td><td>70.9</td></tr>
    <tr><td>9</td><td>26.4</td><td>36.2</td><td>46.8</td><td>134.6</td><td>76.5</td></tr>
    <tr><td>10</td><td>27.5</td><td>37.4</td><td>51.3</td><td>210.7</td><td>76.6</td></tr>
    <tr><td>11</td><td>42.4</td><td>41.9</td><td>64.7</td><td>216.2</td><td>94.1</td></tr>
    <tr><td>12</td><td>51.9</td><td>40.6</td><td>69.3</td><td>580.0</td><td>102.0</td></tr>
    <tr><td>13</td><td>66.2</td><td>42.7</td><td>84.4</td><td>484.3</td><td>117.4</td></tr>
    <tr><td>14</td><td>80.5</td><td>44.6</td><td>99.8</td><td>654.9</td><td>129.4</td></tr>
    <tr><td>15</td><td>94.3</td><td>47.1</td><td>110.0</td><td>819.3</td><td>142.0</td></tr>
    <tr><td>16</td><td>134.8</td><td>55.5</td><td>145.2</td><td>521.5</td><td>181.5</td></tr>
    <tr><td>17</td><td>150.7</td><td>57.1</td><td>159.2</td><td>662.0</td><td>195.0</td></tr>
    <tr><td>18</td><td>171.4</td><td>67.7</td><td>178.5</td><td>669.6</td><td>208.1</td></tr>
    <tr><td>19</td><td>216.6</td><td>72.0</td><td>215.1</td><td>1007.4</td><td>256.9</td></tr>
    <tr><td>20</td><td>223.9</td><td>72.2</td><td>221.0</td><td>1966.3</td><td>297.7</td></tr>
    <tr><td>21</td><td>224.0</td><td>71.5</td><td>223.8</td><td>3654.5</td><td>256.6</td></tr>
    <tr><td>22</td><td>233.9</td><td>75.3</td><td>230.1</td><td>7837.6</td><td>266.6</td></tr>
  </tbody>
</table>

SSD backend:

<table>
  <thead><tr><th>Level</th><th>cli-rec-T0</th><th>bun-par-c16</th><th>bun-seq</th><th>node-par-c16</th><th>node-seq</th></tr></thead>
  <tbody>
    <tr><td>1</td><td>21.0</td><td>47.6</td><td>43.1</td><td>81.9</td><td>75.3</td></tr>
    <tr><td>3</td><td>23.1</td><td>178.1*</td><td>44.1</td><td>86.7</td><td>76.3</td></tr>
    <tr><td>6</td><td>40.2*</td><td>47.7</td><td>52.1</td><td>92.7</td><td>81.4</td></tr>
    <tr><td>9</td><td>37.6</td><td>49.4</td><td>193.4*</td><td>143.7</td><td>86.8</td></tr>
    <tr><td>12</td><td>83.8*</td><td>53.5</td><td>83.3</td><td>561.7</td><td>111.3</td></tr>
    <tr><td>15</td><td>113.1</td><td>62.8</td><td>123.0</td><td>829.4</td><td>154.8</td></tr>
    <tr><td>16</td><td>158.5</td><td>70.3</td><td>161.7</td><td>459.1</td><td>192.5</td></tr>
    <tr><td>19</td><td>232.1</td><td>89.8</td><td>231.1</td><td>1012.6</td><td>263.3</td></tr>
    <tr><td>22</td><td>254.0</td><td>92.7</td><td>250.9</td><td>7543.9</td><td>279.7</td></tr>
  </tbody>
</table>

Rows marked `*` had a large standard deviation (a transient outlier run);
 see caveats.
 The SSD table
is abridged to representative levels;
 the full 22-level SSD data was collected and matches the tmpfs
shape offset by roughly 10 to 20 ms of I/O.

What this shows:

- **`bun-par` is nearly flat and far cheaper at the levels that matter.
  ** It has a higher floor
  (~35 ms,
   from Bun startup plus thread-pool spin-up) but barely rises with level:
   level 19 is 72 ms
  versus `cli-rec-T0` 217 ms and `bun-seq` 215 ms. From level 16 to 22 it is essentially flat
  (72 to 75 ms).
- **The CLI is cheapest only at trivial levels** (level 1,
   12 ms),
   where the work is so small that
  process startup dominates and parallelism cannot pay for its floor.
   The crossover with `bun-par`
  is around level 11 to 12;
   above it,
   `bun-par` wins decisively.
- **`node-par` is pathological** and gets worse with level,
   exploding to 7.8 seconds at level 22.
  This is discussed in its own section;
   it is the reason Node is not used for the parallel path.

### Experiment B: threading scaling at level 19

Mean ms,
 level 19,
 varying the thread/concurrency count.
 This isolates how each parallelism
mechanism scales.

real dataset,
 tmpfs:

<table>
  <thead><tr><th>Threads</th><th>bun-par</th><th>cli-rec (in-frame)</th><th>cli-xargs (cross-file procs)</th><th>node-par</th></tr></thead>
  <tbody>
    <tr><td>1</td><td>275.0*</td><td>244.1*</td><td>1722.2*</td><td>4684.5</td></tr>
    <tr><td>8</td><td>58.4</td><td>216.2</td><td>1467.9</td><td>1185.6</td></tr>
    <tr><td>16</td><td>60.2</td><td>215.2</td><td>1470.8*</td><td>1099.2</td></tr>
  </tbody>
</table>

real dataset,
 SSD:

<table>
  <thead><tr><th>Threads</th><th>bun-par</th><th>cli-rec (in-frame)</th><th>cli-xargs (cross-file procs)</th><th>node-par</th></tr></thead>
  <tbody>
    <tr><td>1</td><td>252.1</td><td>239.6</td><td>1657.3</td><td>4542.1</td></tr>
    <tr><td>8</td><td>91.1</td><td>230.4</td><td>1705.6*</td><td>1030.5</td></tr>
    <tr><td>16</td><td>86.0</td><td>248.2*</td><td>2189.7*</td><td>1019.1</td></tr>
  </tbody>
</table>

many dataset (2,000 files),
 tmpfs:

<table>
  <thead><tr><th>Threads</th><th>bun-par</th><th>cli-xargs (cross-file procs)</th><th>node-par</th></tr></thead>
  <tbody>
    <tr><td>1</td><td>823.4</td><td>22,541</td><td>59,515</td></tr>
    <tr><td>8</td><td>180.6</td><td>23,357</td><td>14,019</td></tr>
    <tr><td>16</td><td>154.8</td><td>23,978</td><td>13,158</td></tr>
  </tbody>
</table>

many dataset (2,000 files),
 SSD:

<table>
  <thead><tr><th>Threads</th><th>bun-par</th><th>cli-xargs (cross-file procs)</th><th>node-par</th></tr></thead>
  <tbody>
    <tr><td>1</td><td>973.7</td><td>23,971</td><td>61,118</td></tr>
    <tr><td>8</td><td>477.4*</td><td>28,541*</td><td>14,510</td></tr>
    <tr><td>16</td><td>573.1*</td><td>28,107*</td><td>13,742</td></tr>
  </tbody>
</table>

What this shows:

- **`bun-par` scales well across files** (real:
   275 ms at T1 to ~60 ms at T8/T16,
   ~4.5x;
   many:
   823 ms
  to 155 ms,
   ~5.3x) and is the fastest everywhere it is measured.
- **The CLI's in-frame `-T<n>` does not scale for small files** (real:
   244 to 215 ms from T1 to T16),
  confirming that the current task's `-T0` buys nothing on this workload.
- **The CLI's cross-file process parallelism is catastrophic for many files.
  ** On the 2,000-file set
  it takes ~23 to 28 seconds and does not improve with more processes,
   because spawning 2,000 `zstd`
  processes dominates everything.
   On the 138-file `real` set it is ~1.5 to 2.2 seconds,
   still ~25x
  slower than `bun-par`.
   Process-per-file is simply the wrong model.
- **`node-par` is slow and erratic**:
   on `many` it ranges from 13 seconds (T16) to 60 seconds (T1).
  Notably its *slowest* point is concurrency 1,
   which rules out a "too much concurrency" explanation.

### Experiment C: in-frame multithreading on the 32 MiB file

Does the CLI's `-T<n>` (and `node:zlib`'s `nbWorkers`) help when the file is actually large?
 Mean ms,
single 32 MiB file.

<table>
  <thead><tr><th>Config</th><th>T1</th><th>T8</th><th>T16</th></tr></thead>
  <tbody>
    <tr><td>CLI level 12 (tmpfs)</td><td>62.9</td><td>51.5</td><td>50.1</td></tr>
    <tr><td>CLI level 19 (tmpfs)</td><td>200.0</td><td>193.2</td><td>194.5</td></tr>
    <tr><td>node nbWorkers level 12 (tmpfs)</td><td>119.6</td><td>115.1</td><td>114.1</td></tr>
    <tr><td>node nbWorkers level 19 (tmpfs)</td><td>278.9</td><td>252.8</td><td>247.6</td></tr>
  </tbody>
</table>

Even on a 32 MiB file,
 in-frame MT gives only a mild speedup at level 12 (1.26x for the CLI) and
essentially nothing at level 19 (the window already spans most of the file,
 so there is little to
parallelize).
 This confirms that in-frame MT is not a useful lever for the ssg workload,
 whose files
are three orders of magnitude smaller.
 SSD numbers match within noise.

### Experiment D: long-distance matching wall time (32 MiB file)

<table>
  <thead><tr><th>Config</th><th>tmpfs</th><th>SSD</th></tr></thead>
  <tbody>
    <tr><td>node level 1, no LDM</td><td>114.2</td><td>114.6</td></tr>
    <tr><td>node level 1, LDM</td><td>83.9</td><td>83.2</td></tr>
    <tr><td>node level 19, no LDM</td><td>233.3</td><td>225.8</td></tr>
    <tr><td>node level 19, LDM</td><td>257.7</td><td>260.2</td></tr>
    <tr><td>CLI level 19, no LDM</td><td>197.6</td><td>196.1</td></tr>
    <tr><td>CLI level 19, LDM</td><td>213.1</td><td>209.0</td></tr>
  </tbody>
</table>

LDM is both faster and far smaller at level 1 on a large redundant file (it finds the long-range
repeats cheaply),
 but it is pure overhead at level 19 (slower,
 same size).
 Irrelevant for small ssg
assets;
 measured to rule out.

### Experiment E: incompressible worst case and keep-if-smaller cost

8 MiB random file,
 level 19.

<table>
  <thead><tr><th>Config</th><th>tmpfs</th><th>SSD</th></tr></thead>
  <tbody>
    <tr><td>node parallel, no keep-if-smaller</td><td>930.5</td><td>907.8</td></tr>
    <tr><td>node parallel, keep-if-smaller</td><td>939.9</td><td>911.9</td></tr>
    <tr><td>CLI</td><td>807.8</td><td>814.9</td></tr>
  </tbody>
</table>

Incompressible data is the slowest per byte (no early exit;
 ~0.8 to 0.9 s for 8 MiB).
 The
keep-if-smaller check costs ~10 ms (under 1%) and,
 more importantly,
 prevents writing a `.zst` that
is larger than its source.
 In the real build this branch is only reached for the rare text file that
fails to compress;
 the common incompressible assets (images,
 fonts) are skipped by extension before
any compression is attempted.

## Node parallelism: worker_threads is the best optimized implementation

Because the workspace has migrated off Bun,
 the parallel engine must work well under Node,
 and Node's
async `zstdCompress` does not (see the pathology section).
 The fast Node path is to shard files
across `node:worker_threads`,
 each worker running synchronous `zstdCompressSync`.
 This avoids Node's
async zlib per-call overhead entirely and uses real OS threads.
 The worker_threads compressor was
validated for correctness (138 files,
 all decode to the exact originals,
 total 374,726 bytes = level
19).

Node engines compared at level 19 (mean ms):

<table>
  <thead><tr><th>Engine</th><th>real (138 files) tmpfs</th><th>real SSD</th><th>many (2,000 files) tmpfs</th></tr></thead>
  <tbody>
    <tr><td>sequential <code>zstdCompressSync</code> loop</td><td>243.4</td><td>257.4</td><td>787.3</td></tr>
    <tr><td>worker_threads, 4 workers</td><td>172.0</td><td>&mdash;</td><td>312.3</td></tr>
    <tr><td><b>worker_threads, 8 workers</b></td><td><b>166.9</b></td><td><b>171.2</b></td><td><b>252.9</b></td></tr>
    <tr><td>worker_threads, 16 workers</td><td>205.0</td><td>207.7</td><td>283.1</td></tr>
    <tr><td>async <code>zstdCompress</code> + Promise.all (16)</td><td>980.9</td><td>987.3</td><td>(pathological)</td></tr>
  </tbody>
</table>

worker_threads with 8 workers is ~1.5x faster than sequential on the 138-file real set and ~3.1x
faster on the 2,000-file set,
 which matters as the blog grows.

A full worker-count sweep (1 to 32 workers) on the real set at level 19 shows the optimum is a
**plateau,
 not a single point**:
 workers 4,
 8,
 and 10 are statistically tied,
 and performance
degrades steadily above ~12 as extra V8 isolates cost more than they return (zstd is CPU-bound,
 so
SMT past the 8 physical cores gives little).

<table>
  <thead><tr><th>Workers</th><th>1</th><th>2</th><th>3</th><th>4</th><th>6</th><th>8</th><th>10</th><th>12</th><th>16</th><th>24</th><th>32</th></tr></thead>
  <tbody>
    <tr><td>tmpfs ms</td><td>281</td><td>365*</td><td>197</td><td>170</td><td>185</td><td>169</td><td>168</td><td>187</td><td>210</td><td>261</td><td>309</td></tr>
    <tr><td>SSD ms</td><td>298</td><td>227</td><td>204</td><td>178</td><td>196</td><td>177</td><td>176</td><td>196</td><td>217</td><td>277</td><td>311</td></tr>
  </tbody>
</table>

(* the W2 tmpfs cell had an outlier run,
 stddev +/- 406 ms;
 treat it as noise.
) The practical rule is
**workers in the 4 to 10 range,
 around the physical-core count (8 here);
 never oversubscribe past
~12**.
 A naive `os.availableParallelism()` returns 16 on this chip,
 which is already ~25% past the
optimum,
 so the implementation should cap at roughly the physical-core count rather than use the
logical count directly.
 There is also a fixed worker-spawn floor of roughly 120 ms (V8 isolate
startup),
 which is why worker_threads helps at high levels and on large file counts but not at
trivial levels,
 where sequential or the CLI's lower startup wins.

The SSD `many` numbers are omitted from the table because parallel writers contending on btrfs
copy-on-write plus LUKS encryption produced very high variance there (stddev up to +/- 458 ms across
worker counts);
 the tmpfs `many` numbers isolate the compute scaling cleanly.

worker_threads level curve (real,
 tmpfs,
 8 workers):
 level 12 = 130.4 ms,
 level 15 = 136.5 ms,
 level
19 = 169.1 ms,
 level 22 = 177.2 ms. Flatter than the sequential curve (the per-file work overlaps
across workers),
 and level 15 saves ~33 ms over level 19 for 0.08 ratio points.

## The Node async parallel pathology (observation, mechanism not isolated)

Node's async `zstdCompress` driven by `Promise.all` is anomalously slow in this benchmark,
 and the
cause was not isolated.
 Reporting the observation,
 not a mechanism:

- At level 19 on the 138-file `real` set it takes ~1.0 second,
   versus Bun's 72 ms (about 14x slower)
  and Node's own *sequential* `zstdCompressSync` at 257 ms (so parallel is ~4x slower than
  sequential on the same runtime).
- It worsens sharply with level:
   tmpfs `real` goes 580 ms (L12),
   1007 ms (L19),
   1966 ms (L20),
  3654 ms (L21),
   7838 ms (L22).
   The sequential path on Node does not do this (267 ms at L22).
- In the threading scan its slowest point is concurrency 1 (4.7 s),
   with more concurrency making it
  faster,
   which contradicts an "oversubscription / too many large contexts" explanation.

A plausible-sounding memory-pressure story was considered and rejected because the concurrency-1
data contradicts it.
 The honest characterization is that Node's async zstd dispatch carries a large
per-call overhead here that compounds at high levels,
 but the benchmark did not instrument the
runtime to prove the mechanism.
 It does not affect the decision:
 the build runs under Node,
 where the
synchronous worker path avoids the async overhead measured here.
 It is documented so a future reader does not
"optimize" the build by switching it to Node async parallel.

## Caveats and threats to validity

- **Single machine,
   single run of the suite.
  ** All numbers are from one AMD Ryzen 7 8700F.
   Absolute
  times will differ elsewhere;
   the *relative* conclusions (Bun parallel beats the CLI at high
  levels;
   CLI cross-file and Node parallel are catastrophic;
   in-frame MT is useless for small files)
  are large enough to travel.
- **CPU governor was not pinned.
  ** It reported 83% scaling,
   not locked to performance.
   Combined with
  background activity,
   this produced occasional outlier runs (rows marked `*` above,
   e.g. `bun-par`
  L3 on SSD at 178 ms +/- 290,
   and the current-task SSD head-to-head at 88 ms +/- 121).
   For those
  rows,
   prefer the minimum or median over the mean.
   The headline `bun-par` numbers are tight
  (stddev ~1 to 2 ms) and trustworthy;
   only flagged rows are noisy.
- **`--adapt` is nondeterministic.
  ** It tunes its compression level to observed I/O throughput,
   so
  the current task produces different output sizes and different wall times on different disks (13 ms
  / level-3-equivalent on tmpfs versus 88 ms +/- 121 on SSD here).
   This means the current build's
  output is not reproducible across environments.
   A fixed level is,
   which is an independent argument
  for the change beyond ratio and speed.
- **Page cache.
  ** Inputs were warm in cache (realistic:
   `dist/` was just written by earlier build
  phases).
   The SSD cost captured is dominated by writing and creating `.zst` files (CoW + LUKS +
  metadata),
   not cold input reads.
   This matches the real pipeline,
   where compression immediately
  follows generation.
- **Worker startup is included** in every node timing (Bun/Node process launch,
   ~10 to 80 ms).
   This
  is correct for the build (the task spawns one process) and is the main reason `bun-par` has a
  ~35 ms floor;
   it does not affect the per-level deltas,
   which are dominated by compression work.
- **tmpfs is RAM.
  ** The tmpfs numbers isolate compute and are not representative of the real build's
  I/O;
   the SSD numbers are.
   Both are reported so the compute-versus-I/O split is visible.

## Which precompressed extensions still benefit from zstd

The compressor skips a blocklist of already-compressed extensions without reading them.
 The blocklist
is purely an optimization:
 keep-if-smaller (write `.zst` only when strictly smaller) is the real
safety net,
 so the only cost of a wrong "skip" is a missed saving,
 and the only cost of a wrong
"compress" is wasted CPU.
 The question is therefore empirical:
 which "already compressed" formats does
a level-19 zstd pass still shrink on real content?

### Methodology

Representative real content,
 not synthetic tones (a pure sine compresses pathologically and would
mislead).
 Three frames,
 a 20-second audio clip at three timestamps,
 and a 6-second video clip were
taken from a 1920x816 AV1 cartoon (`ffmpeg`),
 plus this site's own `dist/` assets.
 Each frame was
re-encoded to every image format (`cwebp`,
 `magick`,
 `ffmpeg` with `libjxl` / `libaom-av1`);
 the audio
clips to every audio codec;
 the video clip to h264 / vp9.
 Every output was compressed with the exact
production settings (level 19,
 content-size off,
 checksum off) and compared to its source size.
 Audio
codecs were measured across three independent clips;
 the spread was within 0.3 points.

### Results

<table>
<tr><th>Extension</th><th>Sample</th><th>zstd-19 savings</th><th>Decision</th></tr>
<tr><td>aac</td><td>cartoon audio, n=3</td><td>+6.0 to 6.3%</td><td>remove (compress)</td></tr>
<tr><td>m4a</td><td>cartoon audio, n=3</td><td>+5.8 to 6.0%</td><td>remove (compress)</td></tr>
<tr><td>ogg (vorbis)</td><td>cartoon audio, n=3</td><td>+3.6 to 3.9%</td><td>remove (compress)</td></tr>
<tr><td>png</td><td>site icons + frames</td><td>+0 to 15% (icons compress,
photos do not)</td><td>remove (compress)</td></tr>
<tr><td>jpg / jpeg</td><td>site + frames, n=7</td><td>+1.0 to 1.3% (every file)</td><td>remove (compress)</td></tr>
<tr><td>mp3</td><td>cartoon audio, n=3</td><td>+0.7%</td><td>keep (skip)</td></tr>
<tr><td>flac</td><td>cartoon audio, n=3</td><td>+0.3%</td><td>keep (skip)</td></tr>
<tr><td>avif</td><td>site + frames</td><td>~0% (only sub-kB files compressed)</td><td>keep (skip)</td></tr>
<tr><td>webp (lossy + lossless)</td><td>frames, n=6</td><td>~0%</td><td>keep (skip)</td></tr>
<tr><td>gif</td><td>frames, n=3</td><td>~0%</td><td>keep (skip)</td></tr>
<tr><td>jxl (lossy + lossless)</td><td>frames, n=6</td><td>~0%</td><td>keep (skip)</td></tr>
<tr><td>woff2</td><td>site fonts, n=8</td><td>+0.1%</td><td>keep (skip)</td></tr>
<tr><td>mp4 / mov (h264)</td><td>cartoon video</td><td>+0.3%</td><td>keep (skip)</td></tr>
<tr><td>webm (vp9)</td><td>cartoon video</td><td>+0.1%</td><td>keep (skip)</td></tr>
</table>

`woff`,
 `gz`,
 and `br` were not sampled (no source on hand) and are kept on first principles:
 WOFF
stores deflate-compressed font tables (like woff2's brotli),
 and gzip / brotli are compression formats
by definition.
 `zst` is always skipped so a `.zst` is never recompressed.

Removing png / jpg / jpeg / ogg / aac / m4a from the blocklist raised this site's compressed count
from 176 to 185 files and total savings from 1.18 MB to 1.28 MB,
 with every produced `.zst` still
round-tripping to its source.

## Proposed implementation

Not yet applied.
 Sketch for when it is.

### Figma decompression

In `package/figma/kiwi/src/index.ts`,
 `decompressZstd` collapses to:

```ts
async function decompressZstd(data: Uint8Array,): Promise<Uint8Array> {
  const { zstdDecompressSync, } = await import('node:zlib');
  return new Uint8Array(
    zstdDecompressSync(Buffer.from(data,),),
  );
}
```

It stays `async` to preserve the public call contract,
 mirroring the `inflateRawSync` schema decode
already in the same function.
 Remove `nano-spawn` from `package/figma/kiwi/package.json`
(its only use was the removed fallback).
 Verify with the package's existing integration test,
 which
decodes real `.fig` / `.deck` / `.jam` files (`bun package/figma/kiwi/src/index.unit.test.ts`).

### ssg compression

Add `package/ssg/aquati.cat/src/build/compress.ts` beside `postprocess.ts`,
 and change the
`build:compress` task from the `zstd` shell command to `node src/build/compress.ts` (Node,
 since Bun
is being removed).
 The script:
 on the main thread,
 walk `dist/`,
 skip known-incompressible extensions
without reading them,
 shard the remaining files across `node:worker_threads` (worker count = physical
cores,
 roughly 8;
 cap below the logical count),
 and in each worker compress with synchronous
`zstdCompressSync` at level 19 (`ZSTD_c_contentSizeFlag = 0`),
 writing `<file>.zst` only when strictly
smaller than the source.
 The core engine is exactly the `compress-node-wt.ts` used in this benchmark
(reproduced below),
 adapted into the package's logging and module conventions.
 If the worker_threads
machinery is judged not worth its complexity for the current `dist/` size,
 the sequential
`zstdCompressSync` fallback is a one-file change away and costs ~75 ms more on today's `dist/` (but
~3x more as the file count grows).

### Tool and config cleanup

Remove `"github:facebook/zstd" = "latest"` from `mise.toml` and `mise.no-env.toml`.

## Reproduction

The benchmark scripts were scratch artifacts under `/tmp/agent/zbench/` (ephemeral,
 RAM-backed) and
a throwaway SSD data dir under `/var/home/user/.cache/agent-zstd-bench/`.
 The two load-bearing units
are reproduced here in full so the measurement can be rebuilt.

`compress-node-wt.ts`,
 the chosen Node engine (worker_threads + synchronous zstd;
 run under `node`):

```ts
import { Worker, isMainThread, workerData, parentPort, } from 'node:worker_threads';
import { readdirSync, readFileSync, writeFileSync, mkdirSync, statSync, } from 'node:fs';
import { dirname, join, } from 'node:path';
import zlib from 'node:zlib';

const c = zlib.constants;

function compressOne(path, IN, OUT, params, keep) {
  const raw = readFileSync(path,);
  const dest = join(OUT, path.slice(IN.length,),);
  mkdirSync(dirname(dest,), { recursive: true, },);
  const out = zlib.zstdCompressSync(raw, { params, },);
  if (keep && out.length >= raw.length) { writeFileSync(dest, raw,); return; }
  writeFileSync(`${dest}.zst`, out,);
}

if (isMainThread) {
  const arg = (name, fb) => {
    const i = process.argv.indexOf(`--${name}`,);
    if (i < 0) return fb;
    const n = process.argv[i + 1];
    return (n === undefined || n.startsWith('--',)) ? 'true' : n;
  };
  const IN = arg('in',), OUT = arg('out',), LEVEL = Number(arg('level', '19',),);
  const WORKERS = Number(arg('workers', '8',),), KEEP = arg('keep-if-smaller',) === 'true';
  const files = readdirSync(IN, { recursive: true, },)
    .map(rel => join(IN, String(rel,),),).filter(p => statSync(p,).isFile());
  const nWorkers = Math.min(WORKERS, files.length,) || 1;
  const shards = Array.from({ length: nWorkers, }, () => [],);
  files.forEach((f, i) => shards[i % nWorkers].push(f,),);
  await Promise.all(shards.map(shard => new Promise((resolve, reject,) => {
    const w = new Worker(new URL(import.meta.url,), { workerData: { shard, IN, OUT, LEVEL, KEEP, }, },);
    w.on('error', reject,);
    w.on('exit', code => code === 0 ? resolve() : reject(new Error(`worker exit ${code}`,),),);
  },),),);
}
else {
  const { shard, IN, OUT, LEVEL, KEEP, } = workerData;
  const params = { [c.ZSTD_c_compressionLevel]: LEVEL, [c.ZSTD_c_contentSizeFlag]: 0, };
  for (const path of shard) compressOne(path, IN, OUT, params, KEEP,);
  parentPort.postMessage('done',);
}
```

`compress-worker.ts`,
 the cross-runtime sequential/async unit used to measure the sequential and
(rejected) async paths under both `bun` and `node`:

```ts
import { readdirSync, readFileSync, writeFileSync, mkdirSync, statSync, } from 'node:fs';
import { dirname, join, } from 'node:path';
import zlib from 'node:zlib';
import { promisify, } from 'node:util';

const zstdCompress = promisify(zlib.zstdCompress,);
const c = zlib.constants;

function arg(name, fallback) {
  const i = process.argv.indexOf(`--${name}`,);
  if (i < 0) return fallback;
  const next = process.argv[i + 1];
  if (next === undefined || next.startsWith('--',)) return 'true';
  return next;
}

const IN = arg('in',);
const OUT = arg('out',);
const LEVEL = Number(arg('level', '19',),);
const MODE = arg('mode', 'par',);
const CONCURRENCY = Number(arg('concurrency', '16',),);
const LONG = arg('long',) === 'true';
const NBWORKERS = Number(arg('nbworkers', '0',),);
const KEEP_IF_SMALLER = arg('keep-if-smaller',) === 'true';

const params = { [c.ZSTD_c_compressionLevel]: LEVEL, [c.ZSTD_c_contentSizeFlag]: 0, };
if (LONG) { params[c.ZSTD_c_enableLongDistanceMatching] = 1; params[c.ZSTD_c_windowLog] = 27; }
if (NBWORKERS > 0) params[c.ZSTD_c_nbWorkers] = NBWORKERS;

const files = readdirSync(IN, { recursive: true, },)
  .map(rel => join(IN, String(rel,),),)
  .filter(p => statSync(p,).isFile());

async function one(path) {
  const raw = readFileSync(path,);
  const dest = join(OUT, path.slice(IN.length,),);
  mkdirSync(dirname(dest,), { recursive: true, },);
  const out = MODE === 'seq'
    ? zlib.zstdCompressSync(raw, { params, },)
    : await zstdCompress(raw, { params, },);
  if (KEEP_IF_SMALLER && out.length >= raw.length) { writeFileSync(dest, raw,); return raw.length; }
  writeFileSync(`${dest}.zst`, out,); return out.length;
}

if (MODE === 'seq') { for (const f of files) await one(f,); }
else {
  let cursor = 0;
  const worker = async () => { while (cursor < files.length) { await one(files[cursor++]!,); } };
  await Promise.all(Array.from({ length: Math.min(CONCURRENCY, files.length,), }, worker,),);
}
```

The full suite consisted of:
 `prep.ts` (deterministic dataset generation on both backends),
`ratio.ts` (in-memory level sweep under bun and node,
 plus a CLI cross-check),
 and `time-matrix.ts`
(the hyperfine orchestrator implementing experiments A through F over both backends,
 clearing the
output dir before each run).
 Re-running requires only `hyperfine`,
 `zstd`,
 `bun`,
 and `node` at the
versions in the environment table;
 the datasets rebuild deterministically from the ssg `dist/`.

To reproduce the headline comparison directly:

```sh
# build dist first (mise run //package/ssg/aquati.cat:build:site), then:
hyperfine --warmup 3 --runs 12 --prepare 'rm -rf /tmp/out && mkdir -p /tmp/out' \
  --command-name current 'cd dist && zstd -q -z -f --no-check --no-content-size -T0 --adapt -r . --output-dir-mirror /tmp/out' \
  --command-name 'cli-matched-L19' 'cd dist && zstd -q -19 --ultra --no-check --no-content-size -T0 -r . --output-dir-mirror /tmp/out -f' \
  --command-name 'proposed-node-wt-L19' 'node compress-node-wt.ts --in dist --out /tmp/out --level 19 --workers 8 --keep-if-smaller'
```
