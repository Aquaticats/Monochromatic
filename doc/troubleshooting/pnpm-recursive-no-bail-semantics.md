# pnpm 12.14.0 and 12.16.1: `pnpm -r --no-bail run` skips dependents although `cli/recursive` docs say all tasks finish

This document records one cluster of task-orchestration documentation defects in
pnpm 12:
 two first-party pages state different `--no-bail` failure semantics,
and the flags that decide those semantics are hidden from `pnpm run --help`.
The product behavior is unambiguous and tested upstream;
only the documentation and help surface are wrong.

## Symptom

`https://pnpm.io/cli/recursive` (source `docs/cli/recursive.md:69-71`) promises:

```text
# docs/cli/recursive.md:69-71
This config does not affect the exit code.
Even if `--no-bail` is used, all tasks will finish but if any of the tasks fail,
the command will exit with a non-zero code.
```

`https://pnpm.io/workspace-task-orchestration` (source `docs/workspace-task-orchestration.md:228-230`)
states the opposite for dependents:

```text
# docs/workspace-task-orchestration.md:228-230
After a task fails, tasks that depend on it are skipped. With `--no-bail`,
independent ready tasks continue to run and the command exits with a non-zero
code after they settle.
```

Surface trigger:
 `pnpm -r --no-bail run <script>`
(or the `pnpm -r --no-bail <script>` shorthand)
in a workspace where one project's task fails and another project's task
depends on it.

Resulting failure mode:
 the dependent task never runs and is reported as skipped,
contradicting the "all tasks will finish" sentence a reader of `cli/recursive`
relied on when aggregating tasks such as tests across a workspace.

Observed error variants:

```text
# fixture output, --no-bail variant
Error: ERR_PNPM_RECURSIVE_FAIL

  × "pnpm recursive run" failed in 1 packages
```

```text
# fixture output, default --bail variant
Error: ERR_PNPM_RECURSIVE_RUN_FIRST_FAIL

  × "pnpm recursive run" failed in <fixture>/pkgs/lib
```

Secondary symptom in the same cluster:
 `--no-bail`,
`--bail`,
and `--resume-from` are documented on `https://pnpm.io/cli/recursive` and
`https://pnpm.io/workspace-task-orchestration`,
and both tested builds accept them,
but `pnpm run --help` lists none of them,
so the help output cannot arbitrate the contradiction either.

## Root cause

The shipped semantics live in the shared workspace task scheduler,
and they match `workspace-task-orchestration.md`,
not `cli/recursive.md`.

`pnpm/crates/workspace-task-scheduler/src/lib.rs:147-156`
defines `bail` and the skipped-task contract:

<!-- monochromatic-lint-disable markdown/semantic-line-breaks -- verbatim upstream source excerpt -->

```rust
// pnpm/crates/workspace-task-scheduler/src/lib.rs:147-156
    /// When `true`, the first failure stops further dispatch; tasks already
    /// running finish. Tasks never dispatched keep their queued status.
    pub bail: bool,
    /// Runs one task's work. Not called for pass-through tasks (no scripts
    /// to run).
    pub run_task: &'a Run,
    /// A task that runs nothing: a pass-through with no such script, or —
    /// without `--bail` — a task some dependency of which did not pass.
    /// Both are reported as skipped.
    pub on_task_skipped: &'a Skip,
```

<!-- monochromatic-lint-enable markdown/semantic-line-breaks -- end verbatim upstream source excerpt -->

`pnpm/crates/workspace-task-scheduler/src/lib.rs:163-168`
records the one option that would let dependents run,
which defaults to off:

<!-- monochromatic-lint-disable markdown/semantic-line-breaks -- verbatim upstream source excerpt -->

```rust
// pnpm/crates/workspace-task-scheduler/src/lib.rs:163-168
    /// Stop dispatching after the first failure. Takes precedence over
    /// [`Self::continue_on_failure`].
    pub bail: bool,
    /// Let dependents run after a failed dependency when `bail` is false.
    /// Otherwise those dependents are reported as skipped.
    pub continue_on_failure: bool,
```

<!-- monochromatic-lint-enable markdown/semantic-line-breaks -- end verbatim upstream source excerpt -->

`pnpm/crates/workspace-task-scheduler/src/synchronous.rs:217-233`
is the branch that produces the observed behavior:

```rust
// pnpm/crates/workspace-task-scheduler/src/synchronous.rs:217-233
    fn settle(&self, state: &mut SchedulerState, index: usize, completion: TaskCompletion) {
        match completion {
            TaskCompletion::Passed => {
                state.complete(index, &self.dependents, &self.concurrency_limits);
            }
            TaskCompletion::Failed if self.options.bail => state.stop(index),
            TaskCompletion::Failed if self.options.continue_on_failure => {
                state.complete(index, &self.dependents, &self.concurrency_limits);
            }
            TaskCompletion::Failed => {
                state.settled[index] = true;
                state.unsettled -= 1;
                state.block(index, &self.dependents, |dependent| {
                    (self.options.on_node_skipped)(self.node_at(dependent));
                });
            }
            TaskCompletion::Aborted | TaskCompletion::Cancelled => state.stop(index),
        }
    }
```

Because `ScheduleGraphOptions::new` sets `continue_on_failure: false`
(`pnpm/crates/workspace-task-scheduler/src/lib.rs:198`),
the third `Failed` arm blocks every dependent and reports it as skipped.

Upstream tests assert exactly this,
so the behavior is intended,
not an accident:

```rust
// pnpm/crates/workspace-task-scheduler/src/tests.rs:880-918
fn scheduler_without_bail_skips_transitive_dependents_of_a_failure() {
    ...
            bail: false,
    ...
    assert!(ran.contains(&dir("unrelated").to_string_lossy().into_owned()));
    assert!(!ran.contains(&dir("a").to_string_lossy().into_owned()));
    assert_eq!(skipped.into_inner().unwrap(), vec![dir("a").to_string_lossy().into_owned()]);
}
```

Call chain from flag to behavior:
 `pnpm/crates/cli/src/cli_args/cli_command.rs:333-334`
parses `--no-bail` as a hidden global,
`pnpm/crates/cli/src/cli_args/dispatch_script.rs:182`
maps it with `no_bail: !config.bail`,
and the `settle` match in
`pnpm/crates/workspace-task-scheduler/src/synchronous.rs:217-233`
turns a failure into skipped dependents.

The help-surface gap has its own cause in the same file:

<!-- monochromatic-lint-disable markdown/semantic-line-breaks -- verbatim upstream source excerpt -->

```rust
// pnpm/crates/cli/src/cli_args/cli_command.rs:322-337
    /// Recursive only: resume execution from the given package.
    #[clap(long = "resume-from", global = true, hide = true)]
    pub resume_from: Option<String>,
    ...
    /// Recursive only: keep going after a project fails.
    #[clap(long = "no-bail", global = true, hide = true, overrides_with = "bail")]
    pub no_bail: bool,
    ...
    /// Stop a recursive command after the first failure.
    #[clap(long, global = true, hide = true, overrides_with = "no_bail")]
    pub bail: bool,
```

<!-- monochromatic-lint-enable markdown/semantic-line-breaks -- end verbatim upstream source excerpt -->

`hide = true` keeps the documented flags out of `pnpm run --help`.

The reading recorded in
`doc/audit/tech-monorepo-manager-vet-2026-09-16.md`
treated the two pages as an unresolved two-way contradiction,
blocking any conclusion about failure tolerance.
That reading was too generous to `cli/recursive.md`.
The probe and the scheduler sources cited here disprove one statement outright:
 dependents do not finish under `--no-bail`,
so the sentence "all tasks will finish" is false as written,
while `workspace-task-orchestration.md:228-230` matches shipped behavior.

