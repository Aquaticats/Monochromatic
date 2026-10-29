# rsync 3.5.0: vanishing TypeScript cache aborts a mutation-test shard with code 24

## Symptom

During a `toml-edit` mutation run on `src/set-replace.ts`,
 `package/mutation-test/cli` launched a disposable Podman shard and its source copy failed:

```text
file has vanished: "/src-ro/package/upstream-fork/deepmerge-ts.fuzz/.cache/typescript/tsconfig.tsbuildinfo"
rsync warning: some files vanished before they could be transferred (code 24)
```

The mutation CLI recorded `container failed without report` for that shard and exited with
 `1 infrastructure errors; see report ...`.
The affected input was a concurrently changed cache file in another package,
 not a `toml-edit` mutant or test assertion.

## Root cause

`package/mutation-test/cli/src/container/worktree.ts:73-89` copies the entire mounted repository
 into the shard's writable work tree:

```ts
// package/mutation-test/cli/src/container/worktree.ts
await spawn('rsync', [
  '--archive',
  '--delete',
  ...RSYNC_EXCLUDES.flatMap(function excludeArgs(pattern,) {
    return ['--exclude', pattern,];
  },),
  `${SOURCE_MOUNT}/`,
  `${WORK_MOUNT}/`,
]);
```

Before this fix,
 the exclusion list omitted generated `.cache` trees.
A different package's type check replaced its `.cache/typescript/tsconfig.tsbuildinfo`
 while rsync was copying from the read-only repository mount.
The read-only mount prevents writes by the shard,
 but it does not freeze concurrent host writes.

In upstream rsync source at `RsyncProject/rsync` commit `f884cfe6892f391487fb9a17fef98dee9f9a1360`,
 `flist.c:1684-1691` sets a vanished-file flag after a source entry disappears:

```c
/* upstream flist.c */
io_error |= IOERR_VANISHED;
rprintf(c, "file has vanished: %s\n", full_fname(thisname));
```

Upstream `cleanup.c:208-217` converts that flag into the exit code,
 and `errcode.h:44` names it:

```c
/* upstream cleanup.c and errcode.h */
if (io_error & IOERR_VANISHED)
    exit_code = RERR_VANISHED;
#define RERR_VANISHED 24 /* file(s) vanished on sender side */
```

Upstream `log.c:99` supplies the `some files vanished before they could be transferred` text.
The container log identified rsync `3.5.0-g483b5efc`;
 the independently read upstream source commit is not asserted to be that exact build.

## Verification

- The failing shard log captured `rsync` code 24 before it could write a shard report.
- `package/mutation-test/cli/src/container/worktree.unit.test.ts` was red before the fix
  because the rsync list lacked `**/.cache`.
- After the change,
  `mise run //package/mutation-test/cli:buildAndTest`,
  `lint:types`,
  and `lint:oxlint` passed.
- A disposable rsync fixture copied `package/example/kept.ts` while excluding
  `package/example/.cache/typescript/tsconfig.tsbuildinfo` using `--exclude '**/.cache'`.

Run the same fixture without touching this repository's state:

```sh
node --input-type=module -e "
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { access, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
const root = await mkdtemp(join(tmpdir(), 'rsync-cache-check-'));
await using cleanup = {
  async [Symbol.asyncDispose]() { await rm(root, { recursive: true, force: true }); },
};
const source = join(root, 'source');
const destination = join(root, 'destination');
await mkdir(join(source, 'package/example/.cache/typescript'), { recursive: true });
await mkdir(destination);
await writeFile(join(source, 'package/example/kept.ts'), 'export const kept = true;');
await writeFile(join(source, 'package/example/.cache/typescript/tsconfig.tsbuildinfo'), 'ephemeral');
execFileSync('rsync', [
  '--archive', '--delete', '--exclude', '**/.cache',
  source + '/', destination + '/',
]);
await access(join(destination, 'package/example/kept.ts'));
await assert.rejects(access(join(destination, 'package/example/.cache/typescript/tsconfig.tsbuildinfo')));
console.log('rsync kept source and excluded transient cache');
"
```

Working catalog:

- Stable source under `src` is copied.
- Generated cache under `**/.cache` is not copied.

Failing catalog:

- Before the fix,
  a changing `.cache/typescript/tsconfig.tsbuildinfo` was included and disappeared during the copy,
  producing code 24.

The container integration recheck on `src/set-replace.ts` is tracked in
 `doc/planning/toml-edit-mutation-campaign.md`.
A previous failure must not be reclassified as a killed or surviving mutant.

## Verified workarounds

- Exclude `**/.cache` in the shard rsync list and the image-context ignore file.
  This is the committed fix in
  `package/mutation-test/cli/src/container/worktree.ts` and
  `package/mutation-test/cli/runtime/containerignore`.
  Tradeoff: packages relying on a checked-in or prepopulated `.cache` artifact inside the mutation
  container must regenerate it in their build or test task.
- Run the failed package source file again in a fresh container to get an infrastructure-clean report.
  Tradeoff: a retry alone leaves the original cache race possible if the exclusion is removed.

## What does not work

Treating code 24 as a mutant verdict does not work:
 the failure happened while preparing `/work`,
 before a mutant was applied or a baseline measured.
The CLI correctly marked the run as an infrastructure failure instead of a test result.
Simply making `/src-ro` read-only does not freeze cache files being changed by other host processes.

## Upstream filing decision

`.out-of-scope/` has no rsync exemption.
Searches of open and closed issues and pull requests in `RsyncProject/rsync`
 for `vanished cache file rsync 24` found no matching report;
 a closed unrelated `configure.ac` PR appeared.
No upstream report is warranted:

- **Upstream fault:** No.
  Rsync correctly reports a file that vanished from its input.
  The mutation harness included a disposable cache it did not need.
- **Upstream fixability:** Rsync could change how it classifies a missing sender file,
  but that would not solve this harness's input-scope error.
- **Supported use case:** Yes.
  Rsync supports exclusion patterns and documented vanished-file diagnostics;
  the harness uses the exclusion boundary.
- **Contribution welcome:** The inspected source snapshot includes `README.md`
  but no CONTRIBUTING or issue/PR template was found.
  No contribution policy ban was found,
  and no upstream change is needed.
- **Likelihood of upstream fix:** There is no upstream defect to fix.
- **Minimal compatible prototype:** No upstream patch was made because the local exclusion solves the incident.
  The disposable rsync fixture and package tests verified the local correction.

## Upstream filing artifact

**Do not file as-is.**
The following draft records why the incident belongs in this repository:

~~~md
Title: rsync reports a vanished cache file in a mutation-test work tree
Labels: none
Description: rsync 3.5.0 reported code 24 when a TypeScript cache file
vanished from the sender mount. flist.c:1684-1691 records IOERR_VANISHED;
cleanup.c:208-217 converts it to RERR_VANISHED. This is expected behavior.
Reproduction: Copy a concurrently modified cache tree from a live repository.
Suggested fix: Exclude generated cache trees in the consuming mutation harness,
not in rsync.
~~~
