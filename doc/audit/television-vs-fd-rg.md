# Television versus fd plus rg audit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/watch-restart` is now `package/watch-restart/cli`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Generated 2026-06-15.

This audit compares [`alexpasmantier/television`](https://github.com/alexpasmantier/television)
with keeping [`sharkdp/fd`](https://github.com/sharkdp/fd)
plus [`BurntSushi/ripgrep`](https://github.com/BurntSushi/ripgrep)
as the repo's file and text search primitives.
It also checks serious adjacent fuzzy-picker alternatives,
because a recommendation that only compares the named tool against incumbents can miss the better third option.

## Recommendation

Keep `fd` plus `rg` as the canonical search primitives.
Do not add Television as a committed repo dependency until its full test validation passes locally
and its shell-template quoting is fixed or avoided.
Treat Television as an optional personal TUI experiment that composes those tools,
not as a replacement for either one.

The decisive fact is that Television's own built-in Unix `files` channel runs `fd -t f`,
and its built-in Unix `text` channel runs `rg . --no-heading --line-number ...`.
Those source definitions live in the audited clone at
`/tmp/agent/television-20260615/cable/unix/files.toml` and
`/tmp/agent/television-20260615/cable/unix/text.toml`.
So adopting Television for file and text search does not remove `fd` or `rg`.
It adds a picker,
 previews,
 shell actions,
 history,
 frecency,
and a Rust dependency graph measured by `cargo metadata --locked` at 40 direct dependencies and 369 packages total.

Use Television only when the wanted user boundary is interactive selection,
and only after accepting the validation and shell-template caveats below.
For scripts,
 agent workflows,
 package tasks,
 and deterministic file or content enumeration,
call `fd` or `rg` directly.

If the repo adopts Television,
write a separate decision record under `doc/decision/search-ui.md` after that pick.
No decision document is written by this audit because no pick was requested.

## Scope and candidate set

The request named Television,
 `fd`,
 and `rg`.
All are open-source CLI tools,
 so SaaS vendor vetting does not apply.
The open-source default is satisfied by every finalist:
Television is MIT licensed,
 `fd` is MIT or Apache-2.0,
and ripgrep is Unlicense or MIT,
 per each cloned `Cargo.toml`.

Audited clone commits:

- Television:
   `/tmp/agent/television-20260615`,
  [`b9ff691`](https://github.com/alexpasmantier/television/tree/b9ff691572c3c05fa69171f62d5b31692e9d846e).
- fd:
   `/tmp/agent/fd-20260615`,
  [`25461e5`](https://github.com/sharkdp/fd/tree/25461e5ce13dc12ff2a75993285a87e99b33db2d).
- ripgrep:
   `/tmp/agent/ripgrep-20260615`,
  [`82313cf`](https://github.com/BurntSushi/ripgrep/tree/82313cf95849bfe425109ad9506a52154879b1b1).
- fzf:
   `/tmp/agent/fzf-20260615`,
  [`3c9965a`](https://github.com/junegunn/fzf/tree/3c9965a61a842ef54e976c7195b985ee43a3e776).
- skim:
   `/tmp/agent/skim-20260615`,
  [`b5b11a4`](https://github.com/skim-rs/skim/tree/b5b11a472f4788b52734c3c1f1416a4de4722195).
- fzy:
   `/tmp/agent/fzy-20260615`,
  [`34b8886`](https://github.com/jhawthorn/fzy/tree/34b88869d022e861da4846c4463aea3ddfb3ff30).

Serious alternatives surveyed:

- `junegunn/fzf`,
   cloned to `/tmp/agent/fzf-20260615` from
  [`junegunn/fzf`](https://github.com/junegunn/fzf).
  Rejected as an `fd` plus `rg` replacement because it is a fuzzy picker.
  It can consume command output and has a built-in file walker,
  but it does not provide ripgrep's text-search engine or fd's exact filesystem filtering surface.
  It is a strong alternative to Television when the only requirement is a mature picker.
- `skim-rs/skim`,
   cloned to `/tmp/agent/skim-20260615` from
  [`skim-rs/skim`](https://github.com/skim-rs/skim).
  Rejected as an `fd` plus `rg` replacement for the same boundary reason,
  and because local validation found interactive ANSI tests failing in this environment.
- `jhawthorn/fzy`,
   cloned to `/tmp/agent/fzy-20260615` from
  [`jhawthorn/fzy`](https://github.com/jhawthorn/fzy).
  Rejected because it is a focused 4,721-line C fuzzy selector without Television's channel system,
  preview/action surface,
   or fd/ripgrep replacement semantics.
  Maintenance signals are also materially weaker than the other picker candidates.
- `Genivia/ugrep` appeared in GitHub search as a search alternative.
  It is a content grep alternative,
   not a fuzzy picker and not a direct `fd` replacement,
  so it was not a finalist for this comparison.
- `npm search 'fuzzy finder cli terminal'` returned JavaScript fuzzy libraries and wrappers
  such as `fuzzy`,
   `fast-fuzzy`,
   `@ff-labs/fff-node`,
   `@ff-labs/fff-bun`,
   and `fzf`.
  They are not ready replacements for a terminal file picker plus `fd` plus `rg` workflow.

## Current repo usage

The repo already treats `fd` and `rg` as baseline tools:

- Root `mise.toml` configures `"cargo:fd-find" = "latest"` with the comment
  `aqua:sharkdp/fd broken as of 2026-03-07, falling back to cargo`.
- Root `mise.toml` configures `ripgrep = "latest"`.
- `mise.lock` pins ripgrep `15.1.0` artifacts by platform with SHA-256 checksums.
- `package/dev-script/file-enforcer/data/packages.overrides.ts` maps the `rg` binary
  to the `ripgrep` package name.
- `package/dev-script/watch-restart/src/filters/hidden.ts` explicitly cites `fd` and `rg`
  ergonomics for default hidden-file exclusion.

No repo file matched `television` before this audit.
That means Television would be a new tool,
while `fd` and `rg` are already part of the local tool contract.

## Source audit: Television

Cloned source path:
 `/tmp/agent/television-20260615`.
Upstream metadata from `gh repo view alexpasmantier/television`:
MIT,
 Rust,
 6,001 stars,
 178 forks,
 latest release `0.15.9` published 2026-06-14.

Production paths inspected:

- `television/main.rs` parses CLI arguments,
   loads configuration and cable channels,
  builds `App`,
   and writes selected entries to stdout.
- `television/app.rs` owns the render loop,
   event loop,
   watch timer,
   history/frecency persistence,
  and external action dispatch.
- `television/television.rs` builds the picker,
   previewer,
   remote control,
   action picker,
  and `CableChannel` from merged channel config.
- `television/channels/channel.rs` is the key integration boundary.
  `load_candidates` runs the configured source command through `shell_command`,
  reads stdout in batches,
   and injects entries into the Nucleo matcher.
- `television/previewer/mod.rs` formats the preview command,
   executes it through the shell,
  caches preview text when configured,
   and aborts jobs that exceed the default timeout.
- `television/utils/command.rs` formats and executes external actions.
- `cable/unix/files.toml` and `cable/unix/text.toml` show the built-in file and text channels
  depending on `fd`,
   `rg`,
   and `bat`.

Important behavior from source:

- Television is a generic shell-command-backed picker.
  File enumeration and text search are delegated to configured source commands.
- The default file channel requires `fd` and `bat`,
   then runs `fd -t f` or `fd -t f -H`.
- The default text channel requires `rg` and `bat`,
   then runs `rg . --no-heading --line-number ...`.
- Preview and action commands are shell strings.
  This is powerful for user customization and riskier than direct argv-based execution.

### Television security finding

Television's command template quoting is not safe for POSIX shell single quotes.
`television/utils/command.rs` formats simple `{}` action templates by wrapping entries in single quotes
and replacing `'` with `\'`.
Inside POSIX single quotes,
 `\'` does not escape a single quote.
The unit test in the same file expects `nvim 'file\'s name.txt'`,
which is not a safe POSIX shell token.

I reproduced this through the release `tv` binary,
not only through a reimplementation of the quoting rule.
The probe created a temporary cable channel whose source emitted
`x'; printf TV_INJECTION_MARKER; echo z #`,
bound `enter` to an action with `command = "printf '<%s>\\n' {}"`,
ran `tv --cable-dir <tmp>/cable files` under a Python pseudo-terminal,
and sent Enter after the candidate was rendered.
The process exited successfully and printed:

```text
<x\>
TV_INJECTION_MARKERz
```

That end-to-end output proves a selected entry containing a single quote can break out
of the intended quoted argument in an actual Television external action.
For repo-local personal use this may only mean broken previews or actions on odd filenames.
For untrusted checkouts,
 shared directories,
 or generated filenames,
it is a command-injection class boundary bug.

`fd` does better at its comparable boundary:
`/tmp/agent/fd-20260615/src/exec/mod.rs` builds `std::process::Command` argv values,
adds path arguments directly,
and does not shell-interpolate filenames for `--exec` or `--exec-batch`.
`fzf` also has a correct POSIX quoting primitive in
`/tmp/agent/fzf-20260615/src/util/util_unix.go`,
where `QuoteEntry` replaces `'` with `'\''`.

### Television tests and validation

Source evidence:

- `.github/workflows/ci.yml` runs `TV_CI=1 cargo test --locked --all-features --workspace`,
  `cargo fmt --all --check`,
   and `cargo clippy -- -D warnings`.
- The test tree includes CLI tests under `tests/cli/`,
  including preview and external action coverage.
- `benches/main.rs` wires Criterion benchmarks for UI,
   candidate loading,
   previewer,
   render,
  strings,
   and update paths.
- I found no cargo-fuzz,
   libFuzzer,
   AFL,
   proptest,
   quickcheck,
   fast-check,
  or mutation-testing harness in the cloned tree.

Local validation:

- `cargo build --locked --release` succeeded in `/tmp/agent/television-20260615`.
- `./target/release/tv --source-command "printf 'alpha\\nbeta\\n'" --take-1` returned `alpha`.
- `./target/release/tv files --input Cargo.toml --take-1` returned `Cargo.toml`,
  exercising the built-in file channel through `fd`.
- `TV_CI=1 CARGO_BUILD_JOBS=2 cargo test --locked --all-features --workspace -- --nocapture --test-threads=4`
  failed before tests ran because the dev dependency path
  `television -> phantom-test -> phantom-daemon -> libghostty-vt -> libghostty-vt-sys`
  fetched Ghostty and Zig failed with `FileNotFound` while running `uucode_build_tables`.
  `cargo tree --locked --invert libghostty-vt-sys` confirmed the path is dev-only through `phantom-test`.

Interpretation:
Television's release binary is buildable here,
but its full upstream test validation is not reproducible in this environment
without resolving the Ghostty/Zig dev-dependency build.
That disqualifies Television as a recommended committed repo dependency for now.
It remains suitable only as a personal experiment or future candidate after full validation succeeds.

## Source audit: fd

Cloned source path:
 `/tmp/agent/fd-20260615`.
Upstream metadata from `gh repo view sharkdp/fd`:
MIT or Apache-2.0,
 Rust,
 43,341 stars,
 1,067 forks,
latest release `v10.4.2` published 2026-03-10.

Production paths inspected:

- `src/main.rs` parses CLI options,
   builds pattern regexes,
  constructs search config,
   then calls `walk::scan`.
- `src/walk.rs` builds an `ignore::WalkBuilder` with hidden-file,
   `.fdignore`,
   `.gitignore`,
  parent ignore,
   global ignore,
   symlink,
   filesystem-boundary,
   depth,
   and thread settings.
  Worker threads filter path,
   type,
   extension,
   size,
   time,
   owner,
   and result limits.
- `src/exec/mod.rs` and `src/exec/job.rs` implement `--exec` and `--exec-batch`
  by constructing `Command` values and passing generated path arguments as argv,
  not as shell text.

Tests and CI:

- `.github/workflows/CICD.yml` runs rustfmt,
   clippy,
   MSRV tests,
  matrix builds across Linux,
   macOS,
   Windows,
   and cross targets,
  and release packaging.
- `tests/tests.rs` is a broad integration suite covering matching,
   globbing,
   hidden files,
  `.fdignore`,
   `.gitignore`,
   symlinks,
   depth,
   file types,
   execution,
   invalid UTF-8,
  size/time filters,
   owner filters,
   hyperlinks,
   and result limits.
- I found no fuzzing or mutation-testing harness.
- The release workflow uses SHA-pinned `actions/upload-artifact` and `actions/attest`,
  so release archives and Debian packages are attested on tagged releases.

Local validation:

- `CARGO_BUILD_JOBS=2 cargo test --locked --all-features` passed in `/tmp/agent/fd-20260615`.
- `/tmp/agent/fd-20260615/target/debug/fd --hidden --type f txt <fixture>` returned the expected fixture file.

Interpretation:
`fd` is the stronger primitive for deterministic path enumeration.
It has narrower scope than Television,
but the source boundary is cleaner and the validation path passed locally.

## Source audit: ripgrep

Cloned source path:
 `/tmp/agent/ripgrep-20260615`.
Upstream metadata from `gh repo view BurntSushi/ripgrep`:
Unlicense or MIT,
 Rust,
 65,063 stars,
 2,594 forks,
latest release `15.1.0` published 2025-10-22.

Production paths inspected:

- `crates/core/main.rs` dispatches search,
   file-list,
   type-list,
   completion,
  and generation modes.
  It chooses sequential or parallel search based on thread count.
- `crates/core/search.rs` builds search workers,
  selects Rust regex or PCRE2 matchers,
  handles binary detection,
  optional preprocessors,
  decompression,
  standard,
   summary,
   and JSON printers.
- `crates/core/main.rs` uses `args.walk_builder()` for recursive traversal;
  ripgrep shares the `ignore` crate family that powers gitignore-aware walking.
- `crates/core/flags/` implements the CLI flag surface;
  `crates/core/flags/defs.rs` is 7,779 lines in the audited clone.

Tests and CI:

- `.github/workflows/ci.yml` runs build and test matrices across pinned,
   stable,
   beta,
   nightly,
  musl,
   cross targets,
   macOS,
   Windows MSVC,
   Windows GNU,
   and Windows ARM.
- The CI builds with and without PCRE2,
  runs shell completion tests,
  checks docs with `RUSTDOCFLAGS=-D warnings`,
  and has a `fuzz_testing` job that compiles fuzz targets.
- `fuzz/Cargo.toml` is a `cargo-fuzz` package.
  `fuzz/fuzz_targets/fuzz_glob.rs` checks `Glob::new` and `Glob::from_str` consistency
  plus round-tripping of the glob text.
- Integration tests under `tests/` cover binary data,
   JSON output,
   multiline search,
  feature combinations,
   regression cases,
   and fixture compression formats.
- I found fuzzing evidence for glob parsing,
  but no mutation-testing harness.

Local validation:

- `CARGO_BUILD_JOBS=2 cargo test --locked --workspace --features pcre2` passed
  in `/tmp/agent/ripgrep-20260615`.
- `/tmp/agent/ripgrep-20260615/target/debug/rg needle <fixture>` returned the expected file matches.

Interpretation:
`rg` is the stronger primitive for text search.
Television's text channel depends on it rather than replacing it.

## Alternative source audits

### fzf

Cloned source path:
 `/tmp/agent/fzf-20260615`.
Upstream metadata:
MIT,
 Go,
 80,961 stars,
 2,793 forks,
latest release `v0.73.1` published 2026-05-25.

Production paths inspected:

- `main.go` parses options and calls `fzf.Run`.
- `src/core.go` wires the reader,
   matcher,
   terminal,
   event coordination,
  filter mode,
   and reload behavior.
- `src/reader.go` reads stdin,
   a command,
   or a built-in file walker.
- `src/matcher.go` parallelizes matching over chunks and maintains matcher caches.
- `src/terminal.go` handles preview and command execution.
- `src/util/util_unix.go` implements POSIX entry quoting with `QuoteEntry`.

Tests and CI:

- `.github/workflows/linux.yml` sets up Go `1.23`,
   Ruby `3.4.6`,
   tmux,
   shell tooling,
  runs lint,
   unit tests,
   Go fuzz tests for `FuzzIndexByteTwo` and `FuzzLastIndexByteTwo`,
  then runs Ruby integration tests in tmux.
- `test/test_preview.rb` contains extensive preview behavior tests.
- `.goreleaser.yml` builds archives across Darwin,
   Linux,
   Windows,
   FreeBSD,
   OpenBSD,
   and Android.

Local validation:

- `GO=/home/user/.local/share/mise/installs/go/1.26.4/bin/go make test` passed.
  This is not an exact CI reproduction because fzf CI pins Go `1.23`.
- `make itest` could not run because local Ruby is not installed.
  This means fzf was not fully validated against its upstream integration suite here.
- `make all` built `target/fzf-linux_amd64`.
- `printf 'alpha\nbeta\ngamma\n' | ./target/fzf-linux_amd64 --filter=alp` returned `alpha`.

Interpretation:
For a pure picker,
 fzf is the mature alternative.
It is not a replacement for `fd` plus `rg`.

### skim

Cloned source path:
 `/tmp/agent/skim-20260615`.
Upstream metadata:
MIT,
 Rust,
 6,848 stars,
 251 forks,
latest release `v4.7.0` published 2026-05-23.

Production paths inspected:

- `src/bin/main.rs` is the `sk` CLI entry point.
- `src/skim.rs` wires the application,
   reader,
   matcher,
   TUI,
   and remote listener.
- `src/reader.rs` collects items from stdin or commands.
- `src/matcher.rs` coordinates fuzzy,
   exact,
   regex,
   split-match,
   and normalized matching engines.
- `src/tui/preview.rs` runs preview commands,
   with optional PTY support.
- `tests/preview.rs` snapshot-tests preview modes,
   offsets,
   wrapping,
   and PTY behavior.

Tests and CI:

- `.github/workflows/test.yml` runs doctests,
   nextest,
   `cargo llvm-cov` coverage output,
  clippy,
   rustfmt,
   MSRV verification,
   and no-default-features build.
- `.github/workflows/release.yml` uses cargo-dist and depends on the custom test workflow.
- I found no fuzzing or mutation-testing harness.

Local validation:

- `cargo test --locked --workspace` compiled and passed 233 unit tests,
  then failed four ANSI integration tests in `tests/ansi.rs` because the test harness timed out
  waiting for a prompt capture.
- `./target/debug/sk --version` returned `sk 4.7.0`.
- `printf 'alpha\nbeta\ngamma\n' | ./target/debug/sk --filter=alp` returned `alpha`.

Interpretation:
Skim is an active Rust picker,
but it is not an `fd` plus `rg` replacement and did not fully validate locally.

### fzy

Cloned source path:
 `/tmp/agent/fzy-20260615`.
Upstream metadata:
MIT,
 C,
 3,258 stars,
 145 forks,
latest release `v1.1` published 2025-07-12.

Production paths inspected:

- `src/fzy.c` is the CLI entry point.
- `src/match.c` implements the fuzzy scoring and positions algorithm.
- `src/tty_interface.c` implements the interactive terminal loop.

Tests and CI:

- `.github/workflows/test.yml` runs GCC and Clang on Ubuntu and macOS,
  Alpine builds,
   multi-architecture tests,
   and Ruby acceptance tests.
- `test/test_properties.c`,
   `test/test_match.c`,
   and `test/test_choices.c` cover core behavior.
- I found no fuzzing,
   mutation-testing,
   or coverage workflow.

Local validation:

- `make test` passed.
- `make` built `./fzy`.
- `printf 'alpha\nbeta\ngamma\n' | ./fzy --show-matches=alp` returned `alpha`.

Interpretation:
fzy is simple and locally verifiable,
but it lacks Television's preview/action/channel system and does not replace `fd` or `rg`.

## Maintenance signals

These counts come from GitHub API sampling via `/tmp/agent/gh_maintenance.py`,
covering issues and pull requests updated since 2025-06-15.
The point is not raw open issue count.
The maintainer-specific signals are owner/collaborator comments,
 reviews,
 authored pull requests,
and stale external pull requests without maintainer response.
The script also recorded generic timeline events such as labels,
 closes,
 references,
 and cross-references;
those events are not counted as maintainer-authored actions.

### Television

- 194 non-PR issues updated since 2025-06-15.
- In the 20 sampled issues,
   6 had owner comments and 16 had generic timeline events
  such as labels,
   closes,
   references,
   or cross-references.
- In the 20 sampled pull requests,
   1 was owner-authored,
  2 had owner reviews,
   and 5 had owner comments.
- 12 sampled external open pull requests were older than 30 days without maintainer review or comment.
- Five latest releases in the sample were `0.15.5` to `0.15.9`,
  published between 2026-04-08 and 2026-06-14.

State:
 active releases with weak public issue and PR support.

### fd

- 118 non-PR issues updated since 2025-06-15.
- In the 20 sampled issues,
   12 had collaborator comments and all 20 had generic timeline events.
- In the 20 sampled pull requests,
   5 were collaborator-authored,
  4 had collaborator reviews,
   and 5 had maintainer comments.
- No sampled external open pull request older than 30 days lacked maintainer review or comment.
- Recent releases include `v10.3.0` in 2025-08 and `v10.4.0` to `v10.4.2` in 2026-03.

State:
 responsive maintainers with an active,
 triaged backlog.

### ripgrep

- 196 non-PR issues updated since 2025-06-15.
- In the 20 sampled issues,
   6 had owner comments and 18 had generic timeline events.
- In the 20 sampled pull requests,
   3 were owner-authored,
  2 had maintainer reviews,
   and 2 had maintainer comments.
- One sampled external open pull request was older than 30 days without maintainer review or comment.
- Recent releases include `15.0.0` and `15.1.0` in 2025-10.

State:
 active,
 owner-driven project with strong release and CI discipline.
Public issue response is thinner than fd but not abandoned.

### fzf

- 294 non-PR issues updated since 2025-06-15.
- In the 20 sampled issues,
   12 had owner or collaborator comments and 11 had generic timeline events.
- In the 20 sampled pull requests,
   7 were maintainer-authored,
  7 had maintainer reviews,
   and 4 had maintainer comments.
- One sampled external open pull request was older than 30 days without maintainer review or comment.
- Five sampled releases from `v0.70.0` to `v0.73.1` were published between 2026-03 and 2026-05.

State:
 actively maintained mature picker.

### skim

- 192 non-PR issues updated since 2025-06-15.
- In the 20 sampled issues,
   the API sample showed no owner or collaborator comments,
  but 19 had generic timeline events.
- In the 20 sampled pull requests,
   the API sample showed no maintainer-authored PRs,
  no maintainer reviews,
   and no maintainer comments by GitHub association.
- One sampled external open pull request was older than 30 days without maintainer review or comment.
- Five sampled releases from `v4.6.0` to `v4.7.0` were published between 2026-04 and 2026-05.

State:
 active releases with weak public maintainer-response signals.

### fzy

- 11 non-PR issues updated since 2025-06-15.
- In the 11 sampled issues,
   2 had owner comments and 8 had generic timeline events.
- In the 20 sampled pull requests,
   none were maintainer-authored,
  none had maintainer reviews,
   and 6 had maintainer comments.
- 13 sampled external open pull requests were older than 30 days without maintainer review or comment.
- The latest release is `v1.1`,
   published 2025-07-12;
  the previous release was in 2018.

State:
 low-activity project with stale backlog shape.

## Limited build provenance and supply-chain notes

This is not a full transitive dependency provenance audit.
I did not audit every native build script,
 compiler flag,
 host imported function,
source archive,
 or vendored dependency in these projects.
The notes below cover the release workflows and native boundaries visible during this tooling audit.

`fd` and `rg` have the cleaner provenance story for repo tooling:

- `fd` release CI builds across a broad platform matrix,
  uploads archives,
  and uses GitHub artifact attestation in `.github/workflows/CICD.yml`.
- `rg` release CI builds with PCRE2,
   generates man pages and completions from the built binary,
  uploads archives with SHA-256 files,
   and builds a Debian package.
- This repo's `mise.lock` pins ripgrep release artifacts with SHA-256 checksums.
- Television release CI builds archives and `.sha256` files,
  but the inspected `.github/workflows/cd.yml` does not include an artifact attestation step.
- Television's full test path pulled a dev-only native/Zig dependency chain through `phantom-test`.
  That did not affect the release build here,
  but it makes test validation heavier and more brittle than `fd` or `rg`.

## Capability comparison

### Deterministic file enumeration

Pick `fd`.

`fd` implements the search semantics directly:
ignore files,
 hidden files,
 symlinks,
 depth,
 type filters,
 extension filters,
size/time/owner filters,
 execution,
 and result limits.
Television delegates the default file channel to `fd -t f`.
Using Television for this boundary adds a shell command,
 a TUI,
 and a matcher around `fd`.
It does not simplify or harden the underlying file search.

### Deterministic text search

Pick `rg`.

Ripgrep implements the text search engine directly:
regex engine selection,
 PCRE2 feature mode,
 binary detection,
 decompression,
preprocessors,
 parallel walking,
 and multiple printer formats.
Television delegates the default text channel to `rg . --no-heading --line-number ...`.
Using Television for this boundary adds interactive filtering over ripgrep output.
It does not replace the text-search engine.

### Human interactive selection

Television remains a future candidate if the desired interaction is channel-based selection with previews and actions,
after the validation and shell-template findings are resolved.
It has stronger built-in domain structure than fzf for this exact use:
channel TOML files,
 metadata,
 requirements,
 multiple source variants,
 previews,
action keybindings,
 remote control,
 history,
 and frecency.

fzf is the better picker if maturity,
 test coverage,
 and shell-quoting hygiene are weighted above channel structure.
Skim is another Rust picker,
but local validation did not complete cleanly.
fzy lacks the Television feature set.

### Security boundary

Prefer `fd` and `rg` for automated or untrusted inputs.

`fd --exec` builds argv directly.
Ripgrep search paths and patterns are parsed by its CLI and regex engine,
not by a shell action template.
Television preview and action commands are shell strings built from selected entries.
The audited simple-brace action formatting is POSIX-unsafe for filenames containing single quotes.

Before Television is adopted,
require these mitigations before using it on untrusted trees or shared data:

- Avoid shell actions and previews that interpolate raw entries.
- Prefer direct `fd` or `rg` in automation.
- Add adversarial tests for filenames containing single quotes,
   newlines,
   semicolons,
  command substitutions,
   leading dashes,
   terminal escapes,
   and path traversal-looking segments.
- Open or track an upstream issue for POSIX shell quoting in action and preview templates.

## Validation commands run

```sh
# Metadata and maintenance
mkdir --parents /tmp/agent && chmod 700 /tmp/agent
gh repo view alexpasmantier/television --json ...
gh repo view sharkdp/fd --json ...
gh repo view BurntSushi/ripgrep --json ...
gh repo view junegunn/fzf --json ...
gh repo view skim-rs/skim --json ...
gh repo view jhawthorn/fzy --json ...
python3 /tmp/agent/gh_maintenance.py > /tmp/agent/gh_maintenance.json

# Clones
gh repo clone alexpasmantier/television /tmp/agent/television-20260615 -- --depth 1
gh repo clone sharkdp/fd /tmp/agent/fd-20260615 -- --depth 1
gh repo clone BurntSushi/ripgrep /tmp/agent/ripgrep-20260615 -- --depth 1
gh repo clone junegunn/fzf /tmp/agent/fzf-20260615 -- --depth 1
gh repo clone skim-rs/skim /tmp/agent/skim-20260615 -- --depth 1
gh repo clone jhawthorn/fzy /tmp/agent/fzy-20260615 -- --depth 1

# Television
cd /tmp/agent/television-20260615
cargo build --locked --release
./target/release/tv --source-command "printf 'alpha\\nbeta\\n'" --take-1
./target/release/tv files --input Cargo.toml --take-1
TV_CI=1 CARGO_BUILD_JOBS=2 cargo test --locked --all-features --workspace -- --nocapture --test-threads=4
cargo tree --locked --invert libghostty-vt-sys
python3 <pty script that drives a temporary malicious-entry cable action through target/release/tv>

# fd
cd /tmp/agent/fd-20260615
CARGO_BUILD_JOBS=2 cargo test --locked --all-features
/tmp/agent/fd-20260615/target/debug/fd --hidden --type f txt <fixture>

# ripgrep
cd /tmp/agent/ripgrep-20260615
CARGO_BUILD_JOBS=2 cargo test --locked --workspace --features pcre2
/tmp/agent/ripgrep-20260615/target/debug/rg needle <fixture>

# fzf
cd /tmp/agent/fzf-20260615
GO=/home/user/.local/share/mise/installs/go/1.26.4/bin/go make test
GO=/home/user/.local/share/mise/installs/go/1.26.4/bin/go make all
printf 'alpha\nbeta\ngamma\n' | ./target/fzf-linux_amd64 --filter=alp
make itest

# skim
cd /tmp/agent/skim-20260615
cargo test --locked --workspace
./target/debug/sk --version
printf 'alpha\nbeta\ngamma\n' | ./target/debug/sk --filter=alp

# fzy
cd /tmp/agent/fzy-20260615
make test
make
printf 'alpha\nbeta\ngamma\n' | ./fzy --show-matches=alp
```

`make itest` for fzf failed because Ruby is absent locally.
The fzf CI workflow runs Ruby integration tests under Ruby 3.4.6,
so this is an environment gap in my local validation,
 not evidence that upstream integration tests fail.
The passing fzf unit-test run used Go 1.26.4 because that is the local Go installed by mise;
fzf CI pins Go 1.23.

`cargo test --locked --workspace` for skim failed four ANSI integration tests due prompt-capture timeouts.
The same command had already passed 233 unit tests before the ANSI integration failures.

## Bottom line

Television is not a competitor to `fd` plus `rg` for the repo's core search primitive role.
It is an optional UI that uses them.

The safe architecture is:

- `fd` for paths.
- `rg` for text.
- `television` or `fzf` only where a human needs an interactive chooser.

Between Television and fzf as the optional chooser,
Television has better built-in channel structure for file/text/git/process workflows,
while fzf has stronger maturity signals and safer shell quoting.
Do not remove `fd` or `rg` in either case.
