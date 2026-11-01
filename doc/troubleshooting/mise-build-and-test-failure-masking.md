# mise 2026.7.0 buildAndTest failure masking and resolution

## Resolution

Repository commit `74740bbc1` fixes the orchestration;
`5419f39d9` regenerates `mise.toml` through file-enforcer.
The historical diagnosis and pre-fix observations remain in this document.

The root task now builds without invoking `try`,
retains errors from each phase,
runs tests even after build failure,
and fails after both phases if either failed.
The retained errors preserve the original exceptions as causes.

`mise.no-env.toml:825` retains the build error:

```javascript
// mise.no-env.toml:825
failures.push(new Error('build phase failed', { cause: error }))
```

`mise.no-env.toml:834-836` retains the test error and rejects the aggregate:

```javascript
// mise.no-env.toml:834-836
  failures.push(new Error('test phase failed', { cause: error }))
}
if (failures.length > 0) { throw new AggregateError(failures, 'buildAndTest failed') }
```

Package selection remains unchanged,
including skipping a missing package build task.
The explicitly named `try` and `prepareAndBuild--allowFailure` remain tolerant.

### Resolution verification

`package/monorepo-task/core-exec/src/build-and-test.unit.test.ts`
exercises the generated task through real mise subprocesses.
Its helpers extract original task and variable definitions from `mise.toml`,
replace only the build/test phase implementations,
and provide disposable configuration and state directories.
A file-backed execution trace proves that tests ran after build failure;
printed commands cannot satisfy that assertion.

The passing regression matrix covers root and package selection,
`./` prefixes,
paths containing spaces,
all build/test success and failure combinations,
missing package build tasks,
retained exception causes,
intentional tolerance,
and canonical/generated configuration synchronization.

Verified commands:

```sh
# Repository root: build the selected package and run its orchestration regression fixture.
mise run buildAndTest -- package/monorepo-task/core-exec/src/build-and-test.unit.test.ts
```

```sh
# Repository root: package checks.
mise run //package/monorepo-task/core-exec:lint:types
mise run //package/monorepo-task/core-exec:lint:oxlint
```

Sensitivity control used copied test helpers and configuration in a disposable tree.
The unmodified copied suite exited `0`.
Removing only the final `AggregateError` check from both copied configuration files
made the same suite exit `1`,
including `expected +0 to equal 1` for failing-build/passing-test cases.
The real repository configuration remained unchanged during this control.

## Symptom

