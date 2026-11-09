# monochromatic-lint

EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE

Rust replacement for the repository's Rust and Markdown/MDX linters.
The `monochromatic-lint` executable is verified in containers
and published to crates.io with prebuilt release binaries,
so `cargo binstall monochromatic-lint` installs it without compiling.
In the Monochromatic repository it replaces `package/linter/rust` and `package/cli/markdown-lint`;
the switch of every repository task and the Git wrapper's Markdown policy is recorded in
`doc/handover/unified-linter-cutover.md`.

It also lints the shell scripts in the steps of GitHub Actions workflows and composite actions;
see "Workflow shell rules".

The crate's Rust visibility supports its executable and verification drivers.
There is no supported public linter library interface.

Design:
`doc/planning/unified-linter.md`.
Execution:
`doc/handover/unified-linter.md`.
Executable evidence,
open decisions,
and the comparison with both earlier linters:
`doc/handover/unified-linter-executable.md`.

## Command line

```sh
monochromatic-lint [OPTIONS] [PATH]...
```

With no path the working directory is walked,
honoring `.gitignore`.
Files ending in `.rs`,
`.md`,
and `.mdx` are linted;
other files are skipped.
`mise run //package/linter/monochromatic-lint:run -- <arguments>` builds and runs the executable from source.

Findings are JSON Lines on standard output,
one record per finding,
with the rule identifier in `code`.
A clean run prints nothing.
Messages that are not findings go to standard error with a `monochromatic-lint:` prefix.
A configuration rejection is one prefixed line,
and it ends with `(monochromatic-lint <version>)`:
the running binary's version,
the same one `--version` prints.
When the nearest `mise.lock` at or above the rejected configuration pins `cargo:monochromatic-lint`
to a version newer than the running one,
the line adds that the resolved binary may be stale,
quoting the measured pin.

Exit statuses:

- `0`:
  no error finding,
  and the warning count is within `--max-warnings`.
- `1`:
  at least one error finding,
  or more warnings than `--max-warnings` allows.
- `2`:
  the run could not be trusted to be complete:
  a usage error,
  an unreadable or invalid configuration,
  no configuration for any input file,
  or a `core/processing-failure` or `core/fix-refused` finding.

Options:

- `--config <FILE>` uses exactly that configuration for every file,
  with patterns relative to the working directory.
- `--fix` applies the fixes that change only whitespace,
  in at most ten passes per file,
  and rechecks the result.
  A fix that rewrites content waits for `--fix-content`;
  see "Fix classes".
- `--fix-content` also applies the fixes that rewrite content,
  and fixes without `--fix`.
  A file is replaced atomically and keeps its mode.
- `--stdin` with `--stdin-filename <FILE>` lints standard input as that file.
  With `--fix` the fixed source goes to standard output and findings go to standard error.
- `--max-warnings <COUNT>`,
  `--quiet`,
  and `--silent` change reporting and the warning threshold,
  never what is linted.
- `--print-config <FILE>` prints the effective rules for one file as strict JSON.
- `--init` writes a starter configuration and refuses to overwrite one.
- `--rules` lists every shipped rule as JSON Lines.
- `--concurrency <COUNT>` bounds worker threads.
  Every file is linted on a thread with an 8 MiB stack,
  including a single file,
  `--concurrency 1` and standard input,
  so how deeply nested a file may be does not depend on the platform's main-thread stack.
- `--ignore-pattern <GLOB>`,
  `--ignore-path <FILE>`,
  and `--no-ignore` change which files the walk finds.
- `--no-error-on-unmatched-pattern` accepts a path argument that matches nothing.
- `--debug` adds execution notes on standard error,
  including every file selected or ignored.
- `--report-unused-disable-directives` reports inline directives that suppressed nothing as warnings,
  and `--report-unused-disable-directives-severity <SEVERITY>` sets their severity;
  see "Inline directives".

## Configuration

Each file is governed by the nearest `monochromatic-lint.config.jsonc` in its directory or an ancestor,
unless `--config` names one.
A configuration is an ordered JSONC array of blocks with `files`,
`ignores`,
and `rules`.
Every block whose patterns select a file contributes its rule settings,
merged in order.

The merge is handwritten for ordered JSONC rule settings.
Records merge by decoded key,
arrays concatenate,
and a kind mismatch anywhere in an input group selects the final value.
Inputs are not modified.
This is not a generic JavaScript deepmerge port.

## Embedded sources

Processors are always on.
Fenced Rust in Markdown and MDX,
rustdoc comments in Rust,
and doc tests inside rustdoc are linted as virtual files named under their host,
such as `guide.md/3.rs` and `lib.rs/12.md/1.rs`.
Configuration patterns select virtual files by those names.
Findings and fixes are reported at host positions.
A fix that cannot be mapped back to the host is dropped and its finding is kept.

## What `--fix` leaves alone

A fix may be reported by a finding and still be withheld,
and the finding stays reported:

- A rustdoc comment inside a fenced listing is quoted source,
  so a finding there carries no fix.
  This holds for a Rust listing in Markdown and for a doc example in a Rust file's doc comment,
  while the Rust file's own doc comments are still broken at their `///` and `//!` markers.
- A pipe table is only reported.
  `markdown/no-pipe-tables` has no fix.
- A bare URL whose link text contains a backtick is reported without a fix,
  because the parser read a code span delimiter into the URL.
- A line break inside a blockquote carries no `>`,
  also when the quote sits in a list item or holds a list.
  A `>` is a token,
  and a fix that only changes whitespace never adds one.
  The new line is a lazy continuation:
  CommonMark 0.31.2,
  section 5.1 "Block quotes",
  rule 2 "Laziness",
  keeps a line without `>` in the paragraph of the
  quote it continues,
  at any depth,
  and the rule never breaks where the next line would start another block,
  the one case laziness excludes.
  A quote whose lines all carry `>` therefore gets a line without it where a break is added;
  it renders the same,
  and an author who wants `>` on every line adds it by hand.

## Fix classes

Every fix carries a class,
and `--rules` prints each rule's class in `fixClass`:
`whitespace`,
`content`,
or `null` for a rule that has no fix.

- A `whitespace` fix changes line breaks and the spaces around them,
  and no token of the text.
  `markdown/semantic-line-breaks` is the only rule of this class.
  In a Rust file the continuation line of a broken doc comment also gains the comment marker,
  which is comment syntax and not part of the text.
- A `content` fix adds,
  removes or rewrites a token.
  These are `markdown/commands-show-output`,
  `markdown/no-trailing-punctuation`,
  `markdown/no-bare-urls`,
  `markdown/fenced-code-language`,
  `markdown/link-image-reference-definitions`,
  `markdown/link-image-style`
  and `markdown/lfs-image-url`.

