# DSH 0.2.1-rc.5 skips linked bundles and fails to load its native binding

## Symptom

On 2026-10-30,
`dsh --profile web` emitted separate failures:

- `dsh: skipping profile bundle` for Advisor and NeuralWatt,
  followed by `is incompatible with dsh 0.2.1-rc.5` and requirements containing `catalog:`.
- `dsh: host preparation failed: No usable native binding found for node-addon-require-builtin-linux-x64-gnu (auto)`.
  The first recorded attempt was `Cannot find module 'node-addon-require-builtin-linux-x64-gnu'`.

The compatibility warnings and native loader exception have different causes.
Fixing the bundle manifests removed the warnings while leaving the native exception reproducible.

## Root cause

### Linked manifests expose catalog references to DSH

The profile links directly to this repository's Advisor and NeuralWatt package directories.
Their `peerDependencies` retain `catalog:` specifications because no packaging step rewrites them.

[DSH's compatibility evaluator][compatibility-source],
`packages/boot/app-boot/src/plugin-compatibility.ts:55`,
checks an explicit host requirement first:

```ts
// packages/boot/app-boot/src/plugin-compatibility.ts:57
if (Object.hasOwn(engines, 'dsh')) {
  const range = engines.dsh
  if (typeof range !== 'string') throw new Error('Plugin manifest engines.dsh must be a string')
  return satisfies(runtimeVersion, range) ? {} : { 'engines.dsh': range }
}
```

Without that field,
the same function checks DSH peer ranges with semver.
It recognizes the enumerated `workspace:` forms,
but does not expand a pnpm catalog:

```ts
// packages/boot/app-boot/src/plugin-compatibility.ts:70
const requirement = ['workspace:^', 'workspace:~', 'workspace:*'].includes(range) ? runtimeVersion : range
if (!satisfies(runtimeVersion, requirement)) unmet[name] = range
```

The fix adds `engines.dsh: "0.2.1-rc.5"` to both package manifests.
The exact prerelease records the host exercised here;
it does not claim compatibility with untested releases.
Dependencies remain catalog-managed.
No exact-version risk exemption was granted.

### The native loader resolves from the wrong package

The installed `node-addon-require-builtin@0.1.9` dependency directory contained both
`node-addon-native-custom-loader` and `node-addon-require-builtin-linux-x64-gnu`.
Resolving the platform package using `createRequire` anchored at the entry package succeeded,
and its native diagnostic reported `backend: 'napi'`,
`abi: 'napi-v9'`.
This rules out a missing download or an unusable native binary for this incident.

[The loader source][native-source],
`packages/loader/index.ts:582`,
uses its own CommonJS scope:

```ts
// packages/loader/index.ts:582
const binding = require(candidate.request) as NativeBinding;
const bindingPath = binding.bindingPath || require.resolve(candidate.request);
```

Under pnpm's isolated layout,
the platform dependency belongs to the entry package,
not the shared loader.
The loader's bare `require` cannot see that sibling dependency.
The fallback attempts then fail because this installation has no local build artifacts.

The relevant upstream revision is
`a58d3599a2e551d7d706d1ff796c9fd4bbaff63c` (`release: 0.1.9`).
The DSH source trace uses
`ce7073d51254c26cd63b22a60835b8f948deb447`;
the same compatibility logic was inspected in the installed rc.5 bundle.
Both source selections were opened and their quoted text confirmed in `agent-browser`.

## Verified repair

The manifest changes landed in `d2d4339f85`.
The package READMEs explain why the explicit host version is required.

For the native loader,
the following scoped configuration was tested in a disposable pnpm project,
then applied to the user's pnpm configuration and the existing DSH global installation workspace:

```yaml
# ~/.config/pnpm/config.yaml and the DSH installation's pnpm-workspace.yaml
# Loader 0.1.9 resolves the entry package binary from its own scope.
packageExtensions:
  node-addon-native-custom-loader@0.1.9:
    optionalDependencies:
      node-addon-require-builtin-linux-x64-gnu: 0.1.9
```

The user configuration preserves the repair for later installs.
The installation workspace also records it locally.
Existing configuration was preserved,
and backups were written under the private scratch directory before editing.

The existing installation was reinstalled with its package manager:

```sh
# Run with DSH_INSTALL_DIR set to the existing global project's directory.
pnpm --dir "$DSH_INSTALL_DIR" install --no-frozen-lockfile --ignore-scripts
```

This invocation retained `@deepseek-ai/dsh@0.2.1-rc.5`.
Pnpm verified the existing lockfile against supply-chain policies,
and generated its own updated installation state.
The repository's concurrently edited `pnpm-lock.yaml` was not changed by this repair.

Tradeoffs:

- The extension is specific to loader `0.1.9` and this Linux x64 glibc host.
  Other platforms require their corresponding optional package.
- The loader now has an additional optional dependency edge to the platform package.
  Remove the extension when an upstream release resolves from the entry package correctly.
- `--ignore-scripts` was sufficient for this existing installation's prebuilt native package.
  This does not establish that every fresh DSH installation can skip all lifecycle scripts.

## Verification

Versions exercised:
DSH `0.2.1-rc.5`,
native loader `0.1.9`,
pnpm `12.16.1`,
and Node `26.13.0` on Linux x64 glibc.

The native fixture had this manifest and the extension configuration shown in the repair section:

```json
{
  "private": true,
  "dependencies": {
    "node-addon-require-builtin": "0.1.9"
  }
}
```

Its dependency installation and consumer probe were:

```sh
# Run in the disposable native fixture directory.
pnpm install --ignore-scripts --no-frozen-lockfile
node -e 'const n=require("node-addon-require-builtin");console.log(n.getBindingInfo());console.log(typeof n.requireBuiltin("fs").readFileSync)'
```

The result reported `bindingSource: 'optional-package'`,
`backend: 'napi'`,
`abi: 'napi-v9'`,
and `function`.

The startup probe used a disposable `DSH_HOME` with a copy of the web profile,
its installed plugin files,
and absolute links back to the workspace plugins.
Credentials and existing session data were not copied.
The initial probe reproduced both reported errors:

```sh
# DSH_HOME points to the disposable copy, not the real profile.
dsh --profile web --help
```

Working cases after repair:

- The same help command exits successfully and prints the web application's flags.
- `dsh --profile web --host 127.0.0.1 --port 0 --no-open` starts the copied profile.
- Startup contains no skipped bundles,
  native-binding exception,
  or failed plugin activation.
- Opening the server's generated authentication URL in `agent-browser` renders
  `DeepSeek Harness`,
  the new-session interface,
  and the Plugins panel.
- The Plugins panel lists Advisor,
  NeuralWatt provider,
  Thinking dismissal shadow,
  Search and Fetch,
  and Conversation rewind.
  The browser error and console checks returned no entries.

Failing controls and limits:

- Before the manifest fix,
  both linked bundles were skipped with `catalog:` requirements.
- After only the manifest fix,
  the native loader still failed with the reported exception.
- A request without an authenticated browser session receives HTTP 401.
  The web server's authentication flow must be exercised through its generated URL.
- This is startup and activation verification.
  The disposable profile has no credentials,
  so no model inference request was made.

## What does not work

Installing another copy of the same native binary is not the diagnosis:
the binary already existed and loaded from the entry package's scope.
The repaired dependency graph gives the shared loader access to it.

The first full-boot fixture symlinked its entire `node_modules` directory to the real profile.
That caused `dsh-rewind-plugin` to resolve outside the disposable profile's interception scope,
producing a separate `ERR_MODULE_NOT_FOUND` for `@deepseek-ai/dsh-llm`.
Copying the installed files into the fixture and preserving absolute workspace-link targets
removed that fixture-only error.
No Rewind package or real profile configuration was changed.

## Upstream filing decision

Checked on 2026-10-30:

1.  Upstream responsibility:
    the native loader's resolution scope is upstream code;
    the missing explicit compatibility declarations were our package metadata.
2.  Fixability:
    resolving the optional package relative to the supplied entry package directory
    is the candidate upstream correction.
    No upstream code was modified.
3.  Supported use:
    upstream documents published optional platform packages and supports Linux x64 glibc with Node 26.
    The installed prebuild passed its runtime validation.
4.  Contribution intake:
    `gh repo view deepseek-ai/dsh-node-addon-require-builtin --json hasIssuesEnabled,hasDiscussionsEnabled`
    returned both fields as `false`.
    `gh issue list` explicitly reported that the repository has disabled issues.
5.  Existing work:
    `gh pr list --repo deepseek-ai/dsh-node-addon-require-builtin --state all --limit 20`
    returned no entries.
    An issue duplicate search cannot run against the disabled issue tracker.
6.  Prototype:
    the consumer-side dependency extension was tested with the actual binary and DSH boot.
    An upstream source patch was not prototyped;
    the issue intake is disabled.

No matching exemption was found in `.out-of-scope/`.
No upstream message was sent.
The retained diagnosis and candidate correction are the filing artifact;
they are not a fileable issue while that repository's issue intake remains disabled.

[compatibility-source]: https://github.com/deepseek-ai/deepseek-harness/blob/ce7073d51254c26cd63b22a60835b8f948deb447/packages/boot/app-boot/src/plugin-compatibility.ts#L55-L72
[native-source]: https://github.com/deepseek-ai/dsh-node-addon-require-builtin/blob/a58d3599a2e551d7d706d1ff796c9fd4bbaff63c/packages/loader/index.ts#L576-L586
