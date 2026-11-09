# Pi 0.78.0 `pi update` reintroduces npm allowScripts warnings through extension npm roots

Note 2026-11-09T16:56:27-05:00:
`.pi/npm` is not in the tree;
its files were deleted on 2026-06-15 in `32b6cd845f`.

## Symptom

Running `pi update` can print npm 11 `allowScripts` warnings for packages that the Monochromatic pnpm workspace
already removes with `pnpm-workspace.yaml` overrides:

```text
npm warn allow-scripts 87 packages have install scripts not yet covered by allowScripts:
npm warn allow-scripts   @google/genai@1.52.0 (install: (install scripts present))
npm warn allow-scripts   koffi@2.16.2 (install: (install scripts present))
npm warn allow-scripts   protobufjs@7.6.2 (install: (install scripts present))
```

The observed global Pi extension tree had one physical installed copy of each package:

```text
/var/home/user/.pi/agent/npm/node_modules/@google/genai/package.json
/var/home/user/.pi/agent/npm/node_modules/koffi/package.json
/var/home/user/.pi/agent/npm/node_modules/protobufjs/package.json
```

The `87 packages` count came from lockfile inventory duplication,
 not from 87 physical directories.

## Root cause

The cause has three layers.

### Pi extension installs do not use Monochromatic's pnpm overrides

`pi update` updates extension npm packages before self-update.
 Pi 0.78.0's installed CLI calls
`packageManager.update()` when the update target includes extensions.

`<pi-coding-agent>` below means
`node_modules/.pnpm/@earendil-works+pi-coding-agent@0.78.0/node_modules/@earendil-works/pi-coding-agent`.

```js
// <pi-coding-agent>/dist/package-manager-cli.js:456-465
case "update": {
    const target = options.updateTarget ?? { type: "all" };
    if (updateTargetIncludesExtensions(target)) {
        const updateSource = target.type === "extensions" ? target.source : undefined;
        await packageManager.update(updateSource);
        if (updateSource) {
            console.log(chalk.green(`Updated ${updateSource}`));
        }
```

The package manager then runs `npm install ... --prefix <installRoot> --legacy-peer-deps` for npm-sourced
extensions.

```js
// <pi-coding-agent>/dist/core/package-manager.js:887-890
async installNpmBatch(specs, scope) {
    const installRoot = this.getNpmInstallRoot(scope, false);
    this.ensureNpmProject(installRoot);
    await this.runNpmCommand(this.getNpmInstallArgs(specs, installRoot));
}
```

```js
// <pi-coding-agent>/dist/core/package-manager.js:1419
return ["install", ...specs, "--prefix", installRoot, "--legacy-peer-deps"];
```

Global extension installs use `agentDir/npm`;
 project extension installs use `.pi/npm`.

```js
// <pi-coding-agent>/dist/core/package-manager.js:1562-1569
getNpmInstallRoot(scope, temporary) {
    if (temporary) {
        return this.getTemporaryDir("npm");
    }
    if (scope === "project") {
        return join(this.cwd, CONFIG_DIR_NAME, "npm");
    }
    return join(this.agentDir, "npm");
}
```

That npm root is independent from the Monochromatic pnpm workspace.
 The pnpm overrides that remove
`@earendil-works/pi-ai>@google/genai` and `@earendil-works/pi-tui>koffi` do not affect
`/var/home/user/.pi/agent/npm`.

### `pi-mcp-adapter` directly depends on Pi host packages

The observed global settings include `npm:pi-mcp-adapter`,
 and the global extension npm project records
`pi-mcp-adapter` as a direct dependency.

`pi-mcp-adapter@2.8.0` declares `@earendil-works/pi-ai` and `@earendil-works/pi-tui` as regular dependencies,
 not
peer dependencies:

```jsonc
// /var/home/user/.pi/agent/npm/node_modules/pi-mcp-adapter/package.json:82-90
"dependencies": {
  "@earendil-works/pi-ai": "^0.74.0",
  "@earendil-works/pi-tui": "^0.74.0",
  "@modelcontextprotocol/ext-apps": "^1.2.2",
  "@modelcontextprotocol/sdk": "^1.25.1",
  "open": "^10.2.0",
  "recheck": "^4.5.0",
  "typebox": "^1.1.24",
  "zod": "^3.25.0 || ^4.0.0"
}
```