Plain `--fix` applies only `whitespace` fixes:
it is the command of the repository's `format:monochromatic-lint` task and of agents before a commit,
and it changes no token.
A `content` fix is withheld without `--fix-content`,
and its finding stays reported.
The in-process entry `monochromatic_lint::run_embedded::fix_source` applies both classes,
because its caller names the rules it wants in its own configuration:
the commit path's `markdown/autofix` runs only `markdown/lfs-image-url`,
which is a content fix.
The reasons and the options that were ranked are in
`doc/planning/monochromatic-lint-fix-split-whitespace-and-content.md`.

## Inline directives

A finding can be suppressed by a comment that names this tool,
in the ESLint family's spelling:

- `monochromatic-lint-disable` suppresses from the comment to the matching enable,
  or to the end of the file.
- `monochromatic-lint-enable` ends a disable.
- `monochromatic-lint-disable-line` suppresses on the comment's own line.
- `monochromatic-lint-disable-next-line` suppresses on the line after the comment.

Each keyword is followed by optional comma-separated rule identifiers
and a mandatory justification after a run of two or more dashes set off by whitespace.
Without identifiers a directive covers every suppressible rule.
Identifiers are the full identifiers that `--rules` lists and findings carry in `code`,
such as `rust/tests-in-sidecars`;
a bare `tests-in-sidecars` is reported as unknown.

```rust
//! Directives sit in ordinary comments,
//! directly above the line a finding is reported on,
//! which for an inline test module is the line of the `mod` keyword.

/// Generated tests.
#[cfg(test)]
// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- generated bindings keep their tests inline
mod tests {}
```

In Markdown a directive is an HTML comment,
for example `<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- quoted from a log -->`.
MDX has no directive form:
its HTML comments are syntax errors,
and its expression comments are not read.

### Which comments count

A directive is read only from a comment that the parser reports,
and only when the comment's text starts with the keyword.
Rust directives are `//` and `/* */` comments;
doc comments (`///`,
`//!`,
`/** */`,
`/*! */`) are rustdoc prose and never directives.
Markdown directives are HTML comments that begin an HTML node,
so a directive shown in a code fence,
an indented code block,
an inline code span or a larger HTML block is only an example.
Text inside Rust strings is never a comment.
A comment whose first word starts with `monochromatic-lint-` but is not one of the four keywords
is reported as malformed;
a comment that merely mentions the tool elsewhere is ordinary text.

### What a directive covers

A finding belongs to the line and byte offset where its first label starts.
A line directive is matched by line,
and a range directive by the position of its comment:
a finding that starts before the comment stays.
The latest range directive that names the rule decides,
so `monochromatic-lint-disable` followed by `monochromatic-lint-enable rust/tests-in-sidecars`
re-enables only that rule.
A line directive wins over a range.
Lines follow the host language:
Rust counts LF lines and byte columns,
Markdown counts CR,
LF and CRLF lines and UTF-16 columns.
Directive findings use the same units as every other finding.

A directive acts on the file that contains it.
Findings in embedded sources are reported at host positions,
so a Rust comment above a rustdoc block covers the Markdown findings inside that block.
Rust fenced in Markdown has only `rust/max-lines` and `rust/require-rustdoc` applied to it,
both unsuppressible,
and a comment inside the fence is code,
not a directive.
The semantic rule `rust/require-explicit-types` goes through the same filter as the syntax rules;
a processing failure never does.

### Rules that refuse directives

`rust/max-lines` and `rust/require-rustdoc` declare themselves unsuppressible.
A directive entry that names either is reported as `directive/non-suppressible`
and the rule's findings stay;
the other entries of the same directive still apply.
No configuration key,
override or nested configuration changes this:
the declaration is part of the rule,
and `--rules` shows it as `"suppressible":false`.

### Exempting a fenced example from `rust/require-rustdoc`

A directive cannot withhold `rust/require-rustdoc`,
but a documentation example that is undocumented on purpose,
such as a "Bad" example in a style guide,
can opt out through its fence.
Put the word `monochromatic-lint-skip-rustdoc` in the fence's info string after `rust`:

````markdown
```rust monochromatic-lint-skip-rustdoc
fn deliberately_undocumented() {}
```
````

GitHub renders the block as Rust,
because the language is the first word of the info string.
The comma form (`rust,monochromatic-lint-skip-rustdoc`) works as well,
and the word may follow other attributes (`rust,ignore monochromatic-lint-skip-rustdoc`).
The word must stand alone,
so `monochromatic-lint-skip-rustdoc-not` exempts nothing.
Only `rust/require-rustdoc` is withheld from the marked fence:
`rust/max-lines` still applies to it,
and an unmarked fence in the same file is checked as before.
The nearest enclosing fence decides,
so a marker on an outer fence that quotes another fence does not exempt the quoted one.
The owner chose this form on 2026-11-07 (exempt only "Bad" examples),
and every other fenced Rust example is documented by hand.

### Findings about directives

These findings are always errors,
have stable identifiers,
and are not themselves suppressible:

- `directive/missing-justification`:
  the directive has no reason,
  so it suppresses nothing.
  Every keyword needs one,
  `monochromatic-lint-enable` included.
- `directive/non-suppressible`:
  an entry names a rule that refuses directives.
- `directive/unknown-rule`:
  an entry names no shipped rule.
- `directive/malformed`:
  an unknown `monochromatic-lint-` keyword,
  an empty entry in the rule list,
  or a line directive written over several lines.

`directive/unused` is reported only on request.
`--report-unused-disable-directives` reports at warning severity,
and `--report-unused-disable-directives-severity` takes `allow`,
`off`,
`warn`,
`deny` or `error`;
the two options exclude each other,
as in oxlint,
and `allow` and `off` report nothing.
Unused directives are not reported by default.
A listed directive is judged per rule,
and the unused identifier is underlined.
A directive is not judged after a processing failure,
because a rule's findings may be missing.

### Choices the design left open

Where the planning documents were silent,
the stricter reading was taken:

- Only full rule identifiers are accepted.
- A directive that is void (no justification,
  or malformed) suppresses nothing and does not start a range.
- Doc comments are never directives,
  although the planning ledger listed "any comment token":
  rustdoc is where this tool's own documentation quotes directives.
- A line directive in a block comment over several lines is malformed,
  as ESLint treats it,
  rather than guessed at.
- An empty entry such as `a,,b` is malformed,
  so a typo cannot widen a directive.
- An `enable` that re-enables nothing is unused,
  and so is a directive that a later one covering the same finding made redundant.

### Fixing and suppression

Suppression runs before fixes are collected,
so `--fix` never applies the fix of a suppressed finding,
and the directive stays in the file.
Every `--fix` pass rechecks the new text,
so a directive covers the line it names in the text that is now there.
A fix of one finding that moves text onto another line,
such as a line break inside a paragraph,
can move a covered finding out of its directive's line;
the next pass then reports it.

## Closure check

