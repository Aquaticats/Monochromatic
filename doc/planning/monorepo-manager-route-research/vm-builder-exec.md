# Replacing vm-builder's `exec` import from file-enforcer

Design research appendix for
[`monorepo-manager-from-scratch-design.md`](../monorepo-manager-from-scratch-design.md),
covering the last item under "Design work not yet started" in that document's "Open questions"
plus the three other migration items named in
[`monorepo-manager-all-rust.md`](../../decision/monorepo-manager-all-rust.md).
Relative links are written for the merged destination,
`doc/planning/monorepo-manager-route-research/`,
which is where the handover's next action says each research result is copied.

Research only.
Nothing here is adopted.
Rule `DRR` requires the user's acceptance before any `doc/decision/` record,
and rule `IWT` kept this session to reads plus this document.

Every claim carries `Verified` with the command or `path:line` that backs it,
or `Unverified`.
Repository paths are relative to `/var/home/user/Monochromatic`.


Merged into "vm-builder migration" in `doc/planning/monorepo-manager-from-scratch-design.md` on 2026-09-17.
Spot-checked then:
`platformCommands` appears only in `exec.ts`,
 its unit test,
 and the README;
vm-builder holds three `run()` helpers;
and `mise.toml` still carries the generated header while `file-enforcer.config.ts:718-721` writes it.

Every decision it feeds applies to meow 0.x only (user,
 2026-09-17).

## Settled requirements recorded, not re-asked

Rule `QGR`:
these are already decided,
 so they are recorded here and never offered as questions.

- meow is a single-file Rust binary that a user runs directly.
  Verified:
  `doc/planning/monorepo-manager-from-scratch-design.md:83-89`.
- file-enforcer's functionality is rewritten in Rust inside that binary.
  Verified:
  `doc/decision/monorepo-manager-all-rust.md:50-51`.
- meow is built in its own git worktree,
  so the TypeScript file-enforcer and meow never enforce the same tree during development.
  Verified:
  `doc/planning/monorepo-manager-from-scratch-design.md:1266-1268`.
- The TypeScript file-enforcer keeps running until meow replaces it.
  Verified:
  `doc/decision/monorepo-manager-all-rust.md:113-114`.
- Mise is removed entirely only once meow supports the full platform matrix.
  Verified:
  `doc/decision/monorepo-manager-all-rust.md:63-65`.
- Every decision applies to meow 0.x only.
  Verified:
  `doc/decision/monorepo-manager-all-rust.md:8-14`.
- Existing VM-builder code stays a machine-setup owner and does not move under a new task runner.
  Verified:
  `doc/planning/mise-removal-coverage.md:549-550`.

## What vm-builder consumes today

### The import and its call sites

- Two files import exactly one symbol from file-enforcer's `/ts` subpath:
  `package/dev-script/vm-builder/src/build-and-import.ts:20`
  and `package/dev-script/vm-builder/src/import.ts:9`,
  both `import { exec, } from '@monochromatic-dev/dev-script-file-enforcer/ts';`.
  Verified:

```bash
rg --line-number --glob '!node_modules' --glob '!**/dist/**' \
  "from '@monochromatic-dev/dev-script-file-enforcer" package/ *.ts --type ts
```

- The workspace dependency edge is
  `package/dev-script/vm-builder/package.json:16`.
  Verified by reading that manifest.
- `exec` is called 14 times,
  seven per file:
  `build-and-import.ts:275`,
   `:296`,
   `:307`,
   `:317`,
   `:360`,
   `:379`,
   `:397`
  and `import.ts:132`,
   `:152`,
   `:163`,
   `:173`,
   `:213`,
   `:231`,
   `:248`.
  Verified:

```bash
rg --line-number "await exec\(|await run\(" \
  package/dev-script/vm-builder/src/build-and-import.ts \
  package/dev-script/vm-builder/src/import.ts
```

- Exactly one call per file consumes the return value:
  the `virsh domstate` read at `build-and-import.ts:296` and `import.ts:152`,
  each wrapped as `(await exec({...})).trim()`.
  Verified by reading both files.
- Two calls per file are existence probes whose only signal is the thrown error:
  `virsh dominfo` (`build-and-import.ts:275`,
   `import.ts:132`)
  and `flatpak info` (`build-and-import.ts:360`,
   `import.ts:231`).
  Both `catch` blocks log through `caughtValueText` and return.
  Verified by reading both files.
- The remaining calls run `virsh destroy`,
   `virsh undefine`,
   `virsh define`,
  and `flatpak override` for effect,
   discarding stdout.
  Verified by reading both files.

### What `exec` actually provides

- `exec` is declared at
  `package/dev-script/file-enforcer/src/pipeline/exec.ts:66`
  and re-exported at
  `package/dev-script/file-enforcer/src/index.ts:96`.
  Verified:
  `rg --line-number "export (async )?(function|const) exec" package/dev-script/file-enforcer/src`.
- It accepts a discriminated union:
  a direct form `{ cmd, args? }`
  and a platform-aware form `{ platformCommands }`
  (`exec.ts:17-32`).
  Verified by reading that file.
- The direct path delegates to `execDirect` (`exec.ts:89-114`),
  which logs the command line at debug level through file-enforcer's own tagged logger
  (`exec.ts:101-105`)
  and returns `nano-spawn`'s `stdout` string (`exec.ts:109-113`).
  Verified by reading that file.
- Failure behavior comes from `nano-spawn`,
   not from file-enforcer:
  the package's own test asserts the thrown message is
  `Command failed with exit code 1: false`
  and that stderr text appears in the message
  (`package/dev-script/file-enforcer/src/pipeline/exec.unit.test.ts:38-58`).
  Verified by reading that file.
- Neither stdout nor stderr is inherited,
  so probe commands stay silent on the user's terminal.
  Verified by inference from `exec.ts:109-112`,
  which passes no stdio options to `nano-spawn`;
  `nano-spawn`'s default captures rather than inherits.
  The capture claim is directly evidenced by `exec.unit.test.ts:20-24` asserting a returned stdout string.

### The platform-aware form has no caller anywhere

- Every `exec` call site in the repository uses the direct `{ cmd, args }` form.
  The `platformCommands` key appears only in
  `package/dev-script/file-enforcer/src/pipeline/exec.ts`,
  `package/dev-script/file-enforcer/src/pipeline/exec.unit.test.ts:108`,
   `:121`,
   `:134`,
  and `package/dev-script/file-enforcer/README.md:116`.
  Verified:
  `rg --line-number --glob '!node_modules' --glob '!**/dist/**' "platformCommands" . --glob '!doc/**'`.
- file-enforcer's two internal callers also use the direct form:
  `package/dev-script/file-enforcer/src/watch/notify.ts:125`,
   `:136`,
   `:172`
  and `package/dev-script/file-enforcer/src/package/manager.ts:333`.
  Verified:
  `rg --line-number --after-context=8 "exec\(" ` on both files.
- So the platform-aware dispatch,
   its `PlatformMatchError`,
  and `evaluatePredicate` in
  `package/dev-script/file-enforcer/src/platform/evaluate-predicate.ts`
  are reachable only from tests and prose.
  They are retired behavior,
   not behavior to port.

### Both internal callers leave the Rust port

- `package/manager.ts` belongs to the `package/` and `data/` modules (FE18 and FE19),
  which move to Meta Package Manager,
   run as an external command.
  Verified:
  `doc/planning/monorepo-manager-route-research/stack-all-rust-rewrite.md:299-305`.
