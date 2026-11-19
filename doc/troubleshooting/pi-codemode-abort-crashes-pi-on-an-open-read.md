# Pi 1.5.2: codemode ending mid-`read` crashes pi with an uncaught `ReadStream` `AbortError`

Status:
 unapproved.
 This records work done by an agent on 2026-11-19 against the incident log
`crash-stderr-of-pi-1.5.2-at-2026-11-19T03-09Z.log`
in
`${HOME}/temp/agent/pi-codemode-abort-crash-troubleshooting-2026-11-18/`,
and the open-source tree at `earendil-works/pi`.
 No Pi session file, Claude session file, or feed was read;
 the stderr log above is the whole incident evidence.
 Upstream already carries the report and the fix (see "Upstream filing artifact"),
 so nothing is filed.
 This repository's Markdown linter did not run over this file:
 `mise run lint:monochromatic-lint` stops with `monochromatic-lint: command not found`,
 because that toolchain binary is not installed here.
 The checks that did run are named in "Verification".

## Symptom

A `pi` run (`@earendil-works/pi-coding-agent` 1.5.2,
 Node v26.13.1,
 `NODE_OPTIONS=--permission-audit`)
 exited with status 1 on its ninth turn.
 Its `codemode` script read several files;
 one did not exist
 (`ENOENT: no such file or directory, access '.../package/wg/cli-quicker/src/command.ts'`),
 the script failed,
 a sibling read reported `Operation aborted`,
 and the whole process died:

```text
node:events:505
    throw er; // Unhandled 'error' event
    ^

AbortError: The operation was aborted
    at stream.<computed>.AbortError.cause (node:internal/streams/add-abort-signal:47:22)
    at module.exports.addAbortSignalNoValidate (node:internal/streams/add-abort-signal:53:5)
    at addAbortSignal (node:internal/streams/add-abort-signal:38:25)
    at ReadStream.Readable (node:internal/streams/readable:376:7)
    at new ReadStream (node:internal/fs/streams:257:3)
    at FileHandle.createReadStream (node:internal/fs/promises:414:12)
    at Object.readLocalFileChunks [as createReadStream] (.../dist/bundle/chunks/chunk-VEJDBY3O.js:789:1792)
    at async selectLinesFromStream (.../dist/bundle/chunks/chunk-VEJDBY3O.js:792:401)
    at async .../dist/bundle/chunks/chunk-VEJDBY3O.js:798:239
Emitted 'error' event on ReadStream instance at:
    at emitErrorNT (node:internal/streams/destroy:170:8)
    at emitErrorCloseNT (node:internal/streams/destroy:129:3)
    at process.processTicksAndRejections (node:internal/process/task_queues:90:21) {
  code: 'ABORT_ERR',
  [cause]: DOMException [AbortError]: This operation was aborted
      ...
      at Execution.finish (.../dist/bundle/chunks/execute-L2EEDV3L.js:2:8423)
      at Execution.handleDone (.../dist/bundle/chunks/execute-L2EEDV3L.js:2:6630)
      at Execution.handleMessage (.../dist/bundle/chunks/execute-L2EEDV3L.js:2:6489)
      at Execution.receive (.../dist/bundle/chunks/execute-L2EEDV3L.js:2:5688)
      at Object.message (.../dist/bundle/chunks/execute-L2EEDV3L.js:2:3941)
      at Worker.<anonymous> (.../dist/bundle/chunks/execute-L2EEDV3L.js:2:12891)
      at Worker.emit (node:events:514:20)
      at MessagePort.<anonymous> (node:internal/worker:354:14)
}
```

Nothing is caught:
 the failure is a process-level `uncaughtException` path in Node's `EventEmitter`,
 so Pi cannot report a tool error and the turn's work is gone.

The same crash needs only one shape of use:
 a `read` tool call that is still inside `fsOpen()` or `handle.stat()`
 when its `AbortSignal` fires.
 Inside `codemode` that happens whenever a script ends while a read is pending,
 whether the script threw, rejected on a missing sibling file,
 or simply finished:

```js
// The script that produced the incident, reduced to its shape.
await Promise.all([
  tools.read({ path: "some/large/existing/file.md" }),
  tools.read({ path: "package/wg/cli-quicker/src/command.ts" }), // ENOENT
]);
```

```js
// Also fatal: the script ends (or calls exit()) with a read still opening.
const pending = tools.read({ path: "some/large/existing/file.md" });
throw new Error("script failed");
```

## Root cause

### Step 1: a codemode script's end aborts every pending nested call

