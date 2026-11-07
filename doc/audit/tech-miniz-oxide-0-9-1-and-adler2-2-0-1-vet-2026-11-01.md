# Vet report: miniz_oxide 0.9.1 and adler2 2.0.1

## Status

- Status:
  complete for the hard gates and the source audit of an already adopted dependency;
  no gate failed.
- Lifecycle phase:
  hard-gate confirmed and validated;
  not scored,
  because no alternative was discovered or compared here.
- Subject:
  `miniz_oxide` 0.9.1 (features `with-alloc` only) and its one runtime dependency `adler2` 2.0.1.
- Scope:
  the pure Rust zlib inflate used by the in-process repository readers of `package/git-policy/cli`
  (`src/native/repo_read_object.rs`,
  `inflate_zlib`).
- Started and last updated:
  2026-11-01.
- Governing skill:
  `.agents/skills/choosing-technology/SKILL.md`,
  last commit `45d42d9b23d2f2752f9c512683b45905e13afe99`;
  the loaded copy has SHA-256 `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`.
- Compatibility fingerprint (SHA-256 of the RFC 8785 form of the inputs listed in the skill):
  `567c792df182bf63e5c08ef139a6e24712485ba97066391b95a8fc469d0182ab`.
- Active audit owner:
  the agent that implemented the follow-up items of the in-process reads work;
  no other owner.
- Prior compatible report:
  none;
  this is the first report for the subject,
  and the dependency was added without one
  (`doc/planning/cli-git-in-process-reads.md`,
  "Technology choice",
  which says its source was not audited).

## Context and hard constraints

The readers inflate loose objects and pack entries,
which are zlib streams in any repository the wrapper runs in,
including a repository someone else made.
The bytes are untrusted,
and the wrapper runs inside the Git command path,
so the high-trust overlay applies.

Hard constraints:

- the source must be inspectable and map to the published artifact;
- the license must allow use in an LGPL-3.0-or-later package;
- no `unsafe`,
  build script,
  native,
  Wasm or prebuilt artifact in the used path;
- a decoder fault must not become a wrong answer where Git would fail,
  and must be survivable;
- Linux x64 is required,
  other platforms are the ones the wrapper builds for.

Base category:
inspectable open-source local technology.
Overlays:
high-trust execution (parses untrusted repository bytes);
native or prebuilt (not applicable,
none present);
incumbent replacement (not applicable,
there is no incumbent:
the crate had no zlib implementation before);
sensitive data and multi-platform (not applicable beyond the build matrix recorded below).

## Discovery and alternatives

Discovery was not performed.
The dependency was chosen by the earlier work from the lockfile and the registry cache only.
That work named three alternatives (the `gix` sub-crates,
a hand-written inflate,
loose objects only) and ranked
them without vetting (`doc/planning/cli-git-in-process-reads.md`,
"Technology choice").
This report neither repeats nor extends that ranking,
so it recommends nothing about the alternatives.
Its limit:
it answers "does the adopted dependency pass the gates",
not "is it the best candidate".

## Evidence records

All accessed 2026-11-01.
Sources:
crates.io API (`https://crates.io/api/v1/crates/miniz_oxide` and `/adler2`),
the upstream repositories at
the commits the published crates record,
and a local run.

### License and terms

- miniz_oxide 0.9.1:
  `MIT OR Zlib OR Apache-2.0` (`Cargo.toml`,
  `LICENSE-MIT.md`,
  `LICENSE-ZLIB.md`,
  `LICENSE-APACHE.md` in the crate).
- adler2 2.0.1:
  `0BSD OR MIT OR Apache-2.0`.
- Status:
  pass.
  Permissive terms with no copyleft or attribution conflict for LGPL-3.0-or-later.

### Provenance and source mapping

- The crates record their commits in `.cargo_vcs_info.json`:
  miniz_oxide `4e582392df3a739d2b0dfd2c537dc33e8942be38`
  (path `miniz_oxide`),
  adler2 `89a031a0f42eeff31c70dc598b398cbf31f1680f`.
