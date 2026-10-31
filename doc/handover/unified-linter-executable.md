# Unified-linter executable handoff

Note 2026-10-30: `package/linter/monochromatic-lint.fuzz/bin/container.mjs`, named below, is `container.ts` now.

## Purpose and scope

Make `monochromatic-lint` a working executable,
as delegated by the main agent on 2026-10-05 under the design in
[`doc/planning/unified-linter.md`](../planning/unified-linter.md).
Before this work `package/linter/monochromatic-lint` was a library only.
This document records what was built,
how it was verified,
what differs from the incumbents,
and what remains.

Changes are confined to `package/linter/monochromatic-lint/`,
`package/linter/monochromatic-lint.fuzz/`,
and this document.
No file listed as owned by the concurrent mutation delegate was edited.
Nothing was published,
installed,
or switched over,
and no incumbent was changed or removed.
This is not a cutover-readiness claim.

## How to read and respond

Inspect `Decisions open for veto` and `Questions the comparison raises` first:
the first lists choices the design left to the implementation,
the second lists what the draft configuration would do to this repository at cutover.
Then read `Differential comparison` and `Limitations and remaining work`.
Respond with a veto or a correction to a specific entry,
an answer to a question,
or the next slice to start.

## Slice status

- Rule coverage audit and the `markdown/lfs-image-url` port:
  done.
- Orchestration and the executable:
  done.
- Binary-level container tests:
  done,
  10 tests.
- Fuzz sidecar extension:
  done,
  one new target;
  the bounded smoke run is recorded under `Fuzzing`.
- Differential comparison:
  done,
  read-only against the repository,
  with fixes applied only in disposable worktrees.

## Rule coverage

Every rule named in the design has an implementation file and a test file.
The number after each test file is its count of `#[test]` functions;
several functions hold more than one incumbent case.

### Rust rules

- `rust/max-lines`:
  `src/rust_rules.rs`,
  tests in `src/rust_rules_tests.rs` (7,
  shared with the next rule).
- `rust/require-rustdoc`:
  `src/rust_rules.rs`,
  tests in `src/rust_rules_tests.rs`.
- `rust/no-anonymous-functions`:
  `src/rust_no_anonymous_functions.rs`,
  tests in `src/rust_no_anonymous_functions_tests.rs` (5).
- `rust/require-explicit-types`:
  `src/rust_explicit_types.rs` with `src/rust_explicit_declarations.rs`,
  `src/rust_explicit_generics.rs`,
  and `src/rust_explicit_inference.rs`,
  tests in `src/rust_explicit_types_tests.rs` (1) and `src/rust_explicit_declarations_tests.rs` (8).

### Markdown rules

- `markdown/heading-increment`,
  `markdown/single-h1`,
  and `markdown/no-emphasis-as-heading`:
  `src/markdown_headings.rs`,
  tests in `src/markdown_basic_tests.rs` (10,
  shared with the other rules that name this file).
- `markdown/commands-show-output`:
  `src/markdown_commands.rs`,
  tests in `src/markdown_commands_tests.rs` (5).
- `markdown/no-duplicate-heading`:
  `src/markdown_duplicate_headings.rs`,
  tests in `src/markdown_duplicate_headings_tests.rs` (3).
- `markdown/no-trailing-punctuation`:
  `src/markdown_punctuation.rs`,
  tests in `src/markdown_punctuation_tests.rs` (3).
- `markdown/no-bare-urls` and `markdown/link-image-style`:
  `src/markdown_links.rs`,
  tests in `src/markdown_basic_tests.rs`.
- `markdown/fenced-code-language`:
  `src/markdown_code.rs`,
  tests in `src/markdown_basic_tests.rs`.
- `markdown/link-image-reference-definitions`:
  `src/markdown_definitions.rs`,
  tests in `src/markdown_definitions_tests.rs` (4).