- `watch/notify.ts` belongs to `watch/` (FE21 to FE23),
  replaced by the daemon watcher;
  desktop notification (FE22) becomes `zbus` or `notify-rust`.
  Verified:
  `stack-all-rust-rewrite.md:306-313` and `:434-435`.
- Consequence:
  after the rewrite,
   no consumer of a TypeScript `exec` remains inside meow's scope.
  `exec` is not ported to Rust;
  it is retired,
  and vm-builder is the only thing that must own a replacement.

### The cost of the barrel import

- The `/ts` subpath resolves to `src/index.ts`
  (`package/dev-script/file-enforcer/package.json` `exports`),
  which is a barrel.
  Verified by reading that manifest.
- vm-builder's last recorded type-check program holds 72 distinct file-enforcer source files.
  Verified:

```bash
rg --only-matching '[^"]*file-enforcer/src/[^"]*' \
  package/dev-script/vm-builder/.cache/typescript/tsconfig.tsbuildinfo \
  | sort --unique | wc --lines
```

- A static relative-import closure from that barrel reaches 75 files,
  against 3 from `pipeline/exec.ts` alone.
  Verified:
  a scratch walker
  (`closure.ts` beside this document)
  run as `node closure.ts package/dev-script/file-enforcer/src/index.ts`
  and `node closure.ts package/dev-script/file-enforcer/src/pipeline/exec.ts`.
  The walker mis-resolved two specifiers
  (`src/package/ensure-package.ts` and `src/data/packages.ts`,
   both of which exist),
  so treat 75 as approximate and 72 as the measured figure.
- Rule `ST3` is why this is the shape:
  cross-package imports resolve to TypeScript source so no build precedes type-check or lint.
  Verified:
  `AGENTS.md:1186` and
  `doc/decision/workspace-ts-source-imports.md`.
  The cost is intended,
   not a defect;
  it is nonetheless 72 files of a package being deleted sitting in a VM builder's program.

### vm-builder already owns a process runner, three times over

- An identical streaming helper `run({ cmd, args })` is declared three times:
  `package/dev-script/vm-builder/src/build-and-import.ts:137`,
  `package/dev-script/vm-builder/src/import.ts:98`,
  and `package/dev-script/vm-builder/src/sign-and-push.ts:63`.
  Verified:
  `rg --line-number "^async function run|^function run|//region Streaming" package/dev-script/vm-builder/src/*.ts`.
- Each spawns through `node:child_process.spawn` with `stdio: 'inherit'`
  and throws `${cmd} exited with code N` on a non-zero exit.
  Verified by reading all three.
- `stdio: 'inherit'` is load-bearing:
  every `run` call in vm-builder is a `sudo` invocation,
  and the password prompt reaches the user's terminal only because stdio is inherited.
  Verified:
  `build-and-import.ts:177`,
   `:222`,
   `:255`,
   `:415`,
   `:423` and `import.ts:268`,
   `:282`
  all pass `cmd: 'sudo'`.
- `sign-and-push.ts` imports no file-enforcer symbol.
  Verified:
  `rg --line-number "spawn|exec" package/dev-script/vm-builder/src/sign-and-push.ts`.
- So the package's real need is two shapes of one responsibility:
  inherit for interactive privileged steps,
  capture for `virsh` and `flatpak` queries.

### Direct spawning is the repository's incumbent pattern

- 123 TypeScript files import `nano-spawn` directly,
  63 of them outside test files.
  Verified:

```bash
rg --files-with-matches --glob '!node_modules' --glob '!**/dist/**' \
  "from 'nano-spawn'" package/ --type ts | wc --lines
```

  and the same command piped through `rg --invert-match "test"`.
- The root file-enforcer configuration itself spawns with `nano-spawn` directly rather than through `exec`
  (`file-enforcer.config.ts:13` and `:774`).
  Verified by reading that file.
- No repository document argues for a shared process-execution owner.
  Verified:
  `rg --files-with-matches --ignore-case "nano-spawn" doc/`
  returns route research,
   troubleshooting,
   handovers,
   and audits,
  none of which proposes one.
- `nano-spawn` is a catalog entry at `pnpm-workspace.yaml:102` (`>=2.1.0`),
  so rule `DM1` is satisfied by a `catalog:` reference.
  Verified by reading that file.

## Coverage ledger

Rule `RCO`.
The incumbent is the TypeScript file-enforcer's public API as vm-builder consumes it.
Each entry names the responsibility,
 the owner after the rewrite,
 the selection status,
the parity test that would prove the replacement,
 and the retired behavior.

Rule `MD5` forbids tables,
 so each entry is a list.

### L1: run an executable with an argv vector and capture stdout

- Consumed by:
  `build-and-import.ts:296` and `import.ts:152` (`virsh domstate`).
- Owner after the rewrite:
  vm-builder itself,
   in a new `src/process.ts`,
   under the recommended option.
  Not meow:
  both of file-enforcer's own callers leave the port,
  so meow ships no TypeScript `exec` successor.
- Selection status:
  recommended,
   awaiting the user's answer to question Q1.
- Parity test:
  a unit test asserting a returned stdout string for a succeeding command,
  matching `exec.unit.test.ts:16-37`.
- Retired behavior:
  none.

### L2: throw on a non-zero exit, with the exit code and stderr in the message

- Consumed by:
  the four probe `catch` blocks
  (`build-and-import.ts:285-291` and `:387-393`,
  `import.ts:142-147` and `:239-244`),
  each of which renders the caught value through
  `caughtValueText` from `@monochromatic-dev/module-caught-value/ts`.
  Verified by reading both files.
- Owner after the rewrite:
  vm-builder's `src/process.ts`.
- Selection status:
  recommended.
- Parity test:
  a unit test asserting the thrown message carries the exit code and the child's stderr,
  matching `exec.unit.test.ts:38-58`.
  This is the one behavior whose wording a user reads,
  because it lands in the two `console.warn` lines.
- Retired behavior:
  none.

### L3: keep the child's stdout and stderr off the parent's terminal

- Consumed by:
  every probe call,
  so `virsh dominfo` and `flatpak info` failures do not print raw tool output before the warning.
- Owner after the rewrite:
  vm-builder's `src/process.ts`.
- Selection status:
  recommended.
- Parity test:
  a unit test that runs a command writing to stdout
  and asserts the parent's stdout received nothing.
  Rule `QPC` applies:
  pair it with a positive control that does inherit,
  otherwise the silent result proves nothing.
- Retired behavior:
  none.

### L4: platform-aware command dispatch

- Consumed by:
  nobody.
  Verified:
   the `platformCommands` search recorded in this document.
- Owner after the rewrite:
  none.
- Selection status:
  retire.
- Parity test:
  none needed;
  the removal test is that the repository still type-checks and lints with the form gone.
- Retired behavior:
  `ExecPlatformInvocation`,
   `execPlatformAware`,
   `execCommand`,
  `isNestedPlatformCommands`,
   `PlatformMatchError`,
   `formatEntry`
  (`exec.ts:25-27`,
   `:131-202`,
   `:212-251`),
  and `evaluatePredicate` with `Predicate`,
   `Command`,
   `PlatformEntry`,
   `PlatformCommands`
  (`platform/evaluate-predicate.ts`).
  Together 253 plus 105 lines.
  Verified:
  `wc --lines` on both files.