- Command:
  `diff -r <registry copy>/src <upstream archive at that commit>/src` for each crate,
  working directory
  the scratch tree,
  Linux x86_64.
  Exit status 0 and no output for both:
  the published source equals the repository
  source at the recorded commit.
- Lockfile checksums (`package/git-policy/cli/Cargo.lock`):
  miniz_oxide
  `b63fbc4a50860e98e7b2aa7804ded1db5cbc3aff9193adaff57a6931bf7c4b4c`;
  cargo verifies them on download.
- Status:
  pass.

### Native, Wasm, build scripts and unsafe code

- `build = false` in both published manifests;
  no `build.rs`;
  no proc macro;
  no `links`.
- `#![forbid(unsafe_code)]`:
  `miniz_oxide/src/lib.rs` line 25 and `adler2/src/lib.rs` line 16.
  The three other
  occurrences of the word `unsafe` in `miniz_oxide/src/deflate/core.rs` (lines 599,
  601, 1354) are comments.
- No native,
  Wasm or prebuilt artifact is shipped or fetched.
- Status:
  pass.

### Dependency surface

- Our features:
  `default-features = false, features = ["with-alloc"]`.
  The lock adds exactly two packages:
  miniz_oxide and adler2 (`Cargo.lock` lines 6 and 1094).
- adler2 has no dependencies (`serde`,
  `simd-adler32` and the `rustc-std-workspace-*` crates of miniz_oxide are
  optional and not enabled).
- Same-author packages:
  both crates are published by the same maintainer account (`oyvindln`),
  so the two packages
  share one maintainer;
  this is a concentration,
  recorded under maintenance.
- Status:
  pass.
  Runtime dependency count 1.

### Production source read

Read:
`src/inflate/mod.rs` (391 lines),
`src/inflate/core.rs` (2381 lines,
the decoder),
`src/inflate/output_buffer.rs`
(124),
and the call sites in this repository.

- `decompress_to_vec_inner` (`inflate/mod.rs` lines 206 to 250) grows its output by doubling up to the caller's
  limit and returns an error at the limit,
  so memory is bounded by the limit we pass.
- The zlib wrapper is parsed with `TINFL_FLAG_PARSE_ZLIB_HEADER`;
  the Adler-32 check is on unless
  `TINFL_FLAG_IGNORE_ADLER32` is given,
  which this crate never passes.
- The non-wrapping-buffer flag we set makes the decoder reject a match distance before the start of the output
  (the check upstream pull request 211 extended to the wrapping mode,
  which we do not use).
- Panic paths:
  `assert!` at `inflate/core.rs` lines 365,
  1120,
  1121,
  1136,
  1137,
  1142,
  1143 and `unreachable!` at
  lines 1148 and 1509 guard decoder invariants;
  they cannot be proven unreachable by reading alone,
  so they
  were exercised (below) and the readers now survive a panic.
- Excerpts and line numbers refer to the registry copy of 0.9.1,
  identical to the upstream commit (see provenance).

### Tests and continuous integration

- Upstream CI (`.github/workflows/test.yml`):
  stable,
  nightly,
  minimum Rust 1.60,
  macOS,
  Windows (MSVC and GNU),
  `--features simd`,
  `--no-default-features`,
  `--features block-boundary`,
  wasm build,
  wasip1 under wasmtime,
  and the serde test crate.
- Upstream tests:
  29 unit tests,
  `tests/flush.rs` (2),
  `tests/test.rs` (20 or 21 with `block-boundary`).
