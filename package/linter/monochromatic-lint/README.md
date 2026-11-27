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
relative to the directory the configuration is read from
(the folder holding a discovered configuration file,
or the working directory for `--config`):

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
One package of this repository selects the layout rules today:
`package/zfs/request-to-openzfs-kernel-device` with its fuzz sidecar,
in `monochromatic-lint.config.jsonc`'s `layout-zfs-request-to-openzfs-kernel-device` block.
Neither rule is on repo-wide,
so no other package's layout is checked by them;
a repo-wide rollout would surface the existing packages and stays a separate decision.
`package/git-policy/cli` is switched on by the `layout-git-policy-cli` block of the same configuration,
with the two lists described under "The allow list a rule may take".
`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md` records that decision,
the lists' regeneration against `main` before they landed,
and why a folder's links are written only once its uses no longer loop.

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
- A group's own code owes its group no summary and makes no edge of the entry graph:
  the finding above reports the file that holds it,
  and a group's summary is what its child folders use.
  A `uses` line naming a use written in a group's own file is dead,
  unless a loop of folders holds that use:
  a `uses` line keeps its use alive through a loop of entries,
  and such a use makes no entry edge.
  A use from that file is reported as a loop whenever it closes one.
  The loop of folders reports it while the two folders sit in one folder loop
  (a use added inside a loop the list already tolerates is reported the same way).
  The loop checks of the entries cannot see it,
  so the entries are searched back from the entry the own file uses,
  over every use between two entries and every use written in a group's own file.
  The shapes that reports are:
  one own-file use that closes a loop with the entry it uses,
  a mutual dependency that two groups' own files close,
  which is one finding for each of the two uses,
  and a ring of entries that any number of own-file uses close.
  A group nested in another group is searched too:
  it is its outer group's entry,
  so a use in its own file leaves from that entry,
  and the entry the finding names in its key is the nested group's folder.
  The shapes it does not report are:
  a use of the group's own child folders,
  which joins no two entries;
  a use a folder loop already holds,
  which the loop checks report;
  and a use of an entry whose own code never reaches the group.
  The entry graph of every use is built once for the package,
  one pass over it groups the entries that reach each other in a loop,
  and a group whose own file closes no loop is walked not at all,
  so a package of such groups pays that one pass and no walk.
  One walk answers every own-file use of one group:
  it reads the uses backwards from that group,
  breadth first,
  and stops as soon as every use of that group has its route,
  so each message spells the fewest uses that lead back,
  and each use takes its route from that one table.
  The table every walk reads is built once for the package,
  so a package of many groups whose own files close loops reads its graph once,
  not once per group.
  A walk stops once every use of its group has its route,
  so an entry that reaches the group directly costs the uses that reach the group
  and nothing of the graph behind that entry.
  When several routes are equally short,
  each step takes the entry whose path sorts first,
  so the message spells the smallest route,
  never the one a walk happened to reach first.
  A walk per use read the graph once for each of them instead,
  which cost 14 to 16 times alpha.15's time on a package of 2,000 folders whose one group's own file uses all of them
  (measured on 2026-11-26 in interleaved rounds,
  on the run's own CPU time,
  because the host's wall time for one run swung from 27 to 547 seconds while its CPU time stayed near 27):
  this release spends 0.07 of alpha.16's CPU time there in each of three rounds,
  0.14 to 0.16 on the same shape with 1,000 folders and thirty uses each,
  and 0.98 to 1.00 on the shape above with its loop open,
  where nothing is walked,
  against 0.94 to 1.11 between two runs of one binary.
  Measured on 2026-11-26,
  one process per binary per round in its own capped scope,
  with only ratios a finding and every ratio taken between runs of one shape:
  on a package of 250 groups reached from 250 folders,
  each folder using one child folder of every group (62,500 uses,
  every link written)
  and each group's own file using one folder that reaches nothing,
  alpha.14 took 13.3 seconds,
  4.41 times alpha.13's time,
  alpha.15 3.3 seconds,
  1.02 to 1.08 times,
  and this release 3.2 and 3.3 seconds,
  0.97 to 1.06 times,
  while the same graph with an empty own file took 3.0 and 3.1 seconds on alpha.13 and this release.
  On a three-layer package that shape does not cover,
  300 groups and 300 folders,
  each folder using every later folder and a child folder of every group
  (134,850 uses,
  every link written),
  alpha.14 took about 51 seconds,
  8.1 to 8.8 times alpha.13's time,
  alpha.15 13.2 to 14.4 seconds,
  2.05 to 2.37 times,
  and this release 6.5 to 7.8 seconds in four rounds,
  1.00 to 1.19 times alpha.13's time and 0.49 to 0.54 times alpha.15's,
  and every binary's output on both packages is byte-identical.
  Both shapes were measured again between the alpha.16 and this release's binaries on 2026-11-26,
  and no difference beyond the host's own noise appeared:
  0.98 to 1.10 times alpha.16's CPU time on the 250-group package,
  against 1.00 to 1.02 between two runs of one binary,
  and 1.01 to 1.04 on the three-layer package,
  where the walk costs alpha.16 0.90 to 1.05 of the run with empty own files and this release 1.00 to 1.05.
  The alpha.17 pass measured one shape of many closing groups
  (300 folders that each use every later one,
  beside 100 groups whose own files each close one loop,
  about 45,000 uses),
  and on it the closing arm cost 0.80 to 1.10 times the same binary's open arm on its branch,
  where alpha.17's costs 3.83 to 5.49 times its own and alpha.16's 0.77 to 0.94;
  on the same shape in miniature (120 folders,
  40 groups) the three were 1.07 to 1.14,
  1.94 to 2.06 and 1.03 to 1.06
  (three interleaved rounds,
  each figure inside one binary and its control arm in the same round,
  on the run's own CPU time,
  2026-11-27).
  This branch was measured on that shape and on one where each own file's use reaches its group
  through everything that reaches it:
  every folder also uses a child of every group,
  and each own file uses the first folder,
  which reaches every folder (about 45,000 uses again),
  so a walk that read every entry reaching the group would cost that whole graph for each of the 100 groups.
  With 300 folders and 100 groups,
  three interleaved rounds and the three binaries rotated,
  on the run's own CPU time (2026-11-27):
  on the first shape the closing arm costs 0.86 to 1.00 times this branch's own open arm,
  alpha.18's 1.01 to 1.35 and alpha.16's 1.03 to 1.20,
  and this branch costs 0.80 to 1.07 times alpha.16's closing arm and 0.72 to 1.04 times alpha.18's;
  on the second shape the closing arm costs 0.98 to 1.36 times this branch's own open arm,
  alpha.18's 2.22 to 2.72 and alpha.16's 0.98 to 1.36,
  and this branch costs 0.92 to 1.13 times alpha.16's closing arm and 0.43 to 0.51 times alpha.18's.
  The same graph with its loop open costs this branch 0.99 to 1.32 times alpha.16's time
  and 1.09 to 1.22 times alpha.18's on the first shape,
  and 1.05 to 1.12 and 0.85 to 1.06 on the second,
  where no walk runs,
  so a ratio inside one binary and its control arm in the same round is the finding,
  never a ratio between two builds.
  Every binary's output on both shapes is byte-identical,
  keys and messages included.
  A dense loop a group's own file closes is written down in one run too:
  on a package of 200 groups whose own files all use one folder,
  which uses a child folder of every group,
  beside 200 folders each using every later one (60,100 uses in the loop),
  one `--debug` run prints 60,300 keys:
  one `uses` line for each of the 59,900 uses inside the loop that are not written in an own file,
  one `own-file-loop` line for each group's own use,
  and one `group-file` line for each group's own file;
  the second run with that 1,681,290-byte list reports nothing.
  Every such finding is keyed `own-file-loop <group> -> <used folder>`,
  where the group is the folder whose own file holds the use,
  so a tree that already holds one can record it on a list,
  and no `uses` link is owed for any of these uses.
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
- A folder has module files but no `mod.rs` to hold the top doc,
  and each of those files is then a finding of its own,
  because the missing doc lists nothing and tolerates nothing else.
- A folder's index file exists and cannot be read.
  One finding says so,
  and each file of the folder is a finding of its own for the same reason:
  an unreadable doc lists nothing.
  An index that is not UTF-8 reaches this finding,
  and so does one whose permission bits allow no read.
  A folder reaches it when the module tree finds that folder's files without reading its index,
  for example when a `#[path]` declaration in another folder names them;
  when the module tree itself must read the index to find them,
  an unreadable index ends the package with a `core/processing-failure` naming it instead.
  A `mod.rs` that no `mod` line declares is therefore read only when it cannot be read:
  a readable one is no part of the package,
  so it is no folder's index,
  it moves no key,
  it hides no code check of the folder's real index,
  and it makes no group of the folder around it.
  Such a file gets no layout finding at all;
  the per-file rules still lint it like any other source file the walk selects.
- A file's name holds a space,
  which a `#[path]` attribute can declare.
  The finding names the file and its folder,
  and its key names both (`space-in-file-name src/m p q.rs`),
  so a line recorded for it never tolerates a file renamed later.
  A key line is read as its whole text and names such a file exactly,
  so the file stays in every list,
  loop and order check:
  a loop through it is reported,
  and its missing list entry is reported under a key that names it.
  What such a name does forbid is a `file-uses` line,
  which separates its words by spaces,
  so a loop through that file cannot be tolerated:
  the file is renamed,
  or the loop is untangled.
- An index file holds code (see "Index files hold only the index").
- The top doc has no file list,
  and a doc with no list then reports every file of its folder as missing from the list,
  one finding per file,
  so a file added to such a folder is a finding and no `no-file-list` line tolerates it.
- The top doc spells the marker nearly,
  so no list is read at all:
  one finding names the doc,
  the line,
  the words found and the marker expected,
  and the doc then also reports the missing list and each unlisted file
  (see "When the marker is nearly there").
- The top doc lists a file twice,
  lists a name that is not a module file of the folder,
  or omits a file.
- A file uses a file that is listed after it
  (the finding names both and the proving reference).
  Uses inside a loop are not reported again here.

The layout rules answer the line of each finding by reading the file once per run,
up to eight mebibytes.
A file of exactly eight mebibytes is read whole,
so even an unterminated last line keeps its span.
A longer file reports the file start for a line whose bytes the bound did not reach
and for a line the bound cuts off,
because a guessed length would point at the wrong bytes.

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

#### When the marker is nearly there

A doc that does not hold the marker has no list,
and the rule reports the missing list as `no-file-list`
and every file of the folder as `missing-entry`.
When the doc spells the marker nearly,
it also reports one `near-miss-marker` finding that names the doc,
the line where the words start,
the words found and the marker expected,
so the person fixing it sees the cause and not only those effects.

A near miss is a run of whole doc lines,
starting where a line starts,
whose words are the marker's words with at most one word added,
dropped or put in place of another.
The words are compared without their capital letters,
so the same words in another case are a near miss too.
A text two words away is a different sentence that its author wrote on purpose,
so no finding asks for it to be changed.
The search reads each doc line a bounded number of times,
so a long run of blank or plain-comment lines after the first doc line
costs its own length and never the square of it.
Which texts count,
with one example of each:

- one word added:
  `Files in this folder, each using only the files listed before it:`.
- one word dropped:
  `Files in this folder, each using only files before it:`.
- one word put in place of another:
  `Files in this folder, each using only files listed after it:`,
  and the same marker without its colon,
  where `it` stands in place of `it:`.
- the same words in another case:
  `files in this folder, Each using only files listed before it:`.
- a paragraph break inside the marker:
  `//! Files in this folder,`,
  a blank `//!` line,
  `//! each using only files listed before it:`.
  A plain comment or an inner attribute between the two halves breaks the paragraph as well,
  and the finding names both lines.

A break between two words inside one paragraph is already the marker,
because the reader joins the lines of a paragraph,
so it is no near miss.
A doc that holds the marker reports no near miss,
even beside text that is nearly the marker,
because it reads its list.
One doc reports one finding per near miss in the order it writes them,
keyed `near-miss-marker <index file>`
and numbered `#2` and then `#3` for a later one.

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

### The allow list a rule may take

A crate that switches a layout rule on does not become correct at once.
Its findings go into a list,
one entry per finding,
and the rule then tolerates exactly the entries of that list:

```jsonc
// monochromatic-lint.config.jsonc
{
  "files": ["package/git-policy/cli/**/*.rs"],
  "rules": {
    "rust/folder-uses-links": {
      "severity": "error",
      "allow": "package/git-policy/cli/layout-allow-list/folder-rule.txt",
    },
  },
}
```

The path is relative to the directory the configuration is read from:
the folder holding a discovered `monochromatic-lint.config.jsonc`,
or the working directory for one named by `--config`,
which is also the base of that configuration's file patterns.
A run whose list cannot be read fails with a `core/processing-failure` finding naming the list,
because a rule checked without its list would report every finding it tolerates.
A list is read only when it is a regular file of at most four mebibytes,
so a mistyped path cannot block on a pipe or read a device.
Four mebibytes is one line per use of a dense loop a group's own file closes:
the largest such loop measured here needed 60,300 lines and 1,681,290 bytes (2026-11-26),
and one mebibyte refused that very list.

A line of a list is one of three shapes,
and a line that reads as none of them fails the run:

- `uses <folder> -> <folder>` tolerates one use between two folders of the package.
  A loop is made of uses,
  so a loop is tolerated only while every use inside it is listed,
  and a round writes one line for each use it leaves alone.
  A folder name may hold a space,
  so the line is read from the two sides of the arrow it holds:
  the folder before the arrow,
  and the folder after it.
  One rule reads every spelling of the line,
  and it holds for the character after the word `uses`,
  for the one before the arrow and for the one after it:
  one whitespace character sets each word off,
  whatever character it is,
  a space,
  a tab or a no-break space,
  and a second whitespace character beside a name belongs to that name,
  on either side of the arrow.
  So a list aligned with tabs reads as one aligned with spaces does,
  a run of two spaces in front of a name is read as that name's own leading space,
  a hand-aligned line names a folder the package does not hold,
  and its finding says which folder the line read and that no source folder is called that.
  A line that sets no arrow off with whitespace on both sides is refused,
  and its sentence names those whitespace characters and where they stand.
  A space left at the end of a name is part of that name too,
  and no folder of a tree the rules accept ends in whitespace,
  because such a name is refused before any check:
  so a line that leaves one before its arrow names a folder no tree holds,
  and the run reports that one line dead,
  naming the folder it read,
  while every other line of the list is still read and every other finding of the rule is still reported.
  A `uses` line holds one arrow only,
  because no folder name it names holds an arrow word,
  and a `uses` line that sets two of them off with whitespace is refused,
  with a sentence that says so.
  The file rule reads no `uses` line,
  and a `file-uses` line whose folder name holds a set-off arrow is read from its last arrow,
  whose place a file name cannot take.
  A line stays alive while its two folders still make that use and still sit in one loop together,
  in the folders' own graph or in the graph of the entries:
  a folder inside a group can be in a loop of its own folders and in the loop of its group at once,
  and listing every use of both leaves neither a loop finding nor a line that tolerates nothing.
- `file-uses <folder> <file> -> <file>` does the same for one use between two files of one folder.
- every other line is the exact key of one finding of the rule.

A path in an entry is relative to the package folder,
the way findings show it,
written with `/` on every platform,
never absolute,
and no line may repeat another.
Every listed path is read back against the package folder before it is compared,
so one spelling of a use decides tolerance,
death and the repeat a second line would be,
in both rules alike:
with the package folder named `p`,
`uses ../p/src/a -> src/b` names the use `uses src/a -> src/b` names,
and a line holding both spellings is refused as a repeat.
A folder outside the package,
which a `#[path]` can declare,
is written by the walk from the package folder,
`../outside`,
exactly as a key writes it,
so a loop through such a folder is recorded like any other loop.
A `.` segment and a trailing separator are read away,
so `uses ./src/a/ -> src/b` is the same entry as `uses src/a -> src/b` and the second line for it is refused.
A `.` on its own names the package folder itself,
which is how its keys write it.
A `uses` path that holds a backslash is refused too,
because a key writes `/` on every machine and a backslash would name a folder no finding names.
The two file words of a `file-uses` line are read as written,
backslash and all,
because a file name is not a path:
a key about a file keeps that name exactly,
so such a line names the file it spells.
A line that is a key may hold a backslash,
because a macro call's key keeps the escapes of its arguments as the source writes them:
such a line is read as the key it spells,
matches the finding whose key it is,
and is reported as tolerating nothing any more when no finding prints it.
The uses and the keys of a list are indexed once when it is read,
so a rule answers each use and each key from that index and never walks the entries for one:
on a package whose 62,750 keys a run prints back as the list,
alpha.16 spent 2.3 to 2.7 times its own CPU time on that tree with an empty list,
and this release 0.8 to 1.0 times over four rounds,
ranges that do not overlap (measured on 2026-11-26).
The ratio sits below one because a run whose findings the list tolerates prints none of them,
so once the scan is gone the listed run is the cheaper one.
A `file-uses` line is matched by walking the entries instead,
because a folder holds few files:
an index for the file uses of a list that holds 517 of them measured 0.86 to 1.04 times the walk's CPU time,
against 0.87 to 0.98 between two runs of one binary
(three interleaved rounds on `package/git-policy/cli`,
2026-11-27).

A rule reports the key of each finding it makes in `--debug`,
as a note that reads `allow-list key: <key>`,
so a reader copies a key instead of guessing its shape.
A key is built from the file and the words the finding itself holds and never from a line number,
so an entry survives an edit above the finding it names.
Every key names exactly one finding:

- A key that names an item carries the item's own name,
  such as `index-item src/a/mod.rs function-end_after_the_log`,
  or its own words when the kind has no name.
  The shapes are exactly these,
  each printed the way the item is written:
  - `impl-<what it implements>` and then the members it declares,
    sorted,
    such as `impl-std::fmt::Display for Holder fmt` or `impl-Holder one two`,
    with the members left out when the block declares none.
  - `private-use-<the tree, with every run of whitespace squeezed to one space>`,
    such as `private-use-std::fmt` or `private-use-std::{ fmt::Debug, }`,
    or `private-use-anonymous`.
  - `macro-call-<path>!<the delimiter and its arguments>`.
    Every run of whitespace in the arguments becomes one space,
    even inside a string,
    and every escape stays as the source writes it,
    so `foo!("a\n", b)` gives `macro-call-foo!("a\n", b)`,
    `vec![1, 2]` gives `macro-call-vec![1, 2]`
    and `thread_local!{ static X: u8 = 0; }` gives
    `macro-call-thread_local!{ static X: u8 = 0; }`.
    A call written without a delimiter keeps the bare form `macro-call-<path>!`,
    which is what `bar!;` prints;
    an empty call keeps its parentheses,
    so `foo!()` gives `macro-call-foo!()`.
  - `extern-crate-<name>`,
    such as `extern-crate-alloc`,
    and `extern-crate-<name>-<the rename as written>`,
    such as `extern-crate-alloc-as al`,
    and `extern-block-<the ABI string, quotes kept, with every run of whitespace squeezed to one space>`,
    such as `extern-block-"C"`,
    or `extern-block-unstated` for a block that names none.
  - `<kind>-<name>` for every other item,
    where the kind is `function`,
    `struct`,
    `enum`,
    `union`,
    `trait`,
    `constant`,
    `static`,
    `type-alias`,
    `macro-rules`,
    `macro-def` or `inline-module`,
    with `anonymous` in place of a name the item does not declare.
- A key about an entry of a `uses` folder carries the fault,
  such as `link src/a/uses/b broken`,
  because one entry can go wrong in ten ways.
- A finding of the folder checks that is neither a use nor a link nor a glob
  starts with its own shape word:
  `unused-link`,
  `group-file`,
  `names-group`,
  `links-group`,
  `nested-group`,
  `below-uses`,
  `summary-missing` or `summary-extra`.
  `own-file-loop` is the word of the one loop finding a list can hold.
- A finding of the file checks starts with its own shape word:
  `no-top-doc`,
  `no-file-list`,
  `near-miss-marker`,
  `unreadable-index`,
  `index-item`,
  `space-in-file-name`,
  `listed-twice`,
  `foreign-name`,
  `missing-entry` or
  `order`.
  `order <index file> <file> <file>` names the file that uses a file its top doc lists after it.
- A key about a file or a folder writes its path relative to the package folder,
  and the package folder itself is written `.`,
  so `no-top-doc .` and `missing-link . -> sub` name it,
  and no key ends in the space a list reader trims away,
  and none holds a newline either,
  because a list line is one line and ends at a newline.
  A name that ends in a space,
  a tab or another character the reader trims,
  or that holds a line break,
  is therefore refused before any check,
  with one `core/processing-failure` that spells that character and names the path.
  A folder name that holds a backslash,
  which a `uses` line writes as the separator `/` on the machines that separate folders with it,
  is refused for the same reason,
  and so is a folder name that holds an arrow word:
  ` -> ` between two names,
  `-> ` at its beginning,
  ` ->` at its end,
  or `->` alone,
  and a tab or a no-break space where those shapes spell a space,
  because the `uses` line naming such a folder would be read as another pair of folders.
  That last refusal is wider than the shapes a run of alpha.17 read back wrongly,
  and it is deliberately a rule about the name:
  a folder name is refused whether or not any `uses` line names it,
  so no later edit of a tree can print a line that the next run cannot read,
  and the price is that a package selecting the folder rule stops on such a name
  even where no line of its list carries it.
  The file rule reads no `uses` line and names such a folder in a key,
  which holds the name as written,
  so the arrow words of a folder name are refused only while the folder rule is selected.
  A file name and an entry of a `uses` folder keep their backslashes and arrow words as the source writes them,
  because neither is a path:
  a `file-uses` line names a file as written,
  and the key about an entry holds the entry's name as written,
  so both read back.
  The refusal is a rule about the tree's names,
  not a condition on the findings one run reports:
  it covers every file and folder the walk reads,
  a crate root and a `src/bin` root included,
  and every entry of a `uses` folder for the characters the reader trims and for a line break,
  whether or not this run has a finding to report about that name,
  so a rename today can never put a key on the list that no line could hold tomorrow.
  Two keys held a line break on alpha.15 although no name the walk read held one,
  and both are gone:
  the ABI string of an `extern` block is keyed with every run of whitespace squeezed to one space,
  so `extern "C<line break>x"` keys `extern-block-"C x"`,
  and an entry of a `uses` folder that ends in a trimmed character or holds a line break is refused like any other name.
  A top doc entry is read without the whitespace at the end of its backticks,
  so an entry written `x.rs ` names the file `x.rs` and no key carries that space.
  An entry whose backticks hold no name at all is keyed by its shape and index alone,
  `foreign-name <index file>` or `listed-twice <index file>`,
  with `#2` for a second such entry,
  so a line can hold it.
  A module file outside the package,
  which a `#[path]` attribute can declare,
  is written by the walk from the package folder:
  `missing-entry ../outside x.rs`.
  No key holds an absolute path,
  so a file inside the package folder reads the same in every checkout.
  A file outside the repository is named from the package folder,
  so its `..` count is the distance between the two folders:
  the same in every checkout of one repository,
  and different when the same outside file is named from a package at another depth.
  A `uses` line names such a folder the same way,
  so a loop through a folder outside the repository is recorded and tolerated like any other loop.
- Two findings a rule still cannot tell apart,
  such as two blocks of one type that declare the same members
  or two glob re-exports of one module,
  are numbered in the order the rule reports them:
  the first keeps the plain key and each later one carries `#2`,
  `#3` and so on,
  and the number goes into the key's first word,
  right after the shape it repeats,
  so `foreign-name#2 src/lib.rs x` can never be read as the entry named `x#2`,
  which prints `foreign-name src/lib.rs x#2`.
  Every key a rule prints starts with a shape word and a space,
  so no literal key can hold `#` there.
  Adding one of them therefore adds a finding no line holds,
  and removing one leaves a line that tolerates nothing.
  The number counts the findings of that shape from the first one the rule reports,
  so it moves only when an identical finding is added above or removed above it:
  an edit above the finding,
  or a different block of the same type,
  renumbers nothing.

The rule holds the list to its size,
and the list's own record of today's findings is the only thing that does:

- A finding is tolerated exactly when its key or the use it reports is on the list.
- An entry that tolerates nothing any more is a finding on its own line.
  That is how a round that fixes a finding is made to delete the entry:
  an entry whose use is gone,
  whose two ends are no longer in one loop,
  or whose key no finding reports,
  fails the run until it is gone.
  An entry of the two use shapes whose folder the package does not hold
  names that folder and says that no source folder of the package has that name,
  which is true of a folder that is missing and of one that holds no module file,
  because a hand-aligned or mistyped line reads as a folder no tree holds,
  and its reader has to re-space the line rather than delete it.
- A use added between two folders or two files that the list already tolerates in one loop is a finding.
  It carries no key,
  so no round can record a new loop edge:
  the use is removed,
  or that loop is untangled first.
- A loop that a use in a group's own file closes is held by one line per use inside it:
  that use by its `own-file-loop` line,
  and every other use inside the loop by a `uses` line.
  The list holds folder pairs,
  and a loop is made of uses,
  so a `uses` line inside such a loop stays alive while its two folders still make that use;
  a loop one of whose own-file findings the list does not hold is reported,
  and every use inside it is printed as the line that records it,
  so one run of the rule writes the whole loop down.
  A use that joins the loop after that has no line,
  and the finding for it carries no key,
  so the loop is untangled before new code can join it.
  A use added in the same change as a new own-file use of that loop is printed with it,
  because the loop is reported then,
  and the review sees both lines in one diff.
  A `uses` line typed in by hand for a use that joined a recorded loop tolerates it,
  as any hand-written line tolerates the finding it names.
- A loop finding of folders,
  entries or files carries no key,
  and neither does the use added inside a tolerated loop,
  so no entry can tolerate either.
  The one exception is a use written in a group's own file,
  which no `uses` line records:
  its finding is keyed `own-file-loop <group> -> <used folder>`,
  so a tree that already holds one puts that line on the list.

What a hand-written line can and cannot hide:

- A line typed in by hand for a finding no round fixed tolerates that finding from then on,
  and nothing in the rule can tell such a line from one recorded on the round that fixed it.
  A list that holds every use of a loop nobody tolerated before therefore hides that loop:
  `uses src/x -> src/y` and `uses src/y -> src/x` silence a new loop of two folders.
  Reviewing the list is what keeps the list honest,
  and writing a line for a new finding is a review blocker.
- A line cannot hide a finding of one of the shapes a line names nothing for:
  a loop finding and a use added inside a tolerated loop carry no key,
  so no line recorded from `--debug` can hold one,
  and only a hand-written `uses` line for every use of that loop can,
  which the point above already covers.
  A use written in a group's own file is the exception,
  and its line is the one a round writes.

Which package a list belongs to is the rule's file,
so the two rules of a crate take two lists.

#### How a list written before alpha.11 maps

An `impl` block's key gained the members it declares,
a numbered key carries its number in a new place,
a folder whose top doc cannot be read reports a new line per file it holds,
and two `uses` shapes that alpha.10 held are no longer tolerated,
so a list written for alpha.10 needs one regeneration,
and every other line stays as it is.
A list written before alpha.10 predates both of those shapes,
so it is regenerated from a `--debug` run rather than mapped here.

- An `impl` block's key gained the members it declares,
  sorted and separated by spaces:
  `impl-Display for Holder` became `impl-Display for Holder fmt`,
  and `impl-Holder` became `impl-Holder one two`.
  What the block implements is printed as the trait first and the type after ` for `,
  with no ` for ` when the block names no trait.
  A block that declares no member keeps the plain key.
  A member added to a block later renames its key the same way,
  so the line for the old key tolerates nothing and the block reports a new finding.
- A numbered key now carries the number in its first word,
  right after the shape it repeats:
  `listed-twice src/a/mod.rs one.rs#2` became `listed-twice#2 src/a/mod.rs one.rs`,
  and `glob src/lib.rs native#2` became `glob#2 src/lib.rs native`.
  A key printed with `#N` there is the Nth finding of a shape whose own words repeat an earlier finding's,
  and the number moves only when such an identical finding is added or removed above it.
- A folder whose top doc cannot be read now reports one `missing-entry` line per file it holds,
  beside the line about the missing or unreadable doc:
  `missing-entry <folder> <file>` for a folder with no `mod.rs`,
  and `missing-entry <index> <file>` for an index file that cannot be read.
  A `no-top-doc` or `unreadable-index` line therefore tolerates only its own finding.
- A `uses` line between two members of one group is dead,
  unless a loop of folders holds the two folders together:
  an entry loop holds a use only when it reaches outside its entry,
  and two members of one group share that group as their entry.
  alpha.10 kept such a line alive through the group's entry loop.
- A `uses <group> -> <folder>` line for a use written in the group's own file is dead,
  unless a loop of folders holds that use:
  a group's own code adds no edge to the graph of the entries,
  and its group's summary is what the child folders use.
  alpha.10 kept such a line alive through the group's entry loop.

A `link` key holds the fault of the entry,
and one entry of a `uses` folder can go wrong in ten ways:
`unreadable`,
`not-a-symbolic-link`,
`broken`,
`points-to-a-file`,
`not-a-source-folder`,
`points-to-its-own-folder`,
`named-after-another-folder`,
`uses-unreadable`,
`uses-is-not-a-folder`,
and `uses-unlistable`.

#### How a list written before alpha.12 maps

alpha.12 changed key shapes that `main`'s tree does not hold,
so this repository's two lists needed no edit,
and a measurement on 2026-11-26 against `main`'s `package/git-policy/cli` confirmed it:
alpha.11 and alpha.12 report the same 2,008 layout findings and the same 1,939 keys,
and with the repository's own configuration and the committed lists both report none.
A tree that holds one of the shapes below needs one regeneration:

- Two glob re-exports written on one line are two findings,
  so a line naming only the first no longer tolerates the second.
- A module file that two `mod` declarations reach,
  such as the shared modules of a library and an executable,
  reported each of its globs twice,
  keyed `glob` and `glob#2`,
  where alpha.11 reported one finding.
- A `mod.rs` that no `mod` line declares was read as its folder's index,
  so a leftover file moved that folder's keys onto itself,
  hid the code check of the folder's real index,
  and could turn the crate-root folder into a group.
- A key about a module file in the package folder itself ended in a space,
  which no list line can hold,
  and a key about a `#[path]` module outside the package held an absolute path
  where it now writes the walk from the package folder (`missing-entry ../outside x.rs`).

#### How a list written before alpha.13 maps

A tree that holds one of these shapes needs one regeneration,
and every other line stays as it is.

- A module file that two `mod` declarations reach is one module,
  so each glob re-export in it is one finding,
  keyed `glob`,
  and a `glob#2` line written for the second report is dead.
- A key about the package folder itself is written `.`:
  `no-top-doc .`,
  `missing-entry . x.rs`,
  `missing-link . -> sub`.
  A key that ended in a space could not be written on any line,
  so such a finding was never tolerated.
- A file whose name holds a space is keyed `space-in-file-name <folder> <file>`.
  The file stays in the list,
  loop and order checks,
  so its folder also reports a missing list entry for it,
  and a loop through it is reported and cannot be tolerated.
- A `mod.rs` that no `mod` line declares is no longer its folder's index:
  the folder goes back to its own index,
  or reports `no-top-doc` when it has none,
  and the keys of that folder move with it.
- A use written in a group's own file that makes its group and another entry use each other
  is a loop finding with no key,
  so a tree that holds such a use needs the use removed rather than a line recorded.
- A configuration naming `max-image-bytes` at or above 2^64
  is refused with the largest accepted value in the message,
  which costs no list line.

#### How a list written before alpha.14 maps

A tree that holds one of these shapes needs one regeneration,
and every other line stays as it is.

- A use written in a group's own file that makes its group and another entry use each other
  now carries the key `own-file-loop <group> -> <used folder>`,
  because every finding of a rule on the day it lands goes on the list.
  A tree that holds one adds that line,
  and a tree that holds none adds nothing.
  A loop that two own-file uses close is two findings,
  one for each of the two uses.
- A glob in a file that two `mod` lines declare is reported when it resolves in either or both
  of the file's module contexts,
  so a tree whose glob resolved in only one context gains a finding,
  and the answer no longer depends on which `mod` line stands first.
  `main`'s 39 Rust packages hold no such glob,
  so the lists a lock bump regenerates gain no line.
- A module file or folder whose name ends in whitespace is refused with one `core/processing-failure`,
  so the keys its findings carried are gone and the file is renamed rather than recorded.
- A top doc entry with whitespace before its closing backtick names the file without that whitespace,
  so the `missing-entry <index file> <file>` line alpha.13 needed is dead.
  A `foreign-name` or `listed-twice` line written for such an entry could never have been alive,
  because alpha.13's key for it ended in the character the list reader trims away.
- The own-file walk back runs once for each group whose own file holds a use,
  where alpha.13 ran it once for each such use.
  On a package of 2,000 folders whose one group's own file uses all of them,
  alpha.13 took about 20 seconds in each of three runs and this release about 1 second,
  while the same graph with an empty own file took about 1 second on both
  (one run of each per round,
  2026-11-26),
  so the walk now costs what reading the graph costs.

#### How a list written before alpha.15 maps

A tree that holds one of these shapes needs one regeneration,
and every other line stays as it is.

- A top doc entry whose backticks hold no name,
  or only whitespace,
  is keyed `foreign-name <index file>` or `listed-twice <index file>` without a name,
  with `#2` for a second such entry,
  where alpha.14 ended the key in the space the list reader trims away,
  so no line could hold it.
- A module file or folder whose name holds a line break is refused with one `core/processing-failure`,
  so the keys its findings carried are gone and the file is renamed rather than recorded.
- A use that joins a loop a listed `own-file-loop` line tolerates is a finding with no key,
  so a tree that holds one untangles that loop rather than recording the use;
  alpha.14 reported nothing there.
  alpha.16 records such a loop one line per use,
  so that reason to untangle is gone,
  and only a use that joins the loop after the lines are written is still a finding.
- A use written in the own file of a group nested in another group
  that closes a loop now carries the key `own-file-loop <nested group> -> <used folder>`;
  alpha.14 did not search a nested group's own file.
- The walk back reads the uses once for the package rather than once for each group whose own file holds a use,
  which costs no list line.

#### How a list written before alpha.16 maps

A tree that holds one of these shapes needs one regeneration,
and every other line stays as it is.

- A loop that a use in a group's own file closes now needs one `uses` line per use inside it,
  where alpha.15 held only the own-file line and the one shortest route its message spelled.
  A tree whose own files close such a loop gains those lines from one `--debug` run,
  and every one of them stays alive,
  so the loop is recorded rather than untangled.
  That holds for a list that holds none of the loop's own-file lines.
  A list written by alpha.15 holds them and none of the loop's uses,
  so the loop counts as tolerated and the run prints no line for its uses:
  the run fails with findings that carry no key until those `own-file-loop` lines of that loop are deleted,
  and the next `--debug` run then prints every line of the loop.
- An entry of a `uses` folder whose name ends in a trimmed character or holds a line break
  is refused with one `core/processing-failure`,
  where alpha.15 printed a two-line key that no list line could hold,
  so the entry is renamed rather than recorded.
- An `extern` block's ABI string is keyed with every run of whitespace squeezed to one space,
  so a string written over two lines gives one key on one line,
  where alpha.15 split the key over two lines and both those lines tolerated nothing.
- A list may hold four mebibytes,
  where alpha.15 read one:
  a dense loop a group's own file closes needs one line per use inside it,
  and the 60,300 lines of one measured loop are 1,681,290 bytes.
- No key of this repository's Rust packages changes shape,
  so the lock bump regenerates both committed lists with no edit
  (2,255 keys on `main` `c3b1b7efef`,
  one run of each binary,
  byte-identical streams,
  2026-11-26).

#### How a list written before alpha.17 maps

A tree that holds one of these shapes needs one regeneration,
and every other line stays as it is.

- A `uses` line is read from the two sides of its arrow,
  so a line naming a folder whose name holds a space,
  a tab or a no-break space reads back,
  where alpha.16 could not read such a line at all.
  A line whose words are set off by exactly one whitespace character each reads as the four words,
  as a line written with tabs always did,
  and a line with any other spacing is read from its arrow,
  so a run of two spaces beside a name is read as that name's own space.
  That changes what a hand-aligned alpha.16 list names:
  alpha.16 read `uses  src/a -> src/b` as the use from `src/a`,
  and alpha.17 reads it as the use from ` src/a`,
  a folder no tree holds,
  which one run reports dead,
  so such a list is re-spaced or regenerated from a `--debug` run rather than deleted.
  This repository's two lists hold no line of that shape (read on 2026-11-27).
- A `uses` or `file-uses` line may name a folder outside the package,
  written by the walk from the package folder,
  where alpha.16 refused the line although its own keys printed it.
- A folder name that holds a backslash or the words ` -> `
  is refused with one `core/processing-failure`,
  where alpha.16 printed a key that named another folder,
  so the name is changed rather than recorded.
- A loop finding that two equally short routes lead back from spells the route its backward walk reached first,
  where alpha.16 spelled the route its forward walk reached,
  so the message of such a finding names other folders and its key names the same use.

#### How a list written before alpha.18 maps

A tree that holds one of these shapes needs one `--debug` run to write its new lines,
or a renamed name,
and every other line stays as it is.

- A top doc that spells the marker nearly,
  and so has no list at all,
  reports one `near-miss-marker <index file>` finding
  (and `near-miss-marker#2` for a second one in that doc),
  beside the `no-file-list` and `missing-entry` lines it already owed.
  The new finding is added to those rather than replacing them,
  because the doc really has no list,
  and a tree whose top docs spell the marker exactly gains no line.
  `package/git-policy/cli` is such a tree:
  the built executable reports no near-miss finding there,
  while the same rules without the crate's two lists report 558 folder findings
  and 71 file findings over it,
  which those lists tolerate
  (measured on 2026-11-27 with the release build of this branch on the tree of its last merge from `main`).
- A folder name that begins with `-> `,
  ends with ` ->`,
  or is `->`
  is refused with one `core/processing-failure`,
  where alpha.17 printed a `uses` key that the next run read as another pair of folders,
  so the name is changed rather than recorded.
- A key about a file or an entry of a `uses` folder whose name holds a backslash
  keeps that backslash,
  where alpha.17 wrote `/` for it and so named a path the tree does not hold.
  A tree that holds such a name gains the line the rule prints for it,
  and an alpha.17 line for it was a line about another path.
- An entry of a `uses` folder whose name holds a backslash or the arrow words
  is no longer refused:
  its `link` or `unused-link` key holds the name as written,
  so a tree that holds one reports that finding and records the key.
- A `uses` line whose name before the arrow ends in whitespace
  is refused with a `core/processing-failure` that names the line,
  where alpha.17 read that space as part of the name and reported the entry dead.
- A dead `uses` or `file-uses` line whose folder the package does not hold
  names that folder,
  where alpha.17 said only to delete the line.
- A tie between two routes of one length now spells the route whose entries sort first,
  so a loop finding of such a tree names a different route than alpha.17 did,
  and its key,
  which names the use that closes the loop,
  is unchanged.
- A listed path is read back against the package folder and written the way a key writes it,
  so `uses ../p/src/a -> src/b` beside the package folder `p` names the use that
  `uses src/a -> src/b` names:
  that line tolerates its use where alpha.17 kept it alive and tolerated nothing,
  and a folder outside the package keeps the spelling `../outside` its keys print.
- A group whose own file closes a loop reads the reverse index of its graph once for the package
  and not once per group,
  so a package of many such groups no longer clones the graph for each of them,
  and no key changes.
  Each group's walk still read every entry that reaches it:
  with 300 folders and 100 groups whose own files each close a loop,
  every folder using a child of every group
  and each own file using the first folder,
  which reaches every folder,
  the closing arm cost 2.22 to 2.72 times alpha.18's own open arm,
  against alpha.16's 0.98 to 1.36
  (three interleaved rounds,
  each figure inside one binary and its control arm in the same round,
  on the run's own CPU time,
  2026-11-27).

#### How a list written before alpha.19 maps

No key any rule prints changes shape,
and no line a run printed reads back differently,
so a list written for alpha.18 needs no new line from this release.
A list that holds one use under two spellings needs one line fewer,
and a `file-uses` line that climbs out with `..` and comes back in stops being dead.
What else changes is which runs stop,
what they say and what a walk costs.

- A `uses` line whose name before the arrow ends in whitespace is reported dead,
  naming the folder it read,
  where alpha.18 refused the whole list with one `core/processing-failure`
  and so hid every other finding of the rule.
  The line names a folder no tree holds,
  so it is re-spaced or deleted,
  and the rest of the list is read in that same run.
  One rule now holds for the space after the word `uses`,
  for the space before the arrow and for the space after it:
  a second whitespace character beside a name belongs to that name.
- A folder name that begins with `-> `,
  ends with ` ->`,
  or is `->` stops the run only while the folder rule is selected.
  A package that selects the file rule alone reads such a tree as alpha.17 did,
  because that rule reads no `uses` line and names such a folder in a key,
  which holds the name as written.
  A folder name that holds ` -> ` between two names stops the run under the same condition,
  while alpha.17 refused that one even under the file rule alone:
  the shape it spells now reads as the other three do.
- The message of that refusal names the folder whose name is the cause,
  and the one shape that name has,
  where alpha.18 named the module file inside that folder and listed four shapes.
  A folder whose name ends in whitespace,
  holds a backslash,
  or holds a line break is named the same way.
- Two spellings of one use are one line:
  `uses src/a -> src/b` beside `uses ../p/src/a -> src/b`
  is refused as the repeat it is,
  where alpha.18 accepted both and kept both alive.
  A `file-uses` line whose folder is written that way now tolerates the use its key names,
  where alpha.18 reported it dead.
- A dead `uses` or `file-uses` line says that no source folder of the package has the name it read,
  where alpha.18 said that no folder of the package is called it,
  which was untrue of a folder on disk that holds no module file.
- A `file-uses` line for a folder whose name holds a space is refused for that cause:
  the refusal names the folder and says that the five words of the shape cannot carry a name holding one,
  where alpha.18 spelled only the shape of the line.
- A group whose own file closes a loop stops its walk as soon as every use of that group has its route,
  so an entry that reaches the group directly costs what it cost on alpha.16
  however large the graph behind it is.
  With 300 folders and 100 groups whose own files each close a loop,
  where every folder uses a child of every group and each own file uses the first folder,
  which reaches every folder,
  the closing arm costs 0.98 to 1.36 times this branch's own open arm,
  where alpha.18's costs 2.22 to 2.72 and alpha.16's 0.98 to 1.36,
  and this branch costs 0.92 to 1.13 times alpha.16's closing arm and 0.43 to 0.51 times alpha.18's;
  on the shape where each own file uses a folder that uses a child of its group back,
  it costs 0.86 to 1.00 times this branch's own open arm,
  where alpha.18's costs 1.01 to 1.35 and alpha.16's 1.03 to 1.20,
  and this branch costs 0.80 to 1.07 times alpha.16's closing arm and 0.72 to 1.04 times alpha.18's
  (three interleaved rounds,
  the three binaries rotated,
  each figure inside one binary and its control arm in the same round,
  on the run's own CPU time,
  2026-11-27),
  and no key changes.

#### How a list written before alpha.20 maps

No key any rule prints changes shape,
and no line a run printed reads back differently,
so a list written for alpha.19 needs no new line from this release.
What else changes is which lines are read,
which runs stop,
and what the messages say.

- A `uses` line is read whatever whitespace character sets its words off,
  after the word and on both sides of the arrow:
  a list aligned with tabs or with no-break spaces is read as one aligned with spaces is.
  alpha.19 read such a line in two cases only:
  when exactly one whitespace character set each of its four words off,
  whatever character that was,
  and when it held one ` -> ` and one space after the word `uses`,
  whatever its names held.
  Every other line refused the whole list,
  which hid every other finding of the rule:
  a tab or a no-break space in a gap beside a name that holds whitespace,
  and a second whitespace character in a gap where either of the two is no space.
  A gap of two spaces or of three was read,
  as the name with the extra spaces in front of it,
  which is how this release reads it too
  (measured on 2026-11-27:
  alpha.19 read `uses src/f<TAB>g -> src/h` and `uses src/r<NBSP>s -> src/t`,
  two names holding that character with gaps of spaces,
  read `uses<TAB>src/a<TAB>-><TAB>src/b` and the same line with a no-break space in every gap,
  read `uses   src/c d -> src/e` and named the folder `  src/c d`,
  the name with two spaces in front of it,
  and refused `uses<TAB>src/c d<TAB>-><TAB>src/e`,
  `uses src/c d<TAB><TAB>-><TAB>src/e`,
  `uses src/a<TAB><TAB>-> src/b` and `uses src/a <TAB>-> src/b`).
  Of the 220 spellings of one line the alpha.19 release read built,
  alpha.19 read 31 as the use,
  reported 26 of them dead,
  and refused the run for the other 163,
  while this release reads the same 31 as the use,
  reports the other 189 dead,
  and refuses none
  (measured on 2026-11-27 against the alpha.16 binary,
  which read all 220 as the use).
  A spelling whose gap is not one whitespace character names the folder the second character made,
  so its line is re-spaced rather than refused,
  which is what alpha.19 already did for a gap of two spaces.
- A gap of two whitespace characters is one separator and one character of the name beside it,
  on either side of the arrow and after the word `uses`,
  and a dead `uses` line spells that character as `\u{9}`,
  `\u{a0}` or `\u{20}`,
  where alpha.19 refused the run for a tab or a no-break space there
  and printed a space at either end of a name as itself.
- A folder name whose arrow word is set off by a tab or a no-break space
  is refused while the folder rule is selected,
  and the refusal names that shape.
  The guard is this release's wider reader:
  alpha.19 read such a tree clean and printed a key,
  and this release's reader takes the first arrow whitespace sets off for the line's own,
  so that key would read as another pair of folders.
  A tree with such a name therefore goes from a clean run in alpha.19 to a refusal that stops the rule in this release.
- A `uses` line that sets no arrow off with whitespace on both sides is refused
  with a sentence naming the whitespace characters and where they stand,
  and a line that sets two of them off says so,
  where alpha.19 sent its reader back to the shape of a line alone.
- A `file-uses` line whose words between the folder's place and the arrow name no folder
  is refused with what is wrong with it:
  a doubled arrow is named as two `->` set off by whitespace,
  a line of five words with another word in the arrow's place names that word and the arrow it must be,
  and any other such line by its word count against the five words the shape has,
  where alpha.19 told its reader to rename a folder that holds no space and is fine.
  A line whose words do name such a folder is refused by naming that folder and the whitespace in its name,
  whether that whitespace is a space,
  a tab,
  a no-break space or a doubled space,
  wherever in its gaps the characters that make the sixth word stand,
  and whether the name holds an arrow word of its own.
  The release of 0.5.2-alpha.21 named the folder only while the character after the word,
  the one before the file and the one before the arrow was one each,
  and answered an arrow-named folder with a sentence about two arrows the line does not hold.
  The release of 0.5.2-alpha.22 read the folder's own arrow as the first one whitespace set off,
  so it named a folder whose own arrow a tab,
  a no-break space or a doubled space sets off
  only while the words joined with one space named another folder the tree held,
  and answered the others with that sentence;
  it named a folder whose own name begins with whitespace only while the character beside it was one.
  This release reads the line's own arrow as the last one whitespace sets off,
  takes the whitespace a doubled gap left off one character at a time at the start and as a whole run before the file,
  names such a folder in both shapes,
  and names the folder whose own name begins with the character where the same text is the line for two folders
  (measured on 2026-11-27:
  600 of 600 lines for the three arrow spellings named the written folder against 0,
  where alpha.22 named `src/x -> y` in all 600,
  and a folder named ` src/c d` was named in 200 of its 200 lines against 72).
- A six-word line whose first folder word is absolute or holds a backslash is refused for that cause,
  as a five-word line is,
  where alpha.22 answered it by naming a folder it found on disk and telling its reader to rename it for a space.
- Every refusal of the entry reader quotes its line,
  and one that holds a character a reader cannot see names it and its place,
  so a line with two faults is answered by one message,
  where alpha.22 had three refusals that quoted no line and one that left such a character unmentioned.
- A dead `file-uses` line is told to take a character out only while that character stands
  in a file name the folder's modules do not hold:
  a line that spells a real module file's own name,
  and one whose character stands in the folder's name,
  are told to be deleted,
  which is what alpha.21 told them,
  where alpha.22 told both to edit the line.
- A sentence about what the line would read as after that edit
  is said only when the edited text does read as a line of the list,
  and a character with no plain name is named by its code point in that sentence,
  where alpha.22 said a line would read as a `uses` line when the next run refused it,
  and named only the characters it holds a plain name for.
- No key any rule prints changes shape,
  and no line a run printed reads back differently,
  so a list written for `0.5.2-alpha.22` needs no new line from this release.
  What changes is which folder a refusal names,
  what its sentences say,
  and which edit a later change fails.
- A refusal says what renaming must give and says what the name is:
  a folder whose name is `->`,
  begins with `-> `,
  ends with ` ->`,
  or holds ` -> ` is told the one rename its own shape needs,
  where alpha.19 told it to keep no arrow word at its beginning or its end and left the middle one in place,
  and an entry of a `uses` folder is named as that entry of a `uses` folder,
  where alpha.19 called it a file.

### Known limits

- An `asm!` call written where an item may sit is a macro call,
  so it is keyed `macro-call-asm!("nop")`;
  the parser gives the index check no assembly-block item.
- A macro call that names no macro is never an item:
  `!();` written where an item may sit produces no finding and no key.
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
- A folder whose name holds a space is reported and tolerated by the folder rule:
  `missing-link src/a b -> src/c` is a key line,
  and `uses src/a b -> src/c` is read from the two sides of its arrow.
  The file rule reports such a folder and no `file-uses` line can tolerate a use in it,
  because that shape is five words set off with one whitespace character each
  and a folder name holding a space makes a sixth,
  so the whole list is refused,
  and the refusal names that folder and says that the space in its name is why:
  a loop of files in such a folder is untangled or the folder is renamed.
  A folder name holding a tab,
  a no-break space or a doubled space is refused the same way,
  and the refusal spells that character and says what renaming must give.
  A six-word line whose first folder word is absolute or holds a backslash is refused for that cause,
  as a five-word line is,
  so its reader is never sent to rename a folder for a space.
  That sentence is for the words that name such a folder,
  so a six-word line whose words between the folder's place and the arrow name no folder
  is a third file name or a doubled arrow,
  and it is refused as the misspelling of the shape it is.
  Three readings of one line decide that,
  in this order:
  the name the line writes between the word and the file before its arrow,
  that name with the whitespace a doubled gap left beside it taken off,
  and the words joined with one space.
  The line's own arrow is the last one whitespace sets off,
  because a file name is one word and holds no whitespace,
  while the folder before it may hold an arrow word of its own,
  so a folder named `src/x -> y` is named by the first reading,
  and so is one whose own arrow a tab,
  a no-break space or a doubled space sets off.
  The first is the name the reader meant whenever the tree holds it,
  so a tree holding both `src/c<TAB>d` and `src/c d` still has the line for the tab folder named as that folder.
  The second reading takes the whole run of whitespace the gap before the file left,
  because no folder name the rules accept ends in whitespace,
  and at the start one character of its run at a time,
  because a folder name may begin with whitespace and only the tree says how much of the run is its own:
  so a folder named ` src/c d` is named for every gap spelling
  whose extra character can be told from that name's own whitespace.
  Where the same text is the line for two folders,
  the first reading names the folder whose own name begins with the character,
  and the third reading is the one that can name a folder the line was not written for:
  a line written for `src/c<TAB>d` beside only `src/c d` names `src/c d`,
  because the first two readings name no folder the tree holds.
  Measured on 2026-11-27 over the read's 5,600-line matrix:
  a folder named ` src/c d` was named in 200 of its 200 lines against 72 in `0.5.2-alpha.22`,
  the 128 word-count refusals among them gone;
  the three arrow spellings set off by a tab,
  a no-break space or a doubled space were named in 600 of 600 lines against 0,
  where `0.5.2-alpha.22` named `src/x -> y` in all 600;
  and of the 200 lines written for `src/c d` beside it,
  160 still named it,
  and the other 40 were the texts both folders share.
  Only a module file name of a folder is refused with `space-in-file-name`,
  and only for the space character itself:
  a tab or a no-break space inside a module file name is keyed as written,
  and no `file-uses` line can name that file either,
  because the two file names of that shape are single words.
  A root file,
  such as `src/bin/my tool.rs`,
  is no module file of a folder,
  so it gets no such finding,
  and its own keys hold its name as written,
  which a line reads.
  A name that ends in a space,
  a tab or another character the list reader trims is refused before any check,
  because no key could end in that character,
  and so is a name that holds a line break,
  because a list line ends at one,
  and a folder name that holds a backslash,
  begins with `-> `,
  ends with ` ->`,
  is `->`,
  holds ` -> `,
  or holds one of those shapes with a tab or a no-break space where they spell a space,
  because no line could name the folder such a name belongs to.
  The arrow words of a folder name are refused only while the folder rule is selected,
  so the file rule alone reads a tree that holds one of any of those shapes,
  names such a folder in a key,
  and prints a key its reader reads back.
- A macro call's key holds its whole argument text,
  so a call written over 6,000 lines gave an 87,828-character key (measured 2026-11-26),
  and any edit inside the call changes that key.
  A list file is read up to four mebibytes,
  so a key longer than that cannot be written on any line:
  a call whose arguments reach four mebibytes must move out of the index file,
  which is what the finding asks for anyway.
  Every other key's length follows the path and the names the finding holds.
- A character a reader cannot see is part of the line that holds it:
  a byte order mark before a line's first word makes that word `\u{feff}uses`,
  so the line is read as one finding's key and reported dead,
  and a zero-width space where a separator belongs does the same,
  while one inside a folder name is read as part of that name.
  The reader does not take such a character off:
  it is not whitespace,
  a line another tool does not read as a `uses` line would start reading as one,
  and a key a macro prints may hold one as its own content.
  Every message about such a character spells it,
  names it,
  and says where it stands,
  counted in characters,
  which are Unicode scalar values,
  from the line's first one:
  an editor may draw a tab as several cells or write one character as more than one byte,
  so the column it shows can differ,
  and no message says which column an editor shows.
  A character with no plain name is named by its code point in the same sentence,
  where the quote spells it.
  A line whose only fault is that character is told to take it out:
  a dead line the reader took for one finding's key,
  a line the reader refuses,
  and a dead `file-uses` line whose file name holds one that no module of its folder holds.
  A line that spells a real module file's own name,
  and one whose character stands in the folder's name rather than a file's,
  is told to be deleted,
  because taking the character out would name another file or folder.
  A whitespace character where one belongs replaces it,
  and a sentence about what the line would read as after that edit
  is said only when the edited text does read as a line of the list.

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
A link that points into a loop of folders makes `cargo package --list` walk without end:
the wrapper measured 1.9 seconds over 3038 paths with its 109 links,
and no answer after 120 seconds with 421 added
(`doc/planning/cli-git-source-layout.md`,
the paragraph under section "The `uses/` links and cargo"
that starts "Measured again on 2026-11-25"),
which is why
the decision record (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`)
writes a folder's links only once its uses no longer loop,
and why the help of a missing-link finding carries that warning before the command it gives.
The layout rules themselves find modules by following `mod` lines and read only the names of the entries of a
`uses` folder,
never the files behind them.

`mise run //package/linter/monochromatic-lint:test:layout` runs the layout tests alone;
`test:layout:container` runs them in the bounded container.

### Where the rules run for `package/git-policy/cli`

The package's own `lint:monochromatic-lint` task runs the repository linter over `package/git-policy/cli`
with the root `monochromatic-lint.config.jsonc`,
so `mise run //package/git-policy/cli:lint` and the repository's `mise run lint`
check whatever that configuration selects.
It selects the layout rules for `package/zfs/request-to-openzfs-kernel-device`,
and its `layout-git-policy-cli` block selects them for `package/git-policy/cli`
with the crate's two lists,
so both commands check the crate's per-file rules and its layout.
The block came from branch `chore/git-policy-cli-layout-rules-on-with-a-shrinking-allow-list`,
which merged on 2026-11-26 (`a1b9b1e850`,
2026-11-26T00:17:50-05:00).
The package's container gate (`native:test:container`) runs neither rule,
because it runs Clippy and the test suite and never the repository linter,
so a round that only runs the gate does not meet the layout rules;
`mise run //package/git-policy/cli:lint:monochromatic-lint` is the command that does.
That task runs the installed `cargo:monochromatic-lint`,
and a list must be read by a release at least as new as the one that wrote it:
a release newer than the list prints keys the list cannot hold,
and a release older than the list matches no key the newer shape changed.
A list regenerated for `0.5.2-alpha.10`,
for example,
leaves 1,297 dead entries under `0.5.2-alpha.9`.
So `toolchain.lock.json` and the two lists move together,
and a release that changes a key shape regenerates the lists in its own round.

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
The root `.gitattributes` and `.lfsconfig` are read up to one mebibyte,
and a tracked image whose object id the rule computes up to the rule's `max-image-bytes` option,
which is sixteen mebibytes unless configuration names another bound,
so a mistyped path cannot read a device or a disk image into memory;
this repository's own `.gitattributes` is 774 bytes,
its `.lfsconfig` 61 bytes and its largest tracked image 3,380,183 bytes (measured 2026-11-25),
about a fifth of the default.
An image over the bound is a `core/processing-failure` finding that names the file
and says to raise `max-image-bytes`:

```jsonc
// monochromatic-lint.config.jsonc
{
  "files": ["**/*.md"],
  "rules": {
    "markdown/lfs-image-url": { "severity": "error", "max-image-bytes": 33554432 },
  },
}
```

The bound is an option rather than a constant of this crate,
because the rule ships inside a published binary and its user cannot edit that source;
raising the option in the repository's own configuration is what a repository with a larger image does.
The alternative,
naming the image and skipping it,
was rejected because a skipped image's object URL is never checked or rewritten,
so a reader who needs the rule's coverage would have to exclude the whole Markdown file from it.

The largest bound the option takes is 18446744073709551614,
one less than the largest value of the unsigned 64-bit word it is read under.
The reader reads one byte past the bound,
so it can tell an image of exactly the bound from one that is over it,
and a bound of 18446744073709551615 has no byte past it.
A configuration naming a larger bound is refused when it loads,
with a message naming `max-image-bytes` and that largest value,
and the run stops before any file is read.
The other repair,
reading the bound without the extra byte,
was rejected because the extra byte is what refuses an image over the bound:
without it the rule would hash the first bound's worth of a larger file and rewrite the link to the wrong object id.
Refusing the value is also what a release build needs:
with overflow checks off,
the extra byte wraps that bound to zero,
the rule hashes empty input,
and `--fix-content` rewrites the link to the empty file's object id.

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

## Splitting a source file

`rust/max-lines` allows 300 code lines per `.rs` file,
so a file that grows past that is split by role into sibling modules in `src/`,
declared in `src/lib.rs` beside it.
The parts keep every item at the path callers already write:
a caller names `crate::markdown_source::MarkdownSource` and calls its methods
wherever the `impl` block that holds them lives,
and a function that moves while callers still name the old file is re-exported
(`pub use crate::closure_step_package::ClosureOutcome;`).
A file that keeps its items and gains siblings names them in its top doc.

A private field of a struct is visible only in the module that declares it and
in that module's children,
so the `impl` block of a struct cannot read its fields from a sibling file
without a change.
The splits of 2026-11-26 and 2026-11-27 widened the fields and helper methods
the moved `impl` reads to `pub(crate)`,
which is how `src/markdown_traversal.rs` already hands `Traversal` to
`src/markdown_source.rs`,
and left every module a crate-root sibling,
because the mutation scopes in `bin/mutation-scopes.ts` select files by
`src/<name>_*.rs` globs that a file in a subfolder would leave.
A module inside a folder named after the origin keeps those fields private,
and is the other way to split such a type.

A constructor stays beside the type it builds,
even when the file that reads its input moves out.
`parse_allow_list` fills the private indexes of `AllowList`,
so it lives in `src/layout_allow_list.rs` while `read_allow_list` moved to
`src/layout_allow_list_reading.rs`,
and every field of the type stays private to the module that declares it.

A re-export is what the callers name,
so what only the test sidecar names is re-exported under `#[cfg(test)]`:
`src/layout_allow_list.rs` re-exports `MAX_BYTES` and `read_at_most` that way,
because a release build imports neither and a plain re-export would be an unused import there.
A work counter that the sidecar names (`super::counted_work`) stays in the origin file for the same reason.

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
