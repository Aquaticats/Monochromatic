# Verification: chokidar + atomic write migration

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/deps-cube` is now `package/dependency-catalog/cli-cube`.

## What this is

Step-by-step manual verification for the editord migration committed as
`f33633bc` (feat) and `493f1e81` (fix). The migration:

- Replaces `fs.writeFile` with an atomic temp + fsync + rename through
  `src/server/operations/write-file-atomic.ts` for every user-project
  mutation (`save`, `apply-workspace-edit`, `new-entry`).
- Swaps native `fs.watch` for chokidar 5 in `DirWatcher`
  (`src/server/operations/watch-filesystem.ts`).
- Adds the previously-missing `dirWatcher.suppressPath` calls for
  workspace-edit, new-entry, move-entry, delete-entry, copy-entry, so
  fs actions stop echoing back as `fileChanged` events.

Automated checks (lint:types, lint, runtime smoke of `writeFileAtomic`
and `DirWatcher` in isolation) all pass. What's left is the end-to-end
checks that need a real daemon and either a browser or a small script.

If you finish a check, mark it done in the **Status** line at the top
of that section. If you find a regression, write a short bug report at
the bottom of this file (template at the end).

## Prerequisites

Bring up the daemon if not already running:

```sh
mise run //packages/desktop-daemon/editord:dev
```

This starts `watch-restart` plus `bun src/server/index.ts`. Watch the
terminal: a successful start logs `editord listening on port 4400`. If
you see `EADDRINUSE`, an older instance is still bound; kill it with
`pkill -f 'bun src/server/index.ts'` and re-run.

The default port is `4400`. The auth token is in
`$TMPDIR/editord-4400.token` (typically `/tmp/editord-4400.token`).
Both are needed by the script in section D.

## A. Edit-and-save flow (DONE)

Status: confirmed by the user.

What this proved: `writeFileAtomic` works in the live save path, no
torn writes observed under interactive use.

## B. LSP rename across files (TODO)

Status: not yet verified.

What this proves: `apply-workspace-edit.ts` writes every affected
file atomically and the per-file `suppressPath` calls actually stop
the watcher from echoing the saves back to the client. Without
suppression, the rename refactor would reload the buffer mid-edit on
every other file, scrambling cursor positions.

### Steps

1. Open the PWA at `http://localhost:4400`. Pick a project that has at
   least one TypeScript symbol used in two or more files (any monorepo
   package works; e.g. `packages/dev-script/deps-cube/src/cache.ts`
   has `createCache` referenced from `index.ts` and others).
2. Open Chrome DevTools (F12). Go to the **Console** tab. Clear the
   log (`Ctrl+L`).
3. Position the cursor inside the symbol name.
4. Press **Shift+F6**. A small floating input appears, pre-filled with
   the current symbol name.
5. Type a new name and press **Enter**. The rename input closes; all
   occurrences of the symbol get replaced.

### What to check

- **On disk**: open one of the other affected files (not the one you
  were editing) directly in a different terminal with `cat` or your
  own editor. The symbol should be renamed in that file too.
- **DevTools Console**: filter for `fileChanged`. Expected: **zero**
  `fileChanged` events for the renamed files during the operation.
  The client maintains the buffers itself based on the `renameResult`
  message; a `fileChanged` event would mean the watcher echoed the
  save back, which is the bug this migration fixes.
- **PWA**: undo (`Ctrl+Z`) should work normally in the currently-open
  file. The non-open files are not in the buffer history.

### Restore

Press **Shift+F6** again with the new name selected, type the original
name, press **Enter**. Or use git: `git checkout -- <files>`.

## C. File-tree mutation echoes (TODO)

Status: not yet verified.