- `markdown/no-pipe-tables`:
  `src/markdown_tables.rs` with `src/markdown_table_text.rs`,
  tests in `src/markdown_tables_tests.rs` (4) and `src/markdown_table_text_tests.rs` (4).
- `markdown/semantic-line-breaks`:
  `src/markdown_semantic_breaks.rs` with `src/markdown_break_points.rs`,
  `src/markdown_block_start.rs`,
  and `src/markdown_prose_context.rs`,
  tests in `src/markdown_semantic_breaks_tests.rs` (5),
  `src/markdown_break_points_tests.rs` (4),
  and `src/markdown_parity_tests.rs` (2).
- `markdown/lfs-image-url`:
  absent before this work and ported here.
  Rule in `src/markdown_lfs_image_url.rs` (11),
  with `src/markdown_lfs_config.rs` (5),
  `src/markdown_lfs_endpoint.rs` (5),
  `src/markdown_lfs_patterns.rs` (5),
  `src/markdown_lfs_oid.rs` (4),
  `src/markdown_lfs_sha256.rs` (3),
  `src/markdown_lfs_target.rs` (7),
  and `src/markdown_lfs_context.rs` (8);
  each count is that module's sibling `*_tests.rs`.

The audit established presence of an implementation and a test file for each rule.
It did not re-derive,
case by case,
that every incumbent test of the twelve previously ported Markdown rules has a native counterpart.
`Differential comparison` is the evidence for those rules on real files.

### The `markdown/lfs-image-url` port

The incumbent's rule tests (19 cases),
`.lfsconfig` tests,
tracked-path tests,
object-id tests,
and context tests were ported as the specification,
with BOM and astral-character fixtures added.
Edits replace only the written destination and use byte offsets directly;
the incumbent's second offset correction after astral characters is not reproduced.

The endpoint normalizer is the restricted standard-library function selected in
[`doc/planning/native-lfs-url-normalization-evaluation.md`](../planning/native-lfs-url-normalization-evaluation.md).
`src/markdown_lfs_endpoint_tests.rs` embeds `fixtures/lfs-url-parity.json` and asserts every case:
868 endpoints (459 equal to the incumbent,
289 rejected natively where the incumbent accepts,
120 rejected by both)
and 73 `.lfsconfig` texts (53,
11,
and 9).
All twelve rejection reasons are exercised.
No dependency was added.

## Orchestration

`src/main.rs` and a `[[bin]]` target named `monochromatic-lint` were added.
The executable's behavior lives in library modules so that tests and the fuzz sidecar run the same code.

- `src/run_process.rs`:
  real arguments,
  working directory,
  streams,
  and exit status.
- `src/run_command.rs`:
  mode selection,
  discovery,
  planning,
  and the file-mode run.
- `src/run_stdin.rs`:
  `--stdin` with `--stdin-filename`.
- `src/run_finish.rs`:
  output controls,
  debug notes,
  and creation of the semantic engine.
- `src/run_plan.rs`:
  memoized nearest-configuration lookup or `--config`,
  and one immutable plan per file.
- `src/run_check.rs`:
  the host file's own rules,
  then always-on processors,
  with each virtual file matched by its own logical path and every finding mapped to the host.
- `src/run_file.rs`:
  read,
  bounded fix loop,
  refusal findings,
  and the write.
- `src/run_write.rs`:
  same-directory temporary file,
  original permission bits copied,
  rename.
- `src/run_workers.rs`:
  bounded worker threads and per-file panic containment.
- `src/run_modes.rs` and `src/run_json.rs`:
  `--rules`,
  `--init`,
  and `--print-config`.
- `src/run_failure.rs`:
  core processing findings.
- `src/run_lfs.rs`:
  per-run sharing of LFS repository facts.
- `src/markdown_rule_settings.rs` and `src/markdown_dispatch.rs`:
  typed Markdown selection and dispatch in the incumbent's registry order.

`mise run //package/linter/monochromatic-lint:run -- <arguments>` builds and runs the executable from source.
`package/linter/monochromatic-lint/README.md` documents the command line,
exit statuses,
configuration lookup,
and embedded sources.

