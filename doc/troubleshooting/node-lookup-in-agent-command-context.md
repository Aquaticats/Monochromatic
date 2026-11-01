# Git wrapper cannot find Node in an agent command context

## Symptom

On 2026-09-12,
a Bash-tool invocation of `git status --short` failed before the Git policy CLI loaded:

```text
/var/home/user/Monochromatic/node_modules/.bin/git: line 48: exec: node: not found
```

`command -v node` in that command context returned exit 1.
Its `PATH` contained `/home/user/.local/share/mise/installs/node/26.8.1/bin`.
The inspected installation directory did not contain that version.
Direct execution of the retained Node 26.7.0 binary returned `v26.7.0`.

This is separate from the preparation-receipt TypeScript and lint findings.
Their verification processes reached the compiler and linter successfully.

## Source trace and limits

The installed repository wrapper at `node_modules/.bin/git:40` checks adjacent Node binaries,
then a command-resolved binary,
and finally executes bare `node`:

```sh
# node_modules/.bin/git:42 through 49
elif [ -x "$basedir/node" ]; then
  exec "$basedir/node"  "$basedir/../@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs" "$@"
elif command -v node >/dev/null 2>&1; then
  exec node  "$basedir/../@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs" "$@"
elif [ -n "$exe" ] && command -v node.exe >/dev/null 2>&1; then
  exec node.exe  "$basedir_win/../@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs" "$@"
else
  exec node  "$basedir/../@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs" "$@"
```

The diagnostic and failed lookup establish the boundary that failed.
They do not establish which component constructed that command's `PATH`.
No claim is made that Node was uninstalled globally,
that Git policy failed,
or that Mise introduced the missing entry.

## Verification

Working observations:

- Direct Node 26.7.0 execution returned its version with exit 0.
- Prepending that installation's `bin` directory for `git status --short` returned the expected scoped changes.
- The receipt verification driver used its own `process.execPath` directory for child-command lookup.
  `preparation-receipt-verification-r3-20260912.out` records successful build,
  types,
  tests and lint under that runner.

Failing observations:

- Bare `command -v node` in the affected Bash-tool context returned exit 1.
- The repository Git wrapper's bare lookup failed with the quoted diagnostic and exit 127.

The process manager did not reproduce the failing context:
`stale-node-path-probe-20260912.out` resolved Node 26.7.0.
A follow-up replacing only its Node-installation path component still resolved Node 26.8.2:
`stale-node-path-probe-controlled-20260912.out`.
Those scripts' expected-failure assertions were incorrect harness assumptions,
not additional tool failures or evidence that the Bash-tool context recovered.

## Observed recovery

The owner reported running `mise upgrade` in the translation-repair worktree,
then restarted the terminal and session.
The agent's subsequent Bash-tool probe returned:

```text
/home/user/.local/share/mise/installs/node/26.8.2/bin/node
v26.8.2
```

An agent probe between the upgrade report and the restart still returned exit 1 for `command -v node`.
This sequence records the observed recovery boundary;
it does not isolate the upgrade and restart as independent causes.
The command-local override is no longer needed in the recovered context.

## Verified workaround

The agent used a command-local override,
not a global installation or configuration change:

```bash
# Run from the intended worktree; the retained executable was verified with --version first.
PATH="${HOME}/.local/share/mise/installs/node/26.7.0/bin:$PATH" git status --short
```

For the already-running Node verification driver,
child commands received `dirname(process.execPath)` prepended to their inherited `PATH`.
This directs plain child lookup toward the selected runner.
It does not bind nested Mise task environments to that version,
as the separate guard-proof incident records.
The tradeoff is explicit runtime selection:
it does not adopt a newly installed version or repair the parent harness environment.
Revalidate the executable before reuse;
the version in this incident is evidence,
not a repository-wide pin.

## What does not work

