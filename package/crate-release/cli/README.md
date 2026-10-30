# @monochromatic-dev/crate-release-cli

Product-neutral release helpers for Rust crates that are published to crates.io by a GitHub Actions workflow and
ship tarballs as GitHub release assets.
Anything specific to one crate (its name, directory, commit scope, fuzz crate, lock task) is an option here,
never a constant.
A product keeps its values in a mise task of its own, for example `native:release` in the git-policy package.

The package is private and has no build output: its two executables run as TypeScript, and tests import the sources
directly.

## Why this location and shape

- `package/crate-release/cli` follows the layout rule `package/<category>/<name>` with singular segments;
  the package name is `@monochromatic-dev/crate-release-cli`.
  The category is the activity (releasing crates), not a product, because the code serves every crate that publishes
  through `.github/workflows/cargo-publish.yml`.
- The sibling `package/mutation-test/cli-cargo-mutants` is the precedent for product-neutral tooling that products wrap
  in their own mise task, so configuration is command-line options, not a config file.
  A config file would add a second place where a crate's name and paths live next to the task that already names them.
- The package reuses `shellQuote` and the character-set helpers of `@monochromatic-dev/mutation-test-cli-cargo-mutants`
  through its `./ts/*` subpath.
  That makes a release tool depend on a mutation-test package; moving those helpers to a shared package of their own
  would remove the odd direction and is a change for all users at once.

## Release command

```bash
# mise task: any crate, with its own values (see the git-policy example below)
mise run //package/crate-release/cli:release -- \
  --crate-name <name> --crate-dir <dir> --scope <scope> \
  [--pin-file <Cargo.toml>]... [--check-bin <name>]... [--lock-task <task>]... \
  <from> <to> <note>
```

The command moves a crate from `<from>` to `<to>` and records the bump as a local commit.
It never pushes and never publishes: the push that triggers the publish workflow is a separate, deliberate step.
In order, it

1. edits the `version` of the `[package]` table in `<crate-dir>/Cargo.toml`;
2. edits the exact pin (`version = "=<from>"`) of the crate in each `--pin-file` manifest;
3. runs `cargo check --offline` (with one `--bin` per `--check-bin`) in the crate directory;
4. runs `mise run <task>` for each `--lock-task`, in order, at the repository root;
5. runs `cargo publish --dry-run --locked --allow-dirty` in a capped transient `systemd-run --user --wait --pipe` unit;
6. commits the manifests and the `Cargo.lock` beside each of them, with the message
   `chore(<scope>): release <to>: <note>`.

Every edit is computed before the first file is written, so a refused manifest leaves all files as they were.
A manifest that already holds `<to>` is reported and not rewritten, which makes a run that failed at a later step safe
to run again.
The first failing step stops the run with `FAILED <step> (<status>)` and exit status 1.
Each step is printed as `== <step>` and a `$ ...` line that can be pasted into a shell.

### What the edits accept

- The crate version is the `version` key of the `[package]` table only.
  A `rust-version` key, a `version` key in any other table and a dependency's version are never touched.
- The current value must equal `<from>` exactly, or `<to>` (no change).
  Any other value, including a longer one that starts with `<from>` such as `0.4.21` for `0.4.2`, is refused with both
  values named.
- A pin is the inline-table dependency `<crate> = { ..., version = "=<version>" }`.
  A loose requirement such as `"0.4.2"` is refused, and a crate whose name only starts with this crate's name is not
  matched.
- Notes are one line without control characters, at most 200 characters.
  Quotes, `$`, `;`, backticks and `--looks-like-an-option` are kept verbatim, because the message reaches Git as one
  argument and no shell reads it.
  Put a lone `--` before a note that starts with a hyphen.

### Options

Run the command without arguments, or with `--help`, for the same list with defaults.

- `--crate-name`, `--crate-dir` and `--scope` are required: the crate name, its repository-relative directory, and the
  scope of the commit subject.
- `--pin-file <Cargo.toml>`, repeatable, names a manifest that pins the crate exactly, such as a fuzz crate.
  The `Cargo.lock` in the same directory is committed with it.