`rust/avoidable-closure` runs the type-aware Dylint lint `avoidable_closure` of
`package/linter/monochromatic-lint-closure` over each Rust package that has a selected file.
The syntax-only `rust/no-anonymous-functions` rule was removed on 2026-10-18,
and a configuration that still names it fails as an unknown built-in rule (status `2`).
The decision and its consequences are recorded in `doc/decision/rust-closure-lint-dylint.md`.

The rule is enabled by naming it in configuration,
with the directory of the Dylint library in its `library` option,
relative to the configuration file:

```jsonc
// monochromatic-lint.config.jsonc
[
  {
    "files": ["**/*.rs"],
    "rules": {
      "rust/avoidable-closure": {
        "severity": "warn",
        "library": "package/linter/monochromatic-lint-closure"
      }
    }
  },
  // A later block raises one tree to error, and turns the rule off for tests.
  {
    "files": ["package/ide/app/**/*.rs"],
    "rules": { "rust/avoidable-closure": { "severity": "error" } }
  },
  {
    "files": ["**/tests/**/*.rs", "**/*_tests.rs"],
    "rules": { "rust/avoidable-closure": { "severity": "off" } }
  }
]
```

The rule has no comment directive:
`rust/avoidable-closure` declares itself unsuppressible,
and `--rules` shows `"suppressible":false`.
A closure that its callee requires carries rustc's own attribute instead
(`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`),
and an `expect` that suppresses nothing is reported by this rule as well,
at the file's severity.

### What a run does

After the per-file rules,
the linter groups the Rust files that select the rule by their owning `Cargo.toml` (the nearest ancestor)
and checks each package once:

1. It skips the package,
   with a warning finding,
   when a prerequisite is missing (see "Prerequisite manifest").
2. It runs `cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked --message-format json`
   with `CARGO_NET_OFFLINE=true` and `DYLINT_RUSTFLAGS` set to request `avoidable_closure` and
   `unfulfilled_lint_expectations` as warnings.
   Every other variable is inherited,
   so the toolchain and build-directory settings of the caller reach the child `cargo`,
   and a package's setup may add the ones its own build needs.
   Dylint builds the library with the active toolchain.
3. It reads cargo's JSON records,
   keeps the two lint codes,
   and reports each as a finding of `rust/avoidable-closure`
   in the file,
   with the severity that the file's own configuration selects.
   A report in a file whose rule is off,
   or in a file that the run was not asked to lint,
   is dropped.
   Positions use the Rust convention of this linter,
   byte columns,
   recomputed from the file.

Only the library and the binaries are compiled (no `--all-targets`),
so `#[cfg(test)]` modules,
`tests/`,
examples and benchmarks are not checked.
Directories ending in `.fuzz` or `.bench` are skipped by name whatever the configuration says.
A package that fails to build,
a cargo failure without any diagnostic,
a missing library directory,
and an invalid
prerequisite manifest are `core/processing-failure` findings (status `2`),
because the result would be incomplete.
The step does not run under `--fix` or `--stdin`.
Closures in generated code (for example `slint::include_modules!` output below the build directory) are skipped by
the lint itself.

### Machine requirements and skips

The check needs the shared nightly toolchain with `rustc-dev`,
`cargo-dylint` and `dylint-link`.
Without them,
or without `cargo` itself,
every package is skipped with one warning finding each.
The only evidence of a missing `cargo-dylint` is cargo's own `no such command` for `dylint`.
Any other failure of `cargo dylint --version`,
including the statuses 126 and 127 of a launcher that cannot start its program,
is a `cargo` that is broken or older than `toolchain.lock.json`,
and it fails the run with status `2`,
with what cargo printed and the command `mise run toolchain:install`,
because a skip that exits `0` would hide the stale install.
A skip is reported as a finding of the rule `rust/avoidable-closure` at warning severity,
at the start of the package's `Cargo.toml`,
and its message names the package and the command that fixes the gap:

```jsonl
{"code":"rust/avoidable-closure","severity":"warn","filename":"package/x/Cargo.toml","message":"rust/avoidable-closure skipped package/x: ..."}
```

The linter never fetches.
Dylint resolves the dependencies of every platform in a lockfile,
so a package whose cargo cache is cold is skipped with the command that fills it:
`cargo fetch --locked` in the package directory.
The same holds for the Dylint driver,
which `cargo dylint` keeps once per toolchain
(in `DYLINT_DRIVER_PATH`,
or `.dylint_drivers` in the home directory)
and rebuilds from the crate `dylint_driver` at exactly its own version whenever the driver is older.
That crate belongs to no package lockfile,
so before the first package of a library the linter reads the toolchain active in the library directory
(`rustup show active-toolchain`),
runs the driver's `-V`,
and,
when the driver is absent or older than `cargo-dylint`,
resolves a throwaway crate that pins `dylint_driver` offline in a scratch directory under the per-user cache directory
(`XDG_CACHE_HOME`,
or `.cache` in the home directory).
If that fails,
every package of the library is skipped with the exact command that fills the caches,
`env --unset=CARGO_NET_OFFLINE cargo dylint --path <library> --manifest-path <manifest> -- --locked`,
which the owner runs once with network access,
and again after each `cargo-dylint` upgrade.
A check that cannot be made (an unreadable version,
no `rustup`,
no known driver directory) skips nothing
and shows only under `--debug`;
the run then decides as before.
A skip is a warning finding like any other:
the exit status stays `0` unless `--max-warnings` is exceeded,
`--max-warnings` counts it,
and `--quiet` and `--silent` hide it with the other warnings.
The repository's own `lint:monochromatic-lint` task runs `mise run fetch:rust-dependencies` first,
so a skip that remains there means a package that fetching could not fill.
The owner decided this on 2026-11-07 ("Fetch then warn"):
a cold-cache skip once hid five packages' findings behind a green run.

A `cargo` that starts and exits 126 or 127 is not a skip.
Those are the statuses of a program that exists and cannot be started,
which cargo itself never returns (a missing `cargo-dylint` answers 101),
so the `cargo` link or launcher is broken:
its real program was deleted by a newer install of a repository tool,
for one
(`doc/troubleshooting/mpm-shadow-launcher-names-a-deleted-release-folder-after-a-newer-install.md`).
The run then fails with a processing failure that quotes what that `cargo` printed on standard error,
because skipping would exit 0 with nothing checked.
A `cargo` that is not on the machine at all stays a skip.

The `cargo` program is `cargo` on `PATH`,
or the program named by the environment variable `MONOCHROMATIC_LINT_CARGO`.
The variable belongs to the machine,
never to repository configuration.
The tests use it to run the whole step against a stub program.
While it is set,
the linter does not look for the compiler's `rustc-dev` component.

### Prerequisite manifest

A package that needs something before `cargo check` can compile it has a file
`monochromatic-lint.prerequisites.jsonc` beside its `Cargo.toml`.
It is a JSONC object whose keys are kinds from a fixed list that this program implements.
A key that is not in the list is an error that names the list;
the file never holds a command,
so repository JSONC cannot choose an executable to run.
Names are plain names (letters,
digits,
`.`,
`_`,
`+`,
`-`,
no separator,
no leading dash).