Those direct dependencies pull in the packages that the workspace had already removed from its pnpm graph.

`@earendil-works/pi-ai@0.74.2` depends on `@google/genai`:

```jsonc
// /var/home/user/.pi/agent/npm/node_modules/@earendil-works/pi-ai/package.json:71-78
"dependencies": {
  "@anthropic-ai/sdk": "^0.91.1",
  "@aws-sdk/client-bedrock-runtime": "^3.1030.0",
  "@google/genai": "^1.40.0",
  "@mistralai/mistralai": "^2.2.0",
  "http-proxy-agent": "^7.0.2",
  "https-proxy-agent": "^7.0.6",
  "openai": "6.26.0",
```

`@google/genai@1.52.0` depends on `protobufjs`:

```jsonc
// /var/home/user/.pi/agent/npm/node_modules/@google/genai/package.json:160-164
"dependencies": {
  "google-auth-library": "^10.3.0",
  "p-retry": "^4.6.2",
  "protobufjs": "^7.5.4",
  "ws": "^8.18.0"
```

`@earendil-works/pi-tui@0.74.2` declares `koffi` as an optional dependency:

```jsonc
// /var/home/user/.pi/agent/npm/node_modules/@earendil-works/pi-tui/package.json:39-44
"dependencies": {
  "get-east-asian-width": "^1.3.0",
  "marked": "^15.0.12"
},
"optionalDependencies": {
  "koffi": "^2.9.0"
},
```

### npm's warning count is inflated by symlinked-home lockfile duplication

npm 11 emits the warning by counting unreviewed install-script nodes and printing `unreviewedScripts.length`.

```js
// /var/home/user/.local/share/mise/installs/node/26.3.0/lib/node_modules/npm/lib/utils/reify-output.js:242-256
const count = unreviewedScripts.length
const pkg = count === 1 ? 'package has' : 'packages have'
const header = `${count} ${pkg} install scripts not yet covered by allowScripts:`

const lines = unreviewedScripts.map(({ node, scripts }) => {
  const { name, version } = trustedDisplay(node)
  /* istanbul ignore next: every test node has a name */
  const display = name || '<unknown>'
  const ver = version ? `@${version}` : ''
  const events = Object.entries(scripts)
    .map(([event, cmd]) => `${event}: ${cmd}`)
    .join('; ')
  return `  ${display}${ver} (${events})`
})
```

The observed global lockfile had 29 entries for each of the three warning packages:

```text
node_modules/@google/genai: count=29 versions=1.52.0
node_modules/koffi: count=29 versions=2.16.2
node_modules/protobufjs: count=29 versions=7.6.2,7.6.1,7.6.0
root version=file:../../../../../var/home/user/.pi/agent/npm
package entries=7011
```

The duplication is reproducible when npm receives a `--prefix` path through `/home/user` while `/home` is a symlink to
`var/home`.
 Repeated lockfile-only installs in a scratch prefix under `/home/user` add another `var/` segment each run:

```json
[
  {
    "run": "1",
    "version": "file:../../../var/home/user/pi-prefix-loop.i4irlY",
    "packageCount": 3,
    "sample": [
      "../../../var/home/user/pi-prefix-loop.i4irlY/node_modules/@aliou/pi-linkup"
    ]
  },
  {
    "run": "2",
    "version": "file:../../../var/home/user/pi-prefix-loop.i4irlY",
    "packageCount": 5,
    "sample": [
      "../../../var/home/user/pi-prefix-loop.i4irlY/node_modules/@aliou/pi-linkup",
      "../../../var/var/home/user/pi-prefix-loop.i4irlY/node_modules/@aliou/pi-linkup"
    ]
  },
  {
    "run": "3",
    "version": "file:../../../var/home/user/pi-prefix-loop.i4irlY",
    "packageCount": 7,
    "sample": [
      "../../../var/home/user/pi-prefix-loop.i4irlY/node_modules/@aliou/pi-linkup",
      "../../../var/var/home/user/pi-prefix-loop.i4irlY/node_modules/@aliou/pi-linkup",
      "../../../var/var/var/home/user/pi-prefix-loop.i4irlY/node_modules/@aliou/pi-linkup"
    ]
  }
]
```

