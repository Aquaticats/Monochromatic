# Electron 43.1.0 ESM main process top-level `await app.whenReady()` deadlocks startup

## Symptom

An Electron app whose main process is native ESM can hang before creating any window when its entry
module uses top-level `await app.whenReady()`.

The hung pattern is:

```js
// doc/troubleshooting/electron-esm-whenready.md
import { app } from 'electron';

await app.whenReady();
startApplication();
```

The working pattern keeps the `await` inside an async function that is called but not awaited by the
entry module:

```js
// doc/troubleshooting/electron-esm-whenready.md
import { app } from 'electron';

void startApplication();

async function startApplication() {
  await app.whenReady();
  startApplicationWindow();
}
```

This repo hit the symptom while building
`package/electron-infra/app-counter/src/main.ts`:
 the app logged that it was waiting for Electron
readiness,
 then the nested Wayland boundary test never observed the renderer state.
 Replacing the
entry-module top-level await with `startElectronCounterApp()` fixed the startup path.

## Root cause

Electron exposes an internal `process.appCodeLoaded` callback to the browser-process Node
environment.
 In `electron/electron` commit `54acd1b3900db4503dc486691f2e68c3aec1efe3`,
`shell/common/node_bindings.cc:991` to `995` installs that callback:

```cpp
// shell/common/node_bindings.cc
process.SetMethod("appCodeLoaded",
                  base::BindRepeating(&NodeBindings::SetAppCodeLoaded,
                                      base::Unretained(this)));
```

The callback only flips `app_code_loaded_` to true.
`shell/common/node_bindings.cc:1085` to `1087`:

```cpp
// shell/common/node_bindings.cc
void NodeBindings::SetAppCodeLoaded() {
  app_code_loaded_ = true;
}
```

The browser startup path waits for that signal before continuing.
 `shell/common/node_bindings.h:178`
to `180` states the contract:

```cpp
// shell/common/node_bindings.h
// Blocks until app code is signaled to be loaded via |SetAppCodeLoaded|.
// Only has an effect if called in the browser process
void JoinAppCode();
```

`electron_browser_main_parts.cc:280` to `284` calls the wait after loading the Node environment:

```cpp
// shell/browser/electron_browser_main_parts.cc
// Load everything.
node_bindings_->LoadEnvironment(node_env_.get());

// Wait for app
node_bindings_->JoinAppCode();
```

`JoinAppCode()` pumps libuv until `app_code_loaded_` becomes true.
`shell/common/node_bindings.cc:1105` to `1111`:

```cpp
// shell/common/node_bindings.cc
// Pump the event loop until we get the signal that the app code has finished
// loading
while (!app_code_loaded_ && !browser->is_shutting_down()) {
  int r = uv_run(uv_loop_, UV_RUN_ONCE);
  if (r == 0) {
    base::RunLoop().QuitWhenIdle();  // Quit from uv.
    break;
```

For ESM entry points,
 `lib/browser/init.ts:194` to `199` awaits the main module import before calling
`appCodeLoaded`:

```ts
// lib/browser/init.ts
runEntryPointWithESMLoader(async (cascadedLoader: any) => {
  try {
    await cascadedLoader.import(main.toString(), undefined, Object.create(null));
    appCodeLoaded!();
  } catch (err) {
    appCodeLoaded!();
```

The `ready` promise resolves later.
 `shell/browser/browser.cc:220` to `225` marks the app ready and
notifies observers:

```cpp
// shell/browser/browser.cc
is_ready_ = true;
if (ready_promise_)
  ready_promise_->Resolve();

for (BrowserObserver& observer : observers_)
  observer.OnFinishLaunching(launch_info.Clone());
```

`app.whenReady()` is bound to `Browser::WhenReady`.
 `shell/browser/browser.cc:228` to `235` shows that
it returns the promise resolved by the ready path:

```cpp
// shell/browser/browser.cc
v8::Local<v8::Value> Browser::WhenReady(v8::Isolate* isolate) {
  if (!ready_promise_) {
    ready_promise_ = std::make_unique<gin_helper::Promise<void>>(isolate);
    if (is_ready()) {
      ready_promise_->Resolve();
    }
  }
  return ready_promise_->GetHandle();
```

The deadlock is therefore circular:

- ESM startup waits for the app entry import to finish before `appCodeLoaded()` runs.
- A top-level `await app.whenReady()` keeps that import unfinished.
- Electron startup waits in `JoinAppCode()` before reaching the path that resolves `whenReady()`.

Electron's ESM tutorial already says ES modules are asynchronous and only side effects from the main
entry imports execute before `ready`.
 `doc/tutorial/esm.md:53` to `57` in the upstream clone:

```md
ES Modules are loaded **asynchronously**. This means that only side effects
from the main process entry point's imports will execute before the `ready` event.

This is important because certain Electron APIs (e.g. [`app.setPath`](../api/app.md#appsetpathname-path))
need to be called **before** the app's `ready` event is emitted.
```

The current tutorial does not explicitly say that `await app.whenReady()` itself must not be the
entry module's top-level await.

## Verification

Verified against:

- Electron package version:
   `43.1.0`,
   resolved by
  `package/electron-infra/app-counter/node_modules/electron`.
- Electron source clone:
   `electron/electron` at
  `54acd1b3900db4503dc486691f2e68c3aec1efe3`.
- Runtime boundary:
   `package/nested-wayland-session/cli` with `DISPLAY` cleared and
  `app.commandLine.appendSwitch('ozone-platform', 'wayland')` in each fixture app.

The reproduction harness creates temporary ESM apps,
 launches each through the repo nested Wayland
session,
 and checks whether the app writes a readiness file within the deadline:

```js
// doc/troubleshooting/electron-esm-whenready.md
const prefix = `import { app } from 'electron';
import { writeFileSync } from 'node:fs';
app.commandLine.appendSwitch('ozone-platform', 'wayland');
`;

const deadlockingMain = `${prefix}await app.whenReady();
writeFileSync(__READY__, 'ready');
app.quit();
`;

const workingMain = `${prefix}void start();
async function start() {
  await app.whenReady();
  writeFileSync(__READY__, 'ready');
  app.quit();
}
`;
```

The command run from `package/electron-infra/app-counter` was a `node --input-type=module`
harness that used `require('electron')` for the Electron binary and
`package/nested-wayland-session/cli/target/release/monochromatic-nested-wayland-session` for the
Wayland boundary.

Observed result:

```json
{
  "cases": [
    {
      "name": "top-level-await",
      "wroteReady": false,
      "content": "",
      "exit": {
        "code": null,
        "signal": "SIGTERM"
      }
    },
    {
      "name": "async-function",
      "wroteReady": true,
      "content": "ready",
      "exit": {
        "code": null,
        "signal": "SIGTERM"
      }
    }
  ]
}
```

### Working catalog

- Static imports before `ready`,
   followed by `void startApplication()` and an internal
  `await app.whenReady()`.
- `app.whenReady().then(...)` without returning or awaiting that promise from the entry module.
- Early side effects that must happen before `ready`,
   such as command-line switch setup,
   when they
  run before the non-awaited startup function is called.

### Failing catalog

- Entry-module `await app.whenReady()`.
- Entry-module top-level await of any promise that cannot resolve until Electron reaches app
  readiness.

## Verified workarounds

### Start an async function without awaiting it at module top level

Patch shape used by `package/electron-infra/app-counter/src/main.ts`:

```ts
// doc/troubleshooting/electron-esm-whenready.md
void startElectronCounterApp();

async function startElectronCounterApp(): Promise<void> {
  await app.whenReady();
  createMainWindow();
}
```

Tradeoff:
 errors thrown after the first async boundary must be caught or otherwise surfaced inside
the startup function,
 because the entry module no longer awaits the promise.

### Use `app.whenReady().then(...)`