## Decisions open for veto

The design left these to the implementation.
Each is implemented as described and tested.

### Exit status of processing findings

A finding that marks incomplete processing exits 2,
following the existing `src/run_output.rs` contract.
That covers parse failures,
MDX errors,
caught panics,
unreadable or non-UTF-8 files,
unavailable semantic coverage,
and refused fixes.
The Markdown incumbent exits 1 for its equivalent synthetic diagnostics.

### Core finding codes

`core/processing-failure` for inability to process,
and `core/fix-refused` for a fix the engine declined.
`core/rust-type-resolution` already existed for the semantic rule.

### No configuration found

When no input file has a configuration file in its directory or any ancestor and `--config` is absent,
the run exits 2 with an explanation instead of printing nothing with status 0.
A run where only some files lack configuration lints the configured ones.
The design states only that an unmatched file is not linted.

### A refused fix reports the original source

When the fix loop refuses,
the file is unchanged and the findings are those of the original source plus one `core/fix-refused` finding.
The same applies when the write itself fails.

### An unmappable fix is dropped and its finding kept

When the processor refuses to project a virtual finding's fix onto the host,
the finding is still reported,
without a fix,
and `--debug` prints the refusal.
A finding whose position cannot be mapped at all becomes a processing finding.

### LFS repository discovery is per file

The repository root is the nearest ancestor of the linted file that holds `.lfsconfig`.
The incumbent searches from the working directory.
Both agree when the working directory is inside the repository.
Exclusion is decided before the endpoint is read,
so an excluded file is unaffected by an unusable endpoint.

### LFS endpoints outside the restricted contract are rejected

This is the difference the main agent asked to have recorded.
289 fixture endpoints that the TypeScript linter accepts are rejected natively with a named reason and a remediation:
other schemes such as `ssh://`,
IPv6 literals,
Unicode hosts,
dot segments,
backslashes,
and characters the incumbent would percent-encode.
A rejected endpoint is a processing finding on every file the rule checks in that repository,
not an inert rule.
This is an intended,
veto-open difference from the incumbent,
not a parity failure.
The rejection message never contains the endpoint,
which may hold credentials.

### LFS object paths that leave the repository

An object URL whose embedded path resolves outside the repository root is reported as missing and is never read.
The incumbent would read the file it resolves to.

### SHA-256 is implemented in the crate

A smudged file's object id is the SHA-256 of its bytes,
and the crate's lockfile holds no hashing crate.
`src/markdown_lfs_sha256.rs` implements FIPS 180-4 and is tested against the published vectors,
every padding boundary,
and digests measured with Node and coreutils.
The alternative is the `sha2` crate,
which is locked elsewhere in this repository.

### `--print-config` prints strict JSON

The output is an object with `file`,
`configuration`,
`base`,
`state`,
and,
when configured,
the merged and defaulted `rules`.
`state` is one of `configured`,
`unconfigured`,
`ignored`,
or `no-configuration`.

### Semantic preparation is source-only

`rust/require-explicit-types` loads its Cargo workspace without running build scripts.
No command-line option requests generated-source preparation.

### Display names

A file inside the working directory is reported relative to it,
without a leading `./`;
a file outside it keeps its absolute path.

### `--debug` names every file

`--debug` prints one note per file selected for linting and one per file ignored by configuration.
The comparison uses these notes to compare file sets exactly.

### Markdown columns count UTF-16 code units

Every Markdown finding reports its column in UTF-16 code units.
The Markdown incumbent reports node-anchored findings the same way,
but counts code points for offset-anchored findings (`pointAt` in `package/cli/markdown-lint/src/source-position.ts`).
On a line holding an astral character before the finding the two therefore print different column numbers
for the same position.
The repository has 16 such findings;
`Markdown findings` under `Differential comparison` has the measurement.

## Verification

