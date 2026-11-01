# Pi 0.84.2 `/reload` retains changed `.mjs` extension code while helper files update on disk

## Symptom

A Pi 0.84.2 session loaded an extension from a built `.mjs` entry point.
The extension bundle and its separately launched helper bundle were rebuilt while Pi remained open.
Running `/reload` reported success,
but the next tool call combined the old in-memory extension factory with the new helper file from disk.

For `@monochromatic-dev/pi-plugin-ask-user-question`,
the new helper required an `editorCommand` request field that the old extension did not write.
The helper printed:

```text
Error: Answer helper request editorCommand must be a nonempty string array.
```

The launched editor also opened `answer-helper.mjs` as answer content because the stale extension still used the old
ambient-editor protocol while the helper path now contained the new protocol.
A complete Pi process restart loaded one coherent artifact generation and restored correct behavior.

## Root cause

### Pi rebuilds its extension runtime

Pi does not deliberately retain the old `ExtensionRunner` during `/reload`.
At tag `v0.84.2`,
`packages/coding-agent/src/core/agent-session.ts:2610-2623` invalidates the old runner,
reloads resources,
and builds a new runtime:

```ts
async reload(options?: { beforeSessionStart?: () => void | Promise<void> }): Promise<void> {
  const oldRunner = this._extensionRunner;
  const previousFlagValues = oldRunner.getFlagValues();
  await emitSessionShutdownEvent(oldRunner, { type: "session_shutdown", reason: "reload" });
  oldRunner.invalidate();
  await this.settingsManager.reload();
  this.syncQueueModesFromSettings();
  resetApiProviders();
  await this._resourceLoader.reload();
  this._buildRuntime({
    activeToolNames: this.getActiveToolNames(),
    flagValues: previousFlagValues,
    includeAllExtensionTools: true,
  });
```

The resource loader also clears Pi's own extension-factory cache.
`packages/coding-agent/src/core/resource-loader.ts:387-392` says:

```ts
async reload(options?: ResourceLoaderReloadOptions): Promise<void> {
  resetTimings("extensions");

  if (this.loaded) {
    clearExtensionCache();
  }
```

The stale code therefore comes from the module loader below Pi's cache,
not from failure to replace Pi's runner or registry.

### Pi asks Jiti to bypass its module cache

`packages/coding-agent/src/core/extensions/loader.ts:436-455` creates Jiti with `moduleCache: false`
and imports the extension by its unchanged absolute path:

```ts
async function loadExtensionModule(extensionPath: string, cacheToken?: ExtensionCacheToken) {
  // Pi cache lookup omitted.

  const jiti = createJiti(import.meta.url, {
    moduleCache: false,
    // Runtime aliases omitted.
  });

  const module = await jiti.import(extensionPath, { default: true });
```

Jiti 2.7.0 documents the intended contract in `README.md:139-149`:

```md
### `moduleCache`

- Type: String
- Default: `true`
- Environment variable: `JITI_MODULE_CACHE`

Runtime module cache (enabled by default).

Disabling allows editing code and importing the same module multiple times.
```

That contract works for the `.ts` positive control but not for the `.mjs` entry.

### Jiti delegates asynchronous `.mjs` loading to Node's native ESM loader

Jiti 2.7.0 classifies `.mjs` as ESM.
`src/eval.ts:37-48` then disables transformation whenever ESM is loaded asynchronously,
without consulting `moduleCache`:

```ts
const isTypescript = /\.[cm]?tsx?$/.test(ext);
const isESM =
  ext === ".mjs" ||
  (ext === ".js" && readNearestPackageJSON(filename)?.type === "module");
const isCommonJS = ext === ".cjs";
const needsTranspile =
  evalOptions.forceTranspile ??
  (!isCommonJS &&
    !(isESM && evalOptions.async) &&
    (isTypescript || isESM || ctx.isTransformRe.test(filename) || hasESMSyntax(source)));
```

`src/eval.ts:66-77` passes the same filename to native import:

```ts
} else {
  debug(
    ctx,
    "[native]",
    evalOptions.async ? "[import]" : "[require]",
    filename,
  );

  if (evalOptions.async) {
    return Promise.resolve(
      nativeImportOrRequire(ctx, filename, evalOptions.async),
    ).catch((error: any) => {
```

The Node 26 [ESM URL documentation][node-esm-urls] states that ES modules are resolved and cached as URLs,
and that a changed query or fragment loads a module again.
Jiti supplies the unchanged filename,
so Node resolves the same URL and returns the first module instance.
Clearing Pi's map and setting Jiti `moduleCache: false` therefore cannot refresh the `.mjs` factory.

### A separately spawned helper reads current disk contents

The ask-user extension launches `answer-helper.mjs` in a new Node process.
That child does not share Pi's ESM module cache and therefore reads the rebuilt helper file.

