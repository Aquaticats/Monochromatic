# monochromatic-lint

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
`doc/handover/cli-git-rust-implementation.md`.
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
- `--fix` applies fixes in at most ten passes per file and rechecks the result.
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
   with a message on standard error,
   when a prerequisite is missing (see "Prerequisite manifest").
2. It runs `cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked --message-format json`
   with `CARGO_NET_OFFLINE=true` and `DYLINT_RUSTFLAGS` set to request `avoidable_closure` and
   `unfulfilled_lint_expectations` as warnings.
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
every package is skipped with one message:

```text
monochromatic-lint: rust/avoidable-closure skipped all 24 package(s): ...
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
A skip is not a finding:
the exit status is unaffected,
`--silent` keeps the message,
and `--max-warnings` does not count it.

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
  The linter builds in `target/dylint` and links both artifacts into it,
  because the IDE's `build.rs` finds them relative to the build directory.
- `android-ndk`:
  `true`;
  `cargo ndk-env --target arm64-v8a` must print the NDK variables,
  which are set for the build,
  and the check compiles for `aarch64-linux-android`.

A missing prerequisite skips the package with a message that names it.
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
The repository root is the nearest ancestor holding `.lfsconfig`;
tracked paths come from the root `.gitattributes`.
The `exclude` option takes gitignore-syntax patterns relative to that root.

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
`mutation:semantic` for the semantic engine and the explicit-type rule;
and `mutation:processors:files` for the processor modules.
The executable and core scopes skip the five tests that load a Cargo workspace,
which the semantic scope and the full container tests run.
The semantic scope has its own per-mutant limit,
and any campaign accepts `-- --shard k/n` (zero-based,
as in cargo-mutants) so that its shards can run in separate bounded containers at once.
`mutation:coverage` compares the scopes' listings with the unscoped listing,
name by name,
and fails when any mutant is outside every scope.
A campaign with a missed or timed-out mutant exits nonzero and is not a passing gate.

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
A package-wide `lint:oxlint` also lints `task.ts`, which no `tsconfig.json` includes:
it does not satisfy the isolated-declarations setting of the shared TypeScript preset,
so adding it to the project fails the type check.
Measured on 2026-10-30, that run exits 0 but the semantic bridge reports `api-unavailable` for six files
and the type-aware rules (`prefer-readonly-parameter-type`, for one) throw instead of checking,
so a green package-wide run would not mean the scripts passed those rules.
Making the package-wide task usable needs `task.ts` to be fixed or excluded, not a looser rule.

## License

LGPL-3.0-or-later.
License texts are in `LICENSES/`.