```jsonc
// package/file-manager/app/monochromatic-lint.prerequisites.jsonc
{
  "pkg-config": ["gtk4"]
}
```

The kinds:

- `pkg-config`:
  a list of module names;
  each must satisfy `pkg-config --exists`.
- `executables`:
  a list of program names;
  each must be found on `PATH` (looked up,
  never run).
- `qmake`:
  `true`;
  a Qt 6 `qmake6` or `qmake` must be on `PATH` or `QMAKE` must name an existing file,
  and `QMAKE` is set for the build.
- `ide-runtime-artifacts`:
  `true`;
  `target/debug/runtime/manifest.json` and `target/crate-licenses.json` must exist.
  The linter builds in `target/dylint` and links the runtime as `target/dylint/debug/runtime`
  and the license list as `target/dylint/crate-licenses.json`,
  because the IDE's `build.rs` takes the parent of the nearest `build` directory above `OUT_DIR` as the profile
  directory and reads `runtime` in it and `crate-licenses.json` beside it.
- `libghostty-offline-inputs`:
  `true`;
  `target/ghostty-offline/ready`,
  `target/ghostty-offline/source/build.zig` and the directory `target/ghostty-offline/packages` must exist.
  They are what `mise run //package/terminal/app:fetch:ghostty-offline-inputs` leaves behind,
  the marker last.
  The linter sets `GHOSTTY_SOURCE_DIR` and `GHOSTTY_ZIG_SYSTEM_DIR` to the source tree and the package directory,
  which `libghostty-vt-sys` reads instead of cloning Ghostty and letting `zig build` download packages.
- `android-ndk`:
  `true`;
  `cargo ndk-env --target arm64-v8a` must print the NDK variables,
  which are set for the build,
  and the check compiles for `aarch64-linux-android`.

A missing prerequisite skips the package with a warning finding that names it.
There is no kind that runs a `mise` task:
preparing a package can need containers and network,
which a linter does not start,
so a missing artifact names the task that makes it.

### Suppression record

A closure that its callee requires carries the `expect` attribute,
and every such attribute is listed here with the file,
the closure it covers,
and the true sentence its reason states.
Configuration cannot replace these attributes:
the callee's own parameter list rules out a named function,
and the rule refuses comment directives.

- `src/markdown_source.rs`:
  the `catch_unwind` closure that runs the parser.
  The attribute sits on the closure argument and reads
  "catch_unwind accepts only a callable without parameters,
  so the borrowed source and the parser options must be captured by a closure".
- `src/run_process.rs`:
  the `catch_unwind` closure in `contained_run`.
  "catch_unwind accepts only a callable without parameters,
  so the borrowed options and the runner must be moved into a closure".
- `src/run_process.rs`:
  the `spawn_scoped` closure in `run_in_scope`.
  "spawn_scoped runs the closure on the spawned thread after this call returns,
  so the borrowed options and the runner must travel inside it".
- `src/run_process.rs`:
  the `std::thread::scope` closure in `run_process_with`.
  "std::thread::scope hands its callback only the scope reference,
  so the borrowed options and the runner can reach run_in_scope only through a closure".
- `src/run_workers.rs`:
  the `catch_unwind` closure in `contained_with`.
  "catch_unwind accepts only a callable without parameters,
  so the plan,
  the fix flag,
  the LFS map,
  the engine and the writer must be moved into a closure".
- `src/run_workers.rs`:
  the `catch_unwind` closure in `contained_source`.
  "catch_unwind accepts only a callable without parameters,
  so the plan,
  the source,
  the fix flag,
  the LFS map and the engine must be moved into a closure".
- `src/run_workers.rs`:
  the `spawn_scoped` closure in `spawn_and_join`.
  "spawn_scoped runs the worker closure on the spawned thread after this call returns,
  so the shared run state must travel inside it".
- `src/run_workers.rs`:
  the `std::thread::scope` closure in `run_workers`.
  "std::thread::scope hands its callback only the scope reference,
  so the shared run state and the worker count can reach spawn_and_join only through a closure".
- `src/cli_options.rs`:
  every closure clap's derive generates for `CliOptions`,
  reported at the field type spans.
  The attribute is the module's inner `#![cfg_attr(...)]` and reads
  "clap's derive generates the parser closures in its FromArgMatches implementation with the field type as their span,
  so no author-written closure exists to refactor".

## Rust test sidecars

`rust/tests-in-sidecars` requires unit tests to live in sidecar files.
It reports a module that has an inline body under a `cfg` that selects test builds
(`cfg(test)`,
`cfg(all(test, ...))`,
`cfg(any(test, ...))`),
once per outermost module,
and a function marked `#[test]` or a runtime's `#[...::test]` outside a sidecar file.
A `#[cfg(test)] mod name;` declaration that loads a file is the sidecar form and is never reported.
A `cfg(not(test))` module and a key-value `cfg(feature = "test")` are not test configurations.

A rule sees one parsed file and cannot see which file declares it,
so a sidecar is recognized by its name:
a file named `*_tests.rs` or `tests.rs`,
or any file under a `tests` directory.
Each finding names the sidecar to create,
such as `thing_tests.rs` for `thing.rs`,
and the declaration to write:
`#[cfg(test)] #[path = "thing_tests.rs"] mod tests;`.
Test-only helper functions without `#[test]`,
doc tests and virtual files in embedded sources are not checked.

Exempt a crate whose whole purpose is testing,
such as a fuzz sidecar crate,
by turning the rule off in a later block,
which leaves its other rules selected:

```jsonc
// monochromatic-lint.config.jsonc
[
  {
    "files": ["**/*.rs"],
    "rules": {
      "rust/tests-in-sidecars": { "severity": "error" }
    }
  },
  {
    "files": ["**/*.fuzz/**/*.rs"],
    "rules": {
      "rust/tests-in-sidecars": { "severity": "off" }
    }
  }
]
```

## Rust layout rules

Two crate-wide rules check that the folder tree of a Rust package shows who depends on whom
(`AGENTS.md` rules LYD,
LYL and LYF).
Both are off by default and are switched on for a package in configuration,
by selecting them for any of its Rust files:

```jsonc
// monochromatic-lint.config.jsonc
[
  {
    "files": ["package/git-policy/cli/**/*.rs"],
    "rules": {
      "rust/folder-uses-links": { "severity": "error" },
      "rust/file-order-in-folder": { "severity": "error" }
    }
  }
]
```

A rule sees the whole package,
not one file,
so it runs once per Cargo package after the per-file rules,
as the closure check does.
The package of a file is the one whose `Cargo.toml` is nearest above it,
and the first selecting file of a package decides the severity.
`--stdin` and embedded sources are not checked.
The rules report findings that no inline directive can suppress.

### What both rules read