Every gate ran through `mise run //package/linter/monochromatic-lint:lint:container`
with `MONOCHROMATIC_LINT_IMAGE_TAG=executable`:
a mount-free,
network-disabled container limited to 2 GiB,
2 CPUs,
and 128 PIDs for tests,
then Clippy with warnings denied against the same image.
This was the first exercise of the runner scripts after they gained the image-tag variable;
they needed no change.
Results are under `Gate log`.

### What the binary-level tests cover

`src/binary_tests.rs` is a separate Cargo test target that starts the built executable as a child process
in disposable directories inside the same container.
It asserts on real exit statuses and on the exact bytes of both streams:

- help,
  version,
  and nine usage errors;
- statuses 0,
  1,
  and 2 with their stream contracts;
- walking with `.gitignore`,
  nested `.gitignore`,
  `.ignore`,
  `.git/info/exclude`,
  hidden directories,
  `node_modules`,
  and every ignore flag;
- nearest-configuration lookup,
  `--config` resolved against the working directory,
  and block merging;
- atomic replacement shown by a changed inode,
  preserved permission bits,
  untouched files keeping inode and modification time,
  symbolic links,
  and no leftover temporary files;
- processor findings at host positions across CRLF,
  a block-quote prefix,
  astral characters,
  MDX,
  and rustdoc,
  with the fix written into the host comment;
- standard-input fixing with source on standard output and only JSONL on standard error;
- the commit-time invocation shape for `markdown/lfs-image-url`;
- identical output at concurrency limits 1,
  2,
  4,
  and 64,
  and an early-closed output pipe producing no error text.

Standard error is asserted to be exactly empty on every non-debug lint run,
which is the check for bare shutdown errors.

## Fuzzing

`package/linter/monochromatic-lint.fuzz` gained the `orchestration` target
(`src/orchestration.rs`,
`fuzz_targets/orchestration.rs`,
two seeds under `seed/orchestration/`,
and its entry in `bin/container.mjs`).
It drives `run_file::process_source`,
the call the executable makes for every file,
with a fixed configuration that selects virtual files by their paths.

Each input does three things.

- It picks one of seven hand-counted hosts,
  in LF or CRLF spelling,
  and asserts the exact sorted rule codes:
  a doc test inside rustdoc,
  a fence inside a block quote,
  a fence below JSX in MDX,
  two Rust levels of nesting (rustdoc,
  doc test,
  rustdoc,
  doc test),
  three fixable Markdown findings in one rustdoc comment,
  a clean host with a hidden doc-test line,
  and an undocumented authored `fn main`.
- It builds an adversarial group of one to three edits against one virtual file,
  with replacements such as a fence line,
  `*/`,
  CRLF,
  an astral character,
  and a rustdoc prefix,
  projects it to the host,
  and requires that an accepted group applies cleanly and that the rewritten host goes through the whole path again.
- It treats the raw input,
  when it is UTF-8,
  as a `.rs`,
  `.md`,
  and `.mdx` host.

For every host it asserts that findings name the host and address host bytes on character boundaries,
that a processing failure stops the fixer,
that a fixed result has no processing finding and is never empty for a non-empty source,
that a result the loop reports as settled is a fixed point,
and that a refused fix carries `core/fix-refused`.

`markdown/lfs-image-url` is deliberately absent from the fuzz configuration:
it reads the filesystem,
and the fuzz process has none of the repository.

The generator controls run as unit tests.
All 9 helper tests pass,
4 of them new.
The adversarial control counts its own reach:
of 1792 generated groups,
563 were applied and 1229 refused,
and the test fails if either path drops below 256.
The first run of the counted cases failed on one expectation,
which is the evidence that the comparison can fail:
an authored `fn main` in a doc test is not wrapped,
so it is an item of the virtual file and needs its own doc comment.
That case is now counted as `authored.rs`.