### L5: debug logging of each spawned command line

- Consumed by:
  vm-builder implicitly,
  through file-enforcer's tagged logger
  (`exec.ts:101-105`,
   tag `execDirect`).
- Owner after the rewrite:
  vm-builder,
   if it wants it.
  Today vm-builder's own output is raw `console.log` with a `[vm-builder]` prefix
  (`build-and-import.ts:175` and every sibling step),
  so the file-enforcer tag is foreign to its output.
  Rule `TLG` permits raw `console` for precise terminal output in a CLI,
  which is what this package is.
- Selection status:
  recommended to drop the inherited tag and keep vm-builder's own prefix.
- Parity test:
  none;
  this is a deliberate behavior change,
   recorded rather than preserved.
- Retired behavior:
  file-enforcer's `execDirect` debug line for vm-builder's 14 calls.

### L6: the `nano-spawn` dependency, supplied transitively

- Consumed by:
  vm-builder,
   which declares no `nano-spawn` entry and reaches it only through file-enforcer.
  Verified by reading `package/dev-script/vm-builder/package.json`.
- Owner after the rewrite:
  vm-builder's own manifest,
  `"nano-spawn": "catalog:"`,
   under rule `DM1`.
- Selection status:
  recommended.
- Parity test:
  `mise run //package/dev-script/vm-builder:lint:types` and `:build` both pass
  with the file-enforcer entry removed.
- Retired behavior:
  the `@monochromatic-dev/dev-script-file-enforcer` workspace edge at
  `package/dev-script/vm-builder/package.json:16`,
  and with it 72 file-enforcer source files leaving vm-builder's type program.

### L7: the `/ts` subpath and the barrel re-export

- Consumed by:
  vm-builder,
   the perf fixture,
   and the root configuration.
- Owner after the rewrite:
  none;
  the package is deleted.
- Selection status:
  the `export { exec, } from './pipeline/exec.ts';` line at `index.ts:96`
  can be deleted once vm-builder stops importing it.
  Optional:
  the file is deleted wholesale later,
  so leaving the line costs nothing but a stale public surface.
- Parity test:
  repository-wide `rg` for the specifier returns only the perf fixture and the root configuration.
- Retired behavior:
  `exec` as a public export.

### L8: the duplicated streaming runner

- Consumed by:
  vm-builder itself,
   in three copies.
- Owner after the rewrite:
  vm-builder's `src/process.ts`,
   as one function.
- Selection status:
  recommended,
   and independent of meow.
- Parity test:
  a unit test asserting a non-zero exit throws,
  plus the existing behavior that `sudo` prompts still reach the terminal,
  which only a manual run can show
  (rule `CB1`:
  that manual step belongs in a runbook if the user wants it proven).
- Retired behavior:
  two of the three copies.

### L9: documentation of `exec`

- Consumed by:
  readers of `package/dev-script/file-enforcer/README.md:134-197`.
- Owner after the rewrite:
  none.
- Selection status:
  the README section is already wrong.
  It documents `exec([...])` and `exec('mise', ['use', 'git'])`,
  neither of which the implementation accepts;
  the implementation takes one object
  (`exec.ts:66`).
  Verified by reading both.
  `README.md:290` also imports
  `@monochromatic-dev/dev-script-file-enforcer/data/packages.ts`,
  a subpath the manifest does not export.
  Verified by reading `package/dev-script/file-enforcer/package.json`.
- Parity test:
  none;
  this is drift to delete,
   not behavior to preserve.
- Retired behavior:
  the whole "Platform-aware exec" section.

Every entry has a viable owner,
so rule `RCO`'s precondition for recommending the incumbent's removal is met
for vm-builder's slice of file-enforcer's API.

## Options for where `exec` lives afterwards

Rule `YKZ`:
each option is designed until a disqualifying problem surfaces or none does.

### Option A: a new workspace module package owning process execution

Shape:
`package/module/process-run`,
 named `@monochromatic-dev/module-process-run` under rule `SGD`,
exporting `runCaptured` and `runInherited`,
built on `nano-spawn`,
with `mise.toml`,
 `package.json`,
 `tsconfig.json`,
 `rolldown.node.config.ts`,
 `README.md`,
`LICENSES/`,
 and unit tests,
 per rules `AP1` to `AP4` and `PKG`.
Rule `AP5` does not apply:
no client bundle.
Name checked against the forbidden strings appendices;
no match.
Verified:

```bash
rg --ignore-case --line-number "subprocess|run-command|spawn|exec" \
  forbidden-strings.append.txt forbidden-strings.append.local.txt
```

The search returned nothing,
and both files were confirmed non-empty by reading their headers (rule `QRY`).

Pros:

- One named owner for a responsibility 63 non-test files currently re-implement.
- Later consumers could migrate:
  `package/open-code-review-issue/cli/src/github-process.ts:122`,
  `package/ssg/aquati.cat/src/lib/git-dates.ts:48`,
  and `package/oxlint-plugin/test-support/src/index.ts:275`
  each hand-roll a `nano-spawn` wrapper.
  Verified:
  `rg --line-number "nano-spawn" package/ --type ts`.
- vm-builder's type program drops from 72 foreign files to a handful.
- Survives meow untouched:
  meow owns no in-process TypeScript spawning.

Cons:

- Creates a published package.
  `package/config/pnpr/config.yaml` is generated by file-enforcer
  and already lists 115 packages including both file-enforcer and vm-builder,
  so a new workspace package joins the publish set automatically.
  Verified:
  `package/config/pnpr/config.yaml:1-3` and `:69-71`.
  That is a supply-chain surface for roughly 30 lines of wrapper.
- Rule `RCI` says to extend a present boundary before proposing a new owner,
  and vm-builder already owns one (entry L8).
- One consumer on day one.
  The three hand-rolled wrappers each carry package-specific error mapping,
  so migrating them is separate work the user has not asked for.
  Rule `AUT` treats that as scope expansion.
- Rule `XNC`:
  start simple,
   refactor when necessary.
  Necessity is not yet demonstrated.

No disqualifying problem.

### Option B: vm-builder owns a private copy

Shape:
one new file `package/dev-script/vm-builder/src/process.ts`
exporting `runInherited({ cmd, args }): Promise<void>`
and `runCaptured({ cmd, args }): Promise<string>`.
`runInherited` is the existing helper,
 moved once instead of copied three times.
`runCaptured` is `nano-spawn` with the return value's `stdout`.
vm-builder's manifest drops the file-enforcer entry and gains `"nano-spawn": "catalog:"`.

Pros:

- Extends a boundary vm-builder already owns,
   which is exactly what rule `RCI` asks for.
- Net line reduction:
  three copies of a 22-line helper collapse into one,
  and the new capture helper is smaller than the 253-line `exec.ts` it replaces.
- No new package,
   no new published artifact,
   no new workspace edge.
- Dependency count unchanged:
  one manifest entry out,
   one in.
- Matches `doc/planning/mise-removal-coverage.md:549-550`,
  which keeps VM-builder code a machine-setup owner.
- Matches the repository's incumbent pattern,
  including the root configuration's own direct `nano-spawn` use.
- Independent of meow:
  it can land before the Rust work starts and shrinks the rewrite's parity surface by one public export.

Cons:

- Leaves the repository-wide duplication untouched.
- vm-builder has no tests today,
  so the parity tests are new work.
  Verified:
  `rg --files package/dev-script/vm-builder/src`
  lists four source files and no `*.test.ts`.
  Rule `PKG` already required them,
   so this is a debt paid,
   not a debt created.