The rules follow the real module tree and never walk folders.
They start at the library root and every executable root
(`src/lib.rs`,
`src/main.rs`,
`src/bin/*.rs`,
`src/bin/*/main.rs`,
and the `path` of `[lib]` and of each `[[bin]]`),
then follow `mod` lines:
`mod x;` loads `x.rs` or `x/mod.rs`,
`#[path = "..."]` moves the file (also with `..`),
and an inline `mod x { ... }` is a module in the file that holds it.
A module that cannot be followed (no file,
both files,
or a file that loads its own container) fails the run with a `core/processing-failure` finding,
and that package is not checked.

A module belongs to the folder of its file:
`a/mod.rs` and `a/b.rs` both belong to the folder `a`.
Test-only code is outside both rules:
files named `*_tests.rs` or `tests.rs`,
files under a `tests` folder,
and anything marked `#[cfg(test)]` (a module,
an item or a `use`),
including a sidecar declared inside a production file with `#[path]`,
or `#[test]` on a function.

A folder uses another folder when code in one of its module files names a module that the other folder defines.
The names that count are paths:
`crate::...`,
`super::...`,
`self::...`,
the leaves of groups such as `use crate::{a, b::{c, d}}`,
and a path that starts with the name of a module declared beside the code (`use child::Thing;`).
A path stops at the deepest module it names;
the names after it are items of that module.
Paths inside macro calls count,
and comments,
strings of every kind and character literals never do,
because the rules read the parser's tokens and not text.
`super::` inside an inline module names the module around it.

### Folder rule

`rust/folder-uses-links` asks every source folder
(a folder that holds a production module file) to hold a `uses` folder of symbolic links,
one for each folder its code uses and none for any other.
Findings:

- A used folder has no link.
  The finding names both folders and the file and line of one reference that proves the use,
  and gives the command that creates the link.
- A link points to a folder the code does not use.
- An entry of `uses` is not a symbolic link,
  is a broken link,
  points to a file,
  points to a folder that is not a source folder of the same package (including a folder outside it),
  points to its own folder,
  or is not named like the folder it points to.
  `uses` itself must be a real folder.
- Folders use each other,
  directly or through others:
  one finding lists the folders and one reference per use inside the loop.
- A `pub use path::*` glob re-export of a module of the package (see below).
- A module file lies below a folder named `uses`.
- A group folder breaks its shelf rules (see "Group folders").

Decisions:

- A folder that holds child folders is a group folder and follows the shelf rules in "Group folders".
  Before the group rules,
  a parent folder with code of its own and its child folders were folders like any other,
  and the parent and a child that named each other were a loop;
  now a folder that holds child folders holds nothing else,
  and a folder inside it never names it.
- A folder that uses nothing needs no `uses` folder,
  because Git cannot store an empty folder.
- The link name must be the name of the folder it points to,
  so two used folders with the same name cannot both be linked:
  rename one.
- Glob re-exports hide which folder an item comes from,
  so each `pub use path::*` (any visibility) of a module of the package is a finding of its own.
  Module names are still resolved through such a glob (a flat `pub use folder::*;` at the crate root makes
  `crate::module` reach `folder/module.rs`),
  one level of glob at a time and with a visited set so rings end,
  and the path of a glob is read through direct children only.
  An item that a glob supplies resolves to the module that re-exports it.
- A name that a module re-exports by name (`pub use sibling::Name;`,
  or `pub use sibling::Thing as Name;`) is followed to the module that defines it,
  through any chain of such re-exports;
  a path that reaches the name through the re-exporting module is a use of the module it names
  and also of the defining module,
  which is the child folder when the re-export comes from a child folder.
  The walk keeps a visited set of module and name pairs,
  so a ring of re-exports ends
  and the use stays on the module where the ring closed.
  The named module stays a used module because dropping it would hide a loop through a re-exporting file:
  on `package/git-policy/cli` the other choice let three files of a fifteen-file loop escape.

### Group folders

A group folder lets a folder tree show who depends on whom at two scales:
the entries of a package at the top,
and the exact folders inside each group.
The owner picked this rule on 2026-11-07T11:05:21-05:00 ("Shelf + summaries (Recommended)",
`doc/planning/cli-git-source-layout-target-tree.md`,
sections "One level of groups in the program" and "The owner's words and the rules that bind this").

#### How the linter knows a folder is a group

A folder is a group when it holds child folders:
its index file is a `mod.rs`,
and at least one source folder sits directly inside it.
The folder of a crate root (`lib.rs`,
`main.rs` or a `src/bin` root) is never a group,
because it is where the groups and the one-folder entries sit.
An **entry** is a group folder,
or a source folder that sits outside every group (a one-folder entry).
The entry of a folder inside a group is that group;
the entry of any other folder is the folder itself.

The form needs no new syntax,
and a group cannot be forgotten:
a group folder that grows code of its own still holds child folders,
so it is still a group,
and the shelf rules below report the code.
Nothing in the top doc says "this is a group".