The bounded smoke run is `mise run //package/linter/monochromatic-lint.fuzz:smoke`:
unit controls,
Clippy,
an AddressSanitizer build,
then 30 seconds per target in mount-free containers limited to 2 GiB,
2 CPUs,
and 128 PIDs.
Its result is under `Gate log`.

## Differential comparison

### Method

The scratch harness is not committed;
its commands are recorded here so the run can be repeated.

- New executable:
  the debug build of commit `2aab0b324`,
  `monochromatic-lint --config <file> --debug --concurrency 4 .`.
- Rust incumbent:
  `package/linter/rust`,
  rebuilt from current source with `mise run //package/linter/rust:build`,
  `rust-linter --threads 4 .`.
  No `rust-linter.toml` exists in the repository,
  so it ran with its built-in defaults.
- Markdown incumbent:
  `node package/cli/markdown-lint/src/cli.ts --json --lfs-image-exclude=package/ssg/`,
  the root `lint:markdown` command plus `--json`.
- File sets:
  the new executable's `--debug` notes,
  `rust-linter --debug files .`,
  and the Markdown incumbent's own `discoverFiles(['.'])`.

Two configurations were used.
`draft` is the fenced block under `Draft repository configuration` in the design,
extracted byte for byte.
`host-only` is the same text with one leading block that globally ignores `**/*.rs/**`,
`**/*.md/**`,
and `**/*.mdx/**`,
so that no virtual file has rules.
The design's build order asks for the comparison with processors off;
processors are always on,
so `host-only` is how that is expressed.
A probe fixture with a bare URL in rustdoc and a closure in a fence gave six findings under `draft`
and none under `host-only`.

Every linter process ran inside `bwrap --ro-bind / /` without network,
so the whole filesystem was read-only to it,
and inside a systemd scope capped at 4 GiB and four CPUs.
A `touch` in the repository from inside that sandbox failed with `Read-only file system`.
Fix runs used the same sandbox with one writable bind:
a disposable `git worktree` of commit `8da338a3e` under the session scratch directory.
The repository itself was never writable to any linter.

Findings were compared twice:
over the live working tree,
and over a pristine worktree of commit `8da338a3e`.
Both gave the same counts and the same classification.
A finding is identical when file,
line,
column,
rule,
and message all match;
for Rust findings severity and the full span (byte offset and length) were compared as well.

### File sets

- Rust:
  803 files in both.
- Markdown and MDX,
  at commit `8da338a3e`:
  the new executable discovers 1512 and its configuration ignores 61,
  leaving 1451;
  the incumbent discovers 1451.
  The live working tree held one more file in both,
  this document.
  The 61 are exactly the files under `package-paused/`,
  `package-deprecated/`,
  and `.out-of-scope/`.
  The incumbent hardcodes those directory names in `walk-files.ts`;
  the draft's `ignored-trees` block reproduces them.
- No file exists in one set and not the other.

### Rust findings

- Incumbent:
  407 (405 `require-rustdoc`,
  2 `max-lines`).
- New,
  `rust/max-lines` and `rust/require-rustdoc`:
  185,
  every one identical to an incumbent finding in message,
  severity,
  and full span.
- Incumbent only:
  222 (220 `require-rustdoc`,
  2 `max-lines`) in 30 files.
  Classification:
  configuration.
  The draft exempts `**/*.fuzz/**/*.rs` (131 `require-rustdoc`,
  1 `max-lines`) and `doc/audit/**/*.rs` (89 and 1),
  which the incumbent's built-in defaults do not.
- Control:
  with those two patterns removed from the configuration,
  the new executable reports 407 findings identical to the incumbent's.
- New only,
  `rust/no-anonymous-functions`:
  1215.
  Classification:
  a rule with no incumbent.

Identifier and shape differences,
all intended:
`rust/max-lines` for `builtin(max-lines)`,
`rust/require-rustdoc` for `builtin(require-rustdoc)`,
and no leading `./` on file names.

### Markdown findings

- Incumbent:
  51644.
- New,
  `host-only`:
  52249.
- Identical:
  51635.