`Execution.finish()` runs on every way a script ends:
 success, script error, timeout, and outside abort.
 It aborts the controller of each nested tool call that has not answered yet.

```ts
// packages/codemode/src/runtime/execution.ts:410-427 (v1.5.2)
private finish(
	error: CodemodeError | undefined,
	value?: unknown,
	writes?: CodemodeStoreWrites,
	during?: CodemodeDuring,
): void {
	if (this.finished) return;
	this.finished = true;
	clearTimeout(this.timer);
	this.signal?.removeEventListener("abort", this.onAbort);

	const now = performance.now();
	for (const pending of this.pending.values()) {
		if (pending.record) pending.record.durationMs = now - pending.startedAt;
		pending.controller.abort();
	}
	this.pending.clear();
	this.unsent = [];
```

Source:
 <https://github.com/earendil-works/pi/blob/b4afdddf44e059c67ccf5097e5317d913521b470/packages/codemode/src/runtime/execution.ts#L421-L425>.

The incident's stack matches this order:
 `Worker` message -> `Execution.receive` -> `handleMessage` -> `handleDone` -> `finish`,
 and the `DOMException` is constructed inside `AbortController.abort()` there.

### Step 2: the read tool gives that signal to an fs `ReadStream`

The read tool's default operations stream a local file,
 and the generator passes the tool call's signal straight into the stream:

```ts
// packages/coding-agent/src/core/tools/read.ts:75-84 (v1.5.2)
async function* readLocalFileChunks(path: string, { signal }: { signal?: AbortSignal }): AsyncIterable<Uint8Array> {
	const handle = await fsOpen(path, "r");
	try {
		const stats = await handle.stat();
		const end = stats.isFile() && stats.size > 0 ? stats.size - 1 : undefined;
		yield* handle.createReadStream({ end, signal, autoClose: false });
	} finally {
		await handle.close();
	}
}
```

Source:
 <https://github.com/earendil-works/pi/blob/b4afdddf44e059c67ccf5097e5317d913521b470/packages/coding-agent/src/core/tools/read.ts#L75-L81>.

The tool does not wait for the stream, though.
 Its own promise settles on the same signal, independently:

```ts
// packages/coding-agent/src/core/tools/read.ts:244-262 (v1.5.2)
		async execute(
			_toolCallId,
			{ path, offset, limit }: { path: string; offset?: number; limit?: number },
			signal?: AbortSignal,
			_onUpdate?,
			ctx?: ExtensionContext,
		) {
			return new Promise<{ content: (TextContent | ImageContent)[]; details: ReadToolDetails | undefined }>(
				(resolve, reject) => {
					if (signal?.aborted) {
						reject(new Error("Operation aborted"));
						return;
					}
					let aborted = false;
					const onAbort = () => {
						aborted = true;
						reject(new Error("Operation aborted"));
					};
					signal?.addEventListener("abort", onAbort, { once: true });
```

Source:
 <https://github.com/earendil-works/pi/blob/b4afdddf44e059c67ccf5097e5317d913521b470/packages/coding-agent/src/core/tools/read.ts#L244-L262>.

That rejection is what the model saw ("Operation aborted").
 The stream the signal reached is a second, unowned object.

### Step 3: an abort before the iterator attaches destroys the stream unwatched

`FileHandle.createReadStream` forwards `signal` to the `ReadStream`, whose constructor calls
 `addAbortSignal`;
 aborting then calls `stream.destroy(new AbortError(...))`.
 With `autoClose: false` (`autoDestroy: false`) the stream is not owned by its async iterator:
 the `yield*` iteration in `readLocalFileChunks` is abandoned as soon as the consumer stops,
 so the iterator detaches its listeners and the stream finishes destroying on a later tick.
 Its `'error'` event then has zero listeners, and `EventEmitter` throws it:

```text
at emitErrorNT (node:internal/streams/destroy:170:8)
at emitErrorCloseNT (node:internal/streams/destroy:129:3)
at process.processTicksAndRejections (node:internal/process/task_queues:90:21)
```

Inside `codemode` the consumer's own exit is the loop check that throws first,
 which is why the crash needs the stream to be destroyed **before** the iterator starts watching it:

```ts
// packages/coding-agent/src/core/tools/read.ts:212-215 (v1.5.2)
	for await (const chunk of chunks) {
		// Leaving the loop closes the stream.
		if (signal?.aborted) throw new Error("Operation aborted");
```

Source:
 <https://github.com/earendil-works/pi/blob/b4afdddf44e059c67ccf5097e5317d913521b470/packages/coding-agent/src/core/tools/read.ts#L212-L215>.

