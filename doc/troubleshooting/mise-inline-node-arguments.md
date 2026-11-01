# Mise 2026.9.5 appends task arguments into Node inline code

## Symptom

Appending source paths to the translation package's `format:oxlint` task failed before Oxlint ran.
Node 26.7.0 emitted:

```text
SyntaxError [ERR_INVALID_TYPESCRIPT_SYNTAX]: Expected ';', '}' or <eof>
```

The evaluated text ended with:

```text
runWorkspaceNode('package/monorepo-task/core-exec', 'oxlint-wrapper', ['--type-aware', '--fix']) src/pair-blocks-stage.ts src/pair-blocks-stage.unit.test.ts
```

This was an invocation mistake,
not an Oxlint finding or evidence that the edited TypeScript failed to parse.

## Root cause

`mise.no-env.toml:588` defines the inherited formatter template without an argument specification:

```toml
# mise.no-env.toml
[task_templates."format:oxlint"]
description = "Auto-fix this package with Oxlint using its package-local tsconfig"
shell = "node --input-type=module-typescript -e"
run = """
{{vars.dispatch_workspace_node}}
ensureOxlintConfig()
runWorkspaceNode('package/monorepo-task/core-exec', 'oxlint-wrapper', ['--type-aware', '--fix'])
"""
```

Mise release `v2026.9.5`,
commit `016fcd16a991c85e099d4f0b571bc44978a9eb94`,
implements Unix inline argument forwarding in upstream `src/task/task_executor.rs:1373`:

```rust
// src/task/task_executor.rs
let mut script = script.to_string();
if !args.is_empty() {
    script = format!("{script} {}", shell_words::join(args));
}
full_args.push(script);
```

The forwarded paths therefore become part of the string Node parses as TypeScript,
not arguments handled by `runWorkspaceNode`.
Mise's [argument documentation][arguments] links to
[inline argument forwarding][running],
which documents appending arguments for regular inline tasks without a `usage` specification.

The source was read in an unmodified private clone of the release tag.
No upstream scripts or builds were executed.

## Verification

The disposable fixture used an isolated home and XDG directories,
without inherited provider credentials.
Its `shell` used the absolute executable from Node's `process.execPath`.
The equivalent task definitions were:

```toml
# Disposable mise.toml; use the tested Node executable as the shell program.
[tasks.probe]
shell = "node --input-type=module-typescript -e"
run = 'console.log("entered")'

[tasks.parsed]
shell = "node --input-type=module-typescript -e"
usage = 'arg "[args]" var=#true'
run = 'console.log(process.env.usage_args)'
```

Working catalog:

- `mise run --no-deps --skip-tools probe`:
  exit zero and `entered`.
- `mise run --no-deps --skip-tools parsed -- input.ts`:
  exit zero and `input.ts`.
- Actual package-scoped `format:oxlint` without appended paths:
  zero warnings and errors across 1388 files.

Failing catalog:

- `mise run --no-deps --skip-tools probe -- input.ts`:
  exit one,
  no stdout and `ERR_INVALID_TYPESCRIPT_SYNTAX`.
- Actual package-scoped `format:oxlint` with appended source paths:
  the same diagnostic,
  before the formatter began.

The automatic harness is `~/temp/agent/check-mise-inline-node-args-20260911.mts`.
Its complete captured output is `~/temp/agent/mise-inline-node-args-verification-20260911.out`.
The package observations are `writer-pairing-evidence-format-20260911.out`
and `writer-pairing-evidence-package-format-20260911.out` in the same private scratch directory.

## Verified workarounds

Use the existing package-scoped formatter without extra arguments:

```bash
# From the repository root.
mise run --no-deps --skip-tools //package/translation-repair/<package>:format:oxlint
```

Tradeoff:
this formats the package,
not only the named files.
Review the scoped Git diff afterward.
The observed run changed no files.

For a task that genuinely needs named inputs,
the disposable `parsed` task proves that a `usage` declaration exposes the argument through `usage_args`
without appending it to Node code.
This is a configuration path,
not a claim that the existing formatter forwards file arguments.
Its template and wrapper would need their own implementation and verification.