## Verification

Version context:

- Pi CLI:
   `0.78.0`
- Node:
   `v26.3.0`
- npm:
   `11.16.0`
- `HOME`:
   `/home/user`
- `/home`:
   symlink to `var/home`

`npm ls` in the global Pi extension root shows the direct ancestry.
 The command exits with `ELSPROBLEMS` because the
installed Pi peer packages are invalid for `@aliou/pi-linkup`'s exact `0.74.0` peer range,
 but the dependency chain is
still printed:

```text
pi-extensions@ /var/home/user/.pi/agent/npm
├─┬ @aliou/pi-linkup@0.11.0
│ ├─┬ @earendil-works/pi-ai@0.74.2 invalid: "0.74.0" from node_modules/@aliou/pi-linkup
│ │ └─┬ @google/genai@1.52.0
│ │   └── protobufjs@7.6.2
│ └─┬ @earendil-works/pi-tui@0.74.2 invalid: "0.74.0" from node_modules/@aliou/pi-linkup
│   └── koffi@2.16.2
└─┬ pi-mcp-adapter@2.8.0
  └── @earendil-works/pi-ai@0.74.2 deduped invalid: "0.74.0" from node_modules/@aliou/pi-linkup
```

A fresh scratch npm root with the observed global extension set and `pi-mcp-adapter` reproduced the three unreviewed
install-script packages with `--legacy-peer-deps --package-lock-only`:

```json
{
  "piMcpAdapter": [{ "k": "node_modules/pi-mcp-adapter", "version": "2.8.0" }],
  "piAi": [{ "k": "node_modules/@earendil-works/pi-ai", "version": "0.74.2" }],
  "piTui": [{ "k": "node_modules/@earendil-works/pi-tui", "version": "0.74.2" }],
  "genai": [{ "k": "node_modules/@google/genai", "version": "1.52.0" }],
  "koffi": [{ "k": "node_modules/koffi", "version": "2.16.2" }],
  "protobufjs": [{ "k": "node_modules/protobufjs", "version": "7.6.2" }],
  "packageCount": 227
}
```

The same scratch install without `pi-mcp-adapter` contained none of the target packages:

```json
{
  "piAi": false,
  "piTui": false,
  "genai": false,
  "koffi": false,
  "protobufjs": false,
  "packageCount": 18
}
```

The stale project-local root under `Monochromatic/.pi/npm` is a separate artifact.
 Its package files are dated
2026-05-14,
 project settings contain no packages,
 and its `protobufjs` version is `7.5.8`,
 not the `7.6.2` version
reported by the 2026-06-01 warning.
 It was not the source of the new `pi update` warning.

## Verified workarounds

### Remove or replace `pi-mcp-adapter`

Removing `pi-mcp-adapter` from the scratch extension set removes `@earendil-works/pi-ai`,
 `@earendil-works/pi-tui`,
`@google/genai`,
 `koffi`,
 and `protobufjs` from the npm lock.
 Tradeoff:
 MCP adapter functionality is gone unless a
replacement extension supplies it.

### Patch or fork `pi-mcp-adapter` metadata

If Pi's extension loader supplies `@earendil-works/pi-ai` and `@earendil-works/pi-tui` as host APIs,
 the package should
move those entries from `dependencies` to `peerDependencies`.
 Tradeoff:
 this must be verified against the adapter's
sampling and TUI paths,
 because `sampling-handler.ts` imports runtime values from `@earendil-works/pi-ai`,
 and panel
rendering imports runtime values from `@earendil-works/pi-tui`.

### Rebuild the Pi extension npm root from a canonical path

A clean scratch install through a `/var/home/user/...` prefix did not create `var/var/...` lockfile entries.
 Rebuilding
