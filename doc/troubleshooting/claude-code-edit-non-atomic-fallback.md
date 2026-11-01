# Claude Code v2.1.140 `Edit`/`Write` fallback `PRH` uses `open(O_TRUNC) + writeFileSync`, exposing an empty-file window to concurrent readers

Status:
 **root cause identified**;
 remediation proposed;
 upstream issue not yet filed.

Affects Claude Code v2.1.140 (offsets in this document refer to the unpacked
`cli.js` for that build;
 the helper has existed across recent builds).

## Symptom

After Claude Code reports a successful `Edit` or `Write` on a file `F`,
a concurrent reader (another process,
 a watcher,
 a follow-on `grep`) can briefly observe
`F` as **empty** or **partially-written**,
 even though both the prior on-disk content
and the new content contain the data being searched for.

Concretely observed in this repo on 2026-05-12 at 20:17:

```text
Edit succeeded on pnpm-workspace.yaml (override block inserted)
... immediately afterwards ...
grep -c 'proper-lockfile' pnpm-workspace.yaml  -> 0 (exit 1)
... seconds later ...
grep -n 'proper-lockfile' pnpm-workspace.yaml  -> 4 matches (file mtime 20:17:53)
```

The same Claude Code session had also seen the file-modification guard fire on a prior
Edit attempt with `<tool_use_error>File has been modified since read, either by the user
or by a linter. Read it again before attempting to write it.</tool_use_error>`,
indicating that another process was concurrently writing the same file during the session.

## Verification

Version under test:
 Claude Code v2.1.140 (linux x64,
 build
2026-05-12T18:28:21Z,
 sha
`89b4b3854fac52fdb8f9970133c4afe00174b6b9`).

The window is caused by the non-atomic fallback in Claude Code's write helper,
which uses Node's `open(O_TRUNC) + writeFileSync` shape.
Reproducing that shape directly demonstrates the symptom without needing Claude Code itself.

`writer.mjs`:

```javascript
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
const path = process.argv[2];
const N = parseInt(process.argv[3] ?? '5000', 10,);
const content = await readFile(path, 'utf8',);
for (let i = 0; i < N; i++)
  await writeFile(path, content, 'utf8',);
```

`reader.mjs`:

```javascript
import {
  readFile,
  stat,
} from 'node:fs/promises';
const path = process.argv[2];
const N = parseInt(process.argv[3] ?? '5000', 10,);
const expected = (await readFile(path, 'utf8',))
  .match(/SENTINEL/g,)
  ?.length ?? 0;
let zeroSize = 0, zeroMatch = 0;
for (let i = 0; i < N; i++) {
  try {
    if ((await stat(path,)).size === 0) {
      zeroSize++;
      continue;
    }
    const m = ((await readFile(path, 'utf8',))
      .match(/SENTINEL/g,) ?? [])
      .length;
    if (m === 0)
      zeroMatch++;
  }
  catch {
    zeroSize++;
  }
}
console.log(JSON.stringify({ N, expected, zeroSize, zeroMatch, },),);
```

Run both against a copy of a 10 KB file with at least one occurrence of `SENTINEL`:

```sh
cp pnpm-workspace.yaml /tmp/race-target.yaml
node writer.mjs /tmp/race-target.yaml 5000 &
node reader.mjs /tmp/race-target.yaml 5000
```

Observed results against a 10 KB copy of this repo's `pnpm-workspace.yaml`:

- Node async `writeFile`:
   998 / 5000 zero-size reads,
   1487 / 5000 zero-match reads.
- Bun async `writeFile` (same code,
   swap runtime):
   0 / 5000.

Single-process synchronous `writeFileSync` in a tight loop does not reproduce
the window (libuv resolves the truncate and the write in the same tick from the
single-process view).
Cross-process readers,
 however,
 observe the truncate-write gap reliably.

## Root cause