## Task dependencies differ from automatic dependency preparation

The input-CLI test harness incorrectly treated `--no-deps` as task-dependency skipping.
After `test:unit` gained a `bootstrap:seal` dependency,
Mise still ran that build in a container where the bootstrap candidate was read-only.
Rolldown reported `UNHANDLEABLE_ERROR` while writing `producer-prepare.d.mts`,
with `Read-only file system (os error 30)`.
No unit test ran in that failed attempt.
This was an invocation/mount-contract error,
not a demonstrated Rolldown defect.

The same inspected Mise release separates these flags at `src/cli/run.rs:247-259`:

```rust
// src/cli/run.rs, separate excerpts
/// Skip automatic dependency preparation
pub no_deps: bool,
/// Run only the specified tasks skipping all dependencies
pub skip_deps: bool,
```

`src/cli/run.rs:731-734` uses `no_deps` for the preparation engine:

```rust
// src/cli/run.rs
let deps_engine = if self.no_deps {
    None
} else if subdir_configs.is_empty() {
    Some(DepsEngine::new(&config)?)
```

`src/task/task_executor.rs:1033-1037` instead clears task graph edges for `skip_deps`:

```rust
// src/task/task_executor.rs
if self.skip_deps {
    t.depends.clear();
    t.depends_post.clear();
    t.wait_for.clear();
}
```

The native failed catalog is `devtest-imcxOL/task.out`:
`--no-deps --skip-tools` runs `bootstrap:seal` and fails before testing.
The corrected catalog is `devtest-ZtulTu/task.out`:
adding `--skip-deps` reaches the requested test files without rebuilding the candidate.
That run still fails its names-only inventory assertions;
it verifies dependency selection,
not successful tests.
The candidate had already been built in the same verification sequence.

For intentionally prebuilt test artifacts,
use the distinct flag:

```sh
# From the development repository root, after normal and bootstrap builds.
mise run --no-deps --skip-deps --skip-tools //package/translation-repair/<package>:test:unit -- \
  src/corpus-run/producer-prepare.unit.test.ts
```

Tradeoff:
`--skip-deps` removes task prerequisites,
so the caller owns artifact freshness.
`--no-deps` may still be appropriate to suppress automatic dependency preparation,
but it does not replace that responsibility or suppress declared task dependencies.
No upstream patch or filing is proposed for this documented flag distinction.

## What does not work

Adding `--` before paths does not repair this invocation:
the failing fixture and package command both used it.
The separator passes the paths to the task;
it does not teach the task how to consume them.

`--no-deps` does not suppress a declared task dependency;
`--skip-deps` is the separately verified control for that task-graph behavior.

Changing the edited TypeScript cannot fix a syntax error in the generated task program.
No source-code correction was attempted as a workaround.

## Upstream filing decision

1.  Upstream fault is not established:
    this package formatter declares no file-argument interface,
    and the observed forwarding matches the inspected executor.
2.  No upstream fix is needed for the verified package invocation.
    A local argument-aware task can use the demonstrated `usage` path.
3.  Inline forwarding and explicit usage specifications are documented.
    Automatic adaptation of this formatter wrapper to file arguments is not established.
4.  Contribution policy was not evaluated because no upstream report is proposed.
5.  No maintainer response or likelihood of an upstream change is claimed.
6.  No upstream patch was built;
    the owned fixture verifies the configuration alternative instead.

The `.out-of-scope/` inventory had no matching Mise exemption.
The low-impact TypeScript-formatting exemption concerns formatting rules,
not task argument handling.
Issue searches for `node shell arguments` and `shell arguments` returned no matches.
The broader PR search for `inline arguments` found
[PR 11344][display-pr],
whose full body and comments describe task-header display,
not this Node parse failure.
Its display handling is already present in the inspected release.

Nothing to file or add upstream.

[arguments]: https://mise.jdx.dev/tasks/task-arguments.html
[running]: https://mise.jdx.dev/tasks/running-tasks.html
[display-pr]: https://github.com/jdx/mise/pull/11344