`/var/home/user/.pi/agent/npm` from a clean package root through the canonical path should remove the inflated warning
count.
 Tradeoff:
 this only fixes duplicate lockfile inventory.
 While `pi-mcp-adapter` remains installed,
 the clean graph
still contains one each of `@google/genai`,
 `koffi`,
 and `protobufjs`.

## What does not work

- Monochromatic's `pnpm-workspace.yaml` overrides do not affect Pi's separate npm extension roots.
- `--legacy-peer-deps` stops npm from auto-solving peers,
   but it does not remove regular dependencies declared by
  `pi-mcp-adapter`.
- `npm approve-scripts` or `npm deny-scripts` changes script policy only.
   It does not remove `@google/genai`,
   `koffi`,
  or `protobufjs` from the dependency graph.
- Cleaning only duplicated lockfile entries reduces the warning count from inflated multiples to three,
   but it does not
  remove the three packages while `pi-mcp-adapter` remains.

## Draft upstream issues

### Why we do not file upstream yet

#### `pi-mcp-adapter` package metadata

1. **Is it really upstream's fault?
   ** Likely yes.
    Pi extension packages are loaded by a host that already provides Pi
   APIs,
    and Pi 0.78.0 explicitly disables peer auto-resolution for host-provided `@earendil-works/pi-*` peers.
2. **Can upstream fix it?
   ** Yes if the adapter only needs host-provided Pi APIs.
    The minimal change is package metadata:
   move `@earendil-works/pi-ai` and `@earendil-works/pi-tui` from `dependencies` to `peerDependencies`.
3. **Are they supporting this use case?
   ** Yes.
    The package is a Pi extension and declares a `pi.extensions` entry.
4. **Will they likely fix it?
   ** Unknown.
    Maintenance activity was not audited beyond the published 2.6.1,
    2.7.0,
    and
   2.8.0 manifests,
    which all retain the same direct Pi dependencies.
5. **Have we prototyped a minimal fix compatible with their architecture?
   ** Not yet.
    Runtime verification must exercise
   MCP sampling and TUI panels after changing the manifest.

#### npm `--prefix` with symlinked `/home`

1. **Is it really upstream's fault?
   ** Likely yes.
    npm repeatedly adds logically duplicate lockfile entries when
   `--prefix` points through `/home/user` and `/home` is a symlink to `var/home`.
2. **Can upstream fix it?
   ** Unknown.
    The failing behavior is in npm's lockfile path handling,
    but the exact source hunk
   was not traced.
3. **Are they supporting this use case?
   ** Unknown.
    `--prefix` is a documented npm install option,
    but symlinked home
   directories were not checked against npm's tests.
4. **Will they likely fix it?
   ** Unknown.
    npm maintenance in this code path was not audited.
5. **Have we prototyped a minimal fix compatible with their architecture?
   ** No. The consumer-side workaround is to run
   Pi extension npm operations through canonical `/var/home/user/...` paths or rebuild the lockfile from that path.

### Draft issue for `pi-mcp-adapter`, do not file as-is

~~~md
Title: Move Pi host packages from dependencies to peerDependencies

`pi-mcp-adapter@2.8.0` declares `@earendil-works/pi-ai` and `@earendil-works/pi-tui` as regular dependencies:

```json
"dependencies": {
  "@earendil-works/pi-ai": "^0.74.0",
  "@earendil-works/pi-tui": "^0.74.0"
}
```

When Pi installs extensions with npm, those direct dependencies install a second Pi host stack under the extension npm
root. That pulls in `@google/genai`, `protobufjs`, and `koffi`, and npm 11 warns that their install scripts are not
covered by `allowScripts`.

Pi 0.78.0's extension installer already treats `@earendil-works/pi-*` packages as host-provided peers and passes
`--legacy-peer-deps` for npm installs, but that cannot suppress regular dependencies.

Suggested fix: move `@earendil-works/pi-ai` and `@earendil-works/pi-tui` to `peerDependencies`, with suitable optional
metadata if the adapter can load without one of the host APIs. Verify MCP sampling and TUI panel code paths after the
metadata change.
~~~