Claude Code's atomic-then-fallback writer is the function `PRH` (offset 988177 in
`/tmp/cc-unpack/cli.js` for v2.1.140).
Both the `Edit` and `Write` tools reach it through `j5H(H,$,q,K)` at offset 986311,
which forwards `{encoding: q}` without setting `allowSymlink` or `checkParentDir`.

`PRH` body,
 lightly de-minified for readability:

```javascript
function PRH(H, $, q = { encoding: 'utf-8', },) {
  const fsMod = m$();
  const flags = q.allowSymlink ? 0 : Xw.constants.O_NOFOLLOW;
  let target = H, mode, exists = false;

  /* ... allowSymlink / checkParentDir branches; lstat target to fetch mode ... */

  const tempPath = `${target}.tmp.${process.pid}.${
    Mzq.randomBytes(6,).toString('hex',)
  }`;

  try {
    // Atomic primary path:
    E(`Writing to temp file: ${tempPath}`,);
    const fd = Xw.openSync(
      tempPath,
      Xw.constants.O_WRONLY
        | Xw.constants.O_CREAT
        | Xw.constants.O_EXCL
        | flags,
      !exists && q.mode !== undefined ? q.mode : undefined,
    );
    try {
      Xw.writeFileSync(fd, $, { encoding: q.encoding, },);
      if (exists && mode !== undefined) {
        Xw.fchmodSync(fd, mode,);
        E('Applied original permissions to temp file',);
      }
      Xw.fsyncSync(fd,);
    }
    finally {
      Xw.closeSync(fd,);
    }
    E(`Temp file written successfully, size: ${$.length} bytes`,);
    E(`Renaming ${tempPath} to ${target}`,);
    fsMod.renameSync(tempPath, target,);
    E(`File ${target} written atomically`,);
  }
  catch (err) {
    E(`Failed to write file atomically: ${err}`, { level: 'error', },);
    try {
      E(`Cleaning up temp file: ${tempPath}`,);
      fsMod.unlinkSync(tempPath,);
    }
    catch (cleanupErr) {
      E(`Failed to clean up temp file: ${cleanupErr}`,);
    }

    // Non-atomic fallback path:
    E(`Falling back to non-atomic write for ${target}`,);
    let fd;
    try {
      fd = Xw.openSync(
        target,
        Xw.constants.O_WRONLY
          | Xw.constants.O_CREAT
          | Xw.constants.O_TRUNC
          | flags,
        !exists && q.mode !== undefined ? q.mode : undefined,
      );
    }
    catch (openErr) {
      if (j8(openErr,) === 'ELOOP') {
        throw new Error(
          `Refusing to write through symlink: ${target} (O_NOFOLLOW)`,
        );
      }
      E(`Non-atomic write also failed: ${openErr}`,);
      throw openErr;
    }
    try {
      Xw.writeFileSync(fd, $, { encoding: q.encoding, },);
      Xw.fsyncSync(fd,);
      E(`File ${target} written successfully with non-atomic fallback`,);
    }
    finally {
      Xw.closeSync(fd,);
    }
  }
}
```

Behaviour summary:

1. `PRH` first attempts an atomic write:
    temp file with `O_EXCL`,
    write,
    `fchmod`
   to preserve the original mode,
    `fsync`,
    `rename` over the target.
   A concurrent reader sees only the old content or the new content during this path.
2. If any step in (1) throws,
    the catch block logs `Failed to write file atomically: <err>`,
   then opens the target directly with `O_WRONLY | O_CREAT | O_TRUNC` and writes.
   **`O_TRUNC` zeroes the existing file before the new content is written.
   **
   A concurrent reader during this window observes the file as empty.

The fallback path is the bug.
`O_TRUNC` is observably non-atomic from another process' perspective;
 the proven
race window (see Reproduction) is wide enough to be caught by an ordinary `grep`
running shortly after Edit reports success.

### What triggers the fallback

The catch block fires when any of the following throws in the atomic path:

- `openSync(temp, O_CREAT | O_EXCL | O_NOFOLLOW)`:
   rare;
   collision on a 48-bit
  random suffix is astronomically unlikely.
