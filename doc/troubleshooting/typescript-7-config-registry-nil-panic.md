# TypeScript 7.1.0-beta's `tsc --api` server panics on a deleted config entry, hiding `lint:oxlint` findings

The TypeScript 7.1 API server that this repository's Oxlint plugin drives through
`typescript/sync` panics with a nil `dirty.SyncMapEntry` while it looks up the tsconfig
for an opened file.
The panic kills that server,
the plugin catches the dead channel and skips each file with
a warning on standard error,
and Oxlint still exits 0 when nothing else was found,
so a crashed run and a clean run look identical from the outside.

## Symptom

One `mise run //package/<package>:lint:oxlint` run prints this on standard error and
nothing else unusual (text recorded by the coordinator on 2026-11-09,
frames trimmed to
the ones this document follows):

```text
panic: runtime error: invalid memory address or nil pointer dereference
[signal SIGSEGV: segmentation violation code=0x1 addr=0x8 pc=0xbf31dc]
goroutine 215 [running]:
github.com/microsoft/TypeScript/tsc/internal/project/dirty.(*SyncMapEntry[...]).ChangeIf
	github.com/microsoft/TypeScript/tsc/internal/project/dirty/syncmap.go:128 +0x3c
github.com/microsoft/TypeScript/tsc/internal/project.(*configFileRegistryBuilder).acquireConfigForFile
	github.com/microsoft/TypeScript/tsc/internal/project/configfileregistrybuilder.go:319 +0x27e
github.com/microsoft/TypeScript/tsc/internal/project.(*configFileRegistryBuilder).findOrAcquireConfigForFile
	github.com/microsoft/TypeScript/tsc/internal/project/configfileregistrybuilder.go:135 +0x37
github.com/microsoft/TypeScript/tsc/internal/project.(*ProjectCollectionBuilder).findOrCreateDefaultConfiguredProjectWorker.func2
	github.com/microsoft/TypeScript/tsc/internal/project/projectcollectionbuilder.go:1108 +0xe8
github.com/microsoft/TypeScript/tsc/internal/core.BreadthFirstSearchParallelEx[...].func1-range1.1
	github.com/microsoft/TypeScript/tsc/internal/core/bfs.go:115 +0x15d
```

What the run does around that text:

- It ended with status 0 whenever no finding was planted,
  so CI read the crashed run as
  clean.
- The `typescript/*` rules kept reporting,
  because `tsgolint` is a separate process and
  was still alive.
- A planted floating promise was still reported by
  `typescript(no-floating-promises)` in a run that panicked,
  because that rule answers from
  `tsgolint`,
  a process of its own.
- A planted `values: string[]` parameter was reported by
  `prefer-readonly-parameter-type(prefer-readonly-parameter-types)` in a run that did not
  panic.
  No recorded run tested that rule in a run that panicked,
  so the loss of its finding was
  not measured directly;
  the code path under "Why the run still exits 0" is what establishes
  that a lost channel takes its findings with it.
- Nothing in standard output or the exit status named the missing analyzer.
  The only trace was the plugin's own warning on standard error,
  in a shape ordinary runs
  also print:

  ```text
  [warn] [...] [preference] semantic evidence unavailable for <file>: <reason>: SemanticBridgeError: ...
  ```

The coordinator measured the panic in three of five runs of
`mise run //package/rolldown-plugin/import-attributes:lint:oxlint`.
This worktree saw none in ten runs of the same task,
recorded under "Verification".

## Which process panics

The panic comes from the TypeScript 7.1 API server,
which Oxlint starts on behalf of the
JavaScript plugin,
and not from `tsgolint` and not from Oxlint itself.
Four independent readings agree.

### The process tree

One run,
sampled every 200 ms,
held these processes (parent to child,
identifiers as
`ps` printed them):