A managed-process probe is not interchangeable with the failed Bash-tool surface.
Replacing only one path component also failed to recreate the full lookup conditions.
Do not infer the original context's resolution from either successful managed probe.

## Separate incident: Mise task children used another Node version

Namespace guard R3's worker asserted Node `26.8.2`,
but `native-sync-promise-awaited-test.out` printed Node `26.7.0` from the failing test and task shell.
The guard worktree's `mise.lock:832` contains `26.7.0`;
the development worktree's owner-modified lock contains `26.8.2` at that line.
Neither lock was edited for this remedy.

The same executable-layer distinction was already recorded for the pairing identity proof in
[writer unit scope](../planning/translation-repair-writer-unit-scope-2026-09-11.md).
The newer launcher checks did not carry that distinction forward.
The retained qualification,
receipt and request failure logs also report Node `26.7.0`.
Their guard detections are not evidence of test execution under `26.8.2`.

The repository test runner launches a fresh bare Node command,
`mise.toml:479`:

```js
// mise.toml
const proc = spawn('node', [...nodeArgs, file], { cwd: testFileCwd(file), stdio: 'inherit' })
```

Mise's documented override is `MISE_${TOOL}_VERSION`:
`docs/configuration.md:662` in the inspected `mise-v2026.9.5-20260911` clone,
commit `016fcd16a991c85e099d4f0b571bc44978a9eb94`,
says `MISE_NODE_VERSION=20` selects Node regardless of the version in configuration files.
Its `src/toolset/builder.rs:132` builds and merges an environment-sourced tool request:

```rust
// src/toolset/builder.rs
for (k, v) in env {
    if let Some(tool_name) = tool_from_env_var_name(&k) {
        let ba: Arc<BackendArg> = Arc::new(tool_name.as_str().into());
        let source = ToolSource::Environment(k, v.clone());
        let mut env_ts = Toolset::new(source.clone());
        for v in v.split_whitespace() {
            let tvr = ToolRequest::new(ba.clone(), v, source.clone())?;
            env_ts.add_version(tvr);
        }
        ts.merge(env_ts);
    }
}
```

The R4 guard launcher sets `MISE_NODE_VERSION=26.8.2`
and a `NODE_OPTIONS` preload that records inheriting Node processes and refuses any other version.
Each designated test must have its own matching runtime record.
This explicitly covers observed task shells and designated test-entry processes,
not descendants that clear the preload environment.
The executable path of only the worker is no longer treated as test-runtime evidence.

The no-network container control in `namespace-guard-tools-r4-20260912.out` passes:

- Mise with the `26.8.2` override executes the probe successfully.
- The same invocation with the `26.7.0` override fails with
  `Preparation runtime mismatch: expected v26.8.2, received v26.7.0`.
- `runtime-checks.jsonl` independently records each selected executable and version.

The exact launcher and preload are retained in agent scratch as
`run-namespace-guard-r4-container-20260912.mts` and `namespace-runtime-check-20260912.mjs`.
The final R5 proof adds a fresh exclusive runtime ledger and run identity for each invocation.
It verifies all 26 designated mutation failures and restored checks on the requested runtime.
Runtime observations are retained per build/test record in
`/var/home/user/temp/agent/namespace-guard-proof-20260912/r5/mutation-verification.json`.
The override's tradeoff is intentional per-experiment version selection;
it is not a repository pin,
an installation change or a claim that `PATH` controls every nested runtime manager.

## Upstream filing decision

- Upstream fault:
  not established;
  the evidence concerns a local command environment.
- Upstream fix:
  no source defect was identified to patch.
- Supported use case:
  no failing upstream API or unsupported combination was established.
- Contribution policy:
  not assessed because there is no candidate upstream report.
- Maintainer disposition:
  not assessed for the same reason.
- Prototype:
  the command-local consumer workaround was exercised;
  no upstream patch is warranted by this evidence.

Nothing to file or add upstream.
No issue draft or external communication was created.
