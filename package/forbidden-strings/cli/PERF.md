# Performance

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Measured wall-clock budget for the `forbidden-strings` scanner.
Numbers below are not aspirational targets;
 they are reproducible measurements
against the binary built from this package's `src/`.

## Runtime rule precompilation

Issue #456 measured repeated startup compilation against the same 10,206-rule runtime file and two targets:

```text
run 1   139.81s
run 2   139.26s
```

Those values are issue-author evidence,
not a measurement reproduced by the current implementation session.
The runtime cache stores a scanner-version and platform-specific compiled artifact under a per-user content-addressed
cache slot.
A cache miss compiles authoritative text and attempts atomic publication;
a valid hit decodes and validates the artifact without compiling rule patterns.

Acceptance requires separate release-binary measurements for:

- Unchanged-build run-to-run timing variation.
- First compilation and publication.
- Repeated validated artifact load.
- A source-content change that selects a new cache slot.
- A planted match proving the timed artifact enforces the intended rule.

### Rejected all-rules `RegexSet` artifact

A bounded reproduction used the issue author's actual fixture:

- 10,206 rules.
- 1,109,429 source bytes.
- One planted positive-control target and one clean control.
- Release `0.4.0` pre-hybrid implementation.
- Debian bookworm-slim image
  `cae69e86e0b024efa293e7ae0c5760d765422473437056e03d7d941fdf24dd8e`.
- 2 GiB memory,
  2 CPUs,
  256-process limit,
  and no network.

```text
first compile + publish   146,320 ms
artifact size             162,668,054 bytes
warm minimum                1,385 ms
warm median                 1,428.5 ms
warm p95                    1,459 ms
warm maximum                1,467 ms
samples                        30
```

The planted rule matched and the clean control exited 0.
The artifact had mode `0600`.
The run proves that precompiling every bare literal as an independent `RegexSet` engine moves the cost but does not meet
the sub-100 ms commit budget.
The implementation therefore pivots to a hybrid runtime matcher:
one direct Aho-Corasick set for exact literals and a precompiled `RegexSet` only for explicit regex rules.

### Accepted hybrid artifact

The same actual fixture,
container image,
resource bounds,
and positive and clean controls produced:

```text
first compile + publish      108 ms
artifact size          1,232,050 bytes
warm minimum                  85 ms
warm median                   88.0 ms
warm p95                      92 ms
warm maximum                  93 ms
samples                       30
```

The hybrid reduced artifact size by 99.24%,
made first compilation 1,354.8 times faster,
and made the warm median 16.2 times faster than the rejected all-engine artifact.
The complete observed unchanged-build band of 85 to 93 ms meets the sub-100 ms objective.
The planted rule still matched,
the clean control still exited 0,
and the artifact retained mode `0600`.

This is the accepted issue #456 result.
It moves exact literals out of per-rule regex engines
while retaining explicit regex rules in the precompiled engine path.

## Headline numbers

Cutover re-measure for the `forbidden-regex` engine,
 2026-07-17,
 issue #387.
 On AMD Ryzen 7 8700F (16 threads),
hyperfine 1.20.0,
 embedded ported baseline only (`--builtin-rules`,
 no user rules file):

- This repo cold start (one small file):
   **13.7 ms ± 0.4 ms**
- This repo full `--all`:
   **131.9 ms ± 11.6 ms**
   (6407 tracked files,
   74.7 MiB,
   at `HEAD` `c8eb46466`)

These supersede the pre-0.2.0 resharp / `regex`-crate headline figures
 (cold start 9.4 ms,
 `--all` 56.6 ms,
 measured 2026-05-16 on a smaller corpus),
 which are preserved in the dated history below.
The Linux kernel `--all` was not part of this cutover re-measure;
 its pre-0.2.0 figure of 1.989 s stands in the history.
Full methodology,
 the apples-to-apples engine A/B on the current corpus,
 and the budget comparison are in the `2026-07-17` block under "Last benched".
README defers to this file for the 0.2.0 numbers and quotes none directly.

## Cli-git integration baseline

Measured on 2026-07-10 at cli-git revision
`6eccb3064250a3c521d13f6824e4658f428c7628`.
This measurement does not replace the scanner benchmarks in this document.
It isolates the added cost of running the scanner through cli-git's trusted policy lifecycle.

The fixture used cli-git's packed production artifact and scanner `0.1.9` in a disposable Linux x64 container capped at
2 GiB RAM and 2 CPUs.
The container ran Node `24.18.0`,
Git `2.39.5`,
and overlayfs.
Its corpus copied the 11 files changed from `origin/main` to the measured revision,
437,892 bytes in total,
and scanned them with one deterministic non-matching literal rule.
The direct scanner and `git cli-git check --policy security/forbidden-strings` used the same scanner,
repository,
filesystem cache state,
and null standard-output sinks.
Pair order alternated.
Warm-up ended after two consecutive five-sample median windows differed by at most 5% for both commands.
The measured set contained 30 successful samples per command.

```text
direct scanner   median 1.993 ms   p95 2.146 ms   MAD 0.140 ms
cli-git policy   median 269.900 ms p95 275.931 ms MAD 1.923 ms
added median     267.907 ms
```

This is a baseline,
not an accepted performance budget.
Issue #356 sets and enforces budgets after every required scenario has a measured operating-system baseline.
Raw samples and exact fixture metadata are stored in
`package/git-policy/cli/perf/forbidden-strings-2026-07-10.json`.

## Cli-git repository-scale manual push

Measured on 2026-07-10 at cli-git revision
`a9096c04a88f5b7413012f34b0287fb0d2dd237c`.
The fixture cloned the complete repository history from remote revision
`ef179a737ccd39fab84e9f320e8777eeb959367b`
to the measured revision and repeatedly pushed the same complete range through both packed cli-git and direct Git.
A deterministic absent rule exercised candidate materialization and scanner traversal without findings.

The disposable Linux x64 container was capped at 2 GiB RAM and 2 CPUs.
It used Node `24.18.0`,
Git `2.39.5`,
scanner `0.1.9`,
and a 1 GiB `/tmp` tmpfs matching the host's verified temporary-storage type.
Pair order alternated after two consecutive three-pair median windows stabilized within 5% for direct and wrapped
commands.
Stability was reached after six warm-up pairs.
All 30 measured pairs completed successfully.

```text
direct Git       median 54.702 ms     p95 55.869 ms     MAD 0.645 ms
cli-git wrapper  median 1,174.731 ms  p95 1,205.668 ms  MAD 15.992 ms
added latency    median 1,119.389 ms  p95 1,152.066 ms  MAD 16.212 ms  maximum 1,205.014 ms
required ceiling                                                         strictly less than 2,000 ms
```

The first implementation created one lazy `git cat-file blob` subprocess per historical candidate and exhausted the
host process table during a real push.
Commit `f621432a6` replaced that fan-out with one `git cat-file --batch` process,
deduplicated scanner-equivalent historical states,
and bounded temporary-file materialization.
Measurements retained 64 file lanes:
eight lanes missed the ceiling on overlay storage,
while 128 lanes increased storage contention.
The accepted tmpfs run added at most 1,205.014 ms relative to paired direct Git,
leaving 794.986 ms below the 2,000 ms ceiling.

The committed harness is
`package/git-policy/cli/perf/manual-push-latency-benchmark.ts`.
Raw samples are stored in
`package/git-policy/cli/perf/manual-push-latency-2026-07-10.json`.
After building and packing both production artifacts,
run it from the repository root with:

```sh
podman run --memory=2g --cpus=2 --rm --tmpfs /tmp:rw,size=1g \
  --security-opt label=disable \
  --volume "$PWD/packages/git-policy/cli/dist/pack/monochromatic-dev-cli-git-0.0.1.tgz:/fixture/cli.tgz:ro" \
  --volume "$PWD/packages/cli/forbidden-strings/target/release/forbidden-strings:/fixture/forbidden-strings:ro" \
  --volume "$PWD:/source:ro" \
  docker.io/library/node:24-slim \
  node /source/packages/git-policy/cli/perf/manual-push-latency-benchmark.ts
```

The release profile is `lto = true`,
 `codegen-units = 1`,
 `opt-level = 3`,
`panic = "unwind"`,
 `overflow-checks = true`,
 `strip = true`.
 The unwind +
overflow-checks pair is load-bearing for the resharp-panic safety wrapper in
`src/rule/engine.rs` and `src/rule.rs`:
 under `panic = "abort"` the process
dies before `catch_unwind` runs (silent fail-open on a corrupt rule);
without overflow checks Rust's `+` silently wraps and resharp builds a wrong
DFA (also silent fail-open).
 See `Cargo.toml:49-97` for the full rationale.

The `--all` walker reads `.git/index` in process via `gix-index`;
 the
previous `git ls-files --cached --ignored --exclude-standard -z` subprocess
was removed on 2026-05-16 (recovered ~167 ms on Mono,
 ~433 ms on Linux).
Historical blocks below still describe the subprocess as the contemporary
implementation — that wording is preserved deliberately as the regression
history is the file's reason for existing.

## Last benched

**2026-07-17 (`forbidden-regex` engine cutover,
 0.2.0)**,
 hyperfine 1.20.0,
 AMD Ryzen 7 8700F (16 threads),
 quiet machine.
 Issue #387,
 the engine-swap cutover re-measure.
 Both binaries measured against the same working tree at `HEAD` `c8eb46466`
 (6407 tracked files,
 74.7 MiB tracked content):