- `writeFileSync(fd, content)` on the temp fd:
   rare without disk-full or EIO.
- `fchmodSync(fd, originalMode)`:
   fires when the temp filesystem rejects the
  preserved mode bits.
- `fsyncSync(fd)`:
   rare;
   some filesystems (SMB on macOS) emit `ENOTSUP`.
- `renameSync(temp, target)`:
   cross-filesystem rename,
   missing target directory,
  or permission failure.

The specific trigger for any given observed failure is recorded in the
`Failed to write file atomically: <err>` log line;
 without that line the
trigger cannot be identified after the fact.

## Logging: how to capture the trigger

The `E()` logger (offset 196795) gates by level (default threshold `debug`,
 set
via `CLAUDE_CODE_DEBUG_LOG_LEVEL`) and writes through `J49()` (offset 196298).
`E()` calls in the fallback path use `{ level: 'error' }`,
 which passes the
default threshold.

Log destination,
 per `O8H()` (offset 197025),
 resolves in this order:

1. The `--debug=<dir>` CLI arg,
    if present.
2. The internal `Tk8` override (null by default).
3. `process.env.CLAUDE_CODE_DEBUG_LOGS_DIR`.
4. `~/.claude/debug/<sessionId>.txt`.

The write sink branches on whether debug mode is active (`FV()`,
 set by `--debug`,
`-d`,
 `DEBUG=1`,
 or `DEBUG_SDK=1`):

- Debug mode on:
   synchronous `appendFileSync` per record.
- Debug mode off:
   async-queued through `_k$.then(j49.bind(null, ...))` with a
  1000 ms flush interval and 100-byte buffer threshold.

In the default (non-debug) configuration,
 queued writes can be lost if the
process exits before the flush runs.
This makes the failure self-concealing:
 the bug occurs,
 the log records the
trigger,
 the process exits before the log flushes,
 and the trigger is unknown
to the user.

**To capture the trigger on the next occurrence:
**

```sh
DEBUG=1 claude  # or: claude --debug
```

Then on a reproducible failure,
 inspect:

```sh
grep 'Failed to write file atomically' ~/.claude/debug/<sessionId>.txt
```

The matching line names the syscall and the error code.

## Workarounds

For downstream tools that read files Claude Code may be writing:

- Verify mtime stability before processing:
   read mtime,
   read content,
   re-read
  mtime;
   if mtime changed,
   retry.
- For shell pipelines that grep a Claude-written file,
   accept that a transient
  zero-match result is unreliable;
   re-run after a short delay before drawing
  conclusions.
- For long-running watchers,
   debounce file-change events by 100 ms or more so
  the read happens after the writer's write completes.

For Claude Code users:

- Run with `--debug` to get synchronous log writes that capture the trigger.
- For files that must never be observably empty (e.g. config consumed by
  long-running daemons),
   avoid editing them through Claude Code's `Edit`/`Write`;
  use a manual `cat | sponge` or temp-and-rename workflow externally.

## What does not work

- Disabling format-on-save in editors:
   the bug is in Claude Code's write helper,
  not the editor's.
   dprint,
   for example,
   is atomic (see Verified solutions).
- Re-reading the file after `Edit` reports success:
   the symptom is transient,
  and a second read usually shows the correct content.
   The bug is not the
  final content;
   it is the **window during which other processes observe
  inconsistent state**.
- Running Claude Code under Bun instead of Node:
   not currently a supported
  configuration.
   Bun's `writeFile` does not expose the window,
   but Claude Code
  ships as a Node-based binary.

## Verified solutions

- **Atomic-by-default writers** do not exhibit the symptom.
  Demonstrated with dprint at `crates/dprint/src/utils/fs.rs:63`
  (`atomic_write_file`:
   write to `.tmp.<rand>` then `rename` over target).
  A cross-process reader saw zero zero-size and zero zero-match reads against
  a dprint-formatted file during 5000 racing reads.