[Issue #302](https://github.com/Aquaticat/Monochromatic/issues/302)
reports a repository task returning success after its build fails.
The issue cites repository revision `4d1d66a0e2440f4402ea1eb454db6e49aacfbc48`.
This investigation reproduced the behavior using task definitions from
`bd08f5df7b1088cef50fd667d781e5f353aefd7f`.
The historical `packages/` directory segment is now `package/`.

A fake failing build prints Node's `Error: BUILD_FAILED`
and mise's `[//:build] ERROR task failed`.
A passing test subsequently prints `TEST_OK`,
and the outer `mise run buildAndTest` exits `0`.
The failure status is suppressed,
 not the inherited build diagnostics.

No stale artifact was needed to reproduce this.
Stale,
 missing,
 or partially written build output is a possible consequence,
not evidence of a mise caching defect.
A source test may pass without consuming any build output.

## Root cause

### The repository explicitly catches the build failure

At the investigated repository revision,
`mise.no-env.toml:628-637` defines `try` as a custom task.
It is not a built-in mise operation:

```toml
# mise.no-env.toml:628-637
[tasks.try]
description = "Run a task, tolerating failures"
usage = 'arg "task" help="Task name to run with failure tolerance"'
shell = "node --input-type=module-typescript -e"
run = """
const { execFileSync } = await import('node:child_process')
// Deliberately tolerant: the child's stderr is inherited (so its failure is
// visible), and try only stops the non-zero exit from propagating.
try { execFileSync('mise', ['run', process.env.usage_task], { stdio: 'inherit' }) } catch { /* tolerated */ }
"""
```

`execFileSync` throws when the child fails.
The catch discards that exception,
and Node finishes successfully.
The fixture's uncaught control fails,
while its caught control succeeds with the same failing child.

`mise.no-env.toml:801-803` invokes that tolerant wrapper before tests:

```javascript
// mise.no-env.toml:801-803
if (args.length === 0) {
  execFileSync('mise', ['run', 'try', '--', 'build'], { stdio: 'inherit' })
  execFileSync('mise', ['run', 'test'], { stdio: 'inherit' })
```

The selected-package branch catches the build failure directly,
and the outside-package branch uses `try` again,
at `mise.no-env.toml:815-822`:

```javascript
// mise.no-env.toml:815-822
    if (allTasks.some((task) => task.name === buildTask)) {
      try { execFileSync('mise', ['run', buildTask], { stdio: 'inherit' }) } catch { /* tolerate build failure, still run tests */ }
    }
  } else {
    execFileSync('mise', ['run', 'try', '--', 'build'], { stdio: 'inherit' })
  }
  await runTestFiles(args, [])
}
```

The exception is not retained for a later aggregate failure.
Tests can therefore determine success even after an attempted build failed.
Task discovery and JSON parsing are outside this catch;
those failures are not suppressed.
If the selected package has no matching build task,
this branch skips the build rather than attempting it.

### Mise observes the task process, not JavaScript's discarded exception

The inspected upstream source is tag `v2026.7.0`,
commit `857b73f6a6b39a3bc90c44119a1e86ee11bd7273`,
from [jdx/mise](https://github.com/jdx/mise).
Paths in this subsection refer to that upstream checkout.

`src/task/task_executor.rs:931-934` chooses the task's interpreter:

```rust
let shell = task.shell()?.unwrap_or(self.clone_default_inline_shell()?);
let (program, _shell_args) = task_shell_parts(&shell, "inline shell")?;
trace!("using shell: {}", shell.join(" "));
let mut full_args = shell.clone();
```

For this repository task,
that interpreter is Node.
The relevant non-raw command completion path in `src/cmd.rs:935-943`
checks the process exit status:

```rust
let status = status.unwrap();
if !status.success() {
    if let Some(duration) = timeout_guard.as_ref().and_then(|g| g.timed_out()) {
        bail!("timed out after {duration:?}");
    }
    self.on_error(combined_output, status)?;
}

Ok(())
```

`src/task/task_executor.rs:602` propagates failed script execution:

```rust
self.exec_script(&script, &args, task, env, prefix).await?;
```

`src/task/task_results_display.rs:81-90` exits nonzero for recorded task failures:

```rust
fn exit_if_failed(&self) {
    if let Some((task, status)) = self.failed_tasks.lock().unwrap().first() {
        let prefix = task.estyled_prefix();
        self.eprint(
            task,
            &prefix,
            &format!("{} task failed", style::ered("ERROR")),
        );
        exit(status.unwrap_or(1));
    }
```

The inner mise process reports build failure correctly.
Our wrapper consumes that failure and returns success to the outer mise process.
The task name `buildAndTest` does not impose a build-success contract on its script.
This is a repository orchestration defect,
 not mise ignoring a failed task process.

## Verification

These observations describe the pre-fix task definitions.
Resolution verification records the corrected behavior.

Installed binaries:

- mise `2026.7.0 linux-x64 (2026-07-02)`.
- Node `v26.8.1`.

The exact-task fixture copied `try`,
 `buildAndTest`,
 `parse_usage_args`,
and `run_test_files` from the frozen repository revision.
It replaced builds and tests with controlled failing or passing tasks.
All invocations ran outside the repository,
with disposable configuration,
 home,
 cache,
 and state directories,
a restricted executable path,
and no inherited credentials or repository task environment.

Correct failure propagation:

- Direct failing `build`:
   exit `1`.
- Native sequential task references:
   exit `1`,
   tests not started.
- `run = ["mise run build", "mise run test"]`:
   exit `1`,
   tests not started.
- Node orchestration without the catch:
   exit `1`,
   tests not started.
- Node orchestration retaining exceptions until both phases finish:
  exit `1`,
   passing test ran.
- Direct failing build with `--continue-on-error`:
   exit `1`.
- Failing tests through each `buildAndTest` invocation shape:
   exit `1`.

Failure masking:

- `mise run try -- build`:
   exit `0` despite `BUILD_FAILED`.
- `buildAndTest` without arguments:
   exit `0`,
   passing test ran.
- `buildAndTest -- package/demo/example/pass.mjs`:
   exit `0`,
   passing test ran.
- `buildAndTest -- pass.mjs`:
   exit `0`,
   passing test ran.
- `--continue-on-error buildAndTest`:
   exit `0`,
   passing test ran.

Initial harness attempts failed before reaching the fake build:
one encountered unrelated user configuration,
and another excluded the fixture itself with the search ceiling.
Neither result was counted as task-behavior evidence.
The working fixture uses canonical paths,
sets XDG directories explicitly,
and places `MISE_CEILING_PATHS` at the fixture's parent.

### Minimal reproduction

This reduced configuration isolates the same process boundary.
It requires Node and mise on `PATH` and belongs in a disposable directory:

```toml
# Disposable fixture: mise.toml
[tasks.build]
shell = "node -e"
run = "throw new Error('BUILD_FAILED')"

[tasks.test]
shell = "node -e"
run = "console.log('TEST_OK')"

[tasks.try]
usage = 'arg "task"'
shell = "node --input-type=module-typescript -e"
run = '''
const { execFileSync } = await import('node:child_process');
try {
  execFileSync('mise', ['run', process.env.usage_task], { stdio: 'inherit' });
} catch (error) {
  console.error('Tolerated build failure:', error.message);
}
'''

[tasks.masked]
run = ["mise run try -- build", "mise run test"]

[tasks.strict]
run = ["mise run build", "mise run test"]
```

Runnable probes in the trusted disposable fixture:

```sh
# Disposable fixture directory
mise run masked
# Expected: BUILD_FAILED, TEST_OK, exit 0.
```

```sh
# Disposable fixture directory
mise run strict
# Expected: BUILD_FAILED, no TEST_OK, exit 1.
```

## Verified workarounds

For fail-fast sequencing,
use the reproduction's `strict` task body.
It retains build failure through ordinary mise command sequencing.
Tradeoff:
 tests do not run after a failed build.
This is a verified mechanism,
not a complete replacement for the repository's argument-sensitive package selection.

To keep complete diagnostics,
retain caught errors while running each phase,
then throw an aggregate error if either phase failed.
The exact-task fixture's `retained` control verified this:
the passing test ran after `BUILD_FAILED`,
and the aggregate exited `1`.
Its runnable task body was:

```toml
# Disposable fixture: additional mise.toml task
[tasks.retained]
shell = "node --input-type=module-typescript -e"
run = '''
import { execFileSync } from 'node:child_process';
const failures = ['build', 'test'].flatMap((name) => {
  try {
    execFileSync('mise', ['run', name], { stdio: 'inherit' });
    return [];
  } catch (error) {
    return [error];
  }
});
if (failures.length > 0) { throw new AggregateError(failures, 'Build/test failed'); }
'''
```

Tradeoff:
 tests may inspect stale or incomplete output after build failure,
so their results do not establish that a fresh build works.

The initial explanation-only investigation made no implementation changes.
The implemented correction is recorded in the Resolution section.

## What does not work

- Keeping the catch and expecting mise to infer failure from stderr:
  the wrapper still exits successfully.
- Adding `--continue-on-error` to the existing wrapper:
  the swallowed error never becomes an outer task failure.
- Treating a passing source test as evidence of a successful build:
  the fixture passes without producing build output.

## Upstream filing artifact

Nothing to file upstream for this finding.
The existing repository issue identifies the responsible wrapper.

### Upstream filing decision

1.  Upstream fault:
     no.
    The task explicitly converts child failure into successful completion.
2.  Upstream fixability:
     no upstream fix is required;
    ordinary sequencing already preserves failure in the controls.
3.  Supported use case:
     mise supports task interpreters and sequential `run` entries;
    see its [task configuration documentation](https://mise.jdx.dev/tasks/task-configuration.html#run).
4.  Contribution policy:
     not assessed for a proposed filing,
    because no upstream defect or contribution is proposed.
5.  Maintainer intent:
     no upstream change requested.
    Issue and PR searches for `"task" "exit code" "catch"`,
    plus an issue search for `"continue-on-error"`,
     returned no matches.
    Those search results are not the basis for the diagnosis.
6.  Upstream prototype:
     not applicable because the upstream-fault gate fails.
    Consumer-side controls demonstrate the required semantics.

`.out-of-scope/` was checked;
no mise-specific exemption was found.
The non-filing decision rests on the observed repository defect,
not on an exemption or an empty tracker search.
