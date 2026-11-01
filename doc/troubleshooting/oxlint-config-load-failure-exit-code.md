# oxlint 1.85.0: a configuration or JS plugin load failure exits 1 on stdout, the same status as "lint found errors"

When oxlint cannot load its configuration
(for example a `jsPlugins` entry whose module imports a package that does not resolve),
it prints a report on stdout and exits 1.
A run that lints successfully and finds errors also exits 1 on stdout.
Nothing in the exit status,
the stream,
or `--format json` output tells a caller that no file was linted.

Found while investigating
[issue #570](https://github.com/Aquaticat/Monochromatic/issues/570)
(`doc/handover/config-oxlint-stale-plugin-bundle-issue-570.md`),
where a stale `config-oxlint` sidecar kept a bare import of an uninstalled package
and every lint run failed this way while looking like an ordinary failing lint.

## Symptom

Verbatim stdout of a run whose only JS plugin imports a missing package,
with an empty stderr and exit status 1:

```text
Failed to parse oxlint configuration file.

  x Failed to load JS plugin: ./plugin.mjs
  |   Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'nonexistent-package-570' imported from <DIR>/plugin-fail/plugin.mjs
  |     at Object.getPackageJSONURL (node:internal/modules/package_json_reader:343:9)
```

The same text appears unchanged under `--format json`,
`--format github`,
and `--format unix`:
the formatter never runs.

Other headers that take the same path,
all exit 1 on stdout
(`apps/oxlint/src/lint.rs` at tag `oxlint_v1.85.0`):

- `Failed to setup JS workspace:` (line 241)
- `Failed to parse oxlint configuration file.` (lines 277,
   309, 338)
- `Failed to parse oxlint configuration file at <path>.` (line 287)
- `Failed to build configuration from <path>.` (line 294)
- `Failed to build configuration.` (line 357)

Consequence in this repository:
the lint wrapper
(`package/monorepo-task/core-exec/src/oxlint-wrapper.ts`,
 `finalizeResult`)
keeps oxlint's exit code,
so a caller that greps the output for rule diagnostics sees none
and can read the run as clean.

## Root cause

### Every non-success result maps to one exit code

`apps/oxlint/src/result.rs:27-51` collapses all failure variants,
whether linting ran or not,
into `ExitCode::FAILURE`:

```rust
// apps/oxlint/src/result.rs:27-51 (oxlint_v1.85.0)
impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None
            | Self::PrintConfigResult
            | Self::ConfigFileInitSucceeded
            | Self::LintSucceeded => ExitCode::SUCCESS,
            Self::ConfigFileInitFailed
            | Self::JsPluginWorkspaceSetupFailed
            | Self::LintFoundErrors
            | Self::LintNoFilesFound
            | Self::LintNoWarningsAllowed
            | Self::LintMaxWarningsExceeded
            | Self::InvalidOptionConfig
            // ... every other InvalidOption* variant ...
            | Self::LintUnprunedSuppressions
            | Self::TsGoLintError => ExitCode::FAILURE,
        }
    }
}
```

The enum already distinguishes `InvalidOptionConfig` from `LintFoundErrors`;
only the mapping discards the difference.

### The npm package narrows the result to a boolean

The `oxlint` npm package (the only build that runs JS plugins) calls Rust through N-API,
and that entry point returns `bool`:

```rust
// apps/oxlint/src/run.rs:151-172 (oxlint_v1.85.0)
pub async fn lint(
    args: Vec<String>,
    // ... callbacks ...
) -> bool {
    lint_impl(/* ... */)
    .await
    .report()
        == ExitCode::SUCCESS
}
```

```ts
// apps/oxlint/src-js/cli.ts:188-201 (oxlint_v1.85.0)
const success = await lint(
  args,
  // ... wrappers ...
);
if (!success) process.exitCode = 1;
```

So even a richer `Termination` mapping would not reach npm users
without widening this return type.

### Config failures bypass the output formatter and go to stdout

`CliRunner::run` takes a single writer,
named `stdout`
(`apps/oxlint/src/lint.rs:78`:
 `pub fn run(self, stdout: &mut dyn Write) -> CliRunResult`);
the file never writes to stderr.
The config-load branch prints a graphical report through that writer
and returns before any formatter is consulted:

```rust
// apps/oxlint/src/lint.rs:269-281 and 316-317 (oxlint_v1.85.0)
let (mut root_config, nested_configs, nested_ignore_patterns) = match config_result {
    Ok(loaded) => (loaded.root, loaded.nested, loaded.nested_ignore_patterns),
    Err(error) => {
        match error {
            CliConfigLoadError::RootConfig(error) => {
                print_and_flush_stdout(
                    stdout,
                    &format!(
                        "Failed to parse oxlint configuration file.\n{}\n",
                        render_report(&handler, &error)
                    ),
                );
            }
            // ...
        }
        return CliRunResult::InvalidOptionConfig;
```

The `Failed to load JS plugin:` line itself comes from
`crates/oxc_linter/src/config/config_builder.rs:812-815`
(`ConfigBuilderError::PluginLoadFailed`).

### Upstream pins the current behaviour in a snapshot

`apps/oxlint/test/fixtures/import_error/output.snap.md` records exit code `1`,
the report on stdout,
and an empty stderr for a plugin that throws on import.
That is a regression snapshot of current output,
not a documented contract:
the oxc website linter docs
(`oxc-project/website` at `d72b239`,
 `src/docs/guide/usage/linter/`)
mention exit codes only for `--deny-warnings`,
`--max-warnings`,
and `--no-error-on-unmatched-pattern`.

### Comparison: ESLint documents a separate status

ESLint's CLI reference
(`eslint/eslint` `docs/src/use/command-line-interface.md:1037-1043`)
reserves status `2` for this case:

```md
- `1`: Linting was successful and there is at least one linting error, ...
- `2`: Linting was unsuccessful due to a configuration problem or an internal error.
```

oxlint positions itself as an ESLint replacement,
and a test comment in its own tree expects a status other than 1
for one failure kind that currently exits 1
(`apps/oxlint/src/lint.rs:2232`:
 `// When there are stale suppressions, exit code should be 2`).

### Not checked

oxlint's default format switches to `agent` when it detects an agent session
(`apps/oxlint/src/command/lint.rs:370-381`,
 `default_output_format`),
which is why an unflagged run inside Claude Code prints the one-line agent format.
That changes finding output only;
config failures are identical in every format.

## Verification

Version under test:
oxlint 1.85.0 from this repository's `node_modules/.bin/oxlint`
(`oxlint --version` prints `Version: 1.85.0`),
source read at tag `oxlint_v1.85.0` (`288d8cc77984b0a3851c58c423ffe9e6edc79f2e`)
and compared with `main` at `f51e67812ed15cea95c9bddfe614d889685f0703`,
where `result.rs` and the `cli.ts` exit line are unchanged.

Harness
(three throwaway directories,
each with `.oxlintrc.json` and `a.js`):

```sh
# plugin-fail/plugin.mjs
import 'nonexistent-package-570';
export default { meta: { name: 'p570' }, rules: {} };
# plugin-fail/.oxlintrc.json
{ "jsPlugins": ["./plugin.mjs"], "rules": { "no-debugger": "error" } }
# lint-error/.oxlintrc.json and clean/.oxlintrc.json
{ "rules": { "no-debugger": "error" } }
# plugin-fail/a.js and lint-error/a.js contain `debugger;`; clean/a.js contains `export const x = 1;`

cd plugin-fail && oxlint -c .oxlintrc.json --format json a.js; echo "exit=$?"
```

### Results

- `plugin-fail`,
  every format (default,
   `json`,
   `github`,
   `unix`):
  exit 1,
  the `Failed to parse oxlint configuration file.` block on stdout,
  empty stderr.
- `lint-error`,
  every format:
  exit 1,
  the `eslint(no-debugger)` diagnostic on stdout
  (valid JSON under `--format json`),
  empty stderr.
- `clean`,
  every format:
  exit 0.

The positive control (`lint-error`) and the failure (`plugin-fail`) share status and stream,
which is the defect.

## Verified workarounds

### Classify by parsing `--format json` output

A run that linted prints a JSON object;
a run that could not load its configuration does not.

```ts
// classify.ts: run with `node classify.ts`
import { spawnSync } from 'node:child_process';

type Outcome = 'clean' | 'findings' | 'could-not-lint';

function classify(cwd: string): Outcome {
  const result = spawnSync('oxlint', ['-c', '.oxlintrc.json', '--format', 'json', 'a.js'], { cwd, encoding: 'utf8' });
  if (result.status === 0) return 'clean';
  try {
    const report = JSON.parse(result.stdout) as { diagnostics: unknown[] };
    return report.diagnostics.length > 0 ? 'findings' : 'could-not-lint';
  } catch {
    return 'could-not-lint';
  }
}
```

Measured output on the harness:
`plugin-fail could-not-lint`,
`lint-error findings`,
`clean clean`.

Tradeoffs:
the caller must own the format,
which conflicts with this repository's wrapper pinning `--format=default`
for its diagnostic augmentation.
Every non-JSON failure,
including tsgolint crashes,
lands in `could-not-lint`,
which is the intended grouping but not a finer diagnosis.

### Require the default-format summary trailer

Under `--format default`,
a run that linted always ends with `Found <n> warnings and <m> errors.`;
the config-failure path returns before printing it.
Verified:
`lint-error` prints `Found 0 warnings and 1 error.`,
`plugin-fail` output has no `Found` line
(`rg --count 'Found \d+ warning'` exits 1).

Tradeoffs:
couples the wrapper to human-readable wording that upstream can change without notice,
and does not apply to other formats.
It is the lower-friction option for `oxlint-wrapper.ts`,
which already consumes default-format text.

### Match the failure headers

Treat stdout that starts with one of the headers under "Symptom" as "could not lint",
and for `Failed to load JS plugin` with `ERR_MODULE_NOT_FOUND`
print repository guidance (install,
 then rebuild the named package).

Tradeoffs:
the list of headers is taken from source and will drift;
an unlisted header falls back to "findings".
Use it for guidance text,
not as the only classifier.

## What does not work

- `--format json` alone:
  the config failure is still plain text and still exits 1;
  only the parse failure distinguishes it.
- Reading stderr:
  oxlint writes nothing there on this path.
- The standalone Rust binary:
  it does not run JS plugins at all,
  a separate upstream problem
  ([oxc-project/oxc#25203](https://github.com/oxc-project/oxc/issues/25203)).

## Upstream filing decision

### Out-of-scope check

`.out-of-scope/` has no entry for oxlint or exit codes
(listing checked 2026-09-25).

### Duplicate search

`gh search issues --repo oxc-project/oxc` for
`exit code`,
`exit codes`,
`exit code config`,
`exit code 2 configuration`,
`stderr oxlint`,
`Failed to parse oxlint configuration file`,
and `Failed to load JS plugin`,
plus `gh search prs` for `exit code`.
No issue or PR asks to separate configuration failures from lint findings.
Neighbours,
none a duplicate:
[#20991](https://github.com/oxc-project/oxc/issues/20991) (`--pass-on-unpruned-suppressions`,
 exit-code policy for suppressions),
[#25203](https://github.com/oxc-project/oxc/issues/25203) (standalone binary ignores `jsPlugins` and exits 0),
[#22270](https://github.com/oxc-project/oxc/issues/22270) (exits without diagnostics after a parser nesting error),
[#26343](https://github.com/oxc-project/oxc/issues/26343) (`--rules` prints nothing and exits 0),
[#23686](https://github.com/oxc-project/oxc/issues/23686) (type-aware fails silently without tsgolint,
 closed).

### Constraints

1.  Upstream's fault:
    yes.
    The distinction exists in `CliRunResult` and is discarded by `result.rs:27-51`
    and by the `bool` N-API return (`run.rs:151-172`).
    The stdout stream is a design choice shared by every oxlint message,
    so the report targets the exit code,
     not the stream.
2.  Can upstream fix it:
    yes;
    the prototype below is a local change to three files plus snapshots.
3.  Supported use case:
    soft yes.
    oxlint ships machine formats (`json`,
     `sarif`,
     `junit`,
     `github`,
     `agent`) for CI consumers
    and advertises ESLint compatibility;
    ESLint documents status 2 for exactly this case.
    No oxc doc promises status semantics either way.
4.  Contributions welcome:
    yes with disclosure.
    `CONTRIBUTING.md:12-21` and `AGENTS.md:7-15` in oxc-project/oxc
    ask for AI-usage disclosure and human review,
    and close unreviewed AI content;
    no ban.
5.  Likely to fix:
    no negative signal found.
    The `import_error` snapshot pins status 1 but as a regression snapshot,
     not a stated non-goal;
    the `lint.rs:2232` test comment points toward distinct statuses.
    Changing status 1 to 2 is observable to callers that test `== 1`,
    so maintainers may want it behind a release note.
6.  Prototype:
    see "Prototype".

### Prototype

Disposable clone of `oxc-project/oxc` at `oxlint_v1.85.0`
(`288d8cc77984b0a3851c58c423ffe9e6edc79f2e`,
 push URL disabled).
Diff:
[`oxlint-config-load-failure-exit-code.patch`](oxlint-config-load-failure-exit-code.patch).
It also applies cleanly (`git apply --check`) to oxc `main` at `f51e67812ed15cea95c9bddfe614d889685f0703`.

The change adds `CliRunResult::exit_code()`
(`0` success;
 `1` linting ran and reported problems;
 `2` linting could not run),
makes `Termination::report` use it,
returns that number from the N-API `lint`,
and sets `process.exitCode` from it in `cli.ts`.
Upstream snapshots that record `# Exit code\n1` for config failures
(for example `apps/oxlint/test/fixtures/import_error/output.snap.md`)
would need regenerating;
the prototype does not touch them.

Build,
secret-free and bounded
(only the disposable clone mounted,
 2 CPUs,
 2 GiB):

```sh
podman run --rm --memory=2g --cpus=2 --volume "${PWD}:/work:Z" --workdir /work/oxc \
  --env CARGO_HOME=/work/cargo-home --env CARGO_BUILD_JOBS=2 \
  --env CARGO_PROFILE_DEV_DEBUG=0 --env CARGO_INCREMENTAL=0 \
  docker.io/library/rust:latest cargo build -p oxlint --lib
# Finished `dev` profile [unoptimized] target(s) in 4m 18s
```

Harness:
a copy of the installed `oxlint` 1.85.0 npm package
with `target/debug/liboxlint.so` placed at `dist/oxlint.linux-x64-gnu.node`
(the first path `dist/bindings.js` tries)
and the `cli.ts` change applied to the built `dist/cli.js` line 83,
run against the three harness directories in every format.

- Pre-patch (installed 1.85.0):
  `plugin-fail` exit 1,
  `lint-error` exit 1,
  `clean` exit 0.
- Post-patch:
  `plugin-fail` exit 2 in all four formats,
  `lint-error` exit 1 in all four formats,
  `clean` exit 0 in all four formats;
  stdout content unchanged.

Not run:
`cargo clippy`,
upstream `just` recipes,
and the snapshot suite
(it would fail on the intended exit-code change until regenerated).
The generated doc comment in `bindings.d.ts` still says it returns `true`/`false`;
a real N-API build regenerates it from the Rust doc comment.

### Decision

Not filed by this session (the task was read-only research).
All six constraints hold,
so the draft is fileable once a human has reviewed the reproduction and prototype,
as oxc's AI policy requires,
and accepts that the change is observable to callers testing `== 1`.

### Draft upstream issue (do not file as-is)

~~~md
Title: linter: configuration and JS plugin load failures exit 1, indistinguishable from lint errors

Labels: A-linter, C-bug

AI assistance disclosure: this report and prototype were prepared with an AI assistant;
the reproduction, source trace, and prototype verification below were run and reviewed by me.

## Summary

When oxlint cannot load its configuration (for example a `jsPlugins` entry whose module
throws `ERR_MODULE_NOT_FOUND`), it prints `Failed to parse oxlint configuration file.` on
stdout and exits 1. A run that lints and finds errors also exits 1 on stdout. `--format json`
does not change the config-failure output. CI scripts and wrappers therefore cannot tell
"no file was linted" from "lint found errors" without parsing text.

ESLint documents exit code 2 for "Linting was unsuccessful due to a configuration problem or
an internal error".

## Reproduction (oxlint 1.85.0, npm package)

plugin.mjs:
    import 'nonexistent-package-570';
    export default { meta: { name: 'p570' }, rules: {} };
.oxlintrc.json:
    { "jsPlugins": ["./plugin.mjs"], "rules": { "no-debugger": "error" } }
a.js:
    debugger;

    oxlint -c .oxlintrc.json --format json a.js; echo $?
    # stdout: "Failed to parse oxlint configuration file. ... Failed to load JS plugin ..."
    # exit: 1
Remove `jsPlugins` and the same command prints JSON with one diagnostic and also exits 1.

## Root cause

- `apps/oxlint/src/result.rs` `Termination::report` maps `InvalidOptionConfig`,
  `JsPluginWorkspaceSetupFailed`, `TsGoLintError` and `LintFoundErrors` alike to `ExitCode::FAILURE`.
- `apps/oxlint/src/run.rs` N-API `lint` returns `bool` (`report() == ExitCode::SUCCESS`),
  and `apps/oxlint/src-js/cli.ts` sets `process.exitCode = 1` on `false`.

## Suggested fix

Add `CliRunResult::exit_code() -> u8` (0 success, 1 lint problems, 2 could not lint), use it
in `Termination::report`, return it from N-API `lint`, and set `process.exitCode` from it in
`cli.ts`. Patch and verification attached. Snapshots under `apps/oxlint/test/fixtures/*` that
record exit code 1 for config failures need regenerating. This changes observable behaviour
for callers that test `== 1`, so it may warrant a release note.
~~~

## Related

- `doc/troubleshooting/oxlint-sweep-output-and-timing.md` Bug 3
  (exit status 1 means findings,
  so `&&` chains skip post-processing).
- `doc/troubleshooting/rolldown-unresolved-import-external.md`
  (the bundler half of issue #570).