- `--check-bin <name>`, repeatable, narrows the offline check to those binaries.
- `--lock-task <task>`, repeatable, names a mise task that refreshes lockfiles after the edit.
- `--memory-max <value>` (default `12G`) and `--unit-name <name>` (default `<crate-name>-publish-dry`) shape the
  dry-run unit.
- `--root <dir>` overrides the repository root, which is otherwise found by walking up from the package to the nearest
  `pnpm-workspace.yaml`.

## Wait command

```bash
mise run //package/crate-release/cli:release:wait -- --crate-name <name> [options] <sha> <version>
```

The command waits for the publish workflow run of commit `<sha>` (a prefix of at least seven hexadecimal digits),
watches it, then checks crates.io for `<version>` and the GitHub release for the expected assets.
It prints

```text
run 40991272842
watch exit 0
crates.io 0.4.3: 2026-10-30T02:26:00.989331Z
release monochromatic-git-policy-cli-v0.4.3 assets: ...x86_64-unknown-linux-gnu.tar.gz, ...
PUBLISHED
```

and exits 0 only when the workflow succeeded, crates.io has the version and the release holds every expected asset.
Otherwise it ends with `PUBLISH CHECK FAILED: ...` naming each reason, and exits 1.
A failed workflow run does not skip the checks, so the report shows everything that is and is not in place.
When no run appears it prints the bound that was waited, for example `within 15 minutes`, and the last answer it got.

The runs are listed as JSON and filtered in code; the hash is never placed in a `gh --jq` program.

### Options

- `--crate-name` is required.
- `--workflow <file>` (default `cargo-publish.yml`) and `--repo <owner/name>` (default: read by `gh` from the working
  directory, which is the repository root).
- `--attempts <count>` (default 45), `--poll-interval <seconds>` (default 20) and `--run-limit <count>` (default 10)
  bound the wait and each listing; the defaults wait at most 15 minutes.
- `--tag-template <text>` (default `{crate}-v{version}`) and `--asset-template <text>`, repeatable (default the x86_64
  and aarch64 Linux tarballs, `{crate}-{version}-<target>.tar.gz`).
  Templates must hold `{version}`.
  Add `--asset-template '{crate}-{version}-sha256sums.txt'` to require the checksums file as well.
- `--user-agent <text>` (default `crate-release-check`) and `--max-time <seconds>` (default 30) shape the crates.io
  request.
- `--root <dir>` as for the release command.

## Example: the git-policy crate

```bash
mise run //package/git-policy/cli:native:release -- 0.4.2 0.4.3 "fix the thing"
git push
mise run //package/git-policy/cli:native:release:wait -- <sha> 0.4.3
```

The tasks in `package/git-policy/cli/mise.toml` supply the crate name, directory, scope, the fuzz crate pin, the
`cli-git-native` binary and the fuzz crate lock task.

## Source layout

- `bin/release.ts` and `bin/wait-publish.ts` only read the process state and spawn; they hold no logic.
- `src/` holds the pure builders and parsers, each with a unit test: version and name validation (`version.ts`,
  `validate.ts`), the manifest edits (`manifest-edit.ts`, `toml-line.ts`), the commit message (`commit-message.ts`),
  the program arguments (`release-steps.ts`, `service-arguments.ts`), the result parsing and formatting
  (`publish-results.ts`), and the option parsers (`release-options.ts`, `wait-options.ts`).
- `release-run.ts` and `wait-run.ts` orchestrate through injected readers, writers, program runners and sleepers, so
  tests drive them without touching a file, a network or a real program.
  `cli.unit.test.ts` starts the real executables with stand-in `cargo`, `git`, `gh`, `curl`, `mise` and `systemd-run`
  programs first on the search path, in a disposable directory under `<home>/temp/agent`.
- No program is started through a shell.
  The only shell-facing text is the `$ ...` line printed for people, and it is built with `shellQuote`.

## Tasks

- `release` runs the release command and `release:wait` runs the wait command.
- `lint`, `lint:types`, `lint:oxlint`, `format:oxlint` and `test:unit` are the usual package checks.