Upstream's own post-mortem states the same five steps
 (constructor-time destroy, iterator detach, later `'error'`, zero listeners)
 in the pull request that fixed it,
 and an independent reproducer in the canonical issue traces it identically.
 Both are quoted under "Duplicate search and work in progress".

### Step 4: why the shipped test did not catch it

The streaming test that covers abort injects a hand-written generator,
 so no `fs` `ReadStream` exists and nothing is destroyed with an error:

```ts
// packages/coding-agent/test/read-streaming.test.ts:264-283 (v1.5.2)
	it("stops consuming the stream when aborted", async () => {
		const controller = new AbortController();
		let chunksRead = 0;
		let closed = false;
		const ops: ReadOperations = {
			readFile: async () => Buffer.alloc(0),
			access: async () => {},
			async *createReadStream(_path, { signal }) {
				expect(signal).toBe(controller.signal);
				try {
					while (true) {
						chunksRead++;
						if (chunksRead === 3) controller.abort();
						await new Promise((resolve) => setTimeout(resolve, 1));
						yield Buffer.from("line\n");
					}
				} finally {
					closed = true;
				}
			},
		};
```

Source:
 <https://github.com/earendil-works/pi/blob/b4afdddf44e059c67ccf5097e5317d913521b470/packages/coding-agent/test/read-streaming.test.ts#L264-L271>.

An earlier reading of mine blamed the consumer's throw for detaching the listeners on its own;
 the run in "Harness 2" disproves it,
 because the abort there happens inside `handle.stat()` and the consumer never runs a loop iteration.

### Fixed upstream, not in any release