What this proves: `move-entry`, `delete-entry`, `copy-entry`,
`new-entry` all suppress watcher echoes for their target paths. Without
this, every right-click action would cause the tree to refresh
unnecessarily (and worse: an `unlink` echo for a delete could remove a
file that's still being displayed).

### Steps

For each of the four actions below, with DevTools open and the
console filter set to `fileChanged`:

1. **Create a file**: right-click an entry in the file tree sidebar,
   pick **New File**. Type a name and press Enter.
2. **Create a folder**: right-click, pick **New Folder**. Type a name
   and press Enter.
3. **Rename / move**: right-click an existing entry, pick **Rename**
   (this is the file-tree rename, **not** the LSP symbol rename from
   section B). Type a new name and press Enter.
4. **Delete**: right-click an entry, pick **Delete**. Confirm.
5. **Copy** (if available): right-click, pick **Copy**, then
   right-click the destination directory and pick **Paste**.

### What to check

Each action must produce **zero** `fileChanged` events in the DevTools
Console during and immediately after the action. The tree may refresh
visually; that's a different message (`listDir` response). Only
`fileChanged` is the regression marker.

To distinguish: in the Console filter, search exactly `"type":"fileChanged"`.

### Restore

Manually delete any test files you created. There's no built-in undo
for tree mutations.

## D. Torn-write race reproduction (TODO)

Status: not yet verified.

What this proves: under high-frequency saves through editord's
WebSocket endpoint, concurrent readers never observe an empty or
partially-written file. This is the same shape of test used in
`/TROUBLESHOOTING.claude-code-edit-non-atomic-fallback.md` to confirm
the Claude Code bug; for editord, the result should be 0 zero-size
reads and 0 zero-match reads.

### Setup

Make a test target file under `/tmp`:

```sh
printf 'line one\nSENTINEL\nline three\n' > /tmp/editord-tear-test.txt
chmod 600 /tmp/editord-tear-test.txt
```

The test path **must** be inside a directory that editord's
`assertWithinRoot` will accept. The daemon's root is whatever directory
was passed at startup (default: the directory you ran `mise run dev`
from, almost certainly `packages/desktop-daemon/editord/`). If `/tmp`
is outside that root, copy the test file into the editord directory
instead and adjust the paths in the scripts below.

### Writer script

Save as `/tmp/editord-tear-writer.mjs`:

```js
// Sends N save messages through editord's WebSocket endpoint, awaiting
// each `saved` ack before the next. Used to drive the torn-write race
// reproduction.
import { readFile, } from 'node:fs/promises';

const PORT = 4400;
const targetPath = process.argv[2];
const N = Number.parseInt(process.argv[3] ?? '5000', 10,);

const tokenPath = `${process.env['TMPDIR'] ?? '/tmp'}/editord-${PORT}.token`;
const token = (await readFile(tokenPath, 'utf8',)).trim();
const initialContent = await readFile(targetPath, 'utf8',);

const ws = new WebSocket(`ws://localhost:${PORT}/ws?token=${token}`,);
await new Promise(function waitOpen(resolve, reject,) {
  ws.addEventListener('open', resolve,);
  ws.addEventListener('error', reject,);
},);

const pending = new Map();
ws.addEventListener('message', function onMessage(event,) {
  const msg = JSON.parse(event.data,);
  if (msg.type === 'saved' && pending.has(msg.id,)) {
    pending.get(msg.id,)();
    pending.delete(msg.id,);
  }
},);

let done = 0;
for (let i = 0; i < N; i++) {
  const id = `save-${i}`;
  await new Promise(function awaitAck(resolve,) {
    pending.set(id, resolve,);
    ws.send(JSON.stringify({
      type: 'save',
      id,
      path: targetPath,
      content: initialContent,
    },),);
  },);
  done++;
}

console.log(`saves completed: ${done}`,);
ws.close();
```

### Reader script

Save as `/tmp/editord-tear-reader.mjs`. This is the same shape as
`reader.mjs` from `/TROUBLESHOOTING.claude-code-edit-non-atomic-fallback.md`:

```js
import {
  readFile,
  stat,
} from 'node:fs/promises';

const path = process.argv[2];
const N = Number.parseInt(process.argv[3] ?? '5000', 10,);
const expected =
  ((await readFile(path, 'utf8',)).match(/SENTINEL/g,) ?? []).length;

let zeroSize = 0;
let zeroMatch = 0;

for (let i = 0; i < N; i++) {
  try {
    const stats = await stat(path,);
    if (stats.size === 0) {
      zeroSize++;
      continue;
    }
    const matches = ((await readFile(path, 'utf8',))
      .match(/SENTINEL/g,) ?? [])
      .length;
    if (matches === 0)
      zeroMatch++;
  }
  catch {
    zeroSize++;
  }
}

console.log(JSON.stringify({
  N,
  expected,
  zeroSize,
  zeroMatch,
},),);
```

### Run

In one terminal:

```sh
bun /tmp/editord-tear-writer.mjs /tmp/editord-tear-test.txt 5000 &
bun /tmp/editord-tear-reader.mjs /tmp/editord-tear-test.txt 5000
wait
```

The `&` starts the writer in the background; the reader runs in the
foreground until done; `wait` keeps the shell alive for the writer to
finish. Each takes ~5 to 30 seconds depending on disk speed.

### Expected result

```json
{ "N": 5000, "expected": 1, "zeroSize": 0, "zeroMatch": 0 }
```

`zeroSize` and `zeroMatch` must both be **0**. Anything non-zero
indicates a torn-write window and is a regression; capture the exact
output, the daemon's stderr, and the editord version, and file as a
new TROUBLESHOOTING file in the repo root.

For comparison, the pre-migration `fs.writeFile` shape was measured at
~30% of reads observing a torn state (see the Claude Code
TROUBLESHOOTING doc, section "Observed results").

### Cleanup

```sh
rm -f /tmp/editord-tear-test.txt /tmp/editord-tear-writer.mjs /tmp/editord-tear-reader.mjs
```

## What's been verified automatically (no manual action needed)

These were run during implementation and are recorded here so you don't
re-run them:

- **`writeFileAtomic` direct calls**: writes correct content, doesn't
  leave orphan temps on success, preserves mode 0640 across writes,
  rejects symlinked targets with `ELOOP` without modifying either the
  symlink or the linked-to file.
- **`DirWatcher` orphan sweep**: matches only `.<basename>.editord.<hex>~`
  pattern, leaves other tilde-files alone, emits zero events for the
  swept temps.
- **`DirWatcher` suppression integration**: calling `suppressPath` and
  then `writeFileAtomic` on the same path produces 0 events; an
  unsuppressed `writeFileAtomic` produces exactly 1 `modified` event
  with the right shape.

If any of these fails on a new build, the bug is in the helper or the
watcher itself, not in the integration; the smoke commands are in the
git history of this branch (search for `wfa-smoke` and `wfa-supp-test`
in the session log).

## Regression bug template

If any of B/C/D fails, append a section here with this shape:

```markdown
## Regression: <one-line summary>

Date: <YYYY-MM-DD>
Section: <B | C | D>
Daemon version: <git rev-parse HEAD output>

### Symptom

<what you observed; copy DevTools console output or script output>

### Reproduction

<minimal steps; one or two paragraphs>

### Suspect

<best guess at the cause: which file, which commit, which event>
```