Rejected for now (open to the owner's veto):
a marker sentence in the top doc,
in the style of the file list marker,
for example `//! Folders in this group:`.
Its pro is that it changes nothing for a folder that has code and child folders today.
Its cons are new syntax that a reader must learn,
and that a group whose marker is missing is not a group,
so the shelf-only restriction is again a convention that nothing enforces.
The structural form changes one earlier decision:
a parent folder with code of its own beside child folders used to be "a folder like any other".
No package of this repository selects the layout rules yet
(measured on 2026-11-07:
no `uses` folder anywhere under `package/`,
and `monochromatic-lint.config.jsonc` does not name `rust/folder-uses-links`),
so no existing layout is broken by the change.

#### What the rule checks

All group checks belong to `rust/folder-uses-links`,
and the file rule leaves a group folder alone,
so a finding is never reported twice.

- A group holds only folders.
  It holds no module file but its `mod.rs`,
  and its `mod.rs` holds only its top doc,
  inner attributes,
  comments and `mod x;` lines.
  Every other item is a finding,
  including a `pub use` re-export,
  which would make a path through the group name the group instead of the folder that defines the item.
  The finding names the file and the item.
- A group's `uses` folder holds one link for every other entry that a folder inside the group uses,
  and no other link.
  A folder inside the group uses an entry when the folder's code names a folder of that entry
  (a use of a folder of the same group is not a use of another entry).
  A used entry without a link,
  and a link to an entry that no folder of the group uses,
  are findings.
  A link that points to a folder inside another group (not to the group) is a finding of its own
  that says to link the group.
  The proving reference of a missing link is the smallest one by file and line,
  and the help gives the command that creates the link.
- A folder inside a group keeps the exact links of the folder rule:
  one link for each folder its code uses and none for any other,
  including folders of other groups (`h/c/uses/a` links `../../../g/a`).
- A folder never links a group itself,
  and its code never names one.
  It links the folder inside the group that its code names.
  A link to a group,
  or a path that names a group,
  is a finding that says so.
  A group's own summary links are the one place a group is linked.
- No two entries use each other,
  directly or through others,
  where a use of a folder inside a group counts as a use of the group.
  Two groups never use each other,
  and a group and a one-folder entry never use each other.
  The folder loop check still finds loops between folders;
  a loop between entries is reported as one more finding that lists the entries and one proving reference per use.
- A group never sits inside a group:
  a folder inside a group that holds child folders is a finding,
  because the owner's rule is one level of groups.

The finding texts,
one for each check
(the file and line are the proving reference of a use,
or the link or file the finding is about):

```text
src/g holds the file helper.rs, but a group folder holds only its child folders and its mod.rs.
src/g/mod.rs is the index of the group folder src/g, so it may hold only its top doc and `mod` lines, but it holds a function `helper`.
src/h/c uses the folder src/g/a (`crate::g::a::X`), which is in the group src/g, but src/h/uses has no link to src/g.
src/h/uses/k links to src/k, but no folder inside src/h uses it.
src/h/uses/a links to src/g/a, which is inside the group src/g, so a group links the group and not a folder inside it.
src/h/c uses the group folder src/g itself (`crate::g`), but a folder names the folders inside a group that it uses, never the group itself.
src/h/c/uses/g links to the group folder src/g, but a folder links the folders inside a group that it uses, never the group itself.
These groups and folders use each other in a loop: src/g uses src/h (src/g/a/mod.rs:1 `crate::h::c::Y`); src/h uses src/g (src/h/d/mod.rs:1 `crate::g::b::X`).
src/g/a holds folders, so it is a group folder, but it sits inside the group folder src/g, and groups go one level deep.
```

A one-folder entry also keeps exact links:
when its code names a folder inside a group,
its `uses` folder links that folder and never the group.
The source lives in `src/layout_group_entries.rs` (which folders are groups),
`src/layout_group_summary_links.rs` (the summaries) and `src/layout_group_folders.rs` (the shelf rules,
the groups that are named or linked,
and the loop between entries).

The links in a group's `uses` folder are written like the links of any other folder
(a link named like its target,
relative,
to a source folder of the package).
A group that none of its children make use of needs no `uses` folder.

### File rule

`rust/file-order-in-folder` asks the files of one folder to use each other in no loop
and the folder's top doc to list them in dependency order.
A group folder (see "Group folders") has no files to order and is left to the folder rule.
Findings:

- Files use each other in a loop.
  One finding lists the files and one reference per use inside the loop.
- A folder has module files but no `mod.rs` to hold the top doc.
- An index file holds code (see "Index files hold only the index").
- The top doc has no file list,
  lists a file twice,
  lists a name that is not a module file of the folder,
  or omits a file.
- A file uses a file that is listed after it
  (the finding names both and the proving reference).
  Uses inside a loop are not reported again here.

The top doc is the `//!` lines at the start of the folder's `mod.rs`
(`lib.rs`,
or `main.rs` when `lib.rs` is absent,
in the folder of a crate root).
The list starts at the marker sentence below
and has one `- ` line per file,
with the file name in backticks first.
The marker is read as prose,
like the Markdown rule `markdown/semantic-line-breaks` reads it:
it may wrap wherever a sentence may,
because the lines of one paragraph are joined with single spaces before the match,
and any run of spaces equals one space.
A blank `//!` line ends a paragraph,
so a marker with a blank line inside it is no marker:

```rust
//! What this folder is for,
//! in words for someone who knows only `git commit` and `git push`.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `saving_a_file.rs`:
//!   writes a file so that a crash never leaves half of it.
//! - `saving_many_files.rs`:
//!   saves several files together,
//!   using `saving_a_file.rs` for each one.
```

Everything else in the doc is free text and is never read.
An entry starts at a line beginning `- `,
and the indented lines under it belong to that entry.
The list ends at the first line that is neither blank,
an entry nor a continuation.
The index file itself is not listed,
and neither are the other crate roots (`main.rs` beside `lib.rs`),
test files and subfolders.
Using the index file with `super::` is never reported,
because the index holds no code that could take part in a loop.
A sibling that reaches a name through the index is a use of the file the index re-exports the name from
(see the re-export decision under the folder rule),
so a re-export hides neither a loop nor a listing order.

#### Index files hold only the index

An index file is the `mod.rs` of a folder,
or the `lib.rs` (else `main.rs`) of the folder of a crate root.
It may hold:

- its `//!` top doc,
  inner attributes and comments;
- `mod x;` lines with any attributes and docs (`#[path]`,
  `#[cfg(test)]`,
  `#[macro_use]`);
- `pub use`,
  `pub(crate) use` and `pub(super) use` re-exports.

Every other item is a finding of `rust/file-order-in-folder`,
on the item's own line (not its attribute or doc line),
one finding for each:
a function,
struct,
enum,
union,
`impl` block,
trait,
constant,
static,
type alias,
`macro_rules!` or macro definition,
a macro call,
an `extern crate` line,
an `extern` block,
a `use` that is not public,
and an inline `mod x { ... }` with a body.
The message names the kind and the name:

```text
src/a/mod.rs is the index of its folder, so it may hold only its top doc, `mod` lines and `pub use` re-exports,
but it holds a function `helper`.
```

The help says to move the item into a file of the folder named for what it does,
to add `mod` for that file in the index,
and to re-export the names other folders use with `pub use`.
Code marked for test builds (`#[cfg(test)]`,
`#[test]`) is not read.
The check runs even when the index is the only file of its folder.

An entry point is a `main.rs` that is the index of its folder (there is no `lib.rs` beside it).
The compiler pins `fn main` to that file,
so it may also hold `fn main` (also `async` or under an attribute)
and plain `use` lines,
which feed only `fn main`.
No code can name `main`,
so it cannot close a loop,
and the index stays out of the loop check.
Any other item is a finding in the entry point's own words:

```text
src/main.rs is the entry point of its program, so it may hold only its top doc, `mod` lines, `use` lines and `fn main`,
but it holds a function `run`.
```

A `main.rs` beside a `lib.rs`,
and a single-file binary under `src/bin`,
are other crate roots.
They stay out of the loop check and the top doc list,
and they are read for code like an entry point:
the same items are allowed and the same message is used.
A `main.rs` that is the index of its own folder under `src/bin/<name>/` is an entry point already.
A root named by a `[[bin]]` path anywhere else is not read for code.

### Known limits

- A group is recognised by its child folders,
  so a folder that holds child folders must be a shelf;
  the rules have no way to say that a folder with code and child folders is on purpose.
- The group checks are part of `rust/folder-uses-links`,
  and `rust/file-order-in-folder` leaves a group folder alone.
  With only the file rule selected,
  code in a group's `mod.rs` is therefore not reported,
  where the file rule's index check reported it before the group rules.
- Only `[lib] path` and `[[bin]] path` written as `path = "..."` lines are read from `Cargo.toml`;
  dotted keys and inline tables are not,
  and test,
  example and benchmark targets are test-only code.
- Names are resolved by module,
  not by item:
  an item that a glob supplies,
  or a `#[macro_export]` macro used as `crate::name!`,
  belongs to the module the path names.
  A name re-exported by name is followed to the module that defines it.
- A glob import (`use super::*;`) names the index and hides which re-exported file it reaches;
  only a re-export that is a glob is a finding of its own.
- When a module re-exports one name twice (for example from two `cfg` branches),
  the first `pub use` in the file decides.
- `cfg` gates other than the ones that name `test` are not evaluated;
  every `mod` line is followed.
- A module that a macro declares is not seen.
- The rules cannot see a use through a trait method or a type alias that never names the module.
- `/*! ... */` top docs are not read.

### Links and other tools

The links form a tree of copies when something follows them.
The linter's own walk never enters a link
(`ignore` is configured with `follow_links(false)`),
so a file behind a `uses` link is linted once;
the tests `layout_uses_links_are_never_entered_by_the_walk` and
`layout_rules_run_in_the_built_executable_and_do_not_follow_links` fail when that setting is reversed.
Only a link that a user names as a command-line path is entered.
`cargo package` follows the links and packs nested copies,
so a package that has `uses` folders needs `exclude = ["**/uses"]` in its `Cargo.toml`
(measured in `doc/planning/cli-git-source-layout.md`,
section "The `uses/` links and cargo").
The layout rules themselves find modules by following `mod` lines and read only the names of the entries of a
`uses` folder,
never the files behind them.

`mise run //package/linter/monochromatic-lint:test:layout` runs the layout tests alone;
`test:layout:container` runs them in the bounded container.

## Explicit Rust types

`rust/require-explicit-types` uses resolved Rust declarations and inferred types,
not method-name guesses.
It checks missing binding annotations,
generic arguments,
and nameable inferred `_` placeholders.
A function item's unnameable type can retain `_`;
that does not exempt a nameable enclosing reference or aggregate type.
Unresolved coverage produces an explicit processing failure rather than a clean result.

Closures have unnameable types,
so prefer a named callback,
which is also what the `avoidable_closure` Dylint lint asks for:

```rust
//! A named callback has a nameable type.
// src/main.rs

/// Return one user's name for the `map` call.
fn user_name(user: &User) -> String {
    return user.name.clone();
}

let names: Vec<String> = users
    .iter()
    .map::<String, _>(user_name)
    .collect::<Vec<String>>();
```

The semantic engine requires an existing Cargo workspace file and installed matching `rust-src`.
It does not install toolchains or fetch dependencies.
The host chooses source-only or generated-source preparation;
repository JSONC cannot provide executable commands.
Syntax-only rules do not load a Cargo workspace.
The executable prepares workspaces source-only:
it reads source and Cargo metadata and generates nothing,
so a definition that only a build step would produce is reported as a processing failure.
Files that select this rule are checked on one thread after the other files.

## Native Markdown rules

Implemented checks cover heading increments,
single top-level titles,
duplicate headings within textual ancestor scopes,
emphasis-only headings,
trailing heading punctuation,
prompt-only shell examples,
fence languages,
bare URLs,
shortcut-reference style,
and unused/duplicate reference definitions.
The rules share one native Markdown/MDX parse and retain authored UTF-8 byte ranges.
MDX expression,
JSX,
and ESM subtrees are not prose-rule inputs.

Fixes preserve surrounding authored syntax.
Punctuation edits consume complete escapes and character references.
Prompt removal preserves LF,
CRLF,
and bare CR.
Reference-definition deletion keeps adjacent container lines separate.
The grouped editor refuses a nonempty-to-empty file rewrite.

Pipe-table conversion and semantic line breaks are implemented.
Production Rust and Markdown commands have not been replaced.

## LFS image URLs

`markdown/lfs-image-url` rewrites a relative image link to an LFS-tracked file
into the object URL built from the repository's `.lfsconfig` endpoint,
and reports an object URL whose target is missing or stale.
The repository root is the nearest ancestor holding `.lfsconfig`,
and the search stops at the top of the checkout the file belongs to:
every checkout top carries `.git`,
as a directory in an ordinary repository and as a file in a linked worktree,
and a file in a linked worktree is never configured by the checkout that encloses it,
because Git gives it none of that checkout's attributes.
Tracking comes from the root `.gitattributes`,
resolved the way Git resolves an attribute:
each line's own `filter` assignment counts from left to right and the last matching line wins,
so `filter=lfs` tracks a path and every other assignment
(`filter=other`,
`filter`,
`filter=`,
`-filter`,
`!filter`) unselects it.
The `exclude` option takes gitignore-syntax patterns relative to that root.
A pattern reaches the matcher through `gitignore_pattern::for_matcher`,
because a brace is text to Git and an alternate group to the glob grammar beneath the matcher.

Endpoint normalization is one standard-library function,
`lfs_object_base`,
with a restricted contract:
`http` or `https`,
an ASCII host name,
an optional decimal port,
and a path without dot segments.
Any other endpoint is rejected with a named reason that never echoes the endpoint,
because it may hold credentials.
One rejected declaration fails the whole read.

## Workflow shell rules

A GitHub Actions workflow is a host for shell scripts.
A path is a host only when it is a `.yml` or `.yaml` file directly in `.github/workflows`,
or an `action.yml` or `action.yaml` anywhere under `.github/actions`;
other YAML is never read,
so a stray `run:` key elsewhere cannot fail a run.
Every step with a `run:` script is linted as a virtual file named `<workflow>/<n>.<suffix>`,
where `n` counts the extracted steps in file order
and the suffix names the dialect:
`.sh` for bash,
sh,
dash,
ash,
ksh and zsh,
`.fish`,
`.ps1` for PowerShell,
and `.cmd`.
Configuration selects steps by those names,
for example `.github/workflows/*.yml/*.sh`.
Findings are reported at the workflow's own lines and columns.

The nine rules are all unsuppressible,
so a directive comment suppresses nothing,
and each takes only `max`:

- `shell/max-lines` counts the non-blank,
  non-comment physical lines of a step's script,
  including backslash continuations and heredoc content.
  `max` defaults to 2.
- `shell/no-pipe`,
  `shell/no-and-chain`,
  `shell/no-or-chain`,
  `shell/no-pipe-stderr`,
  `shell/no-semicolon`,
  `shell/no-dollar-paren`,
  `shell/no-backtick`
  and `shell/no-ampersand`
  ban `|`,
  `&&`,
  `||`,
  `|&`,
  `;`,
  `$(...)`,
  a backtick substitution
  and a single `&`.
  `max` is the occurrences allowed in one step and defaults to 0.
  An occurrence over `max` is reported at its own range.

The scan reads each dialect's own grammar.
It skips GitHub `${{ ... }}` expressions,
comments,
single-quoted text,
and double-quoted text apart from `$(` and a POSIX backtick,
which still run there.
Redirections such as `2>&1`,
`&>` and `>|`,
arithmetic `$((...))`,
and here-document bodies are not operators.
The dialect table follows the fish language manual,
the PowerShell `about_Operators` and `about_Pipeline_Chain_Operators` pages,
and the `cmd` command reference:
fish has no backtick substitution,
PowerShell has neither a backtick substitution nor `|&`
and uses a leading `&` as the call operator,
and `cmd` has only `|`,
`||`,
`&&` and `&`.

The dialect comes from the step's `shell:`,
then the job's `defaults.run.shell`,
then the workflow's.
The word after any leading `env` names the interpreter,
so `env bash {0}` is bash.
`node`,
`python` and the other listed non-shell interpreters are out of scope.
With no `shell:`,
a job whose `runs-on` text contains `windows` is PowerShell and every other job is a POSIX shell.

Only two `run:` forms are read:
a block literal (`run: |`)
and a plain one-line command.
A folded,
quoted,
anchored,
aliased,
tagged or flow `run:`,
a `shell:` that is an expression or an unlisted interpreter,
a step that is not a block mapping,
a second document,
and tab indentation are each a `core/processing-failure`
at the offending byte,
because a skipped step would hide a long script from the rules.

A ratchet holds a workflow at its current worst step by giving it a configuration block
with the workflow's own `max` values:

```jsonc
// monochromatic-lint.config.jsonc
{
  "name": "shell-ratchet-old-workflow",
  "files": [".github/workflows/old.yml/*.sh"],
  "rules": {
    "shell/max-lines": { "severity": "error", "max": 40 },
    "shell/no-pipe": { "severity": "error", "max": 3 },
  },
},
```

## Verification

Package tasks cover type checking,
unit tests,
and mount-free container tests.
`lint:container` runs the library tests,
the black-box tests of the built executable in `src/binary_tests.rs`,
and Clippy inside a bounded container.
Consumer migration and mutation-survivor disposition gates are tracked in the execution document;
their absence is not a completion claim.
`mutation:markdown` runs full container tests and Clippy before scoped native Markdown mutation.
`mutation:executable` does the same for the orchestration modules,
the executable's entry point,
the command grammar,
Rust rule selection,
input expansion and the fix loop;
`mutation:core` for configuration,
findings,
inline directives,
grouped edits and the syntax-only Rust rules;
`mutation:layout` for every layout module,
against the layout tests;
`mutation:workflow-shell` for the workflow walk and the shell dialect scanners,
against the workflow and shell tests;
`mutation:semantic` for the semantic engine,
the explicit-type rule and the manifest lookup;
`mutation:syntax-only` for the router of a build without the `semantic` feature,
built without the default features against its own test target
(`src/syntax_only_engine_tests.rs`,
which `mise run test:syntax-only` also runs);
and `mutation:processors:files` for the processor modules.
The router's source file is compiled only without the `semantic` feature,
so a campaign built with the default features never compiles it,
and every one of its mutants survives there:
this is why it has a scope of its own.
That scope lists one mutant that no test can kill,
`replace RustFileEngine::workspace_count -> usize with 0`,
because the function already returns `0`
(`src/rust_file_engine_syntax.rs`),
so a campaign of that scope always exits with the missed-mutant status.
The executable and core scopes skip the five tests that load a Cargo workspace,
which the semantic scope and the full container tests run.
The semantic scope has its own per-mutant limit,
and any campaign accepts `-- --shard k/n` (zero-based,
as in cargo-mutants) so that its shards can run in separate bounded containers at once.
`mutation:coverage` compares the scopes' listings with the unscoped listing,
name by name,
and fails when any mutant is outside every scope.
A campaign with a missed or timed-out mutant exits nonzero and is not a passing gate.
The session's main agent starts every campaign and waits on it,
never a subagent
([why](../../mutation-test/cli-cargo-mutants/README.md#who-runs-a-campaign)).
Every campaign is resumable:
after a kill,
a reboot or an out-of-memory stop,
run the same task with the same options on the same test image and
it scores only the mutants that no earlier run scored,
whatever their outcome.
Its results are in `target/verification/mutation-<key>/mutants.out/` (`processors-mutation-<key>` for the processor
campaign;
`caught.txt`,
`missed.txt`,
`timeout.txt`,
`unviable.txt`,
`outcomes.json`),
the union of every attempt,
and its `exit.json` records the status of the whole
campaign.
Delete that directory to score everything again.
The mechanism and its limits are in `package/mutation-test/cli-cargo-mutants/README.md`.

Two kinds of mutation are never tried,
by the repository owner's decision of 2026-10-05.
Every cargo-mutants invocation of this package passes these patterns,
each as its own `--exclude-re` argument:

- `replace \+= with \*=`
- `replace -= with /=`

Both replacements leave a counter unchanged,
so a loop that steps its own index never ends,
and cargo-mutants 27.1.0 exits with its timeout status whenever any mutant times out.
`+=` replaced by `-=`,
and `-=` replaced by `+=`,
still test every counter.

The closure check is tested in the same container against a stub `cargo`,
a Rust program that the tests compile from `src/closure_stub_cargo.rs` and point `MONOCHROMATIC_LINT_CARGO` at:
the unit tests cover the manifest grammar,
the prerequisite probes,
the report parsing and the mapping to findings,
and `src/closure_binary_tests.rs` drives the built executable for the skip,
failure and severity cases.
`mise run //package/linter/monochromatic-lint:test:host` runs the one test that needs the real tools
(`cargo-dylint`,
the `rustc-dev` nightly and a warm cargo cache) over `package/wg/cli-quicker-exempt`,
and compares it with an `--all-targets` run of the same library so that the absence of test findings is shown by a
probe that could have found them.

The fuzz sidecar includes counted Markdown/MDX fixtures,
raw source,
reparsing of accepted fixed output,
and an `orchestration` target that drives the executable's per-source path
through nested doc tests and adversarial edit groups.
Evidence belongs to its recorded source snapshot;
new source does not inherit a previous campaign's completion status.

## Script linting

The verification scripts in `bin` are TypeScript.
`mise run //package/linter/monochromatic-lint:lint:scripts:types` type-checks them,
and `mise run //package/linter/monochromatic-lint:lint:scripts:oxlint -- bin/<file>.ts` lints the files it is given.
The Oxlint task takes explicit paths and has no package-wide form on purpose.
A package-wide `lint:oxlint` also lints `task.ts`,
which no `tsconfig.json` includes:
it does not satisfy the isolated-declarations setting of the shared TypeScript preset,
so adding it to the project fails the type check.
Measured on 2026-10-30,
that run exits 0 but the semantic bridge reports `api-unavailable` for six files
and the type-aware rules (`prefer-readonly-parameter-type`,
for one) throw instead of checking,
so a green package-wide run would not mean the scripts passed those rules.
Making the package-wide task usable needs `task.ts` to be fixed or excluded,
not a looser rule.

## License

LGPL-3.0-or-later.
License texts are in `LICENSES/`.