- Run here (Linux x86_64,
  Rust nightly of the repository toolchain,
  `cargo test --offline`,
  in a process with a
  private network namespace,
  `unshare --user --map-root-user --net`,
  limits `CPUQuota=200%`,
  `MemoryMax=2G`,
  `TasksMax=128`,
  `IOWeight=30`,
  scratch target directory),
  on the upstream sources at the recorded commit with
  a manifest reduced to the crate,
  `adler2` and the three features without optional dependencies:
  default features,
  29 + 2 + 20 + 2 passed;
  `--features block-boundary`,
  29 + 2 + 21 + 2 passed;
  `--no-default-features`,
  7 unit tests passed (the integration tests need `with-alloc`).
  The first run of `tests/test.rs` failed 3 of 20 because it reads `../miniz/miniz.c` from the repository root,
  which I
  had not copied;
  with the file in place all pass.
  The failure was my fixture,
  not the crate.
- Not run:
  `--features simd` and the serde crate (the optional dependencies are not in the offline registry cache;
  our
  build enables neither);
  wasm and wasip1 (no such target here);
  the macOS and Windows jobs (no such host here).
  None can affect the used path:
  the used path is `inflate` with `with-alloc` on Linux x86_64.
- Fuzzing:
  five targets in `fuzz/fuzz_targets` (`fuzz_high`,
  `inflate_nonwrapping`,
  `roundtrip`,
  `roundtrip_zlib`,
  `via_flate2`).
  They are not run in CI:
  `run_fuzz.sh` begins with `# TODO: This is broken at the moment.`,
  and
  upstream issue 196 ("Fuzz targets fail to build") was opened 2026-07-18 and last updated,
  closed,
  2026-08-12.
  Not executed here.
- Mutation testing:
  none found (no `cargo-mutants` or `mutants` configuration in the repository).
- Property tests:
  none found.

### Maintenance

- Releases (crates.io):
  0.9.1 on 2026-03-13,
  0.9.0 on 2026-01-12,
  0.8.9 on 2025-06-09;
  0.8.6 yanked.
  Latest release
  is 0.9.1;
  no release in the seven and a half months since.
- Downloads:
  1,057,353,234 total,
  255,799,097 recent for miniz_oxide;
  486 reverse dependencies.
  adler2:
  634,938,948 total.
  Registry popularity is corroboration only.
- Contributors:
  `oyvindln` 267 commits,
  `Frommi` 179,
  the rest at most 15.
  One person does the current maintenance.
- Issue and pull request sample (GitHub API,
  items updated since 2025-11-01):
  18 issues and 27 pull requests,
  which
  is more than 20 items,
  so the 10 most recently updated of each were read:
  - pull requests:
    `oyvindln` merged 221 and 214,
    and merged two outside fixes (204, 211) on 2026-10-13 and
    2026-10-29;
    five outside pull requests are open (209,
    217,
    218,
    219, 220) and one (216) was closed unmerged.
    Maintainer actions are frequent and recent;
    the master branch had a commit on 2026-10-29.
  - issues:
    200 (a decoder bound comparison,
    no memory-safety effect) was closed by 204;
    212 and 215 (deflate) were
    closed by 214 and 221;
    open are 198,
    201,
    210,
    202 and 192.
    Maintainer comments on issues are sparse;
    the
    maintainer acts through pull requests.
- Fixes that are on master but in no release:
  204 (inflate bound check,
  performance only per its report) and 211
  (distance check in the wrapping mode,
  which we do not use).
  Both are outside what we consume.
- Open issue 198 applies to us:
  every block header rebuilds the Huffman tables,
  so chained empty blocks cost time
  proportional to the stream's length,
  not its output (reported as about 1.8 seconds per million blocks in a release
  build,
  1.25 bytes per block).
  The decoder's output limit does not bound it.
- Open issue 201 (a deserialized decompressor can panic) needs the `serde` feature,
  which we do not enable.
- Security disclosures:
  the GitHub security advisories list of the repository is empty;
  the RustSec package page does
  not exist (a missing page is weak evidence:
  no advisory was found,
  which is not proof that none is unpublished).
