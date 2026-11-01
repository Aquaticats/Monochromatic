# Mise usage arguments are not Node eval process arguments

## Additional task names can be forwarded as ordinary arguments

This is a separate invocation mistake,
observed with mise 2026.10.0 on 2026-10-08.
The following command returned success but ran only the section-source matrix:

```sh
# Private consumer-contract directory: contract/integration/native-batch
mise --no-env --no-hooks run test:section-source-association test:builder test:render-owner --jobs 1
```

Process `proc_a236` logged the actual child command:

```text
# Relevant suffix of the child command printed by mise.
section-source-controls.test.mjs test:builder test:render-owner --jobs 1
```

The script did not consume those additional arguments.
Neither `test:builder` nor `test:render-owner` ran in that process.
The current `mise run --help` states:

```text
# mise 2026.10.0: run --help
Usage: mise run [FLAGS] [TASK] [ARGS]…
Put mise flags before the task name; following arguments are passed to that task.
```

Use separate invocations for the already-defined tasks,
or a task whose `run` array sequences the required commands.
The corrected invocation passed `proc_4b3c` and logged both task names:

```sh
# Private consumer-contract directory: contract/integration/native-batch
mise --no-env --no-hooks run test:builder && mise --no-env --no-hooks run test:render-owner
```

Task exit success proves only the tasks actually invoked.
Check task labels and child commands before attributing verification to trailing names.
No upstream defect or filing is indicated:
this behavior matches the command's documented argument grammar.

## Undeclared path arguments can become inline JavaScript

On 2026-10-12,
`proc_3f44` attempted to restrict the inherited auto-mode formatter by appending source paths.
The task did not declare or consume those path arguments.
Its printed Node input ended with this shape:

```text
# Relevant suffix of the inline task input.
runWorkspaceNode('package/monorepo-task/core-exec', 'oxlint-wrapper', ['--fix']) package/pi-plugin/auto-mode/src/policy-data.ts
```

Node `26.11.1` reported `ERR_INVALID_TYPESCRIPT_SYNTAX`
with `Expected ';', '}' or <eof>`.
The error occurred while parsing the inline task,
before the formatter ran.
This is an unsupported invocation of that task,
not an Oxlint finding in the named source file.

The no-argument invocation reached the configuration build in `proc_0d52`,
but terminated with `SIGTERM` without an Oxlint report or source changes.
The shared restricted slice then showed `oom_kill 1`;
the available kernel-journal query returned no matching entries,
so that counter does not identify the killed process.
This is separate from the invalid argument syntax.
At that checkpoint the 2-GiB limit remained unchanged.
A later relative journal query identified the actual `tsc` victim;
the user then authorized the verified 4-GiB correction described in
[Pi worker memory budget](pi-worker-memory-budget.md).

The auto-mode package's formatter and linter now declare variadic usage arguments
and decode `usage_args` through the existing `vars.parse_usage_args` helper.
They retain `vars.dispatch_workspace_node`,
`ensureOxlintConfig`,
and the original Oxlint wrapper.
No arguments still selects the package;
explicit paths select only those files without changing its rule configuration.
This task change still requires a successful real invocation.
Do not attribute package-wide coverage to a selected-file check.

## Symptom

A mise task declares variadic usage arguments,
but its inline Node runner receives no arguments.
The nested compositor then emits:

```txt
Error: parsing command-line arguments
Caused by:
    no client command given
```

The affected invocation supplied compositor and child arguments after `--`,
but `//package/nested-wayland-session/cli:run` launched its release executable with an empty array.

## Cause

Mise's usage parser stores declared task arguments in `usage_args`.
For a task shell such as:

```toml
shell = "node --input-type=module-typescript -e"
```

those values are not ordinary trailing entries in Node's `process.argv`.
Reading `process.argv.slice(1)` therefore produced an empty list in the observed task.

The root `mise.toml` already provides `vars.parse_usage_args` because `usage_args` is shell-encoded.
Splitting that string on spaces would corrupt quoted paths and arguments.

## Fix

Interpolate the shared parser and read mise's environment value:

```toml
run = """
{{vars.cargo_dispatch}}
{{vars.parse_usage_args}}
runCargo(['build', '--release'])
const args = parseUsageArgs(process.env.usage_args ?? '')
execFileSync('./target/release/monochromatic-nested-wayland-session', args, { stdio: 'inherit' })
"""
```

This preserves spaces,
quotes,
backslashes,
and empty arguments according to the repository's shared task encoding.

## Verification

Invoke the actual task with arguments that must reach the executable:

```sh
mise run //package/nested-wayland-session/cli:run -- \
  --socket /tmp/nested.sock \
  --color-scheme dark \
  -- app
```

The compositor must parse the socket,
start its private appearance portal,
and launch `app` rather than reporting that no client command was given.
