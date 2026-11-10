# catalog-tighten.matrix at HEAD cannot start a scenario on a store-linked checkout: node cannot resolve the entrypoint's imports, and corepack rejects the pnpm-shadow banner as a version spec

Two separate causes stop
 `mise run //package/dependency-catalog/cli-tighten.matrix:test:matrix`
on this repository as of 2026-11-10
 (`1ee1d03af7` plus the branch that fixed the entrypoint path).
Each is measured below;
either one alone fails the run.

## Symptom

The full matrix fails every scenario.
With the entrypoint path corrected (GitHub #1303 and its duplicate #5911),
the container now reaches `in-container.ts` and dies loading its imports:

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'nano-spawn' imported from /repo/package/dependency-catalog/cli-tighten.matrix/src/in-container.ts
```

The argv the orchestrator printed for that run also carries a two-line
`FIXTURE_PNPM` value:

```text
--env 'FIXTURE_PNPM=pnpm@12.18.0
pnpm-shadow 0.1.3-alpha.1, shadowing pnpm at /var/mnt/encrypted/repository/Monochromatic/.cache/toolchain-bin/pnpm'
```

Handing that same value to corepack fails before any install starts:

```text
Unknown Syntax Error: Command not found; did you mean one of:

  0. corepack -h
  1. corepack -v
  2. corepack cache clean
  3. corepack disable [--install-directory #0] ...
```

## Root cause

### The shared global virtual store puts every dependency outside the repo

`pnpm-workspace.yaml` selects a home store and a global virtual store:

```yaml
storeDir: ${HOME}/.local/share/pnpm/store
```

(`pnpm-workspace.yaml:575`),
and later:

```yaml
virtualStoreType: global
```

(`pnpm-workspace.yaml:589`).

With that layout,
`node_modules/<pkg>` entries are relative symlinks out of the
repository.
Measured on this worktree:

```text
$ readlink node_modules/nano-spawn
../../../../../../../../home/user/.local/share/pnpm/store/v11/links/@/nano-spawn/2.1.0/fa541bc3676f8c6b600aff5dad14fbd08091f957042cd4ace834175d098073a6/node_modules/nano-spawn
```

The container mounts the monorepo read-only at `/repo` and nothing else
(`buildPodmanArgs`,
`package/dependency-catalog/cli-tighten.matrix/src/matrix.unit.matrix.test.ts`),
so `/repo/node_modules/nano-spawn` resolves through that relative target to
`/home/user/.local/share/pnpm/store/...` inside the container,
where no such
path exists.
The same holds for every other dependency,
the tool's `yaml` reader included,
because package-level `node_modules` directories hold only workspace links.

### The shadowed pnpm answers `--version` with two lines

`resolvePnpmSpec` takes the whole trimmed stdout of `pnpm --version`:

```ts
  const version = (await spawn(
    'pnpm',
    ['--version',],
    { cwd: HOST_REPO_ROOT, },
  )).stdout
    .trim();
```

(`package/dependency-catalog/cli-tighten.matrix/src/matrix.unit.matrix.test.ts:87-92`).

Under mise,
`pnpm` on PATH is this repository's `pnpm-shadow`,
which prints its
own notice after pnpm's version on purpose:

```rust
    return format!("pnpm-shadow {}, shadowing pnpm at {}\n", env!("CARGO_PKG_VERSION"), real_pnpm.display());
```

(`package/pnpm-shadow/cli/src/shadow_notice.rs:113`,
documented as deliberate at `package/pnpm-shadow/cli/src/shadow_notice.rs:101-105`,
"in the shape `hub` and `mpm-shadow` use").

Measured on the host:

```text
$ mise exec -- pnpm --version
12.18.0
pnpm-shadow 0.1.3-alpha.1, shadowing pnpm at /var/mnt/encrypted/repository/Monochromatic/.cache/toolchain-bin/pnpm
```

`buildRootPackageJson` writes that value into the fixture's `packageManager`
(`package/dependency-catalog/cli-tighten.matrix/src/combos.ts:370-383`),
and
`installFixture` passes it to corepack as the command word,
which corepack
rejects as shown in the symptom.

Earlier reading:
 the first diagnosis of the red matrix was the stale
`/repo/packages/...` entrypoint alone.
Fixing that path moved the failure to `ERR_MODULE_NOT_FOUND`,
which disproved
"one path constant is the whole break".

## Verification

Checked on 2026-11-10 against `1ee1d03af7` plus the branch
`fix/catalog-tighten-matrix-ready-bugs`.

Failing arm,
the orchestrator as it is:

```sh
mise run //package/dependency-catalog/cli-tighten.matrix:test:matrix
```

Every scenario ends in `Cannot find package 'nano-spawn' imported from /repo/...`.

Passing arm,
one scenario of the same fixture,
with the store mounted at the
path its symlink resolves to and a clean pnpm spec passed in:

```sh
podman run --rm --memory 2g --cpus 2 --pids-limit 512 --read-only \
  --security-opt=label=disable \
  --tmpfs '/work:rw,exec,size=512m' --tmpfs '/tmp:rw,exec,size=1g' \
  --env HOME=/tmp --env FIXTURE_PNPM=pnpm@12.18.0 \
  --volume "$PWD:/repo:ro" \
  --volume "$HOME/.local/share/pnpm/store:$HOME/.local/share/pnpm/store:ro" \
  --workdir /work docker.io/library/node:24-slim \
  node /repo/package/dependency-catalog/cli-tighten.matrix/src/in-container.ts \
  '{"label":"storeDir relocated","nodeLinker":"isolated","hoist":false,"extraSettings":["storeDir: /tmp/ct-removable-store"],"mutation":"none","expect":"tighten"}'
```

```text
[storeDir relocated] PASS (tighten)
```

Corepack arm,
the polluted spec alone:

```sh
podman run --rm docker.io/library/node:24-slim corepack \
  $'pnpm@12.18.0\npnpm-shadow 0.1.3-alpha.1, shadowing pnpm at /opt/pnpm' --version
```

```text
Unknown Syntax Error: Command not found; did you mean one of: ...
```

`corepack pnpm@12.18.0 --version` from the same image prints a version,
so the
second line is what corepack cannot read.

## Verified workarounds

### Mount the store read-only

Add the resolved store path to the `podman run` mounts,
next to the `/repo`
mount:

```text
--volume "${HOME}/.local/share/pnpm/store:${HOME}/.local/share/pnpm/store:ro"
```

Measured as the passing arm above.
Tradeoffs:
 the container gains read access to every package in the shared store,
not only
this workspace's,
and the argument has to follow `storeDir` wherever it moves
(`pnpm store path` reports the current value).
A checkout whose `node_modules` keeps its own `.pnpm` tree does not need it.

### Take pnpm's own version line only

Read the first line of `pnpm --version` instead of the whole stdout,
for
example `stdout.split('\n',)[0]`,
or take the pnpm version from
`toolchain.lock.json`.
Tradeoff:
 it relies on the wrapper printing its notice after pnpm's version,
which
`pnpm-shadow` does;
a wrapper that printed before would need a stricter match
than "first line".

Both changes touch how the matrix mounts host paths and how it derives a spec,
so they are the owner's call.

## What does not work

- Leaving the mounts as they are:
  measured `ERR_MODULE_NOT_FOUND` above.
- Passing `pnpm --version` stdout through unchanged:
  measured corepack
  `Unknown Syntax Error` above.
- Expecting `node` to find a `node_modules/.pnpm` tree inside the mount:
  this
  checkout has none,
  because `virtualStoreType: global` moves it under
  `storeDir`.

## Upstream filing decision

Nothing to file upstream.
Cause one is this repository's own `pnpm-workspace.yaml` choice.
Cause two is this repository's own `pnpm-shadow` output shape plus one consumer
parse of it in this repository.
Upstream `pnpm --version` and corepack both behave as documented,
so
`.out-of-scope/` and the six-constraint filing check do not apply.
