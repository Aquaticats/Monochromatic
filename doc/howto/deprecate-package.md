# HOWTO.deprecate-package

Procedural guide for archiving a workspace package while keeping it
installable indefinitely for outside consumers.
Codifies what was learned during the 2026-05-19 deprecation of
`@monochromatic-dev/mcp-nvim` (commits `374095ac`,
 `0c279114`,
 `4cdff544`).
Those commits bundled with tsdown through `@monochromatic-dev/config-tsdown`,
removed from `package/` on 2026-07-15 (commit `844e9dedda`,
 surviving only under `package-paused/` and `package-deprecated/`).
The shared bundling config is now `@monochromatic-dev/config-rolldown`
(`package/config/rolldown`) consumed through `rolldown.node.config.ts`,
and the steps below name it.

## When this applies

You stop maintaining a package,
 but you want existing or new consumers
to be able to keep using it.
 Three preconditions usually hold:

- The package was actually shipped or shared with outside users.
- You want a hard deprecation signal (warning on `npm install`),
   not just
  a README banner buried in the monorepo.
- You do not want the package to keep dragging through monorepo
  housekeeping (lint sweeps,
   dep bumps,
   config migrations).

If only the third holds,
 a `package-deprecated/` move plus a README banner
is enough.
 Skip the npm publish steps.

If outside consumption is via cloning the monorepo only,
 npm publish is
optional.
 The benefit of publishing once:
 npm's no-unpublish policy
(packages older than 72 hours cannot be unpublished) guarantees indefinite
availability.
 The deprecation message fires on every `npm install`,
giving users an unambiguous signal.

## Mental model

The `package-deprecated/<category>/<name>/` directory mirrors `package/<category>/<name>/`.
Both globs (`package-deprecated/*/*` and `package/*/*`) appear in
`pnpm-workspace.yaml` (so workspace dep resolution still works) and in
`mise.no-env.toml#config_roots` (so the package's mise tasks are still discoverable
for a one-time rebuild).
The deprecation signal is the directory name plus the npm-registry deprecation
marker.

The published bundle is fully self-contained:
 the `alwaysBundle`
bundle-inclusion patterns of `@monochromatic-dev/config-rolldown`
inline `@monochromatic-dev/**` workspace deps and any other npm deps you
add.
 The published `package.json` should have zero (or near-zero) runtime
dependencies,
 so consumers never run into 404s on private workspace deps.

## Prerequisites

- `pnpm whoami` returns your npm username (pnpm is authenticated against npm).
- `npm whoami` returns your username too — npm CLI auth is **separate** from
  pnpm's.
   If `npm whoami` fails with `ENEEDAUTH`,
   run `npm login` in a
  visible terminal before the deprecate step.
- 2FA OTP available (pnpm publish prompts for it interactively in a real
  terminal;
   pnpm deprecate does not — you must use `npm deprecate` for the
  deprecation marker).
- The package's source is in a publishable state (no secrets,
   license
  header consistent,
   README accurate).

## Procedure

### 1. Set up rolldown bundling

Most workspace packages run directly through Node or a built Node bundle
managed by mise.
 To produce a publishable Node-compatible bundle:

1. Create `rolldown.node.config.ts`.
    The minimum form re-exports the shared
   config
   (see `package/test-harness/core-diagnostic/rolldown.node.config.ts`):

   ```ts
   export { default, } from '@monochromatic-dev/config-rolldown/.node.ts';
   ```

   To bundle additional npm runtime deps so the published package has
   zero transitive deps,
    pass a custom bundle-inclusion set to
   `nodeConfig` through `nodeExternal`
   (see `package/config/oxlint/rolldown.node.config.ts`):

   ```ts
   import {
     nodeConfig,
     nodeExternal,
     type NodeFlavorConfig,
   } from '@monochromatic-dev/config-rolldown/.node.ts';

   const config: NodeFlavorConfig = nodeConfig({
     external: await nodeExternal({
       alwaysBundle: [
         '@monochromatic-dev/**',
         'nano-spawn',
         'neovim',
       ],
     },),
   },);

   export default config;
   ```

   The shared config's default `alwaysBundle` set
   inlines `@monochromatic-dev/**` and `nano-spawn`.
   Passing your own `alwaysBundle` list replaces that default,
    so re-list
   those two
   and add anything else you want self-contained.