```text
3120099 node --input-type=module-typescript -e <mise task shell>
 3120133 node <worktree>/package/monorepo-task/core-exec/dist/final/node/oxlint-wrapper.mjs
  3120159 node <store>/oxlint/1.92.0/.../node_modules/oxlint/bin/oxlint --format=default
   3120720 <store>/@typescript/typescript-linux-x64/7.1.0-beta/.../lib/tsc --api --cwd <package dir> --useCaseSensitiveFileNames=true --callbacks=readFile,fileExists
   3120762 <store>/@typescript/typescript-linux-x64/7.1.0-beta/.../lib/tsc --api --cwd <package dir> --useCaseSensitiveFileNames=true
   3122835 node <store>/oxlint-tsgolint/7.0.2006/.../bin/tsgolint.js headless
    3122863 <store>/@oxlint-tsgolint/linux-x64/7.0.2006/.../tsgolint headless
```

Two API servers exist per run because the plugin starts two:
the rule bridge at
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/semantic-bridge-lifecycle.ts:121`
passes an overlay filesystem,
which is what produces the `--callbacks` argument,
and the
external implementation project at
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/external-implementation-project.ts:84`
starts a second without one.

### The binary that holds the frames

`go version -m` on the API server binary
`<store>/@typescript/typescript-linux-x64/7.1.0-beta/.../lib/tsc` reports:

```text
path	github.com/microsoft/TypeScript/tsc/cmd/tsc
mod	github.com/microsoft/TypeScript/tsc	(devel)
build	vcs.revision=3e9236361fa97a9e6c685f4833ac457ae2d8c50a
build	vcs.time=2026-10-19T20:38:54Z
build	vcs.modified=true
```

That module path is the one the panic's frames print,
and the binary carries the frame
symbols themselves:

```text
$ strings -a <store>/@typescript/typescript-linux-x64/7.1.0-beta/.../lib/tsc | rg 'acquireConfigForFile' | head -2
github.com/microsoft/TypeScript/tsc/internal/project.(*configFileRegistryBuilder).acquireConfigForFile
github.com/microsoft/TypeScript/tsc/internal/project.(*configFileRegistryBuilder).acquireConfigForFile.func2
```

The same two commands on the `tsgolint` binary from `oxlint-tsgolint` 7.0.2006 find the
same function under a different module path and a different name:

```text
$ go version -m <store>/@oxlint-tsgolint/linux-x64/7.0.2006/.../tsgolint | rg 'path|typescript-go' | head -3
path	github.com/typescript-eslint/tsgolint/cmd/tsgolint
mod	github.com/typescript-eslint/tsgolint	v0.25.1-0.20261105135319-420f4f134c48+dirty
dep	github.com/microsoft/typescript-go	(devel)

$ strings -a <store>/@oxlint-tsgolint/linux-x64/7.0.2006/.../tsgolint | rg 'acquireConfigForFile' | head -2
github.com/microsoft/typescript-go/internal/project.(*ConfigFileRegistryBuilder).acquireConfigForFile
```

The panic prints `(*configFileRegistryBuilder)`,
lowercase,
and
`github.com/microsoft/TypeScript/tsc/internal/project/dirty`,
so it cannot have come from
the `tsgolint` binary,
whose corresponding names are `ConfigFileRegistryBuilder` and
`github.com/microsoft/typescript-go/internal/project/dirty`.

### The channel that reports the crash

The plugin imports `typescript/sync`,
whose client spawns the platform binary and reports a
dead child as a read failure,
at
`node_modules/typescript/dist/api/syncChannel.js:131`:

```js
            this.child = spawn(exe, args, {
                stdio: ["pipe", "pipe", "inherit"],
            });
```

Standard error is inherited,
so the panic lands in Oxlint's own standard error,
which is
what the wrapper at
`package/monorepo-task/core-exec/src/oxlint-wrapper.ts` captures and forwards.
The child's death reaches JavaScript as one of the markers in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/semantic-channel-failure.ts:19`,
`Unexpected EOF while reading from child process`.

### The standalone reproduction

Opening one file whose nearest tsconfig matches no files makes that API server panic with
no Oxlint and no plugin in the picture,
printing the same frames,
the same
`addr=0x8`,
and the same `pc=0xbf31dc` as the recorded run.
The harness is under "Verification".

## Root cause

### The frames, in order