Commit `ef9361ab70206e521a46fa7b3d1068442eec4908`
 (2026-11-11, pull request #12025, merged after v1.5.2 was published on 2026-11-10)
 stops giving the stream the signal, and leaves abort to the reader that already checks it:

```diff
-async function* readLocalFileChunks(path: string, { signal }: { signal?: AbortSignal }): AsyncIterable<Uint8Array> {
+async function* readLocalFileChunks(path: string): AsyncIterable<Uint8Array> {
 	const handle = await fsOpen(path, "r");
 	try {
 		const stats = await handle.stat();
 		const end = stats.isFile() && stats.size > 0 ? stats.size - 1 : undefined;
-		yield* handle.createReadStream({ end, signal, autoClose: false });
+		// No signal for the stream: one that aborts before the stream's iterator attaches makes the stream
+		// emit an 'error' no listener handles, which crashes the process (#11956).
+		yield* handle.createReadStream({ end, autoClose: false });
```

Source:
 <https://github.com/earendil-works/pi/blob/ef9361ab70206e521a46fa7b3d1068442eec4908/packages/coding-agent/src/core/tools/read.ts#L76-L85>.

No published version contains it:
 `npm view @earendil-works/pi-coding-agent versions` ends at `1.5.2`
 (checked 2026-11-19),
 and `v1.5.2` is the newest GitHub release.
 The same commit also fixes two neighbours of the same class:
 the session picker passing its signal to a `createReadStream`
 (`packages/coding-agent/src/core/session-manager.ts`),
 and a `gh` spawn without an `'error'` listener (`session-share.ts`).

## Verification

### Version under test

- `@earendil-works/pi-coding-agent` 1.5.2, installed under the pnpm store at
  `<store>/links/@earendil-works/pi-coding-agent/1.5.2/<content-hash>/node_modules/@earendil-works/pi-coding-agent`,
  with `@earendil-works/pi-codemode` 1.5.2 beside it.
  The store root here is `~/.local/share/pnpm/store/v11`, expanded to `${HOME}` in the commands below.
- Release tag `v1.5.2`, published 2026-11-10T12:25:43Z, is commit
  `b4afdddf44e059c67ccf5097e5317d913521b470`;
  the source clone sat at that commit, shallow.
- Bundled files under test:
  `dist/bundle/chunks/chunk-VEJDBY3O.js`
  sha256 `effbbd5d6247983dc11c7ae5796eb24bda88f9ebf78c707d3f92ca73cbb8b51e`,
  `dist/bundle/chunks/execute-L2EEDV3L.js`
  sha256 `21decb679406682a3d810502dd154f77a0d8bcb3f6757aa603af87366396d8a5`.
- Node v26.13.1, `NODE_OPTIONS=--permission-audit` inherited by every run below,
  as in the incident.
- Harness scratch:
  `${HOME}/temp/agent/pi-read-crash-verification-2026-11-19.lpbjViuM`.
  No upstream build or test script was run;
  each harness is a small program that imports the installed distribution.

### Harness 1: the Node pattern alone

A `FileHandle.createReadStream` given a signal that aborts while the stream is open,
 with and without an `'error'` listener:

```ts
// ${HOME}/temp/agent/pi-read-crash-verification-2026-11-19.lpbjViuM/stream-abort.ts
import { open } from 'node:fs/promises';

const controller = new AbortController();
const handle = await open(new URL(import.meta.url), 'r');
const stream = handle.createReadStream({ signal: controller.signal, autoClose: false });
await new Promise<void>((resolve) => {
  if (process.argv[2] === 'handled') {
    stream.once('error', (error: NodeJS.ErrnoException) => {
      if (error.code !== 'ABORT_ERR') throw error;
      console.log(error.code);
    });
  }
  stream.once('close', resolve);
  controller.abort();
});
console.log('closed');
```

```console
$ node stream-abort.ts
node:events:505
    throw er; // Unhandled 'error' event
AbortError: The operation was aborted
  code: 'ABORT_ERR',
Emitted 'error' event on ReadStream instance at:
$ echo $?
1

$ node stream-abort.ts handled
ABORT_ERR
closed
$ echo $?
0
```

The crash and the clean variant differ only in the listener.

### Harness 2: pi's own read tool, aborted during `handle.stat()`

The factory is imported from the installed distribution,
 and the abort lands deterministically inside the open call:

```ts
// ${HOME}/temp/agent/pi-read-crash-verification-2026-11-19.lpbjViuM/pi-read.ts
import { open } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const { createReadToolDefinition } = await import(process.argv[2]);
const mode = process.argv[3];
const fixture = fileURLToPath(import.meta.url);
const handle = await open(fixture, 'r');
const prototype = Object.getPrototypeOf(handle) as typeof handle;
await handle.close();
const controller = new AbortController();
const stat = prototype.stat;
prototype.stat = async function (...args: Parameters<typeof stat>) {
  const stats = await stat.apply(this, args);
  if (mode === 'abort') controller.abort();
  return stats;
};
try {
  const tool = createReadToolDefinition(process.cwd());
  const result = await tool.execute('read', { path: mode === 'missing' ? fixture + '.absent' : fixture }, controller.signal);
  console.log(result.content[0].text.startsWith('import { open }') ? 'read succeeded' : result.content[0].text);
} catch (error) {
  console.log('read rejected:', error instanceof Error ? error.message : error);
}
```

```console
$ node pi-read.ts <installed>/dist/index.js normal
read succeeded
$ echo $?
0

$ node pi-read.ts <installed>/dist/index.js missing
read rejected: ENOENT: no such file or directory, access '.../pi-read.ts.absent'
$ echo $?
0

$ node pi-read.ts <installed>/dist/index.js abort
read rejected: Operation aborted
node:events:505
    throw er; // Unhandled 'error' event
AbortError: The operation was aborted
    at Object.readLocalFileChunks [as createReadStream] (.../dist/core/tools/read.js:46:23)
    at async selectLinesFromStream (.../dist/core/tools/read.js:156:22)
Emitted 'error' event on ReadStream instance at:
  code: 'ABORT_ERR',
$ echo $?
1
```

A missing file alone is harmless, because `access()` rejects before any stream exists.
 The abort during the open is what kills the process.

### Harness 3: pi's codemode runtime, the incident's shape

Pi's own sandbox runs the script, with Pi's own read tool registered as `tools.read`:

```ts
// ${HOME}/temp/agent/pi-read-crash-verification-2026-11-19.lpbjViuM/codemode-repro.ts
import { writeFileSync } from 'node:fs';
import { join } from 'node:path';

const installed = '<installed node_modules>';
const mode = process.argv[2] ?? 'sibling';
const readPackage = process.argv[3] ?? `${installed}/@earendil-works/pi-coding-agent`;
const nodeModules = process.argv[4] ?? readPackage.slice(0, readPackage.length - '/@earendil-works/pi-coding-agent'.length);
const { InlineCodemodeSandbox, loadQuickJSWasm } = await import(`${nodeModules}/@earendil-works/pi-codemode/dist/index.js`);
const { createReadToolDefinition } = await import(`${readPackage}/dist/index.js`);

const dir = process.cwd();
const big = join(dir, 'big.txt');
const small = join(dir, 'small.txt');
writeFileSync(big, 'x\n'.repeat(2 * 1024 * 1024));
writeFileSync(small, 'one line\n');
const absent = join(dir, 'command.ts');
const second = mode === 'control' ? small : absent;

const read = createReadToolDefinition(dir);
const sandbox = new InlineCodemodeSandbox({
  wasm: loadQuickJSWasm(),
  timeoutMs: 30_000,
  tools: [
    {
      name: 'read',
      execute: async (args, { signal }: { signal?: AbortSignal }) =>
        (await read.execute('call', args as { path: string }, signal)).structuredContent,
    },
  ],
});
```

The script bodies and the run,
 below that in the same file:

```ts
const bigArg = JSON.stringify(big);
const secondArg = JSON.stringify(second);
const scripts: Record<string, string> = {
  control: `await Promise.all([tools.read({ path: ${bigArg} }), tools.read({ path: ${secondArg} })]);\ntext('both read');`,
  sibling: `await Promise.all([tools.read({ path: ${bigArg} }), tools.read({ path: ${secondArg} })]);\ntext('both read');`,
  allsettled: `const results = await Promise.allSettled([tools.read({ path: ${bigArg} }), tools.read({ path: ${secondArg} })]);\ntext(results.map((r) => r.status).join(','));`,
  sequential: `for (const path of [${bigArg}, ${secondArg}]) {\n  try { await tools.read({ path }); } catch {}\n}\ntext('sequential done');`,
};
const result = await sandbox.execute(scripts[mode] ?? scripts.sibling);
console.log(mode, readPackage, 'exit path reached');
console.log('result', JSON.stringify({ ok: result.ok, value: result.value, error: result.error?.message, calls: result.calls }).slice(0, 300));
```

```console
$ node codemode-repro.ts control
control .../pi-coding-agent exit path reached
result {"ok":true,"calls":[{"name":"read","status":"ok",...},{"name":"read","status":"ok",...}]}
$ echo $?
0

$ node codemode-repro.ts sibling
result {"ok":false,"error":"ENOENT: no such file or directory, access '.../command.ts'",
        "calls":[{"name":"read","status":"cancelled",...}
node:events:505
    throw er; // Unhandled 'error' event
AbortError: The operation was aborted
    at Object.readLocalFileChunks [as createReadStream] (.../dist/core/tools/read.js:46:23)
    at async selectLinesFromStream (.../dist/core/tools/read.js:156:22)
Emitted 'error' event on ReadStream instance at:
  code: 'ABORT_ERR',
      at Execution.finish (.../pi-codemode/dist/runtime/execution.js:347:32)
      at Execution.handleDone (.../pi-codemode/dist/runtime/execution.js:288:18)
$ echo $?
1
```

That is the incident frame for frame,
 including `Execution.finish` reached through `handleDone` and a `read` recorded as `cancelled`.

### Catalogs

Clean, all measured above:

- stream aborted with an `'error'` listener: `ABORT_ERR`, exit 0.
- `read` of an existing file: text, exit 0.
- `read` of a missing file: `ENOENT` rejection, exit 0.
- codemode script where both reads succeed: `ok: true`, exit 0.
- codemode script with a missing sibling using `Promise.allSettled`: `ok: true`,
  reads `ok` and `error`, exit 0.
- codemode script with a missing sibling using sequential `try`/`catch`: `ok: true`, exit 0.
- codemode script whose reads both exist under 1.5.0: `ok: true`, exit 0.
- codemode script with a missing sibling under 1.5.0: `ok: false` and the ENOENT error, exit 0.
- codemode script with a missing sibling against a build carrying `ef9361ab`: `ok: false` and the ENOENT error, exit 0.

Failing, all measured above:

- stream aborted with no `'error'` listener: uncaught `ABORT_ERR`, exit 1.
- `read` aborted inside `handle.stat()` on 1.5.2: `Operation aborted` plus the crash, exit 1.
- codemode script with a missing sibling under `Promise.all` on 1.5.2: ENOENT, `cancelled` sibling, crash, exit 1.

### Pre/post against the upstream fix

To see the fix itself rather than its description,
 a private copy of the installed package was made in scratch
 (the installed package was not touched),
 and only `dist/core/tools/read.js` was changed:

```diff
-async function* readLocalFileChunks(path, { signal }) {
+async function* readLocalFileChunks(path) {
     const handle = await fsOpen(path, "r");
     try {
         const stats = await handle.stat();
         const end = stats.isFile() && stats.size > 0 ? stats.size - 1 : undefined;
-        yield* handle.createReadStream({ end, signal, autoClose: false });
+        yield* handle.createReadStream({ end, autoClose: false });
```

```console
$ node prototype-check.ts
{"arm":"installed-1.5.2","exitStatus":1,"uncaughtAbortError":true,...}
{"arm":"prototype-with-ef9361ab","exitStatus":0,"uncaughtAbortError":false,
 "stdout":"... result {\"ok\":false,\"error\":\"ENOENT: no such file or directory, access '.../command.ts'\"..."}
```

The script still reports the missing file, and the process survives.
 The copy lives at `${HOME}/temp/agent/pi-fix-prototype-2026-11-19.7JHaZc5o/`,
 disposable and never used as a workaround.

### The 1.5.0 arm

`1.5.0` (2026-11-02) predates the streaming read that pull request #11750 added,
 and its `dist/core/tools/read.js` has no `createReadStream` at all
 (`readFile: (path) => fsReadFile(path)`).
 The incident script was run against it with the same codemode runtime:

```console
$ npm install --prefix .../pi150 --ignore-scripts --no-audit --no-fund @earendil-works/pi-coding-agent@1.5.0
$ node codemode-repro.ts sibling .../pi150/node_modules/@earendil-works/pi-coding-agent <1.5.2 node_modules>
result {"ok":false,"error":"ENOENT: no such file or directory, access '.../command.ts'",
        "calls":[{"name":"read","status":"cancelled",...}]}
$ echo $?
0
```

## Verified workarounds

### Pin `1.5.0` until a release carries `ef9361ab`

```console
$ npm install @earendil-works/pi-coding-agent@1.5.0
```

Measured above:
 the incident's script ends with its ENOENT and exit 0.
 Tradeoffs:
 `1.5.0` has no streaming read, so a large file is loaded whole into memory before truncation,
 and everything `1.5.1` and `1.5.2` changed is given up
 (`1.5.2` is the session-fork-on-missing-cwd release).
 It is a pin, so it must be lifted once a release contains the fix.

### Keep no `read` in flight when the script can end

Let every read settle before the script can end,
 either all at once or one at a time:

```js
// Both measured to exit 0 on 1.5.2, with the missing path reported as a normal read error.
const results = await Promise.allSettled([
  tools.read({ path: big }),
  tools.read({ path: missing }),
]);
text(results.map((result) => result.status).join(",")); // "fulfilled,rejected"

for (const path of [big, missing]) {
  try {
    await tools.read({ path });
  } catch {
    // expected for a path that does not exist
  }
}
```

```js
// Fatal on 1.5.2: the missing sibling ends the script while the other read is still opening.
await Promise.all([tools.read({ path: big }), tools.read({ path: missing })]);
```

```console
$ node codemode-repro.ts sibling
result {"ok":false,"error":"ENOENT: ...","calls":[{"name":"read","status":"cancelled",...}
... Unhandled 'error' event, exit 1
$ node codemode-repro.ts allsettled
result {"ok":true,"calls":[{"name":"read","status":"ok",...},{"name":"read","status":"error",...}]}
$ echo $?
0
$ node codemode-repro.ts sequential
result {"ok":true,"calls":[{"name":"read","status":"ok",...},{"name":"read","status":"error",...}]}
$ echo $?
0
```

Tradeoffs:
 `Promise.allSettled` gives up fail-fast, so the script keeps reading after a path turns out to be missing
 and must decide what to do with each status;
 the sequential form also gives up parallel reads, so batched scripts get slower;
 and the window is reduced, not closed,
 because a user interrupt or the script timeout ends the run the same way while a read is pending.

### Use the `bash` tool for files that may be missing

```js
const found = await tools.bash({ command: "cat -- package/wg/cli-quicker/src/command.ts" });
```

The bash tool runs a child process instead of streaming the file through an `fs` `ReadStream`,
 so a script that never calls `read` has no stream to abort,
 and a missing file is an exit status instead of a rejection that ends the script.
 Not run as a separate arm above, because nothing of the failing shape is left to exercise.
 Tradeoffs:
 no line numbers, truncation notices, or `offset`/`limit` continuation from `read`;
 shell quoting and the bash tool's own output budget apply;
 and the model loses the structured tool result it would otherwise get.

## What does not work

- **Catching it in the driver or a wrapper process.**
  The uncaught exception happens inside the `pi` CLI process;
  a parent that spawned it can only observe exit status 1,
  and no extension hook runs at that point.
- **`signal?.throwIfAborted()` immediately before `createReadStream`.**
  It only covers a check with no `await` between the check and the stream,
  and the abort lands inside `fsOpen()`/`handle.stat()`.
  Upstream considered and rejected this in pull request #12025.
- **A no-op `'error'` listener on the stream.**
  It stops this crash and hides every real stream error.
  Upstream rejected it for that reason.
- **`autoClose: true` with the signal kept.**
  The stream and the generator would both own and close the `FileHandle`,
  and it relies on an internal branch of Node's iterator cleanup.
  Upstream rejected it.
- **Patching the installed bundle (`pnpm patch`, a `sed` on `dist/core/tools/read.js`).**
  Mechanically it works, since the fix is two lines,
  but it edits third-party code, the file is minified, and the patch must be re-applied on every install.
  This repository's troubleshooting rules put local workarounds at our own boundary,
  and the prototype above lives only in `~/temp/agent`.
- **Reproducing the crash on `1.5.0`.**
  Not a defect to chase:
  `1.5.0` has no streaming read, and the harness above exits 0 there.

## Upstream filing artifact

### Upstream filing decision

Walking the six constraints,
 all of them hold,
 and the conclusion is still to post nothing,
 because the report and the fix already exist upstream.

1.  **Is it really upstream's fault?**
    Yes.
    It is a regression from upstream's own streaming read (pull request #11750, merged 2026-11-04),
    reproduced above on the shipped 1.5.2 artifacts,
    and stated as such by the change's author in issue #11956.
2.  **Can upstream fix it?**
    Yes, and they did:
    `ef9361ab` (pull request #12025) removes the signal from the stream and needs nothing new in the architecture.
3.  **Are they supporting this use case?**
    Yes.
    Streaming reads ship with their own test file,
    and the fix adds a regression test for exactly this abort window.
4.  **Would the repo welcome our contribution?**
    Yes, within a gate that matters here.
    `CONTRIBUTING.md` says issues and pull requests from new contributors are auto-closed by default
    and asks reporters to "write in your own voice";
    it allows AI-written code,
    asks for a clearly AI-labelled follow-up when issue text was generated,
    and permanently blocks accounts that send agent-generated tracker traffic.
    Pull requests need an `lgtm` from a maintainer first.
    No ban on AI-assisted reports was found,
    but a fifth duplicate from an unapproved account is exactly the traffic that gate exists for.
5.  **Will they likely fix it?**
    Already fixed, and a repository member (`davidbrai`, association `MEMBER`) replied in issue #12118
    that the fix came after v1.5.2, so it "will be in the next release".
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Not ours to write:
    upstream's own fix exists (`ef9361ab`),
    it is absent from 1.5.2,
    and the pre/post runs above confirm it removes the crash while keeping the script's error.
    Re-deriving the same patch would duplicate merged work.

Because nothing is filed, no draft is kept:
 there is no new-issue body, no comment body, and no reproduction repository.

### Candidate-fix applicability gate

The candidate fix `ef9361ab` is absent from the installed 1.5.2 source
 (`readLocalFileChunks` still takes `{ signal }` in the shipped `dist/core/tools/read.js`),
 the candidate issue's stack, version, and inputs match this incident
 (`dist/bundle/chunks/chunk-VEJDBY3O.js` frames, 1.5.2, a codemode `Promise.all` with one missing path),
 and the reproduction above fails exactly on that pre-fix source.
 The reading here is therefore "this incident ran on stale code",
 not "a new defect is open";
 the gate's excluded hypothesis is recorded here rather than in "What does not work",
 because it is the outcome, not a dead end.

### `.out-of-scope/` check

Read every file in `.out-of-scope/` on 2026-11-19.
 No exemption matches a Pi read-tool or codemode crash:
 `pi-gpt55-long-context.md` covers context-window metadata,
 `codex-harness.md` covers Codex rather than Pi,
 and `claude-code-upstream-bugs.md` covers the Anthropic CLI.
 Upstream tracking is therefore in scope,
 and the decision below rests on the duplicate and the merged fix, not on an exemption.

### Duplicate search and work in progress

Run 2026-11-19 against `earendil-works/pi`
 (`gh search issues`, `gh search prs`, `gh pr list --state all --search`, `gh api .../commits -f path=<file>`).

Canonical report, read in full with its comments:

- [#11956 Crash on cancelling a read during file open](https://github.com/earendil-works/pi/issues/11956),
  opened 2026-11-09, closed COMPLETED 2026-11-11.
  Its author hit it the same way (codemode `Promise.all`, one missing file),
  a second reporter traced the constructor-time destroy,
  another confirmed it on 1.5.1, and a repository member confirmed 1.5.2 shipped before the fix.

The fix:

- [#12025](https://github.com/earendil-works/pi/pull/12025),
  "do not crash when an aborted signal destroys an unwatched file stream",
  merged 2026-11-11, commit `ef9361ab`.
  Its body carries the five-step cause, the alternatives considered, and an audit of every other signal-carrying stream.

Duplicates opened after the fix was merged,
 each auto-closed by the new-contributor gate, each already answered with a pointer to #11956 and #12025:

- [#12118](https://github.com/earendil-works/pi/issues/12118) (2026-11-13, abort after the run finishes),
- [#12167](https://github.com/earendil-works/pi/issues/12167) (2026-11-14, sibling reads after ENOENT),
- [#12253](https://github.com/earendil-works/pi/issues/12253) (2026-11-17, in-flight `tools.read()`),
- [#12271](https://github.com/earendil-works/pi/issues/12271) (2026-11-18, script fails with a read pending).

Work in progress, none of it on this cause:

- Open pull requests touching `read.ts`: #11811 (UTF-16 and non-UTF-8 handling), unrelated.
- Open pull requests mentioning codemode: #11878, #11957, unrelated.
- Upstream branches: `triagent/issue-*`, `update-brace-expansion`, `win-bash-fetcher`, `windows-job-tree-kill`;
  none names the read stream or codemode aborts.
- Maintainer commits on `packages/coding-agent/src/core/tools/read.ts` end at `4be4cb4f` (2026-11-13, image warning),
  after `ef9361ab`;
  on `packages/codemode/src/runtime/execution.ts` the newest is `3daa2028` (2026-10-13).
  Nobody is working further in this path.

Nothing to add to any thread:
 #11956 carries the cause, the stack, the fix commit, and the 1.5.0 downgrade (stated by a member in #12118);
 the duplicates each already point at both.
 A comment from us would only repeat them.

### Risk to the poster

This repository runs a contribution gate:
 `.github/workflows/issue-gate.yml` auto-closes issues from authors not listed in `.github/APPROVED_CONTRIBUTORS`,
 `.github/workflows/approve-contributor.yml` adds a login after a maintainer replies `lgtmi` or `lgtm`,
 and `CONTRIBUTING.md` warns that agent-generated tracker traffic can get an account permanently blocked.
 Filing a fifth duplicate is the exact case that list and gate exist for,
 so a filing here would spend the account owner's standing for nothing.
 Filing is the account owner's decision; this doc recommends against it.

### Readiness check

Run 2026-11-19 against 1.5.2 (installed artifacts) and `main` at `5a14e69c`.

1.  Latest release and other lines:
    `1.5.2` is the newest npm version and the newest GitHub release, and it is affected (measured);
    `1.5.0` is not affected (measured);
    `main` carries the fix (source read, and the same change verified pre/post).
    `1.5.1`, and the 0.x line, were not run.
2.  Prior art:
    read #11956, #12025, #12118, #12167, #12253 and #12271 in full, including comments.
    The member's stated cause matches the trace here, so it is cited, not re-argued.
3.  Intake:
    `.github/ISSUE_TEMPLATE/bug.yml` (description, repro, expected, version; blank issues disabled by `config.yml`),
    `CONTRIBUTING.md`, and `.github/workflows/pr-gate.yml` (pull requests need `lgtm`).
    No filing was prepared, so no template was applied to a body.
4.  Automation:
    `issue-gate.yml` auto-closes new contributors' issues and labels them,
    `approve-contributor.yml` maintains the allow list,
    `issue-triage-labels.yml` applies canonical labels,
    `remove-inprogress-on-close.yml` clears an in-progress label on close.
5.  Duplicate, work-in-progress, and trust searches:
    re-run on 2026-11-19 with the queries named above;
    `.github/APPROVED_CONTRIBUTORS` read, and the fix commit `ef9361ab` confirmed present on `main`.
6.  The reproduction as written:
    harnesses 1 to 3 above were run exactly as shown on 2026-11-19,
    with the outputs quoted.
7.  A public reproduction repository:
    not created, because nothing is filed.
8.  Verdict:
    nothing to post.
    The blocker is absent, not unclear:
    there is no gap in the upstream thread, and the fix is merged.

## After a release carries `ef9361ab`

Re-check `npm view @earendil-works/pi-coding-agent dist-tags` and drop this doc's workarounds,
 then re-run `codemode-repro.ts sibling` against the new distribution:
 the expectation is the ENOENT result with exit 0, as in the prototype arm above.