2. Add build mise tasks to the package's `mise.toml`.
    Mirror the pattern
   in `package/test-harness/core-matrix/mise.toml`:

   ```toml
   [tasks.build]
   extends = "build"
   [tasks."build:js"]
   extends = "build:js"
   [tasks."build:js:node"]
   extends = "build:js:node"
   [tasks."watch:build"]
   extends = "watch:build"
   [tasks."watch:build:js"]
   extends = "watch:build:js"
   [tasks."watch:build:js:node"]
   extends = "watch:build:js:node"
   ```

3. Update `package.json`:

   - `"module": "dist/final/node/index.mjs"`
   - `"bin"`:
      point at the built output,
      not the source
     (`{ "<cmd>": "./dist/final/node/index.mjs" }`)
   - `"exports"`:
      built artifact for `.`,
      source path for `./ts`
   - `"files": ["dist/final", "src"]`
   - Move every dep the bundle inlines (anything you added to
     `alwaysBundle`) from `dependencies` to
     `devDependencies` (so it ends up in dev install but not in the
     published manifest)
   - Keep external runtime deps in `dependencies` if any survived
     bundling decisions

4. Confirm the source shebang is `#!/usr/bin/env node`.
    If the source
   still has a legacy `#!/usr/bin/env bun` shebang,
    change it before
   building.
    Rolldown carries a source `#!` line into the bundle
   (verified 2026-10-22 against rolldown v1.2.14),
   and the shared config injects `#!/usr/bin/env node` only into a
   bin-targeted chunk that starts without `#!`
   (`package/config/rolldown/src/bin-shebang.ts`),
   so a surviving Bun line is left untouched
   and the published bin would then require Bun.

5. `pnpm install` then `mise run //package/<cat>/<name>:build`.
    Confirm
   `dist/final/node/index.mjs` exists,
    has the right shebang,
    and lists
   only `node:*` imports when grep-ed:

   ```sh
   grep -oE 'from"[^"]*"' dist/final/node/index.mjs | sort -u
   ```

6. Smoke test:

   ```sh
   timeout 3 node dist/final/node/index.mjs </dev/null; echo "exit=$?"
   ```

   For a CLI that listens on stdin,
    EOF should produce a clean exit.

7. Commit.
    This is its own logical unit;
    the deprecation move is separate.

### 2. Move source to `package-deprecated/`

The `package-deprecated/<category>/<name>/` mirror is the deprecation
signal.
 From repo root:

```sh
rm -rf package/<cat>/<name>/node_modules package/<cat>/<name>/dist
mkdir -p package-deprecated/<cat>
git mv package/<cat>/<name> package-deprecated/<cat>/<name>
```

The `rm` is so git's rename detection isn't confused by stale node_modules
symlinks and dist artifacts.

### 3. Wire the deprecated glob into workspace and task discovery

`pnpm-workspace.yaml`:

```yaml
packages:
- 'package/*/*'
- 'package-deprecated/*/*'
```

`mise.no-env.toml#monorepo.config_roots`:

```toml
[monorepo]
config_roots = [
  "package/*/*",
  "package-deprecated/*/*",
]
```

Then regenerate `mise.toml`:

```sh
mise run file-enforcer
```

### 4. Mark the package for publish

In `package-deprecated/<cat>/<name>/package.json`:

- `"private": false`
- `"description"`:
   prefix with `DEPRECATED.` (this becomes the
  human-readable summary on the npm page)
- `"repository": { "type": "git", "url": "...", "directory": "package-deprecated/<cat>/<name>" }`
- `"publishConfig": { "access": "public" }` for scoped packages
  (`@monochromatic-dev/...` is scoped;
   npm defaults scoped packages to
  private access,
   so this is required)

### 5. Update the README

Prepend a deprecation block before any other content:

```markdown
> **Deprecated.** [One sentence on why we stopped maintaining it.] The
> published version on npm remains installable indefinitely under npm's
> no-unpublish policy, and the source stays in
> `package-deprecated/<cat>/<name>/` for reference. Use at your own risk;
> no further updates are planned.
```