- Different:
  623 (9 incumbent only,
  614 new only),
  all in 9 files,
  and every one located after the first astral character of its file.
  The rules are `semantic-line-breaks` (9 and 413) and `no-bare-urls` (201).

Classification:
the incumbent's astral offset defects,
in two parts,
each measured.

- Issue #559,
  the second offset correction.
  A scratch copy of the incumbent with only that correction removed from `parse.ts`
  reports 4855 findings on the 9 files,
  the same number as the new executable,
  where the shipped incumbent reports 4250.
  That accounts for 607 of the 623.
- Column unit.
  The remaining 16 are the same positions printed in different units,
  as recorded under `Markdown columns count UTF-16 code units`.
  After converting the new executable's columns on those files to code points,
  all 4855 findings match the corrected incumbent exactly.
  On the two lines inspected by hand,
  the new executable's column is the position just after the break-point character,
  and the incumbent's is the break-point character itself.

No difference remains unclassified.
`markdown/lfs-image-url` reported nothing in either linter,
so `LFS rule control` supplies a case where it must.

### Fixes

Fixes were applied in four disposable worktrees of commit `8da338a3e`,
never in the repository.

- Shipped incumbents (`markdown-lint --fix`,
  then `rust-linter --fix`):
  163 files changed.
  The Rust incumbent changed no file.
- Incumbent with the #559 correction removed:
  165 files changed.
- New executable,
  `host-only`:
  165 files changed,
  each converging in one pass.
  A second run rewrote no file and printed identical findings.
- The new executable's tree is byte-identical to the corrected incumbent's except for one file.
  Against the shipped incumbent it differs in 10 files:
  the 9 with astral characters,
  and the same one file.
- Remaining Markdown findings after fixing are the same 5 `heading-increment` findings in all three.

The one file is `package/linter/monochromatic-lint.fuzz/seed/markdown/boundaries.md`,
a fuzz seed whose heading is `# Done\.`.
The incumbent removes the period and leaves `# Done\`;
the new executable removes the complete escape and leaves `# Done`.
Classification:
an intended fix-quality difference in `markdown/no-trailing-punctuation`,
stated in the crate's `README.md` before this work (punctuation edits consume complete escapes).
In the same file the shipped incumbent also leaves a stray `[`
where it deletes an unused definition after an astral character,
which is issue #559 again.

### Processor findings under the draft configuration

`draft` reports 76312 findings where `host-only` reports 53649.
Every `host-only` finding is also in the `draft` output.
The 22663 additional findings come from virtual files.

- Rust hosts,
  from rustdoc and doc tests:
  21196 `markdown/semantic-line-breaks` in 692 files,
  6 `markdown/fenced-code-language`,
  1 `rust/no-anonymous-functions`,
  and 5 `rust/require-rustdoc` warnings.
- Markdown hosts,
  from fenced Rust and its rustdoc:
  1137 `rust/require-rustdoc` warnings,
  206 `rust/no-anonymous-functions` errors,
  and 112 `markdown/semantic-line-breaks`.

Neither run produced a `core/processing-failure` or `core/fix-refused` finding,
and no fix was refused projection.

`--fix` under `draft`,
in its own disposable worktree,
rewrote 878 files (693 `.rs`,
185 `.md` or `.mdx`),
each converging in one pass.
In the Rust files 49456 changed lines are comment lines;
the only others are 16 blank lines that the diff pairs between two comment hunks,
removed and added unchanged.
A second run rewrote no file.
What remains afterwards is not fixable:
5 `markdown/heading-increment`,
1422 `rust/no-anonymous-functions`,
185 `rust/require-rustdoc` errors,
and 1142 `rust/require-rustdoc` warnings.

Rustdoc fixes follow the Markdown rule's add-only style:
the continuation line keeps the space that followed the break point,
so it reads `///  word` with two spaces.

### LFS rule control