- The debug log line from `exec.ts:101-105` disappears unless re-added.
  Recorded as entry L5 rather than preserved.

No disqualifying problem.

### Option C: an existing repository package that already owns this responsibility

Rule `RCI` requires looking before proposing.
Candidates measured:

- `@monochromatic-dev/dev-script-task-util`,
  described as "CLI utilities for mise task orchestration:
  command executor with allowFailure support,
   file append helper,
   and make-style dependency checker".
  Verified:
  `package/dev-script/task-util/package.json:4`.
  Its `src/command.ts` is the nearest thing to an owner,
  and it already depends on `nano-spawn`.
  Verified by reading that manifest.
- `@monochromatic-dev/git-executable`:
  resolves a native Git executable.
  Verified:
  `package/git/executable/package.json:4`.
- `@monochromatic-dev/dev-script-watch-restart`:
  restarts a long-running child on content change;
  its `SpawnFn` defaults to inherit stdio and is lifecycle-oriented.
  Verified:
  `package/dev-script/watch-restart/src/child.ts:262` and `:272-283`.
- `@monochromatic-dev/oxlint-plugin-test-support`:
  test-only oxlint invocation.
  Verified:
  `package/oxlint-plugin/test-support/src/index.ts:275`.

Shape if `task-util` were chosen:
split `src/command.ts` into a library module plus its bin,
add an `exports` map with a `/ts` subpath,
and have vm-builder import from it.

Pros:

- No new package.
- `task-util` already carries `nano-spawn` and already runs commands.
- It is live:
  `mise.no-env.toml:595`,
   `:603`,
   and `:610` call its `oxlint-wrapper` and `tsc-filter` modules.
  Verified by reading those lines.

Cons:

- `task-util` has no `exports` map at all,
  only `main` and six `bin` entries,
  so there is no `/ts` subpath for rule `ST3` to use.
  Verified by reading `package/dev-script/task-util/package.json`.
- `src/command.ts` is a top-level script:
  it parses `process.argv` through Optique at module scope
  (`command.ts:91-97`)
  and throws at module scope
  (`command.ts:104-117`).
  Importing it for a function would execute an argument parser.
  Verified by reading that file.
- Its declared scope is Mise task orchestration,
  and Mise is being removed entirely.
  Verified:
  `doc/decision/monorepo-manager-all-rust.md:63-65`.
  Pointing a machine-setup script at a task-runner utility ties vm-builder to the thing being deleted.
- None of the other candidates owns "run a command and capture stdout";
  each owns something narrower,
   so adopting one would widen its scope rather than use it.

Disqualifying problem:
choosing `task-util` swaps a dependency on a package being deleted (file-enforcer)
for a dependency on a package whose stated purpose is being deleted (Mise orchestration).
The other candidates do not own the responsibility at all.

### Option D: call meow over its RPC interface

Shape:
vm-builder connects to the daemon's Unix socket and asks it to run `virsh` and `flatpak`.

Pros:

- One process-execution owner for the whole repository,
   in the tool being built.
- Task output would carry meow's structured events.

Cons and disqualifying problems:

- There is no method for it.
  The designed methods are list tasks,
   get one task,
   queue or rerun a task,
   end a task,
  pause,
   resume,
   and set priority.
  Verified:
  `doc/planning/monorepo-manager-from-scratch-design.md:2259-2266`.
  Adding an ad-hoc command executor turns the socket into a remote shell,
  and the design already records that
  "Anyone who can connect to the socket can run tasks as the user,
  so socket permissions are the security boundary".
  Verified:
  `doc/planning/monorepo-manager-from-scratch-design.md:2274-2275`.
- vm-builder needs interactive `sudo` password prompts,
  and meow 0.x offers no pseudo-terminal opt-in;
  the design states that a task needing a terminal runs in the user's own terminal,
  outside the unattended daemon.
  Verified:
  `doc/planning/monorepo-manager-from-scratch-design.md:803-813`.
- meow blocks every configuration-loading command until the repository root is trusted.
  Verified:
  `doc/decision/monorepo-manager-per-user-config.md:63-73`.
  A fresh clone would fail to build a VM until `meow trust` ran.
- Every line meow writes is a JSON object,
  so recovering `virsh domstate`'s raw stdout means parsing meow's framing.
  Verified:
  `doc/planning/monorepo-manager-from-scratch-design.md:137-143`.
- It requires the daemon to be running for a one-shot script.
- `doc/planning/mise-removal-coverage.md:549-550` explicitly keeps VM-builder code out of a new task runner.

Disqualified.

### Option E: shell out to meow's CLI

Shape:
`meow exec -- virsh domstate monochromatic-dev`,
 parsing the JSON lines back into stdout.

Pros:

- No daemon and no socket needed if the subcommand runs in-process.
- Keeps a single Rust binary as the dependency rather than a Node library.

Cons and disqualifying problems:

- No such subcommand exists in the design,
  and inventing one makes meow a general command runner in every repository it ships to,
  which is not among the stated requirements
  (`doc/planning/monorepo-manager-from-scratch-design.md:36-96`).
  Verified by reading that section.
- The JSON line framing still has to be parsed back into raw stdout.
  Verified:
  same output-format section.
- The trust gate still applies to configuration-loading commands.
  Verified:
  `doc/decision/monorepo-manager-per-user-config.md:68`.
- Spawning a Rust binary to spawn `virsh` adds a process for no gain.
- vm-builder is a Node dev script;
  making it depend on a built Rust binary means it cannot run until meow is built,
  which contradicts the separate-worktree decision.

Disqualified,
though less severely than Option D:
it needs no running daemon and no socket security boundary.

### Option F: relocate the existing `exec.ts` into a package that survives

Shape:
move `pipeline/exec.ts` and `platform/evaluate-predicate.ts` verbatim into a new package,
have file-enforcer import them back for its remaining lifetime,
and have vm-builder import from the new home.

Pros:

- Parity is trivially provable:
  the same file,
   at a new path,
   with the same test file moved beside it.
- file-enforcer keeps working unchanged until deletion.
- vm-builder's type program drops to the new package's small closure.

Cons:

- It carries entry L4's dead platform dispatch,
   358 lines across two files,
  into a fresh package on day one.
  Verified:
  `wc --lines` on both files,
   and the `platformCommands` search.
- It pays Option A's whole package cost while inheriting code written for a consumer that is being deleted.
- file-enforcer gains a new workspace edge during its final months,
  which is motion in the wrong direction for a package being removed.
- The new package would have to exist before the rewrite in order to help,
  and would then need its own removal decision later if the shared-owner idea is rejected.

No disqualifying problem,
 but strictly dominated by Option A on content
and by Option B on cost.

## Ranking

`B > A > F > C > E > D`.

Adjacent-pair reasons:

- `B > A`:
  rule `RCI` says extend a present boundary before proposing a new owner,
  and vm-builder already owns three copies of a process runner (entry L8),
  so Option B removes duplication while Option A adds a published package
  and a workspace edge for a single consumer.
- `A > F`:
  both pay the same new-package cost,
  but Option A is written for the consumers it has,
  while Option F imports 358 lines of platform dispatch that no call site in the repository uses.