The wording spread is old.
The same false sentence appears in `versioned_docs/version-10.x/cli/recursive.md:74`,
`versioned_docs/version-11.x/cli/recursive.md:83`,
and every archived copy from `version-2.x` to `version-9.x`,
while `docs/workspace-task-orchestration.md` carries the correct text since
[pnpm/pnpm.io#903](https://github.com/pnpm/pnpm.io/pull/903)
merged in 2026-08.
That pull request rewrote the bail behavior in the new guide and linked it from
`docs/cli/recursive.md`,
but left the old sentence in place.

## Verification

Versions under test:

- pnpm 12.14.0,
  the Mise-managed install this repository resolves
  (`/home/user/.local/share/mise/installs/pnpm/12.14.0/pnpm`),
  Linux x64.
- pnpm 12.16.1,
  the version `toolchain.lock.json` records for its `pnpm` entry
  (`track: latest`),
  taken from `https://github.com/pnpm/pnpm/releases/download/v12.16.1/pnpm-linux-x64.tar.gz`
  with SHA-256 `565ff81174a14f6f7bd5d84c1b17e51031a80230d370a1fe24be5d16bd02a825`,
  matching that lock entry.

Source revisions:

- `https://github.com/pnpm/pnpm` at `e7b88a6a62aa042ac2c8bc9fbce3832226c4db4d`
  (2026-10-29),
  cloned shallowly to `~/temp/agent/pnpm-src-2026-10-28`.
- `https://github.com/pnpm/pnpm.io` at `7aded9fe30ceb31d4302b4189fcac84c958582b0`
  (2026-10-28),
  cloned shallowly twice:
  `~/temp/agent/pnpm-io-src-2026-10-28` for the trace and
  `~/temp/agent/upstream-prototype.KryEvjK1/pnpm.io` for the prototype below.

Runnable harness:
 a disposable three-project workspace,
no external dependencies,
where one project's task fails,
one project depends on it,
and one project is independent.

```jsonc
// pkgs/lib/package.json
{
  "name": "@probe/lib",
  "version": "0.0.0",
  "scripts": {
    "test": "node -e \"console.log('lib#test ran'); process.exit(1)\""
  }
}
```

```jsonc
// pkgs/app/package.json
{
  "name": "@probe/app",
  "version": "0.0.0",
  "dependencies": {
    "@probe/lib": "workspace:*"
  },
  "scripts": {
    "test": "node -e \"console.log('app#test ran')\""
  }
}
```

```jsonc
// pkgs/free/package.json
{
  "name": "@probe/free",
  "version": "0.0.0",
  "scripts": {
    "test": "node -e \"console.log('free#test ran')\""
  }
}
```

```yaml
# pnpm-workspace.yaml
packages:
  - 'pkgs/*'
```

Commands,
run in the fixture root with each binary:

```sh
# fixture root
pnpm install
pnpm -r --no-bail run test
pnpm -r run test
pnpm -r run --dry-run --json test
pnpm -r --resume-from @probe/free run test
```

### Patterns that work cleanly

- `pnpm -r run --dry-run --json test` prints the resolved graph on both
  versions,
  including the default `^test` edge from `pkgs/app` to `pkgs/lib`,
  with `missingScript` flags for pass-through tasks.
- Independent project `@probe/free` completes under `--no-bail` after
  `@probe/lib` fails,
  and the command exits `1`.
- `@probe/app` is correctly skipped as a dependent,
  which matches `workspace-task-orchestration.md`.
- `--resume-from @probe/free` is accepted and runs on both versions even though
  `pnpm run --help` does not list it.
- `pnpm -r run test` under default `--bail` reports
  `ERR_PNPM_RECURSIVE_RUN_FIRST_FAIL` naming the failing package,
  and dispatches nothing further.

### Patterns that fail

One variant,
the dependent-never-runs contradiction with `cli/recursive.md`:

```text
# pnpm 12.14.0 and 12.16.1, pnpm -r --no-bail run test
Scope: all 3 workspace projects
pkgs/free test$ node -e "console.log('free#test ran')"
pkgs/lib test$ node -e "console.log('lib#test ran'); process.exit(1)"
pkgs/lib test: lib#test ran
pkgs/free test: free#test ran
pkgs/lib test: Failed
[ELIFECYCLE] Test failed. See above for more details.
pkgs/free test: Done
Error: ERR_PNPM_RECURSIVE_FAIL

  × "pnpm recursive run" failed in 1 packages
```

`app#test ran` never appears on either version.
The `cli/recursive` sentence "all tasks will finish" is therefore false for
dependents on 12.14.0 and 12.16.1 alike.

Help catalog,
identical on both versions:

```text
# pnpm run --help, lines matching resume|bail|dry-run|reverse
      --dry-run
          With `--dry-run`, print the tasks and their resolved dependency edges as JSON
      --reverse
          Process recursive workspace projects in reverse order
```

No `--bail`,
`--no-bail`,
or `--resume-from` line appears,
although the builds accept all three.

## Verified workarounds

1.  Treat `workspace-task-orchestration.md` as the specification and
    `cli/recursive.md`'s sentence as wrong.
    Shape aggregates so that work which must run after a failure is not a
    dependent of the failing task;
    unrelated tasks do continue under `--no-bail`.
    Tradeoff:
    a task whose dependency failed cannot be forced to run by any
    `--no-bail` spelling,
    and the only failure cause recorded for it is the skipped status.
2.  When a test must run even though its build failed,
    put both steps inside one task script and make the tolerance explicit in
    that script.
    Tradeoff:
    the script owns its own exit-code policy,
    so a careless wrapper can mask a real build failure the aggregate should
    have reported.
3.  Keep the repository-owned fanout that collects failures across packages
    (the Node fanout in `mise.no-env.toml`),
    which is what this repository does today.
    Tradeoff:
    the orchestrator,
    not pnpm,
    owns failure tolerance and aggregate exit status,
    so pnpm's scheduler stays out of the failure-policy loop.

## What does not work

- Relying on `pnpm run --help` to enumerate behavior flags.
  `--resume-from`,
  `--bail`,
  and `--no-bail` are `hide = true` globals
  (`pnpm/crates/cli/src/cli_args/cli_command.rs:321-337`),
  so help silence does not mean absence.
- Expecting `--no-bail` to run dependents after a failed dependency.
  Tested on 12.14.0 and 12.16.1;
  both skip them,
  and upstream tests assert the skip.
- Reading the versioned docs pages as independent corroboration.
  `versioned_docs/version-10.x` and `versioned_docs/version-11.x` repeat the
  same false sentence as the current `docs/cli/recursive.md`.
- Treating the contradiction as proof that pnpm's semantics are undefined.
  The scheduler defines and tests them;
  only `cli/recursive.md` disagrees.
- Asking `pnpm pipeline` to settle the question.
  Its own wording ("keeps going after a task fails,
  so one run reports every failure rather than the first")
  is a third phrasing,
  and pipeline behavior was not probed here.

## Upstream filing artifact

### Out-of-scope check

`.out-of-scope/` in this repository was read on 2026-10-28.
It lists `bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
and `typescript-project-references.md`.
No entry covers pnpm,
documentation contradictions,
or task orchestration,
so no exemption applies.

### Duplicate search

Queries on 2026-10-28,
open and closed:

- `gh search issues --repo pnpm/pnpm 'no-bail'` (15 results),
  `gh search issues --repo pnpm/pnpm.io 'no-bail'`,
  `gh search issues --repo pnpm/pnpm.io 'bail'`,
  `gh search issues --repo pnpm/pnpm.io 'recursive'`,
  `gh search prs --repo pnpm/pnpm.io 'bail'`,
  and `gh search issues '"all tasks will finish"'`.
- Nearest hits:
  [pnpm/pnpm.io#519](https://github.com/pnpm/pnpm.io/issues/519)
  (open,
  the inverse gap of a flag in help but missing from docs),
  [pnpm/pnpm#8013](https://github.com/pnpm/pnpm/issues/8013)
  ("--no-bail behavior inconsistent",
  closed),
  [pnpm/pnpm#14211](https://github.com/pnpm/pnpm/issues/14211)
  (RFC 23 follow-ups,
  closed,
  whose item 5 is the docs work in
  [pnpm/pnpm.io#903](https://github.com/pnpm/pnpm.io/pull/903)),
  and [pnpm/pnpm#7381](https://github.com/pnpm/pnpm/issues/7381)
  (`--continue-on-error` feature request,
  closed).
- No issue or pull request reports the `cli/recursive.md` versus
  `workspace-task-orchestration.md` contradiction,
  and no open pull request touches that sentence.

### Upstream filing decision

Constraint walk:

1. **Is it really upstream's fault?**
   Yes.
   Two first-party pages in `pnpm/pnpm.io` disagree,
   and one contradicts the shipped scheduler
   (`workspace-task-scheduler/src/tests.rs:880`).
   The defect is documentation,
   not consumer misuse.
2. **Can upstream fix it?**
   Yes.
   The minimal fix is one paragraph in `docs/cli/recursive.md`,
   prototyped below.
   Nothing architectural blocks it.
3. **Are they supporting this use case?**
   Yes.
   The same bullet advertises the use case
   ("run tests in every package,
   continue if tests fail in one of them"),
   and RFC 23 plus [pnpm/pnpm#14211](https://github.com/pnpm/pnpm/issues/14211)
   show active investment in workspace task orchestration docs.
4. **Would the repo welcome our contribution?**
   Yes,
   with disclosure.
   `pnpm/CONTRIBUTING.md:325-345` ("AI-assisted contributions")
   states "We use AI coding agents ourselves and welcome contributions made
   with them" and requires a footer naming the agent and the model.
   `pnpm/pnpm.io` ships no `CONTRIBUTING.md` and no issue templates
   (only `.github/workflows`),
   and no policy banning AI-assisted filings was found in either repository.
5. **Will they likely fix it?**
   Likely.
   [pnpm/pnpm.io#903](https://github.com/pnpm/pnpm.io/pull/903)
   merged on 2026-08-29 and
   [pnpm/pnpm.io#953](https://github.com/pnpm/pnpm.io/pull/953)
   on 2026-09-25 show documentation pull requests being merged this quarter.
   Counter-signal:
   [pnpm/pnpm.io#519](https://github.com/pnpm/pnpm.io/issues/519)
   has been open since 2024.
   No documented won't-fix exists,
   so this constraint holds as a soft yes.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Yes.
   See below.

All six constraints hold,
so the draft below is fileable as-is.
It has not been posted;
posting is an external action and waits for user authorization.
The help-hidden flags in `pnpm/crates/cli/src/cli_args/cli_command.rs` are a
separate code change in a separate repository.
They are recorded above but not included in this draft,
because the docs sentence is the load-bearing defect and the prototype stays
minimal.

### Minimal-fix prototype

Disposable clone,
fresh `mktemp` directory,
origin and revision verified before editing:

```text
# ~/temp/agent/upstream-prototype.KryEvjK1/pnpm.io
origin https://github.com/pnpm/pnpm.io.git
HEAD 7aded9fe30ceb31d4302b4189fcac84c958582b0 (2026-10-28 15:29:18 +0100)
```

Pre-patch probe on that tree:

```sh
# prototype clone
rg -n 'all tasks will finish' docs/cli/recursive.md
```

```text
70:Even if `--no-bail` is used, all tasks will finish but if any of the tasks fail,
```

The patch:

```diff
# docs/cli/recursive.md
 This config does not affect the exit code.
-Even if `--no-bail` is used, all tasks will finish but if any of the tasks fail,
-the command will exit with a non-zero code.
+After a task fails, tasks that depend on it are skipped.
+With `--no-bail`, independent ready tasks continue and the command exits with a
+non-zero code after they settle.
```

Post-patch probe:

```sh
# prototype clone
rg -n 'After a task fails' docs/cli/recursive.md docs/workspace-task-orchestration.md
```

```text
docs/workspace-task-orchestration.md:228:After a task fails, tasks that depend on it are skipped. With `--no-bail`,
docs/cli/recursive.md:70:After a task fails, tasks that depend on it are skipped.
```

Verification argument:
 the patched sentences restate the behavior the fixture produced
(dependent `@probe/app` skipped,
independent `@probe/free` completed,
exit `1`)
and the behavior `scheduler_without_bail_skips_transitive_dependents_of_a_failure`
asserts in `workspace-task-scheduler/src/tests.rs:880`.
No upstream build or test script was run;
the clone was never committed to.

### New-issue draft (fileable, not yet posted)

~~~md
Title: docs: cli/recursive says `--no-bail` finishes all tasks; the scheduler skips dependents

### pnpm versions

12.14.0 and 12.16.1 (both reproduced), docs at pnpm.io current `docs/`.

### The contradiction

`docs/cli/recursive.md:69-71` says:

> This config does not affect the exit code.
> Even if `--no-bail` is used, all tasks will finish but if any of the tasks fail,
> the command will exit with a non-zero code.

`docs/workspace-task-orchestration.md:228-230` says:

> After a task fails, tasks that depend on it are skipped. With `--no-bail`,
> independent ready tasks continue to run and the command exits with a non-zero
> code after they settle.

### Actual behavior

The second page matches the scheduler.
`pnpm/crates/workspace-task-scheduler/src/tests.rs:880`
(`scheduler_without_bail_skips_transitive_dependents_of_a_failure`)
asserts that with `bail: false` a dependent is skipped while an unrelated task
runs, and `src/synchronous.rs:225-232` blocks dependents on failure because
`continue_on_failure` defaults to false (`src/lib.rs:198`).

Reproduction (three packages: `lib` fails, `app` depends on `lib`, `free`
independent; each `test` script prints its own name):

```sh
pnpm -r --no-bail run test
```

Observed on 12.14.0 and 12.16.1: `free#test` runs, `lib#test` fails,
`app#test` never runs, exit code 1 with `ERR_PNPM_RECURSIVE_FAIL`.
"all tasks will finish" is therefore false for dependents.

### Suggested fix

Rewrite the paragraph in `docs/cli/recursive.md:69-71` to match
`docs/workspace-task-orchestration.md:228-230`:

```diff
 This config does not affect the exit code.
-Even if `--no-bail` is used, all tasks will finish but if any of the tasks fail,
-the command will exit with a non-zero code.
+After a task fails, tasks that depend on it are skipped.
+With `--no-bail`, independent ready tasks continue and the command exits with a
+non-zero code after they settle.
```

The same stale sentence also appears in `versioned_docs/version-10.x/cli/recursive.md:74`
and `versioned_docs/version-11.x/cli/recursive.md:83`
if you want the versioned copies corrected too.

Written by an agent (DeepSeek Harness, mimo-v2.6-pro).
~~~