A Markdown source was linted from standard input under the logical name `package/music-player/lfs-control.md`,
so nothing was written.
It holds a relative image link to an LFS-tracked file,
an object URL with a stale object id,
a current object URL,
a missing target,
and an untracked target,
after a heading with an astral character.

- Both linters report the same two findings at the same positions with the same messages,
  and exit 1.
- Both produce the same fixed image lines.
- Under the logical name `package/ssg/lfs-control.md`,
  which the configuration excludes,
  both report nothing.
- The result is the same in the live working tree,
  where the target is the smudged image and its id comes from the crate's SHA-256,
  and in a worktree checked out with `GIT_LFS_SKIP_SMUDGE=1`,
  where the target is a pointer file.

### Other observed differences

- Output shape.
  The Markdown incumbent prints one JSON array with `path`,
  `line`,
  `column`,
  `ruleId`,
  `message`,
  and `fixable`.
  The new executable prints JSON Lines in the Rust incumbent's shape,
  which has no `fixable` field.
- Rule identifiers.
  `markdown/<name>` replaces `MD001`,
  `MD014`,
  `MD024`,
  `MD025`,
  `MD026`,
  `MD034`,
  `MD036`,
  `MD040`,
  `MD053`,
  `MD054`,
  and the three unprefixed names.
- Exit status.
  All lint runs exited 1 in every linter.
  The status 2 cases under `Decisions open for veto` did not occur on this repository.
- Size limit.
  The Markdown incumbent skips files over 5 MiB.
  The largest tracked Markdown file is 806795 bytes,
  so the comparison does not exercise that difference.
- Time,
  debug build,
  four threads,
  two runs each:
  5.7 and 7.1 seconds for `host-only`,
  88.3 and 102.4 seconds for `draft`,
  and 14.7 seconds for `draft` once the rustdoc findings are fixed.
  The difference between the last two is the work done for each rustdoc finding,
  which includes projecting its fix to the host.
  The Markdown incumbent took 3.0 and 4.0 seconds and the Rust incumbent 0.3 and 0.4.
  A release build was not measured.

### Questions the comparison raises

These are for whoever decides cutover;
none was acted on.

- `rust/no-anonymous-functions` at `error` on `**/*.rs` reports 1215 findings in Rust files today,
  with no incumbent and no burn-down block.
- The same rule reports 206 errors in Markdown snippets and 1 in a doc test.
  The draft's `snippet-burn-down` block lowers only `rust/require-rustdoc` and `rust/max-lines`.
- Snippet warnings number 1142 in 200 host files;
  the design estimated 905.
  710 of them are `Missing rustdoc on file.`
  A snippet that opens with its file-path comment still gets that finding unless a `//!` line comes first.
- The draft's two added exemptions hide 222 findings the Rust incumbent reports today.
- Fixing rustdoc at cutover rewrites 693 Rust files.

## Self-lint of the crate

The comparison's `host-only` output for `package/linter/monochromatic-lint/src`:

- `rust/max-lines`:
  no finding;
  every file is within 300 code lines.
- `rust/require-rustdoc`:
  85 findings in 33 files,
  all in modules that existed before this work,
  and none in the modules added here.
  Nearly all are undocumented `use` items.
- `rust/no-anonymous-functions`:
  20 findings.
  Fourteen are in existing modules and tests.
  Four are in `src/run_workers.rs`,
  at the `catch_unwind`,
  `spawn_scoped`,
  and `thread::scope` calls:
  the standard library takes a callable there,
  and a named function cannot carry the plan and shared state,
  so each closure only forwards to a named function.
  Two are in `src/run_write_tests.rs` for the same reason.

`package/linter/monochromatic-lint/README.md` has no finding under the Markdown incumbent or under `draft`.

## Limitations and remaining work

- A consumer of `--stdin --fix` must not use standard output when the status is 2 from a setup failure:
  nothing is printed there.
  When a fix is refused or processing fails,
  the original source is printed and the status is 2.
- The new executable has no file size limit and no parse-time budget,
  as the design defers one.