- `F > C`:
  Option F's new package has no owner conflict,
  while Option C attaches a machine-setup script to `task-util`,
  whose declared scope is Mise orchestration and whose `command.ts` runs an argument parser on import.
- `C > E`:
  Option C keeps the replacement inside the TypeScript workspace where vm-builder runs,
  while Option E makes a Node dev script depend on a Rust binary,
  its trust registry,
   and its JSON line framing to read one line of `virsh` output.
- `E > D`:
  Option E needs no running daemon and no socket,
  while Option D additionally requires the daemon to be up,
  routes vm-builder's privileged commands through a socket whose permissions are the only security boundary,
  and has no RPC method for running an ad-hoc command.

## Recommended design

Option B,
 in one new file,
 with vm-builder's manifest edited.

### The module

`package/dev-script/vm-builder/src/process.ts`,
with TSDoc on every declaration per rule `TSD`
and `//region` markers per rule `ST2`:

- `runInherited({ cmd, args }: { readonly cmd: string; readonly args: readonly string[] }): Promise<void>`.
  The existing helper,
   moved once.
  Keeps `node:child_process.spawn` with `stdio: 'inherit'`,
  because that is what makes `sudo` prompts reach the terminal,
  and changing it is not part of this migration.
- `runCaptured({ cmd, args }: { readonly cmd: string; readonly args: readonly string[] }): Promise<string>`.
  Calls `nano-spawn` and returns `stdout`.

Both take a single destructured object parameter per rule `ST9`,
with `readonly` array parameters per rule `TY5`.

`nano-spawn` rather than `node:util.promisify(node:child_process.execFile)`,
for three reasons:

1.  It reproduces today's thrown message exactly,
    including the exit code and the child's stderr,
    which entry L2 shows the four probe `catch` blocks render to the user.
2.  It is already in the catalog at `pnpm-workspace.yaml:102`,
    and vm-builder already receives it transitively,
    so the dependency graph does not grow.
3.  `execFile` buffers through `maxBuffer` and produces a different error shape,
    so it would need its own error mapping to reach parity.
    The `maxBuffer` default is not a risk for `virsh domstate`,
    whose output is one word;
    the error text is the cost.
    Unverified:
    `execFile`'s default `maxBuffer` value was not read from Node's documentation in this session.

Rule `XRT` prefers cross-runtime patterns,
and `nano-spawn` is the cross-runtime choice here,
while `node:child_process.spawn` stays only where inherited stdio demands it.
vm-builder is Node-only by design;
its bundle comment records that.
Verified:
`package/dev-script/vm-builder/rolldown.node.config.ts:7-15`.

### The call sites

- Replace the two `import { exec, }` lines with
  `import { runCaptured, runInherited, } from './process.ts';`.
- Replace all 14 `exec(` calls with `runCaptured(`.
- Delete the three local `run` declarations and point their 7 call sites at `runInherited`.
- `domain-xml.ts` and `sign-and-push.ts` need no other change,
  beyond `sign-and-push.ts` importing `runInherited`.

### The manifest

- Remove `"@monochromatic-dev/dev-script-file-enforcer": "workspace:*"`
  at `package/dev-script/vm-builder/package.json:16`.
- Add `"nano-spawn": "catalog:"`.
- Rule `LFW`:
  regenerate `pnpm-lock.yaml` with pnpm,
   never by hand,
  and report unrelated drift separately.

### The tests

vm-builder has none today,
and rule `PKG` requires passing tests covering every exported code path,
so the new module arrives with `src/process.unit.test.ts`.
Rule `TCV` wants one test per branch,
 not per function:

1.  `runCaptured` returns a command's stdout.
2.  `runCaptured` throws on a non-zero exit,
    and the message carries the exit code.
3.  `runCaptured` throws with the child's stderr text in the message.
4.  `runCaptured` writes nothing to the parent's stdout,
    paired with a positive control that proves the probe can see output at all
    (rule `QPC`).
5.  `runInherited` resolves on a zero exit.
6.  `runInherited` throws on a non-zero exit.

Rule `THR`:
these run `echo`,
 `false`,
 and `sh -c`,
never `virsh`,
 `flatpak`,
 or `sudo`,
and never against the user's libvirt state.

Run with `mise run //package/dev-script/vm-builder:test:unit`,
which the root template globs as `**/*.unit.test.ts`.
Verified:
`mise.no-env.toml:548` and `:566-572`.
Rule `CM3`:
never `bun test`.

### Verification at the user boundary

Rule `VUB` wants the artifact exercised as a user would.
`mise run //package/dev-script/vm-builder:import` imports a qcow2 into libvirt with `sudo`
and mutates the user's libvirt domains,
so rule `THR` forbids an agent from running it as verification.
The honest boundary check is:

- `mise run //package/dev-script/vm-builder:lint:types` and `:lint:oxlint` pass.
- `mise run //package/dev-script/vm-builder:buildAndTest` passes,
  which its own description already calls the package's verification
  (`package/dev-script/vm-builder/mise.toml`,
   `buildAndTest`).
  Verified by reading that file.
- The `sudo` prompt and the `virsh domstate` read are proven only by a real run,
  which is the user's to make.
  Rule `CB1` and rule `RBK`:
  if the user wants that proven,
  it belongs in `doc/runbook/`,
  written through the `runbook` skill.

### What this does not do

It does not remove `exec` from file-enforcer.
`index.ts:96` may stay until the package is deleted;
its two internal callers keep using `pipeline/exec.ts` directly.

## The other three migration items

### The `file-enforcer-perf` fixture

What it is:

- `package/test-fixture/file-enforcer-perf` benchmarks the TypeScript implementation.
  `src/perf.bench.test.ts:9-21` imports
  `cat`,
   `classifyEvent`,
   `dedup`,
   `expandGlob`,
   `getJsonProperty`,
   `mirrorGlobPath`,
  `overwrite`,
   `reset`,
   `trackDest`,
   `trackRead`,
   `trackWriteTime`;
  `src/perf.config.ts:13-21` imports
  `cat`,
   `dedup`,
   `getJsonProperty`,
   `overwrite`,
   `overwriteEach`,
   `readCache`,
   `reset`.
  Verified by reading both files.
- The fixture is 240 files across 20 simulated packages under `tmpdir()`.
  Verified:
  `package/test-fixture/file-enforcer-perf/README.md:28`
  and `src/setup-fixture.ts:25`.
- Six Mise tasks drive it:
  `perf:setup`,
   `perf:micro`,
   `perf:e2e`,
   `perf:constrained`,
   `perf:teardown`,
  plus `perf:enable-counters` and `perf:disable-counters`.
  Verified by reading `package/test-fixture/file-enforcer-perf/mise.toml`.

Ledger:

- Responsibility:
  a speed baseline for file enforcement.
- Owner after the rewrite:
  a Rust bench package in meow's worktree,
  following the repository's existing convention of a `.bench` sibling with a hand-written bench binary
  (`package/forbidden-strings/core.bench`,
   whose `mise.toml` exposes `build`,
   `run`,
   `lint`,
  `lint:clippy`,
   and `lint:rust`).
  Verified by reading that package's `Cargo.toml` and `mise.toml`.
- Selection status:
  hinges on question Q3.
- Parity test:
  the same corpus,
   the same operation set,
  one run of each implementation,
   compared.
  Rule `QNB` applies:
  measure the run-to-run band on one unchanged build first,
  because a single pair of runs resolves nothing smaller than that spread.