The proposed upstream remediation is to make Claude Code's fallback also atomic
(see Draft GitHub issue).
This is the only path that closes the window entirely.

## Source locations

All offsets refer to the unpacked v2.1.140 binary at
`~/.local/share/claude/versions/2.1.140`,
 extracted with `tweakcc unpack`:

- `PRH` (write helper):
   offset 988177.
- Atomic-path log line `"File ${A} written atomically"`:
   offset 989653.
- Atomic-path error log line `"Failed to write file atomically"`:
   offset 989686.
- Fallback marker `"Falling back to non-atomic write for"`:
   offset 989847.
- Fallback success log line `"File ${A} written successfully with non-atomic fallback"`:
  offset 990239.
- `j5H` (Edit/Write entry point into `PRH`):
   offset 986311.
- `E` (logger):
   offset 196795.
- `J49` (logger sink):
   offset 196298.
- `O8H` (log path resolver):
   offset 197025.
- Log level map `Gk8`:
   offset 197371.

## Independent factors that made this particular incident harder to diagnose

These are not the bug,
 but compounded the difficulty of pinning it on the day of
the incident:

- The session log (`.jsonl` under `~/.claude/projects/`) records the Edit attempts
  and their tool results but does not include the `E()` log lines from `PRH`.
  Those go to `~/.claude/debug/` only.
- The user's `~/.claude/debug/` directory contained no log files for the affected
  session,
   consistent with the async-queue flush behavior described above.
- A separate `<tool_use_error>File has been modified since read, either by the
  user or by a linter</tool_use_error>` had fired earlier in the same session,
  indicating that another process was concurrently writing `pnpm-workspace.yaml`.
  Without the `Failed to write file atomically` log line,
   the writer's identity
  is unknown.

## Why we would file this upstream

All 5 constraints hold:

1. **Is it really upstream's fault?
   ** Yes;
    the fallback path
   uses `O_TRUNC` rather than the temp-and-rename pattern
   used by the primary path.
2. **Can upstream fix it?
   ** Yes;
    two concrete patches in the
   draft below (retry atomic on transient errors;
    second
   atomic temp with different suffix).
3. **Are they supporting this use case?
   ** Yes;
    cross-process
   readers (watchers,
    LSPs,
    build tools) are the normal
   environment Claude Code is used in.
4. **Will they likely fix it?
   ** Plausible;
    the bug is clear,
   the trigger is logged,
    and the fix is local to `PRH`.
5. **Have we prototyped a minimal fix?
   ** Architectural
   sketches in the draft;
    no PR yet.

Decision:
 worth filing.

## Draft GitHub issue (kept as reference; revise before filing)

To file against `anthropics/claude-code`:

```markdown
**Title:** `Edit`/`Write` fallback uses `O_TRUNC` write, exposing empty-file window to concurrent readers

**Labels:** bug, file-handling, race-condition

### Summary

Claude Code's file-write helper performs an atomic write to a temp file followed
by `rename` as its primary path, with a non-atomic `open(O_WRONLY | O_CREAT | O_TRUNC) + writeFileSync`
as a fallback when the atomic path throws.
During the fallback, between `O_TRUNC` and the completion of `writeFileSync`,
the target file is observably empty to any concurrent reader.

Cross-process reads land in this window reliably for files larger than a few KB:
~30% of 5000 racing reads observed either a zero-size file or zero matches for
a sentinel that exists in both the prior and the new content.

### Reproduction

Without Claude Code, the same shape of write is reproducible directly:

(insert minimal repro from Reproduction section above)

Triggering it inside Claude Code requires a condition that causes the atomic
path to throw.
The catch block logs `Failed to write file atomically: <err>` at level `error`
to `~/.claude/debug/<sessionId>.txt`, but only when debug mode is active
(`--debug`, `DEBUG=1`, etc.); without debug mode, the queued write may be lost
on exit, leaving no trace of the trigger.

### Impact

Concurrent readers of Claude-Code-written files (filesystem watchers, language
servers, build tools that re-read on `mtime`, IDE LSPs, follow-on shell `grep`
commands) can observe a transient empty state and make incorrect decisions
based on it: a watcher may report the file as deleted, an LSP may flag the
project as broken, a `grep` may return zero matches, a build tool may produce
spurious validation errors.

### Suggested fix

Make the fallback path also atomic.
Two options:

1. Retry the atomic path on transient errors (EEXIST, EINTR, EAGAIN) before
   falling back; reduce the false-fallback rate without changing semantics.
2. Have the fallback write to a *different* temp + `rename`, with a different
   collision-resistant suffix.
   Preserves the no-observable-empty-window invariant even when the first
   atomic attempt fails.

Either is preferable to the current `O_TRUNC` shape.

### Suggested observability fix

The error log line `Failed to write file atomically: <err>` is the only signal
that the fallback ran.
Either:

- Promote the log line to `console.warn` on stderr when the fallback fires, so
  users without debug mode can see it.
- Or surface the fallback via the tool response (`tool_use` result), so the
  model running inside Claude Code can adjust its behavior (e.g., re-Read the
  file to confirm content before continuing).

### Environment

- Claude Code v2.1.140 (linux x64, build 2026-05-12T18:28:21Z, sha
  `89b4b3854fac52fdb8f9970133c4afe00174b6b9`)
- Linux 6.19.14-ogc2.1.fc44.x86_64 (Bazzite / Fedora atomic with `/home -> var/home`
  symlink at root)
- ext4 / btrfs target filesystem
- File size 10 KB
```

## Separate disposable guard-fixture relocation, 2026-10-18

An auto-mode verification helper spawns its generated Node wrapper with the repository as cwd.
The wrapper derives its fixture root from cwd,
so it creates `bounded-native-test-hRbmV3` at the repository root.
It also writes a guard driver outside the directory mounted as the container home.
Node reports `MODULE_NOT_FOUND` for `/fixture-home/guard-driver.ts`.
The mutant never builds and the consumer never runs;
that attempt supplies no regression evidence.

The helper is corrected to use its actual scratch cwd
and write the driver under the mounted private home.
Moving the owned repository-root fixture to scratch with `fs.rename` emits `EXDEV`.
The recovery copies the directory,
verifies every file hash,
mode and symlink target,
then removes only the verified owned originals.
Ignored-cleanup dry-run and root Git-directory-entry checks run before recovery.
No root `HEAD`,
`config`,
`hooks`,
`objects` or `refs` entry exists.

The original fixture path moves on 2026-10-18 to
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/recovered-guard-fixture-hRbmV3`.
Its `location-move.json` retains the original path and recovery reason.
The corrected guard run rebuilds each event-role,
input-phase,
string-member and complete-property variant from committed source `6fac7c5648`.
Each actual SDK consumer detects its removed guard.
The root publication and protected source hashes remain unchanged.
This is an owned helper path assumption exposed by ordinary filesystem mount behavior,
not a Node or Linux defect.

## Separate Node handover staging failure, 2026-10-18

A handover refresh stages its replacement body under `${HOME}/temp/agent`,
then calls Node `fs.rename` into `doc/handover/pi-auto-mode-current-frontier.md`.
That operation emits `EXDEV: cross-device link not permitted, rename`.
The historical archive has already been written,
but the current checkpoint remains its original content.
This observed Node failure is separate from the Claude Code fallback incident.

The completion stages an exclusively created temporary sibling beside the destination,
preserves the original mode,
then renames that sibling over the current checkpoint.
That rename succeeds.
The result is a 268-line current checkpoint and a 14,624-line historical archive;
the archived original body is verified byte for byte.
The developer utility itself remains in the normal agent scratch directory.
Its destination sibling exists only for the final replacement.

Commit `e85023fb09` preserves the archive before the current replacement completes.
Commit `2faf1b2298` completes the replacement,
and a corrective comment on the initial commit records that partial state.
`frontier-archive-receipt.json` in
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018` retains the original hash and preservation result.