`findOrCreateDefaultConfiguredProjectWorker` starts a parallel breadth-first search over
candidate config files,
at
[`projectcollectionbuilder.go:1086`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/projectcollectionbuilder.go#L1086-L1086):

```go
	search := core.BreadthFirstSearchParallelEx(
```

Every node of a level is visited in its own goroutine,
at
[`bfs.go:100`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/core/bfs.go#L100-L115),
and the visit itself is the callback at line 115:

```go
			go func(i int, j *breadthFirstSearchJob[N]) {
				...
				isResult, stop := visit(j.node)
```

The visit asks the config registry for the config,
with the create load kind,
at
[`projectcollectionbuilder.go:1108`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/projectcollectionbuilder.go#L1108-L1108):

```go
			config := b.configFileRegistryBuilder.findOrAcquireConfigForFile(node.configFileName, configFilePath, path, node.loadKind, node.logger.Fork("Acquiring config for open file"))
```

`findOrAcquireConfigForFile` forwards the create case at
[`configfileregistrybuilder.go:135`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/configfileregistrybuilder.go#L135-L135):

```go
		return c.acquireConfigForFile(configFileName, configFilePath, filePath, logger)
```

`acquireConfigForFile` stores a fresh entry,
throws away the boolean that says whether its
value was stored,
and changes whatever came back,
at
[`configfileregistrybuilder.go:316`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/configfileregistrybuilder.go#L316-L319):

```go
	entry, _ := c.configs.LoadOrStore(configFilePath, newConfigFileEntry(c.hasRelativePatternCapability, configFileName))
	var needsRetainOpenFile bool
	var contentMappersChanged bool
	entry.ChangeIf(
```

### Why the receiver is nil

`dirty.SyncMap.LoadOrStore` has four ways out,
and two of them answer a nil entry with
`false`,
at
[`syncmap.go:226`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/dirty/syncmap.go#L226-L259):

```go
	if baseValue, ok := m.base[key]; ok {
		if dirty, ok := m.dirty.Load(key); ok {
			dirty.mu.Lock()
			defer dirty.mu.Unlock()
			if dirty.delete {
				return nil, false
			}
			return dirty, true
		}
		...
	entry, loaded := m.dirty.LoadOrStore(key, &SyncMapEntry[K, V]{...})
	if loaded {
		entry.mu.Lock()
		defer entry.mu.Unlock()
		if entry.delete {
			return nil, false
		}
	}
	return entry, loaded
```

A caller that ignores the boolean cannot tell "your value is stored,
use this entry" from
"another goroutine deleted this key,
so there is no entry".
`acquireConfigForFile` is such a caller,
and `ChangeIf` starts by locking the receiver's
mutex,
at
[`syncmap.go:127`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/dirty/syncmap.go#L127-L128):

```go
func (e *SyncMapEntry[K, V]) ChangeIf(cond func(V) bool, apply func(V)) bool {
	e.mu.Lock()
```

`SyncMapEntry` declares the pointer field first and the mutex second,
at
[`syncmap.go:47`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/dirty/syncmap.go#L47-L48):

```go
	m  *SyncMap[K, V]
	mu sync.Mutex
```

A nil receiver therefore puts `&e.mu` at offset 8,
which is the address the recorded run
faults on:
`addr=0x8`,
`pc` in `ChangeIf`,
and the frame's line 128 is the line that locks.
A five-line Go program over a mirror of that struct prints the same offset:

```text
offsetof mu: 8
```

The goroutine's own printed arguments are not usable as evidence here.
The recorded trace shows a non-zero value in the receiver's slot,
which the Go runtime
prints from registers that the crashing instruction no longer owns;
the faulting address,
the frame's line,
and the struct layout are what identify the nil receiver.

### Where the tombstone comes from

`Cleanup` marks every config entry with no remaining project,
open file,
or extending config
as deleted,
at
[`configfileregistrybuilder.go:812`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/configfileregistrybuilder.go#L812-L816):

```go
		entry.DeleteIf(func(value *configFileEntry) bool {
			shouldDelete := len(value.retainingProjects) == 0 && len(value.retainingOpenFiles) == 0 && len(value.retainingConfigs) == 0
```

The tombstone stays in the builder's dirty map until `Finalize` folds that map into the next
registry,
at
[`configfileregistrybuilder.go:80`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/configfileregistrybuilder.go#L80-L80).
Every acquire of the same config path inside that window reads the tombstone and gets nil.
`ProjectCollectionBuilder.cleanupConfiguredProjects` runs that cleanup at
`internal/project/projectcollectionbuilder.go:575`,
so the window opens whenever a project
stops retaining a config and a later open in the same update asks for it again,
and the
parallel search widens it,
since a sibling goroutine can be inside `acquireConfigForFile`
while another phase of the same update cleans up.

This is why the crash is intermittent:
it needs a deleted entry to still be in the dirty
map when a later acquire arrives,
which depends on which files and projects a run touches
and in what order.
The standalone harness makes it deterministic by opening a file whose
nearest tsconfig has no files of its own,
which walks the same "no configured project"
search on a freshly created registry.

The sibling call site in the same file shows the intended contract,
guarding on the boolean,
at
[`configfileregistrybuilder.go:177`](https://github.com/microsoft/TypeScript/blob/3e9236361fa97a9e6c685f4833ac457ae2d8c50a/tsc/internal/project/configfileregistrybuilder.go#L177-L177):

```go
			if loaded {
```

`acquireConfigForFile` (line 316) and `acquireConfigForProject` (line 286) both discard it.

### Why the run still exits 0

The panic kills the API server,
so the plugin's next request fails with a dead-channel
error.
The bridge retires,
reported by the marker list in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/semantic-channel-failure.ts:19`,
and the rule visitor catches the failure,
logs one warning,
and skips the file,
at
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/readonly-rule-visitor.ts:109`:

```ts
      catch (error) {
        rl.warn(
          `semantic evidence unavailable for ${context.filename}: ${...}`,
        );
      }
```

Oxlint counts the diagnostics it produced,
and a skipped file contributes none,
so its exit
status says what the surviving analyzers found.
The wrapper passes that status through,
so a crashed run and a clean run are the same
status.

### The same defect upstream today

Neither branch changed on the default branch at `bbcbce9`:
`LoadOrStore` still answers
`(nil, false)` for a tombstone at
[`syncmap.go:244`](https://github.com/microsoft/TypeScript/blob/bbcbce924a13164ca05f0276f34433925954971c/tsc/internal/project/dirty/syncmap.go#L244-L244),
and `acquireConfigForFile` still discards the boolean at
[`configfileregistrybuilder.go:319`](https://github.com/microsoft/TypeScript/blob/bbcbce924a13164ca05f0276f34433925954971c/tsc/internal/project/configfileregistrybuilder.go#L319-L322).

## Verification

Versions under test,
all resolved from this worktree:

- `typescript` 7.1.0-beta,
  pinned in the catalog at `pnpm-workspace.yaml:196`;
  platform
  package `@typescript/typescript-linux-x64` 7.1.0-beta,
  binary `lib/tsc`,
  build
  `vcs.revision=3e9236361fa97a9e6c685f4833ac457ae2d8c50a`,
  built with go1.27.2.
- `oxlint` 1.92.0.
- `oxlint-tsgolint` 7.0.2006,
  embedding `github.com/microsoft/typescript-go`.
- Node 26.13.0,
  on Linux x86-64.

Source excerpts come from a fetch of
`github.com/microsoft/TypeScript` at
`3e9236361fa97a9e6c685f4833ac457ae2d8c50a`,
the revision the shipped binary records.

### Harness

Two folders and one short script reproduce the crash without Oxlint.
The folder whose
tsconfig matches no files makes it panic;
the other makes the same script succeed,
which
shows the harness can report both answers.

```ts
// open-file.ts, run from a folder holding tsconfig.json: node open-file.ts ./x.ts
import { API, } from "typescript/sync";

const api = new API({ cwd: process.cwd(), },);
api.createSnapshot({ openFiles: [process.argv[2] ?? "./x.ts",], },);
console.log("snapshot created");
api.close();
```

The script resolves `typescript` from a `node_modules` that links this worktree's
`node_modules/typescript`,
either by running inside a folder with that link or by copying
the link beside the script.

### Layouts that panic

```text
# panics/tsconfig.json: {"files":[]}    panics/x.ts: export const x = 1;
$ node open-file.ts ./x.ts
panic: runtime error: invalid memory address or nil pointer dereference
[signal SIGSEGV: segmentation violation code=0x1 addr=0x8 pc=0xbf31dc]

goroutine 150 [running]:
github.com/microsoft/TypeScript/tsc/internal/project/dirty.(*SyncMapEntry[...]).ChangeIf(0xecbee0, 0x326107602060, 0x326107b68d20?)
	github.com/microsoft/TypeScript/tsc/internal/project/dirty/syncmap.go:128 +0x3c
...
file:///.../typescript/dist/api/syncChannel.js:413
        return new Error(`Unexpected EOF while reading from child process (${detail})`);
...
$ echo $?
1
```

A second layout fails the same way,
with `tsconfig.json` holding
`{"include":["src"]}` and an empty `src` directory,
then opening `./x.ts`.
That is the same pair of rows the upstream report tabulates:
a nearest tsconfig that
matches no files of its own,
and an opened file it does not include.

### Layouts that succeed

```text
# clean/tsconfig.json: {"include":["src"]}   clean/src/a.ts: export const a = 1;
$ node open-file.ts ./src/a.ts
snapshot created
$ echo $?
0
```

A folder with no `tsconfig.json` at all also prints `snapshot created` and exits 0.

### One layout that does not panic here

The upstream report's own minimal case,
`tsconfig.json` holding `{}` and a `w.mjs` beside it,
printed `snapshot created` on this machine instead of panicking.
The report measured macOS arm64 with Node 24;
this measurement is Linux x86-64 with
Node 26.13.0,
and the two differ in how an open file with no `allowJs` reaches the default
configured project.
Not explained,
and not needed for the diagnosis:
the panic site and its trigger rows
reproduce.

### The lint task, ten sequential runs

The package task was run ten times in a row,
the whole sequence inside one
`systemd-run --user --scope` limited to `CPUQuota=300%`,
`MemoryMax=12G`,
`TasksMax=768`,
and `IOWeight=10`,
with two findings planted in one new file
(`values: string[]` for the repository's readonly rule,
and one dropped promise for
`typescript(no-floating-promises)`).
The planted file was deleted afterwards.

- Run 1:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 2:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 3:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 4:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 5:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 6:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 7:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 8:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 9:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.
- Run 10:
  no panic;
  readonly finding reported;
  floating-promise finding reported;
  exit 1.

Every run also printed one benign `semantic evidence unavailable ...` warning for the
package's `task.ts`,
whose own nearest config does not include it.
Both planted findings
appeared in all ten runs,
so the panic did not reproduce here and no run could test what a
panicking lint run reports.
What separated the two worktrees was not identified:
the cause under "Root cause" makes both
the file set a run opens and the interleaving of its updates matter,
and neither was varied
here.

What the ten runs do establish:

- exit 1 came from the planted findings,
  so the planted-file harness reaches both rules;
- the readonly rule answers through the API server,
  so it is the rule a dead server removes;
- the floating-promise rule answers from `tsgolint`,
  a process of its own,
  which is why the
  recorded panicking run still reported its planted promise while the API server was dead.

## Verified workarounds

### Fail the run when an analysis child crashes

This repository's own wrapper is the tool that owns every `lint:oxlint` run,
so the guard
belongs there:
`package/monorepo-task/core-exec/src/oxlint-wrapper.ts`,
landed in commit
`bba304d1b1`.
It reads the crash report from the captured standard error,
forwards Oxlint's own output,
then throws:

```text
[task-oxlint] oxlint reported a crash in one of its analysis processes, so this run's findings are incomplete: panic: runtime error: invalid memory address or nil pointer dereference. Find or report that crash; a run whose analyzer died cannot say the code is clean.
```

The recognition lives in
`package/monorepo-task/core-exec/src/oxlint-analysis-crash.ts`:
a crash header
(`panic: ` or `fatal error: `) plus the Go runtime's goroutine dump line or its signal line,
in one linear pass over the captured stream.

Tradeoffs:

- The run exits non-zero with a message,
  which is what makes CI stop trusting it,
  and it
  cannot recover the findings the crash removed;
  nothing in the wrapper can re-run the dead
  analyzer's work.
- A `--fix` run is judged by its final convergence pass.
  A crash that only an earlier pass
  saw is not reported.
- Any Go child of Oxlint counts,
  `tsgolint` included,
  so a crash in a type-aware rule fails
  the run for the same reason:
  the surviving analyzers do not speak for the dead one.

The order was a failing control first,
then the fix,
in two commits:

```text
[task-oxlint analysis integrity] [fails a run whose analyzer crashed while reporting a clean summary] [FAIL] (40ms) AssertionError: expected +0 to equal 1
[task-oxlint analysis integrity] [fails a run whose analyzer crashed beside a reported diagnostic] [FAIL] (48ms) AssertionError: expected 'panic: runtime error: invalid memory …' to include 'findings are incomplete'
```

After the fix the same three test files pass:

```text
[detectNativeAnalysisCrash] [PASS] finds the header of a panic report carrying a goroutine dump, finds the header of a report carrying only the signal line, finds a Go fatal error, which no recover intercepts, finds a report that follows ordinary Oxlint output, finds a report whose last line carries no newline, ignores a clean run, ignores a crash header with no runtime report beside it, ignores a goroutine dump with no crash header above it, ignores a message that merely mentions a panic, ignores a goroutine line whose number is not a number
[task-oxlint analysis integrity] [PASS] fails a run whose analyzer crashed while reporting a clean summary, fails a run whose analyzer crashed beside a reported diagnostic, passes a clean run and says nothing about a crash, keeps the findings status and says nothing about a crash, passes a run whose standard error quotes a crash header alone
```

### Read the crash instead of the status

Where the wrapper is not the caller,
the same rule applies by hand:
a run whose standard
error carries a Go crash report is not evidence,
whatever status it reports.
The tradeoff is that the reader has to inspect standard error,
since no status encodes it.

### Keeping the analyzer alive

Not verified,
and not recommended from what is known:
the crash needs a deleted config entry
in the builder's dirty map,
which a sequence of opens and closes such as the plugin's own can
produce,
and no consumer-side change to the plugin was measured to avoid it.

## What does not work

- Treating exit 0 as evidence.
  The crash removes findings and leaves the status clean,
  which
  is the whole defect.
- Treating `semantic evidence unavailable` as the crash signal on its own.
  Ordinary runs
  print it for files whose nearest tsconfig does not include them;
  all ten recorded lint
  runs printed one for `task.ts` while nothing crashed.
- Retrying the lint run.
  The panic is intermittent,
  so a retry can pass while the crashed
  run's findings stay unknown,
  and the retry hides the incident.
- Upgrading.
  The upstream report records the same panic on `7.1.0-dev.20261027.1` and
  `7.1.0-dev.20261029.1`;
  this document did not re-test any other version,
  and the default
  branch still has both halves of the defect.
- Clearing caches.
  Not attempted,
  and nothing in the cause suggests a cache.

## Upstream filing decision

Filing is out of scope for this task,
which ends at a draft.
The duplicate search found reports already open on the same site,
so even an authorized
filing would be a comment rather than a new issue.

### Duplicate and in-progress checks

Recorded 2026-11-09:

- `gh search issues --repo microsoft/TypeScript 'acquireConfigForFile'` returned issue
  [#65054](https://github.com/microsoft/TypeScript/issues/65054),
  "API server panics in acquireConfigForFile when opening a JS file a tsconfig does not
  include",
  open since 2026-10-30,
  with the same five frames and the same
  `syncmap.go:128` site.
  Its comments tabulate the trigger layouts,
  including both that reproduce here.
- `gh search issues --repo microsoft/TypeScript 'LoadOrStore'` returned nothing relevant;
  the related open report is
  [#64911](https://github.com/microsoft/TypeScript/issues/64911),
  "API:
  `snapshot.update` with both `removePrograms` and `createPrograms` crashes tsgo
  (nil dereference in `ProjectCollectionBuilder.HandleAPIRequest`)".
- `gh search prs --repo microsoft/TypeScript 'syncmap'` returned
  [#65072](https://github.com/microsoft/TypeScript/pull/65072),
  "fix(65071):
  resurrect dirty.SyncMap entries after Delete in LoadOrStore",
  closed on
  2026-11-03 without merging,
  after review comments on reusing the entry identity.
  Its body names the same nil return and the same missing boolean check.
- No open pull request claims this work today,
  and the code path shows no maintainer change
  since `7502ae1a2d` (2026-11-05,
  "LSP perf optimizations"),
  which touched
  `dirty/syncmap.go` without changing `LoadOrStore`.
- `.out-of-scope/` holds no exemption for TypeScript or for this bug class.
  `typescript-project-references.md` is about not adopting project references,
  and
  `low-impact-typescript-formatting.md` about formatter rules,
  so neither applies.

### The six-constraint check

1.  Is it really upstream's fault:
    yes.
    A nil return from `LoadOrStore` is upstream's own contract break,
    and the panicking
    dereference is in upstream's own registry builder.
    No consumer input makes an unchecked nil dereference correct.
2.  Can upstream fix it:
    yes.
    The minimal fix is in `dirty.SyncMap.LoadOrStore`:
    a tombstoned key must not answer nil
    to a caller that supplied a value.
    The closed pull request above is one shape of it;
    replacing the tombstone with the
    caller's value,
    or returning a third outcome that names the delete,
    are others.
    The call sites that discard the boolean could be guarded as well,
    following line 177.
3.  Are they supporting this use case:
    yes.
    `internal/api` exists to serve external consumers,
    `createSnapshot({ openFiles })` is a
    documented call,
    and the report itself reproduces on the shipped package.
4.  Would the repo welcome our contribution:
    not assessed.
    Filing is out of scope here,
    so the contribution policy,
    issue forms,
    and
    AI-assistance rules were not read.
5.  Will they likely fix it:
    not assessed.
    The duplicate and its fix attempt show engagement,
    but no maintainer statement was read.
6.  Have we prototyped a minimal fix:
    no.
    Constraints 4 and 5 were left unassessed,
    and a prototype is what makes a filing
    credible.

### Risk to the poster

A comment here would go on a thread strangers and maintainers read,
under a real person's
account,
and some projects run trust systems that close or denounce filings they judge
low-effort.
This repository's instruction keeps the work at a draft,
so nothing is recommended and the
decision to post belongs to that person.
Before any posting,
constraints 4 and 5 must be assessed,
the trust-system search run,
and
the draft re-read against what #65054 already contains.

### Draft comment, not for posting

```md
Independent reproduction from a consumer, plus the consequence that matters for CI.

Oxlint 1.92.0 with `oxlint-tsgolint` 7.0.2006 runs our repository's Oxlint JS plugin, which
drives this API server through `typescript/sync`. A type-aware lint run panicked in this
exact frame chain three times in five runs on one worktree, and in none of ten runs on
another. The plugin treats a dead channel as "semantic evidence unavailable" for each file
and carries on, and Oxlint's exit status counts only the analyzers that survived, so a run
whose only other finding was absent exited 0 while that rule had reported nothing.
That silent-success shape is not in #65054's body or its comments so far, and it follows
from the same root cause: nothing in Oxlint's status says an analyzer died.

Two additions to the diagnosis, both at 3e9236361fa97a9e6c685f4833ac457ae2d8c50a:

- `dirty.SyncMap.LoadOrStore` answers `(nil, false)` in two branches (base-map tombstone and
  dirty-map tombstone), so a caller cannot distinguish "stored your value" from "deleted".
- `acquireConfigForFile` (line 316) and `acquireConfigForProject` (line 286) both discard
  that boolean; the `updateExtendingConfigs` site at line 177 guards on it, which is the
  intended contract.

On `addr=0x8`: `SyncMapEntry` holds `m` then `mu`, so a nil receiver puts the mutex at
offset 8, which matches the fault address and line 128 of `ChangeIf`.
```

## Related records

- `doc/troubleshooting/typescript-go-tuple-type-panic.md` records a different TypeScript
  panic,
  an `interface conversion` in the API's type serializer.
  Its cause and its fix path
  are unrelated to this one;
  the two share only the surface,
  an Oxlint run.
- `doc/troubleshooting/oxlint-typescript-semantic-bridge-epipe-teardown.md` covers the
  channel teardown errors this crash surfaces as.