Update the "Configuration" or "Installation" section to document the
`npm install` path for outside consumers,
 alongside the original
clone-the-monorepo path.

### 6. Install, rebuild, commit

```sh
pnpm install
mise run //package-deprecated/<cat>/<name>:build
```

Verify the bundle from the new location is identical in shape to the old.
Commit (single cohesive `feat(<pkg>): deprecate to package-deprecated/`
commit;
 the cleanups from step 7 below go in a follow-up).

### 7. Pre-publish dry run

```sh
cd package-deprecated/<cat>/<name>
pnpm publish --dry-run
```

Expected output:
 `📦 @scope/name@<version> → https://registry.npmjs.org/`.

If it fails with
`[ERR_PNPM_CANNOT_RESOLVE_WORKSPACE_PROTOCOL] Cannot resolve workspace protocol of dependency "@monochromatic-dev/config-typescript"`,
drop `@monochromatic-dev/config-typescript` from the package's
`devDependencies` and re-run.
 The local build still resolves it through
the root-hoisted `node_modules/@monochromatic-dev/config-typescript`.
See issue [#204](https://github.com/Aquaticat/Monochromatic/issues/204)
for the open investigation.

### 8. Publish

`pnpm publish` requires OTP and pnpm v11 does not prompt interactively in
non-TTY contexts.
 Run via a visible terminal so you can type the OTP:

```sh
terminal-exec -- bash -c '
  cd package-deprecated/<cat>/<name>
  pnpm publish
  echo "exit=$?"
  echo "Press Enter to close..."
  read
'
```

After the terminal closes,
 verify the publish landed:

```sh
curl -s https://registry.npmjs.org/$(printf '%s' "@scope/name" | python3 -c "import sys,urllib.parse; print(urllib.parse.quote(sys.stdin.read(), safe=''))") \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print('version:', list(d.get('versions', {}).keys()))"
```

Do not rely on `pnpm view`;
 it caches and can show stale data.

### 9. Deprecate on the registry

`pnpm deprecate` does **not** handle 2FA OTP — it surfaces the
`ERR_PNPM_UNAUTHORIZED: You must provide a one-time pass. Upgrade your
client to npm@latest in order to use 2FA` error without prompting.
 Use
`npm deprecate` instead.

`npm deprecate` requires npm CLI auth separately from pnpm.
 If
`npm whoami` fails,
 run `npm login` first.

`npm deprecate <pkg>` without `@<version>` is parsed as
`<pkg>@*` in modern npm and 404s.
 Always pass the explicit version.

```sh
terminal-exec -- bash -c '
  npm deprecate "@scope/name@<version>" "<deprecation message>"
  echo "exit=$?"
  echo "Press Enter to close..."
  read
'
```

The deprecation message format that worked here:

```text
No longer maintained. <one sentence on why>. Source archived at
https://github.com/Aquaticat/Monochromatic/tree/main/package-deprecated/<cat>/<name> ;
use at your own risk.
```

### 10. Verify deprecation landed

```sh
curl -s https://registry.npmjs.org/$(printf '%s' "@scope/name" | python3 -c "import sys,urllib.parse; print(urllib.parse.quote(sys.stdin.read(), safe=''))") \
  | python3 -c "import sys,json; d=json.load(sys.stdin); v=d.get('versions', {}).get('<version>', {}); print('deprecated:', repr(v.get('deprecated', '<not set>')))"
```

Expect the deprecation message string.
 If you see `<not set>`,
 the
deprecate step did not complete (most commonly because the OTP entry timed
out or the user closed the terminal early).
 Retry step 9.

## Known sharp edges

These tripped up the mcp-nvim deprecation and are likely to recur.
 Each
links to an open investigation issue.

### config-typescript not symlinked into consumer's `node_modules`

The lockfile says `@monochromatic-dev/config-typescript` is linked into
the publishable package's local `node_modules`;
 the filesystem disagrees.
`pnpm publish` then refuses to resolve `workspace:*` for that dep.

`--force` reinstall,
 `--no-frozen-lockfile`,
 full `rm -rf node_modules`
followed by fresh install — none of these create the symlink.
 Other
workspace deps in the same `devDependencies` block do symlink normally.

Workaround:
 drop `config-typescript` from the deprecated package's
`devDependencies`.
 The root-hoisted copy at
`node_modules/@monochromatic-dev/config-typescript` is reachable via
Node's upward module resolution,
 so `tsc`,
 `rolldown`,
 and the build still
work.
 The published bundle does not ship `tsconfig.json` anyway,
 so
the loss of declarative type-config provenance is local-only.

See [#204](https://github.com/Aquaticat/Monochromatic/issues/204) for the
open investigation.

### pnpm deprecate cannot prompt for OTP

pnpm v11's `deprecate` subcommand surfaces the npm registry's
"must provide one-time pass" error without offering an interactive
prompt.
 The error message tells you to "Upgrade your client to npm@latest
in order to use 2FA",
 which is misleading because the npm CLI itself
handles 2FA fine.
 Use `npm deprecate` instead.

### npm CLI auth is separate from pnpm CLI auth

`pnpm whoami` succeeds,
 `npm whoami` fails with `ENEEDAUTH`.
 The two
clients can use different `.npmrc` files or scopes.
 If you only ever
publish via pnpm,
 you may not realize npm is unauthenticated until the
deprecate step.

Run `npm login` once before step 9;
 npm then shares the token with
subsequent invocations.

### `npm deprecate <pkg>` without `@<version>` 404s in modern npm

The CLI parses `@scope/name` as `@scope/name@*`,
 then asks the registry
for that version range,
 and the registry returns 404.
 Always pass
`@scope/name@<exact-version>` to deprecate a specific version,
 or
`@scope/name@'>=0.0.0'` (quoted,
 with explicit range) to deprecate all
versions.

### Four packages had self-referential `workspace:*` deps

`config-typescript`,
 `module-es`,
 `module-hyperscript`,
 and `module-logger`
each listed themselves as workspace deps in their `package.json`.
 They
were removed in commit `4cdff544` as a side effect of unblocking the
mcp-nvim publish.
 The original intent is unknown;
 see
[#202](https://github.com/Aquaticat/Monochromatic/issues/202) for the
investigation.
 If a future deprecation reintroduces a similar publish
failure mode,
 the self-refs may need to be considered as part of the
diagnosis.

### `config-typescript` previously declared overreaching peer-deps

`@types/mdx` and `@types/bun` were peer-deps of `config-typescript`,
removed in `4cdff544`.
 The removal did not fix the publish blockage (so
they were not the cause),
 but the original intent is unknown.
 See
[#203](https://github.com/Aquaticat/Monochromatic/issues/203).

### `pnpm view <pkg>` caches; verify via direct registry calls

After publish and after deprecate,
 `pnpm view <pkg> deprecated` may
return empty even after the registry has been updated.
 The npm CLI has
the same issue.
 The registry HTTP endpoint
(`https://registry.npmjs.org/<encoded-pkg>`) is authoritative.

URL-encode the scope separator:
 `@scope/name` becomes
`@scope%2Fname` in the registry path.
 The Python one-liner in steps 8
and 10 handles this correctly.

### Bundle bigger than expected when bundling `neovim`

The `neovim` npm package depends on the full `winston` logger
(plus `logform`,
 `safe-stable-stringify`,
 `@colors/colors`,
`triple-beam`,
 `@dabh/diagnostics`,
 and others).
 Bundling neovim grew
the mcp-nvim output from 24 KB to 245 KB.
 Acceptable for a CLI;
 worth
flagging if the deprecated package is supposed to stay tiny.

## Verification checklist before declaring done

- [ ] `dist/final/node/index.mjs` exists at the new location with the
      correct shebang
- [ ] `grep -oE 'from"[^"]*"' dist/final/node/index.mjs | sort -u` shows
      only `node:*` imports (or expected external deps)
- [ ] `pnpm publish --dry-run` succeeds from the package directory
- [ ] Registry HTTP returns the published version
- [ ] Registry HTTP returns the deprecation message on the published
      version
- [ ] README's DEPRECATED block is present and correct
- [ ] `mise run //package-deprecated/<cat>/<name>:build` still works
      from the repo root (regression check on the move + glob updates)
- [ ] Any other workspace packages that depended on this package via
      `workspace:*` were updated or accept the deprecation