- Retired behavior:
  the mitata micro-benchmarks,
  which measure TypeScript functions that will not exist.

Options:

1.  Retire it with the TypeScript implementation.
    Pros:
    no work,
     and the measured functions vanish anyway.
    Cons:
    the design's risk list has a byte-identical output gate but no speed gate,
    so the rewrite would ship with no evidence it is not slower.
2.  Keep the fixture generator,
     rewrite the measurement.
    `src/setup-fixture.ts` produces the corpus and is implementation-agnostic;
    keep it,
     add a Rust bench in meow's worktree against the same corpus,
    and compare.
    Pros:
    gives the rewrite a speed gate at the cost of one bench binary.
    Cons:
    the corpus generator has to stay alive after its package's other files die,
    which means moving it somewhere that survives.
3.  Freeze the current numbers,
     then retire.
    Run `perf:micro` and `perf:e2e` once before the rewrite starts,
    record the numbers in the design,
    and retire the package.
    Pros:
    cheapest way to keep a baseline.
    Cons:
    numbers recorded on one machine on one day are not a gate;
    rule `QNB` says a single run resolves nothing.
4.  Move to `package-paused/` until meow's file enforcement lands.
    The repository has that mechanism and the test glob already skips it.
    Verified:
    `mise.no-env.toml:564` excludes `package-paused/` from the test glob,
    and `package-paused/README.md:1-10` describes pausing as temporary and reversible,
    distinct from the `package-deprecated/` move in `doc/howto/deprecate-package.md`.
    Pros:
    reversible and cheap.
    Cons:
    a paused benchmark is a benchmark nobody runs,
    so it decays.

Ranking:
`2 > 4 > 3 > 1`.

- `2 > 4`:
  Option 2 produces the comparison the design's risk list is missing,
  while Option 4 only defers the decision.
- `4 > 3`:
  Option 4 keeps the ability to re-measure,
  while Option 3 keeps numbers that rule `QNB` says are not a gate.
- `3 > 1`:
  a recorded number with a caveat still beats no number.

### The `prefer-readonly-parameter-type` fixture read

What it is:

- `package/oxlint-plugin/prefer-readonly-parameter-type/src/workspace-source-effect.unit.test.ts:20-31`
  reads `package/dev-script/file-enforcer/src/cargo/apply-plan.ts` from disk
  and asserts effect summaries for `setIfDiffers`,
   `applyEnforcement`,
   and `applyCargoPlan`.
  Verified by reading the test.
- The test's own comment states the point:
  live workspace analysis proves the chain mutation-free without catalog entries,
  and every link reaches `Object.entries` inside `@monochromatic-dev/module-toml-edit`.
  Verified:
  the comment at `workspace-source-effect.unit.test.ts:50-61`.
- The fixture source is 183 lines and imports `tomlSet` from
  `@monochromatic-dev/module-toml-edit/ts` at `apply-plan.ts:17-19`,
  with the three functions at `:46`,
   `:88`,
   and `:128`.
  Verified:
  `rg --line-number` on that file and `wc --lines`.
- The test locates each function by `SOURCE.indexOf('function <name>')`,
  so any rename in the fixture source breaks it silently.
  Verified:
  `workspace-source-effect.unit.test.ts:67-70`.
- It runs under `test:unit`,
   not under a named CI job;
  `readonly-semantic-bridge.yml:72-78` runs only
  `build:js:node`,
   `test:semantic-bridge-host`,
   and `test:external-consumer-host`.
  Verified by reading that workflow and
  `package/oxlint-plugin/prefer-readonly-parameter-type/mise.toml:38-44`.

Ledger:

- Responsibility:
  a live cross-package call chain whose links reach a workspace package's internals.
- Owner after the rewrite:
  a fixture package the plugin owns.
- Selection status:
  hinges on question Q4.
- Parity test:
  the same three assertions pass against the new fixture path,
  and a guard run with the fixture's workspace dependency removed fails
  (rule `GFP`:
  the guard proves nothing until it is shown to fail).
- Retired behavior:
  the dependency on an unrelated package's source layout.

A constraint worth naming:
after file-enforcer is deleted,
`@monochromatic-dev/module-toml-edit` has no remaining non-fuzz TypeScript consumer.
Verified:
`rg --files-with-matches "module-toml-edit" package/ --type ts`
returns only `module/toml-edit` itself,
 `module/toml-edit.fuzz`,
 `module/jsonc-edit/src/index.ts`,
file-enforcer,
 and this test;
and the `jsonc-edit` hit is a doc comment,
 not an import
(`package/module/jsonc-edit/src/index.ts:4-8`).

Options:

1.  Vendor the fixture into a fixture package that keeps a real workspace dependency.
    A new `package/test-fixture/prefer-readonly-parameter-type`,
    following the sibling `package/test-fixture/oxlint-no-restricted-syntax`,
    holding a frozen copy of `apply-plan.ts`
    and declaring `@monochromatic-dev/module-toml-edit` as a dependency
    so the chain still crosses a package boundary through `/ts`.
    Pros:
    keeps the property under test (live workspace source,
     real cross-package call),
    survives file-enforcer's deletion,
    and stops unrelated churn from breaking the assertions.
    Cons:
    one more package;
    and it keeps `module-toml-edit` alive as a dependency target,
    which may be the right outcome or an artificial one.
2.  Repoint at another live source.
    Cons:
    the measurement shows no other call chain into `module-toml-edit` outside file-enforcer and its fuzz suite,
    so there is nothing to repoint at without first writing one.
3.  Author a minimal chain purpose-built for the assertion.
    Two tiny fixture packages,
     one calling the other.
    Pros:
    smallest surface,
     and every assertion is authored rather than inherited.
    Cons:
    a purpose-built chain risks proving the analyzer handles the shape the test was written for,
    which is weaker evidence than a chain someone wrote for real reasons.
4.  Inline the source text into the test.
    Cons:
    disqualifying.
    The test opens a semantic file and resolves workspace package calls;
    a string with no package around it cannot exercise workspace resolution.

Ranking:
`1 > 3 > 2 > 4`.

- `1 > 3`:
  Option 1 preserves a chain written for real reasons,
  which is the evidence the test's comment claims;
  Option 3 proves the analyzer handles a shape authored to be handled.
- `3 > 2`:
  Option 3 can be written today,
  while Option 2 has no target to point at.
- `2 > 4`:
  Option 2 at least keeps workspace resolution in the test;
  Option 4 removes the thing under test.

### The `sync:files` tasks

What they are:

- `mise.no-env.toml:1120-1128` defines `sync:files`
  (alias `file-enforcer`,
   running `node package/dev-script/file-enforcer/src/cli.ts`)
  and `watch:sync:files` (the same with `--watch`).
  Verified by reading that file.
- The generated `mise.toml:1121-1128` carries the same two tasks.
  Verified by reading that file.
- `sync:files` writes at least
  `./CLAUDE.md`,
   `./LICENSE`,
   `./mise.toml`,
   `./.browserslistrc.resolved.local.json`,
  per-package `LICENSES/*.txt`,
   Cargo manifests,
  the pnpr `config.yaml`,
   the forbidden-strings runtime rules and local appendix,
  and three `package/git-policy/cli/src/optional/*` trees.
  Verified:
  `rg --only-matching 'dest:[^,]*' file-enforcer.config.ts | sort --unique`.