The current extension writes the editor command at
`package/pi-plugin/ask-user-question/src/request-external-answer.ts:203-212`:

```ts
await writeHelperRequest({
  workspace,
  request: {
    host: channel.host,
    port: channel.port,
    token: channel.token,
    answerPath: workspace.answerPath,
    editorCommand,
  },
},);
```

The current helper requires it at
`package/pi-plugin/ask-user-question/src/helper-request.ts:66-78`:

```ts
return {
  port: requirePort({ value, },),
  token: requireStringField({ value, key: 'token', },),
  answerPath: requireStringField({ value, key: 'answerPath', },),
  editorCommand: requireStringArrayField({
    value,
    key: 'editorCommand',
  },),
};
```

The failure was exactly this mixed generation:
old extension code omitted `editorCommand`,
while the newly spawned helper required it.

## Verification

### Versions

- Pi `0.84.2`,
  source tag `v0.84.2`,
  commit `914cf1472e715297caa30db4b9535d534a9eb718`.
- Jiti `2.7.0`,
  source tag `v2.7.0`,
  commit `fd3bb289b75ed207edfb686d671ed50144f7e90f`.
- Node `26.7.0` for the installed Pi probe.
- Node `22` container for the Jiti patch probe.

### Pi loader harness

The harness uses Pi's installed loader directly.
It rewrites one extension at the same path,
clears Pi's factory cache as `/reload` does,
and compares `.ts` with `.mjs`.
Run it from the repository root:

```sh
node --input-type=module <<'EOF'
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const packageEntry = import.meta.resolve('@earendil-works/pi-coding-agent');
const loaderUrl = new URL('./core/extensions/loader.js', packageEntry);
const {
  clearExtensionCache,
  loadExtensionsCached,
} = await import(loaderUrl);

async function probe(extension) {
  const root = await mkdtemp(join(tmpdir(), `pi-${extension}-reload-probe-`));
  const extensionPath = join(root, `extension.${extension}`);
  const stateKey = `__piReloadProbe_${extension}`;
  const source = (version) =>
    `export default function () { globalThis.${stateKey} = ${JSON.stringify(version)}; }\n`;

  try {
    await writeFile(extensionPath, source('v1'), 'utf8');
    clearExtensionCache();
    await loadExtensionsCached([extensionPath], root);
    const first = globalThis[stateKey];

    await writeFile(extensionPath, source('v2'), 'utf8');
    clearExtensionCache();
    await loadExtensionsCached([extensionPath], root);
    const afterReload = globalThis[stateKey];

    return {
      extension,
      first,
      afterReload,
      reloaded: afterReload === 'v2',
    };
  }
  finally {
    delete globalThis[stateKey];
    clearExtensionCache();
    await rm(root, { recursive: true, force: true });
  }
}

console.log(JSON.stringify([
  await probe('ts'),
  await probe('mjs'),
]));
EOF
```

Observed output:

```json
[
  {"extension":"ts","first":"v1","afterReload":"v2","reloaded":true},
  {"extension":"mjs","first":"v1","afterReload":"v1","reloaded":false}
]
```

The `.ts` row is the positive control.
It proves the harness can observe a refreshed module.
The `.mjs` row isolates the native ESM cache failure.

### Real Pi artifact mismatch

The real session sequence was:

1. Start Pi with the old ask-user `.mjs` extension and old helper.
2. Rebuild both files with the new required `editorCommand` protocol.
3. Run `/reload`.
4. Call `ask_user_question`.
5. Observe `Answer helper request editorCommand must be a nonempty string array.`
6. Restart Pi completely.
7. Call `ask_user_question` again.
8. Submit two lines from Nano.

After the restart,
the tool returned exactly:

```text
nano line one
nano line two
```

### Jiti patch prototype

The prototype was applied to a disposable Jiti 2.7.0 clone.
The patch is saved as [pi-mjs-extension-reload.patch](pi-mjs-extension-reload.patch).
It makes native async ESM conditional on module caching being enabled.
When caching is disabled,
Jiti transforms and evaluates the current source instead.

The test and build ran in a secret-free Podman container bounded to 2 GiB RAM and 2 CPUs:

```sh
podman run --memory=2g --cpus=2 --rm \
  --volume "$HOME/temp/agent/jiti-20260826:/work:Z" \
  --workdir /work \
  docker.io/library/node:22-bookworm \
  bash -lc 'corepack enable && corepack prepare pnpm@10.30.3 --activate && pnpm build && pnpm vitest run test/esm-temp-file.test.ts'
```

Without the source fix,
the new test produced:

```text
✓ reloads changed .ts modules when moduleCache is false
× reloads changed .mjs modules when moduleCache is false
AssertionError: expected 'v1' to be 'v2'
Tests  1 failed | 5 passed (6)
```