- Classification:
  active releases up to March,
  responsive maintainer in October,
  one maintainer,
  a backlog of
  untriaged reports,
  fix backlog unreleased.

### Human-auditability

- Non-test source we link:
  `inflate/` is 3,404 lines (`core.rs` 2381 including its test module,
  `stream.rs` 508,
  `mod.rs` 391,
  `output_buffer.rs` 124) plus `shared.rs`,
  and adler2 is 442 lines in two files;
  the deflate half
  (about 3,300 lines) is compiled but not called by this crate.
- Runtime dependencies: 1.
  Event buses,
  plugin handshakes,
  generated code,
  credentials,
  network and process
  execution:
  none;
  the crate does no I/O.
- Trade-off:
  the inflate core is one large state machine that a person can read in a session but not hold in mind;
  the alternative of a hand-written decoder moves that cost to this repository.

## Consumer boundary

These are the checks of the readers against the crate,
all in `package/git-policy/cli`:

- `repo_read_object::tests::damaged_streams_never_panic`:
  every loose object of a fixture repository (commit,
  tree,
  blobs) with one byte changed at every position by four patterns and every truncation,
  through `parse_loose` and
  `inflate_exact` under `catch_unwind`.
  No panic.
  This is a sample,
  not a proof.
- Found by this audit,
  in our code not in the crate,
  and fixed with tests that failed first
  (`bytes_after_a_loose_stream_are_left_to_git`,
  `a_stream_longer_than_the_bound_is_left_to_git`,
  `a_panicking_reader_is_left_to_git`):
  - a loose object with bytes after its zlib stream was answered;
    Git says `garbage at end of loose object`;
    the reader now uses the decoder's low-level function to learn the consumed length;
  - the cost of issue 198 is now bounded by `INFLATE_INPUT_LIMIT` (512 KiB of compressed bytes per stream;
    a longer
    stream,
    and so a larger object,
    goes to Git);
  - a panic in any reader is caught at the one hook and Git answers instead (the release profile keeps
    `panic = "unwind"`).

## Hard-gate outcome

- License:
  pass.
- Source inspectable and mapped to the artifact:
  pass (no difference between the registry copy and the upstream commit).
- No build script,
  native,
  Wasm or prebuilt artifact:
  pass.
- No `unsafe` in the used path:
  pass (`forbid(unsafe_code)` in both crates).
- Security boundary for untrusted input:
  pass with mitigations.
  The crate has no memory-unsafe path in safe Rust,
  one open resource-cost issue (198) that the input bound now limits,
  and decoder asserts that were exercised with
  damaged input and are caught if they ever fire.
- Required platform (Linux x86_64):
  pass.
- Reproducible validation:
  pass (upstream tests run here).

No gate failed,
so the dependency stays and no replacement was considered.

## Scored concerns and evidence limits

No weighted scoring was done:
scoring needs at least two validated finalists,
and this report vets one adopted dependency.
Concerns a later comparison should weigh,
none of them a hard failure:

- one active maintainer for both packages;
- fixes merged on master and unreleased for seven and a half months;
- fuzzing exists but is not run in CI,
  and no mutation or property tests;
- open reports of unbounded cost (198) and of a serde-only panic (201);
- the upstream platform jobs for macOS,
  Windows and Wasm were not reproduced here.

Evidence limits:
the issue sample is the most recent 18 issues and 10 pull requests,
read by title and,
for six items,
by body;
the other bodies were not read.
The damaged-stream test exercises one repository's objects,
not a corpus.
The fuzz targets were not run.
The independent review of the readers by a second vendor (recorded in the planning
document) was static and did not cover this dependency.

## Recommendation

Keep `miniz_oxide` 0.9.1 and `adler2` 2.0.1 as adopted.
Revisit when a release after 0.9.1 appears (it will carry the pending inflate fixes),
when issue 198 is fixed upstream
(the input bound can then be raised),
or when the maintainer situation changes.
This is a vetting result,
not an adoption record:
no decision document was written.