```ts
// doc/troubleshooting/electron-esm-whenready.md
app.whenReady().then(() => {
  createMainWindow();
});
```

Tradeoff:
 this repo's TypeScript style prefers `async` and `await`,
 so this workaround is better for
external Electron examples than for repo code.

## What does not work

- Moving the same `await app.whenReady()` to another statically imported ESM module still keeps the
  primary import chain unresolved,
   so `appCodeLoaded()` still waits.
- Waiting longer does not help.
   The verification gave the deadlocking fixture a deadline and it did
  not write the ready file,
   while the async-function fixture wrote the ready file in the same nested
  Wayland environment.
- Switching from pure Wayland to X11 would not address the causal chain,
   because the deadlock occurs
  before the Electron `ready` promise resolves.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers Electron ESM startup documentation.
 Checked
`.out-of-scope/bun-install.md`,
 `.out-of-scope/cargo-workspace.md`,
`.out-of-scope/claude-code-upstream-bugs.md`,
 `.out-of-scope/codex-harness.md`,
`.out-of-scope/jsr.md`,
 `.out-of-scope/lightningcss.md`,
`.out-of-scope/low-impact-typescript-formatting.md`,
 `.out-of-scope/module-es-monolith.md`,
`.out-of-scope/pi-gpt55-long-context.md`,
 `.out-of-scope/terminal-title-fork-parity-tests.md`,
 and
`.out-of-scope/typescript-project-references.md`.

Duplicate search:

- `gh issue list --repo electron/electron --state all --search "whenReady ESM" --limit 10` found
  `electron/electron#40719`,
   closed as not planned,
   with the same symptom.
- `gh pr list --repo electron/electron --state all --search "whenReady ESM" --limit 10` found
  `electron/electron#41369`,
   an open draft docs PR for the same warning.

Constraint check:

- Is it really upstream's fault?
   Yes for documentation clarity,
   not for runtime behavior.
   Electron's
  code intentionally waits for the ESM import chain before readiness.
- Can upstream fix it?
   Yes.
   A docs note can state that `app.whenReady()` must not be top-level
  awaited in a main-process ESM entry.
- Are they supporting this use case?
   Yes.
   `doc/tutorial/esm.md` documents main-process ESM.
- Would the repo welcome contribution?
   Yes with normal contribution process.
   The open draft PR has
  maintainer review asking to explain the underlying reason,
   not rejecting the topic.
- Will they likely fix it?
   Plausible.
   The draft PR remains open and a maintainer asked whether the
  author still wants it landed in 2026.
- Have we prototyped a minimal fix compatible with their architecture?
   Yes by reference to existing
  upstream PR `electron/electron#41369`,
   which is a docs-only fix.
   This repo also verified the
  runtime claim against Electron `43.1.0`.

### Additive comment draft for `electron/electron#41369`

Do not post without user approval.
 This draft adds a current-version reproduction and source trace
to the existing docs PR instead of opening a duplicate issue.

~~~md
I reproduced the top-level `await app.whenReady()` deadlock on Electron `43.1.0` using a native ESM
main process under a nested Wayland compositor.

The source path still matches the explanation requested in review:

- `lib/browser/init.ts` awaits `cascadedLoader.import(main.toString(), ...)` before calling
  `appCodeLoaded()`.
- `shell/common/node_bindings.cc` has `JoinAppCode()` pump the loop until `app_code_loaded_` is true.
- `shell/browser/browser.cc` resolves the `whenReady()` promise only from the later readiness path.

A minimal ESM entry with top-level `await app.whenReady()` never wrote its ready marker within the
same deadline where this equivalent non-top-level await version did:

```js
import { app } from 'electron';
import { writeFileSync } from 'node:fs';

void start();

async function start() {
  await app.whenReady();
  writeFileSync(process.env.READY_FILE, 'ready');
  app.quit();
}
```

That matches this PR's proposed warning and confirms the behavior is still current on Electron
`43.1.0`.
~~~