- Two user-facing diagnostics tell people to run it:
  `package/config/pnpr/src/publish-plan.ts:44`
  and `package/config/pnpr/src/publish-missing-versions.ts:205`.
  Verified:
  `rg --line-number "sync:files" .`.
- No GitHub workflow runs it.
  Verified:
  `rg --files-with-matches "sync:files|file-enforcer" .github/`
  returns only `pnpr-publish.yml`,
  whose single hit is a comment at `:15` naming `file-enforcer.config.ts`.

A discrepancy worth recording:

- `doc/decision/monorepo-manager-all-rust.md:76` records that the root `mise.toml` is hand-maintained,
  but `file-enforcer.config.ts:712-724` still generates it from `mise.no-env.toml`,
  and `mise.toml:1` still carries the generated header.
  Verified by reading all three.
  Rule `WC2` still describes the generated arrangement.
  Whichever way this resolves,
   it changes what `sync:files` owns,
  so the migration cannot be scoped without it.

Ledger:

- Responsibility:
  running file enforcement on demand and in watch mode.
- Owner after the rewrite:
  meow.
  On-demand enforcement becomes a meow command;
  watch mode is already assigned to the daemon watcher
  (`doc/planning/monorepo-manager-from-scratch-design.md:2465-2467`
  and `stack-all-rust-rewrite.md:306-313`).
  Verified by reading both.
- Selection status:
  determined by settled requirements,
   so it is recorded rather than asked.
  Mise is removed entirely when meow takes over,
  so both tasks disappear with `mise.toml`;
  nothing migrates them,
   they end.
- Parity test:
  the differential harness the design already requires,
  running both enforcers in a throwaway worktree and comparing bytes
  (`doc/planning/monorepo-manager-from-scratch-design.md:899-903`).
  Verified by reading that passage.
  Plus a test asserting the two pnpr diagnostics name a command that exists.
- Retired behavior:
  the `file-enforcer` task alias,
  and the `--watch` flag on the TypeScript CLI.

The one genuinely open piece is the wording of those two diagnostics,
which cannot be written until meow's command surface is named.
Rule `DGT` requires them to name the affected input and the calls plainly,
so they change from "run `mise run sync:files`" to whatever meow's enforcement command is called.
That is downstream of meow's CLI design,
 not of this research.

Options considered and rejected for completeness:

1.  Point `sync:files` at meow early.
    Rejected:
    it puts both enforcers on the same tree,
    which is exactly what the separate-worktree decision prevents
    (`doc/planning/monorepo-manager-from-scratch-design.md:1266-1268`).
2.  Add a second Mise task,
     `sync:files:meow`,
     for A/B runs.
    Rejected as unnecessary:
    the differential harness runs in a throwaway worktree,
    so a repository task adds a second place to keep in sync for no new evidence.
    It remains the fallback if the user wants the differential runnable by hand.

## Transition plan

While the TypeScript file-enforcer still runs and meow does not yet replace it:

1.  The `exec` replacement is independent of meow and can land first.
    It needs no Rust,
     no daemon,
     and no configuration format.
    Landing it early shrinks the rewrite's parity surface by one public export
    and removes 72 foreign files from vm-builder's type program before the port starts.
2.  The fixture read can also land early,
    and doing so decouples the plugin's tests from file-enforcer's deletion date.
    Its cost is one fixture package.
3.  The perf fixture must keep working against the TypeScript implementation
    for as long as that implementation is the one shipping,
    because it is the only speed baseline the rewrite can be compared against.
    Retiring it before meow's file enforcement exists destroys the baseline.
4.  `sync:files` keeps running the TypeScript CLI unchanged.
    Nothing about it moves until meow supports the full platform matrix,
    at which point Mise and the tasks go together.
5.  `export { exec, } from './pipeline/exec.ts';` at
    `package/dev-script/file-enforcer/src/index.ts:96`
    can be deleted once vm-builder stops importing it,
    but leaving it is harmless:
    the file is deleted wholesale later,
    and `watch/notify.ts` and `package/manager.ts` import `pipeline/exec.ts` directly,
    not through the barrel.
6.  No coexistence protocol is needed for any of this.
    The interim-coexistence concern in
    `stack-all-rust-rewrite.md:1225-1231`
    is about two enforcers writing the same tree,
    which the separate-worktree decision already prevents.
    Verified by reading both.

Ordering within the recommended option:

1.  Add `src/process.ts` with its tests,
     still unused.