- `rust/require-explicit-types` on virtual Rust reports a processing finding,
  as the processor handoff specifies.
  A configuration that selects it for `**/*.rs` must ignore `**/*.md/**` and `**/*.mdx/**` for that rule.
  The draft configuration does not select the rule,
  so the comparison did not load a Cargo workspace.
- The semantic engine runs on the calling thread after the workers finish,
  so files that need it are not linted concurrently.
- Paths that are not valid UTF-8 are linted,
  and their display names use replacement characters.
- `--print-config` prints a relative `--config` path joined to the working directory without removing `..` segments.
- Rust-side and Markdown-side column units were not compared with each other.
- The comparison ran a debug build on one Linux host.
  Windows and a release build were not exercised.
- No new mutation campaign was run over the orchestration modules.
- The fuzz smoke run is 30 seconds per target;
  it is a smoke run,
  not a campaign.

## Gate log

- Baseline before any change:
  203 tests passed and Clippy passed.
  Image `57e310bd9253f1fd6e84437012336fb14c9ecf36fef71091b3fd9fe15beab3f5`.
- Rule port and orchestration:
  332 tests passed and Clippy passed.
  Image `882b3fade84d247266837a4fcb881d9031668cf879ddbb66d33a0416307de40b`.
- Binary-level tests,
  first run:
  failed,
  334 passed and 1 failed.
  The failing test was a new strict-JSON control that parsed a scalar document root,
  which the repository's JSONC parser rejects by design.
  The test was corrected to wrap scalars in an array and to assert the rejection.
- Binary-level tests,
  second run:
  335 library tests and 10 executable tests passed,
  and Clippy passed,
  in 482.78 seconds.
  Image `82de8e9784de52011127aa5ad43cfdf5839779f22e74f74ade0f998e608ae127`.
- Fuzz smoke,
  `mise run //package/linter/monochromatic-lint.fuzz:smoke` at commit `2aab0b324`:
  9 helper tests and Clippy passed in the build container,
  the AddressSanitizer build passed,
  and every target exited 0 after 30 seconds with no crash artifact.
  Executions:
  `merge_values` 100598,
  `configuration` 8671,
  `rust_style` 1052,
  `rust_explicit_types` 425,
  `markdown` 36571,
  and `orchestration` 408.
  The new target added 121 corpus units and peaked at 570 MB resident.
  Its rate,
  13 executions per second,
  is low because every input runs the whole lint and fix path several times;
  408 executions is a smoke result,
  not a campaign.
  Build image `d6b84c70360f9b75d787d27c5473829223626270b8c7ee87847cd98578366f5d`,
  run image `e5fb7a506f2a2b68f9dced09822fc09b6b81e13a2b4fcb92347be093deafddc2`.
  Evidence was written to the ignored `target/verification/campaign-IvsNWV` directory of the fuzz package.
- Final gate at commit `2aab0b324`,
  covering the `--debug` file notes:
  335 library tests and 10 executable tests passed,
  and Clippy passed,
  in 312.28 seconds.
  Image `59eb21a45a09dfd956606f040fb66aec6fa3393f608dbdba4e899cfc7c559173`.

## Commits

- `2cdcea1f3`:
  `feat(linter-monochromatic-lint): port markdown/lfs-image-url with a pending endpoint normalizer`.
- `9adb2f918`:
  `feat(linter-monochromatic-lint): wire rules, processors and the fix loop into an executable`.
- `c46396109`:
  `feat(linter-monochromatic-lint): implement the selected LFS endpoint normalizer and orchestration controls`.
- `6b260a81e`:
  `test(linter-monochromatic-lint): run the built executable in disposable fixtures and print strict JSON configuration`.
- `4b6d46b9a`:
  `test(linter-monochromatic-lint.fuzz): fuzz the per-source orchestration path with nested doc tests and adversarial edit groups`.
- `2aab0b324`:
  `feat(linter-monochromatic-lint): name every selected file in --debug output`.