- New binary:
 the live release `0.2.0` at
 `package/forbidden-strings/cli/target/release/forbidden-strings`,
 loading the embedded ported baseline via `--builtin-rules`
 with no user rules file (env `FORBIDDEN_STRINGS_RULES` removed).
- Old binary:
 the published `0.1.9` installed from crates.io into a disposable temp root,
 loading `data/builtin-rules.txt` via `--rules`.

Cold start scanned one small clean file
 (`package/forbidden-strings/cli/tsconfig.json`,
 100 bytes,
 no findings);
 `--all` scanned the whole tree with `--ignore-failure`
 (findings on both sides make the exit non-zero).
Each measurement is 30 runs after 3 warm-up runs.

```text
                    old 0.1.9 (resharp)   new 0.2.0 (forbidden-regex)
cold start          12.1 ms ± 0.5 ms      13.7 ms ± 0.4 ms
full --all          168.8 ms ± 3.3 ms     131.9 ms ± 11.6 ms
```

Reading:

- Cold start rises about 1.6 ms new versus old,
 near the combined sigma.
 The new binary loads a precompiled serialized `RegexSet` (`from_bytes`)
 instead of compiling 259 rules at startup,
 so its startup CPU is far lower
 (user 8.2 ms new versus 65.5 ms old)
 even though wall time is close.
- Full `--all` is about 0.78x on the new engine
 (131.9 ms versus 168.8 ms on the identical corpus),
 so the engine is faster on the full scan,
   not slower.
- The rise from the pre-0.2.0 headline `--all` of 56.6 ms is corpus growth,
 not the engine:
 tracked files grew from 3471 to 6407 and content from 57 MiB to 74.7 MiB
 since that measurement,
   and the old engine on this larger corpus is 168.8 ms.
 The `~5000 tracked files` re-bench trigger in "When to re-bench" is now met,
 which is itself why the absolute number moved.

Budget comparison:

- Pre-commit budget.
 The README states a sub-100 ms pre-commit budget for the native-binary startup;
 the "When to re-bench" trigger sets a 30 ms startup ceiling.
 New cold start 13.7 ms is within both,
   with margin.
- Pre-push and full-scan budget.
 The README states no separate numeric pre-push figure;
 the operative ceilings are the "When to re-bench" 180 ms `--all` trigger
 and the cli-git manual-push ceiling of strictly less than 2000 ms
 (see "Cli-git repository-scale manual push").
 New full `--all` 131.9 ms is within both,
   with wide margin.
- No budget is exceeded;
 there is no blocking finding.

---

**2026-06-19 (abandoned resharp-only serialize-cache experiment)**,
hyperfine 1.20.0.
 Main binary:
`/var/home/user/Monochromatic/package/forbidden-strings/cli/target/release/forbidden-strings`.
Experiment binary:
`/tmp/agent/forbidden-strings-resharp-cache-20260619/packages/cli/forbidden-strings/target/release/forbidden-strings`.
Both scanned `/var/home/user/Monochromatic` with
`forbidden-strings.local.example.txt`.
 The `--all` runs used
`--ignore-failure` because rule 404 fired on
`doc/troubleshooting/mise-env-redacted-values.patch` at that revision.

Experiment summary:
 migrating every regex rule to resharp and using resharp's
`serialize` feature as an on-disk ruleset cache was abandoned.
 Warm cache loads
were still much slower than main's hybrid engine,
 and cold cache writes were
seconds slower.
 Do not revive this exact design unless the cache format changes
substantially,
 for example per-entry files,
 a size cap,
 mmap,
 or a smaller
upstream dump representation.

```text
startup, main                         10.3 ms ± 0.9 ms   30 runs
startup, resharp cache warm          244.4 ms ± 2.8 ms   30 runs
startup, resharp cache cold            4.329 s ± 0.128 s  5 runs
--all, main                           78.8 ms ± 4.0 ms   20 runs
--all, resharp cache warm            358.4 ms ± 4.6 ms   20 runs
--all, resharp cache cold              4.254 s ± 0.064 s  3 runs
```

The warm experiment cache file for the example ruleset was 255,270,294 bytes
(244 MiB from `du --summarize`).
 The warm serialize cache cut cold resharp
startup by about 17.7x,
 but main still beat the warm-cache experiment by
23.8x on startup and 4.55x on `--all`.

---

**2026-05-16 (post-emit-hit-consolidation,
 A/B vs immediate predecessor)**,
hyperfine 1.20.0,
 same hardware.
 Apples-to-apples A/B between two binaries
built from the same commit modulo the `scan.rs` consolidation:
 the
"baseline" is HEAD pre-refactor (the post-perf-fix block below);
 the
"refactor" extracts a private `emit_hit(li, path, start, end, rule_idx)`
helper in `scan_format.rs` and replaces the four near-identical
line/col-compute + push sequences in `scan.rs::scan_content` (AC literal
phase,
 prefix-matched par_iter,
 residual Single shard,
 residual Combined
par_iter) with one call each.
 The helper is `#[inline]`;
 with
`lto = true` and `codegen-units = 1`,
 LTO already inlines tiny crate-
internal helpers,
 so the attribute is belt-and-suspenders.

```text
                   baseline           refactor           delta
Mono startup       9.8 ms ± 0.8 ms    9.4 ms ± 0.8 ms    within sigma
Mono --all         60.3 ms ± 4.3 ms   56.6 ms ± 3.1 ms   within sigma
Linux --all        2.024 s ± 0.109 s  1.989 s ± 0.246 s  within sigma
```

Mono runs:
 30 per binary.
 Linux runs:
 15 per binary.
 The `--all` benches
used `--ignore-failure` to absorb the one rule that fires on existing
test-fixture content (AWS access key prefix in `algebra_tests.rs:119`)
on Mono and three pre-existing kernel-fixture rule hits on Linux.

Notes from the bench:

- A sporadic large-outlier event (single ~750-950 ms wall on Mono `--all`)
  hit whichever binary ran second in the back-to-back hyperfine session,
  regardless of which one.
   Order-reversed reruns reproduced the outlier
  on the other binary,
   confirming it is system noise unrelated to the
  refactor (hyperfine itself flagged it:
   "Statistical outliers were
  detected.
   Consider re-running this benchmark on a quiet system").
  Numbers reported above are from clean runs.
- All three deltas trend slightly faster post-refactor but every delta is
  smaller than the combined sigma;
   treat the refactor as null-cost.

---

**2026-05-16 (post-perf-fix:
 gix-index + binary-tail-cap)**,
 hyperfine
1.20.0,
 same hardware (AMD Ryzen 7 8700F,
 16 threads).
 Binary:
`package/forbidden-strings/cli/target/release/forbidden-strings` built
from this package's `src/` after two perf changes landed on top of the
soundness audit:

- `walk.rs` now reads `.git/index` in-process via `gix-index` instead of
  forking `git ls-files --cached --ignored --exclude-standard -z`.
   The
  subprocess cost (88.7 ± 0.8 ms standalone on Mono,
   350.7 ± 3.6 ms on
  Linux per invocation) is replaced with a single-digit-millisecond
  index read.
- `main.rs::read_with_binary_check` caps per-file scan at 8 KiB for
  files larger than that whose first 8 KiB contains a NUL byte.
   The
  first 8 KiB is always scanned,
   so secrets in the leading window still
  fire (closes BUG 5's soundness intent);
   the tail past 8 KiB is
  skipped for binary files (recovers the ~65 ms binary-scan cost on
  Mono).

### 2026-05-16 (post-perf-fix) realistic ruleset on Monochromatic, 30 runs

Corpus:
 Monochromatic git-tracked content,
 3,471 files,
 57 MiB.
 Same
example ruleset (`forbidden-strings.local.example.txt`,
 259 regex
rules).
 One rule fires on existing test-fixture content (AWS access key
prefix in `algebra_tests.rs:119`);
 bench used `--ignore-failure`.

```text
example-startup    8.8 ms ±  0.5 ms    (user 44.1 ms,  sys 13.0 ms)
example-all       58.6 ms ±  3.4 ms    (user 371.0 ms, sys 139.1 ms)
                                       6.3x parallelism, ~973 MiB/s wall
```

### 2026-05-16 (post-perf-fix) Linux kernel corpus, 15 runs

Same shallow clone of `torvalds/linux` to `/tmp/claude/linux`,
93,696 files,
 2.0 GiB.
 Same binary,
 same example ruleset.

```text
linux-all         1.901 s ± 0.190 s    (user 23.310 s, sys 2.096 s)
                                       12.2x parallelism, ~1.05 GiB/s wall
```

### Comparison with 2026-05-16 (post-soundness-audit, pre-perf-fix)

```text
                       pre-perf-fix       post-perf-fix    delta
Monochromatic startup  9.3 ms ± 0.7 ms    8.8 ± 0.5 ms     within sigma
Monochromatic --all    225.7 ± 11.5 ms    58.6 ± 3.4 ms    -167 ms (3.85x)
Linux --all            2.334 ± 0.112 s    1.901 ± 0.190 s  -433 ms (1.23x)
```

Mono `--all` now sits 7 ms above the pre-audit baseline (51.3 ± 3.6 ms
on 2026-05-15) -- the residual gap is the binary-scan cost on files
≤ 8 KiB (BUG 5 soundness is preserved for that range).
 The audit's
ten remaining soundness fixes ride for free in wall-time terms.

---

**2026-05-16 (post-soundness-audit + post-\s-byte-alt-expansion)**,
 hyperfine
1.20.0,
 same hardware (AMD Ryzen 7 8700F,
 16 threads).
 Binary:
`package/forbidden-strings/cli/target/release/forbidden-strings` built from
this package's `src/` after the 12-commit soundness audit (commits 468fcf73
to 4289cdb3) closed BUGs 1 through 11 from `/tmp/fs-bug-brief.md`.
 The
audit produced one perf-relevant change:
 BUG 8's fix expands `\s` in the
rule source to a non-capturing alternation `(?:\s|<Unicode-WS bytes>)` so
NBSP (U+00A0),
 em-spaces,
 ideographic space,
 and the rest of the Unicode
whitespace set match `\s` under the `unicode(false)` fast-compile path.

### 2026-05-16 realistic ruleset on Monochromatic, 30 runs

Corpus:
 Monochromatic git-tracked content,
 3,471 files,
 57 MiB.
 Same example
ruleset (`forbidden-strings.local.example.txt`,
 259 regex rules,
 853 total
lines).
 One rule fired on existing test fixture content
(`./package/forbidden-strings/cli/src/rule/algebra_tests.rs:119:32..43 rule=849`,
an AWS access key prefix in an assertion);
 bench used `--ignore-failure` to
absorb the resulting non-zero exit.

```text
example-startup    9.3 ms ±  0.7 ms    (user 44.8 ms,  sys 13.4 ms)
example-all      225.7 ms ± 11.5 ms    (user 774.0 ms, sys 190.5 ms)
                                       3.4x parallelism, ~253 MiB/s wall
```

Phase-timing breakdown (`FORBIDDEN_STRINGS_DEBUG_TIMING=1`,
 3 runs):

```text
phase 0 read_rules_file:           0.0 to 0.1 ms
phase 1 classify+regex_compile:    4.3 to 5.2 ms
phase 2 extract_gating_substrings: 0.2 ms
phase 3 ac_build:                  0.4 ms
phase 4 residual_shards:           0.0 ms
```

Bucket breakdown (`FORBIDDEN_STRINGS_DEBUG_BUCKETS=1`):

```text
ac_cs_lit=0
ac_cs_regex_prefix=157
ac_ci_regex_prefix=171
residual=4 (in 4 single + 0 combined shards)
regex_rules_total=259
```

### 2026-05-16 Linux kernel corpus, 5 runs each

Fresh shallow clone of `torvalds/linux` to `/tmp/claude/linux`,
 commit
`6916d570`,
 93,696 git-tracked files,
 2.0 GiB.
 Same binary,
 same example
ruleset.

```text
linux-startup    9.8 ms ± 0.4 ms      (user 46.9 ms,   sys 11.7 ms)
linux-all        2.334 s ± 0.112 s    (user 25.151 s,  sys 2.280 s)
                                      10.8x parallelism, ~876 MiB/s wall
```

Three rule hits were present in the kernel test fixtures (each
matches its own example ruleset entry,
 not regressions from the
audit):

```text
./drivers/of/unittest-data/tests-phandle.dtsi:9:3..33 rule=404
./tools/testing/selftests/sgx/sign_key.pem:1:1..31 rule=621
./tools/testing/kunit/configs/all_tests.config:11:23..32 rule=849
```

### Comparison with 2026-05-15

```text
                       2026-05-15           2026-05-16           delta
Monochromatic startup  9.4 ms ± 0.8 ms      9.3 ms ± 0.7 ms      within sigma
Monochromatic --all    50.4 ms ± 2.6 ms     225.7 ms ± 11.5 ms   +175.3 ms (4.5x)
Linux startup          9.1 ms ± 0.9 ms      9.8 ms ± 0.4 ms      within sigma
Linux --all            1.836 s ± 0.051 s    2.334 s ± 0.112 s    +498 ms (+27%)
```

Both `--all` benches regressed.
 The cause is **not** BUG 8's
source-level expansion of `\s` to `(?:\s|<Unicode-WS-bytes>)`,
despite the temporal correlation;
 an earlier draft of this section
attributed the regression there in error.
 The expansion is within
sigma of zero on Mono,
 confirmed by empirical A/B bisect (30 runs
each,
 the shipped binary vs the same binary with
`expand_unicode_whitespace` patched to return its input verbatim):

```text
                                              Mono --all wall (30 runs)
HEAD (4289cdb3 = expansion shipped)           227.5 ± 10.1 ms
HEAD with expand_unicode_whitespace no-op'd   222.5 ± 12.5 ms
                                              1.02x ± 0.07 (within sigma)
```

Per-commit bisect on Mono `--all` (hyperfine 30 runs,
 same hardware)
attributes the regression to earlier audit commits:

```text
                                                  Mono --all      Δ vs pre-audit
pre-audit (50b96ce0)                              51.3 ± 3.6 ms   baseline
pre-7377c6f6 (after BUGs 1, 2, 4)                 53.8 ± 4.8 ms   within sigma
7377c6f6 walker union (BUG 3)                     138.0 ± 3.8 ms  +84 ms
559123c3 remove is_likely_binary (BUG 5)          202.9 ± 11.9 ms +65 ms more
HEAD (4289cdb3 = expansion shipped)               220.8 ± 9.3 ms  +18 ms more
```

Decomposition of the three audit-era costs:

- **+84 ms (BUG 3,
   commit 7377c6f6)**:
   the `--all` walker now unions
  with `git ls-files --cached --ignored --exclude-standard -z` to
  recover force-added gitignored files.
   Standalone hyperfine on
  the Mono repo:

  ```text
  git -C . ls-files --cached --ignored --exclude-standard -z
                                            88.7 ± 0.8 ms (10 runs)
  ```

  The walker pass itself is ~10 ms on Mono;
   the cost is the
  subprocess fork+exec+index-walk inside git,
   not the walker.
  Standalone Linux kernel measurement:
   350.7 ± 3.6 ms (accounts
  for most of the +498 ms Linux regression below).

- **+65 ms (BUG 5,
   commit 559123c3)**:
   removing `is_likely_binary`
  short-circuit means binary content (50 files / 31.8 MiB on Mono)
  now flows through AC + per-prefix `find_all`.
   AC scan over the
  binary volume is ~6 ms;
   the rest is regex `find_all` triggered
  by coincidental short-prefix hits in random bytes.

- **+18 ms residual**:
   extractor changes for `(?u)` / `(?x:)` /
  bare `_`,
   engine-error surface,
   plus measurement noise from
  the expansion (the expansion itself is within sigma of zero).

Corpus-magnitude difference is therefore about binary-scan ratio
(BUG 5) and subprocess wall under file-count load (BUG 3),
 not
NFA growth.
 Linux's larger `--all` regression (+498 ms vs Mono's
+175 ms) tracks its larger subprocess time (350 ms standalone
vs Mono's 89 ms) and its much larger binary content surface.

The original BUG 8 fix (commit 0479371a) forced rules containing
`\s/\S/\w/\W/\d/\D/\b/\B` shorthand onto `unicode(true)` compile;
that was ~95x in phase 1 and 64x in startup wall (575 ms on Mono),
which exceeds the re-bench threshold for "20 ms startup,
 150 ms
--all".
 The source-level expansion that shipped on 2026-05-16 (commit
4289cdb3) restores startup to within-sigma of baseline and keeps
`--all` under 300 ms on Mono with full Unicode whitespace coverage.

### Option A (rayon::join walker + subprocess): investigated, rejected

Parallelising the walker pass and `git ls-files` subprocess via
`rayon::join` was investigated under the hypothesis that the ~88 ms
subprocess could hide behind walker time.
 Instrumentation showed:

```text
walker_arm:        10 ms
git_arm:           92 ms
rayon::join total: 94 ms      (max(walker, git) + ~2 ms join overhead)
```

`rayon::join` did parallelise (total ≈ max not sum),
 but the walker
is too fast (~10 ms) to fill the subprocess runtime span.
 The ceiling for
savings is ~10 ms,
 within measurement sigma.
 Apples-to-apples bench:

```text
                       Mono --all (30 runs)
HEAD                   226.3 ± 8.1 ms
HEAD + rayon::join     223.8 ± 9.0 ms
                       1.01x ± 0.05 (within sigma)
```

The change added a `move ||` wrapper,
 a paired closure,
 and a
`rayon::join` call to `walk.rs::list_files` for no measurable win.
Reverted;
 not shipped.

The actual lever to remove BUG 3's +84 ms is replacing the
subprocess with an in-process git-index reader.
 Shipped as a
follow-up commit using `gix-index` (`default-features = false` +
`sha1` only) -- see the "post-perf-fix" block at the top for the
new numbers;
 the subprocess is gone.

### Soundness tradeoff: shorthand atom semantics

The `regex` crate's `unicode(false)` mode interprets `\s`,
 `\S`,
`\w`,
 `\W`,
 `\d`,
 `\D`,
 `\b`,
 `\B` as ASCII-only.
 Closing BUG 8
("`\s` should match NBSP") without paying the unicode-on compile
cost required choosing per-atom semantics:

- **`\s`:
   Unicode-aware via source rewrite.
  ** Each `\s` in the rule
  source becomes `(?:\s|<UNICODE_WS_ALT>)` (free) or
  `(?:[...\s...]|<UNICODE_WS_ALT>)` (inside a character class).
   The
  alternation covers every Unicode whitespace code point's UTF-8
  bytes (U+00A0,
   U+1680,
   U+180E,
   U+2000..U+200A,
   U+2028..U+2029,
  U+202F,
   U+205F,
   U+3000,
   U+FEFF).
   Sound for the NBSP repro and the
  rest of the Unicode whitespace set.
   Wall cost:
   within measurement
  sigma per the A/B bisect above (1.02x ± 0.07 vs no-op'd expansion).
- **`\S`,
   `\w`,
   `\W`,
   `\d`,
   `\D`,
   `\b`,
   `\B`:
   byte-level under
  `unicode(false)`.
  ** Behaviour matches PCRE's default (ASCII-only).
  Practical scope on the betterleaks corpus:
  - `\S` (2 rules):
     used as `[\S]{N}` capture or `[\s\S]` "any
    byte" idiom.
     Neither is a silent miss under byte mode;
     the
    second is correct ("any byte" is what `[\s\S]` always meant
    under byte semantics).
  - `\w` (133 rules):
     always in non-required positions like
    `[\w.-]{0,50}` or inside captures.
     The `{0,N}` quantifier
    means a Unicode-character preface still matches via the
    "zero" lower bound;
     no silent miss in real-world rules.
  - `\d` (12 rules):
     used for numeric IDs like `\d{15,16}`
    (credit cards) where the author wants ASCII digits,
     not
    Bengali numerals.
     Byte mode matches author intent.
  - `\b` (93 rules):
     boundary anchors against literal prefixes
    like `\bA3-`,
     `\b(p8e-`.
     Under byte mode,
     position between a
    non-ASCII byte and an ASCII word byte is still a boundary
    (because non-ASCII bytes are non-word in ASCII mode),
     so the
    rule fires when it should.
  - `\W`,
     `\D`,
     `\B`:
     zero uses in the example ruleset.

For a future rule that genuinely requires Unicode-aware `\w/\d/\b`,
the author can opt in via the `(?u)` flag,
 which the scanner already
rejects from the AC fast path and routes to residual with
`unicode(true)` compile (see BUG 2's leading-flag check).
 The
expansion at this level is byte-mode-only.

---

**2026-05-15 (post-algebra-complement-gate-fix)**,
 hyperfine 1.20.0,
 same
hardware (AMD Ryzen 7 8700F,
 16 threads).
 Binary:
`package/forbidden-strings/cli/target/release/forbidden-strings` built from
this package's `src/` after `src/rule/atom.rs` started treating `~(...)`
complement operands as non-contributing gates and `src/rule/walker.rs`
started treating `&` as a transparent intersection separator.

### 2026-05-15 realistic ruleset on Monochromatic, 30 runs

Corpus:
 Monochromatic git-tracked content,
 3,454 files,
 47.2 MiB.
 Same example
ruleset (`forbidden-strings.local.example.txt`,
 259 regex rules,
 853 total
lines).
 The scan produced no rule hits.

```text
example-startup    9.4 ms ± 0.8 ms     (user 37.9 ms,  sys 12.7 ms)
example-all       50.4 ms ± 2.6 ms     (user 331.4 ms, sys 95.4 ms)
                                      8.5x parallelism, ~936 MiB/s wall
```

Phase-timing breakdown (`FORBIDDEN_STRINGS_DEBUG_TIMING=1`,
 3 runs):

```text
phase 0 read_rules_file:           0.0 to 0.1 ms
phase 1 classify+regex_compile:    4.0 to 4.9 ms
phase 2 extract_gating_substrings: 0.2 ms
phase 3 ac_build:                  0.4 to 0.5 ms
phase 4 residual_shards:           0.0 ms
```

Bucket breakdown (`FORBIDDEN_STRINGS_DEBUG_BUCKETS=1`):

```text
ac_cs_lit=0
ac_cs_regex_prefix=157
ac_ci_regex_prefix=171
residual=4 (in 4 single + 0 combined shards)
regex_rules_total=259
```

### 2026-05-15 Linux kernel corpus, 5 runs each

Fresh shallow clone of `torvalds/linux` to `/tmp/claude/linux`,
 commit
`70eda6866` (92,549 git-tracked files,
 1.47 GiB).
 Same binary,
 same example
ruleset.

```text
linux-startup    9.1 ms ± 0.9 ms      (user 33.6 ms,   sys 16.6 ms)
linux-all        1.836 s ± 0.051 s    (user 22.975 s,  sys 1.284 s)
                                      13.2x parallelism, ~822 MiB/s wall
```

Phase-timing breakdown (`FORBIDDEN_STRINGS_DEBUG_TIMING=1`,
 2 runs):

```text
phase 0 read_rules_file:           0.1 ms
phase 1 classify+regex_compile:    4.0 to 4.9 ms
phase 2 extract_gating_substrings: 0.2 ms
phase 3 ac_build:                  0.4 ms
phase 4 residual_shards:           0.0 ms
```

Two rule hits were present in the kernel test fixtures:

```text
./drivers/of/unittest-data/tests-phandle.dtsi:9:3..33 rule=404
./tools/testing/selftests/sgx/sign_key.pem:1:1..31 rule=621
```

These are corpus fixtures matching the example ruleset,
 not regressions from
the algebra extractor fix.

### Comparison with 2026-05-10

```text
Monochromatic startup   9.0 ms ± 0.7 ms     9.4 ms ± 0.8 ms     within sigma
Monochromatic --all    47.3 ms ± 2.9 ms    50.4 ms ± 2.6 ms     +3.1 ms
Linux startup           8.9 ms ± 0.7 ms     9.1 ms ± 0.9 ms     within sigma
Linux --all             2.250 s ± 0.253 s   1.836 s ± 0.051 s   -414 ms
```

The algebra extractor change runs in Phase 2,
 which remains 0.2 ms on both
corpora.
 The current Linux shallow clone has 92,549 files and 1.47 GiB,
 versus
the 2026-05-10 clone's 93,697 files and 1.48 GiB;
 this bench did not isolate
which corpus or cache differences account for the lower `linux-all` wall time.
Monochromatic `--all` remains near the previous wall time despite tracked content
growing to 47.2 MiB.

---

**2026-05-10 (post-utf8-walker-fix)**,
 hyperfine 1.20.0,
 same hardware
(AMD Ryzen 7 8700F,
 16 threads).
 The walker fix in `src/rule/atom.rs`
rewrites `walk_literal_bytes` to iterate by `char` rather than casting
each `u8` to `char` (the former silently mojibake'd non-ASCII multi-byte
UTF-8 into wrong codepoints).
 The fix's code path runs in
`extract_gating_substrings` (Phase 2,
 the rule-load path);
 the per-file
scan hot path is unchanged.

### Realistic ruleset on Monochromatic, 30 runs

```text
example-startup    9.0 ms ± 0.7 ms     (user 34.5 ms, sys 13.8 ms)
example-all       47.3 ms ± 2.9 ms     (user 277.1 ms, sys 93.7 ms)
```

Phase-timing breakdown (`FORBIDDEN_STRINGS_DEBUG_TIMING=1`,
 3 runs):

```text
phase 0 read_rules_file:           0.0 to 0.1 ms
phase 1 classify+regex_compile:    3.7 to 6.1 ms
phase 2 extract_gating_substrings: 0.2 to 0.3 ms   (unchanged from 2026-05-03)
phase 3 ac_build:                  0.5 to 0.6 ms
phase 4 residual_shards:           0.0 ms
```

### Comparison with 2026-05-03 baseline

Both rows are Monochromatic `--all` against the same example ruleset
(`forbidden-strings.local.example.txt`):

```text
            2026-05-03         2026-05-10         delta
example-all 37.2 ms ± 3.6 ms   47.3 ms ± 2.9 ms   +10.1 ms (+27%)
```

The shift is **unrelated to the walker fix**:

- Phase 2 (`extract_gating_substrings`,
   the only phase the walker fix
  runs in) is identical at 0.2 to 0.3 ms across both benches.
- Repo content changed between benches:
   tracked file count grew from
  ~2700 to 2860 (+5%);
   tracked byte volume shifted from ~21 MiB to
  19.8 MiB.
   New content under `package/pi-plugin/auto-mode/src/` triggers
  two true rule violations (rules 104 and 319 in
  `signals.unit.test.ts`) and apparently more AC prefix-match queueing
  that drives extra per-file `find_all` invocations.
- Math sanity:
   even if the walker fix had doubled Phase 2,
   the absolute
  delta would be ~0.3 ms,
   not 10 ms. The +10 ms must come from
  per-file work the walker doesn't touch.

The two reported hits should be triaged separately (they are
pre-existing rule violations against pre-existing content,
 just newly
visible in this bench's output).

### Linux kernel corpus, 5 runs each

Fresh shallow clone of `torvalds/linux` to `/tmp/claude/linux`
(93,697 git-tracked files,
 1.48 GiB).
 Same binary,
 same example
ruleset.

```text
linux-startup    8.9 ms ± 0.7 ms      (user 38.5 ms,   sys 10.1 ms)
linux-all        2.250 s ± 0.253 s    (user 24.826 s,  sys 1.569 s)
                                      11x parallelism, ~660 MiB/s wall
```

Phase-timing breakdown (`FORBIDDEN_STRINGS_DEBUG_TIMING=1`,
 2 runs):

```text
phase 0 read_rules_file:           0.0 to 0.1 ms
phase 1 classify+regex_compile:    5.0 to 5.9 ms
phase 2 extract_gating_substrings: 0.3 to 0.4 ms   (unchanged from 2026-05-03)
phase 3 ac_build:                  0.6 to 0.7 ms
phase 4 residual_shards:           0.0 ms
```

Comparison with 2026-05-03 baseline (same corpus,
 same ruleset):

```text
                  2026-05-03         2026-05-10         delta
linux-startup     8.5 ms ± 0.8 ms    8.9 ms ± 0.7 ms    +0.4 ms (within sigma)
linux-all         2.153 s ± 0.367 s  2.250 s ± 0.253 s  +97 ms (within sigma)
```

Both differences sit within overlapping standard deviations.
 The
walker fix has no measurable impact on the Linux corpus,
 which is
expected:
 the per-file scan path the Linux benchmark stresses is
independent of `walk_literal_bytes` (called only in Phase 2).

Two pre-existing rule violations visible in the Linux corpus:

```text
./tools/testing/selftests/sgx/sign_key.pem:1:1..31  rule=619
./drivers/of/unittest-data/tests-phandle.dtsi:9:3..33 rule=402
```

These are kernel test fixtures matching the example ruleset;
 not
introduced or affected by the walker fix.

Corpus drift vs 2026-05-03 baseline:

```text
                2026-05-03      2026-05-10      delta
file count      93,694          93,697          +3
byte volume     1.58 GiB        1.48 GiB        -100 MiB (~6%)
```

Slight shrinkage offsets the file-count growth;
 net per-byte
throughput is essentially unchanged.

### Audit blast radius of the walker fix

`rg -nP '[\xe2-\xf4]' forbidden-strings.local.txt
forbidden-strings.append.local.txt forbidden-strings.local.example.txt
data/betterleaks-default-config.toml` returned no non-ASCII leading
literals in any active regex rule.
 The fix is preventive:
 future rules
containing em-dashes,
 smart quotes,
 ellipsis,
 or emoji as a leading
literal will now gate correctly through Aho-Corasick instead of
silently dropping into the residual gate (or worse,
 registering a
mojibake pattern that AC never matches).

Test coverage:
 22 tests in `src/rule/atom_tests.rs` and
`src/rule/extract_tests.rs` cover 1 / 2 / 3 / 4-byte UTF-8 widths,
the escape branch,
 the alternation branch,
 anchor strip,
 the
`MIN_PREFIX_LEN` byte-length semantic,
 and end-to-end
`extract -> AhoCorasick -> match` round-trips for 2-byte,
 3-byte,
 and
4-byte leading characters.

---

**2026-05-03 (post-unicode-off + post-greedy-combine + post-source-split)**,
with `hyperfine 1.20.0`.
 Binary:
 `target/release/forbidden-strings` built
with `mise run //package/forbidden-strings/cli:build`.

The "realistic" ruleset is the betterleaks-port baseline:
 259 regex
rules + 3 literals (851 total lines).

The current numbers reflect the hybrid engine (`CompiledRegex::{Resharp,Plain}`
in `src/rule/engine.rs`:
 257 of 259 rules compile via the standard `regex`
crate,
 ~100x faster than resharp;
 2 rules use resharp's set-algebra
operators),
 the regex-crate `unicode(false)` mode with try-and-fallback
to `unicode(true)` for rules that need unicode-property classes (a 17x
wall-time win on its own),
 the file-split refactor (every source file
under 500 lines),
 and greedy combine-partition for residual shards (see
Architecture summary).

### Realistic ruleset (259 rules, betterleaks port) on Monochromatic, 15 runs

```text
--all              37.2 ms ± 3.6 ms    (user 239.4 ms, sys 71.8 ms)
```

Range 30.9 ms .. 43.5 ms. Effective parallelism = ~6.4x cores.

Phase-timing breakdown (`FORBIDDEN_STRINGS_DEBUG_TIMING=1`):

```text
phase 0 read_rules_file:           0.0 ms
phase 1 classify+regex_compile:    ~5 ms    (unicode-off plain + 2 resharp)
phase 2 extract_gating_substrings: ~0.3 ms
phase 3 ac_build:                  ~0.4 ms
phase 4 residual_shards:           ~0.0 ms  (4 residuals, below greedy threshold)
```

Bucket distribution:

```text
ac_cs_lit=0
ac_cs_regex_prefix=157           (case-sensitive AC; substring-extracted from regex rules)
ac_ci_regex_prefix=171           (case-insensitive AC; substring-extracted from regex rules)
residual=4 (in 4 single shards)  (L114 beamer, L251 facebook, L769 twitch, L796 vercel-ai)
regex_rules_total=259
```

### Linux kernel corpus (93,694 files, 1799.2 MiB tracked bytes)

Startup-only,
 30 runs each:

```text
example ruleset (851 rules)        8.5 ms ± 0.8 ms    (user 35.8 ms, sys 11.9 ms)
synth-1k ruleset (1000 rules)      9.0 ms ± 0.6 ms    (user 18.0 ms, sys 32.9 ms)
```

`--all` (full corpus walk + scan),
 5 runs each:

```text
example ruleset    (851 rules; 4 residuals)   2153 ms ± 367 ms   (user 22.5 s, sys 0.9 s, 10.5x parallelism, ~836 MiB/s wall)
runtime ruleset    (860 rules; 4 residuals)   2012 ms ± 336 ms   (user 22.7 s, sys 0.9 s, 11.3x parallelism, ~894 MiB/s wall)
synth-residual-20  (20 hard residuals)        640  ms ±  55 ms   (user  5.9 s, sys 0.8 s,  9.2x parallelism, ~2.8 GiB/s wall)
synth-1000-0pct    (1000 rules; 0 residuals)  139  ms ±   9 ms   (user 0.55 s, sys 0.8 s,  4.0x parallelism, ~12.9 GiB/s wall)
```

Observations:

- The production-shape ~2 s headline is bounded by **4 betterleaks-shape
  residuals** (L114 beamer,
   L251 facebook,
   L769 twitch,
   L796 vercel-ai)
  each running per-rule `find_all` across 1.8 GiB.
   Linux kernel source
  triggers many false-positive prefix hits on these rules
  (`SK`,
   `Q~`,
   `\d{15,16}`,
   `hvs.`),
   so the regex crate's literal-prefix
  fast path runs on the prefixes but the full regex is then evaluated
  on every prefix hit.
   With 4 rules × 1.8 GiB workload distributed
  over 10.5x cores,
   the floor is the regex engine's per-byte throughput
  on the captured prefix-hit windows.
- The synth-residual-20 case is **faster than the 4-residual production
  case** despite having 5x more residual rules.
   The synthetic residuals
  use `_RESID_<tag>_` substrings that fire rarely on real source,
   so
  the residual-bucket combined gate triggers per-rule scans only on a
  handful of files.
   The production residuals have shorter,
   more common
  prefixes that fire on hundreds of files.
- The 0-residual case (synth-1000-0pct) shows the no-residual ceiling:
  12.9 GiB/s end-to-end on the Linux corpus,
   single-AC-pass dominated.
- Per-byte slowdown vs Mono:
   Mono `--all` is 37 ms over 21.4 MiB
  (~580 MiB/s),
   Linux `--all` is ~2 s over 1799 MiB (~900 MiB/s).
  Linux is faster per-byte because fixed setup amortizes over more
  files and the AC + regex hot path benefits from longer contiguous
  scans per file.

## Ceiling analysis (2026-05-03, post-Linux-bench)

After the Linux corpus bench (above),
 the question "is more optimisation
worth pursuing?
" was investigated.
 The conclusion:
 **production-shape
rulesets are at the practical floor on Linux scale**,
 bounded by rule-
grammar realities (false-positive rate of literal anchors against kernel
source),
 memory bandwidth (AC fast path),
 and the regex engine's
per-byte throughput on betterleaks-shape patterns.
 Mono (the actual
CI workload) is at 37 ms,
 ~14x under any reasonable budget.

### The 4 production residuals: source-pattern analysis

`forbidden-strings.local.example.txt` lines 114,
 251,
 769,
 796:

```text
L114  /(?:[\\'"`\s>=:(,)])([a-zA-Z0-9_~.]{3}\dQ\~[a-zA-Z0-9_~.-]{31,34})(?:$|[\\'"`\s<),])/
L251  /(?i)\b(\d{15,16}(\||%)[0-9a-z\-_]{27,40})(?:\\?['"`]|[\s;]|\\[nr]|$)/
L769  /SK[0-9a-fA-F]{32}/
L796  /\b((?:hvs\.[\w-]{90,120}|s\.(?i:[a-z0-9]{24})))(?:\\?['"`]|[\s;]|\\[nr]|$)/
```

Per-rule promotion analysis:

- **L114**:
   single required substring `Q~` (2 chars,
   below
  `MIN_PREFIX_LEN = 4`),
   no top-level alternation.
   The regex crate's
  literal-prefix optimisation already memchr-scans for `Q~` internally.
  Promoting to AC moves the same filter earlier in the pipeline;
   the
  expensive part (full regex verification on each candidate window) is
  unchanged.
   Not promotable without lowering MIN_PREFIX_LEN,
   which
  would create AC false-positive storms on `Q~` in arbitrary source.
- **L251**:
   no usable literal substring at all.
   The required content
  is `\d{15,16}` (numeric only) followed by `|` or `%`.
   Not promotable.
- **L769**:
   single required substring `SK` (2 chars).
   Same conclusion
  as L114.
- **L796**:
   top-level alternation between `hvs\.[\w-]{90,120}` and
  `s\.(?i:[a-z0-9]{24})`.
   Branch 1 has the 4-char anchor `hvs.`;
  Branch 2 has only `s.` (2 chars before flag scope).
   Multi-substring
  AC gating could route Branch 1 to AC,
   but the regex crate is already
  internally memchr-optimising on `hvs.`:
   the gain is moving the
  filter from regex-internal to our AC bucket,
   not changing the work.
  Branch 2 stays on the regex engine regardless.

The actual bottleneck on Linux:
 **the production residuals' literal
anchors are too common in Linux kernel source.
** Hundreds of
`SK`/`hvs.`/`Q~`/`\d{15,16}` occurrences in test fixtures,
 hex
constants,
 sample payloads.
 Each occurrence triggers a full regex
verification on the surrounding byte span.
 That cost is bounded by the
regex engine's per-byte throughput,
 not by anything optimiser-side.

### Rayon batching tested and rejected (2026-05-03)

Hypothesis:
 `synth-1000-0pct --all` on Linux gets 4.0x parallelism on
16 cores (139 ms wall,
 553 ms user) because per-file AC work is
~1 us at 19 KB/file and rayon's task-creation overhead is comparable.
`with_min_len(N)` would amortise that overhead by batching N files
per task.

Tested by adding `.with_min_len(64)` to the outer `par_iter` in
`main.rs` (single-line change,
 rebuilt,
 measured,
 reverted).

Apples-to-apples results,
 hyperfine 5 runs per Linux config,
 15 runs
for Mono:

```text
                              baseline           with_min_len(64)
Linux synth-1000-0pct --all   139 ms ± 9         140 ms ± 8           within sigma
Linux example --all           2153 ms ± 367      2204 ms ± 111        within sigma
Mono example --all            43 ms ± 3          44 ms ± 4            within sigma
```

No measurable change on any workload.
 Reverted;
 not shipped.

The hypothesis was wrong because the synth-1000-0pct's 4.0x
parallelism is **memory-bandwidth bound,
 not scheduler-overhead
bound**.
 Aggregate AC throughput at 12.9 GiB/s is in striking
distance of practical commodity-machine memory bandwidth (DDR5
sequential-read ~10-20 GB/s when L3-cold).
 With 16 threads each
streaming bytes through SIMD AC,
 the memory bus saturates well
before scheduler overhead becomes the bottleneck.
 Batching frees
scheduler capacity,
 but there's no scheduler-capacity bottleneck
to relieve.

### Combine-residuals-on-Linux: not tested, math-rejected

Combining the 4 residual rules into one Combined-shard gate was
tested on Mono earlier (handover round 1) and regressed +86 ms (650
ms -> 736 ms).
 The mechanism:
 each individual rule has a strong
literal-prefix anchor that the regex crate accelerates with
memchr/Teddy;
 combined into one alternation,
 the union of disparate
prefixes loses that optimisation,
 and per-byte scan cost goes up by
~32 us per file.

Per-file overhead × Linux file count:
 32 us × 93,694 files =
**3.0 s of additional cost on Linux**.
 The current Linux --all is
2.0 s;
 combining would push it to ~5 s.
 Not worth running.

A per-corpus heuristic to choose between Combined and All-Singles
(based on file count or byte volume) would itself need a stat-walk
pass to gather metadata before scanning starts,
 adding ~30 ms tax
on Linux.
 The current code's "all-Singles below threshold,
 Combined
above" decision is bench-derived for Mono's residual count (4 < 16,
all-Singles);
 it would be incorrect to apply the same threshold to
Linux without knowing whether the corpus is Mono-like or kernel-like.
The simplest correct rule is "all-Singles when total residual count
is small,
" which is what we have.

### Genuinely still open (low priority)

- **Multi-substring AC gating per rule.
  ** Would help if a future
  ruleset has alternation-bearing residuals where each branch's
  required literal is >= MIN_PREFIX_LEN.
   The current 4 production
  residuals do not match this profile (analysed above).
   Not worth
  building speculatively.
- **Per-file residual contention reduction.
  ** Currently the residual
  scan runs each shard's `is_match` sequentially within a file.
   With
  4 single-shards on Mono and 4 on Linux,
   parallelising across shards
  inside a file might win on Linux (4 shards × per-rule find_all
  could fan out across cores).
   Bench-untested.
   Marginal ceiling
  since wall is already at 2 s with 10.5x parallelism across files.
- **Walker pseudo-required quantifier detection.
  ** The walker doesn't
  detect `={0,3}` as making the preceding `=` optional;
   minor
  unsoundness hazard but no observed false negatives in the
  betterleaks corpus.

## Architecture summary

The hot path runs **two** Aho-Corasick `find_overlapping_iter` passes per file:

1. **Case-sensitive AC**:
    emits user-authored literal-rule hits AND queues
   regex-rule prefix matches whose required substring is case-sensitive
   (e.g. `\b(p8e-(?i)[a-z0-9]{32})`:
    the leading `p8e-` is case-sensitive).
2. **Case-insensitive AC** (`AhoCorasickBuilder::ascii_case_insensitive(true)`):
   queues regex-rule prefix matches whose required substring is case-insensitive
   (e.g. the betterleaks shape `(?i)[\w.-]{0,50}(?:adafruit)...` puts `adafruit`
   here).
    Literal rules NEVER live in this bucket;
    user literals are always
   case-sensitive.

Shipped optimisations in load order:

- **Substring-extracting prefix walker.
  ** `rule.rs::extract_required_prefix`
  walks the regex source past optional/required atoms (character classes,
  perl-classes,
   `(?:...)` groups),
   recursing into required group bodies to
  pull out literal substrings that must appear in any match.
   Examples:
  `[a-z]{4}_RESID_<tag>_[A-Za-z0-9]{12}` extracts `_RESID_<tag>_`;
  `(?i)[\w.-]{0,50}(?:adafruit)...` extracts `adafruit` (ci=true).
- **`(?i)` flag handling.
  ** Leading `(?i)` flag groups and `(?i)` inline
  flag changes set the extracted substring's case-insensitive bit;
   the
  loader routes the substring onto the case-insensitive AC bucket.
- **Escape-character soundness.
  ** The walker treats `\<non-alphanumeric>`
  as a literal of the second char (`\_` -> `_`,
   `\=` -> `=`,
   etc.).
  Previously `\_` ended the walk;
   ~25 betterleaks-shape rules with
  `doo\_v1\_` style bodies now route onto AC.
- **Alternation-soundness bail.
  ** When the walker encounters a top-level
  `|` (alternation in the current scope),
   it returns `None`.
   Without
  this,
   `/foobar|barfoo/` would extract "foobar" and AC-gate on it,
  silently missing files that contain only "barfoo".
- **Scoped flag-group skip.
  ** `(?flags:body)` and `(?-flags:body)` are
  skipped without extraction (the body's flag context may invert outer
  ci;
   merging into the outer accumulator would require multi-ci
  tracking).
   Rules whose only required substring lives inside a scoped
  flag group land in residual.
- **Hybrid engine (`CompiledRegex::{Resharp, Plain}`).
  ** Each rule
  compiles via the standard `regex` crate when its source contains no
  set-algebra operators (`A&B` intersection,
   `~(A)` complement,
   class-
  level `[A&&B]` / `[A~~B]`).
   Rules using set-algebra fall back to
  resharp.
   The combined gate for residual shards picks the engine per
  chunk via `uses_set_algebra`,
   a shallow string scan over the chunk's
  rules.
   On the betterleaks corpus this routes 257 of 259 rules to
  the regex crate;
   phase 1 (classify + per-rule compile) drops from
  ~2 s on resharp-only to ~440 ms on hybrid.
   Live in
  `src/rule/engine.rs`.
- **Unicode-off compile with try-and-fallback.
  ** Each non-set-algebra
  rule compiles first with `unicode(false)`;
   on failure the loader
  retries with `unicode(true)`.
   Disabling unicode strips case-folding
  tables,
   codepoint-range expansion,
   and unicode-aware `\b`/`\d`/`\w`
  semantics from the compile and per-byte scan.
   Bench-verified 90x
  Phase 1 speedup AND 17x Mono `--all` wall-time speedup (641 ms ->
  37 ms).
   Soundness preserved:
   rules using unicode-property classes
  (`\p{Han}`,
   etc.),
   multi-byte chars inside `[...]` classes,
   or the
  `(?u)` flag transparently fall through to unicode-on.
   Literal
  multi-byte UTF-8 sequences in the regex source compile fine in
  bytes-mode without unicode:
   the parser treats them as the matching
  byte sequence,
   so they take the fast path.
   Lives in
  `src/rule.rs::compile_plain_rule` and the matching combined-gate
  fallback in `src/rule/shards.rs::try_compile_combined`.
- **Greedy combine-partition for residual shards.
  **
  `src/rule/shards.rs::build_residual_shards` now uses divide-and-
  conquer:
   try compiling all positions into one combined-alternation
  gate;
   on success emit one Combined shard;
   on failure split in half
  and recurse via `rayon::join`.
   Bottom-out at len=1 emits a Single
  shard reusing the Phase-2a-compiled per-rule regex (no fresh
  Regex:
  :
  new).
   Threshold guard `GREEDY_COMBINE_THRESHOLD=16`:
   below
  it,
   all positions are emitted as Singles directly.
   The threshold
  is bench-derived on Mono's 4-residual case,
   where each rule has a
  strong literal-prefix anchor (`SK`,
   `Q~`,
   `\d{15,16}`,
   `hvs.`) that
  the regex crate accelerates with memchr/Teddy;
   combined into one
  gate,
   that optimisation is lost and per-byte scan cost rises
  ~24 us per file (+86 ms across 2700 files).
   For larger residual
  buckets where individual rules already lack a usable literal
  prefix,
   the trade flips and Combined wins.

## L2 line-start index (shipped)

`scan.rs` builds a `Vec<usize>` of newline byte offsets via
`memchr::memchr_iter` the first time any hit fires,
 and shares it through
a `OnceLock` across the AC literal-emit path,
 the prefix-matched
`par_iter`,
 and every residual-shard `par_iter`.
 `line_and_col` and
`end_in_line` are now `partition_point` lookups on that vec instead of
walks from byte 0.

Why this matters in the worst case:
 a single file with **N hits**
previously paid 2 walks per hit,
 each O(file_size),
 so total cost was
O(N * file_size).
 The pathological case "rogue agent wrote one forbidden
literal a million times in one 43 MB file" would have taken ~18 minutes
of pure column-counting at 10 GiB/s memory bandwidth.

With L2:

```text
1M hits in a 43 MB file:    1.48 s wall  (99% CPU)
                            outputs 1M correct redacted lines
```

The clean-path numbers are unchanged:
 in 99% of files no hit fires,
 so
`build_line_index` never runs and the pre-L2 cost shape holds.
 The L2
build itself is SIMD memchr + an `i + 1` push per newline,
 and lookups
are `partition_point` (O(log L)).

## Mmap experiment (rejected)

The plan's E2 entry hypothesised that swapping `fs::read` for
`memmap2::Mmap::map` in the per-thread fused scan loop would save one
alloc + memcpy per file and let the kernel readahead-pipeline page
faults across `--all`.
 Tested apples-to-apples against `fs::read`
(both binaries built from the same source modulo the read-vs-mmap
swap) on **2026-05-02**:

Monochromatic,
 30 runs each:

```text
                    fs::read              mmap                  delta
example-all         15.7 ms ± 1.6 ms      21.2 ms ± 1.0 ms      mmap +35% wall
synth1k-all         26.5 ms ± 2.3 ms      32.8 ms ± 1.2 ms      mmap +24% wall
```

Linux kernel (93693 files,
 1.5 GiB),
 5 runs each:

```text
                    fs::read              mmap                  delta
10k+0% residual     687 ms ± 81 ms        986 ms ± 103 ms       mmap +43% wall
10k+20% residual    729 ms ± 130 ms       947 ms ± 7 ms         mmap +30% wall
```

System CPU (kernel time) is the smoking gun:

```text
Linux 10k+0% residual:
  fs::read:  user 6640 ms,  sys 1195 ms,  wall 687 ms
  mmap:      user 6326 ms,  sys 3491 ms,  wall 986 ms
                                       ^^^^^^^^^^
  mmap nearly tripled kernel time, +2300 ms aggregate sys time.
```

Why mmap loses for our workload:
 per-file mmap setup creates a
`vm_area_struct` and corresponding page table entries;
 per-file
unmap tears them down.
 The Linux kernel corpus hits this path 93693
times,
 the Mono repo 2699 times.
 With files averaging 16 KB,
 the
saved alloc + memcpy (≈ tens of microseconds per file) is dwarfed
by the VMA bookkeeping and page-fault handling.
 `madvise(SEQUENTIAL)`
made the regression worse,
 not better,
 because each `madvise` is an
extra syscall per file.

Rejection criteria:
 every measured workload regressed by ≥ 24% wall;
no workload showed any improvement.
 The dependency (`memmap2`) and
the `unsafe { Mmap::map(...) }` block were removed.
 Don't re-attempt
on this code base unless input file sizes shift to averages well
above ~1 MiB per file (the threshold where saved memcpy outweighs
VMA setup is roughly that order on Linux).

This is the kind of optimisation that "should" win on paper and
empirically does not.
 Future "mmap is faster,
 just use mmap" PRs
should be challenged with the apples-to-apples bench reproduced
in `## Reproduce`.

## Reproduce

```sh
BIN=target/release/forbidden-strings
EX=../../../forbidden-strings.local.example.txt   # adjust path as needed
RU=/tmp/claude/synth-rules.txt                    # regen with node /tmp/claude/gen-fs-rules.ts

# realistic ruleset
hyperfine --warmup 3 --runs 30 \
  --command-name 'example-startup' "$BIN --rules $EX" \
  --command-name 'example-all'     "$BIN --rules $EX --all"

# synthetic 1k ruleset on Monochromatic
hyperfine --warmup 3 --runs 30 --ignore-failure \
  --command-name '1k-startup'   "$BIN --rules $RU" \
  --command-name '1k-all-mono'  "$BIN --rules $RU --all"

# synthetic 10k ruleset on Monochromatic (regen with node /tmp/claude/gen-fs-rules-10k.ts)
RU10=/tmp/claude/synth-rules-10k.txt
hyperfine --warmup 3 --runs 30 --ignore-failure \
  --command-name '10k-startup'  "$BIN --rules $RU10" \
  --command-name '10k-all-mono' "$BIN --rules $RU10 --all"

# residual sweep on the Linux kernel
# Clone (one-time): git clone --depth=1 --single-branch https://github.com/torvalds/linux.git /tmp/claude/linux
# Generate variants (one-time): node /tmp/claude/gen-residual-sweep.ts
cd /tmp/claude/linux && hyperfine --warmup 2 --runs 5 --ignore-failure \
  --command-name 'r=0'    "$BIN --rules /tmp/claude/sweep-resid0000.txt --all" \
  --command-name 'r=100'  "$BIN --rules /tmp/claude/sweep-resid0100.txt --all" \
  --command-name 'r=500'  "$BIN --rules /tmp/claude/sweep-resid0500.txt --all" \
  --command-name 'r=1000' "$BIN --rules /tmp/claude/sweep-resid1000.txt --all" \
  --command-name 'r=2000' "$BIN --rules /tmp/claude/sweep-resid2000.txt --all"

# L2 pathological case (1M-hit single file)
node -e 'const fs = await import("node:fs"); const line = "PLACEHOLDER_DOES_NOT_EXIST_IN_THIS_REPO_XX\n"; const buf = Buffer.alloc(line.length * 1_000_000); for (let loopIndex = 0; loopIndex < 1_000_000; loopIndex++) buf.write(line, loopIndex * line.length); fs.writeFileSync("/tmp/claude/million-hits.txt", buf);'
/usr/bin/time -v $BIN --rules $EX /tmp/claude/million-hits.txt > /dev/null

# Mmap A/B (only relevant if reconsidering the mmap rejection above)
# Build two binaries:
#   1. fs::read variant: src/main.rs uses `fs::read(p).unwrap_or_default()`
#   2. mmap variant: src/main.rs uses `unsafe { Mmap::map(&File::open(p)?) }`
#                    plus `memmap2 = "0.9"` in Cargo.toml
# Then bench:
hyperfine --warmup 3 --runs 30 --ignore-failure \
  --command-name 'fs-read' "/path/to/fs-read-binary --rules $EX --all" \
  --command-name 'mmap'    "/path/to/mmap-binary    --rules $EX --all"
```

`/tmp/claude/clean.txt` is a single-line plain-text file;
`/tmp/claude/violating.txt` contains one literal taken from `synth-rules.txt`;
`/tmp/claude/violating-10k.txt` contains one set-algebra hit and one residual hit
for the 10k ruleset.
 Generators for synthetic rulesets and the sweep variants live
under `/tmp/claude/gen-*.ts` (Bun TypeScript).

## Engine constraint

The scanner uses [resharp][resharp] for regex matching.
Resharp is **load-bearing for the rule grammar**,
 not just a performance choice:

- `A&B` set intersection
- `~(A)` complement
- Class-level `[A&&B]` intersection and `[A~~B]` symmetric difference

These operators have no equivalent in `regex` / `regex-automata` / PCRE.
The package README documents the worked example `/key_[0-9]{5}&~(key_0{5})/`,
which flags any five-digit `key_` value except the all-zeros placeholder
without lookaround.
 Switching engines would require either dropping support
for these rule shapes or maintaining two parsers and routing rules between them.

Concretely:
 any plan or rationale that proposes "swap the combined gate to
`regex-automata` to enable DFA serialization" is rejected.
The combined-regex compile cost is bounded by sharding (see Architecture summary)
and is now a small fraction of overall runtime.

[resharp]: https://github.com/ieviev/resharp

## Comparison with betterleaks v1.1.2

Betterleaks is the upstream source for the ported rule corpus (see
`src/mise.port-betterleaks.ts`).
 Measuring against it is informative
because both tools scan for secrets,
 but the comparison is not a
horse race;
 the tools serve different use cases and the numbers
reflect architectural choices,
 not engine quality in the abstract.

**Betterleaks v1.1.2**,
 Go binary,
 RE2 engine,
 Aho-Corasick keyword
prefilters,
 CEL-based post-match filtering,
 `dir` mode (no git history).
Installed via mise/aqua.
 Tested 2026-05-03 on the same machine as all
other benches (AMD Ryzen 7 8700F,
 16 threads).

### Same-content comparison (git-tracked files only)

To isolate engine throughput from file-walking differences,
 both tools
were run on identical file sets:
 Monochromatic's git-tracked content
(~21 MiB,
 ~2700 files) and the Linux kernel (torvalds/linux,
 ~1.58 GiB,
~93k files).

Monochromatic,
 git-tracked content via `/tmp` copy (betterleaks) and
`--all` (forbidden-strings),
 5 runs each:

```text
forbidden-strings   startup-only     7.3 ms ±  0.6 ms
forbidden-strings   --all            28 ms           (1 run, /tmp copy)
betterleaks         startup-only   174 ms             (scanned 0 bytes)
betterleaks         dir              557 ms             (6.61 MiB in 557 ms)
```

Startup ratio:
 **~24x**.
 Full-scan ratio:
 **~20x**.

Linux kernel,
 `--all` vs `dir` (betterleaks `dir` also walks git-tracked
files in this corpus since there are no node_modules-like subtrees),
 single
run each:

```text
forbidden-strings   --all          1.6 s    (user 22 s,  sys 0.8 s, 13x parallelism)
betterleaks         dir            5.3 s    (user 66 s,  sys 1.8 s, 12x parallelism)
```

Scan ratio:
 **~3.3x**.
 Per-byte throughput:
 forbidden-strings ~1.0 GB/s,
betterleaks ~0.3 GB/s.

### Why the gap widens on monorepos

Betterleaks `dir` walks the entire directory tree;
 it does not respect
`.gitignore`.
 On the Monochromatic monorepo (which contains
`node_modules/`,
 `dist/`,
 `target/`,
 etc.),
 this means scanning 4.28 GB
of content instead of the 21 MiB of git-tracked source:

```text
forbidden-strings   --all            43 ms    (working tree + .gitignore + gix-index; 21 MiB)
betterleaks         dir              86.5 s   (full tree walk; 4.28 GB)
```

Wall-clock ratio:
 **~2000x**.
 The ratio is dominated by the 200x data
volume difference,
 not the engine;
 but the data volume difference is
real and user-observable.
 Forbidden-strings walks the working tree via
the `ignore` crate (which honours `.gitignore`) and unions with an
in-process `.git/index` read (`gix-index`) to recover force-added
gitignored files,
 so it skips generated and vendored content by
default.
 Betterleaks' `git` command scans git history (patches),
 which
is a different (and more expensive) workflow;
 its `dir` command is the
closest comparable mode for working-tree-only scanning.

### Why forbidden-strings is faster per byte

Three architectural choices account for most of the per-byte gap:

1. **Dual AC gate with lazy regex dispatch.
   ** On the 99%-clean file
   path,
    forbidden-strings runs two Aho-Corasick passes and,
    finding
   zero hits,
    skips the regex engine entirely.
    RE2 (betterleaks' engine)
   also uses Aho-Corasick keyword prefilters,
    but the RE2 match path
   is heavier per file even on no-match content because the prefilter
   hit must be verified against the full DFA.
    Forbidden-strings' AC
   only queues a `find_all` when the prefix is seen;
    on a clean file,
   no `find_all` runs at all.

2. **Hybrid engine dispatch.
   ** 257 of 259 ported rules compile via the
   `regex` crate (which applies memchr/Teddy literal-prefix acceleration
   per-rule).
    RE2 compiles all rules into a shared DFA that cannot
   apply per-rule literal-prefix fast paths.
    The per-rule acceleration
   matters on large corpora where many files contain a short prefix hit
   but don't match the full rule.

3. **Native binary startup.
   ** The Rust binary (LTO + `codegen-units = 1` +
   `panic = "unwind"` + `overflow-checks = true` + `strip = true`) starts
   in ~9 ms. The Go binary starts in ~174 ms (GC init,
    goroutine scheduler,
   config parse).
    For the pre-commit hook use case (scan a handful of
   staged files,
    sub-5 ms budget),
    the startup gap alone makes betterleaks
   unsuitable regardless of per-byte throughput.
    The unwind +
   overflow-checks pair is load-bearing for the resharp-panic safety
   wrapper;
    see `Cargo.toml:49-97`.

### What betterleaks does that forbidden-strings does not

The speed gap is not free.
 Betterleaks provides capabilities that
forbidden-strings deliberately omits (see README "When to pick
something else"):

- **CEL-based filtering.
  ** Post-match filters evaluate entropy,
   BPE
  token efficiency,
   git author,
   file path,
   and string-allowlist
  membership.
   Forbidden-strings has no post-match filtering;
   the port
  drops these filters and documents the resulting false-positive
  increase.
- **Async HTTP validation.
  ** `validate` blocks call provider APIs to
  check if a detected secret is live.
   No equivalent in
  forbidden-strings.
- **Git history scanning.
  ** `betterleaks git` walks every diff in
  every commit.
   Forbidden-strings only scans the working tree.
- **SARIF/JSON/CSV output.
  ** Machine-readable reports for GitHub code
  scanning upload,
   CI dashboards,
   etc. Forbidden-strings only emits
  `path:line:cols rule=N` to stderr.
- **Per-rule path scoping.
  ** `path = '''(?i)\.ya?ml$'''` restricts a
  rule to matching files.
   Forbidden-strings scans every file
  unconditionally (by policy).
- **Allowlists.
  ** Per-rule `[[rules.allowlists]]` with regex-based
  exceptions.
   No equivalent.

These features are part of why betterleaks is slower per byte:
 CEL
evaluation,
 entropy/token-efficiency scoring,
 and validation all add
per-match cost that forbidden-strings avoids by not having them.

### When the comparison is not relevant

- If the rule set can ship in the repo and set-algebra is unnecessary,
  betterleaks is the better tool (larger ecosystem,
   real validation,
  SARIF output).
   Forbidden-strings' niche is rules that cannot be
  committed.
- Betterleaks' `git` command (scanning commit history) has no
  forbidden-strings equivalent.
   The per-byte comparison only covers
  working-tree scanning.
- The rule corpora are not identical:
   the port is lossy (CEL filters,
  entropy,
   allowlists,
   path scoping all dropped).
   Betterleaks produces
  fewer false positives on the same rules because it has more filtering.

### Reproduce

```sh
# Install betterleaks (e.g. via mise/aqua or brew)
# Build forbidden-strings
mise run //package/forbidden-strings/cli:build
FS=package/forbidden-strings/cli/target/release/forbidden-strings
RULES=forbidden-strings.local.txt

# Startup-only (0 bytes scanned)
hyperfine --warmup 2 --runs 5 "$FS --rules $RULES"
betterleaks dir /dev/null   # reports startup time in "scanned 0 bytes in Nms"

# Same-content Monochromatic (git-tracked only)
git ls-files -z | xargs -0 -I{} cp --parents "{}" /tmp/fs-bench-mono/
hyperfine --warmup 2 --runs 5 "$FS --rules $RULES --all"
cd /tmp/fs-bench-mono && time betterleaks dir .

# Same-content Linux kernel
cd /tmp/claude/linux && hyperfine --warmup 2 --runs 3 "$FS --rules $RULES --all"
cd /tmp/claude/linux && time betterleaks dir .

# Full-directory Monochromatic (betterleaks scans everything; fs scans git-tracked only)
hyperfine --warmup 2 --runs 5 "$FS --rules $RULES --all"
cd /path/to/monochromatic && time betterleaks dir .
```

## When to re-bench

Re-run the commands above and append a dated block to **Last benched** /
**Results** (do not overwrite:
 regressions need history) when **any** of:

- A change touches `src/lib.rs` (CLI dispatch + file fan-out;
   `src/main.rs` is a
  five-line wrapper around `run_cli_from_env`),
   `src/scan.rs` (per-file scan logic),
  `src/walk.rs` (walker + gix-index union),
   or `src/rule.rs` (rule loading and
  bucketing)
- A change touches `Cargo.toml` profile or dependency versions
- The repo grows past ~5000 tracked files or ~100 MiB total
- Realistic `--all` exceeds **180 ms** in casual use (~3x current 58.6 ms),
  startup-only exceeds **30 ms** (~3x current 8.8 ms),
   or synthetic-1k
  `--all` exceeds **500 ms** (~3.5x current 139 ms)

If none of the above hold,
 the numbers in this file are still trustworthy.

The deferred opportunity catalog (extension/size pre-filter,
 chunked-concat,
bucketed alternation) lives in `~/.claude/plans/dapper-coalescing-horizon.md`.
Do not re-derive that analysis on every session:
 read the plan,
 then decide.

Items already resolved in this code base:

- **L2 (line-start index for `line_and_col`)**:
   shipped 2026-05-02 evening.
  See `## L2 line-start index (shipped)` above.
- **E2 (`mmap` for `--all`)**:
   tested and rejected 2026-05-02 evening with
  apples-to-apples bench data.
   See `## Mmap experiment (rejected)` above.
  Do not re-attempt without input file sizes shifting to >> 1 MiB averages.
- **E1 (extension/size pre-filter)**:
   rejected by user policy
  (rule must scan files regardless of extension/size to catch
  accidentally renamed files and adversarial agent-generated content).