With the patch:

```text
Test Files  1 passed (1)
Tests  6 passed (6)
```

## Verified workarounds

### Restart Pi after rebuilding a built extension

Build the package,
then exit and restart Pi before invoking the changed tool.
This loaded a coherent extension and helper generation and returned both Nano lines correctly.

Tradeoff:
the current process and in-memory interaction state are interrupted.
The saved Pi session can still be resumed.

### Use a TypeScript entry during extension development

The positive control shows Pi and Jiti reload a changed `.ts` entry at the same path.
A development-only package entry can therefore target TypeScript source while production remains built `.mjs`.

Tradeoff:
source loading differs from the shipped artifact boundary,
and independently built helper files can still drift unless their protocol remains compatible.

### Build before starting Pi

Avoid changing built extension artifacts in a running Pi process.
This prevents mixed generations without relying on reload behavior.

Tradeoff:
every extension edit requires a process restart before testing.

Ranking:
restart Pi after each build is preferred over a TypeScript development entry because it verifies the shipped artifact.
A TypeScript entry is preferred over relying only on build ordering because it preserves a usable reload loop
for extension-only edits.

## What does not work

- `/reload` after changing a `.mjs` bundle does not refresh that native ESM module in Pi 0.84.2 on Node.
  Pi's success message confirms resource reload completion,
  not that Node discarded native ESM cache entries.
- Clearing Pi's `extensionCache` is insufficient.
  The stale instance resides in Node's native ESM module map below that cache.
- Jiti `moduleCache: false` alone is insufficient for asynchronous `.mjs` imports in 2.7.0.
- Rebuilding only the helper makes the mismatch more visible,
  but does not refresh the old extension closure.
- Pi issue [#6108][] concerns the opposite release-binary failure,
  re-evaluating unchanged transitive dependencies.
  It does not cover stale `.mjs` entry points on the Node installation path.

## Upstream filing decision

The `.out-of-scope/` directory has no exemption for Jiti,
Pi extension reload,
or native ESM cache behavior.

Tracker searches covered open and closed Jiti issues and pull requests for
`mjs reload cache`,
`native ESM cache`,
and `moduleCache false`.
Jiti issue [#418][] is the matching report.
Its maintainer requested a Jiti-only minimal reproduction,
which the verification harness and prototype now provide.
No separate issue should be opened.

The six constraints are:

1. **Is it really upstream's fault?**
   Yes for Jiti.
   Its documented `moduleCache: false` contract says changed code can be imported repeatedly,
   but `.mjs` takes the exact native URL path and remains cached.
   Pi exposes the defect but does not create Node's cache behavior.
2. **Can upstream fix it?**
   Yes.
   The prototype changes one loader decision and preserves cached native ESM when `moduleCache` remains enabled.
3. **Are they supporting this use case?**
   Yes.
   Jiti documents repeated imports after editing as the purpose of disabling `moduleCache`.
4. **Would the repository welcome the contribution?**
   Probably.
   The Jiti 2.7.0 clone contains no `CONTRIBUTING.md`,
   issue template,
   pull-request template,
   or AI-contribution ban.
   Issue [#418][] remains open and a maintainer explicitly requested a minimal Jiti reproduction.
5. **Will they likely fix it?**
   Plausibly.
   The issue is labeled `bug` and has no rejection or wont-fix statement.
6. **Have we prototyped a minimal compatible fix?**
   Yes.
   The saved patch adds the `.ts` positive control and `.mjs` regression row,
   fails before the source change,
   and passes all targeted tests after it.

No external comment was posted.
The additive comment draft is ready for a human to review and post to [#418][]:

~~~md
I reproduced this with Jiti 2.7.0 alone on Node,
including a TypeScript positive control.

With `fsCache: false` and `moduleCache: false`,
write `config.ts` and `config.mjs` as `v1`,
import each,
rewrite each at the same path as `v2`,
and import again.
The `.ts` import returns `v2`.
The `.mjs` import returns stale `v1`.

The cause is `src/eval.ts`:
async ESM always skips transformation and calls native import with the unchanged file URL,
regardless of `moduleCache`.
Node then returns its cached ESM module.

I prototyped making the native-ESM shortcut conditional on `ctx.opts.moduleCache`.
With the added `.ts` and `.mjs` test rows,
the unpatched source reports 1 failed and 5 passed,
with `.mjs` receiving `v1` instead of `v2`.
The patch reports all 6 targeted tests passing.

This comment was prepared with AI assistance and reviewed against Jiti 2.7.0 source,
a Jiti-only reproduction,
and a pre-patch/post-patch test run.
~~~

[#418]: https://github.com/unjs/jiti/issues/418
[#6108]: https://github.com/earendil-works/pi/issues/6108
[node-esm-urls]: https://nodejs.org/api/esm.html#urls