2.  Commit,
     so the guard tests exist before the code that depends on them changes
    (rule `GCE`,
     and rule `GFP`'s note that restoring a guard discards uncommitted work).
3.  Repoint the 14 `exec` call sites and the 7 `run` call sites.
4.  Delete the three local `run` declarations and the two `exec` imports.
5.  Edit the manifest,
     regenerate the lockfile with pnpm (rule `LFW`).
6.  Run `lint:types`,
     `lint:oxlint`,
     and `buildAndTest` for the package
    (rules `CM5`,
     `CM6`).
7.  Hand the real VM run to the user,
     in a runbook if they want it recorded.

## Risks

- Error text is user-visible.
  The four probe `catch` blocks print the caught value,
  so a replacement built on anything but `nano-spawn` changes what a user reads
  when `virsh dominfo` or `flatpak info` is absent.
  Verified:
  `build-and-import.ts:285-291` and `:387-393`.
- vm-builder has no tests and no CI job today,
  so a regression surfaces only when the user next builds a VM.
  Verified:
  `rg --files package/dev-script/vm-builder/src`
  and the absence of any workflow naming vm-builder in `.github/workflows/`.
- `sudo` prompts depend on inherited stdio.
  Any future move of vm-builder under a task runner breaks them,
  because meow 0.x has no pseudo-terminal opt-in.
  Verified:
  `doc/planning/monorepo-manager-from-scratch-design.md:803-813`.
- A new package under Option A becomes a published package automatically,
  because `package/config/pnpr/config.yaml` is generated.
  Verified:
  `package/config/pnpr/config.yaml:1-3`.
- The perf fixture writes 240 files under `tmpdir()`,
  and `/tmp` on this machine is a 16 GiB tmpfs with user quotas.
  Verified:
  `findmnt --noheadings --output TARGET,FSTYPE,OPTIONS /tmp`
  reported `tmpfs rw,nosuid,nodev,seclabel,size=16777216k,nr_inodes=1048576,inode64,usrquota`.
- The `mise.toml` generation discrepancy recorded under the `sync:files` item
  means the scope of what `sync:files` owns is currently ambiguous.
- The `prefer-readonly-parameter-type` test locates functions by string search,
  so it can break on a rename in an unrelated package today,
  before any migration.
  Verified:
  `workspace-source-effect.unit.test.ts:67-70`.

## Questions for the user

Rule `QGR`:
only choices that hinge on non-measurable preference or authority appear here.
Rule `QSP`:
they are separable,
 and answering several of them is reachable.
Rule `OPT`:
each carries pros,
 cons,
 and a ranking with the reason deciding each adjacent pair.

### Q1: does the repository get a shared process-execution owner?

This is separable from "what replaces vm-builder's `exec`",
which the settled rules already determine (Option B).
Q1 asks whether,
 in addition,
 the repository should start consolidating
the 63 non-test files that spawn processes directly.

- Option 1a:
  no shared owner;
  vm-builder owns its runner and the incumbent pattern stands.
  Pros:
  no new published package,
  matches 63 existing non-test call sites and the root configuration's own direct use,
  and rule `XNC` says refactor only when necessary.
  Cons:
  the duplication stays,
  and each hand-rolled wrapper keeps its own error mapping.
- Option 1b:
  create `@monochromatic-dev/module-process-run` now,
  and have vm-builder be its first consumer.
  Pros:
  one owner,
  a migration target for the three existing hand-rolled wrappers,
  and a smaller type program for every future consumer.
  Cons:
  a published package for roughly 30 lines,
  one consumer on day one,
  and scope the user has not asked for.
- Option 1c:
  create it later,
   driven by a second consumer.
  Pros:
  defers the decision without blocking this migration.
  Cons:
  "later" has no trigger unless one is written down.

Ranking:
`1a > 1c > 1b`.
`1a > 1c` because a deferral with no trigger is an open item that decays,
while a recorded "no" can be reopened by any future consumer.
`1c > 1b` because both postpone the value,
but `1c` postpones the cost too.

### Q2: when does the `exec` replacement land?

- Option 2a:
  now,
   before the Rust work starts.
  Pros:
  removes a public export from the rewrite's parity surface before porting begins,
  and the change needs nothing from meow.
  Cons:
  touches a package nobody is currently working in,
  and vm-builder's only real verification is a VM build the user must run.
- Option 2b:
  with the rewrite,
   as part of file-enforcer's deletion.
  Pros:
  one migration,
   one review.
  Cons:
  couples an independent change to a long project,
  and the parity surface stays larger for longer.
- Option 2c:
  now,
   but without removing the workspace dependency,
  so the manifest edit and lockfile regeneration happen later.
  Pros:
  smallest diff.
  Cons:
  keeps the 72-file type program for no benefit,
  which is most of the reason to do this at all.

Ranking:
`2a > 2b > 2c`.
`2a > 2b` because the change has no dependency on meow,
and a smaller parity surface makes the port easier to judge.
`2b > 2c` because `2b` eventually gets the whole benefit,
while `2c` pays the cost of the change and keeps the cost of the dependency.

### Q3: does the rewrite get a speed gate?

The design's risk list has a byte-identical output gate but no performance gate.

- Option 3a:
  yes,
   with a Rust bench against the existing corpus generator.
  Pros:
  evidence that the rewrite is not slower,
  on the same 240-file corpus the current benchmarks use.
  Cons:
  the corpus generator must survive its package,
  and a bench binary is new work in meow's worktree.
- Option 3b:
  no;
  retire the perf fixture with the TypeScript implementation.
  Pros:
  no work.
  Cons:
  the rewrite ships with no speed evidence,
  and Rust being faster is an assumption until measured.
- Option 3c:
  freeze the current numbers,
   then retire.
  Pros:
  cheapest baseline.
  Cons:
  rule `QNB`:
  numbers from one run on one machine are not a gate.

Ranking:
`3a > 3c > 3b`.
`3a > 3c` because only a re-runnable bench can show a regression after the port,
while frozen numbers cannot be compared under the same conditions.
`3c > 3b` because a recorded number with its caveat still beats no number.

### Q4: where does the `prefer-readonly-parameter-type` fixture live?

- Option 4a:
  a new `package/test-fixture/prefer-readonly-parameter-type`
  holding a frozen copy of `apply-plan.ts`
  and depending on `@monochromatic-dev/module-toml-edit`.
  Pros:
  keeps a real cross-package chain,
  survives file-enforcer's deletion,
  and stops unrelated churn from breaking assertions.
  Cons:
  one more package,
  and it keeps `module-toml-edit` as a dependency target
  when nothing else in TypeScript will call it.
- Option 4b:
  author a minimal two-package chain for the assertion.
  Pros:
  smallest surface.
  Cons:
  proves the analyzer handles a shape authored to be handled.
- Option 4c:
  leave it and fix it when file-enforcer is deleted.
  Pros:
  no work now.
  Cons:
  the test already breaks on any rename inside `apply-plan.ts`,
  so the fragility is present today,
   not only at deletion.

Ranking:
`4a > 4b > 4c`.
`4a > 4b` because a chain written for real reasons is stronger evidence
than one written to satisfy the assertion.
`4b > 4c` because both are work,
but `4b` also removes a fragility that exists today.

### Q5: is the root `mise.toml` generated or hand-maintained?

Recorded as hand-maintained in
`doc/decision/monorepo-manager-all-rust.md:76`,
still generated in
`file-enforcer.config.ts:712-724`.
This is the user's call because it is an authority question about which record stands,
and it changes what `sync:files` owns.

- Option 5a:
  the decision stands;
  remove the generator and hand-maintain `mise.toml`,
  folding the `[env]` block into the file.
  Pros:
  the record and the code agree.
  Cons:
  the `_.path` entries for every workspace bin directory become manual,
  and the file is 1,100 or more lines.
- Option 5b:
  the code stands;
  correct the decision record to say `mise.toml` stays generated until Mise is removed.
  Pros:
  no behavior change,
  and the generation dies with Mise anyway.
  Cons:
  a decision record is corrected rather than honored.
- Option 5c:
  leave both as they are.
  Pros:
  none beyond doing nothing.
  Cons:
  a record and the code disagree,
  which is exactly the state rule `DCK` says to remove.

Ranking:
`5b > 5a > 5c`.
`5b > 5a` because the generator dies with Mise regardless,
so hand-maintaining a generated file buys nothing before then.
`5a > 5c` because either resolution beats leaving a record and the code in conflict.

## Evidence index

Every command in this document was run from `/var/home/user/Monochromatic`
with no repository mutation.

- Consumer discovery:
  `rg --line-number --glob '!node_modules' --glob '!**/dist/**' "dev-search" ...`
  and the specific searches quoted beside each claim.
- Broad sanity check (rule `QRY`):
  `rg --line-number --glob '!node_modules' --glob '!**/dist/**' --glob '!**/.cache/**' "dev-script-file-enforcer" .`
  returned lockfile entries,
   the root manifest,
   the root configuration,
  the perf fixture,
   vm-builder,
   file-enforcer itself,
   the pnpr configuration,
  and one paused-package handover.
- Type program size:
  the `tsbuildinfo` search recorded under "The cost of the barrel import"
  reported 72.
- Import closure:
  `node closure.ts package/dev-script/file-enforcer/src/index.ts` reported 75,
  `node closure.ts package/dev-script/file-enforcer/src/pipeline/exec.ts` reported 3.
  The walker is `closure.ts` beside this document,
   in the session scratchpad.
- Spawn pattern census:
  the `nano-spawn` search recorded under
  "Direct spawning is the repository's incumbent pattern"
  reported 123,
  and the same piped through `rg --invert-match "test" | wc --lines` reported 63.
- Filesystem:
  `findmnt --noheadings --output TARGET,FSTYPE,OPTIONS /tmp`.

Unverified claims,
 marked as such where they appear:

- Node's default `maxBuffer` for `execFile` was not read from Node's documentation in this session.
- `nano-spawn`'s default stdio is inferred from the repository's own tests rather than read from its source.

## Rule slips in this session

None recorded.
All Bash calls stayed within rule `1CB`'s three `&&`-chained steps,
used long-form flags per rule `LFF`,
passed an explicit path to `rg` per rule `RGP`,
and wrapped no routine verification in an external `timeout` per rule `TMO`.
One command failed because a backtick inside an `rg` pattern triggered shell command substitution;
it was rerun with a pattern containing no backtick,
and no mutation occurred.
